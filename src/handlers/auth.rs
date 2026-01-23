//! Authentication HTTP Handlers
//! 
//! Handles: guest-init, qrcode/create, qrcode/status

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::api;
use crate::server::AppState;
use crate::api::login::{GuestInitResponse, CreateQrCodeResponse, PollStatusResponse};

// ============================================================================
// Handlers
// ============================================================================

/// 初始化访客登录会话
///
/// 通过 Playwright 获取访客 Cookie，存储到内存中供后续 QR 登录使用
#[utoipa::path(
    post,
    path = "/api/auth/guest-init",
    tag = "auth",
    summary = "初始化访客会话",
    description = "获取访客 Cookie，这是 QR 登录的第一步",
    responses(
        (status = 200, description = "访客 Cookie", body = GuestInitResponse)
    )
)]
pub async fn guest_init_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::info!("Guest init requested");
    
    match api::login::fetch_guest_cookies().await {
        Ok(cookies) => {
            // Store cookies in state
            {
                let mut guest = state.guest_cookies.write().await;
                *guest = Some(cookies.clone());
            }
            
            tracing::info!("Guest cookies obtained successfully");
            Json(GuestInitResponse {
                success: true,
                cookies: Some(cookies),
                error: None,
            }).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get guest cookies: {}", e);
            Json(GuestInitResponse {
                success: false,
                cookies: None,
                error: Some(e.to_string()),
            }).into_response()
        }
    }
}

/// 创建登录二维码
///
/// 使用访客 Cookie 调用官方 API 创建二维码
#[utoipa::path(
    post,
    path = "/api/auth/qrcode/create",
    tag = "auth",
    summary = "创建登录二维码",
    description = "需要先调用 guest-init 获取访客 Cookie",
    responses(
        (status = 200, description = "二维码信息", body = CreateQrCodeResponse)
    )
)]
pub async fn create_qrcode_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Get guest cookies
    let cookies = {
        let guard = state.guest_cookies.read().await;
        guard.clone()
    };
    
    let cookies = match cookies {
        Some(c) => c,
        None => {
            return Json(CreateQrCodeResponse {
                success: false,
                qr_url: None,
                qr_id: None,
                code: None,
                error: Some("请先调用 /api/auth/guest-init 获取访客 Cookie".to_string()),
            }).into_response();
        }
    };
    
    match api::login::create_qrcode(&cookies).await {
        Ok(resp) => {
            if resp.success {
                if let Some(data) = resp.data {
                    // Store qr_id and code for polling
                    {
                        let mut info = state.qrcode_info.write().await;
                        *info = Some((data.qr_id.clone(), data.code.clone()));
                    }
                    
                    Json(CreateQrCodeResponse {
                        success: true,
                        qr_url: Some(data.url),
                        qr_id: Some(data.qr_id),
                        code: Some(data.code),
                        error: None,
                    }).into_response()
                } else {
                    Json(CreateQrCodeResponse {
                        success: false,
                        qr_url: None,
                        qr_id: None,
                        code: None,
                        error: Some("QR code data missing".to_string()),
                    }).into_response()
                }
            } else {
                Json(CreateQrCodeResponse {
                    success: false,
                    qr_url: None,
                    qr_id: None,
                    code: None,
                    error: resp.msg,
                }).into_response()
            }
        }
        Err(e) => {
            Json(CreateQrCodeResponse {
                success: false,
                qr_url: None,
                qr_id: None,
                code: None,
                error: Some(e.to_string()),
            }).into_response()
        }
    }
}

/// 轮询二维码登录状态
///
/// - code_status=0: 等待扫码
/// - code_status=1: 已扫码，等待确认
/// - code_status=2: 登录成功
#[utoipa::path(
    get,
    path = "/api/auth/qrcode/status",
    tag = "auth",
    summary = "轮询二维码状态",
    description = "轮询直到 code_status=2 表示登录成功",
    responses(
        (status = 200, description = "二维码状态", body = PollStatusResponse)
    )
)]
pub async fn poll_qrcode_status_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Get guest cookies
    let cookies = {
        let guard = state.guest_cookies.read().await;
        guard.clone()
    };
    
    let cookies = match cookies {
        Some(c) => c,
        None => {
            return Json(PollStatusResponse {
                success: false,
                code_status: -1,
                login_info: None,
                new_cookies: None,
                error: Some("请先调用 /api/auth/guest-init".to_string()),
            }).into_response();
        }
    };
    
    // Get qr_id and code
    let qrcode_info = {
        let guard = state.qrcode_info.read().await;
        guard.clone()
    };
    
    let (qr_id, code) = match qrcode_info {
        Some(info) => info,
        None => {
            return Json(PollStatusResponse {
                success: false,
                code_status: -1,
                login_info: None,
                new_cookies: None,
                error: Some("请先调用 /api/auth/qrcode/create".to_string()),
            }).into_response();
        }
    };
    
    match api::login::check_qrcode_status(&cookies, &qr_id, &code).await {
        Ok((resp, new_cookies)) => {
            let code_status = resp.data
                .as_ref()
                .and_then(|d| d.code_status)
                .unwrap_or(-1);
            
            let login_info = resp.data.as_ref().and_then(|d| d.login_info.clone());
            
            // If login success, use FULL synced cookies (NOT merged with guest cookies)
            // This prevents 461 errors caused by mixing guest and user cookies
            if code_status == 2 {
                if let Some(ref new_c) = new_cookies {
                    // FULL REPLACEMENT: Use only the synced cookies, do NOT merge with guest cookies
                    let final_cookies = new_c.clone();
                    
                    tracing::info!("Using FULL synced cookies ({} total), NOT merging with guest cookies.", final_cookies.len());
                    
                    // Extract user_id from login_info or use a default
                    let user_id = login_info
                        .as_ref()
                        .and_then(|info| info.user_id.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    // Create and save credentials with ONLY the synced cookies
                    let creds = crate::auth::credentials::UserCredentials::new(
                        user_id.clone(),
                        final_cookies,
                        None, // No x_s_common in pure algo mode
                    );
                    
                    match state.auth.save_credentials(&creds).await {
                        Ok(_) => {
                            tracing::info!("Login successful! Credentials saved for user: {}", user_id);
                        }
                        Err(e) => {
                            tracing::error!("Failed to save credentials: {}", e);
                        }
                    }
                }
            }
            
            Json(PollStatusResponse {
                success: resp.success,
                code_status,
                login_info,
                new_cookies,
                error: None,
            }).into_response()
        }
        Err(e) => {
            Json(PollStatusResponse {
                success: false,
                code_status: -1,
                login_info: None,
                new_cookies: None,
                error: Some(e.to_string()),
            }).into_response()
        }
    }
}
