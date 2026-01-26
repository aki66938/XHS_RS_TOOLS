use anyhow::{anyhow, Result};
use std::fs;
use std::collections::HashMap;
use xhs_rs::api::creator::info::{get_creator_user_info, get_creator_home_info};

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();
    
    // 1. Load cookie-creator.json
    let cookie_path = "cookie-creator.json";
    if !std::path::Path::new(cookie_path).exists() {
        return Err(anyhow!("cookie-creator.json not found! Please run 'cargo run' and scan QR code first."));
    }
    
    #[derive(serde::Deserialize)]
    struct CookieFile {
        cookies: HashMap<String, String>,
    }

    let cookie_content = fs::read_to_string(cookie_path)?;
    let cookie_file: CookieFile = serde_json::from_str(&cookie_content)?;
    let cookies = cookie_file.cookies;
    
    println!("Loaded {} cookies from {}", cookies.len(), cookie_path);
    
    // 2. Call User Info
    println!("\n=== Testing get_creator_user_info ===");
    match get_creator_user_info(&cookies).await {
        Ok(info) => {
            println!("Success!");
            println!("User ID: {:?}", info.user_id);
            println!("Name: {:?}", info.user_name);
            println!("Role: {:?}", info.role);
            if let Some(perms) = info.permissions {
                println!("Permissions count: {}", perms.len());
            } else {
                println!("Permissions: None");
            }
        },
        Err(e) => {
            println!("Failed: {}", e);
        }
    }
    
    // 3. Call Home Info
    println!("\n=== Testing get_creator_home_info ===");
    match get_creator_home_info(&cookies).await {
        Ok(info) => {
            println!("Success!");
            println!("Name: {:?}", info.name);
            println!("Fans: {:?}", info.fans_count);
            println!("Likes: {:?}", info.faved_count);
            println!("Desc: {:?}", info.personal_desc);
        },
        Err(e) => {
            println!("Failed: {}", e);
        }
    }
    
    Ok(())
}
