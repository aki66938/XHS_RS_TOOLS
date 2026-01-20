//! Image URL Extraction API
//!
//! Extracts image download URLs from note details

use crate::api::XhsApiClient;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 图片地址请求参数
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ImagesRequest {
    /// 笔记 ID (必填)
    pub note_id: String,
    /// xsec_token (必填，从 feed/search 结果获取)
    pub xsec_token: String,
}

/// 图片地址响应
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ImagesResponse {
    pub success: bool,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub data: Option<ImagesData>,
}

/// 图片数据
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ImagesData {
    /// 笔记 ID
    pub note_id: String,
    /// 笔记标题
    pub title: String,
    /// 作者昵称
    pub author: String,
    /// 笔记描述
    #[serde(default)]
    pub desc: Option<String>,
    /// 图片数量
    pub image_count: usize,
    /// 图片列表
    pub images: Vec<ImageItem>,
}

/// 单个图片项
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ImageItem {
    /// 图片索引 (从1开始)
    pub index: usize,
    /// 图片宽度
    pub width: i32,
    /// 图片高度
    pub height: i32,
    /// 有水印图片 URL (url_default / WB_DFT)
    pub url_watermark: String,
    /// 无水印图片 URL (url_pre / WB_PRV)
    pub url_original: String,
}

/// 获取图片下载地址
///
/// 从笔记详情中提取所有图片的下载 URL
/// 返回有水印和无水印两个版本
pub async fn get_image_urls(api: &XhsApiClient, req: ImagesRequest) -> Result<ImagesResponse> {
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
        return Ok(ImagesResponse {
            success: false,
            msg: Some(msg.to_string()),
            data: None,
        });
    }
    
    // 提取笔记卡片
    let note_card = raw
        .pointer("/data/items/0/note_card")
        .ok_or_else(|| anyhow!("No note_card found in response"))?;
    
    // 检查笔记类型 (normal = 图文笔记)
    let note_type = note_card.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if note_type == "video" {
        return Ok(ImagesResponse {
            success: false,
            msg: Some("This note is a video, not an image note. Use /api/note/video instead.".to_string()),
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
    let desc = note_card.get("desc").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    // 提取图片列表
    let mut images = Vec::new();
    
    if let Some(image_list) = note_card.get("image_list").and_then(|v| v.as_array()) {
        for (idx, img) in image_list.iter().enumerate() {
            if let Some(item) = parse_image_item(img, idx + 1) {
                images.push(item);
            }
        }
    }
    
    if images.is_empty() {
        return Ok(ImagesResponse {
            success: false,
            msg: Some("No images found in this note".to_string()),
            data: None,
        });
    }
    
    Ok(ImagesResponse {
        success: true,
        msg: None,
        data: Some(ImagesData {
            note_id: req.note_id,
            title,
            author,
            desc,
            image_count: images.len(),
            images,
        }),
    })
}

/// 解析单张图片
fn parse_image_item(img: &serde_json::Value, index: usize) -> Option<ImageItem> {
    let width = img.get("width")?.as_i64()? as i32;
    let height = img.get("height")?.as_i64()? as i32;
    
    // 优先从 url_pre / url_default 获取
    let url_original = img.get("url_pre")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            // 回退到 info_list 中的 WB_PRV
            img.get("info_list")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.iter().find(|item| {
                    item.get("image_scene").and_then(|s| s.as_str()) == Some("WB_PRV")
                }))
                .and_then(|item| item.get("url").and_then(|v| v.as_str()))
        })?
        .to_string();
    
    let url_watermark = img.get("url_default")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            // 回退到 info_list 中的 WB_DFT
            img.get("info_list")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.iter().find(|item| {
                    item.get("image_scene").and_then(|s| s.as_str()) == Some("WB_DFT")
                }))
                .and_then(|item| item.get("url").and_then(|v| v.as_str()))
        })?
        .to_string();
    
    Some(ImageItem {
        index,
        width,
        height,
        url_watermark,
        url_original,
    })
}
