//! 公共 API 请求模块
//! 
//! 所有 XHS API 接口共享相同的认证层结构（Cookie + Headers + Signature），
//! 但各接口的签名数据是独立存储的。
//! 
//! 此模块抽离了"读取签名 → 构建 Headers → 发送请求"的公共逻辑，
//! 使得每个具体接口只需关注 URL 和 Payload 的构造。
//!
//! ## 签名策略 (Signature Strategy)
//! 1. **纯算法优先**: 调用 Python Agent 生成签名 (xhshow)
//! 2. **浏览器兜底**: 若 Agent 不可用，回退到存储的签名

use crate::auth::AuthService;
use crate::auth::credentials::ApiSignature;
use crate::client::XhsClient;
use crate::signature::{SignatureService, Signature, parse_cookie_string};
use anyhow::{Result, anyhow};
use std::sync::Arc;

const ORIGIN: &str = "https://www.xiaohongshu.com";
const REFERER: &str = "https://www.xiaohongshu.com/";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

/// Endpoint Key 到 API URI 的映射
/// 用于纯算法签名生成
/// 注意：某些端点需要查询参数，直接包含在 URI 中
fn endpoint_to_uri(endpoint_key: &str) -> Option<&'static str> {
    match endpoint_key {
        // User
        "user_me" => Some("/api/sns/web/v2/user/me"),
        "user_selfinfo" => Some("/api/sns/web/v1/user/selfinfo"),
        // Search
        "search_trending" => Some("/api/sns/web/v1/search/querytrending"),
        "search_notes" => Some("/api/sns/web/v1/search/notes"),
        "notification_mentions" => Some("/api/sns/web/v1/you/mentions?num=20&cursor="),
        "notification_connections" => Some("/api/sns/web/v1/you/connections?num=20&cursor="),
        "notification_likes" => Some("/api/sns/web/v1/you/likes?num=20&cursor="),
        // Home Feed
        "home_feed_recommend" => Some("/api/sns/web/v1/homefeed"),
        key if key.starts_with("home_feed_") => Some("/api/sns/web/v1/homefeed"),
        // Note (动态参数，需要特殊处理)
        "note_page" => None,  // 需要 note_id 和 xsec_token，无法静态映射
        _ => None,
    }
}

/// 解析 URI，分离 path 和 query params
/// 注意：空值参数会被过滤（与 Python parse_qs 默认行为一致）
/// 例如: "/api/foo?num=20&cursor=" -> ("/api/foo", [("num", "20")])
fn parse_uri_with_params(uri: &str) -> (&str, Vec<(&str, &str)>) {
    if let Some(idx) = uri.find('?') {
        let path = &uri[..idx];
        let query = &uri[idx + 1..];
        let params: Vec<(&str, &str)> = query
            .split('&')
            .filter(|s| !s.is_empty())
            .filter_map(|kv| {
                let mut parts = kv.splitn(2, '=');
                match (parts.next(), parts.next()) {
                    // 只保留非空值参数（与 Python parse_qs 一致）
                    (Some(k), Some(v)) if !v.is_empty() => Some((k, v)),
                    _ => None,
                }
            })
            .collect();
        (path, params)
    } else {
        (uri, vec![])
    }
}

/// XHS API 公共客户端
/// 
/// 封装了所有 API 请求的公共逻辑：
/// - 从 AuthService 获取 Cookie
/// - 优先使用 SignatureService (纯算法) 生成签名
/// - 回退到存储的签名 (浏览器捕获)
/// - 构建标准浏览器 Headers
pub struct XhsApiClient {
    http_client: XhsClient,
    auth: Arc<AuthService>,
    signature_service: SignatureService,
}

impl XhsApiClient {
    /// 创建新的 API 客户端
    pub fn new(http_client: XhsClient, auth: Arc<AuthService>) -> Self {
        Self { 
            http_client, 
            auth,
            signature_service: SignatureService::new(),
        }
    }

    /// 获取认证服务引用
    pub fn auth(&self) -> &Arc<AuthService> {
        &self.auth
    }

    /// 执行 GET 请求（纯算法优先 + 存储回退）
    /// 
    /// 优先使用 Python Agent 生成签名，失败时回退到存储的签名
    /// 
    /// # Arguments
    /// * `endpoint_key` - 签名存储的 key（如 "search_trending", "notification_mentions"）
    /// 
    /// # Returns
    /// 响应文本内容
    pub async fn get(&self, endpoint_key: &str) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        
        let cookie_str = credentials.cookie_string();
        
