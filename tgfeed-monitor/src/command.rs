use tgfeed_repo::models::Subscription;

use crate::{MonitorError, MonitorResult, MonitorService};

// TODO: change handle to id, as handle can be easily changed

impl MonitorService {
    pub(crate) async fn subscribe_to_channel(
        &self,
        user_id: i64,
        channel_handle: String,
    ) -> MonitorResult<()> {
        if self
            .repo
            .is_user_subscribed(user_id, &channel_handle)
            .await?
        {
            return Ok(());
        }

        let resolved = self.client.resolve_username(&channel_handle).await?;

        let peer = match resolved {
            Some(peer) => peer,
            None => return Err(MonitorError::NotFound(channel_handle)),
        };

        let channel_id = peer.id().bare_id();

        if let Err(error) = self.client.join_chat(&peer).await {
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
        self.repo
            .remove_subscription(user_id, &channel_handle)
            .await?;

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
