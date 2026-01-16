use crate::api::XhsApiClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Likes response (赞和收藏 通知)
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LikesResponse {
    pub success: bool,
    pub msg: String,
    pub data: Option<LikesData>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LikesData {
    pub message_list: Vec<serde_json::Value>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub cursor: i64,
}

/// 通知页-赞和收藏
/// 
/// 获取赞和收藏通知列表
pub async fn get_likes(api: &XhsApiClient) -> Result<LikesResponse> {
    let text = api.get("notification_likes").await?;
    let result = serde_json::from_str::<LikesResponse>(&text)?;
    Ok(result)
}
