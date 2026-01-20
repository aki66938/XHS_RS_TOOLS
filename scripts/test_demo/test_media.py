"""
Media API Tests: video URL extraction, image URL extraction, media download
"""

import urllib.request
import json
import os
from .base import BASE_URL

# 固定测试用视频笔记 (Dillon是涤纶的视频)
VIDEO_NOTE_ID = "695f42df000000002102b712"
VIDEO_XSEC_TOKEN = "ABSWQGp8zRp5VzyF6DXyPCnHYqTVqYMCxRrQEjjM1wi9Q="

# 固定测试用图片笔记 (饼饼爱做饭的图文)
IMAGE_NOTE_ID = "69670c6b000000000a03e8ae"
IMAGE_XSEC_TOKEN = "AB-RzpMwiY0mkowlwcU5dThBNNrd9vP0RhN7_Msjov71w="


def test_video_urls():
    """测试视频地址解析"""
    print("\n[API] POST /api/note/video (视频地址解析)")
    
    try:
        payload = {
            "note_id": VIDEO_NOTE_ID,
            "xsec_token": VIDEO_XSEC_TOKEN
        }
        
        req = urllib.request.Request(
            f"{BASE_URL}/api/note/video",
            data=json.dumps(payload).encode('utf-8'),
            headers={'Content-Type': 'application/json'},
            method='POST'
        )
        
        with urllib.request.urlopen(req, timeout=30) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success") and data.get("data"):
            video_data = data["data"]
            videos = video_data.get("videos", [])
            
            print(f"    ✅ 获取视频地址成功")
            print(f"       标题: {video_data.get('title', 'N/A')}")
            print(f"       作者: {video_data.get('author', 'N/A')}")
            print(f"       时长: {video_data.get('duration', 0) // 1000}秒")
            print(f"       画质数量: {len(videos)}")
            
            for v in videos[:3]:
                print(f"       - {v['quality']}: {v['width']}x{v['height']} ({v['size'] // 1024 // 1024}MB)")
            
            # 返回最高画质的URL用于下载测试
            if videos:
                return videos[0].get("url")
            return None
        else:
            print(f"    ⚠️ {data.get('msg', 'Unknown error')}")
            return None
            
    except Exception as e:
        print(f"    ⚠️ {str(e)[:60]}")
        return None


def test_image_urls():
    """测试图片地址解析"""
    print("\n[API] POST /api/note/images (图片地址解析)")
    
    try:
        payload = {
            "note_id": IMAGE_NOTE_ID,
            "xsec_token": IMAGE_XSEC_TOKEN
        }
        
        req = urllib.request.Request(
            f"{BASE_URL}/api/note/images",
            data=json.dumps(payload).encode('utf-8'),
            headers={'Content-Type': 'application/json'},
            method='POST'
        )
        
        with urllib.request.urlopen(req, timeout=30) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success") and data.get("data"):
            img_data = data["data"]
            images = img_data.get("images", [])
            
            print(f"    ✅ 获取图片地址成功")
            print(f"       标题: {img_data.get('title', 'N/A')}")
            print(f"       作者: {img_data.get('author', 'N/A')}")
            print(f"       图片数量: {img_data.get('image_count', 0)}")
            
            for img in images[:3]:
                print(f"       - 图{img['index']}: {img['width']}x{img['height']}")
            
            # 返回第一张图片的无水印URL用于下载测试
            if images:
                return images[0].get("url_original")
            return None
        else:
            print(f"    ⚠️ {data.get('msg', 'Unknown error')}")
            return None
            
    except Exception as e:
        print(f"    ⚠️ {str(e)[:60]}")
        return None


def test_media_download(media_url: str, save_path: str):
    """测试媒体下载"""
    print(f"\n[API] POST /api/media/download ({save_path})")
    
    try:
        payload = {
            "url": media_url,
            "save_path": save_path
        }
        
        req = urllib.request.Request(
            f"{BASE_URL}/api/media/download",
            data=json.dumps(payload).encode('utf-8'),
            headers={'Content-Type': 'application/json'},
            method='POST'
        )
        
        # 下载可能需要较长时间
        with urllib.request.urlopen(req, timeout=300) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success") and data.get("data"):
            dl_data = data["data"]
            file_size = dl_data.get("file_size", 0)
            content_type = dl_data.get("content_type", "unknown")
            
            size_str = f"{file_size // 1024 // 1024}MB" if file_size > 1024*1024 else f"{file_size // 1024}KB"
            print(f"    ✅ 下载成功: {size_str} ({content_type})")
            return True
        else:
            print(f"    ⚠️ {data.get('msg', 'Unknown error')}")
            return False
            
    except Exception as e:
        print(f"    ⚠️ {str(e)[:60]}")
        return False


def test_media():
    """测试所有媒体 API"""
    print("\n" + "-" * 50)
    print("[Media] 媒体采集测试")
    print("-" * 50)
    
    # 1. 获取视频地址
    video_url = test_video_urls()
    
    # 2. 下载视频 (如果成功获取URL)
    if video_url:
        test_media_download(video_url, "./test_video_download.mp4")
    
    # 3. 获取图片地址
    image_url = test_image_urls()
    
    # 4. 下载图片 (如果成功获取URL)
    if image_url:
        test_media_download(image_url, "./test_image_download.webp")

