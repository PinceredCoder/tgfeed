use tgfeed_repo::models::Subscription;

use crate::{MonitorResult, MonitorService};

impl MonitorService {
    pub(crate) async fn subscribe_to_channel(
        &self,
        user_id: i64,
        channel_handle: String,
    ) -> MonitorResult<()> {
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
                && let Some(new_handle) = self.get_handle(&resolved)
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
}
