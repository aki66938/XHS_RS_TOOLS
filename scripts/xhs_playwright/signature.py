"""
Signature capture for XHS API endpoints
"""

from datetime import datetime
from typing import Callable, Dict, Any

from .config import ENDPOINT_PATTERNS
from .storage import save_signature


class SignatureCapture:
    """
    Captures XHS API signatures from browser network requests.
    
    Usage:
        capture = SignatureCapture()
        page.on("request", capture.create_request_handler())
        # ... trigger API calls ...
        capture.save_all_signatures()
    """
    
    def __init__(self):
        self.x_s_common: str = ""
        self.signatures: Dict[str, Dict[str, Any]] = {}
        self._log_file = "debug_login.log"
    
    @property
    def captured_endpoints(self) -> list:
        """List of endpoint names that have been captured"""
        return list(self.signatures.keys())
    
    def create_request_handler(self) -> Callable:
        """
        Create a request handler function for Playwright page.on("request", ...).
        
        Returns:
            Async function to handle network requests
        """
        async def handle_request(request):
            url = request.url
            headers = request.headers
            
            # Capture x_s_common from any XHS API request
            if 'x-s-common' in headers:
                if len(headers['x-s-common']) > len(self.x_s_common):
                    self.x_s_common = headers['x-s-common']
            
            # Check if this matches any endpoint we care about
            for endpoint, pattern in ENDPOINT_PATTERNS.items():
                if pattern in url and 'x-s' in headers:
                    # Get POST body if present
                    post_data = request.post_data if request.method == "POST" else None
                    
                    # Special handling for home_feed to distinguish categories through payload
                    storage_endpoint = endpoint
                    if endpoint == "home_feed" and post_data:
                        try:
                            import json
                            body = json.loads(post_data)
                            category = body.get("category")
                            if category:
                                # Map internal category strings to our keys
                                # e.g. "homefeed.fashion_v3" -> "home_feed_fashion"
                                # or just use the raw category: "home_feed_homefeed_fashion_v3"
                                # Simple mapping for standard categories:
                                if category == "homefeed_recommend":
                                    storage_endpoint = "home_feed_recommend"
                                else:
                                    # Safe fallback: sanitize category string
                                    safe_cat = category.split('.')[0] if '.' in category else category
                                    # Handle "homefeed.fashion_v3" style
                                    if '.' in category:
                                        parts = category.split('.')
                                        if len(parts) > 1:
                                            # "fashion_v3" -> "fashion"
                                            # "movie_and_tv_v3" -> "movie_and_tv" (Fix: Handle multiple underscores)
                                            raw_cat = parts[1]
                                            if raw_cat.endswith("_v3"):
                                                raw_cat = raw_cat[:-3]
                                            else:
                                                # Fallback to old behavior only if no version suffix
                                                raw_cat = raw_cat.split('_')[0]
                                            
                                            storage_endpoint = f"home_feed_{raw_cat}"
                                    else:
                                        storage_endpoint = f"home_feed_{category}"
                                    
                            self._log(f"Detected feed category: {category} -> {storage_endpoint}")
                        except Exception as e:
                            self._log(f"Failed to parse feed body: {e}")

                    self.signatures[storage_endpoint] = {
                        "x-s": headers.get("x-s", ""),
                        "x-t": headers.get("x-t", ""),
                        "x-s-common": headers.get("x-s-common", ""),
                        "x-b3-traceid": headers.get("x-b3-traceid", ""),
                        "x-xray-traceid": headers.get("x-xray-traceid", ""),
                        "method": request.method,
                        "post_body": post_data,
                    }
                    
                    # Log capture
                    self._log(f"Captured signature for {storage_endpoint}")
                    break
        
        return handle_request
    
    def _log(self, message: str):
        """Write debug log"""
        try:
            with open(self._log_file, "a") as f:
                f.write(f"[{datetime.now()}] {message}\n")
        except:
            pass
    
    def save_all_signatures(self) -> list:
        """
        Save all captured signatures to MongoDB.
        
        Returns:
            List of saved endpoint names
        """
        saved = []
        for endpoint, sig_data in self.signatures.items():
            save_signature(endpoint, sig_data)
            saved.append(endpoint)
        return saved
    
    def get_signature(self, endpoint: str) -> dict:
        """Get captured signature for a specific endpoint"""
        return self.signatures.get(endpoint, {})
    
    def has_signature(self, endpoint: str) -> bool:
        """Check if signature exists for endpoint"""
        return endpoint in self.signatures
