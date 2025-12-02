mod config;
mod models;

pub use config::Config;

use crate::claude::models::{ClaudeMessage, ClaudeRequest, ClaudeResponse};
use crate::{MessageData, Summarizer, TgfeedAiError, TgfeedAiResult};

pub struct ClaudeClient {
    client: reqwest::Client,
    api_key: String,
}

impl ClaudeClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: config.api_key.clone(),
        }
    }
}

impl Summarizer for ClaudeClient {
    async fn summarize(&self, messages: Vec<MessageData>) -> TgfeedAiResult<String> {
        if messages.is_empty() {
            return Ok("No messages to summarize.".to_string());
        }

        let formatted: Vec<String> = messages
            .iter()
            .map(
                |MessageData {
                     channel_handle,
                     text,
                     date,
                 }| format!("@{channel_handle} ({date}):\n{text}",),
            )
            .collect();

        // TODO: from config

        let prompt = format!(
            r#"
            Summarize the following Telegram channel messages. They are news. Group by topic if possible. 
            Be concise:
            
            {}
            "#,
            formatted.join("\n\n")
        );
        let request = ClaudeRequest {
            model: "claude-sonnet-4-5-20250929".to_string(),
            max_tokens: 1024,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response = response.json::<ClaudeResponse>().await?;

        if let Some(error) = response.error {
            return Err(TgfeedAiError::Api(error.message));
        }

        Ok(response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_else(|| "No summary generated.".to_string()))
    }
}
