use crate::auth::AuthService;
use crate::client::XhsClient;
use crate::models::search::QueryTrendingResponse;
use anyhow::Result;
use std::sync::Arc;

const ORIGIN: &str = "https://www.xiaohongshu.com";
const REFERER: &str = "https://www.xiaohongshu.com/";

pub async fn query_trending(client: &XhsClient, auth: &Arc<AuthService>) -> Result<QueryTrendingResponse> {
    let base_url = "https://edith.xiaohongshu.com/api/sns/web/v1/search/querytrending";
    // Construct query parameters
    let params = [
        ("source", "Explore"),
        ("search_type", "trend"),
        ("last_query", ""),
        ("last_query_time", "0"),
        ("word_request_situation", "FIRST_ENTER"),
        ("hint_word", ""),
        ("hint_word_type", ""),
        ("hint_word_request_id", ""),
    ];
    
    let url = reqwest::Url::parse_with_params(base_url, &params)?;

    // Get credentials (cookies)
    let credentials = auth.get_credentials().await?;
    
    // Get captured signature for this endpoint
    let signature = auth.get_endpoint_signature("search_trending").await?
        .ok_or_else(|| anyhow::anyhow!("No signature found for search_trending endpoint. Please login again."))?;
    
    tracing::info!("Using captured signature for search_trending");

    let response = client.get_client()
        .get(url)
        // Standard browser headers
        .header("accept", "application/json, text/plain, */*")
        .header("accept-language", "zh-CN,zh;q=0.9")
        .header("cache-control", "no-cache")
        .header("pragma", "no-cache")
        .header("priority", "u=1, i")
        // Security headers
        .header("sec-ch-ua", r#""Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24""#)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", r#""Windows""#)
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-site")
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36")
        // XHS specific headers - use captured signature
        .header("origin", ORIGIN)
        .header("referer", REFERER)
        .header("x-s", &signature.x_s)
        .header("x-t", &signature.x_t)
        .header("x-s-common", &signature.x_s_common)
        .header("x-b3-traceid", &signature.x_b3_traceid)
        .header("x-xray-traceid", &signature.x_xray_traceid)
        .header("cookie", credentials.cookie_string())
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;
    tracing::info!("Query Trending Response [{}]: {}", status, text);

    // Note: 406 error is due to x-s signature issues, NOT cookie issues
    // Do NOT invalidate credentials here - that would break the session
    if status.as_u16() == 406 {
        tracing::warn!("Received 406 error - x-s signature may be invalid (cookies are still valid)");
    }

    let result = serde_json::from_str::<QueryTrendingResponse>(&text)?;
    Ok(result)
}
