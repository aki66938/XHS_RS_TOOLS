"""
XHS Signature Agent Server (Pure Algorithm)

A FastAPI-based microservice that provides XHS API signature generation
using the xhshow library. This server acts as a Signature Gateway for
the Rust Core, enabling algorithm-first API interactions.

Migrated to undetected-chromedriver (UC) for better anti-detection in Docker.

Usage:
    python scripts/agent_server.py
    
Endpoints:
    POST /sign - Generate signatures for a given request
    GET /guest-cookies - Get guest cookies via UC
    POST /sync-login-cookies - Sync full browser cookies via UC
    GET /health - Health check
"""
import asyncio
import json
import uvicorn
import os
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Optional, Dict, Any, List
from xhshow import Xhshow
import logging
import time
import undetected_chromedriver as uc
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("agent_server")

logger.info(f"Agent Server Starting...")
logger.info(f"HTTP_PROXY: {os.environ.get('HTTP_PROXY')}")
logger.info(f"HTTPS_PROXY: {os.environ.get('HTTPS_PROXY')}")
logger.info(f"NO_PROXY: {os.environ.get('NO_PROXY')}")

app = FastAPI(
    title="XHS Signature Agent",
    description="Pure Algorithm Signature Gateway for Xiaohongshu API (UC Powered)",
    version="1.3.0"
)

# Initialize Xhshow client (singleton)
xhs_client = Xhshow()


class SignRequest(BaseModel):
    """Request model for signature generation"""
    method: str
    uri: str
    cookies: Dict[str, str]
    params: Optional[Dict[str, Any]] = None
    payload: Optional[Dict[str, Any]] = None


class SignResponse(BaseModel):
    """Response model containing generated signatures"""
    success: bool
    x_s: Optional[str] = None
    x_t: Optional[str] = None
    x_s_common: Optional[str] = None
    x_b3_traceid: Optional[str] = None
    x_xray_traceid: Optional[str] = None
    error: Optional[str] = None


class GuestCookiesResponse(BaseModel):
    """Response model for guest cookies"""
    success: bool
    cookies: Optional[Dict[str, str]] = None
    error: Optional[str] = None


@app.post("/sign", response_model=SignResponse)
async def generate_signature(request: SignRequest):
    """Generate XHS API signatures"""
    try:
        from urllib.parse import urlparse, parse_qs
        parsed = urlparse(request.uri)
        uri_path = parsed.path
        
        params = dict(request.params) if request.params else {}
        if parsed.query:
            query_params = parse_qs(parsed.query)
            for key, values in query_params.items():
                params[key] = values[0] if values else ""
        
        result = xhs_client.sign_headers(
            method=request.method.upper(),
            uri=uri_path,
            cookies=request.cookies,
            params=params if params else None,
            payload=request.payload
        )
        
        return SignResponse(
            success=True,
            x_s=result.get("x-s"),
            x_t=str(result.get("x-t", "")),
            x_s_common=result.get("x-s-common"),
            x_b3_traceid=result.get("x-b3-traceid"),
            x_xray_traceid=result.get("x-xray-traceid")
        )
    except Exception as e:
        return SignResponse(success=False, error=str(e))


def get_chrome_options():
    options = uc.ChromeOptions()
    # options.add_argument('--headless=new') # Headless often triggers anti-bot, but might be needed in Docker.
    # For Docker without Xvfb, we MUST use headless. 
    # UC's headless handling is tricky. Let's try standard headless first.
    # Headless mode REMOVED for Docker with Xvfb (Selenium Base Image)
    # The base image provides Xvfb on DISPLAY=:99
    # options.add_argument('--headless=new') 
    options.add_argument("--no-sandbox")
    options.add_argument("--disable-gpu")
    options.add_argument("--disable-dev-shm-usage")
    options.add_argument("--window-size=1920,1080")
    options.add_argument("--lang=zh-CN")
    
    # Explicitly set binary location for Selenium Base Image
    # In selenium/standalone-chrome, chrome is usually at /opt/google/chrome/google-chrome
    # or /usr/bin/google-chrome
    if os.path.exists("/opt/google/chrome/google-chrome"):
        options.binary_location = "/opt/google/chrome/google-chrome"
    elif os.path.exists("/usr/bin/google-chrome"):
        options.binary_location = "/usr/bin/google-chrome"
    
    # Explicitly configure proxy from Environment
    # Chrome sometimes ignores Env Vars if not set specifically
    proxy_url = os.environ.get('HTTP_PROXY')
    if proxy_url:
        logger.info(f"Setting Chrome Proxy: {proxy_url}")
        options.add_argument(f"--proxy-server={proxy_url}")
        
    no_proxy = os.environ.get('NO_PROXY')
    if no_proxy:
         logger.info(f"Setting Chrome Bypass: {no_proxy}")
         options.add_argument(f"--proxy-bypass-list={no_proxy}")

    return options

def get_driver_executable_path():
    # Helper to find chromedriver in common locations
    candidates = [
        "/usr/bin/chromedriver",
        "/usr/local/bin/chromedriver",
        "/opt/selenium/chromedriver",
        "/usr/bin/chromedriver-original" # Sometimes moved
    ]
    for path in candidates:
        if os.path.exists(path):
            return path
    return None

