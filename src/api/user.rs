use crate::auth::AuthService;
use crate::client::XhsClient;
use crate::models::user::UserMeResponse;
use anyhow::Result;
use std::sync::Arc;

const ORIGIN: &str = "https://www.xiaohongshu.com";
const REFERER: &str = "https://www.xiaohongshu.com/";

pub async fn get_current_user(client: &XhsClient, auth: &Arc<AuthService>) -> Result<UserMeResponse> {
    let url = "https://edith.xiaohongshu.com/api/sns/web/v2/user/me";
    
    // Get credentials from auth service (passively)
    let credentials_opt = auth.try_get_credentials().await?;
    
    // If no credentials, return a dummy "not logged in" response immediately
    let credentials = match credentials_opt {
        Some(creds) => creds,
        None => {
            return Ok(UserMeResponse {
                code: -1,
                success: false,
                msg: "Not logged in".to_string(),
                data: crate::models::user::UserInfo {
                     nickname: Some("".to_string()),
                     desc: Some("".to_string()),
                     gender: Some(0),
                     images: None,
                     imageb: None,
                     guest: true,
                     red_id: None,
                     user_id: "".to_string(),
                }
            });
        }
    };
    let (xs, xt, xs_common) = auth.sign_request(url, "GET", None).await?;

    let response = client.get_client()
        .get(url)
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
        .header("x-b3-traceid", "c229fcfefe758690")
        .header("x-s", xs)
        .header("x-s-common", xs_common)
        .header("cookie", credentials.cookie_string())
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;
    tracing::info!("Get User Me Response [{}]: {}", status, text);

    // Check for 406 error and invalidate credentials if needed
    if status.as_u16() == 406 {
        tracing::warn!("Received 406 error, invalidating credentials");
        auth.invalidate_credentials().await?;
    }

    let result = serde_json::from_str::<UserMeResponse>(&text)?;
    
    // Auto-invalidate credentials when XHS returns "登录已过期" (code: -100)
    if result.code == -100 {
        tracing::warn!("XHS returned code -100 (login expired), invalidating credentials");
        auth.invalidate_credentials().await?;
    }
    
    Ok(result)
}
