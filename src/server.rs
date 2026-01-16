use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::{self, XhsApiClient},
    auth::AuthService,
    client::XhsClient,
    models::{
        feed::{HomefeedRequest, HomefeedResponse, HomefeedData, HomefeedItem, NoteCard, NoteUser, NoteCover, CoverImageInfo, InteractInfo, NoteVideo, VideoCapa},
        search::{QueryTrendingResponse, QueryTrendingData, TrendingQuery, TrendingHintWord},
        user::{UserMeResponse, UserInfo},
    },
    api::notification::{
        mentions::{MentionsResponse, MentionsData},
        connections::{ConnectionsResponse, ConnectionsData},
        likes::{LikesResponse, LikesData},
    },
    api::login::{GuestInitResponse, CreateQrCodeResponse, PollStatusResponse, QrCodeStatusData, LoginInfo},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        query_trending_handler,
        user_me_handler,
        guest_init_handler,
        create_qrcode_handler,
        poll_qrcode_status_handler,
        api::feed::category::get_category_feed,
        api::note::page::get_note_page,
        mentions_handler,
        connections_handler,
        likes_handler,
    ),
    components(
        schemas(
            GuestInitResponse, CreateQrCodeResponse, PollStatusResponse, QrCodeStatusData, LoginInfo,
            QueryTrendingResponse, QueryTrendingData, TrendingQuery, TrendingHintWord,
            UserMeResponse, UserInfo,
            MentionsResponse, MentionsData,
            ConnectionsResponse, ConnectionsData,
            LikesResponse, LikesData,
            HomefeedRequest, HomefeedResponse, HomefeedData, HomefeedItem, NoteCard, NoteUser, NoteCover, CoverImageInfo, InteractInfo, NoteVideo, VideoCapa
        )
    ),
    tags(
        (name = "xhs", description = "小红书 API 接口"),
        (name = "auth", description = "认证相关"),
        (name = "Feed", description = "主页发现频道：recommend(推荐)、fashion(穿搭)、food(美食)、cosmetics(彩妆)、movie_and_tv(影视)、career(职场)、love(情感)、household_product(家居)、gaming(游戏)、travel(旅行)、fitness(健身)"),
        (name = "Note", description = "笔记相关接口")
    )
)]
struct ApiDoc;

pub struct AppState {
    pub api: XhsApiClient,
    pub auth: Arc<AuthService>,
    /// Guest cookies for QR login (populated by guest-init)
    pub guest_cookies: Arc<RwLock<Option<std::collections::HashMap<String, String>>>>,
    /// Current QR code info (qr_id, code)
    pub qrcode_info: Arc<RwLock<Option<(String, String)>>>,
}


/// 猜你想搜
/// 
/// 获取小红书首页搜索框的热门搜索推荐词
#[utoipa::path(
    get,
    path = "/api/search/trending",
    tag = "xhs",
    summary = "猜你想搜",
    responses(
        (status = 200, description = "热门搜索词列表", body = QueryTrendingResponse)
    )
)]
async fn query_trending_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::search::query_trending(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

/// 页面-我
/// 
/// 获取当前登录用户的个人信息
#[utoipa::path(
    get,
    path = "/api/user/me",
    tag = "xhs",
    summary = "页面-我",
    responses(
        (status = 200, description = "当前用户信息（未登录时返回 Not logged in）", body = UserMeResponse)
    )
)]
async fn user_me_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::user::get_current_user(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}


/// 页面-主页发现-推荐
/// 
/// 获取小红书主页推荐内容流
#[utoipa::path(
    post,
    path = "/api/feed/homefeed/recommend",
    tag = "xhs",
    summary = "页面-主页发现-推荐",
    request_body = HomefeedRequest,
    responses(
        (status = 200, description = "主页推荐内容", body = HomefeedResponse)
    )
)]
async fn homefeed_recommend_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::feed::recommend::get_homefeed_recommend(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}




// ============================================================================
// Login Handlers (Pure Rust + Python Agent)
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
async fn guest_init_handler(
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
async fn create_qrcode_handler(
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
async fn poll_qrcode_status_handler(
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
            
            // If login success, merge new cookies with guest cookies and save
            if code_status == 2 {
                if let Some(ref new_c) = new_cookies {
                    let mut merged = cookies.clone();
                    merged.extend(new_c.clone());
                    
                    // Extract user_id from login_info or use a default
                    let user_id = login_info
                        .as_ref()
                        .and_then(|info| info.user_id.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    // Create and save credentials
                    let creds = crate::auth::credentials::UserCredentials::new(
                        user_id.clone(),
                        merged,
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


/// 通知页-评论和@
/// 
/// 获取评论和@通知列表
#[utoipa::path(
    get,
    path = "/api/notification/mentions",
    tag = "xhs",
    summary = "通知页-评论和@",
    responses(
        (status = 200, description = "评论和@通知列表")
    )
)]
async fn mentions_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::notification::mentions::get_mentions(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

/// 通知页-新增关注
/// 
/// 获取新增关注通知列表
#[utoipa::path(
    get,
    path = "/api/notification/connections",
    tag = "xhs",
    summary = "通知页-新增关注",
    responses(
        (status = 200, description = "新增关注通知列表")
    )
)]
async fn connections_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::notification::connections::get_connections(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

/// 通知页-赞和收藏
/// 
/// 获取赞和收藏通知列表
#[utoipa::path(
    get,
    path = "/api/notification/likes",
    tag = "xhs",
    summary = "通知页-赞和收藏",
    responses(
        (status = 200, description = "赞和收藏通知列表")
    )
)]
async fn likes_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::notification::likes::get_likes(&state.api).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

pub async fn start_server() -> anyhow::Result<()> {
    // Initialize MongoDB connection
    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    
    tracing::info!("Initializing AuthService with MongoDB...");
    let auth = Arc::new(AuthService::new(&mongodb_uri).await?);
    
    let client = XhsClient::new()?;
    let api = XhsApiClient::new(client, auth.clone());
    
    // Initialize shared state for login flow
    let guest_cookies = Arc::new(RwLock::new(None));
    let qrcode_info = Arc::new(RwLock::new(None));
    
    let state = Arc::new(AppState { api, auth, guest_cookies, qrcode_info });

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/search/trending", get(query_trending_handler))
        .route("/api/user/me", get(user_me_handler))
        .route("/api/feed/homefeed/recommend", post(homefeed_recommend_handler))
        .route("/api/feed/homefeed/:category", post(api::feed::category::get_category_feed))
        .route("/api/note/page", get(api::note::page::get_note_page))
        .route("/api/notification/mentions", get(mentions_handler))
        .route("/api/notification/connections", get(connections_handler))
        .route("/api/notification/likes", get(likes_handler))
        // New login flow routes
        .route("/api/auth/guest-init", post(guest_init_handler))
        .route("/api/auth/qrcode/create", post(create_qrcode_handler))
        .route("/api/auth/qrcode/status", get(poll_qrcode_status_handler))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    // Get port from environment variable, default to 3000
    let port = std::env::var("PORT")
        .or_else(|_| std::env::var("XHS_API_PORT"))
        .unwrap_or_else(|_| "3000".to_string());
    
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Server running on http://{}/swagger-ui/", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

