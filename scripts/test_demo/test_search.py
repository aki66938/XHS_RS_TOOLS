"""
Search API Tests
"""
import time
import urllib.parse
import urllib.request
import json
from .base import BASE_URL, print_success, print_warning, print_error


def test_trending():
    """测试热搜 API"""
    print("\n[API] GET /api/search/trending")
    try:
        req = urllib.request.Request(f"{BASE_URL}/api/search/trending")
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            queries = data.get("data", {}).get("queries", [])
            print_success("热搜 (Top 3):")
            for q in queries[:3]:
                print(f"       - {q.get('title', q.get('search_word', 'N/A'))}")
        else:
            print_warning(data.get("msg", "无数据"))
    except Exception as e:
        print_error(f"Error: {e}")


def test_search_recommend():
    """测试搜索推荐 API"""
    print("\n[API] GET /api/search/recommend (搜索推荐)")
    try:
        keyword = "湖州"
        encoded_kw = urllib.parse.quote(keyword)
        req = urllib.request.Request(f"{BASE_URL}/api/search/recommend?keyword={encoded_kw}")
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            items = data.get("data", {}).get("sug_items", [])
            print_success(f"获取搜索推荐成功 (关键词: {keyword}, 结果数: {len(items)})")
            for i, item in enumerate(items[:3], 1):
                print(f"       [{i}] {item.get('text', 'N/A')} ({item.get('type', 'unknown')})")
        else:
            print_warning(data.get("msg", "无数据"))
    except Exception as e:
        print_error(f"Error: {e}")


import random
import string

# 统一的测试关键词，用于所有相关搜索测试
TEST_KEYWORD = "鬼灭之刃"

def generate_simple_search_id():
    """生成简单 search_id (格式: 2fvzx + 16位随机字符)"""
    suffix = ''.join(random.choices(string.ascii_lowercase + string.digits, k=16))
    return f"2fvzx{suffix}"

def generate_search_id():
    """生成 search_id (格式: 2fvzx + 16位随机字符, 共21位)"""
    suffix = ''.join(random.choices(string.ascii_lowercase + string.digits, k=16))
    return f"2fvzx{suffix}"

def test_search_notes() -> str:
    """测试搜索笔记 API，使用 2fvzx 格式的 search_id"""
    print("\n[API] POST /api/search/notes (搜索笔记)")
    
    # 使用统一格式的 search_id
    test_search_id = generate_search_id()
    test_keyword = TEST_KEYWORD
    
    print(f"    Testing with Search ID: {test_search_id}")
    
    try:
        payload = {
            "keyword": test_keyword,
            "page": 1,
            "page_size": 20,
            "search_id": test_search_id,  # 使用固定的 search_id
            "sort": "general",
            "note_type": 0,
            "ext_flags": [],
            "filters": [
                {"tags": ["general"], "type": "sort_type"},
                {"tags": ["不限"], "type": "filter_note_type"},
                {"tags": ["不限"], "type": "filter_note_time"},
                {"tags": ["不限"], "type": "filter_note_range"},
                {"tags": ["不限"], "type": "filter_pos_distance"}
            ],
            "geo": "",
            "image_formats": ["jpg", "webp", "avif"]
        }
        data_json = json.dumps(payload).encode('utf-8')
        req = urllib.request.Request(
            f"{BASE_URL}/api/search/notes", 
            data=data_json, 
            headers={'Content-Type': 'application/json'}
        )
        with urllib.request.urlopen(req, timeout=20) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            items = data.get("data", {}).get("items", [])
            # 从响应中提取服务端生成的 search_id
            search_id = data.get("data", {}).get("search_id", "")
            print_success(f"获取搜索笔记成功 (ID: {search_id}, 结果: {len(items)})")
        else:
            print_warning(data.get("msg", "无数据"))
    except Exception as e:
        print_error(f"Error: {e}")
    return test_search_id  # 返回固定的 search_id 供后续测试



def test_search_onebox(search_id: str):
    """测试 OneBox API"""
    print("\n[API] POST /api/search/onebox")
    if not search_id:
        print_warning("跳过 (无 search_id，需先成功调用 search/notes)")
        return
    try:
        payload = {
            "keyword": "鬼灭之刃",
            "search_id": search_id,
            "biz_type": "web_search_user"
        }
        data_json = json.dumps(payload).encode('utf-8')
        req = urllib.request.Request(
            f"{BASE_URL}/api/search/onebox", 
            data=data_json, 
            headers={'Content-Type': 'application/json'}
        )
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        print_success(f"OneBox 调用完成: {data.get('msg')}, Success: {data.get('success')}")
    except Exception as e:
        print_error(f"Error: {e}")


def test_search_user(search_id: str):
    """测试用户搜索 API"""
    if not search_id:
        return
    print("\n[API] POST /api/search/usersearch (搜索用户)")
    try:
        payload = {
            "keyword": "鬼灭之刃",
            "search_id": search_id,
            "page": 1,
            "page_size": 15,
            "biz_type": "web_search_user"
        }
        data_json = json.dumps(payload).encode('utf-8')
        req = urllib.request.Request(
            f"{BASE_URL}/api/search/usersearch", 
            data=data_json, 
            headers={'Content-Type': 'application/json'}
        )
        with urllib.request.urlopen(req, timeout=10) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            users = data.get("data", {}).get("users", [])
            print_success(f"获取用户列表成功 (Count: {len(users)})")
            if users:
                user = users[0]
                print(f"       [1] {user.get('name')} (红薯号: {user.get('red_id')})")
        else:
            print_warning(data.get("msg", "无数据"))
    except Exception as e:
        print_error(f"Error: {e}")


def test_search_filter(search_id: str):
    """测试 Filter API"""
    if not search_id:
        return
    print("\n[API] GET /api/search/filter")
    try:
        keyword = urllib.parse.quote(TEST_KEYWORD)
        sid = urllib.parse.quote(search_id)
        url = f"{BASE_URL}/api/search/filter?keyword={keyword}&search_id={sid}"
        with urllib.request.urlopen(url, timeout=10) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            filters = data.get("data", {}).get("filters", [])
            print_success(f"获取筛选器成功 (Filter Count: {len(filters)})")
        else:
            print_warning(data.get("msg", "无数据"))
    except Exception as e:
        print_error(f"Error: {e}")
