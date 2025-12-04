use tgfeed_ai::{MessageData, Summarizer};
use tgfeed_repo::models::Subscription;

use crate::{MonitorError, MonitorResult, MonitorService};

// TODO: from config?
const MAX_SUBSCRIPTIONS_PER_USER: usize = 30;

impl<S: Summarizer> MonitorService<S> {
    pub(crate) async fn subscribe_to_channel(
        &self,
        user_id: i64,
        channel_handle: String,
    ) -> MonitorResult<()> {
        let current_subs = self.repo.get_user_subscriptions(user_id).await?;
        if current_subs.len() >= MAX_SUBSCRIPTIONS_PER_USER {
            return Err(MonitorError::SubscriptionLimit(MAX_SUBSCRIPTIONS_PER_USER));
        }

        let channel = self.resolve_peer(&channel_handle).await?;
        let channel_id = channel.id().bare_id();

        if self.repo.is_user_subscribed(user_id, channel_id).await? {
            return Ok(());
        }

        if let Err(error) = self.client.join_chat(&channel).await {
            tracing::warn!(
                %error,
                "Could not join channel @{channel_handle} (might already be member)"
            );
        }

        self.repo
            .add_subscription(Subscription {
                user_id,
                channel_id,
                channel_handle,
                subscribed_at: chrono::Utc::now(),
            })
            .await?;

        Ok(())
    }

    pub(crate) async fn unsubscribe_from_channel(
        &self,
        user_id: i64,
        channel_handle: String,
    ) -> MonitorResult<()> {
        let removed = self
            .repo
            .remove_subscription_by_handle(user_id, &channel_handle)
            .await?;

        if !removed {
            tracing::warn!(
                %channel_handle,
                "handle not found in the database, trying to resolve the peer"
            );

            let resolved = self.resolve_peer(&channel_handle).await?;
            let channel_id = resolved.id().bare_id();

            if self.repo.remove_subscription(user_id, channel_id).await?
                && let Some(new_handle) = Self::get_handle(&resolved)
            {
                self.repo
                    .update_subscription_handle(channel_id, &new_handle)
                    .await?;
            }
        }

        Ok(())
    }

    pub(crate) async fn list_subscriptions(&self, user_id: i64) -> MonitorResult<Vec<String>> {
        Ok(self
            .repo
            .get_user_subscriptions(user_id)
            .await
            .map(|s| s.into_iter().map(|c| c.channel_handle).collect())?)
    }

    pub(crate) async fn summarize(&self, user_id: i64) -> MonitorResult<String> {
        let subscriptions = self.repo.get_user_subscriptions(user_id).await?;

        if subscriptions.is_empty() {
            return Ok("No subscriptions to summarize.".to_string());
        }

        let since = match self.repo.get_last_summarize_time(user_id).await {
            Ok(time) => time,
            Err(error) => {
                tracing::error!(%error, "Failed to get last summarize time");
                chrono::Utc::now() - chrono::Duration::days(3)
            }
        };

        let mut channels_map: std::collections::HashMap<i64, String> = subscriptions
            .into_iter()
            .map(|s| (s.channel_id, s.channel_handle))
            .collect();

        let channel_ids = channels_map.keys().cloned().collect::<Vec<_>>();

        // Get messages
        let messages = self
            .repo
            .get_messages_since(&channel_ids, since, 150)
            .await?;

        if messages.is_empty() {
            return Ok("No new messages since last summary.".to_string());
        }

        let messages_data = messages
            .into_iter()
            .filter_map(|m| {
                let channel_handle = channels_map.remove(&m.channel_id)?;

                Some(MessageData {
                    channel_handle,
                    text: m.text,
                    date: m.date,
                })
            })
            .collect::<Vec<_>>();

        self.repo.update_summarize_time(user_id).await?;

        Ok(self.summarizer.summarize(messages_data).await?)
    }
}
