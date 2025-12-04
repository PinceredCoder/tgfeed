use chrono::Utc;
use mongodb::bson::doc;

use crate::models::StoredMessage;
use crate::{Repo, TgFeedRepoResult};

impl Repo {
    pub async fn store_message(&self, msg: StoredMessage) -> TgFeedRepoResult<()> {
        // Upsert to avoid duplicates
        self.messages()
            .update_one(
                doc! {
                    "channel_id": msg.channel_id,
                    "message_id": msg.message_id
                },
                doc! { "$set": mongodb::bson::to_document(&msg)? },
            )
            .upsert(true)
            .await?;

        Ok(())
    }

    pub async fn get_messages_since(
        &self,
        channel_ids: &[i64],
        since: chrono::DateTime<Utc>,
        limit: i64,
    ) -> TgFeedRepoResult<Vec<StoredMessage>> {
        use futures::TryStreamExt;

        let cursor = self
            .messages()
            .find(doc! {
                "channel_id": { "$in": channel_ids },
                "date": { "$gte": since },
                "$expr": { "$gt": [{ "$strLenCP": "$text" }, 30] }
            })
            .sort(doc! { "date": -1 })
            .limit(limit)
            .await?;

        let messages: Vec<StoredMessage> = cursor.try_collect().await?;
        Ok(messages)
    }
}
