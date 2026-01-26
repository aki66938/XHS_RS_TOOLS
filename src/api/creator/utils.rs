use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ORIGIN, REFERER, USER_AGENT};
use std::collections::HashMap;
use crate::config::get_agent_url;
use crate::api::login::{AgentSignRequest, AgentSignResponse};

// ============================================================================
// Constants
// ============================================================================

pub const CREATOR_ORIGIN: &str = "https://creator.xiaohongshu.com";
pub const CREATOR_REFERER: &str = "https://creator.xiaohongshu.com/";
pub const XHS_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

// ============================================================================
// Public Functions
// ============================================================================

/// Sign request using Agent
pub async fn sign_request(
    cookies: &HashMap<String, String>,
    method: &str,
    uri: &str,
    payload: Option<serde_json::Value>,
) -> Result<(String, String, String)> {
    let client = reqwest::Client::new();
    let url = format!("{}/sign", get_agent_url());
    
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
    ))
}

/// Build common headers for Creator API
pub fn build_creator_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert(ORIGIN, HeaderValue::from_static(CREATOR_ORIGIN));
    headers.insert(REFERER, HeaderValue::from_static(CREATOR_REFERER));
    headers.insert(USER_AGENT, HeaderValue::from_static(XHS_USER_AGENT));
    // CRITICAL: Creator Center / UGC context
    headers.insert("xsecappid", HeaderValue::from_static("ugc"));
    headers
}

pub fn cookies_to_string(cookies: &HashMap<String, String>) -> String {
    cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ")
}
