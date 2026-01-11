//! Test browser-based login flow
//! 
//! This example demonstrates:
//! 1. Launching a browser window
//! 2. User scanning QR code to login
//! 3. Capturing credentials
//! 4. Storing to MongoDB

use xhs_rs::auth::AuthService;
use xhs_rs::utils::print_qr_to_terminal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║       XHS 浏览器登录测试                                    ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    // Initialize auth service with MongoDB
    println!("正在连接 MongoDB...");
    let auth_service = AuthService::new("mongodb://localhost:27017").await?;
    
    // Get or trigger login
    println!("正在检查凭据...\n");
    let credentials = auth_service.get_credentials().await?;
    
    println!("\n✅ 登录成功!");
    println!("   用户 ID: {}", credentials.user_id);
    println!("   Cookies: {} 个", credentials.cookies.len());
    println!("   x-s-common: {:.50}...", credentials.x_s_common);
    
    // Test QR code display
    println!("\n测试终端二维码显示:");
    print_qr_to_terminal("https://www.xiaohongshu.com/login", "示例二维码")?;
    
    Ok(())
}
