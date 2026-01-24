use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::feed::HomefeedItem;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QueryTrendingResponse {
    pub code: i32,
    pub success: bool,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub data: Option<QueryTrendingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QueryTrendingData {
    pub word_request_id: String,
    pub title: String,
    pub queries: Vec<TrendingQuery>,
    pub hint_word: Option<TrendingHintWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TrendingQuery {
    pub title: String,
    pub desc: Option<String>,
    pub search_word: String,
    #[serde(rename = "type")]
    pub query_type: String,
    pub hint_word_request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TrendingHintWord {
    #[serde(rename = "type")]
    pub hint_type: String,
    pub search_word: String,
    pub hint_word_request_id: String,
    pub title: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "code": 0,
    "success": true,
    "msg": "success",
    "data": {
        "sug_items": [
            { "type": "history", "text": "杭州" },
            { "type": "sug", "text": "杭州旅游攻略" }
        ]
    }
}))]
pub struct SearchRecommendResponse {
    pub code: i32,
    pub success: bool,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub data: Option<SearchRecommendData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchRecommendData {
    pub search_cpl_id: Option<String>,
    pub word_request_id: Option<String>,
    #[serde(default)]
    pub sug_items: Vec<SugItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SugItem {
    #[serde(rename = "type")]
    pub item_type: String,
    pub text: String,
    pub search_type: Option<String>,
}

// =================== Search Notes ===================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "keyword": "搜索关键词",
    "page": 1,
    "page_size": 20,
    "sort": "general",
    "note_type": 0,
    "search_id": "search_id_example",
    "ext_flags": [],
    "filters": [
        {"tags": ["general"], "type": "sort_type"},
        {"tags": ["不限"], "type": "filter_note_type"},
        {"tags": ["不限"], "type": "filter_note_time"},
        {"tags": ["不限"], "type": "filter_note_range"},
        {"tags": ["不限"], "type": "filter_pos_distance"}
    ],
    "geo": "",
    "image_formats": ["jpg", "webp", "avif"]
}))]
pub struct SearchNotesRequest {
    pub keyword: String,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
    #[serde(default)]
    pub search_id: Option<String>,
    #[serde(default = "default_sort")]
    pub sort: String,
    /// 笔记类型: 0=综合(默认), 1=图文, 2=视频
    #[serde(default)]
    pub note_type: i32,
    /// 扩展筛选标志 (通常为空数组)
    #[serde(default)]
    pub ext_flags: Vec<serde_json::Value>,
    /// 筛选条件 (必需字段)
    /// 
    /// 包含排序和筛选类型，使用默认值即可
    #[serde(default = "default_filters")]
    pub filters: Vec<SearchFilterOption>,
    #[serde(default)]
    pub geo: String,
    #[serde(default = "default_image_formats")]
    pub image_formats: Vec<String>,
}

fn default_page() -> i32 { 1 }
fn default_page_size() -> i32 { 20 }
fn default_sort() -> String { "general".to_string() }
fn default_image_formats() -> Vec<String> { vec!["jpg".to_string(), "webp".to_string(), "avif".to_string()] }
fn default_filters() -> Vec<SearchFilterOption> {
    vec![
        SearchFilterOption { tags: vec!["general".to_string()], filter_type: "sort_type".to_string() },
        SearchFilterOption { tags: vec!["不限".to_string()], filter_type: "filter_note_type".to_string() },
        SearchFilterOption { tags: vec!["不限".to_string()], filter_type: "filter_note_time".to_string() },
        SearchFilterOption { tags: vec!["不限".to_string()], filter_type: "filter_note_range".to_string() },
        SearchFilterOption { tags: vec!["不限".to_string()], filter_type: "filter_pos_distance".to_string() },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchFilterOption {
    pub tags: Vec<String>,
    #[serde(rename = "type")]
    pub filter_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchNotesResponse {
    pub code: i32,
    pub success: bool,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub data: Option<SearchNotesData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchNotesData {
    /// 搜索会话ID (用于关联 onebox 等后续请求)
    #[serde(default)]
    pub search_id: Option<String>,
    pub has_more: bool,
    #[serde(default)]
    pub items: Vec<HomefeedItem>,
}

// =================== Search OneBox ===================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "keyword": "牢A斩杀线",
    "search_id": "demo_sid_1234567890",
    "biz_type": "web_search_user",
    "request_id": "1234567890-1234567890123"
}))]
pub struct SearchOneboxRequest {
    pub keyword: String,
    pub search_id: String,
    pub biz_type: String,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchOneboxResponse {
    pub code: i32,
    pub success: bool,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub data: Option<serde_json::Value>, 
}

// =================== Search Filter ===================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchFilterResponse {
    pub code: i32,
    pub success: bool,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub data: Option<SearchFilterData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchFilterData {
    #[serde(default)]
    pub filters: Vec<FilterItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilterItem {
    #[serde(rename = "type")]
    pub filter_type: String,
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub filter_tags: Vec<FilterTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilterTag {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub show_type: Option<i32>,
}

// =================== Search User ===================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchUserRequestBody {
    pub search_user_request: SearchUserRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "keyword": "搜索关键词",
    "search_id": "search_id_example",
    "page": 1,
    "page_size": 15,
    "biz_type": "web_search_user",
    "request_id": "request_id_example"
}))]
pub struct SearchUserRequest {
    pub keyword: String,
    pub search_id: Option<String>,
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_page_size_15")]
    pub page_size: i32,
    #[serde(default = "default_biz_type_user")]
    pub biz_type: String,
    pub request_id: Option<String>,
}

fn default_page_size_15() -> i32 { 15 }
fn default_biz_type_user() -> String { "web_search_user".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchUserResponse {
    pub code: i32,
    pub success: bool,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub data: Option<SearchUserData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchUserData {
    pub has_more: bool,
    #[serde(default)]
    pub users: Vec<SearchUserItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchUserItem {
    pub id: String,
    pub name: String,
    pub image: Option<String>,
    #[serde(rename = "fans")]
    pub fan_count: Option<String>, 
    pub note_count: Option<i32>,
    pub desc: Option<String>,
    pub red_id: Option<String>,
    pub link: Option<String>,
}
