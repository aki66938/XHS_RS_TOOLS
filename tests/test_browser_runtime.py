import asyncio
import os
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch, AsyncMock

from fastapi.testclient import TestClient
from scripts.agent_server import app
from scripts.xhs_browser.config import BrowserSettings
from scripts.xhs_browser.service import BrowserService, validate_cookies, CookieValidationError
from scripts.xhs_browser.sessions import verified_identity, LoginSessions
from scripts.xhs_browser.drafts import DraftWorkflows, validate_request


class ValidationTests(unittest.TestCase):
    def test_cookie_presence_does_not_confirm_identity(self):
        self.assertIsNone(verified_identity('creator', {'http_status': 200, 'body': {'code': 0, 'data': {}}}))
        self.assertIsNone(verified_identity('explore', {'http_status': 200, 'body': {'code': 0, 'data': {'user_id': 'guest', 'guest': True}}}))
        self.assertIsNone(verified_identity('creator', {'http_status': 403, 'body': {'code': 0, 'data': {'userId': 'id'}}}))
        self.assertEqual(verified_identity('creator', {'http_status': 200, 'body': {'code': 0, 'data': {'userId': 'id'}}}), 'id')

    def test_missing_required_cookies_fails(self):
        with self.assertRaises(CookieValidationError):
            validate_cookies({}, 'explore')
        validate_cookies({'a1': 'seed'}, 'explore')
        with self.assertRaises(CookieValidationError):
            validate_cookies({'a1': 'seed'}, 'explore', authenticated=True)

    def test_no_legacy_backend(self):
        with patch.dict(os.environ, {'XHS_BROWSER_BACKEND': 'uc'}):
            with self.assertRaises(ValueError):
                BrowserSettings.from_env()

    def test_http_contract(self):
        with TestClient(app) as client:
            self.assertEqual(client.get('/health').json()['backend'], 'nodriver')
            self.assertEqual(client.get('/guest-cookies?target=unknown').status_code, 422)
            self.assertFalse(client.post('/sync-login-cookies', json={}).json()['success'])
            self.assertEqual(client.get('/browser/login/absent').json()['state'], 'not_found')
            self.assertEqual(client.get('/browser/drafts/invalid!').status_code, 422)


class CollectionTests(unittest.IsolatedAsyncioTestCase):
    async def test_failed_navigation_always_closes_browser(self):
        backend = SimpleNamespace(start=AsyncMock(), close=AsyncMock(), navigate=AsyncMock(side_effect=RuntimeError('navigation failed')))
        service = BrowserService(factory=lambda _: backend)
        with self.assertRaises(RuntimeError):
            await service.collect()
        backend.close.assert_awaited_once()

    async def test_sync_injects_input_before_navigation(self):
        backend = SimpleNamespace(start=AsyncMock(), close=AsyncMock(), navigate=AsyncMock(), set_cookies=AsyncMock(), cookies=AsyncMock(return_value={'a1': 'seed', 'web_session': 'session'}))
        service = BrowserService(factory=lambda _: backend)
        result = await service.collect(cookies={'a1': 'seed', 'web_session': 'session'})
        self.assertEqual(result['web_session'], 'session')
        backend.set_cookies.assert_awaited_once_with({'a1': 'seed', 'web_session': 'session'})
        backend.close.assert_awaited_once()


class SessionProtectionTests(unittest.IsolatedAsyncioTestCase):
    async def test_explicit_close_refuses_to_discard_open_editor(self):
        backend = SimpleNamespace(evaluate=AsyncMock(return_value=True), close=AsyncMock())
        sessions = LoginSessions(BrowserSettings())
        sessions.sessions['session'] = {'backend': backend}
        self.assertEqual((await sessions.close('session'))['state'], 'editor_busy')
        backend.close.assert_not_awaited()
        self.assertIn('session', sessions.sessions)
        await sessions.close_all()
        backend.close.assert_awaited_once()


