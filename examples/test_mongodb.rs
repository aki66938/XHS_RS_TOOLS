use xhs_rs::auth::{CredentialStorage, UserCredentials};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("Testing MongoDB connection...");
    
    // Connect to MongoDB
    let storage = CredentialStorage::new("mongodb://localhost:27017").await?;
    
    println!("✅ MongoDB connected successfully!");
    
    // Create test credentials
    let mut cookies = HashMap::new();
    cookies.insert("a1".to_string(), "test_a1_value".to_string());
    cookies.insert("web_session".to_string(), "test_session".to_string());
    
    let creds = UserCredentials::new(
        "test_user_123".to_string(),
        cookies,
        "test_x_s_common".to_string(),
    );
    
    // Save credentials
    storage.save_credentials(&creds).await?;
    println!("✅ Credentials saved!");
    
    // Retrieve credentials
    let retrieved = storage.get_active_credentials().await?;
    
    match retrieved {
        Some(c) => {
            println!("✅ Retrieved credentials for user: {}", c.user_id);
            println!("   Cookie string: {}", c.cookie_string());
        }
        None => println!("❌ No credentials found"),
    }
    
    // Invalidate
    storage.invalidate_all().await?;
    println!("✅ Credentials invalidated!");
    
    // Verify invalidation
    let after_invalidate = storage.get_active_credentials().await?;
    if after_invalidate.is_none() {
        println!("✅ Verified: No active credentials after invalidation");
    }
    
    println!("\n🎉 All MongoDB tests passed!");
    
    Ok(())
}
