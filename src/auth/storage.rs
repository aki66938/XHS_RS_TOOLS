use anyhow::Result;
use mongodb::{bson::doc, options::ClientOptions, Client, Collection};
use tracing::{info, warn};

use super::credentials::UserCredentials;

const DATABASE_NAME: &str = "xhs_tools";
const COLLECTION_NAME: &str = "credentials";

/// MongoDB-based credential storage
pub struct CredentialStorage {
    database: mongodb::Database,
    collection: Collection<UserCredentials>,
}

impl CredentialStorage {
    /// Create a new storage instance connected to MongoDB
    pub async fn new(mongodb_uri: &str) -> Result<Self> {
        info!("Connecting to MongoDB at {}", mongodb_uri);
        
        let client_options = ClientOptions::parse(mongodb_uri).await?;
        let client = Client::with_options(client_options)?;
        
        // Ping to verify connection
        client
            .database("admin")
            .run_command(doc! { "ping": 1 }, None)
            .await?;
        
        info!("Successfully connected to MongoDB");
        
        let database = client.database(DATABASE_NAME);
        let collection = database.collection::<UserCredentials>(COLLECTION_NAME);
        
        Ok(Self { database, collection })
    }
    
    /// Get the currently active (valid) credentials
    pub async fn get_active_credentials(&self) -> Result<Option<UserCredentials>> {
        let filter = doc! { "is_valid": true };
        let result = self.collection.find_one(filter, None).await?;
        
        if let Some(ref creds) = result {
            info!("Found active credentials for user: {}", creds.user_id);
        } else {
            info!("No active credentials found");
        }
        
        Ok(result)
    }
    
    /// Save or update credentials
    pub async fn save_credentials(&self, creds: &UserCredentials) -> Result<()> {
        // First, invalidate any existing credentials
        self.invalidate_all().await?;
        
        // Insert new credentials
        self.collection.insert_one(creds, None).await?;
        info!("Saved new credentials for user: {}", creds.user_id);
        
        Ok(())
    }
    
    /// Mark all credentials as invalid
    pub async fn invalidate_all(&self) -> Result<()> {
        let filter = doc! { "is_valid": true };
        let update = doc! { "$set": { "is_valid": false } };
        
        let result = self.collection.update_many(filter, update, None).await?;
        
        if result.modified_count > 0 {
            warn!("Invalidated {} existing credentials", result.modified_count);
        }
        
        Ok(())
    }
    
    /// Invalidate credentials for a specific user
    pub async fn invalidate_user(&self, user_id: &str) -> Result<()> {
        let filter = doc! { "user_id": user_id };
        let update = doc! { "$set": { "is_valid": false } };
        
        self.collection.update_many(filter, update, None).await?;
        warn!("Invalidated credentials for user: {}", user_id);
        
        Ok(())
    }
    
    /// Get API signature for a specific endpoint
    pub async fn get_api_signature(&self, endpoint: &str) -> Result<Option<super::credentials::ApiSignature>> {
        let sig_collection = self.database.collection::<super::credentials::ApiSignature>("api_signatures");
        
        let filter = doc! { "endpoint": endpoint, "is_valid": true };
        let result = sig_collection.find_one(filter, None).await?;
        
        if let Some(ref sig) = result {
            info!("Found signature for endpoint: {}", sig.endpoint);
        } else {
            info!("No signature found for endpoint: {}", endpoint);
        }
        
        Ok(result)
    }
}
