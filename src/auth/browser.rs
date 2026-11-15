//! Interactive login delegated to the nodriver HTTP agent.
use anyhow::{anyhow, Result};
use crate::config::get_agent_url;

pub async fn trigger_browser_login() -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(65))
        .no_proxy()
        .build()?;
    let response: serde_json::Value = client
        .post(format!("{}/browser/login", get_agent_url()))
        .json(&serde_json::json!({"target": "explore"}))
        .send().await?.error_for_status()?.json().await?;
    let sid = response["session_id"].as_str()
        .ok_or_else(|| anyhow!("Browser agent did not create a login session"))?;
    let url = format!("{}/browser/login/{}", get_agent_url(), sid);
    tracing::info!("Complete login in the project's Chrome window");
    let result = async {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(180);
        while tokio::time::Instant::now() < deadline {
            let response: serde_json::Value = client.get(&url).send().await?
                .error_for_status()?.json().await?;
            if response["state"] == "authenticated" {
                return Ok(());
            }
            if response["success"] == false {
                return Err(anyhow!("Browser login failed: {}", response["state"]));
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        Err(anyhow!("Browser login timed out"))
    }.await;
    let _ = client.delete(&url).send().await;
    result
}
