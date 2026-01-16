"""
XHS Signature Agent Server (Pure Algorithm)

A FastAPI-based microservice that provides XHS API signature generation
using the xhshow library. This server acts as a Signature Gateway for
the Rust Core, enabling algorithm-first API interactions.

Usage:
    python scripts/agent_server.py
    
Endpoints:
    POST /sign - Generate signatures for a given request
    GET /guest-cookies - Get guest cookies via Playwright
    GET /health - Health check
"""
import asyncio
import json
import uvicorn
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Optional, Dict, Any, List
from xhshow import Xhshow

app = FastAPI(
    title="XHS Signature Agent",
    description="Pure Algorithm Signature Gateway for Xiaohongshu API",
    version="1.1.0"
)

# Initialize Xhshow client (singleton)
xhs_client = Xhshow()


class SignRequest(BaseModel):
    """Request model for signature generation"""
    method: str  # GET or POST
    uri: str  # API path, e.g., /api/sns/web/v1/homefeed
    cookies: Dict[str, str]  # Cookie dictionary
    params: Optional[Dict[str, Any]] = None  # Query parameters (for GET)
    payload: Optional[Dict[str, Any]] = None  # Request body (for POST)


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
    """
    Generate XHS API signatures for a given request.
    
    This endpoint uses the xhshow library to compute the required
    signature headers (x-s, x-t, x-s-common, etc.) that are needed
    to make authenticated requests to the XHS API.
    
    Note: If the URI contains query parameters (e.g., ?num=20&cursor=),
    they will be automatically extracted and merged with the params field.
    """
    try:
        # Parse URI to extract path and query parameters
        from urllib.parse import urlparse, parse_qs
        parsed = urlparse(request.uri)
        uri_path = parsed.path  # Pure path without query string
        
        # Merge query parameters from URI with explicit params
        params = dict(request.params) if request.params else {}
        if parsed.query:
            query_params = parse_qs(parsed.query)
            for key, values in query_params.items():
                # parse_qs returns lists, take first value
                params[key] = values[0] if values else ""
        
        # Debug log
        import logging
        logging.info(f"[Agent] URI: {request.uri} -> path: {uri_path}, params: {params}")
        
        # Generate signatures using xhshow
        result = xhs_client.sign_headers(
            method=request.method.upper(),
            uri=uri_path,  # Use pure path
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
        return SignResponse(
            success=False,
            error=str(e)
        )


@app.get("/guest-cookies", response_model=GuestCookiesResponse)
async def get_guest_cookies():
    """
    Get guest cookies from xiaohongshu.com using Playwright.
    
    This endpoint launches a headless browser, visits the XHS homepage,
    waits for JavaScript to generate cookies, and returns them.
    
    Includes retry mechanism for improved reliability.
    
    Returns the essential cookies needed for QR code login:
    - a1, webId, gid, web_session, websectiga, acw_tc, etc.
    """
    import logging
    
    max_retries = 3
    
    for attempt in range(max_retries):
        try:
            from playwright.async_api import async_playwright
            
            logging.info(f"[Guest Cookies] Attempt {attempt + 1}/{max_retries}")
            
            async with async_playwright() as p:
                # Launch headless browser with longer timeout
                browser = await p.chromium.launch(
                    headless=True,
                    timeout=60000  # 60 seconds for browser launch
                )
                
                context = await browser.new_context(
                    user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36",
                    viewport={"width": 1920, "height": 1080}
                )
                
                # Set default timeouts
                context.set_default_timeout(30000)
                context.set_default_navigation_timeout(60000)
                
                page = await context.new_page()
                
                try:
                    # Visit homepage with timeout and multiple wait strategies
                    await page.goto(
                        "https://www.xiaohongshu.com/", 
                        wait_until="domcontentloaded",  # Faster than networkidle
                        timeout=45000
                    )
                    
                    # Wait for cookies to be generated
                    await asyncio.sleep(3)
                    
                    # Try to wait for network idle if possible
                    try:
                        await page.wait_for_load_state("networkidle", timeout=10000)
                    except:
                        pass  # Ignore if networkidle times out
                    
                    # Additional wait for JS execution
                    await asyncio.sleep(2)
                    
                finally:
                    # Always extract cookies and close browser
                    cookies_list = await context.cookies()
                    await browser.close()
                
                # Convert to dictionary
                cookies_dict = {c['name']: c['value'] for c in cookies_list}
                
                logging.info(f"[Guest Cookies] Got {len(cookies_dict)} cookies")
                
                # Verify essential cookies exist
                required = ['a1', 'webId', 'gid', 'web_session']
                missing = [k for k in required if k not in cookies_dict]
                
                if missing:
                    if attempt < max_retries - 1:
                        logging.warning(f"[Guest Cookies] Missing cookies: {missing}, retrying...")
                        await asyncio.sleep(2)
                        continue
                    return GuestCookiesResponse(
                        success=False,
                        error=f"Missing required cookies after {max_retries} attempts: {missing}"
                    )
                
                return GuestCookiesResponse(
                    success=True,
                    cookies=cookies_dict
                )
                
        except Exception as e:
            logging.error(f"[Guest Cookies] Attempt {attempt + 1} failed: {e}")
            if attempt < max_retries - 1:
                await asyncio.sleep(3)  # Wait before retry
                continue
            return GuestCookiesResponse(
                success=False,
                error=f"Failed after {max_retries} attempts: {str(e)}"
            )
    
    return GuestCookiesResponse(
        success=False,
        error="Unexpected error in retry loop"
    )


@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "service": "xhs-signature-agent"}


if __name__ == "__main__":
    print("Starting XHS Signature Agent Server...")
    print("Endpoints:")
    print("  POST /sign - Generate signatures")
    print("  GET /guest-cookies - Get guest cookies via Playwright")
    print("  GET /health - Health check")
    print("  GET /docs - OpenAPI documentation")
    uvicorn.run(app, host="127.0.0.1", port=8765)

