//! Requires the nodriver agent on localhost:8765. Opens a browser for human login.
use std::path::PathBuf;
use xhs_rs::auth::AuthService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let service = AuthService::new(PathBuf::from("cookie.json")).await?;
    let credentials = service.get_credentials().await?;
    println!("Login verified; {} cookies stored locally", credentials.cookies.len());
    Ok(())
}
