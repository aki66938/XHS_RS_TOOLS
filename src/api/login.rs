//! Login API - QR Code Login Flow
//!
//! This module handles the XHS QR code login process:
//! 1. Fetch guest cookies from Python Agent (Playwright)
//! 2. Create QR code using official API
//! 3. Poll QR code status until login success
//! 4. Store user credentials in MongoDB
//!
//! Design Principles:
//! - Single Responsibility: Each function does one thing
//! - KISS: Simple, straightforward implementation

use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Constants
// ============================================================================

const XHS_ORIGIN: &str = "https://www.xiaohongshu.com";
const XHS_REFERER: &str = "https://www.xiaohongshu.com/";
const XHS_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

const AGENT_URL: &str = "http://127.0.0.1:8765";
const QRCODE_CREATE_URL: &str = "https://edith.xiaohongshu.com/api/sns/web/v1/login/qrcode/create";
const QRCODE_STATUS_URL: &str = "https://edith.xiaohongshu.com/api/sns/web/v1/login/qrcode/status";

// ============================================================================
// Agent Request/Response Models
// ============================================================================

/// Request to Python Agent for signature generation
#[derive(Debug, Serialize)]
struct AgentSignRequest {
    method: String,
    uri: String,
    cookies: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<serde_json::Value>,
}

/// Response from Python Agent signature endpoint
#[derive(Debug, Deserialize)]
struct AgentSignResponse {
    success: bool,
    x_s: Option<String>,
    x_t: Option<String>,
    x_s_common: Option<String>,
    x_b3_traceid: Option<String>,
    x_xray_traceid: Option<String>,
    error: Option<String>,
}

/// Response from Python Agent guest-cookies endpoint
#[derive(Debug, Deserialize)]
struct AgentGuestCookiesResponse {
    success: bool,
    cookies: Option<HashMap<String, String>>,
    error: Option<String>,
}

// ============================================================================
// XHS API Response Models
// ============================================================================

/// XHS QR Code Create Response
#[derive(Debug, Deserialize, Serialize)]
pub struct QrCodeCreateResponse {
    pub success: bool,
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<QrCodeCreateData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QrCodeCreateData {
    pub url: String,
    pub qr_id: String,
    pub code: String,
}

/// XHS QR Code Status Response
#[derive(Debug, Deserialize, Serialize)]
pub struct QrCodeStatusResponse {
    pub success: bool,
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<QrCodeStatusData>,
}

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct QrCodeStatusData {
    pub code_status: Option<i32>,
    pub login_info: Option<LoginInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct LoginInfo {
    pub user_id: Option<String>,
    pub session: Option<String>,
}

// ============================================================================
// Public API Response Models (for Rust Server endpoints)
// ============================================================================

/// Response for guest-init endpoint
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GuestInitResponse {
    pub success: bool,
    pub cookies: Option<HashMap<String, String>>,
    pub error: Option<String>,
}

/// Response for qrcode/create endpoint
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateQrCodeResponse {
    pub success: bool,
    pub qr_url: Option<String>,
    pub qr_id: Option<String>,
    pub code: Option<String>,
    pub error: Option<String>,
}

/// Response for qrcode/status endpoint
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PollStatusResponse {
    pub success: bool,
    pub code_status: i32,  // 0=waiting, 1=scanned, 2=confirmed
    pub login_info: Option<LoginInfo>,
    pub new_cookies: Option<HashMap<String, String>>,
    pub error: Option<String>,
}

// ============================================================================
// Core Functions
// ============================================================================

/// Fetch guest cookies from Python Agent (uses Playwright internally)
///
/// Returns a HashMap of cookies needed for QR code login
pub async fn fetch_guest_cookies() -> Result<HashMap<String, String>> {
    let client = reqwest::Client::new();
    let url = format!("{}/guest-cookies", AGENT_URL);
    
    tracing::info!("Fetching guest cookies from Agent...");
    
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(30))  // Playwright needs time
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

/// Get signature from Python Agent
async fn sign_request(
    cookies: &HashMap<String, String>,
    method: &str,
    uri: &str,
    payload: Option<serde_json::Value>,
) -> Result<(String, String, String, String)> {
    let client = reqwest::Client::new();
    let url = format!("{}/sign", AGENT_URL);
    
    let request = AgentSignRequest {
        method: method.to_string(),
        uri: uri.to_string(),
        cookies: cookies.clone(),
        payload,
    };
    
    let response = client
        .post(&url)
        .json(&request)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| anyhow!("Failed to connect to Agent: {}", e))?;
    
    let result: AgentSignResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse signature response: {}", e))?;
    
    if !result.success {
        return Err(anyhow!("Sign error: {}", result.error.unwrap_or_default()));
    }
    
    Ok((
        result.x_s.unwrap_or_default(),
        result.x_t.unwrap_or_default(),
        result.x_s_common.unwrap_or_default(),
        result.x_b3_traceid.unwrap_or_default(),
    ))
}

/// Build common headers for XHS API requests
fn build_common_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert(ORIGIN, HeaderValue::from_static(XHS_ORIGIN));
    headers.insert(REFERER, HeaderValue::from_static(XHS_REFERER));
    headers.insert(USER_AGENT, HeaderValue::from_static(XHS_USER_AGENT));
    headers
}

