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
    "session": "040069b948b3ed3fc37f584f5c3b4b8b909397",
    "secure_session": "X6b2acsession.040069b948b3ed3fc37f584f5c3b4b8b909397",
    "user_id": "683cf0a5000000001b019329"
}))]
pub struct LoginInfo {
    pub session: String,
    pub secure_session: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code_status": 2,
    "login_info": {
        "session": "040069b948b3ed3fc37f584f5c3b4b8b909397",
        "secure_session": "X6b2acsession.040069b948b3ed3fc37f584f5c3b4b8b909397",
        "user_id": "683cf0a5000000001b019329"
    }
}))]
pub struct QrCodeStatusData {
    /// 状态码: 0=未扫码, 1=已扫码等待确认, 2=登录成功
    pub code_status: i32,
    /// 登录成功时返回的登录信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_info: Option<LoginInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code": 0,
    "success": true,
    "msg": "成功",
    "data": {
        "code_status": 2,
        "login_info": {
            "session": "040069b948b3ed3fc37f584f5c3b4b8b909397",
            "secure_session": "X6b2acsession.040069b948b3ed3fc37f584f5c3b4b8b909397",
            "user_id": "683cf0a5000000001b019329"
        }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_s_common: Option<String>,
    pub created_at: String,
    pub is_valid: bool,
}


