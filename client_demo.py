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
import argparse

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
# Login Flow Helpers
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


def run_login_flow():
    """Run full user login flow (Init, QR, Poll)"""
    # Check session first
    if check_session():
        print("\n    Session 有效，跳过登录")
        return True
    
    print("\n    需要登录")
    if guest_init():
        if create_qrcode():
            if poll_qrcode_status():
                return True
    
    print("\n❌ 登录失败")
    return False


# ============================================================================
# Creator Login Flow
# ============================================================================

def test_creator_login():
    """测试创作者中心登录接口"""
    print("\n" + "=" * 50)
    print("  测试创作者中心登录 (Creator Center Login)")
    print("=" * 50)
    
    # Step 1: Creator Guest Init
    print("\n[1/3] 初始化创作者访客会话 (/api/creator/auth/guest-init)...")
    cookies = {}
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/creator/auth/guest-init", method='POST')
        with urllib.request.urlopen(req, timeout=60) as response:
            data = json.loads(response.read().decode('utf-8'))
            
        if data.get("success"):
            cookies = data.get("cookies", {})
            print(f"    ✅ 获取 ugc 访客 Cookie 成功 (数量: {len(cookies)})")
            if 'xsecappid' in cookies:
                print(f"    Context check: xsecappid={cookies['xsecappid']}")
        else:
            print(f"    ❌ 失败: {data.get('error')}")
            return
    except Exception as e:
        print(f"    ❌ 错误: {e}")
        return

    # Step 2: Create Creator QR Code
    print("\n[2/3] 创建创作者登录二维码 (/api/creator/auth/qrcode/create)...")
    qr_id = None
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/creator/auth/qrcode/create", method='POST')
        req.add_header('Content-Type', 'application/json')
        # Fix: Wrap cookies to match CreatorQrcodeCreateRequest schema
        body = json.dumps({"cookies": cookies}).encode('utf-8')
        
        with urllib.request.urlopen(req, data=body, timeout=30) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            qr_url = data.get("qr_url")
            qr_id = data.get("qr_id")
            
            print(f"    ✅ 二维码创建成功")
            print(f"    QR ID: {qr_id}")
            print(f"    URL: {qr_url}")
            
            if HAS_QRCODE and qr_url:
                print("\n" + "=" * 50)
                print("  请使用小红书 App 扫描以下二维码 (创作者中心):")
                print("=" * 50)
                qr = qrcode.QRCode(border=1)
                qr.add_data(qr_url)
                qr.print_ascii(invert=True)
                print("=" * 50)
            else:
                print(f"    扫码链接: {qr_url}")
                
                
            # print("\n⚠️  注意: 请立刻手动扫码，并在浏览器 F12 中捕获轮询请求 (status)！")
            
        else:
            print(f"    ❌ 失败: {data.get('error')}")
            return 
    except Exception as e:
        print(f"    ❌ 错误: {e}")
        return

    # Step 3: Polling Logic
    if not qr_id:
        print("\n    ❌ 无法轮询: 缺少 qr_id")
        return

    print("\n[3/3] 等待扫码登录 (Polling /api/creator/auth/qrcode/status)...")
    print("    ", end="", flush=True)
    
    start_time = time.time()
    last_status = -1
    
    while time.time() - start_time < 120:
        try:
            req = urllib.request.Request(f"{BASE_URL}/api/creator/auth/qrcode/status", method='POST')
            req.add_header('Content-Type', 'application/json')
            poll_payload = json.dumps({"qr_id": qr_id, "cookies": cookies}).encode('utf-8')
            
            with urllib.request.urlopen(req, data=poll_payload, timeout=30) as response:
                poll_data = json.loads(response.read().decode('utf-8'))
            
            if poll_data.get("success"):
                inner_data = poll_data.get("data", {})
                
                # Retrieve status from inner data
                status = None
                if isinstance(inner_data, dict):
                    status = inner_data.get("status")

                if status == 2: # Waiting
                     if last_status != 2:
                        print(".", end="", flush=True)
                        last_status = 2
                elif status == 3: # Scanned
                     if last_status != 3:
                        print("\n    ✓ 已扫码，等待确认...", end="", flush=True)
                        last_status = 3
                elif status == 1: # Success (Login Confirmed)
                    print("\n")
                    print(f"    ✅ 登录成功! (Status: {status})")
                    
                    # Check for synced cookies
                    new_cookies = poll_data.get("new_cookies", {})
                    if new_cookies:
                        print(f"    🍪 获取新 Cookie: {len(new_cookies)} 个 (已同步 User Session)")
                        # Optional: Print some key cookies to verify
                        if 'a1' in new_cookies:
                             print(f"       => Found 'a1': {new_cookies['a1'][:8]}...")
                    else:
                        print("    ⚠️  未发现新 Cookie，同步可能未成或未返回")
                        
                    print(f"    完整响应: {json.dumps(poll_data, indent=2, ensure_ascii=False)}")
                    return
                elif status is not None:
                    # Known success or other state
                    print("\n")
                    print(f"    ✅ 状态变更: {status}")
                    print(f"    完整响应: {json.dumps(poll_data, indent=2, ensure_ascii=False)}")
                    return
                else:
                     print("?", end="", flush=True)
            else:
                 print("x", end="", flush=True)
                 
        except Exception:
            print("!", end="", flush=True)
        
        time.sleep(2)

    print("\n    ❌ 登录超时")



