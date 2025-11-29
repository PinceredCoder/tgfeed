#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    #[error("Grammers session storage error: {0}")]
    Session(#[from] sqlite::Error),

    #[error("Grammers invocation error: {0}")]
    Invocation(#[from] grammers_mtsender::InvocationError),

    #[error("Sign in error: {0}")]
    SignIn(Box<grammers_client::SignInError>),

    #[error("Repository error: {0}")]
    Repo(#[from] tgfeed_repo::TgFeedRepoError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<grammers_client::SignInError> for MonitorError {
    fn from(err: grammers_client::SignInError) -> Self {
        MonitorError::SignIn(Box::new(err))
    }
}

pub type MonitorResult<T> = Result<T, MonitorError>;
