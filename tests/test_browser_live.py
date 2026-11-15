"""Real local Chrome tests. XHS_LIVE_BROWSER_TEST=1 enables them; no XHS traffic."""
import asyncio
import os
import tempfile
import time
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from scripts.xhs_browser.backends import NodriverBackend
from scripts.xhs_browser.config import BrowserSettings

HTML = b'''<!doctype html><input id="title"><input type="file" id="upload"><div id="host"></div>
<script>const root=document.querySelector('#host').attachShadow({mode:'closed'});
root.innerHTML='<button onclick="document.body.dataset.saved=1">\\u6682\\u5b58\\u79bb\\u5f00</button>';</script>'''

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-Type', 'text/html; charset=utf-8')
        self.end_headers()
        self.wfile.write(HTML)
    def log_message(self, *args):
        pass

@unittest.skipUnless(os.getenv('XHS_LIVE_BROWSER_TEST') == '1', 'set XHS_LIVE_BROWSER_TEST=1')
class LiveBrowserTests(unittest.IsolatedAsyncioTestCase):
    async def test_upload_chinese_input_closed_shadow_and_persistent_profile(self):
        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        url = f'http://127.0.0.1:{server.server_port}/'
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            upload = root / 'test.txt'
            upload.write_text('fixture')
            settings = BrowserSettings(headless=True, timeout=20)
            browser = NodriverBackend(settings, profile_dir=root / 'profile')
            try:
                await browser.start()
                await browser.navigate(url)
                self.assertIs(await browser.evaluate('false'), False)
                await browser.fill('#title', '中文输入验证')
                self.assertEqual(await browser.evaluate("document.querySelector('#title').value"), '中文输入验证')
                await browser.upload('#upload', [str(upload)])
                self.assertEqual(await browser.evaluate("document.querySelector('#upload').files[0].name"), 'test.txt')
                await browser.click_draft_save()
                self.assertEqual(await browser.evaluate('document.body.dataset.saved'), '1')
                await browser.evaluate("localStorage.setItem('draft','preserved')")
                await browser.restore_cookies([{'name':'fixture','value':'persisted','domain':'127.0.0.1','path':'/','expires':time.time()+3600}])
                self.assertEqual((await browser.cookies(url))['fixture'], 'persisted')
                await browser.close()
                browser = NodriverBackend(settings, profile_dir=root / 'profile')
                await browser.start()
                await browser.navigate(url)
                self.assertEqual(await browser.evaluate("localStorage.getItem('draft')"), 'preserved')
                self.assertEqual((await browser.cookies(url))['fixture'], 'persisted')
            finally:
                await browser.close()
                server.shutdown()
                server.server_close()

if __name__ == '__main__':
    unittest.main()
