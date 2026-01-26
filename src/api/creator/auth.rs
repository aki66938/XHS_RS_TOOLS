//! Creator Center Authentication API
//!
//! Handles the QR code login process specifically for the Creator Center (creator.xiaohongshu.com).
//! Similar to the user login flow but operating in the 'ugc' context.

use anyhow::{anyhow, Result};
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use serde::Deserialize;
use std::collections::HashMap;
use crate::config::get_agent_url;
use crate::api::login::{
    QrCodeCreateResponse,
    QrCodeCreateData,
    AgentGuestCookiesResponse,
};

// ============================================================================
// Constants
// ============================================================================

// Creator QR Code API is actually the same endpoint on customer.xiaohongshu.com
const QRCODE_CREATE_URL: &str = "https://customer.xiaohongshu.com/api/cas/customer/web/qr-code";

use crate::api::creator::utils::{sign_request, build_creator_headers, cookies_to_string};

// ============================================================================
// Core Functions
// ============================================================================

/// Fetch guest cookies for Creator Center options
///
/// Calls Agent with `target=creator` to initialize cookies on creator.xiaohongshu.com
pub async fn fetch_creator_guest_cookies() -> Result<HashMap<String, String>> {
    let client = reqwest::Client::new();
    // Use the new target parameter we added to agent_server.py
    let url = format!("{}/guest-cookies?target=creator", get_agent_url());
    
    tracing::info!("Fetching CREATOR guest cookies from Agent...");
    
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(45)) // Little extra buffer
        .send()
        .await
        .map_err(|e| anyhow!("Failed to connect to Agent: {}", e))?;
    
    let result: AgentGuestCookiesResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse Agent response: {}", e))?;
    
    if !result.success {
        return Err(anyhow!("Agent error: {}", result.error.unwrap_or_default()));
    }
    
    result.cookies.ok_or_else(|| anyhow!("No cookies returned"))
}

/// Create QR code for Creator Center Login
pub async fn create_creator_qrcode(cookies: &HashMap<String, String>) -> Result<QrCodeCreateResponse> {
    let uri = "/api/cas/customer/web/qr-code";
    let payload = serde_json::json!({"service": "https://creator.xiaohongshu.com"});
    
    // Get signature
    let (x_s, x_t, x_s_common) = 
        sign_request(cookies, "POST", uri, Some(payload.clone())).await?;
    
    // Build request
    let mut headers = build_creator_headers();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json;charset=UTF-8"));
    headers.insert("x-s", HeaderValue::from_str(&x_s)?);
    headers.insert("x-t", HeaderValue::from_str(&x_t)?);
    headers.insert("x-s-common", HeaderValue::from_str(&x_s_common)?);
    headers.insert("cookie", HeaderValue::from_str(&cookies_to_string(cookies))?);
    
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    
    tracing::info!("Creating Creator QR code...");
    
    let response = client
        .post(QRCODE_CREATE_URL)
        .json(&payload)
        .send()
        .await?;
    
    let status = response.status();
    let text = response.text().await?;
    
    tracing::debug!("Creator QR Response [{}]: {}", status, text);
    
    if status.as_u16() >= 400 {
        return Err(anyhow!("API Error ({}): {}", status, text));
    }
    
    #[derive(Debug, Deserialize)]
    struct RawCreatorQrResponse {
        success: bool,
        code: i32,
        msg: Option<String>,
        data: Option<RawCreatorQrData>,
    }
    #[derive(Debug, Deserialize)]
    struct RawCreatorQrData {
        url: String,
        id: String, // Matched from actual API response
    }
    
    let raw: RawCreatorQrResponse = serde_json::from_str(&text)
        .map_err(|e| anyhow!("Parse error: {} - Body: {}", e, text))?;
        
    let data = raw.data.map(|d| QrCodeCreateData {
        url: d.url,
        qr_id: d.id, // Map 'id' to 'qr_id'
        code: "".to_string(),
    });
    
    Ok(QrCodeCreateResponse {
        success: raw.success,
        code: raw.code,
        msg: raw.msg,
        data,
    })
}

