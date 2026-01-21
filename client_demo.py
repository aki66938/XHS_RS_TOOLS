#!/usr/bin/env python3
"""
XHS API 客户端演示 (Pure Rust Architecture v2.0)

测试模块位于 scripts/test_demo/
"""

import sys
import os
import urllib.request
import json
import time

# Optional: QR code display in terminal
try:
    import qrcode
    HAS_QRCODE = True
except ImportError:
    HAS_QRCODE = False

# Add scripts directory to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'scripts'))

BASE_URL = "http://localhost:3005"

from scripts.test_demo.test_user import test_user_me
from scripts.test_demo.test_search import (
    test_trending, test_search_recommend, test_search_notes,
    test_search_onebox, test_search_user, test_search_filter
)
from scripts.test_demo.test_feed import test_homefeed, test_category_feeds
from scripts.test_demo.test_notification import test_notifications
from scripts.test_demo.test_note import test_note_page, test_note_detail
from scripts.test_demo.test_pagination import test_homefeed_pagination
from scripts.test_demo.test_media import test_media


# ============================================================================
# Login Flow
# ============================================================================

def guest_init():
    """Step 1: 获取访客 Cookie"""
    print("\n[1/3] 初始化访客会话...")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/auth/guest-init", method='POST')
        with urllib.request.urlopen(req, timeout=60) as response:
            data = json.loads(response.read().decode('utf-8'))
            
        if data.get("success"):
            cookies = data.get("cookies", {})
            print(f"    ✅ 获取访客 Cookie 成功 (数量: {len(cookies)})")
            return True
        else:
            print(f"    ❌ 失败: {data.get('error')}")
            return False
    except Exception as e:
        print(f"    ❌ 错误: {e}")
        return False


def create_qrcode():
    """Step 2: 创建二维码"""
    print("\n[2/3] 创建登录二维码...")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/auth/qrcode/create", method='POST')
        with urllib.request.urlopen(req, timeout=30) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            qr_url = data.get("qr_url")
            qr_id = data.get("qr_id")
            
            print(f"    ✅ 二维码创建成功")
            print(f"    QR ID: {qr_id}")
            
            if HAS_QRCODE and qr_url:
                print("\n" + "=" * 50)
                print("  请使用小红书 App 扫描以下二维码:")
                print("=" * 50)
                qr = qrcode.QRCode(border=1)
                qr.add_data(qr_url)
                qr.print_ascii(invert=True)
                print("=" * 50)
            else:
                print(f"    扫码链接: {qr_url}")
            
            return True
        else:
            print(f"    ❌ 失败: {data.get('error')}")
            return False
    except Exception as e:
        print(f"    ❌ 错误: {e}")
        return False


def poll_qrcode_status(timeout=120):
    """Step 3: 轮询二维码状态"""
    print("\n[3/3] 等待扫码登录...")
    print("    ", end="", flush=True)
    
    start_time = time.time()
    last_status = -1
    
    while time.time() - start_time < timeout:
        try:
            req = urllib.request.Request(f"{BASE_URL}/api/auth/qrcode/status")
            # Increase timeout to 60s to allow blocking cookie sync (browser launch takes time)
            with urllib.request.urlopen(req, timeout=60) as response:
                data = json.loads(response.read().decode('utf-8'))
            
            if data.get("success"):
                code_status = data.get("code_status", -1)
                
                if code_status == 2:
                    print("\n")
                    print("    ✅ 登录成功!")
                    login_info = data.get("login_info", {})
                    if login_info:
                        print(f"    User ID: {login_info.get('user_id', 'N/A')}")
                    new_cookies = data.get("new_cookies", {})
                    if new_cookies:
                        print(f"    获取新 Cookie: {len(new_cookies)} 个")
                    return True
                    
                elif code_status == 1 and last_status != 1:
                    print("✓", end="", flush=True)
                    last_status = 1
                elif code_status == 0:
                    print(".", end="", flush=True)
            else:
                print("x", end="", flush=True)
                
        except Exception:
            print("!", end="", flush=True)
        
        time.sleep(2)
    
    print("\n    ❌ 登录超时")
    return False


def login_flow():
    """完整登录流程"""
    print("\n" + "=" * 50)
    print("  开始登录流程 (Pure Rust Architecture)")
    print("=" * 50)
    
    if not guest_init():
        return False
    if not create_qrcode():
        return False
    return poll_qrcode_status()


# ============================================================================
# Main
# ============================================================================

def print_banner():
    """打印启动横幅"""
    print("\n" + "=" * 50)
    print("      XHS API 客户端演示")
    print("      (Pure Rust Architecture v2.0)")
    print("=" * 50 + "\n")


def check_session() -> bool:
    """检查现有 Session 是否有效"""
    print("\n[检查] 验证现有 Session...")
    return test_user_me()


def test_all_apis():
    """测试所有 API"""
    print("\n" + "=" * 50)
    print("  开始测试所有 API 端点")
    print("=" * 50)
    
    # User
    test_user_me()
    
    # Search
    test_trending()
    test_search_recommend()
    sid = test_search_notes()
    test_search_onebox(sid)
    test_search_user(sid)
    test_search_filter(sid)
    
    # Feed
    test_homefeed()
    
    # Notifications
    test_notifications()
    
    # Category Feeds
    test_category_feeds()
    
    # Notes
    test_note_page()
    test_note_detail()
    
    # Pagination Test (分页测试)
    test_homefeed_pagination()
    
    # Media Test (媒体采集测试)
    test_media()
    
    print("\n" + "=" * 50)
    print("  ✅ 所有 API 测试完成")
    print("=" * 50)


def main():
    """主入口"""
    print_banner()
    
    # Check session
    if check_session():
        print("\n    Session 有效，跳过登录")
        test_all_apis()
        return
    
    print("\n    需要登录")
    
    if login_flow():
        time.sleep(2)
        test_all_apis()
    else:
        print("\n❌ 登录失败，无法测试 API")


if __name__ == "__main__":
    main()
