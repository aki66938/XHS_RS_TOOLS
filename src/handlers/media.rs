//! Media HTTP Handlers
//!
//! Handles: video URL extraction, image URL extraction, media download

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::api::media;
use crate::server::AppState;

// ============================================================================
// Handlers
// ============================================================================

/// 获取视频下载地址
///
/// 从笔记详情中提取所有画质的视频下载 URL
#[utoipa::path(
    post,
    path = "/api/note/video",
    tag = "Media",
    summary = "视频地址解析",
    description = "从视频笔记中提取所有画质的视频下载 URL，返回 CDN 直链",
    request_body = media::video::VideoRequest,
    responses(
        (status = 200, description = "视频地址列表", body = media::video::VideoResponse),
        (status = 500, description = "请求失败")
    )
)]
pub async fn video_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<media::video::VideoRequest>,
) -> impl IntoResponse {
    match media::video::get_video_urls(&state.api, req).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

/// 获取图片下载地址
///
/// 从图文笔记中提取所有图片的下载 URL
/// 返回有水印 (url_watermark) 和无水印 (url_original) 两个版本
#[utoipa::path(
    post,
    path = "/api/note/images",
    tag = "Media",
    summary = "图片地址解析",
    description = "从图文笔记中提取所有图片的下载 URL。返回两个版本：url_watermark (有水印) 和 url_original (无水印)",
    request_body = media::images::ImagesRequest,
    responses(
        (status = 200, description = "图片地址列表", body = media::images::ImagesResponse),
        (status = 500, description = "请求失败")
    )
)]
pub async fn images_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<media::images::ImagesRequest>,
) -> impl IntoResponse {
    match media::images::get_image_urls(&state.api, req).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

/// 下载媒体文件
///
/// 将视频或图片下载到服务端本地目录
#[utoipa::path(
    post,
    path = "/api/media/download",
    tag = "Media",
    summary = "媒体下载",
    description = "将视频或图片文件下载到服务端本地指定路径，支持 xhscdn.com 域名",
    request_body = media::download::DownloadRequest,
    responses(
        (status = 200, description = "下载结果", body = media::download::DownloadResponse),
        (status = 500, description = "下载失败")
    )
)]
pub async fn download_handler(
    Json(req): Json<media::download::DownloadRequest>,
) -> impl IntoResponse {
    match media::download::download_media(req).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}
