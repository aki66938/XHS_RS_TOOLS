//! 签名服务模块 (Signature Service Module)
//!
//! 提供两种签名获取策略：
//! 1. **纯算法 (Pure Algorithm)**: 调用 Python Agent 的 `/sign` 端点，使用 xhshow 库生成签名
//! 2. **浏览器捕获 (Browser Capture)**: 从 MongoDB 读取之前通过 Playwright 捕获的签名（兜底）
//!
//! 默认优先使用纯算法，失败时自动降级到浏览器捕获。

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::get_agent_url;

/// 签名请求结构
#[derive(Debug, Serialize)]
pub struct SignRequest {
    pub method: String,
    pub uri: String,
    pub cookies: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// 签名响应结构
#[derive(Debug, Deserialize)]
pub struct SignResponse {
    pub success: bool,
    pub x_s: Option<String>,
    pub x_t: Option<String>,
    pub x_s_common: Option<String>,
    pub x_b3_traceid: Option<String>,
    pub x_xray_traceid: Option<String>,
    pub error: Option<String>,
}

/// 签名结果（用于请求构建）
#[derive(Debug, Clone)]
pub struct Signature {
    pub x_s: String,
    pub x_t: String,
    pub x_s_common: String,
    pub x_b3_traceid: String,
    pub x_xray_traceid: String,
}

/// 签名服务 - 提供签名获取的统一接口
pub struct SignatureService {
    client: reqwest::Client,
}

impl SignatureService {
    /// 创建签名服务实例
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// 通过 Python Agent 获取签名（纯算法）
    ///
    /// # Arguments
    /// * `method` - HTTP 方法 (GET/POST)
    /// * `uri` - API 路径 (如 /api/sns/web/v1/homefeed)
    /// * `cookies` - Cookie 字典
    /// * `payload` - POST 请求体 (可选)
    ///
    /// # Returns
    /// 生成的签名或错误
    pub async fn get_signature_from_agent(
        &self,
        method: &str,
        uri: &str,
        cookies: HashMap<String, String>,
        payload: Option<serde_json::Value>,
    ) -> Result<Signature> {
        let request = SignRequest {
            method: method.to_uppercase(),
            uri: uri.to_string(),
            cookies,
            params: None,
            payload,
        };

        let url = format!("{}/sign", get_agent_url());
        
        tracing::debug!("[SignatureService] Calling Agent: {} {}", method, uri);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| anyhow!("Agent connection failed: {}. Is agent_server.py running?", e))?;

        let sign_resp: SignResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Agent response: {}", e))?;

        if !sign_resp.success {
            return Err(anyhow!(
                "Agent signing failed: {}",
                sign_resp.error.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        Ok(Signature {
            x_s: sign_resp.x_s.unwrap_or_default(),
            x_t: sign_resp.x_t.unwrap_or_default(),
            x_s_common: sign_resp.x_s_common.unwrap_or_default(),
            x_b3_traceid: sign_resp.x_b3_traceid.unwrap_or_default(),
            x_xray_traceid: sign_resp.x_xray_traceid.unwrap_or_default(),
        })
    }

    /// 检查 Agent 是否可用
    pub async fn is_agent_available(&self) -> bool {
        let url = format!("{}/health", get_agent_url());
        match self.client.get(&url).timeout(std::time::Duration::from_secs(2)).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

impl Default for SignatureService {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 Cookie 字符串解析为 HashMap
pub fn parse_cookie_string(cookie_str: &str) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    for item in cookie_str.split(';') {
        if let Some((key, value)) = item.trim().split_once('=') {
            cookies.insert(key.to_string(), value.to_string());
        }
    }
    cookies
}
