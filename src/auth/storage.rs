//! JSON file-based credential storage
//!
//! Stores credentials in `cookie.json` in the project root directory.

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn};

use super::credentials::UserCredentials;

const COOKIE_FILE: &str = "cookie.json";

/// JSON file-based credential storage
pub struct CredentialStorage {
    file_path: PathBuf,
}

impl CredentialStorage {
    /// Create a new storage instance
    pub async fn new() -> Result<Self> {
        let file_path = PathBuf::from(COOKIE_FILE);
        info!("Using JSON credential storage: {}", file_path.display());
        Ok(Self { file_path })
    }
    
    /// Get the currently active (valid) credentials
    pub async fn get_active_credentials(&self) -> Result<Option<UserCredentials>> {
        if !self.file_path.exists() {
            info!("No cookie.json found");
            return Ok(None);
        }
        
        let content = tokio::fs::read_to_string(&self.file_path).await?;
        let creds: UserCredentials = serde_json::from_str(&content)?;
        
        if creds.is_valid {
            info!("Found active credentials for user: {}", creds.user_id);
            Ok(Some(creds))
        } else {
            info!("Found credentials but marked as invalid");
            Ok(None)
        }
    }
    
    /// Save or update credentials
    pub async fn save_credentials(&self, creds: &UserCredentials) -> Result<()> {
        let content = serde_json::to_string_pretty(creds)?;
        tokio::fs::write(&self.file_path, content).await?;
        info!("Saved credentials for user: {} to {}", creds.user_id, self.file_path.display());
        Ok(())
    }
    
    /// Mark all credentials as invalid
    pub async fn invalidate_all(&self) -> Result<()> {
        if !self.file_path.exists() {
            return Ok(());
        }
        
        let content = tokio::fs::read_to_string(&self.file_path).await?;
        let mut creds: UserCredentials = serde_json::from_str(&content)?;
        
        if creds.is_valid {
            creds.invalidate();
            let content = serde_json::to_string_pretty(&creds)?;
            tokio::fs::write(&self.file_path, content).await?;
            warn!("Invalidated credentials for user: {}", creds.user_id);
        }
        
        Ok(())
    }
    
    /// Invalidate credentials for a specific user (same as invalidate_all for single-user storage)
    pub async fn invalidate_user(&self, user_id: &str) -> Result<()> {
        if !self.file_path.exists() {
            return Ok(());
        }
        
        let content = tokio::fs::read_to_string(&self.file_path).await?;
        let mut creds: UserCredentials = serde_json::from_str(&content)?;
        
        if creds.user_id == user_id && creds.is_valid {
            creds.invalidate();
            let content = serde_json::to_string_pretty(&creds)?;
            tokio::fs::write(&self.file_path, content).await?;
            warn!("Invalidated credentials for user: {}", user_id);
        }
        
        Ok(())
    }
    
    /// Get API signature for a specific endpoint (legacy, returns None for JSON storage)
    pub async fn get_api_signature(&self, _endpoint: &str) -> Result<Option<super::credentials::ApiSignature>> {
        // API signatures are not stored in JSON storage (they are generated on-demand via Agent)
        Ok(None)
    }
}
