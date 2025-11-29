use mongodb::bson::doc;

use crate::models::Subscription;
use crate::{Repo, TgFeedRepoResult};

impl Repo {
    pub async fn add_subscription(&self, sub: Subscription) -> TgFeedRepoResult<()> {
        self.subscriptions()
            .update_one(
                doc! { "user_id": sub.user_id, "channel_handle": &sub.channel_handle },
                doc! { "$set": mongodb::bson::to_document(&sub)? },
            )
            .upsert(true)
            .await?;

        Ok(())
    }

    pub async fn remove_subscription(
        &self,
        user_id: i64,
        channel_handle: &str,
    ) -> TgFeedRepoResult<bool> {
        let result = self
            .subscriptions()
            .delete_one(doc! { "user_id": user_id, "channel_handle": channel_handle })
            .await?;

        Ok(result.deleted_count > 0)
    }

    pub async fn get_user_subscriptions(
        &self,
        user_id: i64,
    ) -> TgFeedRepoResult<Vec<Subscription>> {
        use futures::TryStreamExt;

        let cursor = self
            .subscriptions()
            .find(doc! { "user_id": user_id })
            .await?;

        let subs: Vec<Subscription> = cursor.try_collect().await?;
        Ok(subs)
    }

    pub async fn is_subscribed(&self, channel_handle: &str) -> TgFeedRepoResult<bool> {
        let count = self
            .subscriptions()
            .count_documents(doc! { "channel_handle": channel_handle })
            .await?;

        Ok(count > 0)
    }

    pub async fn get_channel_subscribers(
        &self,
        channel_handle: &str,
    ) -> TgFeedRepoResult<Vec<i64>> {
        use futures::TryStreamExt;

        let cursor = self
            .subscriptions()
            .find(doc! { "channel_handle": channel_handle })
            .await?;

        let subs: Vec<Subscription> = cursor.try_collect().await?;
        Ok(subs.into_iter().map(|s| s.user_id).collect())
    }

    pub async fn is_user_subscribed(
        &self,
        user_id: i64,
        channel_handle: &str,
    ) -> TgFeedRepoResult<bool> {
        let count = self
            .subscriptions()
            .count_documents(doc! { "user_id": user_id, "channel_handle": channel_handle })
            .await?;

        Ok(count > 0)
    }

    pub async fn has_subscribers(&self, channel_handle: &str) -> TgFeedRepoResult<bool> {
        let count = self
            .subscriptions()
            .count_documents(doc! { "channel_handle": channel_handle })
            .await?;

        Ok(count > 0)
    }

    pub async fn get_subscribed_channels(&self) -> TgFeedRepoResult<Vec<String>> {
        let cursor = self
            .subscriptions()
            .distinct("channel_handle", doc! {})
            .await?;

        let channel_handles = cursor
            .into_iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        Ok(channel_handles)
    }
}
