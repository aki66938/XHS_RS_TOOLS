use crate::auth::AuthService;
use crate::client::XhsClient;
use crate::models::feed::{HomefeedRequest, HomefeedResponse};
use anyhow::Result;
use std::sync::Arc;

const ORIGIN: &str = "https://www.xiaohongshu.com";
const REFERER: &str = "https://www.xiaohongshu.com/";

/// Get homefeed recommend (页面-主页发现-推荐)
pub async fn get_homefeed_recommend(
    client: &XhsClient, 
    auth: &Arc<AuthService>,
    request: HomefeedRequest,
) -> Result<HomefeedResponse> {
    let url = "https://edith.xiaohongshu.com/api/sns/web/v1/homefeed";

    // Get credentials (cookies)
    let credentials = auth.get_credentials().await?;
    
    // Get captured signature for this endpoint
    let signature = auth.get_endpoint_signature("home_feed_recommend").await?
        .ok_or_else(|| anyhow::anyhow!("No signature found for home_feed_recommend endpoint. Please login again."))?;
    
    tracing::info!("Using captured signature for home_feed_recommend");

    // Use the captured POST body (signature is bound to this specific body)
    // If no captured body, fall back to the request parameter
    let body = signature.post_body.clone().unwrap_or_else(|| {
        serde_json::to_string(&request).unwrap_or_default()
    });
    
    tracing::info!("Homefeed request body length: {} chars", body.len());

    let response = client.get_client()
        .post(url)
        // Standard browser headers
        .header("accept", "application/json, text/plain, */*")
        .header("accept-language", "zh-CN,zh;q=0.9")
        .header("content-type", "application/json;charset=UTF-8")
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
        .header("xy-direction", "98")
        .header("cookie", credentials.cookie_string())
        .body(body)
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;
    tracing::info!("Homefeed Response [{}]: {} chars", status, text.len());

    // Note: 406 error is due to x-s signature issues, NOT cookie issues
    if status.as_u16() == 406 {
        tracing::warn!("Received 406 error - x-s signature may be invalid (cookies are still valid)");
    }

    let result = serde_json::from_str::<HomefeedResponse>(&text)?;
    Ok(result)
}
