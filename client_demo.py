#!/usr/bin/env python3
"""
XHS API 客户端演示 (Pure Rust Architecture v2.0)

演示完整的登录流程和所有 API 端点测试
"""
import urllib.request
import urllib.parse
import json
import time

# Optional: QR code display in terminal
try:
    import qrcode
    HAS_QRCODE = True
except ImportError:
    HAS_QRCODE = False

BASE_URL = "http://localhost:3005"


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
            with urllib.request.urlopen(req, timeout=10) as response:
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
# API Tests
# ============================================================================

def test_user_me():
    """测试用户信息 API"""
    print("\n[API] GET /api/user/me")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/user/me")
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            user = data.get("data", {})
            print(f"    ✅ 用户: {user.get('nickname')} (ID: {user.get('red_id')})")
            return True
        else:
            print(f"    ❌ {data.get('msg')}")
            return False
    except Exception as e:
        print(f"    ❌ Error: {e}")
        return False


def test_trending():
    """测试热搜 API"""
    print("\n[API] GET /api/search/trending")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/search/trending")
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            queries = data.get("data", {}).get("queries", [])
            print(f"    ✅ 热搜 (Top 3):")
            for q in queries[:3]:
                print(f"       - {q.get('title', q.get('search_word', 'N/A'))}")
            return True
        else:
            print(f"    ❌ {data.get('msg')}")
            return False
    except Exception as e:
        print(f"    ❌ Error: {e}")
        return False


def test_homefeed():
    """测试推荐流 API"""
    print("\n[API] POST /api/feed/homefeed/recommend")
    try:
        req_data = json.dumps({
            "cursor_score": "", "num": 5, "refresh_type": 1,
            "note_index": 0, "category": "homefeed_recommend"
        }).encode('utf-8')
        
        req = urllib.request.Request(
            f"{BASE_URL}/api/feed/homefeed/recommend",
            data=req_data, headers={'Content-Type': 'application/json'}, method='POST'
        )
        
        with urllib.request.urlopen(req, timeout=15) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            items = data.get("data", {}).get("items", [])
            print(f"    ✅ 获取推荐流成功 (共 {len(items)} 条)")
            for i, item in enumerate(items[:3]):
                if "note_card" in item:
                    note = item["note_card"]
                    title = note.get('display_title', note.get('title', '无标题'))[:25]
                    author = note.get('user', {}).get('nickname', '未知')
                    print(f"       [{i+1}] {title}... (作者: {author})")
            return True
        else:
            print(f"    ❌ {data.get('msg')}")
            return False
    except Exception as e:
        print(f"    ❌ Error: {e}")
        return False


def test_notifications():
    """测试通知 API"""
    endpoints = [
        ("/api/notification/mentions", "评论和@"),
        ("/api/notification/connections", "新增关注"),
        ("/api/notification/likes", "赞和收藏"),
    ]
    
    for path, name in endpoints:
        print(f"\n[API] GET {path} ({name})")
        try:
            req = urllib.request.Request(f"{BASE_URL}{path}")
            with urllib.request.urlopen(req, timeout=10) as response:
                data = json.loads(response.read().decode('utf-8'))
            
            if data.get("success"):
                messages = data.get("data", {}).get("message_list", [])
                print(f"    ✅ 获取成功 (消息数: {len(messages)})")
            else:
                print(f"    ⚠️ {data.get('msg', '无数据')}")
        except Exception as e:
            print(f"    ❌ Error: {e}")


def test_category_feeds():
    """测试所有分类 Feed API"""
    categories = [
        ("fashion", "穿搭"), ("food", "美食"), ("cosmetics", "彩妆"),
        ("movie_and_tv", "影视"), ("career", "职场"), ("love", "情感"),
        ("household_product", "家居"), ("gaming", "游戏"),
        ("travel", "旅行"), ("fitness", "健身"),
    ]
    
    for cat_key, cat_name in categories:
        print(f"\n[API] POST /api/feed/homefeed/{cat_key} ({cat_name})")
        try:
            req_data = json.dumps({
                "cursor_score": "", "num": 5, "refresh_type": 1,
                "note_index": 0, "category": f"homefeed.{cat_key}_v3"
            }).encode('utf-8')
            
            req = urllib.request.Request(
                f"{BASE_URL}/api/feed/homefeed/{cat_key}",
                data=req_data, headers={'Content-Type': 'application/json'}
            )
            
            with urllib.request.urlopen(req, timeout=15) as response:
                data = json.loads(response.read().decode('utf-8'))
            
            if data.get("success"):
                items = data.get("data", {}).get("items", [])
                print(f"    ✅ 获取{cat_name}成功 (共 {len(items)} 条)")
            else:
                print(f"    ⚠️ {data.get('msg', '无数据')}")
        except Exception as e:
            print(f"    ❌ Error: {e}")


def test_note_page():
    """测试图文详情 API"""
    print("\n[API] GET /api/note/page (图文详情)")
    try:
        test_note_id = "695f0f1d00000000210317c5"
        test_xsec_token = "ABSWQGp8zRp5VzyF6DXyPCnEsSakbUyTGAP3_so8877G4="
        
        params = urllib.parse.urlencode({
            "note_id": test_note_id, "cursor": "",
            "xsec_token": test_xsec_token, "image_formats": "jpg,webp,avif"
        })
        
        req = urllib.request.Request(f"{BASE_URL}/api/note/page?{params}")
        with urllib.request.urlopen(req, timeout=15) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            comments = data.get("data", {}).get("comments", [])
            print(f"    ✅ 获取图文详情成功 (评论数: {len(comments)})")
            for i, comment in enumerate(comments[:3]):
                content = comment.get('content', '无内容')[:30]
                user = comment.get('user_info', {}).get('nickname', '未知')
                print(f"       [{i+1}] {user}: {content}...")
        else:
            print(f"    ⚠️ {data.get('msg', '无数据')}")
    except Exception as e:
        print(f"    ❌ Error: {e}")


def test_all_apis():
    """测试所有 API"""
    print("\n" + "=" * 50)
    print("  开始测试所有 API 端点")
    print("=" * 50)
    
    test_user_me()
    test_trending()
    test_homefeed()
    test_notifications()
    test_category_feeds()
    test_note_page()
    
    print("\n" + "=" * 50)
    print("  ✅ 所有 API 测试完成")
    print("=" * 50)


# ============================================================================
# Main
# ============================================================================

def main():
    print("\n" + "=" * 50)
    print("      XHS API 客户端演示")
    print("      (Pure Rust Architecture v2.0)")
    print("=" * 50)
    
    print("\n[检查] 验证现有 Session...")
    if test_user_me():
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
