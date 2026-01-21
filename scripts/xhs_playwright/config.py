"""
Configuration constants for XHS Playwright automation
"""

from pathlib import Path

# Cookie file path (project root)
COOKIE_FILE = Path(__file__).parent.parent.parent / "cookie.json"

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

# QR Code status API (for XHR monitoring)
QRCODE_STATUS_URL = "/api/sns/web/v1/login/qrcode/status"

# QR Code selectors (img.qrcode-img is the actual QR image element)
QR_SELECTORS = {
    "qr_image": 'img.qrcode-img',            # Main QR image element
    "qr_wrapper": '.qrcode',                  # QR wrapper div
    "login_button": 'div.login-btn',
    "login_container": '.login-container',
}
