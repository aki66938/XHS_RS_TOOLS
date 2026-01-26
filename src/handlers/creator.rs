//! Creator Authentication Handlers
//!
//! Exposes REST endpoints for Creator Center login flow.

use axum::{Json, response::IntoResponse, extract::State};
use std::sync::Arc;
use crate::server::AppState;
use crate::api::creator::{auth, models::{CreatorQrcodeCreateRequest, CreatorQrcodeStatusRequest}};
use crate::api::login::{GuestInitResponse, CreateQrCodeResponse};

/// 1. 初始化创作者访客会话
///
/// 获取创作者中心的访客 Cookie (xsecappid=ugc)
#[utoipa::path(
    post,
    path = "/api/creator/auth/guest-init",
    tag = "Creator",
    responses(
        (status = 200, description = "Guest session initialized", body = GuestInitResponse)
    )
)]
pub async fn creator_guest_init_handler() -> impl IntoResponse {
    match auth::fetch_creator_guest_cookies().await {
        Ok(cookies) => Json(GuestInitResponse {
            success: true,
            cookies: Some(cookies),
            error: None,
        }),
        Err(e) => {
            let resp = GuestInitResponse {
                success: false,
                cookies: None,
                error: Some(e.to_string()),
            };
            Json(resp)
        },
    }
}

/// 2. 申请创作者登录二维码
///
/// 使用访客 Cookie 申请创作者登录二维码
#[utoipa::path(
    post,
    path = "/api/creator/auth/qrcode/create",
    tag = "Creator",
    request_body = CreatorQrcodeCreateRequest, 
    responses(
        (status = 200, description = "QR Code created", body = CreateQrCodeResponse)
    )
)]
pub async fn creator_create_qrcode_handler(
    Json(payload): Json<CreatorQrcodeCreateRequest>
) -> impl IntoResponse {
    match auth::create_creator_qrcode(&payload.cookies).await {
        Ok(response) => {
            let resp = CreateQrCodeResponse {
                success: response.success,
                qr_url: response.data.as_ref().map(|d| d.url.clone()),
                qr_id: response.data.as_ref().map(|d| d.qr_id.clone()),
                code: response.data.as_ref().map(|d| d.code.clone()),
                error: response.msg,
            };
            Json(resp)
        },
        Err(e) => {
            tracing::error!("Create QR failed: {}", e);
            let resp = CreateQrCodeResponse {
                success: false,
                qr_url: None,
                qr_id: None,
                code: None,
                error: Some(e.to_string()),
            };
            Json(resp)
        }
    }
}


/// 3. 轮询创作者登录状态
///
/// 轮询创作者登录状态 (Status 1 = Login Success)
#[utoipa::path(
    post,
    path = "/api/creator/auth/qrcode/status",
    tag = "Creator",
    request_body = CreatorQrcodeStatusRequest, 
    responses(
        (status = 200, description = "Status checked", body = serde_json::Value)
    )
)]
pub async fn creator_check_qrcode_status(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatorQrcodeStatusRequest>
) -> impl IntoResponse {
    match auth::check_creator_qrcode_status(&payload.qr_id, &payload.cookies).await {
        Ok((mut json, new_cookies)) => {
            if let Some(nc) = new_cookies {
                // Save credentials to cookie-creator.json
                let user_id = json.get("data")
                    .and_then(|d| d.get("user_id"))
                    .and_then(|u| u.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                    
                let creds = crate::auth::credentials::UserCredentials::new(
                    user_id.clone(),
                    nc.clone(),
                    None, 
                );
                
                if let Err(e) = state.creator_auth.save_credentials(&creds).await {
                    tracing::error!("Failed to save Creator credentials: {}", e);
                } else {
                    tracing::info!("Saved Creator credentials for user: {}", user_id);
                }

                if let Some(obj) = json.as_object_mut() {
                    obj.insert("new_cookies".to_string(), serde_json::to_value(nc).unwrap_or_default());
                }
            }
            Json(json)
        },
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

// Import for Creator Info Handlers
use crate::api::creator::{info, models::{CreatorUserInfo, CreatorHomeInfo}};

/// 4. 获取创作者用户信息
///
/// 获取创作者中心用户基础信息
#[utoipa::path(
    get,
    path = "/api/galaxy/user/info",
    tag = "Creator",
    responses(
        (status = 200, description = "User info retrieved", body = CreatorUserInfo)
    )
)]
pub async fn creator_user_info_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 1. Get credentials from creator_auth
    let cookies_result = state.creator_auth.try_get_credentials().await;
    
    let cookies = match cookies_result {
        Ok(Some(creds)) => creds.cookies.clone(),
        Ok(None) => return Json(serde_json::json!({
            "success": false,
            "error": "Not logged in (Creator). Please login first."
        })).into_response(),
        Err(e) => return Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })).into_response(),
    };
    
    // 2. Call API
    match info::get_creator_user_info(&cookies).await {
        Ok(info) => Json(serde_json::json!({
            "success": true,
            "data": info
        })).into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false, 
            "error": e.to_string()
        })).into_response(),
    }
}

/// 5. 获取创作者主页信息
///
/// 获取创作者中心个人主页信息
#[utoipa::path(
    get,
    path = "/api/galaxy/creator/home/personal_info",
    tag = "Creator",
    responses(
        (status = 200, description = "Home info retrieved", body = CreatorHomeInfo)
    )
)]
pub async fn creator_home_info_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 1. Get credentials from creator_auth
    let cookies_result = state.creator_auth.try_get_credentials().await;
    
    let cookies = match cookies_result {
        Ok(Some(creds)) => creds.cookies.clone(),
        Ok(None) => return Json(serde_json::json!({
            "success": false,
            "error": "Not logged in (Creator). Please login first."
        })).into_response(),
        Err(e) => return Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })).into_response(),
    };
    
    // 2. Call API
    match info::get_creator_home_info(&cookies).await {
        Ok(info) => Json(serde_json::json!({
            "success": true,
            "data": info
        })).into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false, 
            "error": e.to_string()
        })).into_response(),
    }
}
