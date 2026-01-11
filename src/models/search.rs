use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
