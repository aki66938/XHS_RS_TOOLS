"""
Playwright browser management for XHS automation
"""

import json
from playwright.async_api import async_playwright, Browser, BrowserContext, Page

from .config import (
    BROWSER_ARGS,
    EXTRA_HEADERS,
    ANTI_DETECTION_JS,
    XHS_EXPLORE_URL,
    XHS_SEARCH_URL,
    XHS_NOTIFICATION_URL,
    QR_SELECTORS,
    LOGIN_TIMEOUT_SECONDS,
    FEED_CHANNELS,
    ENDPOINT_PATTERNS,
    QRCODE_STATUS_URL,
)


class QrCodeStatusMonitor:
    """
    监听二维码登录状态的 XHR response。
    
    状态码:
        0 - 未扫码
        1 - 已扫码，等待确认
        2 - 登录成功 (包含 login_info)
    """
    
    def __init__(self):
        self.latest_status = None
        self.login_info = None
        self._status_history = []
    
    def create_response_handler(self):
        """创建 response 事件处理器"""
        async def handler(response):
            try:
                if QRCODE_STATUS_URL in response.url and response.status == 200:
                    data = await response.json()
                    if data.get("success") and "data" in data:
                        status_data = data["data"]
                        code_status = status_data.get("code_status")
                        
                        self.latest_status = data  # 保存完整响应
                        self._status_history.append(code_status)
                        
                        # 登录成功时保存 login_info
                        if code_status == 2 and "login_info" in status_data:
                            self.login_info = status_data["login_info"]
                            print(f"[QR Monitor] 登录成功! user_id: {self.login_info.get('user_id')}")
                        elif code_status == 1:
                            print("[QR Monitor] 已扫码，等待确认...")
            except Exception as e:
                # 忽略 JSON 解析错误等
                pass
        return handler
    
    def get_code_status(self) -> int:
        """获取当前状态码"""
        if self.latest_status:
            return self.latest_status.get("data", {}).get("code_status", -1)
        return -1
    
    def is_logged_in(self) -> bool:
        """检查是否登录成功"""
        return self.get_code_status() == 2
    
    def get_full_response(self) -> dict:
        """获取完整的最新响应（全量返回）"""
        return self.latest_status or {"code": -1, "success": False, "msg": "未获取到状态"}



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
        try:
            await page.wait_for_timeout(poll_interval)
        except Exception as e:
            # Handle browser/page closed by user
            if "Target" in str(e) and "closed" in str(e):
                print("\n[Browser] 用户关闭了浏览器窗口")
                result["status"] = "cancelled"
                result["error"] = "Browser was closed by user"
                return result
            raise
        
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
    
    Visits explore page and clicks search box to trigger querytrending API.
    
    Args:
        page: Playwright page object
    """
    print("[签名捕获] 正在触发 search_trending API...")
    
    try:
        # Go to explore page first
        await page.goto(XHS_EXPLORE_URL, wait_until="domcontentloaded")
        await page.wait_for_timeout(2000)
        
        # Click on the search input to trigger querytrending
        # The search box has various selectors, try multiple
        search_selectors = [
            'input[placeholder*="搜索"]',
            '.search-input',
            '#search-input',
            'input[type="search"]',
            '.nav-search input',
        ]
        
        clicked = False
        for selector in search_selectors:
            try:
                search_box = page.locator(selector).first
                if await search_box.is_visible(timeout=2000):
                    # Define predicate for querytrending response
                    def trending_predicate(response):
                        return ENDPOINT_PATTERNS["search_trending"] in response.url and response.status == 200
                    
                    async with page.expect_response(trending_predicate, timeout=8000):
                        await search_box.click()
                    
                    print("  -> [search_trending] ✅ 成功捕获签名")
                    clicked = True
                    break
            except Exception:
                continue
        
        if not clicked:
            print("  -> [search_trending] ⚠️ 未找到搜索框元素")
        
        # Close search dropdown by pressing Escape
        await page.keyboard.press("Escape")
        await page.wait_for_timeout(500)
        
        # Re-navigate to explore page to reset state for channel traversal
        await page.goto(XHS_EXPLORE_URL, wait_until="domcontentloaded")
        await page.wait_for_timeout(2000)
        
    except Exception as e:
        print(f"[签名捕获] search_trending 采集失败: {e}")


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

        print(f"  -> [{channel}] 准备采集 (目标: {target_category})", flush=True)
        
        try:
            # Locate tab
            tab = page.get_by_text(channel, exact=True).first
            if not await tab.is_visible():
                print(f"     ⚠️ 找不到频道: {channel}", flush=True)
                continue

            # Click the tab
            await tab.click()
            
            # Wait for network request (给足够时间让 API 请求触发)
            await page.wait_for_timeout(2000)
            
            # Scroll to trigger more content loading
            await page.mouse.wheel(0, 500)
            await page.wait_for_timeout(1000)
            
            # Wait for network idle
            try:
                await page.wait_for_load_state("networkidle", timeout=5000)
            except:
                pass
            
            print(f"     ✅ 成功捕获签名: {channel}", flush=True)
            
            # Human Delay (1-2s)
            delay = random.uniform(1000, 2000)
            await page.wait_for_timeout(delay)

        except Exception as e:
            print(f"     ❌ 采集失败 {channel}: {e}", flush=True)
            pass
            
    print("[Browser] 频道遍历完成\n", flush=True)


async def trigger_notification_signatures(page: Page):
    """
    Navigate to notification page to capture mentions and connections signatures.
    
    Steps:
    1. Navigate to /notification (default: mentions tab)
    2. Wait for mentions API response
    3. Click "新增关注" tab  
    4. Wait for connections API response
    """
    import random
    
    print("\n[Browser] 开始采集通知页签名...")
    
    # Navigate to notification page
    await page.goto(XHS_NOTIFICATION_URL)
    await page.wait_for_load_state("networkidle")
    await page.wait_for_timeout(2000)  # Allow initial API to fire
    
    # Mentions is default tab, should have already triggered
    print("  -> [评论和@] 默认页面已加载")
    
    # Click "新增关注" tab
    try:
        # Define predicate for connections response
        def connections_predicate(response):
            if ENDPOINT_PATTERNS["notification_connections"] not in response.url:
                return False
            return response.status == 200
        
        # Find and click the connections tab
        connections_tab = page.get_by_text("新增关注", exact=True).first
        
        if await connections_tab.is_visible():
            async with page.expect_response(connections_predicate, timeout=8000):
                await connections_tab.click()
            print("  -> [新增关注] ✅ 成功捕获签名")
        else:
            print("  -> [新增关注] ⚠️ 找不到Tab元素")
        
        # Human delay
        delay = random.uniform(2000, 3000)
        await page.wait_for_timeout(delay)
        
    except Exception as e:
        print(f"  -> [新增关注] ❌ 采集失败: {e}")
    
    print("[Browser] 通知页签名采集完成\n")


async def trigger_note_page_signature(page: Page):
    """
    Navigate to a note detail page to capture comment/page signature.
    
    This opens a sample note from the homepage to trigger the comment API.
    The signature captured can be reused for any note_id.
    """
    import random
    
    print("\n[Browser] 开始采集图文详情签名...")
    
    # First, ensure we're on explore page
    # Use domcontentloaded instead of networkidle to avoid timeout
    if "/explore" not in page.url:
        await page.goto(XHS_EXPLORE_URL, wait_until="domcontentloaded")
        await page.wait_for_timeout(3000)  # Wait for content to render
    
    try:
        # Wait for note cards to appear
        # Try multiple selectors for note cards
        selectors = [
            'section.note-item',           # Standard note item
            '.note-item',                   # Alternative
            'a[href*="/explore/"]',         # Links to note pages
            '.feeds-page section',          # Feed page sections
        ]
        
        note_card = None
        for selector in selectors:
            try:
                locator = page.locator(selector).first
                if await locator.is_visible(timeout=2000):
                    note_card = locator
                    print(f"  -> 找到笔记卡片 (selector: {selector})")
                    break
            except:
                continue
        
        if note_card:
            # Define predicate for comment page response
            def comment_predicate(response):
                pattern = ENDPOINT_PATTERNS.get("note_page", "/api/sns/web/v2/comment/page")
                if pattern not in response.url:
                    return False
                return response.status == 200
            
            # Click note card to open detail modal
            try:
                async with page.expect_response(comment_predicate, timeout=15000):
                    await note_card.click()
                
                print("  -> [图文详情] ✅ 成功捕获签名")
            except Exception as click_err:
                print(f"  -> [图文详情] ⚠️ 点击或等待响应失败: {click_err}")
            
            # Wait a moment then close the modal
            await page.wait_for_timeout(2000)
            
            # Close modal by pressing Escape
            try:
                await page.keyboard.press("Escape")
                await page.wait_for_timeout(500)
            except:
                pass
        else:
            print("  -> [图文详情] ⚠️ 找不到笔记卡片")
            
    except Exception as e:
        print(f"  -> [图文详情] ❌ 采集失败: {e}")
    
    # Human delay
    delay = random.uniform(1000, 2000)
    await page.wait_for_timeout(delay)
    
    print("[Browser] 图文详情签名采集完成\n")
