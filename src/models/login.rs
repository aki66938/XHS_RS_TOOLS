use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "qr_id": "123456789012345678",
    "code": "123456",
    "url": "https://www.xiaohongshu.com/mobile/login?qrId=123456789012345678&ruleId=4&xhs_code=123456&timestamp=1768000000000&channel_type=web&component_id=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "multi_flag": 0
}))]
pub struct QrCodeData {
    pub qr_id: String,
    pub code: String,
    pub url: String,
    pub multi_flag: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code": 0,
    "success": true,
    "msg": "成功",
    "data": {
        "qr_id": "123456789012345678",
        "code": "123456",
        "url": "https://www.xiaohongshu.com/mobile/login?qrId=123456789012345678&ruleId=4&xhs_code=123456&timestamp=1768000000000&channel_type=web&component_id=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "multi_flag": 0
    }
}))]
pub struct QrCodeCreateResponse {
    pub code: i32,
    pub success: bool,
    pub msg: String,
    pub data: QrCodeData,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code_status": 3
}))]
pub struct QrCodeStatusData {
    pub code_status: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code": 0,
    "success": true,
    "msg": "成功",
    "data": {
        "code_status": 3
    }
}))]
pub struct QrCodeStatusResponse {
    pub code: i32,
    pub success: bool,
    pub msg: String,
    pub data: Option<QrCodeStatusData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "success": true,
    "step": "qrcode",
    "qr_base64": "data:image/png;base64,..."
}))]
pub struct QrCodeSessionResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qr_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Cookie information (sanitized for display)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "a1",
    "value": "192xxxxxxxxxxxxxxe0c",
    "domain": ".xiaohongshu.com"
}))]
pub struct CookieInfo {
    pub name: String,
    pub value: String,
    pub domain: String,
}

/// Session/Credential information response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code": 0,
    "success": true,
    "msg": "Session found",
    "data": {
        "user_id": "5ceac80d00000000xxxxxxxx",
        "cookie_count": 13,
        "cookies": [
            {"name": "a1", "value": "192xxxxxxxxxxxxxxe0c", "domain": ".xiaohongshu.com"}
        ],
        "x_s_common": "2UQAPsHC+aIjqArjwjHjNsQhPsHCH0rjNsQhPaHCHdH...",
        "created_at": "2026-01-11T05:00:00Z",
        "is_valid": true
    }
}))]
pub struct SessionInfoResponse {
    pub code: i32,
    pub success: bool,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<SessionInfoData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionInfoData {
    pub user_id: String,
    pub cookie_count: usize,
    pub cookies: Vec<CookieInfo>,
    pub x_s_common: String,
    pub created_at: String,
    pub is_valid: bool,
}
