"""Signature gateway and nodriver workflows. Run: python -m scripts.agent_server."""
import logging
import os
from contextlib import asynccontextmanager
from typing import Any, Literal
from urllib.parse import urlparse, parse_qs

import uvicorn
from fastapi import FastAPI
from fastapi.responses import JSONResponse
from filelock import Timeout as ProfileLocked
from pydantic import BaseModel, Field
from xhshow import Xhshow

from .xhs_browser.config import BrowserSettings
from .xhs_browser.service import BrowserService
from .xhs_browser.sessions import LoginSessions
from .xhs_browser.drafts import DraftWorkflows

logger = logging.getLogger(__name__)
settings = BrowserSettings.from_env()
browser_service = BrowserService(settings)
login_sessions = LoginSessions(settings)
draft_workflows = DraftWorkflows(login_sessions)
xhs_client = Xhshow()
Target = Literal["explore", "creator"]


@asynccontextmanager
async def lifespan(app):
    try:
        yield
    finally:
        await login_sessions.close_all()


app = FastAPI(title="XHS Browser Agent", version="2.0.0", lifespan=lifespan)


@app.exception_handler(ValueError)
async def invalid_request(request, exc):
    return JSONResponse(status_code=422, content={"success": False, "state": "invalid_request", "error": str(exc)})


@app.exception_handler(FileNotFoundError)
async def missing_file(request, exc):
    return JSONResponse(status_code=422, content={"success": False, "state": "missing_file"})


@app.exception_handler(ProfileLocked)
async def profile_busy(request, exc):
    return JSONResponse(status_code=409, content={"success": False, "state": "profile_in_use"})


@app.exception_handler(TimeoutError)
async def browser_timeout(request, exc):
    return JSONResponse(status_code=504, content={"success": False, "state": "browser_timeout"})


class SignRequest(BaseModel):
    method: str
    uri: str
    cookies: dict[str, str]
    params: dict[str, Any] | None = None
    payload: dict[str, Any] | None = None


class SignResponse(BaseModel):
    success: bool
    x_s: str | None = None
    x_t: str | None = None
    x_s_common: str | None = None
    x_b3_traceid: str | None = None
    x_xray_traceid: str | None = None
    error: str | None = None


class GuestCookiesResponse(BaseModel):
    success: bool
    cookies: dict[str, str] | None = None
    error: str | None = None


@app.post("/sign", response_model=SignResponse)
async def generate_signature(request: SignRequest):
    try:
        parsed = urlparse(request.uri)
        params = dict(request.params or {})
        params.update({k: v[0] if v else "" for k, v in parse_qs(parsed.query).items()})
        result = xhs_client.sign_headers(
            method=request.method.upper(), uri=parsed.path, cookies=request.cookies,
            params=params or None, payload=request.payload,
        )
        return SignResponse(
            success=True, x_s=result.get("x-s"), x_t=str(result.get("x-t", "")),
            x_s_common=result.get("x-s-common"), x_b3_traceid=result.get("x-b3-traceid"),
            x_xray_traceid=result.get("x-xray-traceid"),
        )
    except Exception as exc:
        return SignResponse(success=False, error=str(exc))


@app.get("/guest-cookies", response_model=GuestCookiesResponse)
async def get_guest_cookies(target: Target = "explore"):
    return await collect_cookies(target)


class SyncCookiesRequest(BaseModel):
    web_session: str | None = None
    cookies: dict[str, str] | None = None
    target: Target = "explore"


@app.post("/sync-login-cookies", response_model=GuestCookiesResponse)
async def sync_login_cookies(request: SyncCookiesRequest):
    cookies = {"web_session": request.web_session} if request.web_session else {}
    cookies.update(request.cookies or {})
    if not cookies:
        return GuestCookiesResponse(success=False, error="Missing cookies to synchronize")
    return await collect_cookies(request.target, cookies)


async def collect_cookies(target, cookies=None):
    try:
        return GuestCookiesResponse(success=True, cookies=await browser_service.collect(target, cookies))
    except TimeoutError:
        return GuestCookiesResponse(success=False, error="Browser timed out before valid cookies were available")
    except Exception as exc:
        logger.warning("Browser collection failed: %s", type(exc).__name__)
        return GuestCookiesResponse(success=False, error=f"Browser operation failed ({type(exc).__name__})")


class LoginRequest(BaseModel):
    target: Target = "creator"


@app.post("/browser/login")
async def start_login(request: LoginRequest):
    return await login_sessions.start(request.target)


@app.get("/browser/login/{session_id}")
async def login_status(session_id: str):
    return await login_sessions.status(session_id)


@app.get("/browser/login/{session_id}/view")
async def login_view(session_id: str):
    return await login_sessions.view(session_id)


@app.delete("/browser/login/{session_id}")
async def close_login(session_id: str):
    return await login_sessions.close(session_id)


class DraftRequest(BaseModel):
    session_id: str = Field(pattern=r"^[a-f0-9]{32}$")
    request_id: str = Field(pattern=r"^[A-Za-z0-9_-]{1,64}$")
    title: str = Field(min_length=1, max_length=20)
    body: str = Field(min_length=1, max_length=1000)
    images: list[str] = Field(min_length=1, max_length=18)


@app.post("/browser/drafts")
async def create_draft(request: DraftRequest):
    return await draft_workflows.create(request.session_id, request.request_id, request.title, request.body, request.images)


@app.get("/browser/drafts/{request_id}")
async def draft_status(request_id: str):
    return draft_workflows.get(request_id)


@app.post("/browser/drafts/{request_id}/verify")
async def verify_draft(request_id: str, session_id: str):
    return await draft_workflows.verify(session_id, request_id)


@app.get("/health")
async def health_check():
    return {"status": "healthy", "service": "xhs-browser-agent", "backend": "nodriver"}


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    uvicorn.run("scripts.agent_server:app", host=os.getenv("XHS_AGENT_HOST", "127.0.0.1"), port=8765)
