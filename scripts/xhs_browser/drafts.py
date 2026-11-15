"""Deterministic image-note drafts. A saved draft is local to its Chrome profile."""
import asyncio
import hashlib
import json
import os
import re
from pathlib import Path
from datetime import datetime, timezone

from .config import PROJECT_ROOT

EDITOR_URL = "https://creator.xiaohongshu.com/publish/publish?target=image"
TITLE = 'input[placeholder="填写标题会有更多赞哦"]'
BODY = 'div.tiptap[contenteditable="true"]'
UPLOAD = 'input[type="file"][multiple]'
DRAFT_LIST_TEXT = "草稿存储于当前使用的浏览器本地"


def validate_request(title, body, images):
    if not title.strip() or len(title) > 20:
        raise ValueError("Title must contain 1–20 characters")
    if not body.strip() or len(body) > 1000:
        raise ValueError("Body must contain 1–1000 characters")
    if not 1 <= len(images) <= 18:
        raise ValueError("Provide 1–18 images")
    root = Path(os.getenv("XHS_UPLOAD_DIR", str(PROJECT_ROOT / "uploads"))).resolve()
    files = []
    for image in images:
        path = Path(image).resolve(strict=True)
        if not path.is_relative_to(root) or not path.is_file():
            raise ValueError("Images must be files inside XHS_UPLOAD_DIR")
        if path.suffix.lower() not in {".jpg", ".jpeg", ".png", ".webp"} or not 0 < path.stat().st_size <= 32*1024*1024:
            raise ValueError("Images must be PNG/JPEG/WebP and at most 32 MB")
        files.append(str(path))
    return files


