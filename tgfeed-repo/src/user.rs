use bson::doc;

use crate::{Repo, TgFeedRepoResult};

impl Repo {
    pub async fn is_user_allowed(&self, user_id: i64) -> TgFeedRepoResult<bool> {
        let count = self
            .users()
            .count_documents(doc! { "telegram_id": user_id, "allowed": true })
            .await?;

        Ok(count > 0)
    }
}
