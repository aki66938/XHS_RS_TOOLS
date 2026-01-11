use xhs_rs::server;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting XHS Rust Tools Server...");
    
    server::start_server().await?;

    Ok(())
}
