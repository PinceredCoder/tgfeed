use crate::service::TgFeedService;

impl TgFeedService {
    /// Handle /subscribe @channel command
    pub async fn handle_subscribe(&self, args: &str) -> anyhow::Result<String> {
        let channel_handle = args.trim().trim_start_matches('@');

        if channel_handle.is_empty() {
            return Ok("Usage: /subscribe @channelname".to_string());
        }

        // Resolve the channel
        let resolved = match self.client.resolve_username(channel_handle).await? {
            Some(chat) => chat,
            None => return Ok(format!("Channel @{} not found", channel_handle)),
        };

        let channel_id = resolved.id();

        // Check if already subscribed
        if self.repo.is_subscribed(channel_id.bare_id()).await? {
            return Ok(format!("Already subscribed to @{}", channel_handle));
        }

        if matches!(resolved, grammers_client::types::Peer::Channel(_)) {
            if let Err(e) = self.client.join_chat(resolved).await {
                tracing::warn!("Could not join channel: {}", e);
                // Continue anyway - might already be a member or it's a broadcast channel
            }
        } else {
            return Ok("This doesn't appear to be a channel".to_string());
        }

        // Save subscription
        self.repo
            .add_subscription(channel_id.bare_id(), channel_handle.to_string())
            .await?;

        Ok(format!("✅ Subscribed to @{}", channel_handle))
    }

    /// Handle /unsubscribe @channel command
    pub async fn handle_unsubscribe(&self, args: &str) -> anyhow::Result<String> {
        let channel_handle = args.trim().trim_start_matches('@');

        if channel_handle.is_empty() {
            return Ok("Usage: /unsubscribe @channelname".to_string());
        }

        // TODO: add method to repo to find by handle
        // Find subscription by username
        let subs = self.repo.get_subscriptions().await?;
        let sub = subs.iter().find(|s| s.channel_handle == channel_handle);

        // TODO: refactor (do not actually unsubscribe, mark the user)
        match sub {
            Some(s) => {
                self.repo.remove_subscription(s.channel_id).await?;
                Ok(format!("✅ Unsubscribed from @{}", channel_handle))
            }
            None => Ok(format!("Not subscribed to @{}", channel_handle)),
        }
    }

    /// Handle /list command
    pub async fn handle_list(&self) -> anyhow::Result<String> {
        let subs = self.repo.get_subscriptions().await?;

        if subs.is_empty() {
            return Ok("No active subscriptions".to_string());
        }

        let mut response = String::from("📋 Active subscriptions:\n");
        for sub in subs {
            response.push_str(&format!("• @{}\n", sub.channel_handle));
        }

        Ok(response)
    }

    /// Handle /summarize command
    pub async fn handle_summarize(&self, user_id: i64) -> anyhow::Result<String> {
        let since = self.repo.get_last_summarize_time(user_id).await?;
        let messages = self.repo.get_messages_since(since).await?;

        if messages.is_empty() {
            return Ok("No new messages since last summary.".to_string());
        }

        let count = messages.len();
        let summary = "<NOT IMPLEMENTED>"; //self.summarizer.summarize(&messages).await?;

        // Update last summarize time
        self.repo.update_summarize_time(user_id).await?;

        Ok(format!(
            "📰 Summary of {} messages since {}:\n\n{}",
            count,
            since.format("%Y-%m-%d %H:%M UTC"),
            summary
        ))
    }

    /// Handle /help command
    pub fn handle_help(&self) -> String {
        r#"🤖 TGFeed Bot Commands:

/subscribe @channel - Subscribe to a channel
/unsubscribe @channel - Unsubscribe from a channel
/list - Show all subscriptions
/summarize - Get AI summary of recent messages
/help - Show this help message"#
            .to_string()
    }
}
