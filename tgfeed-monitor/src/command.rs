use crate::MonitorService;

impl MonitorService {
    pub(crate) async fn subscribe_to_channel(&self, channel: &str) -> Result<(), String> {
        // Resolve the channel
        let resolved = self
            .client
            .resolve_username(channel)
            .await
            .map_err(|e| format!("Failed to resolve channel: {}", e))?;

        let peer = match resolved {
            Some(peer) => peer,
            None => return Err(format!("Channel @{} not found", channel)),
        };

        let channel_id = peer.id().bare_id();

        // Check if already subscribed
        if self
            .repo
            .is_subscribed(channel_id)
            .await
            .map_err(|e| e.to_string())?
        {
            return Err(format!("Already subscribed to @{}", channel));
        }

        // Try to join the channel
        if let Err(e) = self.client.join_chat(&peer).await {
            tracing::warn!("Could not join channel (might already be member): {}", e);
        }

        // Save subscription
        self.repo
            .add_subscription(channel_id, channel.to_string())
            .await
            .map_err(|e| e.to_string())?;

        tracing::info!("Subscribed to @{}", channel);
        Ok(())
    }

    pub(crate) async fn unsubscribe_from_channel(&self, channel: &str) -> Result<(), String> {
        let subs = self
            .repo
            .get_subscriptions()
            .await
            .map_err(|e| e.to_string())?;

        let sub = subs.iter().find(|s| s.channel_handle == channel);

        match sub {
            Some(s) => {
                self.repo
                    .remove_subscription(s.channel_id)
                    .await
                    .map_err(|e| e.to_string())?;
                tracing::info!("Unsubscribed from @{}", channel);
                Ok(())
            }
            None => Err(format!("Not subscribed to @{}", channel)),
        }
    }

    pub(crate) async fn list_subscriptions(&self) -> Vec<String> {
        match self.repo.get_subscriptions().await {
            Ok(subs) => subs.into_iter().map(|s| s.channel_handle).collect(),
            Err(e) => {
                tracing::error!("Failed to list subscriptions: {}", e);
                vec![]
            }
        }
    }
}
