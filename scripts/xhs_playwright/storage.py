"""
MongoDB storage operations for credentials and signatures
"""

from datetime import datetime
from pymongo import MongoClient
from .config import MONGODB_URI, DATABASE_NAME


def save_credentials(cookies: dict, x_s_common: str = "") -> str:
    """
    Save user credentials to MongoDB.
    Invalidates all existing credentials before saving new ones.
    
    Args:
        cookies: Dictionary of cookies from browser
        x_s_common: The x-s-common header value
        
    Returns:
        The user_id from cookies
    """
    client = MongoClient(MONGODB_URI)
    db = client[DATABASE_NAME]
    collection = db["credentials"]
    
    # Invalidate all existing credentials
    collection.update_many({}, {"$set": {"is_valid": False}})
    
    # Build user ID from cookie values
    user_id = "".join(cookies.get(k, "") for k in ["a1", "webId", "gid", "web_session"])
    
    # Prepare document
    doc = {
        "user_id": user_id,
        "cookies": cookies,
        "x_s_common": x_s_common,
        "created_at": datetime.utcnow(),
        "updated_at": datetime.utcnow(),
        "is_valid": True
    }
    
    collection.insert_one(doc)
    client.close()
    
    return user_id


def save_signature(endpoint: str, signature_data: dict) -> None:
    """
    Save API endpoint signature to MongoDB.
    Uses upsert to update existing or insert new.
    
    Args:
        endpoint: Endpoint name (e.g., "user_me", "home_feed_recommend")
        signature_data: Dictionary with signature headers
    """
    client = MongoClient(MONGODB_URI)
    db = client[DATABASE_NAME]
    collection = db["api_signatures"]
    
    collection.update_one(
        {"endpoint": endpoint},
        {
            "$set": {
                "endpoint": endpoint,
                "x_s": signature_data.get("x-s", ""),
                "x_t": signature_data.get("x-t", ""),
                "x_s_common": signature_data.get("x-s-common", ""),
                "x_b3_traceid": signature_data.get("x-b3-traceid", ""),
                "x_xray_traceid": signature_data.get("x-xray-traceid", ""),
                "method": signature_data.get("method", "GET"),
                "post_body": signature_data.get("post_body", ""),
                "captured_at": datetime.utcnow(),
                "is_valid": True
            }
        },
        upsert=True
    )
    client.close()
    print(f"[MongoDB] Saved signature for endpoint: {endpoint}")


def invalidate_all_credentials() -> int:
    """
    Invalidate all stored credentials.
    
    Returns:
        Number of documents updated
    """
    client = MongoClient(MONGODB_URI)
    db = client[DATABASE_NAME]
    collection = db["credentials"]
    
    result = collection.update_many({}, {"$set": {"is_valid": False}})
    client.close()
    
    return result.modified_count
