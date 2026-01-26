//! Creator Center User Info APIs
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use reqwest::header::HeaderValue;

use crate::api::creator::utils::{sign_request, build_creator_headers, cookies_to_string};
use crate::api::creator::models::{CreatorUserInfo, CreatorHomeInfo};

// ============================================================================
// Constants
// ============================================================================

const CREATOR_USER_INFO_URI: &str = "/api/galaxy/user/info";
const CREATOR_USER_INFO_URL: &str = "https://creator.xiaohongshu.com/api/galaxy/user/info";

const CREATOR_HOME_INFO_URI: &str = "/api/galaxy/creator/home/personal_info";
const CREATOR_HOME_INFO_URL: &str = "https://creator.xiaohongshu.com/api/galaxy/creator/home/personal_info";

// ============================================================================
// API Functions
// ============================================================================

/// Get Creator User Info (role, permissions, etc.)
pub async fn get_creator_user_info(cookies: &HashMap<String, String>) -> Result<CreatorUserInfo> {
    
    // Get signature (GET request, no payload)
    let (x_s, x_t, x_s_common) = 
        sign_request(cookies, "GET", CREATOR_USER_INFO_URI, None).await?;
        
    let mut headers = build_creator_headers();
    headers.insert("x-s", HeaderValue::from_str(&x_s)?);
    headers.insert("x-t", HeaderValue::from_str(&x_t)?);
    headers.insert("x-s-common", HeaderValue::from_str(&x_s_common)?);
    headers.insert("cookie", HeaderValue::from_str(&cookies_to_string(cookies))?);
    
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
        
    tracing::info!("Fetching Creator User Info...");
    
    let response = client
        .get(CREATOR_USER_INFO_URL)
        .send()
        .await?;
        
    let status = response.status();
    let text = response.text().await?;
    
    if status.as_u16() >= 400 {
        return Err(anyhow!("API Error ({}): {}", status, text));
    }
    
    // Parse generic wrapper: { code, data: ... }
    #[derive(serde::Deserialize)]
    struct ResponseWrapper {
        code: i32,
        msg: Option<String>,
        data: Option<CreatorUserInfo>,
    }
    
    let wrapper: ResponseWrapper = serde_json::from_str(&text)
        .map_err(|e| anyhow!("Parse error: {} - Body: {}", e, text))?;
        
    if wrapper.code != 0 {
        return Err(anyhow!("API Failed (code {}): {}", 
            wrapper.code, 
            wrapper.msg.unwrap_or_default()
        ));
    }
    
    wrapper.data.ok_or_else(|| anyhow!("No data returned"))
}

/// Get Creator Home Info (fans, likes, etc.)
pub async fn get_creator_home_info(cookies: &HashMap<String, String>) -> Result<CreatorHomeInfo> {
    
    // Get signature (GET request, no payload)
    let (x_s, x_t, x_s_common) = 
        sign_request(cookies, "GET", CREATOR_HOME_INFO_URI, None).await?;
        
    let mut headers = build_creator_headers();
    headers.insert("x-s", HeaderValue::from_str(&x_s)?);
    headers.insert("x-t", HeaderValue::from_str(&x_t)?);
    headers.insert("x-s-common", HeaderValue::from_str(&x_s_common)?);
    headers.insert("cookie", HeaderValue::from_str(&cookies_to_string(cookies))?);
    
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
        
    tracing::info!("Fetching Creator Home Info...");
    
    let response = client
        .get(CREATOR_HOME_INFO_URL)
        .send()
        .await?;
        
    let status = response.status();
    let text = response.text().await?;
    
    if status.as_u16() >= 400 {
        return Err(anyhow!("API Error ({}): {}", status, text));
    }
    
    // Parse generic wrapper
    #[derive(serde::Deserialize)]
    struct ResponseWrapper {
        code: i32,
        msg: Option<String>,
        data: Option<CreatorHomeInfo>,
    }
    
    let wrapper: ResponseWrapper = serde_json::from_str(&text)
        .map_err(|e| anyhow!("Parse error: {} - Body: {}", e, text))?;
        
    if wrapper.code != 0 {
        return Err(anyhow!("API Failed (code {}): {}", 
            wrapper.code, 
            wrapper.msg.unwrap_or_default()
        ));
    }
    
    wrapper.data.ok_or_else(|| anyhow!("No data returned"))
}