class DraftWorkflows:
    def __init__(self, sessions):
        self.sessions = sessions
        self.folder = sessions.settings.data_dir / "tasks"
        self.lock = asyncio.Lock()
        if self.folder.exists():
            for path in self.folder.glob("*.json"):
                task = json.loads(path.read_text())
                if task.get("state") in {"opening_editor", "uploading", "editing", "saving"}:
                    self.write(task, "uncertain", success=False, interrupted_step=task["state"], error="Agent restarted during workflow")

    def path(self, request_id):
        if not re.fullmatch(r"[A-Za-z0-9_-]{1,64}", request_id):
            raise ValueError("request_id must contain 1–64 letters, digits, hyphens or underscores")
        return self.folder / f"{request_id}.json"

    def write(self, task, state, **extra):
        task.update(state=state, updated_at=datetime.now(timezone.utc).isoformat(), **extra)
        self.folder.mkdir(parents=True, exist_ok=True, mode=0o700)
        path = self.path(task["request_id"])
        temporary = path.with_suffix(".tmp")
        fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        with os.fdopen(fd, "w", encoding="utf-8") as out:
            json.dump(task, out, ensure_ascii=False, indent=2)
        temporary.replace(path)
        return task

    def get(self, request_id):
        path = self.path(request_id)
        return json.loads(path.read_text()) if path.exists() else {"success": False, "state": "not_found"}

    async def wait(self, backend, expression, timeout=40):
        async with asyncio.timeout(timeout):
            while True:
                try:
                    result = await backend.evaluate(expression)
                except RuntimeError:
                    result = None
                if result:
                    return result
                await asyncio.sleep(.3)

    async def editor_is_dirty(self, backend):
        return await backend.evaluate(r"""Boolean(
            document.querySelector('div.tiptap[contenteditable="true"]') &&
            (document.querySelector('div.tiptap').innerText.trim() ||
             document.querySelector('input[placeholder="填写标题会有更多赞哦"]')?.value ||
             /图片编辑\s*\d+\/18/.test(document.body.innerText)))""")

    async def create(self, sid, request_id, title, body, images):
        async with self.lock:
            path = self.path(request_id)
            files = validate_request(title, body, images)
            digest = hashlib.sha256(json.dumps({"title": title, "body": body, "images": [hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in files]}, ensure_ascii=False).encode()).hexdigest()
            if path.exists():
                previous = self.get(request_id)
                if previous.get("fingerprint") != digest:
                    return {"success": False, "state": "idempotency_conflict"}
                # Never replay a possibly completed save after timeout or restart.
                return {**previous, "reused": True}
            identity = await self.sessions.status(sid)
            if identity.get("state") != "authenticated":
                return {"success": False, "state": "login_required"}
            async with self.sessions.lock:
                session = self.sessions.sessions.get(sid)
                if not session or session["target"] != "creator":
                    return {"success": False, "state": "creator_session_required"}
                backend = session["backend"]
                if await self.editor_is_dirty(backend):
                    return {"success": False, "state": "editor_busy", "message": "Save or discard the existing editor content first"}
                task = {"request_id": request_id, "fingerprint": digest, "title": title, "body": body, "image_count": len(files), "success": False, "storage": "browser_local"}
                self.write(task, "opening_editor")
                try:
                    await backend.navigate(EDITOR_URL)
                    await self.wait(backend, f"Boolean(document.querySelector({json.dumps(UPLOAD)}))")
                    self.write(task, "uploading")
                    await backend.upload(UPLOAD, files)
                    await self.wait(backend, f"Boolean(document.querySelector({json.dumps(TITLE)}) && document.querySelector({json.dumps(BODY)}))", 90)
                    # Editor presence alone is insufficient for a multi-image upload.
                    await self.wait(backend, f"/图片编辑\\s*{len(files)}\\/18/.test(document.body.innerText)", 90)
                    if await backend.evaluate("Boolean(document.querySelector('button.feature-guide__btn'))"):
                        await backend.click('button.feature-guide__btn')
                    self.write(task, "editing")
                    await backend.fill(TITLE, title)
                    await backend.fill(BODY, body)
                    await self.wait(backend, f"document.querySelector({json.dumps(TITLE)}).value === {json.dumps(title)} && document.querySelector({json.dumps(BODY)}).innerText.trim() === {json.dumps(body.strip())}")
                    screenshot = self.folder / f"{request_id}-preview.png"
                    await backend.screenshot(screenshot)
                    self.write(task, "saving", preview=str(screenshot))
                    await backend.click_draft_save()
                    await self.wait(backend, self.list_expression(title))
                    screenshot = self.folder / f"{request_id}-saved.png"
                    await backend.screenshot(screenshot)
                    return self.write(task, "saved", success=True, evidence=str(screenshot), verification="draft_list_title")
                except Exception as exc:
                    state = "uncertain" if task["state"] == "saving" else "failed"
                    return self.write(task, state, success=False, failed_step=task["state"], error=type(exc).__name__, detail=str(exc)[:200])

    def list_expression(self, title):
        return f"document.body.innerText.includes({json.dumps(DRAFT_LIST_TEXT)}) && Array.from(document.querySelectorAll('*')).some(e=>e.children.length===0 && e.textContent.trim()==={json.dumps(title)})"

    async def verify(self, sid, request_id):
        async with self.lock:
            task = self.get(request_id)
            if "title" not in task:
                return task
            identity = await self.sessions.status(sid)
            if identity.get("state") != "authenticated":
                return {"success": False, "state": "login_required"}
            async with self.sessions.lock:
                session = self.sessions.sessions.get(sid)
                if not session or session["target"] != "creator":
                    return {"success": False, "state": "creator_session_required"}
                backend = session["backend"]
                if await self.editor_is_dirty(backend):
                    return {"success": False, "state": "editor_busy"}
                await backend.navigate(EDITOR_URL)
                await self.wait(backend, "Array.from(document.querySelectorAll('*')).some(e=>e.children.length===0 && /^草稿箱\\(\\d+\\)$/.test(e.textContent.trim()))")
                await backend.evaluate(r"""(() => {
                    const e=Array.from(document.querySelectorAll('*')).find(e=>e.children.length===0 && /^草稿箱\(\d+\)$/.test(e.textContent.trim()));
                    e.setAttribute('data-xhs-draftbox','true');
                })()""")
                await backend.click('[data-xhs-draftbox="true"]')
                await self.wait(backend, f"document.body.innerText.includes({json.dumps(DRAFT_LIST_TEXT)})")
                # The drawer defaults to VIDEO even when the upload page is image mode.
                await backend.evaluate(r"""(() => {
                    const e=Array.from(document.querySelectorAll('*')).find(e=>e.children.length===0 && /^图文笔记\(\d+\)$/.test(e.textContent.trim()));
                    if (!e) throw new Error('Image draft tab not found');
                    e.setAttribute('data-xhs-image-drafts','true');
                })()""")
                await backend.click('[data-xhs-image-drafts="true"]')
                try:
                    # IndexedDB counts and rows arrive after the drawer itself renders.
                    await self.wait(backend, self.list_expression(task["title"]), 15)
                except TimeoutError:
                    evidence = self.folder / f"{request_id}-missing.png"
                    await backend.screenshot(evidence)
                    return self.write(task, "missing", success=False, evidence=str(evidence), verification="draft_not_found")
                evidence = self.folder / f"{request_id}-verified.png"
                await backend.screenshot(evidence)
                return self.write(task, "saved", success=True, evidence=str(evidence), verification="reopened_draft_list")
