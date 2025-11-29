use tgfeed_repo::models::StoredMessage;

use crate::{MonitorResult, MonitorService};

impl MonitorService {
    pub(crate) async fn handle_update(&self, update: grammers_client::Update) -> MonitorResult<()> {
        // TODO: refactor
        match update {
            grammers_client::Update::NewMessage(message) if !message.outgoing() => {
                let peer_id = message.peer_id();

                // Only process channel messages
                if peer_id.kind() != grammers_session::defs::PeerKind::Channel {
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
                    .and_then(|p| p.username().map(String::from))
                    .unwrap_or_default();

                let stored = StoredMessage {
                    id: None,
                    channel_id,
                    channel_handle: channel_handle.clone(),
                    message_id: message.id(),
                    text: message.text().to_string(),
                    date: chrono::Utc::now(),
                };

                self.repo.store_message(stored).await?;
                tracing::debug!("Stored message from @{}", channel_handle);
            }
            _ => {}
        }

        Ok(())
    }
}
