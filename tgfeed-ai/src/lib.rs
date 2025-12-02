pub mod claude;
mod config;
mod error;

pub use config::Config;
pub use error::*;

pub struct MessageData {
    pub channel_handle: String,
    pub text: String,
    pub date: chrono::DateTime<chrono::Utc>,
}

pub trait Summarizer {
    fn summarize(&self, messages: Vec<MessageData>)
    -> impl Future<Output = TgfeedAiResult<String>>;
}
