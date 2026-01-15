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
        login::{QrCodeSessionResponse, SessionInfoResponse, SessionInfoData, CookieInfo, QrCodeStatusResponse, QrCodeStatusData, LoginInfo, CaptureStatusData, CaptureStatusResponse},
        search::{QueryTrendingResponse, QueryTrendingData, TrendingQuery, TrendingHintWord},
        user::{UserMeResponse, UserInfo},
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        query_trending_handler,
        user_me_handler,
        start_login_session_handler,
        get_session_handler,
        get_qrcode_status_handler,
        get_capture_status_handler,
        api::feed::category::get_category_feed,
        api::note::page::get_note_page,
        mentions_handler,
        connections_handler,
    ),
    components(
        schemas(
            QrCodeSessionResponse, SessionInfoResponse, SessionInfoData, CookieInfo,
            QrCodeStatusResponse, QrCodeStatusData, LoginInfo,
            CaptureStatusData, CaptureStatusResponse,
            QueryTrendingResponse, QueryTrendingData, TrendingQuery, TrendingHintWord,
            UserMeResponse, UserInfo,
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
    /// 共享的二维码登录状态（由 Python 脚本更新）
    pub qrcode_status: Arc<RwLock<Option<QrCodeStatusData>>>,
    /// 共享的采集状态（由 Python 脚本更新）
    pub capture_status: Arc<RwLock<Option<CaptureStatusData>>>,
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


/// Start Login Session (Streamed Response)
///
/// Returns a JSON stream. First message contains QR code. Subsequent messages stream login status updates.
#[utoipa::path(
    post,
    path = "/api/auth/login-session",
    tag = "auth",
    responses(
        (status = 200, description = "Login Session Stream (JSON Lines)", body = QrCodeSessionResponse)
    )
)]
async fn start_login_session_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 重置 qrcode_status 状态
    {
        let mut status = state.qrcode_status.write().await;
        *status = Some(QrCodeStatusData {
            code_status: 0,  // 初始状态：未扫码
            login_info: None,
        });
    }
    
    // 重置 capture_status 状态
    {
        let mut status = state.capture_status.write().await;
        *status = Some(CaptureStatusData {
            is_complete: false,
            signatures_captured: vec![],
            total_count: 0,
            message: "采集中...".to_string(),
        });
    }
    
    match api::login::start_login_session(
        state.qrcode_status.clone(),
        state.capture_status.clone()
    ).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })).into_response(),
    }
}

/// Get Current Session Info
///
/// View stored cookies and session details. Cookie values are masked for security.
#[utoipa::path(
    get,
    path = "/api/auth/session",
    tag = "auth",
    responses(
        (status = 200, description = "Current Session Information", body = SessionInfoResponse)
    )
)]
async fn get_session_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match api::login::get_session_info(&state.auth).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => Json(serde_json::json!({
            "code": -1,
            "success": false,
            "msg": e.to_string(),
            "data": null
        })).into_response(),
    }
}

/// 获取二维码登录状态
///
/// 轮询此接口获取扫码状态:
/// - code_status=0: 未扫码
/// - code_status=1: 已扫码，等待确认
/// - code_status=2: 登录成功 (包含 login_info)
#[utoipa::path(
    get,
    path = "/api/auth/qrcode-status",
    tag = "auth",
    summary = "获取二维码登录状态",
    responses(
        (status = 200, description = "二维码登录状态", body = QrCodeStatusResponse)
    )
)]
async fn get_qrcode_status_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let status = state.qrcode_status.read().await;
    match status.as_ref() {
        Some(data) => Json(QrCodeStatusResponse {
            code: 0,
            success: true,
            msg: "成功".to_string(),
            data: Some(data.clone()),
        }).into_response(),
        None => Json(QrCodeStatusResponse {
            code: 0,
            success: true,
            msg: "等待扫码".to_string(),
            data: Some(QrCodeStatusData {
                code_status: -1,  // -1 表示登录流程未开始
                login_info: None,
            }),
        }).into_response(),
    }
}

/// 获取采集任务状态
///
/// 轮询此接口检查 Playwright 签名采集是否完成。
/// 只有 is_complete=true 时，才能安全调用其他需要签名的 API。
#[utoipa::path(
    get,
    path = "/api/auth/capture-status",
    tag = "auth",
    summary = "获取采集任务状态",
    responses(
        (status = 200, description = "采集任务状态", body = CaptureStatusResponse)
    )
)]
async fn get_capture_status_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let status = state.capture_status.read().await;
    match status.as_ref() {
        Some(data) => Json(CaptureStatusResponse {
            code: 0,
            success: true,
            msg: "成功".to_string(),
            data: Some(data.clone()),
        }).into_response(),
        None => Json(CaptureStatusResponse {
            code: 0,
            success: true,
            msg: "采集未开始".to_string(),
            data: Some(CaptureStatusData {
                is_complete: false,
                signatures_captured: vec![],
                total_count: 0,
                message: "等待登录流程启动".to_string(),
            }),
        }).into_response(),
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

pub async fn start_server() -> anyhow::Result<()> {
    // Initialize MongoDB connection
    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    
    tracing::info!("Initializing AuthService with MongoDB...");
    let auth = Arc::new(AuthService::new(&mongodb_uri).await?);
    
    let client = XhsClient::new()?;
    let api = XhsApiClient::new(client, auth.clone());
    
    // 初始化共享状态
    let qrcode_status = Arc::new(RwLock::new(None));
    let capture_status = Arc::new(RwLock::new(None));
    
    let state = Arc::new(AppState { api, auth, qrcode_status, capture_status });

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/search/trending", get(query_trending_handler))
        .route("/api/user/me", get(user_me_handler))
        .route("/api/feed/homefeed/recommend", post(homefeed_recommend_handler))
        .route("/api/feed/homefeed/:category", post(api::feed::category::get_category_feed))
        .route("/api/note/page", get(api::note::page::get_note_page))
        .route("/api/notification/mentions", get(mentions_handler))
        .route("/api/notification/connections", get(connections_handler))
        .route("/api/auth/login-session", post(start_login_session_handler))
        .route("/api/auth/session", get(get_session_handler))
        .route("/api/auth/qrcode-status", get(get_qrcode_status_handler))
        .route("/api/auth/capture-status", get(get_capture_status_handler))
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
