"""
XHS Playwright Package - Simplified (Pure Algorithm Architecture)

仅保留登录和 Cookie 获取功能，签名通过纯算法实时生成。
Credentials 存储于本地 cookie.json 文件。
"""

from .config import COOKIE_FILE, XHS_EXPLORE_URL
from .storage import save_credentials
from .qr_code import base64_to_ascii, extract_from_page
from .browser import (
    create_browser_context,
    setup_anti_detection,
    navigate_to_login,
    wait_for_login_complete,
    QrCodeStatusMonitor,
)

__all__ = [
    # Config
    "COOKIE_FILE",
    "XHS_EXPLORE_URL",
    # Storage
    "save_credentials",
    # QR Code
    "base64_to_ascii",
    "extract_from_page",
    # Browser
    "create_browser_context",
    "setup_anti_detection",
    "navigate_to_login",
    "wait_for_login_complete",
    "QrCodeStatusMonitor",
]
