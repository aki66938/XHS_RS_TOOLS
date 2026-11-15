"""Own login browsers; verify identity before marking a session authenticated."""
import asyncio
import json
import os
import time
import uuid
from datetime import datetime, timezone

from .backends import NodriverBackend
from .config import TARGET_URLS, HOME_URLS, PROJECT_ROOT, validate_target

IDENTITY_URLS = {
    "creator": "https://creator.xiaohongshu.com/api/galaxy/user/info",
    "explore": "https://edith.xiaohongshu.com/api/sns/web/v2/user/me",
}


def verified_identity(target, response):
    if not isinstance(response, dict) or response.get("http_status") != 200:
        return None
    body = response.get("body")
    if not isinstance(body, dict) or body.get("code") != 0 or body.get("success") is False:
        return None
    data = body.get("data")
    if not isinstance(data, dict) or data.get("guest") is True:
        return None
    user_id = data.get("user_id") or data.get("userId")
    return str(user_id) if user_id else None


def save_credentials(target, user_id, cookies):
    filename = "cookie-creator.json" if target == "creator" else "cookie.json"
    destination = PROJECT_ROOT / filename
    now = datetime.now(timezone.utc).isoformat()
    document = {
        "user_id": user_id, "cookies": cookies, "x_s_common": None,
        "created_at": now, "updated_at": now, "is_valid": True,
    }
    temporary = destination.with_suffix(".json.tmp")
    fd = os.open(temporary, os.O_CREAT | os.O_WRONLY | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as output:
        json.dump(document, output, ensure_ascii=False, indent=2)
    temporary.replace(destination)
    destination.chmod(0o600)
    return filename


class LoginSessions:
    def __init__(self, settings):
        self.settings = settings
        self.sessions = {}
        self.lock = asyncio.Lock()
        self.reaper = None

    async def start(self, target):
        validate_target(target)
        async with self.lock:
            for sid, session in self.sessions.items():
                if session["target"] == target:
                    return {"success": True, "session_id": sid, "state": session["state"]}
            profile_dir = self.settings.data_dir / "profiles" / target
            had_profile = (profile_dir / "Default").exists()
            backend = NodriverBackend(self.settings, profile_dir=profile_dir)
            try:
                async with asyncio.timeout(self.settings.timeout):
                    await backend.start()
                    state_file = self.settings.data_dir / "browser-state" / f"{target}.json"
                    if state_file.exists() and not had_profile:
                        await backend.restore_cookies(json.loads(state_file.read_text()))
                    await backend.navigate(HOME_URLS[target] if had_profile or state_file.exists() else TARGET_URLS[target])
                sid = uuid.uuid4().hex
                self.sessions[sid] = {
                    "backend": backend, "target": target, "state": "waiting_for_login",
                    "last_used": time.monotonic(),
                }
                if self.reaper is None:
                    self.reaper = asyncio.create_task(self.reap_idle())
                return {"success": True, "session_id": sid, "state": "waiting_for_login", "target": target}
            except BaseException:
                await backend.close()
                raise

    async def status(self, sid):
        async with self.lock:
            session = self.sessions.get(sid)
            if session is None:
                return {"success": False, "state": "not_found"}
            session["last_used"] = time.monotonic()
            backend, target = session["backend"], session["target"]
            try:
                expression = """(async () => {
                    try {
                        const response = await fetch(URL, {credentials:'include', signal:AbortSignal.timeout(8000)});
                        return {http_status:response.status, body:await response.json()};
                    } catch (_) { return {http_status:0}; }
                })()""".replace("URL", json.dumps(IDENTITY_URLS[target]))
                async with asyncio.timeout(12):
                    response = await backend.evaluate(expression, await_promise=True)
                    user_id = verified_identity(target, response)
                    if user_id:
                        cookies = await backend.cookies(HOME_URLS[target])
                        filename = save_credentials(target, user_id, cookies)
                        folder = self.settings.data_dir / "browser-state"
                        folder.mkdir(parents=True, exist_ok=True, mode=0o700)
                        folder.chmod(0o700)
                        state_file = folder / f"{target}.json"
                        state_file.write_text(json.dumps(await backend.export_cookies()))
                        state_file.chmod(0o600)
                        session["state"] = "authenticated"
                        return {"success": True, "session_id": sid, "state": "authenticated", "credentials_file": filename}
                    session["state"] = "unverified"
                    return {"success": True, "session_id": sid, "state": "unverified", "message": "Complete login in Chrome; identity endpoint has not confirmed a user"}
            except Exception as exc:
                session["state"] = "browser_error"
                return {"success": False, "session_id": sid, "state": "browser_error", "error": type(exc).__name__}

    async def view(self, sid):
        async with self.lock:
            session = self.sessions.get(sid)
            if session is None:
                return {"success": False, "state": "not_found"}
            session["last_used"] = time.monotonic()
            backend = session["backend"]
            folder = self.settings.data_dir / "screenshots"
            folder.mkdir(parents=True, exist_ok=True)
            screenshot = folder / f"{sid}.png"
            await asyncio.wait_for(backend.screenshot(screenshot), self.settings.timeout)
            view = await backend.evaluate("""JSON.stringify({
                url: location.origin + location.pathname,
                title: document.title,
                text: document.body.innerText.slice(0,8000),
                inputs: Array.from(document.querySelectorAll('input,textarea,[contenteditable="true"]')).map(e => ({
                    tag:e.tagName,type:e.type,placeholder:e.getAttribute('placeholder'),class:e.className,
                    accept:e.accept,multiple:e.multiple
                })),
                buttons: Array.from(document.querySelectorAll('button')).map(e=>({text:e.innerText,class:e.className}))
            })""")
            return {"success": True, "session_id": sid, "screenshot": str(screenshot), "page": json.loads(view)}

    async def editor_open(self, backend):
        return bool(await backend.evaluate('Boolean(document.querySelector(\'div.tiptap[contenteditable="true"]\'))'))

    async def close(self, sid, force=False):
        async with self.lock:
            session = self.sessions.get(sid)
            if session and not force and await self.editor_open(session["backend"]):
                return {"success": False, "state": "editor_busy", "message": "Save the open editor before closing Chrome"}
            session = self.sessions.pop(sid, None)
            if session:
                await session["backend"].close()
            return {"success": True, "state": "closed"}

    async def reap_idle(self):
        while True:
            await asyncio.sleep(30)
            async with self.lock:
                for sid, session in list(self.sessions.items()):
                    if time.monotonic() - session["last_used"] > 600:
                        try:
                            if await self.editor_open(session["backend"]):
                                # Opening a local draft may remove its list entry.
                                # Never discard a human's in-progress editor on idle.
                                continue
                        except Exception:
                            # Unknown page state is not permission to discard content.
                            continue
                        self.sessions.pop(sid)
                        await session["backend"].close()

    async def close_all(self):
        if self.reaper:
            self.reaper.cancel()
            try:
                await self.reaper
            except asyncio.CancelledError:
                pass
            self.reaper = None
        for sid in list(self.sessions):
            await self.close(sid, force=True)
