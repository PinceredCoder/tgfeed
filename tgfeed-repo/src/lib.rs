mod config;
mod error;
mod message;
mod models;
mod subscription;
mod summarize;

pub use config::Config;
pub use error::{TgFeedRepoError, TgFeedRepoResult};

use crate::models::{StoredMessage, Subscription, SummarizeState};

#[derive(Clone)]
pub struct Repo {
    db: mongodb::Database,
}

impl Repo {
    pub async fn new(config: &Config) -> TgFeedRepoResult<Self> {
        let client_options =
            mongodb::options::ClientOptions::parse(&config.connection_string).await?;

        let client = mongodb::Client::with_options(client_options)?;

        let db = client.database(&config.database_name);

        Ok(Self { db })
    }

    fn subscriptions(&self) -> mongodb::Collection<Subscription> {
        self.db.collection("subscriptions")
    }

    fn messages(&self) -> mongodb::Collection<StoredMessage> {
        self.db.collection("messages")
    }

    fn summarize_state(&self) -> mongodb::Collection<SummarizeState> {
        self.db.collection("summarize_state")
    }
}
