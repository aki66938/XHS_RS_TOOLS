import asyncio
from contextlib import asynccontextmanager
from .backends import create_backend
from .config import BrowserSettings, TARGET_URLS, HOME_URLS, validate_target


class CookieValidationError(RuntimeError):
    pass


def validate_cookies(cookies, target, authenticated=False):
    validate_target(target)
    # Creator cookies have a different schema: don't require webId/gid there.
    required = {"a1"} if target == "explore" else {"a1", "xsecappid"}
    if authenticated and target == "explore":
        required.add("web_session")
    missing = sorted(k for k in required if not cookies.get(k))
    if missing:
        raise CookieValidationError("Missing required cookies: " + ", ".join(missing))


class BrowserService:
    def __init__(self, settings=None, factory=create_backend):
        self.settings = settings or BrowserSettings.from_env()
        self.factory = factory
        self.lock = asyncio.Lock()

    @asynccontextmanager
    async def operation(self):
        # Avoid competing Chrome instances and blocking the FastAPI event loop.
        async with self.lock:
            backend = self.factory(self.settings)
            try:
                await backend.start()
                yield backend
            finally:
                await backend.close()

    async def collect(self, target="explore", cookies=None):
        validate_target(target)
        if cookies is not None and not cookies:
            raise ValueError("Missing cookies to synchronize")
        async with self.operation() as backend:
            if cookies:
                await backend.set_cookies(cookies)
            url = HOME_URLS[target] if cookies else TARGET_URLS[target]
            async with asyncio.timeout(self.settings.timeout):
                await backend.navigate(url)
                while True:
                    result = await backend.cookies(url)
                    try:
                        validate_cookies(result, target, authenticated=bool(cookies))
                        if cookies and "web_session" in cookies and result.get("web_session") != cookies["web_session"]:
                            raise CookieValidationError("Login session changed during synchronization")
                        return result
                    except CookieValidationError:
                        await asyncio.sleep(.25)
