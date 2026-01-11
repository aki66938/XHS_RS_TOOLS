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
                
                # Continue to wait for login
                print("\n等待登录...", end="", flush=True)
                start_time = time.time()
                while time.time() - start_time < 120:
                    user = validate_session()
                    if user:
                        print(f"\n\n✅ 登录成功！")
                        print(f"   用户: {user.get('nickname')} (ID: {user.get('red_id')})")
                        return True
                    print(".", end="", flush=True)
                    time.sleep(2)
                
                print("\n❌ 登录超时")
                return False
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
