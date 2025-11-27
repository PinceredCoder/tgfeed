use chrono::Utc;
use mongodb::bson::doc;

use crate::models::SummarizeState;
use crate::{Repo, TgFeedRepoResult};

impl Repo {
    pub async fn get_last_summarize_time(
        &self,
        user_id: i64,
    ) -> TgFeedRepoResult<chrono::DateTime<Utc>> {
        let state = self
            .summarize_state()
            .find_one(doc! { "user_id": user_id })
            .await?;

        match state {
            Some(s) => Ok(s.last_summarized_at),
            // Default to 7 days ago if never summarized
            None => Ok(Utc::now() - chrono::Duration::days(7)),
        }
    }

    pub async fn update_summarize_time(&self, user_id: i64) -> TgFeedRepoResult<()> {
        let state = SummarizeState {
            user_id,
            last_summarized_at: Utc::now(),
        };

        self.summarize_state()
            .update_one(
                doc! { "user_id": user_id },
                doc! { "$set": mongodb::bson::to_document(&state)? },
            )
            .upsert(true)
            .await?;

        Ok(())
    }
}
