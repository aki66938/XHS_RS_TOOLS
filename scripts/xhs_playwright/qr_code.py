"""
QR Code extraction and ASCII conversion utilities
"""

import base64
import io
from typing import Optional

# Optional dependencies for QR decoding
try:
    from PIL import Image
    from pyzbar.pyzbar import decode as pyzbar_decode
    HAS_PYZBAR = True
except ImportError:
    HAS_PYZBAR = False

try:
    import qrcode
    HAS_QRCODE = True
except ImportError:
    HAS_QRCODE = False

from .config import QR_SELECTORS, QR_POLL_INTERVAL_MS, QR_MAX_ATTEMPTS


def base64_to_ascii(base64_data: str) -> str:
    """
    Convert base64 PNG QR code to ASCII art.
    
    Uses pyzbar to decode QR content, then regenerates as ASCII.
    Falls back to simple representation if dependencies missing.
    
    Args:
        base64_data: Base64 encoded PNG image data
        
    Returns:
        ASCII art representation of QR code
    """
    if not base64_data:
        return "[无法获取二维码]"
    
    # Clean base64 data
    if ',' in base64_data:
        base64_data = base64_data.split(',')[1]
    
    try:
        # Decode base64 to image
        img_data = base64.b64decode(base64_data)
        
        if HAS_PYZBAR:
            # Try to decode QR content
            img = Image.open(io.BytesIO(img_data))
            decoded = pyzbar_decode(img)
            
            if decoded and HAS_QRCODE:
                # Regenerate as ASCII QR
                qr_content = decoded[0].data.decode('utf-8')
                qr = qrcode.QRCode(
                    version=1,
                    error_correction=qrcode.constants.ERROR_CORRECT_L,
                    box_size=1,
                    border=1
                )
                qr.add_data(qr_content)
                qr.make(fit=True)
                
                # Convert to ASCII
                modules = qr.get_matrix()
                lines = []
                for row in modules:
                    line = ""
                    for cell in row:
                        line += "█" if cell else " "
                    lines.append(line)
                return "\n".join(lines)
        
        # Fallback: simple ASCII placeholder
        return _simple_ascii_placeholder()
        
    except Exception as e:
        print(f"[QR] ASCII conversion failed: {e}")
        return _simple_ascii_placeholder()


def _simple_ascii_placeholder() -> str:
    """Generate a simple placeholder ASCII art"""
    return """
╔═══════════════════════════════╗
║                               ║
║   请使用小红书APP扫描二维码    ║
║   (在浏览器窗口中查看)         ║
║                               ║
╚═══════════════════════════════╝
"""


async def extract_from_page(page) -> dict:
    """
    Extract QR code from the login modal on page.
    
    Uses img.qrcode-img selector to find QR image element.
    Returns the full data:image URI for client-side processing.
    
    Args:
        page: Playwright page object
        
    Returns:
        dict with 'base64', 'ascii', 'success' keys
    """
    result = {
        "success": False,
        "base64": "",
        "ascii": "",
        "error": None
    }
    
    for attempt in range(QR_MAX_ATTEMPTS):
        try:
            # Wait for QR image element (img.qrcode-img)
            qr_img = page.locator(QR_SELECTORS["qr_image"])
            await qr_img.wait_for(timeout=5000, state="visible")
            
            # Get src attribute
            src = await qr_img.get_attribute("src", timeout=3000)
            
            # Basic validation: Check if it's a data URI and has sufficient length
            # A real QR code base64 string is usually > 2000 chars
            # Loading placeholders are usually much smaller
            if src and src.startswith("data:image") and len(src) > 2000:
                # Return full data URI, client will handle parsing
                result["base64"] = src
                result["ascii"] = base64_to_ascii(src)
                result["success"] = True
                return result
            else:
                # If found but too small, it might be loading or a placeholder
                if src and len(src) <= 2000:
                    print(f"[QR] 忽略无效图片 (长度: {len(src)})")
                
                # Wait before retrying
                if attempt < QR_MAX_ATTEMPTS - 1:
                    await page.wait_for_timeout(QR_POLL_INTERVAL_MS)
                
        except Exception as e:
            if attempt < QR_MAX_ATTEMPTS - 1:
                await page.wait_for_timeout(QR_POLL_INTERVAL_MS)
            else:
                result["error"] = str(e)
    
    return result

