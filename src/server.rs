//! Server Core Module
//! 
//! Contains AppState, routing configuration, and server startup logic.
//! All handlers are delegated to the `handlers` module.

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::{self, XhsApiClient},
    auth::AuthService,
    client::XhsClient,
    handlers,
    openapi::ApiDoc,
};

// ============================================================================
// Application State
// ============================================================================

pub struct AppState {
    pub api: XhsApiClient,
    pub auth: Arc<AuthService>,
    /// Guest cookies for QR login (populated by guest-init)
    pub guest_cookies: Arc<RwLock<Option<std::collections::HashMap<String, String>>>>,
    /// Current QR code info (qr_id, code)
    pub qrcode_info: Arc<RwLock<Option<(String, String)>>>,
}

// ============================================================================
// Server Startup
// ============================================================================

pub async fn start_server() -> anyhow::Result<()> {
    // Initialize AuthService (uses JSON file storage)
    tracing::info!("Initializing AuthService with JSON file storage...");
    let auth = Arc::new(AuthService::new().await?);
    
    let client = XhsClient::new()?;
    let api = XhsApiClient::new(client, auth.clone());
    
    // Initialize shared state for login flow
    let guest_cookies = Arc::new(RwLock::new(None));
    let qrcode_info = Arc::new(RwLock::new(None));
    
    let state = Arc::new(AppState { api, auth, guest_cookies, qrcode_info });

    let app = Router::new()
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        
        // Search routes
        .route("/api/search/trending", get(handlers::query_trending_handler))
        .route("/api/search/recommend", get(handlers::search_recommend_handler))
        .route("/api/search/notes", post(handlers::search_notes_handler))
        .route("/api/search/onebox", post(handlers::search_onebox_handler))
        .route("/api/search/filter", get(handlers::search_filter_handler))
        .route("/api/search/usersearch", post(handlers::search_user_handler))
        
        // User routes
        .route("/api/user/me", get(handlers::user_me_handler))
        
        // Feed routes
        .route("/api/feed/homefeed/recommend", post(handlers::homefeed_recommend_handler))
        .route("/api/feed/homefeed/:category", post(api::feed::category::get_category_feed))
        
        // Note routes
        .route("/api/note/page", get(api::note::page::get_note_page))
        .route("/api/note/detail", post(api::note::detail::get_note_detail))
        
        // Notification routes
        .route("/api/notification/mentions", get(handlers::mentions_handler))
        .route("/api/notification/connections", get(handlers::connections_handler))
        .route("/api/notification/likes", get(handlers::likes_handler))
        
        // Media routes
        .route("/api/note/video", post(handlers::video_handler))
        .route("/api/note/images", post(handlers::images_handler))
        .route("/api/media/download", post(handlers::download_handler))
        
        // Auth routes
        .route("/api/auth/guest-init", post(handlers::guest_init_handler))
        .route("/api/auth/qrcode/create", post(handlers::create_qrcode_handler))
        .route("/api/auth/qrcode/status", get(handlers::poll_qrcode_status_handler))
        
        // Middleware
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
