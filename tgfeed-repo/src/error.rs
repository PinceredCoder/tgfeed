#[derive(Debug, thiserror::Error)]
pub enum TgFeedRepoError {
    #[error("database error: {0}")]
    MongodbError(#[from] mongodb::error::Error),

    #[error("serialization error: {0}")]
    MongodbSerializationError(#[from] mongodb::bson::ser::Error),
}

pub type TgFeedRepoResult<T> = Result<T, TgFeedRepoError>;
