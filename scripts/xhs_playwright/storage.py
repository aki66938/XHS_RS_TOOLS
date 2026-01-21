"""
JSON file storage operations for credentials
"""

import json
from datetime import datetime, timezone
from pathlib import Path

# Cookie file path (project root)
COOKIE_FILE = Path(__file__).parent.parent.parent / "cookie.json"


def save_credentials(cookies: dict) -> str:
    """
    Save user credentials to cookie.json.
    
    Args:
        cookies: Dictionary of cookies from browser
        
    Returns:
        The user_id from cookies
    """
    # Build user ID from cookie values (same logic as before)
    user_id = cookies.get("web_session", "")[:24] or "unknown"
    
    # Prepare document
    now = datetime.now(timezone.utc).isoformat()
    doc = {
        "user_id": user_id,
        "cookies": cookies,
        "x_s_common": None,
        "created_at": now,
        "updated_at": now,
        "is_valid": True
    }
    
    # Write to JSON file
    with open(COOKIE_FILE, 'w', encoding='utf-8') as f:
        json.dump(doc, f, indent=2, ensure_ascii=False)
    
    return user_id


def invalidate_all_credentials() -> int:
    """
    Invalidate all stored credentials.
    
    Returns:
        1 if credentials were invalidated, 0 otherwise
    """
    if not COOKIE_FILE.exists():
        return 0
    
    with open(COOKIE_FILE, 'r', encoding='utf-8') as f:
        doc = json.load(f)
    
    if doc.get("is_valid"):
        doc["is_valid"] = False
        doc["updated_at"] = datetime.now(timezone.utc).isoformat()
        
        with open(COOKIE_FILE, 'w', encoding='utf-8') as f:
            json.dump(doc, f, indent=2, ensure_ascii=False)
        
        return 1
    
    return 0
