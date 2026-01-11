//! Authentication service that manages credentials and triggers browser login when needed

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::auth::{CredentialStorage, UserCredentials};
use crate::auth::browser::trigger_python_login;

/// Global authentication state
pub struct AuthService {
    storage: CredentialStorage,
    cached_credentials: Arc<RwLock<Option<UserCredentials>>>,
}

impl AuthService {
    /// Create a new authentication service
    pub async fn new(mongodb_uri: &str) -> Result<Self> {
        let storage = CredentialStorage::new(mongodb_uri).await?;
        
        // Try to load existing credentials
        let cached = storage.get_active_credentials().await?;
        
        if cached.is_some() {
            info!("Loaded existing credentials from database");
        }
        
        Ok(Self {
            storage,
            cached_credentials: Arc::new(RwLock::new(cached)),
        })
    }
    
    /// Get current credentials passively (check cache and DB only)
    /// Returns None if no valid credentials found, does NOT trigger login
    pub async fn try_get_credentials(&self) -> Result<Option<UserCredentials>> {
        // Check cache first
        {
            let cache = self.cached_credentials.read().await;
            if let Some(ref creds) = *cache {
                if creds.is_valid && !creds.is_potentially_expired() {
                    return Ok(Some(creds.clone()));
                }
            }
        }
        
        // Try to load from database
        if let Some(creds) = self.storage.get_active_credentials().await? {
            let mut cache = self.cached_credentials.write().await;
            *cache = Some(creds.clone());
            return Ok(Some(creds));
        }
        
        Ok(None)
    }

    /// Get current credentials, triggering login if needed
    pub async fn get_credentials(&self) -> Result<UserCredentials> {
        // Try passive retrieval first
        if let Some(creds) = self.try_get_credentials().await? {
            return Ok(creds);
        }
        
        // No valid credentials - need to trigger login
        info!("No valid credentials found, triggering browser login...");
        self.trigger_login().await?;
        
        // Reload credentials from database after login
        if let Some(creds) = self.storage.get_active_credentials().await? {
            let mut cache = self.cached_credentials.write().await;
            *cache = Some(creds.clone());
            return Ok(creds);
        }
        
        Err(anyhow::anyhow!("Failed to get credentials after login"))
    }
    
    /// Trigger browser-based login using Python Playwright
    pub async fn trigger_login(&self) -> Result<()> {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║           需要登录小红书                                    ║");
        println!("║  即将打开浏览器，请在浏览器中扫码登录                        ║");
        println!("╚════════════════════════════════════════════════════════════╝\n");
        
        // Run Python Playwright script (which saves to MongoDB directly)
        trigger_python_login().await?;
        
        info!("Login successful, credentials saved to MongoDB");
        
        Ok(())
    }
    
    /// Mark current credentials as invalid (e.g., after 406 error)
    pub async fn invalidate_credentials(&self) -> Result<()> {
        warn!("Invalidating current credentials");
        
        self.storage.invalidate_all().await?;
        
        let mut cache = self.cached_credentials.write().await;
        *cache = None;
        
        Ok(())
    }
    
    /// Get captured signature for a specific endpoint
    pub async fn get_endpoint_signature(&self, endpoint: &str) -> Result<Option<super::credentials::ApiSignature>> {
        self.storage.get_api_signature(endpoint).await
    }
    
    /// Generate a dummy signature - in new architecture, we use x-s-common from stored credentials
    /// The actual signing happens in the browser during login
    pub async fn sign_request(&self, _url: &str, _method: &str, _body: Option<&str>) -> Result<(String, i64, String)> {
        // Get credentials which contain pre-captured x-s-common
        let creds = self.get_credentials().await?;
        
        // Return placeholder x-s and x-t - these should ideally come from browser
        // For now, we return empty strings as we need a different approach for signing
        let x_t = chrono::Utc::now().timestamp_millis();
        
        Ok(("".to_string(), x_t, creds.x_s_common.clone()))
    }
}
