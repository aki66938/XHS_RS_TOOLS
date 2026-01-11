use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code": 0,
    "success": true,
    "msg": "成功",
    "data": {
        "user_id": "5ceac80d00000000xxxxxxxx",
        "red_id": "123456789",
        "nickname": "用户名称",
        "desc": "用户简介信息",
        "gender": 0,
        "guest": false,
        "images": "https://sns-avatar-qc.xhscdn.com/avatar/xxxxxxxx",
        "imageb": "https://sns-avatar-qc.xhscdn.com/avatar/xxxxxxxx"
    }
}))]
pub struct UserMeResponse {
    pub code: i32,
    pub success: bool,
    pub msg: String,
    pub data: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "user_id": "5ceac80d00000000xxxxxxxx",
    "red_id": "123456789",
    "nickname": "用户名称",
    "desc": "用户简介信息",
    "gender": 0,
    "guest": false,
    "images": "https://sns-avatar-qc.xhscdn.com/avatar/xxxxxxxx",
    "imageb": "https://sns-avatar-qc.xhscdn.com/avatar/xxxxxxxx"
}))]
pub struct UserInfo {
    pub user_id: String,
    pub red_id: Option<String>,
    pub nickname: Option<String>,
    pub desc: Option<String>,
    pub gender: Option<i32>,
    pub guest: bool,
    pub images: Option<String>,
    pub imageb: Option<String>,
}
