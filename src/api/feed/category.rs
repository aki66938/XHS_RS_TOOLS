use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use crate::{
    auth::AuthService,
    client::XhsClient,
    models::feed::{HomefeedRequest, HomefeedResponse},
    server::AppState,
};

/// Get feed for specific category (页面-主页发现-频道)
/// Path param: category (e.g., "fashion", "food", "travel")
#[utoipa::path(
    post,
    path = "/api/feed/homefeed/{category}",
    request_body = HomefeedRequest,
    responses(
        (status = 200, description = "Success", body = HomefeedResponse),
        (status = 500, description = "Internal Error")
    ),
    tag = "Feed"
)]
pub async fn get_category_feed(
    State(state): State<Arc<AppState>>,
    Path(category): Path<String>,
    Json(request): Json<HomefeedRequest>,
) -> impl axum::response::IntoResponse {
    match get_feed_internal(&state.client, &state.auth, &category, request).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

async fn get_feed_internal(
    client: &XhsClient, 
    auth: &Arc<AuthService>,
    category: &str,
    request: HomefeedRequest,
) -> anyhow::Result<HomefeedResponse> {
    // URL is usually the same for all feeds, just payload differs
    let url = "https://edith.xiaohongshu.com/api/sns/web/v1/homefeed";
    let origin = "https://www.xiaohongshu.com";
    let referer = "https://www.xiaohongshu.com/explore";

    // Construct signature key: home_feed_fashion, home_feed_food, etc.
    // Specially handle "recommend" if user calls it via this route
    let signature_key = if category == "recommend" {
        "home_feed_recommend".to_string()
    } else {
        format!("home_feed_{}", category)
    };

    // Get credentials (cookies)
    let credentials = auth.get_credentials().await?;
    
    // Get captured signature for this endpoint
    let signature = auth.get_endpoint_signature(&signature_key).await?
        .ok_or_else(|| anyhow::anyhow!("No signature found for category '{}' (key: {}). Please login again and ensure script traversed this channel.", category, signature_key))?;
    
    tracing::info!("Using captured signature for category: {}", category);

    // Use the captured POST body (signature is bound to this specific body)
    // If no captured body, fall back to the request parameter
    let body = signature.post_body.clone().unwrap_or_else(|| {
        serde_json::to_string(&request).unwrap_or_default()
    });
    
    tracing::info!("Feed request body length: {} chars", body.len());

    let response = client.get_client()
        .post(url)
        // Standard headers
        .header("accept", "application/json, text/plain, */*")
        .header("accept-language", "zh-CN,zh;q=0.9")
        .header("content-type", "application/json;charset=UTF-8")
        .header("priority", "u=1, i")
        .header("sec-ch-ua", r#""Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24""#)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", r#""Windows""#)
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-site")
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36")
        // XHS headers
        .header("origin", origin)
        .header("referer", referer)
        .header("x-s", &signature.x_s)
        .header("x-t", &signature.x_t)
        .header("x-s-common", &signature.x_s_common)
        .header("x-b3-traceid", &signature.x_b3_traceid)
        .header("x-xray-traceid", &signature.x_xray_traceid)
        .header("xy-direction", "98")
        .header("cookie", credentials.cookie_string())
        .body(body)
        .send()
        .await?;

    let resp_text = response.text().await?;
    // tracing::info!("Feed Response [200 OK]: {} chars", resp_text.len());
    
    let feed_resp: HomefeedResponse = serde_json::from_str(&resp_text)?;
    Ok(feed_resp)
}
