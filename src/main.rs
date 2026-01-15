use xhs_rs::server;
use tracing::info;
use tracing_subscriber::fmt::time::OffsetTime;
use time::UtcOffset;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging with local timezone
    // 尝试获取本地时区偏移，失败则使用 UTC+8
    let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::from_hms(8, 0, 0).unwrap());
    let timer = OffsetTime::new(offset, time::macros::format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]"
    ));
    
    tracing_subscriber::fmt()
        .with_timer(timer)
        .init();
    
    info!("Starting XHS Rust Tools Server...");
    
    server::start_server().await?;

    Ok(())
}