def test_creator_apis():
    """测试创作者中心 API (User Info, Home Info)"""
    print("\n" + "=" * 50)
    print("  测试创作者中心 API (Creator Center APIs)")
    print("=" * 50)
    
    # 1. User Info
    print("\n[1/2] 获取用户信息 (/api/galaxy/user/info)...")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/galaxy/user/info")
        with urllib.request.urlopen(req, timeout=30) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            info = data.get("data", {})
            print(f"    ✅ 成功")
            print(f"    User Name: {info.get('userName')}")
            print(f"    User ID: {info.get('userId')}")
            print(f"    Role: {info.get('role')}")
            print(f"    Permissions: {len(info.get('permissions', []))} items")
        else:
            print(f"    ❌ 失败: {data.get('error')}")
    except Exception as e:
        print(f"    ❌ 请求错误: {e}")

    # 2. Home Info
    print("\n[2/2] 获取主页信息 (/api/galaxy/creator/home/personal_info)...")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/galaxy/creator/home/personal_info")
        with urllib.request.urlopen(req, timeout=30) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            info = data.get("data", {})
            print(f"    ✅ 成功")
            print(f"    Name: {info.get('name')}")
            print(f"    Fans: {info.get('fans_count')}")
            print(f"    Likes: {info.get('faved_count')}")
            print(f"    Desc: {info.get('personal_desc')}")
        else:
            print(f"    ❌ 失败: {data.get('error')}")
    except Exception as e:
        print(f"    ❌ 请求错误: {e}")

# ============================================================================
# Main Helper
# ============================================================================

def print_banner():
    """打印启动横幅"""
    print("\n" + "=" * 50)
    print("      XHS API 客户端演示 (v2.1 Integrated)")
    print("      (Pure Rust Architecture)")
    print("=" * 50 + "\n")


def check_session() -> bool:
    """检查现有 Session 是否有效"""
    # Simply call test_user_me but capture output to avoid spam if just checking
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/user/me")
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode('utf-8'))
            return data.get("success", False)
    except:
        return False


def test_all_apis():
    """测试所有 API"""
    print("\n" + "=" * 50)
    print("  开始测试所有 API 端点")
    print("=" * 50)
    
    test_user_me()
    test_trending()
    test_search_recommend()
    sid = test_search_notes()
    test_search_onebox(sid)
    test_search_user(sid)
    test_search_filter(sid)
    test_homefeed()
    test_notifications()
    test_category_feeds()
    test_note_page()
    test_note_detail()
    test_homefeed_pagination()
    test_media()
    
    print("\n" + "=" * 50)
    print("  ✅ 所有 API 测试完成")
    print("=" * 50)


def interactive_menu():
    """Interactive CLI Menu"""
    while True:
        print("\n" + "-" * 30)
        print("  功能菜单:")
        print("  1. 用户登录 (User Login Flow)")
        print("  2. 创作者登录 (Creator Login Flow)")
        print("  3. 测试创作者接口 (Test Creator APIs)")
        print("  4. 测试所有用户接口 (Test All User APIs)")
        print("  5. 测试单个接口...")
        print("  0. 退出 (Exit)")
        print("-" * 30)
        
        choice = input("请选择 (0-5): ").strip()
        
        if choice == '0':
            print("再见!")
            sys.exit(0)
        elif choice == '1':
            run_login_flow()
        elif choice == '2':
            test_creator_login()
        elif choice == '3':
            test_creator_apis()
        elif choice == '4':
            if run_login_flow():
                test_all_apis()
        elif choice == '5':
            display_api_menu()
        else:
            print("无效选择，请重试")


def display_api_menu():
    """Sub-menu for individual APIs"""
    sid = ""
    while True:
        print("\n  接口测试菜单:")
        print("  1. User Me")
        print("  2. Search Trending")
        print("  3. Search Notes")
        print("  4. Home Feed")
        print("  5. Note Detail")
        print("  6. Media Download")
        print("  0. 返回上级")
        
        c = input("选择接口 (0-6): ").strip()
        if c == '0': return
        elif c == '1': test_user_me()
        elif c == '2': test_trending()
        elif c == '3': sid = test_search_notes()
        elif c == '4': test_homefeed()
        elif c == '5': test_note_detail()
        elif c == '6': test_media()
        else: print("无效选择")


def main():
    print_banner()
    
    parser = argparse.ArgumentParser(description="XHS API Client Demo")
    parser.add_argument("--test", choices=["user", "creator", "all"], help="运行指定测试集")
    args = parser.parse_args()
    
    if args.test == "creator":
        test_creator_login()
    elif args.test == "user":
        if run_login_flow():
            test_all_apis()
    elif args.test == "all":
        print("Running User tests...")
        if run_login_flow():
            test_all_apis()
        print("\nRunning Creator tests...")
        test_creator_login()
    else:
        # Default to interactive mode
        try:
            interactive_menu()
        except KeyboardInterrupt:
            print("\n操作取消，退出")
            sys.exit(0)


if __name__ == "__main__":
    main()
