import urllib.request
import json
import time
import base64
import sys

# Optional dependencies
try:
    import cv2
    import numpy as np
    HAS_OPENCV = True
except ImportError:
    HAS_OPENCV = False

try:
    from PIL import Image
    from pyzbar.pyzbar import decode as pyzbar_decode
    import io
    HAS_PYZBAR = True
except ImportError:
    HAS_PYZBAR = False

try:
    import qrcode
    HAS_QRCODE = True
except ImportError:
    HAS_QRCODE = False

BASE_URL = "http://localhost:3005"

def check_existing_session():
    """检查是否存在有效Session"""
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/auth/session")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            if data.get("success") and data.get("data", {}).get("is_valid"):
                return data["data"]
            return None
    except Exception as e:
        print(f"检查Session失败: {e}")
        return None

def validate_session():
    """验证Session有效性并获取用户信息"""
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/user/me")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            if data.get("success"):
                return data["data"]
            return None
    except:
        return None

def check_qrcode_status():
    """检查二维码登录状态"""
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/auth/qrcode-status")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            if data.get("success") and data.get("data"):
                return data["data"]
            return None
    except:
        return None

def check_capture_status():
    """检查采集任务状态"""
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/auth/capture-status")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            if data.get("success") and data.get("data"):
                return data["data"]
            return None
    except:
        return None

def login_interactive():
    """交互式登录流程"""
    print("启动登录会话...")
    session_url = f"{BASE_URL}/api/auth/login-session"
    
    try:
        req = urllib.request.Request(session_url, method='POST')
        with urllib.request.urlopen(req, timeout=60) as response:
            data = json.loads(response.read().decode('utf-8'))
            
            if data.get("success") and data.get("qr_base64"):
                b64_data = data["qr_base64"]
                
                # Clean base64 string
                if "," in b64_data:
                    b64_data = b64_data.split(",")[1]
                
                # Decode QR using available libraries
                # Priority: pyzbar > cv2 (pyzbar is generally more robust for QR)
                qr_content = None

                if HAS_PYZBAR:
                    try:
                        img_bytes = base64.b64decode(b64_data)
                        img = Image.open(io.BytesIO(img_bytes))
                        decoded = pyzbar_decode(img)
                        if decoded:
                            qr_content = decoded[0].data.decode('utf-8')
                    except Exception as e:
                        print(f"⚠️ pyzbar解码错误: {e}")

                if not qr_content and HAS_OPENCV:
                     try:
                        img_bytes = base64.b64decode(b64_data)
                        nparr = np.frombuffer(img_bytes, np.uint8)
                        cv_img = cv2.imdecode(nparr, cv2.IMREAD_COLOR)
                        
                        detector = cv2.QRCodeDetector()
                        
                        # Strategies to try
                        strategies = [
                            ("Raw", lambda i: i),
                            ("Sharpen", lambda i: cv2.filter2D(i, -1, np.array([[-1,-1,-1], [-1,9,-1], [-1,-1,-1]]))),
                            ("Resize 2x", lambda i: cv2.resize(i, None, fx=2, fy=2, interpolation=cv2.INTER_CUBIC)),
                            ("Gray", lambda i: cv2.cvtColor(i, cv2.COLOR_BGR2GRAY)),
                            ("Binary", lambda i: cv2.threshold(cv2.cvtColor(i, cv2.COLOR_BGR2GRAY), 128, 255, cv2.THRESH_BINARY)[1]),
                        ]
                        
                        for name, func in strategies:
                            try:
                                processed = func(cv_img)
                                qr_content, _, _ = detector.detectAndDecode(processed)
                                if qr_content:
                                    # print(f"  >> QR解码成功 (策略: {name})")
                                    break
                            except:
                                continue

                     except Exception as e:
                        print(f"⚠️ OpenCV解码错误: {e}")
                
                if qr_content and HAS_QRCODE:
                    print("\n" + "="*50)
                    print("  扫描以下二维码登录小红书:")
                    print("="*50)
                    
                    qr = qrcode.QRCode(border=1)
                    qr.add_data(qr_content)
                    qr.print_ascii()
                    
                    print("="*50 + "\n")
                else:
                    print("⚠️ 无法在终端解析二维码 (但登录继续)")
                    # Save QR image for manual scanning
                    try:
                        with open("qr_login.png", "wb") as f:
                            f.write(base64.b64decode(b64_data))
                        print("  >> 已保存二维码图片至: qr_login.png")
                        print("  >> 请打开图片进行扫描，或查看自动打开的浏览器窗口")
                    except Exception as e:
                        print(f"  >> 保存二维码图片失败: {e}")
                
                # 阶段 1: 轮询 qrcode-status 等待登录成功
                print("\n等待扫码", end="", flush=True)
                start_time = time.time()
                timeout = 120
                login_success = False
                
                while time.time() - start_time < timeout:
                    qr_status = check_qrcode_status()
                    if qr_status:
                        code_status = qr_status.get("code_status", -1)
                        if code_status == 2:  # 登录成功
                            login_info = qr_status.get("login_info", {})
                            print(f"\n\n✅ 登录成功！")
                            print(f"   User ID: {login_info.get('user_id', 'N/A')}")
                            login_success = True
                            break
                        elif code_status == 1:  # 已扫码
                            print("✓", end="", flush=True)
                        else:  # 未扫码
                            print(".", end="", flush=True)
                    else:
                        print(".", end="", flush=True)
                    time.sleep(1)
                
                if not login_success:
                    print("\n❌ 登录超时")
                    return False
                
                # 阶段 2: 等待采集完成（无限轮询直到服务端返回完成）
                print("\n等待签名采集", end="", flush=True)
                
                while True:
                    capture = check_capture_status()
                    if capture:
                        if capture.get("is_complete"):
                            count = capture.get("total_count", 0)
                            message = capture.get("message", "")
                            
                            if count > 0:
                                # 采集成功
                                print(f"\n\n✅ 采集完成！")
                                print(f"   签名数量: {count}")
                                print(f"   已采集: {', '.join(capture.get('signatures_captured', []))}")
                                
                                # 阶段 3: 获取用户信息
                                user = validate_session()
                                if user:
                                    print(f"   用户: {user.get('nickname')} (ID: {user.get('red_id')})")
                                return True
                            else:
                                # 采集失败 (total_count == 0)
                                print(f"\n\n❌ 采集失败！")
                                print(f"   原因: {message}")
                                return False
                        # 显示当前采集进度
                        current_count = capture.get("total_count", 0)
                        if current_count > 0:
                            print(f"({current_count})", end="", flush=True)
                        else:
                            print(".", end="", flush=True)
                    else:
                        print(".", end="", flush=True)
                    time.sleep(2)
            else:
                print(f"❌ API错误: {data}")
                return False
    except Exception as e:
        print(f"❌ 错误: {e}")
        return False

