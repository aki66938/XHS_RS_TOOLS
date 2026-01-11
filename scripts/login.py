#!/usr/bin/env python3
"""
XHS Login Script - Main Entry Point

Slim orchestration layer using xhs_playwright package.
Supports:
- Interactive login with QR code display
- Headless mode with JSON output
- Signature capture for API endpoints
"""

import sys
import json
import asyncio
import argparse
from datetime import datetime
from playwright.async_api import async_playwright

# Import from xhs_playwright package
from xhs_playwright import (
    save_credentials,
    SignatureCapture,
)
from xhs_playwright.browser import (
    create_browser_context,
    setup_anti_detection,
    navigate_to_login,
    wait_for_login_complete,
    trigger_signature_pages,
    traverse_feed_channels,
)
from xhs_playwright.qr_code import extract_from_page, base64_to_ascii


async def run_extract_qr(headless: bool = False) -> dict:
    """
    Extract QR code only mode.
    
    Returns:
        dict with qr_base64, qr_ascii, success
    """
    async with async_playwright() as p:
        browser, context = await create_browser_context(p, headless)
        page = await context.new_page()
        await setup_anti_detection(page)
        
        await navigate_to_login(page)
        qr_result = await extract_from_page(page)
        
        await browser.close()
        return qr_result


async def run_full_login(headless: bool = False, json_mode: bool = False) -> dict:
    """
    Full login flow with signature capture.
    
    Args:
        headless: Run browser headlessly
        json_mode: Output JSON only (for API integration)
        
    Returns:
        dict with success, user_id, cookie_count, signatures_captured
    """
    result = {
        "success": False,
        "user_id": None,
        "cookie_count": 0,
        "signatures_captured": [],
        "error": None
    }
    
    async with async_playwright() as p:
        browser, context = await create_browser_context(p, headless)
        page = await context.new_page()
        await setup_anti_detection(page)
        
        # Set up signature capture
        sig_capture = SignatureCapture()
        page.on("request", sig_capture.create_request_handler())
        
        # Navigate and show login
        if not await navigate_to_login(page):
            result["error"] = "Failed to navigate to login"
            await browser.close()
            return result
        
        # Extract and display QR
        qr_result = await extract_from_page(page)
        
        if qr_result["success"]:
            if json_mode:
                # Output JSON format expected by Rust: QrCodeSessionResponse
                # Fields: success, step, qr_base64, error
                print(json.dumps({
                    "success": True,
                    "step": "qrcode",
                    "qr_base64": qr_result["base64"]  # full data:image/png;base64,... URI
                }), flush=True)
            else:
                print("\n" + "="*50)
                print("  扫描以下二维码登录小红书:")
                print("="*50)
                print(qr_result["ascii"])
                print("="*50 + "\n")
        else:
            result["error"] = "Failed to extract QR code"
            await browser.close()
            return result
        
        # Wait for login
        if not json_mode:
            print("等待登录", end="", flush=True)
        
        login_result = await wait_for_login_complete(page, context)
        
        if not login_result["success"]:
            result["error"] = "Login timeout or failed"
            await browser.close()
            return result
        
        if not json_mode:
            print("\n\n✅ 登录成功！")
        
        # [PRIORITY FIX] Save credentials IMMEDIATELY after login
        # This ensures the session is usable even if signature capture fails later
        cookies = login_result["cookies"]
        # x_s_common might be captured during login flow, or empty initially
        user_id = save_credentials(cookies, sig_capture.x_s_common)
        
        if not json_mode:
            print(f"[Login] 凭证已保存 (User ID: {user_id})")

        # Navigate to capture more signatures (Basic only)
        try:
            print("[Browser] 开始基础签名采集...")
            await trigger_signature_pages(page)
            
            # [ENABLED] Robotic traversal for full coverage
            print("[Browser] 完成基础签名采集，开始拟人化遍历频道...")
            await traverse_feed_channels(page) 
            
        except Exception as e:
            import traceback
            print(f"[Browser] ❌ 签名采集发生错误 (非致命): {e}")
            print(traceback.format_exc())

        await page.wait_for_timeout(2000)
        
        # Save signatures
        saved_endpoints = sig_capture.save_all_signatures()
        
        if not json_mode:
            for ep in saved_endpoints:
                print(f"[签名捕获] 已保存 {ep} 签名")
        
        result["success"] = True
        result["user_id"] = user_id
        result["cookie_count"] = len(cookies)
        result["signatures_captured"] = saved_endpoints
        
        await browser.close()
    
    # Output JSON if in json_mode
    if json_mode and result["success"]:
        print(json.dumps(result))
    
    return result


def main():
    """CLI entry point"""
    parser = argparse.ArgumentParser(description="XHS Login Script")
    parser.add_argument("--mode", choices=["login", "extract-qr"], default="login",
                        help="Operation mode")
    parser.add_argument("--headless", action="store_true", 
                        help="Run browser headlessly")
    parser.add_argument("--json", action="store_true",
                        help="Output JSON format (for API integration)")
    
    args = parser.parse_args()
    
    if args.mode == "extract-qr":
        result = asyncio.run(run_extract_qr(headless=args.headless))
        if args.json:
            print(json.dumps(result))
    else:
        asyncio.run(run_full_login(headless=args.headless, json_mode=args.json))


if __name__ == "__main__":
    main()
