//! Video URL Extraction API
//!
//! Extracts video download URLs from note details

use crate::api::XhsApiClient;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 视频地址请求参数
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct VideoRequest {
    /// 笔记 ID (必填)
    pub note_id: String,
    /// xsec_token (必填，从 feed/search 结果获取)
    pub xsec_token: String,
}

/// 视频地址响应
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct VideoResponse {
    pub success: bool,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub data: Option<VideoData>,
}

/// 视频数据
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct VideoData {
    /// 笔记 ID
    pub note_id: String,
    /// 笔记标题
    pub title: String,
    /// 作者昵称
    pub author: String,
    /// 视频时长 (ms)
    pub duration: i64,
    /// 视频列表 (多种画质)
    pub videos: Vec<VideoItem>,
    /// 封面图 URL
    #[serde(default)]
    pub cover: Option<String>,
}

/// 单个视频项
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct VideoItem {
    /// 画质标识 (如 h265_1080p, h264_720p)
    pub quality: String,
    /// 主下载 URL
    pub url: String,
    /// 备用下载 URL
    #[serde(default)]
    pub backup_url: Option<String>,
    /// 视频宽度
    pub width: i32,
    /// 视频高度
    pub height: i32,
    /// 文件大小 (bytes)
    pub size: i64,
    /// 编码格式 (hevc/h264)
    pub codec: String,
}

/// 获取视频下载地址
///
/// 从笔记详情中提取所有画质的视频下载 URL
pub async fn get_video_urls(api: &XhsApiClient, req: VideoRequest) -> Result<VideoResponse> {
    let path = "/api/sns/web/v1/feed";
    
    // 构造请求体
    let payload = serde_json::json!({
        "source_note_id": req.note_id,
        "image_formats": ["jpg", "webp", "avif"],
        "xsec_source": "pc_feed",
        "xsec_token": req.xsec_token,
        "extra": {"need_body_topic": "1"}
    });
    
    let text = api.post_algo(path, payload).await?;
    let raw: serde_json::Value = serde_json::from_str(&text)?;
    
    // 检查响应状态
    if raw.get("success").and_then(|v| v.as_bool()) != Some(true) {
        let msg = raw.get("msg").and_then(|v| v.as_str()).unwrap_or("Unknown error");
        return Ok(VideoResponse {
            success: false,
            msg: Some(msg.to_string()),
            data: None,
        });
    }
    
    // 提取笔记卡片
    let note_card = raw
        .pointer("/data/items/0/note_card")
        .ok_or_else(|| anyhow!("No note_card found in response"))?;
    
    // 检查是否为视频类型
    let note_type = note_card.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if note_type != "video" {
        return Ok(VideoResponse {
            success: false,
            msg: Some("This note is not a video".to_string()),
            data: None,
        });
    }
    
    // 提取基本信息
    let title = note_card.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let author = note_card
        .pointer("/user/nickname")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    // 提取视频时长
    let duration = note_card
        .pointer("/video/capa/duration")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) * 1000; // 转为毫秒
    
    // 提取封面
    let cover = note_card
        .pointer("/image_list/0/url_default")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // 提取视频流
    let mut videos = Vec::new();
    
    // 解析 h265 流
    if let Some(h265_streams) = note_card.pointer("/video/media/stream/h265").and_then(|v| v.as_array()) {
        for stream in h265_streams {
            if let Some(item) = parse_video_stream(stream, "hevc") {
                videos.push(item);
            }
        }
    }
    
    // 解析 h264 流
    if let Some(h264_streams) = note_card.pointer("/video/media/stream/h264").and_then(|v| v.as_array()) {
        for stream in h264_streams {
            if let Some(item) = parse_video_stream(stream, "h264") {
                videos.push(item);
            }
        }
    }
    
    // 按文件大小降序排列 (最高画质在前)
    videos.sort_by(|a, b| b.size.cmp(&a.size));
    
    Ok(VideoResponse {
        success: true,
        msg: None,
        data: Some(VideoData {
            note_id: req.note_id,
            title,
            author,
            duration,
            videos,
            cover,
        }),
    })
}

/// 解析单个视频流
fn parse_video_stream(stream: &serde_json::Value, codec: &str) -> Option<VideoItem> {
    let master_url = stream.get("master_url")?.as_str()?;
    let width = stream.get("width")?.as_i64()? as i32;
    let height = stream.get("height")?.as_i64()? as i32;
    let size = stream.get("size")?.as_i64()?;
    
    // 构建画质标识
    let quality = format!("{}_{}", codec, 
        if height >= 1080 { "1080p" } 
        else if height >= 720 { "720p" } 
        else { "480p" }
    );
    
    // 获取备用 URL
    let backup_url = stream
        .get("backup_urls")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Some(VideoItem {
        quality,
        url: master_url.to_string(),
        backup_url,
        width,
        height,
        size,
        codec: codec.to_string(),
    })
}
