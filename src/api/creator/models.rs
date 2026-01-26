//! Creator Center API Models
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Request body for creating Creator QR Code
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatorQrcodeCreateRequest {
    /// Guest cookies obtained from /api/creator/auth/guest-init
    #[schema(example = json!({"web_session": "xxxxx", "xsecappid": "ugc"}))]
    pub cookies: HashMap<String, String>,
}

/// Request body for polling Creator QR Code status
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatorQrcodeStatusRequest {
    /// QR Code ID returned from creation step
    #[schema(example = "68c517598657858235023360")]
    pub qr_id: String,
    
    /// Guest cookies
    #[schema(example = json!({"web_session": "xxxxx", "xsecappid": "ugc"}))]
    pub cookies: HashMap<String, String>,
}

/// Response for Creator User Info (/api/galaxy/user/info)
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreatorUserInfo {
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: Option<String>,
    #[serde(rename = "userAvatar")]
    pub user_avatar: Option<String>,
    pub role: Option<String>,
    pub permissions: Option<Vec<String>>, 
}

/// Response for Creator Home Info (/api/galaxy/creator/home/personal_info)
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreatorHomeInfo {
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub follow_count: Option<i32>,
    pub fans_count: Option<i32>,
    pub faved_count: Option<i32>, 
    pub red_num: Option<String>,
    pub personal_desc: Option<String>,
    pub grow_info: Option<CreatorGrowInfo>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreatorGrowInfo {
    pub level: Option<i32>,
    pub fans_count: Option<i32>,
    pub max_fans_count: Option<i32>,
}
