use std::sync::OnceLock;

use regex::Regex;
use tgfeed_ai::Summarizer;
use tgfeed_common::event::BotEvent;
use tgfeed_repo::models::StoredMessage;

use crate::{MonitorError, MonitorResult, MonitorService};

// ERID tokens are typically 8+ characters, alphanumeric
pub(crate) const AD_PATTERN_STR: &str = r"(?i:#реклама|(?:^|[\s\/\\?&])erid[\s:=]+[a-z0-9]{8,})";

static AD_PATTERN: OnceLock<Regex> = OnceLock::new();

pub(crate) fn get_ad_pattern() -> &'static Regex {
    AD_PATTERN.get_or_init(|| Regex::new(AD_PATTERN_STR).unwrap())
}

impl<S: Summarizer> MonitorService<S> {
    pub(crate) async fn handle_update(&self, update: grammers_client::Update) -> MonitorResult<()> {
        match update {
            grammers_client::Update::NewMessage(message) if !message.outgoing() => {
                let peer_id = message.peer_ref().id;

                // Only process channel messages
                if peer_id.kind() != grammers_session::types::PeerKind::Channel {
                    return Ok(());
                }

                let channel_id = peer_id.bare_id();

                // Check if subscribed
                if !self.repo.is_subscribed(channel_id).await? {
                    return Ok(());
                }

                let channel_handle = message
                    .peer()
                    .ok()
                    .and_then(|p| Self::get_handle(p))
                    .ok_or_else(|| MonitorError::EmptyHandle)?;

                let message_id = message.id();

                tracing::info!(
                    %channel_handle,
                    %message_id,
                    ?peer_id,
                    "new message"
                );

                let text = message.text().to_string();

                // Skip ads: messages with an ad hashtag or an Erid token
                if get_ad_pattern().is_match(&text) {
                    tracing::info!(
                        %channel_handle,
                        %message_id,
                        "skipping ad message"
                    );

                    return Ok(());
                }

                // Skip empty messages - probably some media files
                if text.is_empty() {
                    tracing::info!(
                        %channel_handle,
                        %message_id,
                        "skipping empty message"
                    );

                    return Ok(());
                }

                // Do not store too short
                if text.len() >= 20 {
                    let stored = StoredMessage {
                        id: None,
                        channel_id,
                        message_id,
                        text: text.clone(),
                        date: chrono::Utc::now(),
                    };

                    self.repo.store_message(stored).await?;
                }

                let entities = tgfeed_common::utils::convert_entities(message.fmt_entities());

                match self.repo.get_channel_subscribers(channel_id).await {
                    Ok(subscribers) => {
                        let event = BotEvent::NewMessage {
                            channel_id,
                            channel_handle,
                            message_id,
                            text,
                            subscribers,
                            entities,
                        };

                        if let Err(error) = self.event_tx.send(event).await {
                            tracing::error!(%error, "Failed sending event to bot");
                        }
                    }
                    Err(error) => {
                        tracing::error!(%error, "Failed to get subscribers");
                    }
                };
            }
            _ => {}
        }

        Ok(())
    }
}