class FakeBrowser:
    def __init__(self):
        self.uploads = 0
        self.saves = 0
        self.fail_save = False
        self.image_tab_selected = False

    async def evaluate(self, expression):
        if (expression.startswith('Boolean(') and '图片编辑' in expression) or 'feature-guide__btn' in expression:
            return False
        if "草稿存储于当前使用的浏览器本地" in expression and "some(e" in expression:
            return self.image_tab_selected
        return True

    async def navigate(self, url):
        self.url = url
        self.image_tab_selected = False

    async def click(self, selector):
        if 'image-drafts' in selector:
            self.image_tab_selected = True

    async def upload(self, selector, paths):
        self.uploads += 1

    async def fill(self, selector, text):
        pass

    async def screenshot(self, path):
        Path(path).write_bytes(b'fake screenshot')

    async def click_draft_save(self):
        self.saves += 1
        self.image_tab_selected = True
        if self.fail_save:
            raise TimeoutError('save response unknown')


class DraftTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.image = self.root / 'test.png'
        self.image.write_bytes(b'test image')
        self.env = patch.dict(os.environ, {'XHS_UPLOAD_DIR': str(self.root)})
        self.env.start()
        self.browser = FakeBrowser()
        sessions = SimpleNamespace(
            settings=BrowserSettings(data_dir=self.root / 'state'),
            lock=asyncio.Lock(), sessions={'session': {'backend': self.browser, 'target': 'creator'}},
            status=AsyncMock(return_value={'state': 'authenticated'}),
        )
        self.workflows = DraftWorkflows(sessions)

    async def asyncTearDown(self):
        self.env.stop()
        self.temp.cleanup()

    async def create(self, **kwargs):
        return await self.workflows.create('session', 'task', kwargs.get('title', '测试标题'), '测试正文', [str(self.image)])

    async def test_same_request_does_not_duplicate_upload_or_save(self):
        self.assertEqual((await self.create())['state'], 'saved')
        repeated = await self.create()
        self.assertTrue(repeated['reused'])
        self.assertEqual((self.browser.uploads, self.browser.saves), (1, 1))

    async def test_timeout_after_save_is_uncertain_and_never_replayed(self):
        self.browser.fail_save = True
        self.assertEqual((await self.create())['state'], 'uncertain')
        self.assertEqual((await self.create())['state'], 'uncertain')
        self.assertEqual(self.browser.saves, 1)

    async def test_verify_selects_image_category_after_drawer_defaults_to_video(self):
        await self.create()
        result = await self.workflows.verify('session', 'task')
        self.assertEqual(result['state'], 'saved')
        self.assertTrue(self.browser.image_tab_selected)

    async def test_changed_content_with_same_key_is_rejected(self):
        await self.create()
        self.assertEqual((await self.create(title='不同的标题'))['state'], 'idempotency_conflict')
        self.assertEqual(self.browser.uploads, 1)

    async def test_task_survives_service_recreation(self):
        await self.create()
        recovered = DraftWorkflows(self.workflows.sessions)
        self.assertEqual(recovered.get('task')['state'], 'saved')

    async def test_interrupted_save_becomes_uncertain_after_restart(self):
        self.workflows.write({'request_id': 'task', 'success': False}, 'saving')
        recovered = DraftWorkflows(self.workflows.sessions)
        self.assertEqual(recovered.get('task')['state'], 'uncertain')
        self.assertEqual(recovered.get('task')['interrupted_step'], 'saving')

    async def test_upload_cannot_follow_symlink_outside_root(self):
        folder = self.root / 'uploads'
        folder.mkdir()
        link = folder / 'outside.png'
        link.symlink_to(self.image)
        with patch.dict(os.environ, {'XHS_UPLOAD_DIR': str(folder)}):
            with self.assertRaises(ValueError):
                validate_request('标题', '正文', [str(link)])

    async def test_invalid_task_id_cannot_escape_task_directory(self):
        with self.assertRaises(ValueError):
            self.workflows.get('../../cookie')


if __name__ == '__main__':
    unittest.main()
