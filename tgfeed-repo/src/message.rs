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
        since: chrono::DateTime<Utc>,
    ) -> TgFeedRepoResult<Vec<StoredMessage>> {
        use futures::TryStreamExt;

        let cursor = self
            .messages()
            .find(doc! {
                "date": { "$gte": since }
            })
            .sort(doc! { "date": 1 })
            .await?;

        let messages: Vec<StoredMessage> = cursor.try_collect().await?;
        Ok(messages)
    }
}