@app.get("/guest-cookies", response_model=GuestCookiesResponse)
async def get_guest_cookies(target: str = "explore"):
    """
    Get guest cookies using undetected-chromedriver
    
    Args:
        target: Target page ("explore" [default], "creator")
    """
    driver = None
    try:
        logger.info(f"[Guest Cookies] Starting UC Driver (Headed) for target: {target}...")
        options = get_chrome_options()
        
        driver_path = get_driver_executable_path()
        if driver_path:
             logger.info(f"[Guest Cookies] Found chromedriver at {driver_path}, skipping download.")
             driver = uc.Chrome(options=options, driver_executable_path=driver_path, version_main=120) 
        else:
             logger.warning("[Guest Cookies] Chromedriver not found, attempting auto-download...")
             driver = uc.Chrome(options=options, version_main=None) 
        
        target_url = "https://www.xiaohongshu.com/explore"
        if target == "creator":
            target_url = "https://creator.xiaohongshu.com/login"
            
        logger.info(f"[Guest Cookies] Navigating to {target_url}...")
        driver.get(target_url)
        
        # Wait for page load
        time.sleep(5)
        
        title = driver.title
        logger.info(f"[Guest Cookies] Page Title: {title}")
        
        # Check title to verify load
        if "xiaohongshu" not in title and "小红书" not in title:
             logger.warning(f"[Guest Cookies] Suspicious title: {title}")
        
        # Get cookies
        selenium_cookies = driver.get_cookies()
        cookies_dict = {c['name']: c['value'] for c in selenium_cookies}
        
        logger.info(f"[Guest Cookies] Got {len(cookies_dict)} cookies")
        
        # Verify
        required = ['a1', 'webId', 'gid', 'web_session']
        missing = [k for k in required if k not in cookies_dict]
        
        if missing:
             logger.warning(f"[Guest Cookies] Missing: {missing}")
        
        return GuestCookiesResponse(success=True, cookies=cookies_dict)

    except Exception as e:
        logger.error(f"[Guest Cookies] Failed: {e}")
        return GuestCookiesResponse(success=False, error=str(e))
    finally:
        if driver:
            try:
                driver.quit()
            except:
                pass


class SyncCookiesRequest(BaseModel):
    web_session: Optional[str] = None
    cookies: Optional[Dict[str, str]] = None
    target: str = "explore"


@app.post("/sync-login-cookies", response_model=GuestCookiesResponse)
async def sync_login_cookies(request: SyncCookiesRequest):
    """Sync login cookies using UC"""
    web_session = request.web_session
    input_cookies = request.cookies
    target = request.target
    
    # Merge input cookies
    cookies_to_inject = {}
    if web_session:
        cookies_to_inject["web_session"] = web_session
    if input_cookies:
        cookies_to_inject.update(input_cookies)
        
    if not cookies_to_inject:
        return GuestCookiesResponse(success=False, error="Missing cookies (web_session or cookies dict)")
        
    driver = None
    try:
        logger.info(f"[Cookie Sync] Starting sync... Target: {target}, Injecting {len(cookies_to_inject)} cookies")
        options = get_chrome_options()
        
        # Use same driver path logic as get_guest_cookies
        driver_path = get_driver_executable_path()
        if driver_path:
            logger.info(f"[Cookie Sync] Found chromedriver at {driver_path}")
            driver = uc.Chrome(options=options, driver_executable_path=driver_path, version_main=120)
        else:
            logger.warning("[Cookie Sync] Chromedriver not found, attempting auto-download...")
            driver = uc.Chrome(options=options, version_main=None)
        
        # Domain init
        driver.get("https://www.xiaohongshu.com/404")
        time.sleep(2)
        
        # Add cookies
        for name, value in cookies_to_inject.items():
            driver.add_cookie({
                "name": name,
                "value": value,
                "domain": ".xiaohongshu.com",
                "path": "/"
            })
        
        # Determine target URL
        target_url = "https://www.xiaohongshu.com/"
        if target == "creator":
            target_url = "https://creator.xiaohongshu.com/creator/home"
            
        # Go to target to trigger full cookie generation
        logger.info(f"[Cookie Sync] Navigating to {target_url}...")
        driver.get(target_url)
        time.sleep(5)
        
        selenium_cookies = driver.get_cookies()
        cookies_dict = {c['name']: c['value'] for c in selenium_cookies}
        
        logger.info(f"[Cookie Sync] Got {len(cookies_dict)} cookies")
        
        # Verify critical cookies (check loosely as requirements differ by target)
        return GuestCookiesResponse(success=True, cookies=cookies_dict)
        
    except Exception as e:
        logger.error(f"[Cookie Sync] Failed: {e}")
        return GuestCookiesResponse(success=False, error=str(e))
    finally:
        if driver:
            try:
                driver.quit()
            except:
                pass


@app.get("/health")
async def health_check():
    return {"status": "healthy", "service": "xhs-signature-agent", "backend": "undetected-chromedriver"}


if __name__ == "__main__":
    print("Starting XHS Signature Agent Server (UC Powered)...")
    uvicorn.run(app, host="0.0.0.0", port=8765)
