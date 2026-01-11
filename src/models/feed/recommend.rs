use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Homefeed request body - 主页发现请求参数
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "cursor_score": "",
    "num": 47,
    "refresh_type": 1,
    "note_index": 34,
    "unread_begin_note_id": "",
    "unread_end_note_id": "",
    "unread_note_count": 0,
    "category": "homefeed_recommend",
    "search_key": "",
    "need_num": 22,
    "image_formats": ["jpg", "webp", "avif"],
    "need_filter_image": false
}))]
pub struct HomefeedRequest {
    #[serde(default)]
    pub cursor_score: String,
    #[serde(default = "default_num")]
    pub num: i32,
    #[serde(default = "default_refresh_type")]
    pub refresh_type: i32,
    #[serde(default)]
    pub note_index: i32,
    #[serde(default)]
    pub unread_begin_note_id: String,
    #[serde(default)]
    pub unread_end_note_id: String,
    #[serde(default)]
    pub unread_note_count: i32,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub search_key: String,
    #[serde(default = "default_need_num")]
    pub need_num: i32,
    #[serde(default = "default_image_formats")]
    pub image_formats: Vec<String>,
    #[serde(default)]
    pub need_filter_image: bool,
}

fn default_num() -> i32 { 47 }
fn default_refresh_type() -> i32 { 1 }
fn default_category() -> String { "homefeed_recommend".to_string() }
fn default_need_num() -> i32 { 22 }
fn default_image_formats() -> Vec<String> { vec!["jpg".to_string(), "webp".to_string(), "avif".to_string()] }

impl Default for HomefeedRequest {
    fn default() -> Self {
        Self {
            cursor_score: String::new(),
            num: 47,
            refresh_type: 1,
            note_index: 34,
            unread_begin_note_id: String::new(),
            unread_end_note_id: String::new(),
            unread_note_count: 0,
            category: "homefeed_recommend".to_string(),
            search_key: String::new(),
            need_num: 22,
            image_formats: vec!["jpg".to_string(), "webp".to_string(), "avif".to_string()],
            need_filter_image: false,
        }
    }
}

/// Homefeed response - 主页发现响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code": 0,
    "success": true,
    "msg": "成功",
    "data": {
        "cursor_score": "1.7681358649530034E9",
        "items": []
    }
}))]
pub struct HomefeedResponse {
    pub code: i32,
    pub success: bool,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub data: Option<HomefeedData>,
}

/// 主页内容数据
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HomefeedData {
    /// 分页游标
    #[serde(default)]
    pub cursor_score: Option<String>,
    /// 笔记列表
    #[serde(default)]
    pub items: Vec<HomefeedItem>,
}

/// 单条笔记项
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "69539b19000000002202c106",
    "model_type": "note",
    "track_id": "2fu7gpj53ojaaenshxkib",
    "xsec_token": "ABgmZhb7UheMUTk-zbKLSjLizyXRfHgBLRwTg3lxgVx_s=",
    "note_card": {}
}))]
pub struct HomefeedItem {
    /// 笔记ID
    pub id: String,
    /// 模型类型 (note)
    #[serde(default)]
    pub model_type: Option<String>,
    /// 追踪ID
    #[serde(default)]
    pub track_id: Option<String>,
    /// 安全Token
    #[serde(default)]
    pub xsec_token: Option<String>,
    /// 是否忽略
    #[serde(default)]
    pub ignore: Option<bool>,
    /// 笔记卡片详情
    #[serde(default)]
    pub note_card: Option<NoteCard>,
}

/// 笔记卡片信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "type": "normal",
    "display_title": "男生没方向，一定要去闯闯的6个职业！",
    "user": {},
    "cover": {},
    "interact_info": {}
}))]
pub struct NoteCard {
    /// 笔记类型 (normal, video)
    #[serde(rename = "type")]
    #[serde(default)]
    pub note_type: Option<String>,
    /// 展示标题
    #[serde(default)]
    pub display_title: Option<String>,
    /// 作者信息
    #[serde(default)]
    pub user: Option<NoteUser>,
    /// 封面信息
    #[serde(default)]
    pub cover: Option<NoteCover>,
    /// 互动信息
    #[serde(default)]
    pub interact_info: Option<InteractInfo>,
    /// 视频信息 (视频笔记才有)
    #[serde(default)]
    pub video: Option<NoteVideo>,
}

/// 笔记作者信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "user_id": "664ec6ef0000000007004173",
    "nickname": "小李学姐爱学习",
    "avatar": "https://sns-avatar-qc.xhscdn.com/avatar/..."
}))]
pub struct NoteUser {
    /// 用户ID
    #[serde(default)]
    pub user_id: Option<String>,
    /// 昵称
    #[serde(default)]
    pub nickname: Option<String>,
    /// 昵称(冗余字段)
    #[serde(default)]
    pub nick_name: Option<String>,
    /// 头像URL
    #[serde(default)]
    pub avatar: Option<String>,
    /// 安全Token
    #[serde(default)]
    pub xsec_token: Option<String>,
}

/// 笔记封面信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NoteCover {
    /// 宽度
    #[serde(default)]
    pub width: Option<i32>,
    /// 高度
    #[serde(default)]
    pub height: Option<i32>,
    /// 预览图URL
    #[serde(default)]
    pub url_pre: Option<String>,
    /// 默认图URL
    #[serde(default)]
    pub url_default: Option<String>,
    /// 文件ID
    #[serde(default)]
    pub file_id: Option<String>,
    /// 图片列表
    #[serde(default)]
    pub info_list: Vec<CoverImageInfo>,
}

/// 封面图片信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CoverImageInfo {
    /// 场景类型 (WB_PRV, WB_DFT)
    #[serde(default)]
    pub image_scene: Option<String>,
    /// 图片URL
    #[serde(default)]
    pub url: Option<String>,
}

/// 笔记互动信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "liked": false,
    "liked_count": "1008"
}))]
pub struct InteractInfo {
    /// 是否已点赞
    #[serde(default)]
    pub liked: Option<bool>,
    /// 点赞数
    #[serde(default)]
    pub liked_count: Option<String>,
}

/// 视频信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NoteVideo {
    /// 视频能力信息
    #[serde(default)]
    pub capa: Option<VideoCapa>,
}

/// 视频能力信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VideoCapa {
    /// 视频时长(秒)
    #[serde(default)]
    pub duration: Option<i32>,
}
