mod config;
mod error;
mod message;
pub mod models;
mod subscription;
mod summarize;
mod user;

pub use config::Config;
pub use error::{TgFeedRepoError, TgFeedRepoResult};

use crate::models::{StoredMessage, Subscription, SummarizeState, User};

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

        let this = Self { db };
        this.create_indexes().await?;

        Ok(this)
    }

    async fn create_indexes(&self) -> TgFeedRepoResult<()> {
        use mongodb::IndexModel;
        use mongodb::bson::doc;
        use mongodb::options::IndexOptions;

        // Subscriptions indexes
        self.subscriptions()
            .create_index(IndexModel::builder().keys(doc! { "user_id": 1 }).build())
            .await?;

        self.subscriptions()
            .create_index(IndexModel::builder().keys(doc! { "channel_id": 1 }).build())
            .await?;

        // Unique constraint: one subscription per user per channel
        self.subscriptions()
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "user_id": 1, "channel_id": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        // Messages indexes
        self.messages()
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "channel_id": 1, "date": -1 })
                    .build(),
            )
            .await?;

        // Unique constraint: one message per channel per message_id
        self.messages()
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "channel_id": 1, "message_id": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        // Summarize state index
        self.summarize_state()
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "user_id": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        // Users index
        self.users()
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "telegram_id": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        tracing::info!("Database indexes created/verified");

        Ok(())
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

    fn users(&self) -> mongodb::Collection<User> {
        self.db.collection("users")
    }
}
