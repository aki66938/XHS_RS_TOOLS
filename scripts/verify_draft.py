"""Run the real stdio MCP workflow against the local agent; never publishes."""
import argparse
import asyncio
import json
import sys
import time
from pathlib import Path

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from .xhs_browser.config import PROJECT_ROOT


async def verify(args):
    parameters = StdioServerParameters(
        command=sys.executable, args=["-m", "scripts.mcp_server"], cwd=str(PROJECT_ROOT),
    )
    async with stdio_client(parameters) as streams:
        async with ClientSession(*streams) as client:
            await client.initialize()
            async def call(name, arguments=None):
                result = await client.call_tool(name, arguments or {})
                if result.isError:
                    raise RuntimeError(str(result.content))
                value = result.structuredContent or json.loads(result.content[0].text)
                print(json.dumps({"tool": name, "result": value}, ensure_ascii=False), flush=True)
                if value.get("success") is False:
                    raise RuntimeError("Workflow stopped: " + value.get("state", "unknown"))
                return value

            await call("browser_health")
            session = await call("start_login", {"target": "creator"})
            sid = session["session_id"]
            deadline = time.monotonic() + 180
            while True:
                identity = await call("get_login_status", {"session_id": sid})
                if identity.get("state") == "authenticated":
                    break
                if time.monotonic() >= deadline:
                    raise TimeoutError("Complete login in the project's Chrome window and rerun")
                await asyncio.sleep(2)
            if not args.verify_only:
                await call("create_image_note_draft", {
                    "session_id": sid, "request_id": args.request_id,
                    "title": args.title, "body": args.body,
                    "images": [str(Path(image).resolve()) for image in args.image],
                })
            if args.restart:
                await call("close_browser_session", {"session_id": sid})
                session = await call("start_login", {"target": "creator"})
                sid = session["session_id"]
            result = await call("verify_saved_draft", {"session_id": sid, "request_id": args.request_id})
            print("VERIFIED: " + result["evidence"], flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--request-id", required=True)
    parser.add_argument("--image", action="append", default=[])
    parser.add_argument("--title", default="nodriver草稿验证")
    parser.add_argument("--body", default="工程验证草稿，仅用于测试上传、输入和持久化，不公开发布。")
    parser.add_argument("--verify-only", action="store_true", help="Check an existing recorded task; do not create a draft")
    parser.add_argument("--restart", action="store_true", help="Restart Chrome before verifying the draft")
    args = parser.parse_args()
    if not args.verify_only and not args.image:
        parser.error("provide --image or use --verify-only")
    try:
        asyncio.run(verify(args))
    except Exception as error:
        while isinstance(error, BaseExceptionGroup) and error.exceptions:
            error = error.exceptions[0]
        print(f"FAILED: {error}", file=sys.stderr)
        raise SystemExit(1)