/// Check Creator QR Code Status
///
/// Polls the status of the QR code.
/// Status codes: 2 (Waiting), 3 (Scanned), 1 (Success)
pub async fn check_creator_qrcode_status(
    qr_id: &str,
    cookies: &HashMap<String, String>
) -> Result<(serde_json::Value, Option<HashMap<String, String>>)> {
    let uri = "/api/cas/customer/web/qr-code";
    
    let service = "https://creator.xiaohongshu.com";
    // Construct query component specifically for signature and request
    let query = format!("service={}&qr_code_id={}&source=", urlencoding::encode(service), qr_id);
    let full_uri = format!("{}?{}", uri, query);
    
    // Get signature (GET request, no payload)
    let (x_s, x_t, x_s_common) = 
        sign_request(cookies, "GET", &full_uri, None).await?;
        
    let mut headers = build_creator_headers();
    headers.insert("x-s", HeaderValue::from_str(&x_s)?);
    headers.insert("x-t", HeaderValue::from_str(&x_t)?);
    headers.insert("x-s-common", HeaderValue::from_str(&x_s_common)?);
    headers.insert("cookie", HeaderValue::from_str(&cookies_to_string(cookies))?);
    
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
        
    let url = format!("{}?{}", QRCODE_CREATE_URL, query);
    
    tracing::debug!("Polling Creator QR: {}", url);
    
    let response = client
        .get(&url)
        .send()
        .await?;
        
    // Extract new cookies from Set-Cookie headers
    let mut new_cookies: HashMap<String, String> = HashMap::new();
    for (name, value) in response.headers().iter() {
        if name.as_str() == "set-cookie" {
            if let Ok(v) = value.to_str() {
                // Parse "name=value; ..." format
                if let Some(main) = v.split(';').next() {
                    let mut parts = main.splitn(2, '=');
                    if let (Some(k), Some(val)) = (parts.next(), parts.next()) {
                        new_cookies.insert(k.trim().to_string(), val.trim().to_string());
                    }
                }
            }
        }
    }
    
    let status = response.status();
    let text = response.text().await?;
    
    if status.as_u16() >= 400 {
        return Err(anyhow!("API Error ({}): {}", status, text));
    }
    
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| anyhow!("Parse error: {} - Body: {}", e, text))?;

    // Check data.status
    // 1: Success
    // 2: Waiting
    // 3: Scanned
    let code_status = json.get("data")
        .and_then(|d| d.get("status"))  // Creator API uses "status" inside data, not "code_status"
        .and_then(|s| s.as_i64())
        .map(|s| s as i32);
        
    let cookies_to_return = if code_status == Some(1) {
        tracing::info!("Creator Login confirmed! Starting blocking cookie synchronization...");
        
        // Collect identifying cookies for the Creator session
        let mut tokens_to_sync = HashMap::new();
        
        if let Some(sid) = new_cookies.get("customer-sso-sid").or_else(|| cookies.get("customer-sso-sid")) {
            tokens_to_sync.insert("customer-sso-sid".to_string(), sid.clone());
        }
        if let Some(token) = new_cookies.get("access-token-creator.xiaohongshu.com").or_else(|| cookies.get("access-token-creator.xiaohongshu.com")) {
            tokens_to_sync.insert("access-token-creator.xiaohongshu.com".to_string(), token.clone());
        }
        // Also try specific session ID if available
        if let Some(sess) = new_cookies.get("galaxy.creator.beaker.session.id").or_else(|| cookies.get("galaxy.creator.beaker.session.id")) {
             tokens_to_sync.insert("galaxy.creator.beaker.session.id".to_string(), sess.clone());
        }
        if let Some(web) = new_cookies.get("web_session").or_else(|| cookies.get("web_session")) {
            tokens_to_sync.insert("web_session".to_string(), web.clone());
        }

        if !tokens_to_sync.is_empty() {
            // Call sync with target="creator" and all collected tokens
            match crate::api::login::sync_login_cookies(&tokens_to_sync, Some("creator")).await {
                Ok(synced_cookies) => {
                    tracing::info!("Creator Cookie synchronization successful! Got {} cookies.", synced_cookies.len());
                    
                    let mut merged = new_cookies.clone();
                    merged.extend(synced_cookies);
                    
                    Some(merged)
                }
                Err(e) => {
                    tracing::warn!("Creator Cookie synchronization failed: {}. Falling back.", e);
                    if new_cookies.is_empty() { None } else { Some(new_cookies) }
                }
            }
        } else {
            tracing::warn!("Creator Login success but no valid session tokens found for sync. Skipping.");
            if new_cookies.is_empty() { None } else { Some(new_cookies) }
        }
    } else {
        if new_cookies.is_empty() { None } else { Some(new_cookies) }
    };
        
    Ok((json, cookies_to_return))
}