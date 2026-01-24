//! Feed HTTP Handlers
//! 
//! Handles: homefeed/recommend

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::api;
use crate::server::AppState;


// ============================================================================
// Handlers
// ============================================================================

/// 页面-主页发现-推荐 (内部接口)
/// 
/// 此接口从属于 /api/feed/homefeed/{category}，不单独在 Swagger 中显示
/// 获取小红书主页推荐内容流
pub async fn homefeed_recommend_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::feed::recommend::get_homefeed_recommend(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}
