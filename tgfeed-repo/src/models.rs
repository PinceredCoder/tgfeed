use mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Subscription {
    pub user_id: i64,
    pub channel_id: i64,
    pub channel_handle: String,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub subscribed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StoredMessage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub channel_id: i64,
    pub message_id: i32,
    pub text: String,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub date: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SummarizeState {
    /// User ID who requested summarization
    pub user_id: i64,
    /// Last time /summarize was called
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub last_summarized_at: chrono::DateTime<chrono::Utc>,
}
