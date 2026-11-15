"""Own Chrome through nodriver; resolve the live page again after login redirects."""
import asyncio
import base64
import json
import logging
import os
import tempfile
import urllib.request
from pathlib import Path
from contextlib import asynccontextmanager
from urllib.parse import urlsplit

import httpx
from filelock import FileLock
from websockets.asyncio.client import connect

from .config import BrowserSettings


class NodriverBackend:
    def __init__(self, settings: BrowserSettings, profile_dir=None):
        self.settings = settings
        self.browser = None
        self.process = None
        self.profile = None
        self.origin = None
        self.endpoint = None
        self.profile_dir = Path(profile_dir) if profile_dir else None
        self.profile_lock = None

    async def start(self):
        import nodriver as nd
        from nodriver.core.util import free_port

        if self.profile_dir:
            self.profile_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
            self.profile_dir.chmod(0o700)
            self.profile_lock = FileLock(str(self.profile_dir) + ".lock")
            self.profile_lock.acquire(timeout=0)
            profile_path = str(self.profile_dir)
        else:
            self.profile = tempfile.TemporaryDirectory(prefix="xhs-nodriver-")
            profile_path = self.profile.name
        args = ["--window-size=1440,1000"]
        if self.settings.proxy:
            args.append(f"--proxy-server={self.settings.proxy}")
        config = nd.Config(
            user_data_dir=profile_path, headless=self.settings.headless,
            browser_executable_path=self.settings.executable, browser_args=args,
            lang="zh-CN", sandbox=os.getenv("XHS_BROWSER_SANDBOX", "1") != "0",
        )
        config.host, config.port = "127.0.0.1", free_port()
        # Preserve site isolation; nodriver 0.47 adds this flag unconditionally.
        launch_args = [a for a in config() if a != "--disable-features=IsolateOrigins,site-per-process"]
        self.endpoint = f"http://127.0.0.1:{config.port}"
        try:
            self.process = await asyncio.create_subprocess_exec(
                config.browser_executable_path, *launch_args,
                stdin=asyncio.subprocess.DEVNULL, stdout=asyncio.subprocess.DEVNULL,
                stderr=asyncio.subprocess.DEVNULL,
            )
            opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))

            def ready():
                try:
                    with opener.open(self.endpoint + "/json/version", timeout=.3) as response:
                        return bool(json.load(response).get("webSocketDebuggerUrl"))
                except (OSError, ValueError):
                    return False

            async with asyncio.timeout(self.settings.timeout):
                while not await asyncio.to_thread(ready):
                    if self.process.returncode is not None:
                        raise RuntimeError("Chrome exited before DevTools became ready")
                    await asyncio.sleep(.1)
                # nodriver discovers targets and owns the browser connection.
                logging.getLogger("nodriver.core.browser").setLevel(logging.WARNING)
                self.browser = nd.Browser(config)
                await self.browser.start()
                self.browser._process = self.process
                self.browser._process_pid = self.process.pid
        except BaseException:
            await self.close()
            raise

    async def target(self, browser=False):
        async with httpx.AsyncClient(trust_env=False, timeout=3) as client:
            if browser:
                return (await client.get(self.endpoint + "/json/version")).json()
            targets = (await client.get(self.endpoint + "/json/list")).json()
        pages = [t for t in targets if t["type"] == "page"]
        if self.origin:
            pages = [t for t in pages if t["url"].startswith(self.origin + "/")]
        if not pages:
            raise RuntimeError("The browser page was closed or navigated away")
        return pages[0]

    @asynccontextmanager
    async def connection(self, browser=False):
        target = await self.target(browser)
        async with asyncio.timeout(self.settings.timeout):
            async with connect(target["webSocketDebuggerUrl"], proxy=None, max_size=32*1024*1024) as ws:
                request_id = 0
                async def send(method, params=None):
                    nonlocal request_id
                    request_id += 1
                    await ws.send(json.dumps({"id": request_id, "method": method, "params": params or {}}))
                    while True:
                        response = json.loads(await ws.recv())
                        if response.get("id") == request_id:
                            if "error" in response:
                                raise RuntimeError(f"CDP command failed: {method}")
                            return response.get("result", {})
                yield send

    async def command(self, method, params=None, browser=False):
        async with self.connection(browser) as send:
            return await send(method, params)

    async def navigate(self, url):
        result = await self.command("Page.navigate", {"url": url})
        if result.get("errorText"):
            raise RuntimeError("Browser navigation failed")
        parts = urlsplit(url)
        self.origin = f"{parts.scheme}://{parts.netloc}" if parts.netloc else None
        async with asyncio.timeout(self.settings.timeout):
            while True:
                try:
                    if await self.evaluate("document.readyState") in {"interactive", "complete"}:
                        return
                except RuntimeError:
                    # A document's execution context is destroyed during navigation.
                    # Retry only this read, never uploads or save clicks.
                    pass
                await asyncio.sleep(.2)

    async def cookies(self, url):
        values = (await self.command("Network.getCookies", {"urls": [url]}))["cookies"]
        result = {}
        for cookie in sorted(values, key=lambda c: len(c["path"])):
            result[cookie["name"]] = cookie["value"]
        return result

    async def export_cookies(self):
        cookies = (await self.command("Storage.getCookies", browser=True))["cookies"]
        return [c for c in cookies if c["domain"].lstrip(".") == "xiaohongshu.com" or c["domain"].endswith(".xiaohongshu.com")]

    async def restore_cookies(self, cookies):
        fields = {"name", "value", "domain", "path", "secure", "httpOnly", "sameSite", "expires"}
        params = [{k: v for k, v in c.items() if k in fields and not (k == "expires" and v <= 0)} for c in cookies]
        await self.command("Storage.setCookies", {"cookies": params}, browser=True)

    async def set_cookies(self, cookies):
        await self.restore_cookies([
            {"name": k, "value": v, "domain": ".xiaohongshu.com", "path": "/", "secure": True}
            for k, v in cookies.items()
        ])

    async def evaluate(self, expression, await_promise=False):
        result = await self.command("Runtime.evaluate", {
            "expression": expression, "awaitPromise": await_promise, "returnByValue": True,
        })
        if result.get("exceptionDetails"):
            raise RuntimeError("Page script execution failed")
        return result["result"].get("value")

    async def screenshot(self, path):
        result = await self.command("Page.captureScreenshot", {"format": "png"})
        path = Path(path)
        path.write_bytes(base64.b64decode(result["data"]))
        path.chmod(0o600)

    async def _query(self, send, selector):
        root = (await send("DOM.getDocument"))["root"]["nodeId"]
        node_id = (await send("DOM.querySelector", {"nodeId": root, "selector": selector}))["nodeId"]
        if not node_id:
            raise RuntimeError("Expected page element not found")
        return node_id

    async def fill(self, selector, text):
        async with self.connection() as send:
            node_id = await self._query(send, selector)
            await send("DOM.focus", {"nodeId": node_id})
            await send("Input.insertText", {"text": text})

    async def upload(self, selector, paths):
        async with self.connection() as send:
            node_id = await self._query(send, selector)
            await send("DOM.setFileInputFiles", {"nodeId": node_id, "files": paths})

    async def click(self, selector):
        async with self.connection() as send:
            node_id = await self._query(send, selector)
            await self._click_node(send, {"nodeId": node_id})

    async def _click_node(self, send, node):
        await send("DOM.scrollIntoViewIfNeeded", node)
        model = (await send("DOM.getBoxModel", node))["model"]["content"]
        x, y = sum(model[::2])/4, sum(model[1::2])/4
        for kind in ["mousePressed", "mouseReleased"]:
            await send("Input.dispatchMouseEvent", {"type": kind, "x": x, "y": y, "button": "left", "clickCount": 1})

    async def click_draft_save(self):
        # The live creator site puts its footer inside a CLOSED shadow root.
        # Match an exact button label; never select the adjacent publish button.
        async with self.connection() as send:
            tree = (await send("DOM.getDocument", {"depth": -1, "pierce": True}))["root"]
            matches = []
            def walk(node, parent=None):
                if node.get("nodeValue", "").strip() == "暂存离开" and parent and parent["nodeName"] == "BUTTON":
                    matches.append(parent["backendNodeId"])
                for child in node.get("children", []) + node.get("shadowRoots", []):
                    walk(child, node)
            walk(tree)
            if len(matches) != 1:
                raise RuntimeError("Could not uniquely locate the draft-save button")
            await self._click_node(send, {"backendNodeId": matches[0]})

    async def close(self):
        if self.process and self.process.returncode is None:
            try:
                # Let Chrome flush IndexedDB/local drafts before terminating it.
                await asyncio.wait_for(self.command("Browser.close", browser=True), 3)
            except Exception:
                pass
            try:
                await asyncio.wait_for(self.process.wait(), 3)
            except asyncio.TimeoutError:
                pass
        if self.browser and self.browser.connection:
            try:
                await asyncio.wait_for(self.browser.connection.disconnect(), 2)
            except Exception:
                pass
        if self.process and self.process.returncode is None:
            try:
                self.process.terminate()
                await asyncio.wait_for(self.process.wait(), 3)
            except asyncio.TimeoutError:
                self.process.kill()
                await self.process.wait()
            except ProcessLookupError:
                pass
        self.browser = self.process = None
        if self.profile:
            self.profile.cleanup()
            self.profile = None
        if self.profile_lock:
            self.profile_lock.release()
            self.profile_lock = None


def create_backend(settings):
    return NodriverBackend(settings)