        // 优先尝试纯算法签名
        if let Some(uri) = endpoint_to_uri(endpoint_key) {
            // 解析 URI，分离 path 和 query params
            let (path, params) = parse_uri_with_params(uri);
            let base_url = format!("https://edith.xiaohongshu.com{}", path);
            
            match self.get_algo_signature("GET", uri, &cookie_str, None).await {
                Ok(signature) => {
                    tracing::info!("[XhsApiClient] GET {} using ALGO (path: {}, params: {:?})", endpoint_key, path, params);
                    // 使用 .query() 传递参数，而不是直接拼在 URL 中
                    let response = self.build_get_request_algo(&base_url, &signature, &cookie_str)
                        .query(&params)
                        .send()
                        .await?;
                    return self.handle_response(response, endpoint_key).await;
                }
                Err(algo_err) => {
                    tracing::warn!("[XhsApiClient] Algo failed for {}: {}, trying stored signature", endpoint_key, algo_err);
                }
            }
        }
        
        // 回退到存储的签名
        let signature = self.get_signature(endpoint_key).await?;
        let url = signature.request_url.clone()
            .ok_or_else(|| anyhow!("No request_url found for endpoint: {}", endpoint_key))?;
        
        tracing::info!("[XhsApiClient] GET {} using STORED signature", endpoint_key);
        
        let response = self.build_get_request(&url, &signature, &cookie_str)
            .send()
            .await?;
        
