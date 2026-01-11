"""
XHS Playwright Package
Modular login and signature capture for Xiaohongshu API
"""

from .config import MONGODB_URI, DATABASE_NAME, XHS_EXPLORE_URL
from .storage import save_credentials, save_signature
from .qr_code import base64_to_ascii, extract_from_page
from .signature import SignatureCapture
from .browser import (
    create_browser_context,
    setup_anti_detection,
    navigate_to_login,
    wait_for_login_complete,
    trigger_signature_pages,
)

__all__ = [
    # Config
    "MONGODB_URI",
    "DATABASE_NAME", 
    "XHS_EXPLORE_URL",
    # Storage
    "save_credentials",
    "save_signature",
    # QR Code
    "base64_to_ascii",
    "extract_from_page",
    # Signature
    "SignatureCapture",
    # Browser
    "create_browser_context",
    "setup_anti_detection",
    "navigate_to_login",
    "wait_for_login_complete",
    "trigger_signature_pages",
    "traverse_feed_channels",
]
