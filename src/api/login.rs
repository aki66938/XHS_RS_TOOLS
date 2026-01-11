use crate::auth::AuthService;
use crate::client::XhsClient;
use crate::models::login::{QrCodeCreateResponse, QrCodeStatusResponse};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

const ORIGIN: &str = "https://www.xiaohongshu.com";
const REFERER: &str = "https://www.xiaohongshu.com/";

pub async fn create_qrcode(client: &XhsClient, auth: &Arc<AuthService>) -> Result<QrCodeCreateResponse> {
    let url = "https://edith.xiaohongshu.com/api/sns/web/v1/login/qrcode/create";
    let body_json = json!({"qr_type": 1});
    let body_str = body_json.to_string();

    // Get credentials and signature from auth service
    let credentials = auth.get_credentials().await?;
    let (xs, xt, xs_common) = auth.sign_request(url, "POST", Some(&body_str)).await?;
    
    tracing::info!("Using Credentials - xs_common: {:.50}...", xs_common);

    let response = client.get_client()
        .post(url)
        .header("content-type", "application/json;charset=UTF-8")
        .header("origin", ORIGIN)
        .header("referer", REFERER)
        .header("accept", "application/json, text/plain, */*")
        .header("accept-language", "zh-CN,zh;q=0.9")
        .header("priority", "u=1, i")
        .header("sec-ch-ua", r#""Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24""#)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"Windows\"")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-site")
        .header("x-t", xt.to_string())
        .header("x-b3-traceid", "523a6c2267e872aa")
        .header("x-s", xs)
        .header("x-s-common", xs_common)
        .header("cookie", credentials.cookie_string())
        .json(&body_json)
        .send()
        .await?;
        
    let status = response.status();
    let text = response.text().await?;
    tracing::info!("Create QR Code Response [{}]: {}", status, text);

    // Check for 406 error and invalidate credentials if needed
    if status.as_u16() == 406 {
        tracing::warn!("Received 406 error, invalidating credentials");
        auth.invalidate_credentials().await?;
    }

    let result = serde_json::from_str::<QrCodeCreateResponse>(&text)?;
    Ok(result)
}

pub async fn check_qrcode_status(client: &XhsClient, auth: &Arc<AuthService>, qr_id: &str, code: &str) -> Result<QrCodeStatusResponse> {
    let base_url = "https://edith.xiaohongshu.com/api/sns/web/v1/login/qrcode/status";
    let url = format!("{}?qr_id={}&code={}", base_url, qr_id, code);
    
    // Get credentials and signature from auth service
    let credentials = auth.get_credentials().await?;
    let (xs, xt, xs_common) = auth.sign_request(&url, "GET", None).await?;

    let response = client.get_client()
        .get(&url)
        .header("x-login-mode;", "")
        .header("origin", ORIGIN)
        .header("referer", REFERER)
        .header("accept", "application/json, text/plain, */*")
        .header("accept-language", "zh-CN,zh;q=0.9")
        .header("sec-ch-ua", r#""Google Chrome";v="143", "Chromium";v="143", "Not A(Brand";v="24""#)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"Windows\"")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-site")
        .header("x-t", xt.to_string())
        .header("x-s", xs)
        .header("x-s-common", xs_common)
        .header("cookie", credentials.cookie_string())
        .send()
        .await?;

    let status = response.status();
    
    // Check for 406 error
    if status.as_u16() == 406 {
        tracing::warn!("Received 406 error, invalidating credentials");
        auth.invalidate_credentials().await?;
    }

    let result = response.json::<QrCodeStatusResponse>().await?;
    Ok(result)
}


pub async fn start_login_session() -> Result<crate::models::login::QrCodeSessionResponse> {
    use tokio::process::Command;
    use std::process::Stdio;
    use tokio::io::{BufReader, AsyncBufReadExt};
    use crate::models::login::QrCodeSessionResponse;

    tracing::info!("Starting Python login session...");

    let mut command = Command::new("python");
    // Ensure we flush output (Python by default buffers stdout if not TTY)
    // -u force stdout to be unbuffered (or line buffered)
    command.arg("-u") 
        .arg("scripts/login.py")
        .arg("--headless")
        .arg("--json");
        
    command.stdout(Stdio::piped());
    // Also capture stderr to log errors
    command.stderr(Stdio::piped());
    
    let mut child = command.spawn().map_err(|e| anyhow::anyhow!("Failed to spawn python script: {}", e))?;
    
    let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
    let mut reader = BufReader::new(stdout);
    
    let mut line = String::new();
    let response = loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
             // Check stderr if stdout is empty
             return Err(anyhow::anyhow!("Python script exited without output"));
        }
        
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        
        // Try to parse JSON
        if let Ok(resp) = serde_json::from_str::<QrCodeSessionResponse>(trimmed) {
             break resp;
        } else {
             tracing::debug!("Python Output (Ignored): {}", trimmed);
        }
    };
    
    // Spawn background task to monitor the rest of the session
    tokio::spawn(async move {
        tracing::info!("Login session background task started (PID: {:?})", child.id());
        
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        tracing::info!("Session Update: {}", trimmed);
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading script output: {}", e);
                    break;
                }
            }
        }
        
        match child.wait().await {
            Ok(status) => tracing::info!("Login session process exited with: {}", status),
            Err(e) => tracing::error!("Login session process error: {}", e),
        }
    });
    
    Ok(response)
}

/// Get current session/credential information
pub async fn get_session_info(auth: &Arc<AuthService>) -> Result<crate::models::login::SessionInfoResponse> {
    use crate::models::login::{SessionInfoResponse, SessionInfoData, CookieInfo};
    
    match auth.try_get_credentials().await? {
        Some(creds) => {
            let cookies: Vec<CookieInfo> = creds.cookies.iter().map(|(name, value)| {
                // Mask cookie values for security (show first 3 and last 3 chars)
                let masked_value = if value.len() > 10 {
                    format!("{}...{}", &value[..3], &value[value.len()-3..])
                } else {
                    value.clone()
                };
                
                CookieInfo {
                    name: name.clone(),
                    value: masked_value,
                    domain: ".xiaohongshu.com".to_string(),
                }
            }).collect();
            
            // Mask x_s_common (show first 30 chars)
            let masked_xs_common = if creds.x_s_common.len() > 30 {
                format!("{}...", &creds.x_s_common[..30])
            } else {
                creds.x_s_common.clone()
            };
            
            Ok(SessionInfoResponse {
                code: 0,
                success: true,
                msg: "Session found".to_string(),
                data: Some(SessionInfoData {
                    user_id: creds.user_id,
                    cookie_count: creds.cookies.len(),
                    cookies,
                    x_s_common: masked_xs_common,
                    created_at: creds.created_at.to_string(),
                    is_valid: creds.is_valid,
                }),
            })
        },
        None => {
            Ok(SessionInfoResponse {
                code: -1,
                success: false,
                msg: "No active session found. Please login first.".to_string(),
                data: None,
            })
        }
    }
}
