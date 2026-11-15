"""Business MCP over the project's local nodriver agent. No generic browser tools."""
import os
import re
from typing import Literal

import httpx
from mcp.server.fastmcp import FastMCP

mcp = FastMCP("xhs-rs-tools")
AGENT_URL = os.getenv("XHS_AGENT_URL", "http://127.0.0.1:8765").rstrip("/")


def identifier(value, session=False):
    pattern = r"[a-f0-9]{32}" if session else r"[A-Za-z0-9_-]{1,64}"
    if not re.fullmatch(pattern, value):
        raise ValueError("Invalid session or request ID")
    return value


async def request(method, path, **kwargs):
    async with httpx.AsyncClient(timeout=210, trust_env=False) as client:
        try:
            response = await client.request(method, AGENT_URL + path, **kwargs)
        except httpx.RequestError:
            return {"success": False, "state": "agent_unavailable"}
        if response.is_error:
            try:
                value = response.json()
                if isinstance(value, dict) and value.get("success") is False:
                    return value
            except ValueError:
                pass
            return {"success": False, "state": "agent_error", "http_status": response.status_code}
        return response.json()


@mcp.tool()
async def browser_health() -> dict:
    """Report the nodriver agent's availability. Does not start a browser."""
    return await request("GET", "/health")


@mcp.tool()
async def start_login(target: Literal["creator", "explore"] = "creator") -> dict:
    """Open the project's persistent Chrome profile for human login, or restore its session."""
    return await request("POST", "/browser/login", json={"target": target})


@mcp.tool()
async def get_login_status(session_id: str) -> dict:
    """Check the site's identity endpoint and save credentials locally. Never returns cookies.

    unverified means identity has not been confirmed, not necessarily logged out.
    """
    return await request("GET", f"/browser/login/{identifier(session_id, True)}")


@mcp.tool()
async def get_browser_view(session_id: str) -> dict:
    """Return current page text and a local screenshot path for reviewing the workflow."""
    return await request("GET", f"/browser/login/{identifier(session_id, True)}/view")


@mcp.tool()
async def close_browser_session(session_id: str) -> dict:
    """Close Chrome cleanly while preserving login and local drafts in its profile."""
    return await request("DELETE", f"/browser/login/{identifier(session_id, True)}")


@mcp.tool()
async def create_image_note_draft(session_id: str, request_id: str, title: str, body: str, images: list[str]) -> dict:
    """Upload images, enter text, save a LOCAL creator draft, and verify its title in the draft list.

    Requires an authenticated creator session. Images must be absolute paths within
    XHS_UPLOAD_DIR (defaults to the repository's uploads directory). Reuse request_id
    only for the same content: a repeated call returns its recorded result and never
    blindly repeats an uncertain save. This tool does not publish a note.
    """
    return await request("POST", "/browser/drafts", json={
        "session_id": identifier(session_id, True), "request_id": identifier(request_id),
        "title": title, "body": body, "images": images,
    })


@mcp.tool()
async def get_draft_task(request_id: str) -> dict:
    """Read a saved task result. failed/uncertain states require inspection before another attempt."""
    return await request("GET", f"/browser/drafts/{identifier(request_id)}")


@mcp.tool()
async def verify_saved_draft(session_id: str, request_id: str) -> dict:
    """Reopen the draft list and verify a task's title, including after a Chrome restart."""
    return await request("POST", f"/browser/drafts/{identifier(request_id)}/verify", params={"session_id": identifier(session_id, True)})


if __name__ == "__main__":
    mcp.run(transport="stdio")
