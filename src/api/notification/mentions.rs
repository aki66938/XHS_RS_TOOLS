use crate::api::XhsApiClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Mentions response (评论和@ 通知)
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MentionsResponse {
    pub code: Option<i32>,
    pub success: bool,
    pub msg: String,
    pub data: Option<MentionsData>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MentionsData {
    pub cursor: Option<i64>,
    #[serde(rename = "strCursor")]
    pub str_cursor: Option<String>,
    pub message_list: Vec<serde_json::Value>,
}

/// 通知页-评论和@
/// 
/// 获取评论和@通知列表
pub async fn get_mentions(api: &XhsApiClient) -> Result<MentionsResponse> {
    let text = api.get("notification_mentions").await?;
    let result = serde_json::from_str::<MentionsResponse>(&text)?;
    Ok(result)
}
