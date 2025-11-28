use std::sync::Arc;

use crate::config::Config;
use crate::utils::prompt;

pub struct TgFeedService {
    client: grammers_client::Client,
    api_hash: String,
    // need to store to keep session alive
    #[allow(unused)]
    handle: grammers_mtsender::SenderPoolHandle,
    // TODO: remove
    #[allow(unused)]
    repo: tgfeed_repo::Repo,
}

impl TgFeedService {
    pub fn new(config: &Config, repo: tgfeed_repo::Repo) -> anyhow::Result<Self> {
        let session = Arc::new(grammers_session::storages::SqliteSession::open(
            &config.session_file,
        )?);
        let sender_pool = grammers_mtsender::SenderPool::new(Arc::clone(&session), config.api_id);
        let client = grammers_client::client::Client::new(&sender_pool);

        let grammers_mtsender::SenderPool {
            runner,
            updates: _updates,
            handle,
        } = sender_pool;

        tokio::spawn(runner.run());

        Ok(TgFeedService {
            client,
            handle,
            api_hash: config.api_hash.clone(),
            repo,
        })
    }

    pub async fn authorize(&self) -> anyhow::Result<()> {
        tracing::info!("Checking authorization status...");

        if self.client.is_authorized().await? {
            tracing::info!("Already authorized");
            return Ok(());
        }

        tracing::info!("Not authorized, starting sign-in flow...");

        let phone = prompt("Enter your phone number (e.g., +1234567890): ")?;
        let token = self
            .client
            .request_login_code(&phone, &self.api_hash)
            .await?;

        let code = prompt("Enter the code you received: ")?;

        let signed_in = self.client.sign_in(&token, &code).await;

        match signed_in {
            Ok(_user) => {
                tracing::info!("Signed in successfully!");
            }
            Err(grammers_client::SignInError::PasswordRequired(password_token)) => {
                let password = prompt("2FA is enabled. Enter your password: ")?;
                self.client
                    .check_password(password_token, password.trim())
                    .await?;
                tracing::info!("Signed in with 2FA!");
            }
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }
}