/// Convert cookies HashMap to cookie string
fn cookies_to_string(cookies: &HashMap<String, String>) -> String {
    cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Create QR code using official API
pub async fn create_qrcode(cookies: &HashMap<String, String>) -> Result<QrCodeCreateResponse> {
    let uri = "/api/sns/web/v1/login/qrcode/create";
    let payload = serde_json::json!({"qr_type": 1});
    
    // Get signature
    let (x_s, x_t, x_s_common, x_b3_traceid) = 
        sign_request(cookies, "POST", uri, Some(payload.clone())).await?;
    
    // Build request
    let mut headers = build_common_headers();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json;charset=UTF-8"));
    headers.insert("x-s", HeaderValue::from_str(&x_s)?);
    headers.insert("x-t", HeaderValue::from_str(&x_t)?);
    headers.insert("x-s-common", HeaderValue::from_str(&x_s_common)?);
    headers.insert("x-b3-traceid", HeaderValue::from_str(&x_b3_traceid)?);
    headers.insert("cookie", HeaderValue::from_str(&cookies_to_string(cookies))?);
    
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    
    tracing::info!("Creating QR code...");
    
    let response = client
        .post(QRCODE_CREATE_URL)
        .json(&payload)
        .send()
        .await?;
    
    let status = response.status();
    let text = response.text().await?;
    
    tracing::debug!("QR Create Response [{}]: {}", status, text);
    
    if status.as_u16() == 406 {
        return Err(anyhow!("Signature rejected (406): cookies may be invalid"));
    }
    
    serde_json::from_str(&text).map_err(|e| anyhow!("Parse error: {} - Body: {}", e, text))
}

/// Check QR code status using official API
///
/// Returns the status and any new cookies from Set-Cookie headers
pub async fn check_qrcode_status(
    cookies: &HashMap<String, String>,
    qr_id: &str,
    code: &str,
) -> Result<(QrCodeStatusResponse, Option<HashMap<String, String>>)> {
    let uri = format!("/api/sns/web/v1/login/qrcode/status?qr_id={}&code={}", qr_id, code);
    let url = format!("{}?qr_id={}&code={}", QRCODE_STATUS_URL, qr_id, code);
    
    // Get signature
    let (x_s, x_t, x_s_common, x_b3_traceid) = 
        sign_request(cookies, "GET", &uri, None).await?;
    
    // Build request
    let mut headers = build_common_headers();
    headers.insert("x-s", HeaderValue::from_str(&x_s)?);
    headers.insert("x-t", HeaderValue::from_str(&x_t)?);
    headers.insert("x-s-common", HeaderValue::from_str(&x_s_common)?);
    headers.insert("x-b3-traceid", HeaderValue::from_str(&x_b3_traceid)?);
    headers.insert("cookie", HeaderValue::from_str(&cookies_to_string(cookies))?);
    
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    
    let response = client.get(&url).send().await?;
    
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
    
    let status_response: QrCodeStatusResponse = response.json().await?;
    
    let cookies_to_return = if new_cookies.is_empty() {
        None
    } else {
        Some(new_cookies)
    };
    
    Ok((status_response, cookies_to_return))
}