def test_all_apis():
    """测试所有API"""
    print("\n" + "="*50)
    print("  开始测试API接口")
    print("="*50)
    
    # 1. Session API
    print("\n[1] GET /api/auth/session")
    session = check_existing_session()
    if session:
        print(f"    ✅ Session 存在")
        print(f"    - Cookie 数量: {session.get('cookie_count')}")
        print(f"    - 有效状态: {session.get('is_valid')}")
    else:
        print("    ❌ 无有效 Session")

    # 2. User API
    print("\n[2] GET /api/user/me")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/user/me")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            if data.get("success"):
                user = data["data"]
                print(f"    ✅ 用户信息获取成功")
                print(f"    - 昵称: {user.get('nickname')}")
                print(f"    - Red ID: {user.get('red_id')}")
                print(f"    - 简介: {user.get('desc')[:20]}...")
            else:
                print(f"    ❌ 获取失败: {data.get('msg')}")
    except Exception as e:
        print(f"    ❌ Error: {e}")

    # 3. Trending API
    print("\n[3] GET /api/search/trending")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/search/trending")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            if data.get("success"):
                queries = data["data"]["queries"]
                print(f"    ✅ 获取热搜成功 (Top 3):")
                for q in queries[:3]:
                    print(f"    - {q.get('title', q.get('search_word', 'N/A'))}")
            else:
                print(f"    ⚠️ 无数据 (可能是签名问题)")
                print(f"      {data.get('msg')}")
    except Exception as e:
        print(f"    ❌ Error: {e}")
    
    # 4. Homefeed Recommend API
    print("\n[4] POST /api/feed/homefeed/recommend")
    try:
        req_data = json.dumps({
            "cursor_score": "",
            "num": 10,
            "refresh_type": 1,
            "note_index": 0,
            "category": "homefeed_recommend",
            "need_num": 10,
            "image_formats": ["jpg", "webp", "avif"]
        }).encode('utf-8')
        req = urllib.request.Request(
            f"{BASE_URL}/api/feed/homefeed/recommend",
            data=req_data,
            headers={'Content-Type': 'application/json'},
            method='POST'
        )
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            if data.get("success"):
                items = data["data"]["items"]
                print(f"    ✅ 获取推荐流成功 (Items: {len(items)})")
                
                # Show top 3 items
                for i, item in enumerate(items[:3]):
                    if "note_card" in item:
                        note = item["note_card"]
                        title = note.get('display_title', note.get('title', '无标题'))
                        author = note.get('user', {}).get('nickname', '未知作者')
                        print(f"    [{i+1}] {title[:20]}... (作者: {author})")
            else:
                 print(f"    ⚠️ 无数据 (可能是签名问题)")
                 print(f"      {data.get('msg')}")
    except Exception as e:
        print(f"    ❌ Error: {e}")

    # Test All Category Feeds
    # Mapping of keys to Chinese names (Keys must match internal signature naming)
    feed_tests = [
        ("fashion", "穿搭"),
        ("food", "美食"), 
        ("cosmetics", "彩妆"),
        ("movie_and_tv", "影视"), 
        ("career", "职场"),
        ("love", "情感"),               # Updated from emotion
        ("household_product", "家居"),  # Updated from home
        ("gaming", "游戏"),             # Updated from game
        ("travel", "旅行"),
        ("fitness", "健身"),
    ]

    for cat_key, cat_name in feed_tests:
        print(f"\n[+] POST /api/feed/homefeed/{cat_key} ({cat_name})")
        try:
            req_body = {
                "cursor_score": "",
                "num": 20,
                "refresh_type": 1,
                "note_index": 0,
                "unread_begin_note_id": "",
                "unread_end_note_id": "",
                "unread_note_count": 0,
                "category": f"homefeed.{cat_key}_v3" 
            }
            json_data = json.dumps(req_body).encode('utf-8')
            
            req = urllib.request.Request(
                f"{BASE_URL}/api/feed/homefeed/{cat_key}", 
                data=json_data,
                headers={'Content-Type': 'application/json'}
            )
            
            with urllib.request.urlopen(req) as response:
                data = json.loads(response.read().decode('utf-8'))
            
            if data.get("success"):
                items = data["data"]["items"]
                print(f"    ✅ 获取{cat_name}流成功 (Items: {len(items)})")
                for i, item in enumerate(items[:3]):
                    if "note_card" in item:
                        note = item["note_card"]
                        title = note.get('display_title', note.get('title', '无标题'))
                        author = note.get('user', {}).get('nickname', '未知作者')
                        print(f"    [{i+1}] {title[:20]}... (作者: {author})")
            else:
                 print(f"    ⚠️ 无数据 (可能是签名未采集)")
                 msg = data.get('msg')
                 if msg:
                    print(f"      {msg}")

        except Exception as e:
            print(f"    ❌ Error: {e}")
    
    # ------------------------------------------
    # [v1.1.0] Notification API Tests
    # ------------------------------------------
    print("\n[+] GET /api/notification/mentions (评论和@)")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/notification/mentions")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            messages = data.get("data", {}).get("message_list", [])
            print(f"    ✅ 获取评论和@成功 (消息数: {len(messages)})")
            for i, msg in enumerate(messages[:3]):
                title = msg.get('title', '未知')
                user = msg.get('user_info', {}).get('nickname', '未知')
                print(f"    [{i+1}] {title} (用户: {user})")
        else:
            print(f"    ⚠️ 无数据")
            msg = data.get('msg')
            if msg:
                print(f"      {msg}")
    except Exception as e:
        print(f"    ❌ Error: {e}")

    print("\n[+] GET /api/notification/connections (新增关注)")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/notification/connections")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            messages = data.get("data", {}).get("message_list", [])
            print(f"    ✅ 获取新增关注成功 (消息数: {len(messages)})")
            for i, msg in enumerate(messages[:3]):
                user = msg.get('user', {}).get('nickname', '未知')
                print(f"    [{i+1}] {user} 开始关注你")
        else:
            print(f"    ⚠️ 无数据")
            msg = data.get('msg')
            if msg:
                print(f"      {msg}")
    except Exception as e:
        print(f"    ❌ Error: {e}")
    
    # ------------------------------------------
    # [v1.2.0] Note Page API Test
    # ------------------------------------------
    print("\n[+] GET /api/note/page (图文详情)")
    try:
        # Use a sample note_id and xsec_token for testing
        # In real usage, these would come from a note detail page
        test_note_id = "695f0f1d00000000210317c5"
        test_xsec_token = "ABSWQGp8zRp5VzyF6DXyPCnEsSakbUyTGAP3_so8877G4="
        
        params = urllib.parse.urlencode({
            "note_id": test_note_id,
            "cursor": "",
            "xsec_token": test_xsec_token,
            "image_formats": "jpg,webp,avif"
        })
        
        req = urllib.request.Request(f"{BASE_URL}/api/note/page?{params}")
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            comments = data.get("data", {}).get("comments", [])
            print(f"    ✅ 获取图文详情成功 (评论数: {len(comments)})")
            for i, comment in enumerate(comments[:3]):
                content = comment.get('content', '无内容')[:30]
                user = comment.get('user_info', {}).get('nickname', '未知')
                print(f"    [{i+1}] {user}: {content}...")
        else:
            print(f"    ⚠️ 无数据")
            msg = data.get('msg')
            if msg:
                print(f"      {msg}")
    except Exception as e:
        print(f"    ❌ Error: {e}")
    
    print("\n" + "="*50)
    print("  ✅ API测试完成")
    print("="*50)

def main():
    print("\n" + "="*50)
    print("      XHS API 客户端演示")
    print("="*50 + "\n")
    
    print("[检查] 正在检查现有Session...")
    session = check_existing_session()
    
    if session:
        print(f"  ✅ 找到Session (Cookie数量: {session.get('cookie_count')})")
        print("\n[验证] 正在验证Session有效性...")
        user = validate_session()
        
        if user:
            print(f"  ✅ Session有效")
            print(f"     用户: {user.get('nickname')} (ID: {user.get('red_id')})")
            test_all_apis()
            return
        else:
            print("  ❌ Session已过期")
    else:
        print("  ❌ 无Session，需要登录")
    
    if login_interactive():
        test_all_apis()

if __name__ == "__main__":
    main()
