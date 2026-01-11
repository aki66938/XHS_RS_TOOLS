use mongodb::bson;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User credentials captured from browser login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    /// XHS user ID
    pub user_id: String,
    
    /// All captured cookies as key-value pairs
    /// Key cookies: a1, web_session, webId, gid, xsecappid
    pub cookies: HashMap<String, String>,
    
    /// The x-s-common header value captured from network requests
    pub x_s_common: String,
    
    /// When these credentials were first created
    pub created_at: bson::DateTime,
    
    /// When these credentials were last updated
    pub updated_at: bson::DateTime,
    
    /// Whether these credentials are currently valid
    pub is_valid: bool,
}

impl UserCredentials {
    /// Create new credentials
    pub fn new(user_id: String, cookies: HashMap<String, String>, x_s_common: String) -> Self {
        let now = bson::DateTime::now();
        Self {
            user_id,
            cookies,
            x_s_common,
            created_at: now,
            updated_at: now,
            is_valid: true,
        }
    }
    
    /// Get cookies as a single string for HTTP headers
    pub fn cookie_string(&self) -> String {
        self.cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ")
    }
    
    /// Check if credentials might be expired (older than 7 days)
    pub fn is_potentially_expired(&self) -> bool {
        let now = bson::DateTime::now();
        let age_ms = now.timestamp_millis() - self.updated_at.timestamp_millis();
        let days = age_ms / (1000 * 60 * 60 * 24);
        days > 7
    }
    
    /// Mark credentials as invalid
    pub fn invalidate(&mut self) {
        self.is_valid = false;
        self.updated_at = bson::DateTime::now();
    }
    
    /// Refresh the updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = bson::DateTime::now();
    }
}

/// API endpoint signature captured from browser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSignature {
    /// Endpoint name (e.g., "user_me", "search_trending")
    pub endpoint: String,
    
    /// x-s header value
    pub x_s: String,
    
    /// x-t header value (timestamp)
    pub x_t: String,
    
    /// x-s-common header value
    pub x_s_common: String,
    
    /// x-b3-traceid header value
    pub x_b3_traceid: String,
    
    /// x-xray-traceid header value
    pub x_xray_traceid: String,
    
    /// HTTP method (GET, POST)
    #[serde(default)]
    pub method: Option<String>,
    
    /// POST request body (for signature binding)
    #[serde(default)]
    pub post_body: Option<String>,
    
    /// When this signature was captured
    pub captured_at: bson::DateTime,
    
    /// Whether this signature is currently valid
    pub is_valid: bool,
}
