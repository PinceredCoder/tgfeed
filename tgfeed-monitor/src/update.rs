use tgfeed_common::event::BotEvent;
use tgfeed_repo::models::StoredMessage;

use crate::{MonitorError, MonitorResult, MonitorService};

impl MonitorService {
    pub(crate) async fn handle_update(&self, update: grammers_client::Update) -> MonitorResult<()> {
        match update {
            grammers_client::Update::NewMessage(message) if !message.outgoing() => {
                let peer_id = message.peer_ref().id;

                // Only process channel messages
                if peer_id.kind() != grammers_session::types::PeerKind::Channel {
                    return Ok(());
                }

                let channel_id = peer_id.bare_id();

                // TODO: try p.usernameS()
                let channel_handle = message
                    .peer()
                    .ok()
                    .and_then(|p| p.username().map(String::from))
                    .ok_or_else(|| MonitorError::EmptyHandle)?;

                // Check if subscribed
                if !self.repo.is_subscribed(&channel_handle).await? {
                    return Ok(());
                }

                let text = message.text().to_string();
                let message_id = message.id();

                tracing::info!(
                    %channel_handle,
                    %message_id,
                    ?peer_id,
                    "new message"
                );

                let stored = StoredMessage {
                    id: None,
                    channel_id,
                    channel_handle: channel_handle.clone(),
                    message_id,
                    text: text.clone(),
                    date: chrono::Utc::now(),
                };

                self.repo.store_message(stored).await?;

                let event = BotEvent::NewMessage {
                    channel_id,
                    channel_handle,
                    message_id,
                    text,
                };

                if let Err(error) = self.event_tx.send(event).await {
                    tracing::error!(%error, "Failed sending event to bot");
                }
            }
            _ => {}
        }

        Ok(())
    }
}
