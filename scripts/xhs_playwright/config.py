"""
Configuration constants for XHS Playwright automation
"""

# MongoDB Configuration
MONGODB_URI = "mongodb://localhost:27017"
DATABASE_NAME = "xhs_tools"

# XHS URLs
XHS_EXPLORE_URL = "https://www.xiaohongshu.com/explore"
XHS_SEARCH_URL = "https://www.xiaohongshu.com/search_result?keyword=test"
XHS_NOTIFICATION_URL = "https://www.xiaohongshu.com/notification"

# Timeouts
LOGIN_TIMEOUT_SECONDS = 180
QR_POLL_INTERVAL_MS = 500
QR_MAX_ATTEMPTS = 30

# Browser Arguments for anti-detection
BROWSER_ARGS = [
    '--disable-blink-features=AutomationControlled',
    '--no-sandbox',
    '--disable-web-security',
    '--disable-features=IsolateOrigins,site-per-process',
]

# Extra HTTP Headers
EXTRA_HEADERS = {
    'Accept-Language': 'zh-CN,zh;q=0.9',
    'sec-ch-ua': '"Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24"',
    'sec-ch-ua-mobile': '?0',
    'sec-ch-ua-platform': '"Windows"',
}

# Anti-detection JavaScript
ANTI_DETECTION_JS = '''
    Object.defineProperty(navigator, 'webdriver', {get: () => undefined});
    Object.defineProperty(navigator, 'plugins', {get: () => [1, 2, 3, 4, 5]});
    Object.defineProperty(navigator, 'languages', {get: () => ['zh-CN', 'zh', 'en']});
    window.chrome = {runtime: {}};
'''

# Endpoint URL patterns for signature capture
ENDPOINT_PATTERNS = {
    "user_me": "/api/sns/web/v2/user/me",  # 登录后自动触发
    "search_trending": "/api/sns/web/v1/search/querytrending",  # 登录后自动触发
    # Feed endpoint (homefeed_recommend 登录后自动触发)
    "home_feed": "/api/sns/web/v1/homefeed",
    # Notification endpoints (v1.1.0) - 需要 playwright 触发
    "notification_mentions": "/api/sns/web/v1/you/mentions",
    "notification_connections": "/api/sns/web/v1/you/connections",
    # Note endpoints - 需要 playwright 触发
    "note_page": "/api/sns/web/v2/comment/page",  # 图文详情（评论分页）
}

# QR Code status API (for XHR monitoring, not signature capture)
QRCODE_STATUS_URL = "/api/sns/web/v1/login/qrcode/status"


# Channel Mapping: Tab Name -> API Category ID
# This ensures we wait for the EXACT correct response
CHANNEL_MAP = {
    "推荐": "homefeed_recommend", # Implicit
    "穿搭": "homefeed.fashion_v3",
    "美食": "homefeed.food_v3",
    "彩妆": "homefeed.cosmetics_v3",
    "影视": "homefeed.movie_and_tv_v3",
    "职场": "homefeed.career_v3",
    "情感": "homefeed.love_v3",              # Verified
    "家居": "homefeed.household_product_v3", # Verified
    "游戏": "homefeed.gaming_v3",            # Verified
    "旅行": "homefeed.travel_v3",
    "健身": "homefeed.fitness_v3",
}

# Feed Channels to traverse (order matters for UI)
FEED_CHANNELS = [
    "推荐", "穿搭", "美食", "彩妆", "影视", 
    "职场", "情感", "家居", "游戏", "旅行", "健身"
]

# QR Code selectors (img.qrcode-img is the actual QR image element)
QR_SELECTORS = {
    "qr_image": 'img.qrcode-img',            # Main QR image element
    "qr_wrapper": '.qrcode',                  # QR wrapper div
    "login_button": 'div.login-btn',
    "login_container": '.login-container',
}

# Login success indicators
LOGIN_SUCCESS_INDICATORS = [
    '/user/profile/',
    'web_session',
]
