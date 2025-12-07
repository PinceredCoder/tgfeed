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
            Сделай сводку новотей из следущих сообщений из Telegram-каналов. Сгруппируй по теме, если возможно.
            Форматируй, исползуья HTML-тэги: <b>bold</b>, <i>italic</i>, <u>underline</u>. Не используй Markdown.
            Будь краток. В начале напиши, на основе какого количества постов сделана сводка.
            
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

        // Check status before parsing
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(%status, %body, "Claude API error");
            return Err(TgfeedAiError::Api(format!("{status}: {body}")));
        }

        let response = response.json::<ClaudeResponse>().await?;

        if let Some(error) = response.error {
            return Err(TgfeedAiError::Api(error.message));
        }

        let summary = response
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<String>();

        tracing::info!(%summary, "generated summary");

        if summary.is_empty() {
            Ok("No summary generated".to_string())
        } else {
            Ok(summary)
        }
    }
}
