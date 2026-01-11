"""
Playwright browser management for XHS automation
"""

from playwright.async_api import async_playwright, Browser, BrowserContext, Page

from .config import (
    BROWSER_ARGS,
    EXTRA_HEADERS,
    ANTI_DETECTION_JS,
    XHS_EXPLORE_URL,
    XHS_SEARCH_URL,
    QR_SELECTORS,
    LOGIN_TIMEOUT_SECONDS,
    FEED_CHANNELS,
)


async def create_browser_context(
    playwright,
    headless: bool = False
) -> tuple[Browser, BrowserContext]:
    """
    Create a Playwright browser and context with anti-detection settings.
    
    Args:
        playwright: Playwright instance
        headless: Whether to run headless
        
    Returns:
        Tuple of (browser, context)
    """
    browser = await playwright.chromium.launch(
        headless=headless,
        args=BROWSER_ARGS
    )
    
    context = await browser.new_context(
        viewport={'width': 1280, 'height': 800},
        user_agent='Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36',
        locale='zh-CN',
        extra_http_headers=EXTRA_HEADERS
    )
    
    return browser, context


async def setup_anti_detection(page: Page) -> None:
    """
    Apply anti-detection JavaScript to page.
    
    Args:
        page: Playwright page object
    """
    await page.add_init_script(ANTI_DETECTION_JS)


async def navigate_to_login(page: Page) -> bool:
    """
    Navigate to XHS explore page where login modal auto-appears.
    
    Args:
        page: Playwright page object
        
    Returns:
        True if login modal appeared, False otherwise
    """
    await page.goto(XHS_EXPLORE_URL, wait_until="domcontentloaded")
    await page.wait_for_timeout(2000)
    
    try:
        # Wait for QR code modal to appear (auto-popup on explore page)
        qr_wrapper = page.locator(QR_SELECTORS["qr_wrapper"])
        await qr_wrapper.wait_for(timeout=10000, state="visible")
        return True
    except Exception as e:
        print(f"[Browser] Login modal did not appear: {e}")
        return False


async def wait_for_login_complete(
    page: Page,
    context: BrowserContext,
    timeout: int = LOGIN_TIMEOUT_SECONDS
) -> dict:
    """
    Wait for user to complete QR code login.
    
    Strategies:
    1. URL redirect to /user/profile
    2. Visual element (Avatar) appearance
    3. Cookie change (if web_session updates)
    4. Login modal disappearance
    
    Args:
        page: Playwright page object
        context: Browser context
        timeout: Maximum wait time in seconds
        
    Returns:
        dict with 'success', 'cookies', 'user_info' keys
    """
    result = {
        "success": False,
        "cookies": {},
        "user_info": None,
        "status": "waiting"
    }
    
    # Capture initial state
    initial_cookies = await context.cookies()
    initial_web_session = next((c['value'] for c in initial_cookies if c['name'] == 'web_session'), None)
    
    print(f"[Browser] 等待登录... (初始 Session: {initial_web_session[:10] + '...' if initial_web_session else 'None'})")
    
    poll_interval = 2000  # 2 seconds
    max_polls = (timeout * 1000) // poll_interval
    
    for i in range(int(max_polls)):
        await page.wait_for_timeout(poll_interval)
        
        # 1. URL Check
        current_url = page.url
        if "/user/profile/" in current_url:
            print(f"[Browser] 登录成功: 检测到 URL 跳转 ({current_url})")
            result["status"] = "confirmed"
            break
            
        # 2. Visual Check (Avatar)
        try:
            avatar = page.locator(".user-side-bar .avatar-item, .side-bar .avatar-item")
            if await avatar.count() > 0 and await avatar.is_visible():
                print("[Browser] 登录成功: 检测到用户头像")
                result["status"] = "confirmed"
                break
        except:
            pass
            
        # 3. Cookie Change Check
        current_cookies = await context.cookies()
        current_web_session = next((c['value'] for c in current_cookies if c['name'] == 'web_session'), None)
        
        if current_web_session and current_web_session != initial_web_session:
             # Double check: sometimes session refreshes but user not logged in? 
             # Usually a change in web_session + presence of other cookies (like 'sid' or 'ur_id') is a good sign
             print(f"[Browser] 登录成功: 检测到 Session 变化")
             result["status"] = "confirmed"
             result["cookies"] = {c['name']: c['value'] for c in current_cookies}
             break
             
        # 4. Login Modal Gone (Weak check, maybe user closed it?)
        # Only use this if coupled with a valid session cookie
        if current_web_session:
             try:
                 qr_img = page.locator(QR_SELECTORS["qr_image"])
                 if not await qr_img.is_visible():
                     # QR gone and we have session... assume success?
                     # Let's wait one more loop to be sure or verify avatar
                     pass 
             except:
                 pass
            
    if result["status"] == "confirmed":
        # Wait a bit for cookies to settle
        await page.wait_for_timeout(2000)
        
        # Get final cookies
        cookies = await context.cookies()
        result["cookies"] = {c['name']: c['value'] for c in cookies}
        result["success"] = True
    
    return result