        self.handle_response(response, endpoint_key).await
    }

    /// 执行 GET 请求（纯算法签名优先）
    /// 
    /// 优先使用 Python Agent 生成签名，失败时回退到存储的签名
    /// 
    /// # Arguments
    /// * `uri` - API 路径（如 "/api/sns/web/v1/user/selfinfo"）
    /// 
    /// # Returns
    /// 响应文本内容
    pub async fn get_algo(&self, uri: &str) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        
        let cookie_str = credentials.cookie_string();
        let url = format!("https://edith.xiaohongshu.com{}", uri);
        
        // 尝试纯算法签名
        match self.get_algo_signature("GET", uri, &cookie_str, None).await {
            Ok(signature) => {
                tracing::info!("[XhsApiClient] GET {} using ALGO signature", uri);
                let response = self.build_get_request_algo(&url, &signature, &cookie_str)
                    .send()
                    .await?;
                self.handle_response(response, uri).await
            }
            Err(algo_err) => {
                // 算法失败，记录警告并回退
                tracing::warn!("[XhsApiClient] Algo failed for {}: {}, falling back to stored signature", uri, algo_err);
                // 这里需要 endpoint_key 来查找存储的签名，但我们没有
                // 对于纯算法路径，失败即失败
                Err(algo_err)
            }
        }
    }

    /// 执行带动态查询参数的 GET 请求（纯算法签名）
    /// 
    /// 用于需要动态构造查询参数的接口（如 notification）
    /// 使用与 get 方法相同的 path/params 分离逻辑
    /// 
    /// # Arguments
    /// * `uri` - API 完整路径（含查询参数，如 "/api/sns/web/v1/you/likes?num=20&cursor="）
    /// 
    /// # Returns
    /// 响应文本内容
    pub async fn get_with_query(&self, uri: &str) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        
        let cookie_str = credentials.cookie_string();
        
        // 解析 URI，分离 path 和 query params（与 get 方法相同逻辑）
        let (path, params) = parse_uri_with_params(uri);
        let base_url = format!("https://edith.xiaohongshu.com{}", path);
        
        // 尝试纯算法签名
        match self.get_algo_signature("GET", uri, &cookie_str, None).await {
            Ok(signature) => {
                tracing::info!("[XhsApiClient] GET {} using ALGO (path: {}, params: {:?})", uri, path, params);
                // 使用 .query() 传递参数，保持与 get 方法一致
                let response = self.build_get_request_algo(&base_url, &signature, &cookie_str)
                    .query(&params)
                    .send()
                    .await?;
                self.handle_response(response, uri).await
            }
            Err(algo_err) => {
                tracing::warn!("[XhsApiClient] Algo failed for {}: {}", uri, algo_err);
                Err(algo_err)
            }
        }
    }

    /// 执行带自定义 URL 的 GET 请求（纯算法优先）
    /// 
    /// 用于需要动态构造 URL 参数的接口（如 note_page）
    /// 优先使用 Python Agent 纯算法签名
    /// 
    /// # Arguments
    /// * `endpoint_key` - 端点标识（用于日志和回退）
    /// * `url` - 完整的请求 URL（含查询参数）
    pub async fn get_with_url(&self, endpoint_key: &str, url: &str) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        
        let cookie_str = credentials.cookie_string();
        
        // 从 URL 中解析 path 和 params
        if let Some(idx) = url.find("edith.xiaohongshu.com") {
            let uri_start = url[idx..].find('/').map(|i| idx + i).unwrap_or(url.len());
            let uri = &url[uri_start..];
            
            // 解析 path 和 params
            // let (path, params) = parse_uri_with_params(uri);
            
            // 尝试纯算法签名
            match self.get_algo_signature("GET", uri, &cookie_str, None).await {
                Ok(signature) => {
                    // Use URL directly to avoid double encoding of query params by reqwest
                    tracing::info!("[XhsApiClient] GET {} using ALGO (url: {})", endpoint_key, url);
                    let response = self.build_get_request_algo(url, &signature, &cookie_str)
                        .send()
                        .await?;
                    return self.handle_response(response, endpoint_key).await;
                }
                Err(algo_err) => {
                    tracing::warn!("[XhsApiClient] Algo failed for {}: {}, trying stored signature", endpoint_key, algo_err);
                }
            }
        }
        
        // 回退到存储的签名
        let signature = self.get_signature(endpoint_key).await?;
        
        tracing::info!("[XhsApiClient] GET {} with custom URL using STORED signature", endpoint_key);
        
        let response = self.build_get_request(url, &signature, &cookie_str)
            .send()
            .await?;
        
        self.handle_response(response, endpoint_key).await
    }

    /// 执行 POST 请求（纯算法优先 + 存储回退）
    /// 
    /// 优先使用 Python Agent 生成签名
    /// 
    /// # Arguments
    /// * `endpoint_key` - 签名存储的 key（如 "home_feed_recommend"）
    pub async fn post(&self, endpoint_key: &str) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        
        let cookie_str = credentials.cookie_string();
        
        // 优先尝试纯算法签名
        if let Some(uri) = endpoint_to_uri(endpoint_key) {
            let url = format!("https://edith.xiaohongshu.com{}", uri);
            
            // 构建 Home Feed 的默认 payload
            let payload = self.build_default_payload(endpoint_key);
            let body = serde_json::to_string(&payload)?;
            
            match self.get_algo_signature("POST", uri, &cookie_str, Some(payload)).await {
                Ok(signature) => {
                    tracing::info!("[XhsApiClient] POST {} using ALGO", endpoint_key);
                    let response = self.build_post_request_algo(&url, &signature, &cookie_str, body)
                        .send()
                        .await?;
                    return self.handle_response(response, endpoint_key).await;
                }
                Err(algo_err) => {
                    tracing::warn!("[XhsApiClient] Algo failed for {}: {}, trying stored signature", endpoint_key, algo_err);
                }
            }
        }
        
        // 回退到存储的签名
        let signature = self.get_signature(endpoint_key).await?;
        let url = signature.request_url.clone()
            .unwrap_or_else(|| format!("https://edith.xiaohongshu.com/api/sns/web/v1/{}", endpoint_key));
        let body = signature.post_body.clone().unwrap_or_default();
        
        tracing::info!("[XhsApiClient] POST {} using STORED signature", endpoint_key);
        
        let response = self.build_post_request(&url, &signature, &cookie_str, body)
            .send()
            .await?;
        
        self.handle_response(response, endpoint_key).await
    }

    /// 构建 Home Feed 请求的默认 Payload
    fn build_default_payload(&self, endpoint_key: &str) -> serde_json::Value {
        // 从 endpoint_key 提取 category
        let category = if endpoint_key == "home_feed_recommend" {
            "homefeed_recommend".to_string()
        } else if endpoint_key.starts_with("home_feed_") {
            let cat = endpoint_key.strip_prefix("home_feed_").unwrap_or("recommend");
            format!("homefeed.{}_v3", cat)
        } else {
            "homefeed_recommend".to_string()
        };
        
        serde_json::json!({
            "cursor_score": "",
            "num": 20,
            "refresh_type": 1,
            "note_index": 0,
            "unread_begin_note_id": "",
            "unread_end_note_id": "",
            "unread_note_count": 0,
            "category": category,
            "search_key": "",
            "need_num": 18,
            "image_formats": ["jpg", "webp", "avif"],
            "need_filter_image": false
        })
    }

    /// 执行 POST 请求（使用用户提供的 payload）
    /// 
    /// 用于 homefeed 等需要用户控制分页参数的接口
    /// 
    /// # Arguments
    /// * `endpoint_key` - 签名存储的 key（如 "home_feed_fashion"）
    /// * `payload` - 用户提供的完整请求体
    pub async fn post_with_payload(&self, endpoint_key: &str, payload: serde_json::Value) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        
        let cookie_str = credentials.cookie_string();
        
        // 优先尝试纯算法签名
        if let Some(uri) = endpoint_to_uri(endpoint_key) {
            let url = format!("https://edith.xiaohongshu.com{}", uri);
            let body = serde_json::to_string(&payload)?;
            
            // DEBUG: 输出实际发送的 body
            tracing::info!("[XhsApiClient] POST {} body: {}", endpoint_key, body);
            
            match self.get_algo_signature("POST", uri, &cookie_str, Some(payload)).await {
                Ok(signature) => {
                    tracing::info!("[XhsApiClient] POST {} with custom payload using ALGO", endpoint_key);
                    let response = self.build_post_request_algo(&url, &signature, &cookie_str, body)
                        .send()
                        .await?;
                    return self.handle_response(response, endpoint_key).await;
                }
                Err(algo_err) => {
                    tracing::warn!("[XhsApiClient] Algo failed for {}: {}", endpoint_key, algo_err);
                    return Err(algo_err);
                }
            }
        }
        
        Err(anyhow!("No URI mapping for endpoint: {}", endpoint_key))
    }

    /// 执行 POST 请求（纯算法签名优先）
    /// 
    /// 优先使用 Python Agent 生成签名
    /// 
    /// # Arguments
    /// * `uri` - API 路径（如 "/api/sns/web/v1/homefeed"）
    /// * `payload` - JSON 请求体
    /// 
    /// # Returns
    /// 响应文本内容
    pub async fn post_algo(&self, uri: &str, payload: serde_json::Value) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        
        let cookie_str = credentials.cookie_string();
        let url = format!("https://edith.xiaohongshu.com{}", uri);
        let body = serde_json::to_string(&payload)?;
        
        // DEBUG: 输出实际发送的 payload
        tracing::info!("[XhsApiClient] POST {} payload: {}", uri, body);
        
        // 尝试纯算法签名
        match self.get_algo_signature("POST", uri, &cookie_str, Some(payload)).await {
            Ok(signature) => {
                tracing::info!("[XhsApiClient] POST {} using ALGO signature", uri);
                let response = self.build_post_request_algo(&url, &signature, &cookie_str, body)
                    .send()
                    .await?;
                self.handle_response(response, uri).await
            }
            Err(algo_err) => {
                tracing::warn!("[XhsApiClient] Algo failed for {}: {}", uri, algo_err);
                Err(algo_err)
            }
        }
    }

    /// 执行带自定义 body 的 POST 请求
    /// 
    /// 用于需要动态构造请求体的接口
    pub async fn post_with_body(&self, endpoint_key: &str, url: &str, body: String) -> Result<String> {
        let credentials = self.auth.try_get_credentials().await?
            .ok_or_else(|| anyhow!("Not logged in. Please call /api/auth/login-session first."))?;
        let signature = self.get_signature(endpoint_key).await?;
        
        tracing::info!("[XhsApiClient] POST {} with custom body_len: {}", endpoint_key, body.len());
        
        let response = self.build_post_request(url, &signature, &credentials.cookie_string(), body)
            .send()
            .await?;
        
        self.handle_response(response, endpoint_key).await
    }

    // ==================== 私有辅助方法 ====================

    /// 获取指定接口的签名（从存储）
    /// 兜底方法，当纯算法失败时使用
    async fn get_signature(&self, endpoint_key: &str) -> Result<ApiSignature> {
        self.auth.get_endpoint_signature(endpoint_key).await?
            .ok_or_else(|| anyhow!(
                "No signature found for endpoint: {}. Please login again to capture signatures.", 
                endpoint_key
            ))
    }

    /// 获取纯算法签名
    /// 通过 Python Agent 调用 xhshow 库生成签名
    async fn get_algo_signature(
        &self, 
        method: &str, 
        uri: &str, 
        cookie_str: &str,
        payload: Option<serde_json::Value>,
    ) -> Result<Signature> {
        let cookies = parse_cookie_string(cookie_str);
        self.signature_service
            .get_signature_from_agent(method, uri, cookies, payload)
            .await
    }

    /// 构建 GET 请求（使用纯算法签名）
    fn build_get_request_algo(&self, url: &str, signature: &Signature, cookie: &str) -> reqwest::RequestBuilder {
        self.http_client.get_client()
            .get(url)
            .header("accept", "application/json, text/plain, */*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("pragma", "no-cache")
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("user-agent", USER_AGENT)
            .header("origin", ORIGIN)
            .header("referer", REFERER)
            .header("x-s", &signature.x_s)
            .header("x-t", &signature.x_t)
            .header("x-s-common", &signature.x_s_common)
            .header("x-b3-traceid", &signature.x_b3_traceid)
            .header("x-xray-traceid", &signature.x_xray_traceid)
            .header("cookie", cookie)
    }

    /// 构建 POST 请求（使用纯算法签名）
    fn build_post_request_algo(&self, url: &str, signature: &Signature, cookie: &str, body: String) -> reqwest::RequestBuilder {
        self.http_client.get_client()
            .post(url)
            .header("accept", "application/json, text/plain, */*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")  // 修复：添加缺失的 header
            .header("content-type", "application/json;charset=UTF-8")
            .header("pragma", "no-cache")  // 修复：添加缺失的 header
            .header("priority", "u=1, i")
            .header("sec-ch-ua", r#""Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24""#)
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", r#""Windows""#)
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("user-agent", USER_AGENT)
            .header("origin", ORIGIN)
            .header("referer", REFERER)
            .header("x-s", &signature.x_s)
            .header("x-t", &signature.x_t)
            .header("x-s-common", &signature.x_s_common)
            .header("x-b3-traceid", &signature.x_b3_traceid)
            .header("x-xray-traceid", &signature.x_xray_traceid)
            .header("cookie", cookie)
            .body(body)
    }

    /// 构建 GET 请求（含所有 headers）
    fn build_get_request(&self, url: &str, signature: &ApiSignature, cookie: &str) -> reqwest::RequestBuilder {
        self.http_client.get_client()
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
            .header("user-agent", USER_AGENT)
            // XHS specific headers
            .header("origin", ORIGIN)
            .header("referer", REFERER)
            .header("x-s", &signature.x_s)
            .header("x-t", &signature.x_t)
            .header("x-s-common", &signature.x_s_common)
            .header("x-b3-traceid", &signature.x_b3_traceid)
            .header("x-xray-traceid", &signature.x_xray_traceid)
            .header("cookie", cookie)
    }

    /// 构建 POST 请求（含所有 headers）
    fn build_post_request(&self, url: &str, signature: &ApiSignature, cookie: &str, body: String) -> reqwest::RequestBuilder {
        self.http_client.get_client()
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
            .header("user-agent", USER_AGENT)
            // XHS specific headers
            .header("origin", ORIGIN)
            .header("referer", REFERER)
            .header("x-s", &signature.x_s)
            .header("x-t", &signature.x_t)
            .header("x-s-common", &signature.x_s_common)
            .header("x-b3-traceid", &signature.x_b3_traceid)
            .header("x-xray-traceid", &signature.x_xray_traceid)
            .header("xy-direction", "98")
            .header("cookie", cookie)
            .body(body)
    }

    /// 处理响应（日志 + 错误状态码处理）
    async fn handle_response(&self, response: reqwest::Response, endpoint_key: &str) -> Result<String> {
        let status = response.status();
        let text = response.text().await?;
        
        tracing::info!("[XhsApiClient] {} Response [{}]: {} chars", endpoint_key, status, text.len());
        
        // 处理常见错误状态码
        match status.as_u16() {
            406 => {
                tracing::warn!(
                    "[XhsApiClient] {} received 406 - signature may be invalid (cookies are still valid)",
                    endpoint_key
                );
            }
            461 => {
                tracing::warn!(
                    "[XhsApiClient] {} received 461 - XHS rate limit or risk control triggered",
                    endpoint_key
                );
                return Err(anyhow!(
                    "XHS 风控触发 (461): 请稍后重试或更换关键词。Response: {}",
                    text
                ));
            }
            status_code if status_code >= 400 => {
                tracing::warn!(
                    "[XhsApiClient] {} received {} - request failed",
                    endpoint_key, status_code
                );
                return Err(anyhow!(
                    "XHS API 错误 ({}): {}",
                    status_code, text
                ));
            }
            _ => {}
        }
        
        Ok(text)
    }
}
