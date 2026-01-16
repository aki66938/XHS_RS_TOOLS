use crate::api::XhsApiClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Connections response (新增关注 通知)
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ConnectionsResponse {
    pub success: bool,
    pub msg: String,
    pub data: Option<ConnectionsData>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ConnectionsData {
    pub message_list: Vec<serde_json::Value>,
}

/// 通知页-新增关注
/// 
/// 获取新增关注通知列表
pub async fn get_connections(api: &XhsApiClient) -> Result<ConnectionsResponse> {
    let text = api.get("notification_connections").await?;
    let result = serde_json::from_str::<ConnectionsResponse>(&text)?;
    Ok(result)
}