async def trigger_signature_pages(page: Page) -> None:
    """
    Navigate to pages that trigger API calls for signature capture.
    
    Visits search page to trigger trending and homefeed APIs.
    
    Args:
        page: Playwright page object
    """
    print("[签名捕获] 正在访问搜索页面获取签名...")
    
    try:
        await page.goto(XHS_SEARCH_URL, wait_until="domcontentloaded")
        await page.wait_for_timeout(3000)  # Wait for API calls
    except Exception as e:
        print(f"[签名捕获] 访问搜索页面失败: {e}")
    
    # Return to explore to trigger more APIs if needed
    try:
        await page.goto(XHS_EXPLORE_URL, wait_until="domcontentloaded")
        await page.wait_for_timeout(2000)
    except Exception as e:
        print(f"[签名捕获] 返回主页失败: {e}")


async def traverse_feed_channels(page: Page):
    """
    Traverse feed channels with human-like behavior:
    1. Click Channel
    2. WAIT for specific API response (200 OK + Correct Category)
    3. Random Delay (mimic reading)
    """
    from .config import CHANNEL_MAP, ENDPOINT_PATTERNS
    import random
    import json

    print(f"\n[Browser] 开始拟人化遍历 {len(FEED_CHANNELS)} 个频道...")
    
    # Ensure on explore page
    if "/explore" not in page.url:
        await page.goto(XHS_EXPLORE_URL)
        await page.wait_for_load_state("networkidle")

    for channel in FEED_CHANNELS:
        target_category = CHANNEL_MAP.get(channel)
        if not target_category:
            continue

        print(f"  -> [{channel}] 准备采集 (目标: {target_category})")
        
        try:
            # Locate tab
            tab = page.get_by_text(channel, exact=True).first
            if not await tab.is_visible():
                print(f"     ⚠️ 找不到频道: {channel}")
                continue

            # Define predicate for expected response
            # Must match: URL endpoint + POST method + Status 200 + Category in payload(implied by response context or request)
            # Actually, to be strict, we should check REQUEST payload.
            # But response object has .request property.
            def response_predicate(response):
                if ENDPOINT_PATTERNS["home_feed"] not in response.url:
                    return False
                if response.status != 200:
                    return False
                if response.request.method != "POST":
                    return False
                
                # Deep inspect request payload
                try:
                    post_data = response.request.post_data
                    if not post_data: return False
                    data = json.loads(post_data)
                    return data.get("category") == target_category
                except:
                    return False

            # Click and Wait with race condition handling
            # We start waiting BEFORE clicking to avoid missing fast responses
            async with page.expect_response(response_predicate, timeout=8000) as response_info:
                await tab.click()
                # Optional: scroll a bit to ensure activity? No, click is enough to trigger.
            
            print(f"     ✅ 成功捕获签名: {channel}")
            
            # Human Delay (2-4s)
            delay = random.uniform(2000, 4000)
            await page.wait_for_timeout(delay)

        except Exception as e:
            print(f"     ❌ 采集失败 {channel}: {e}")
            # Retry logic could go here, but keep it simple for now
            pass
            
    print("[Browser] 频道遍历完成\n")
