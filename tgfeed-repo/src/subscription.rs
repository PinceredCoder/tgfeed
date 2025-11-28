use mongodb::bson::doc;

use crate::models::Subscription;
use crate::{Repo, TgFeedRepoResult};

impl Repo {
    pub async fn add_subscription(
        &self,
        channel_id: i64,
        channel_handle: String,
    ) -> TgFeedRepoResult<()> {
        let sub = Subscription {
            channel_id,
            channel_handle,
            subscribed_at: chrono::Utc::now(),
        };

        self.subscriptions()
            .update_one(
                doc! { "channel_id": channel_id },
                doc! { "$set": mongodb::bson::to_document(&sub)? },
            )
            .upsert(true)
            .await?;

        Ok(())
    }

    pub async fn remove_subscription(&self, channel_id: i64) -> TgFeedRepoResult<bool> {
        let result = self
            .subscriptions()
            .delete_one(doc! { "channel_id": channel_id })
            .await?;

        Ok(result.deleted_count > 0)
    }

    pub async fn get_subscriptions(&self) -> TgFeedRepoResult<Vec<Subscription>> {
        use futures::TryStreamExt;

        let cursor = self.subscriptions().find(doc! {}).await?;

        let subs: Vec<Subscription> = cursor.try_collect().await?;
        Ok(subs)
    }

    pub async fn is_subscribed(&self, channel_id: i64) -> TgFeedRepoResult<bool> {
        let count = self
            .subscriptions()
            .count_documents(doc! { "channel_id": channel_id })
            .await?;

        Ok(count > 0)
    }
}
