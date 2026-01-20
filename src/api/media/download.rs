//! Media Download API
//!
//! Downloads media files (video/image) to local storage

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// 媒体下载请求参数
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DownloadRequest {
    /// 媒体文件 URL (必填)
    /// 支持 xhscdn.com 域名的视频和图片
    pub url: String,
    /// 保存路径 (必填)
    /// 例如: "./downloads/video.mp4"
    pub save_path: String,
}

/// 媒体下载响应
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DownloadResponse {
    pub success: bool,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub data: Option<DownloadData>,
}

/// 下载结果数据
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DownloadData {
    /// 保存的文件路径
    pub saved_path: String,
    /// 文件大小 (bytes)
    pub file_size: u64,
    /// 内容类型 (如 video/mp4, image/jpeg)
    pub content_type: String,
}

/// 允许的 CDN 域名白名单
const ALLOWED_DOMAINS: &[&str] = &[
    "xhscdn.com",
    "xiaohongshu.com",
];

/// 下载媒体文件到本地
///
/// 支持视频和图片的下载
pub async fn download_media(req: DownloadRequest) -> Result<DownloadResponse> {
    // 验证 URL 域名白名单
    if !is_url_allowed(&req.url) {
        return Ok(DownloadResponse {
            success: false,
            msg: Some("URL domain not in whitelist. Only xhscdn.com and xiaohongshu.com are allowed.".to_string()),
            data: None,
        });
    }
    
    // 确保保存目录存在
    let save_path = Path::new(&req.save_path);
    if let Some(parent) = save_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await
                .map_err(|e| anyhow!("Failed to create directory: {}", e))?;
        }
    }
    
    // 创建 HTTP 客户端
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5分钟超时
        .build()?;
    
    // 发送下载请求
    let response = client
        .get(&req.url)
        .header("Accept", "*/*")
        .header("Accept-Language", "zh-CN,zh;q=0.9")
        .header("Origin", "https://www.xiaohongshu.com")
        .header("Referer", "https://www.xiaohongshu.com/")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36")
        .send()
        .await
        .map_err(|e| anyhow!("Failed to download: {}", e))?;
    
    // 检查响应状态
    if !response.status().is_success() {
        return Ok(DownloadResponse {
            success: false,
            msg: Some(format!("Download failed with status: {}", response.status())),
            data: None,
        });
    }
    
    // 获取内容类型
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    
    // 获取文件内容
    let bytes = response.bytes().await
        .map_err(|e| anyhow!("Failed to read response body: {}", e))?;
    
    let file_size = bytes.len() as u64;
    
    // 写入文件
    let mut file = fs::File::create(&req.save_path).await
        .map_err(|e| anyhow!("Failed to create file: {}", e))?;
    
    file.write_all(&bytes).await
        .map_err(|e| anyhow!("Failed to write file: {}", e))?;
    
    file.flush().await
        .map_err(|e| anyhow!("Failed to flush file: {}", e))?;
    
    tracing::info!(
        "[MediaDownload] Downloaded {} -> {} ({} bytes)", 
        req.url, req.save_path, file_size
    );
    
    Ok(DownloadResponse {
        success: true,
        msg: None,
        data: Some(DownloadData {
            saved_path: req.save_path,
            file_size,
            content_type,
        }),
    })
}

/// 检查 URL 是否在白名单中
fn is_url_allowed(url: &str) -> bool {
    for domain in ALLOWED_DOMAINS {
        if url.contains(domain) {
            return true;
        }
    }
    false
}
