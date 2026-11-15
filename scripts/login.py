"""Interactive login through the project's nodriver agent."""
import argparse
import asyncio
import json
import os
import time

import httpx


async def login(target="explore", timeout=180):
    url = os.getenv("XHS_AGENT_URL", "http://127.0.0.1:8765").rstrip("/")
    async with httpx.AsyncClient(trust_env=False, timeout=65) as client:
        response = await client.post(url + "/browser/login", json={"target": target})
        response.raise_for_status()
        result = response.json()
        if not result.get("success"):
            return result
        sid = result["session_id"]
        print("请在项目打开的 Chrome 窗口完成小红书登录。", flush=True)
        try:
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                response = await client.get(f"{url}/browser/login/{sid}")
                response.raise_for_status()
                result = response.json()
                if result.get("state") == "authenticated":
                    return {**result, "success": True}
                if not result.get("success"):
                    return result
                await asyncio.sleep(2)
            return {"success": False, "state": "login_timeout"}
        finally:
            await client.delete(f"{url}/browser/login/{sid}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", choices=["creator", "explore"], default="explore")
    parser.add_argument("--timeout", type=int, default=180)
    args = parser.parse_args()
    result = asyncio.run(login(args.target, args.timeout))
    print("JSON_RESULT:" + json.dumps(result, ensure_ascii=False))
    raise SystemExit(0 if result.get("success") else 1)
