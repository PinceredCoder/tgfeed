mod command;
mod config;
mod error;
mod update;
mod utils;

use std::mem::MaybeUninit;
use std::sync::Arc;

pub use config::Config;
pub use error::*;
use tgfeed_common::command::MonitorCommand;
use tgfeed_common::event::BotEvent;
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::utils::prompt;

pub struct MonitorService {
    client: grammers_client::Client,
    api_hash: String,
    // need to store to keep session alive
    handle: grammers_mtsender::SenderPoolHandle,
    updates: MaybeUninit<UnboundedReceiver<grammers_session::updates::UpdatesLike>>,
    repo: tgfeed_repo::Repo,
    command_rx: mpsc::Receiver<MonitorCommand>,
    event_tx: mpsc::Sender<BotEvent>,
}

impl MonitorService {
    pub fn new(
        config: &Config,
        repo: tgfeed_repo::Repo,
        command_rx: mpsc::Receiver<MonitorCommand>,
        event_tx: mpsc::Sender<BotEvent>,
    ) -> MonitorResult<Self> {
        let session = Arc::new(grammers_session::storages::SqliteSession::open(
            &config.session_file,
        )?);
        let sender_pool = grammers_mtsender::SenderPool::new(Arc::clone(&session), config.api_id);
        let client = grammers_client::client::Client::new(&sender_pool);

        let grammers_mtsender::SenderPool {
            runner,
            updates,
            handle,
        } = sender_pool;

        tokio::spawn(runner.run());

        Ok(MonitorService {
            client,
            handle,
            updates: MaybeUninit::new(updates),
            api_hash: config.api_hash.clone(),
            repo,
            command_rx,
            event_tx,
        })
    }

    pub async fn authorize(&self) -> MonitorResult<()> {
        tracing::info!("Checking authorization status...");

        if self.client.is_authorized().await? {
            self.log_credentials().await?;
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

        self.log_credentials().await?;

        Ok(())
    }

    async fn log_credentials(&self) -> MonitorResult<()> {
        let me = self.client.get_me().await?;
        tracing::info!(
            "Logged in as: {} (ID: {})",
            me.username().unwrap_or("N/A"),
            me.bare_id()
        );
        Ok(())
    }

    pub async fn run(mut self) -> MonitorResult<()> {
        let mut updates = self.client.stream_updates(
            unsafe { self.updates.assume_init_read() },
            grammers_client::UpdatesConfiguration {
                catch_up: false,
                ..Default::default()
            },
        );

        tracing::info!("Start listening for updates...");
        loop {
            tokio::select! {
                Some(cmd) = self.command_rx.recv() => {
                    if matches!(cmd, MonitorCommand::Shutdown) {
                        tracing::warn!("received shutdown command");
                        break;
                    }

                    self.handle_command(cmd).await;
                }

                update = updates.next() => {
                    match update {
                        Ok(update) => {
                            // TODO: spawn new task
                            if let Err(e) = self.handle_update(update).await {
                                tracing::error!("Error handling update: {}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error receiving update: {}", e);
                        }
                    }
                }
            }
        }

        tracing::info!("Saving session file...");
        updates.sync_update_state();

        self.handle.quit();

        // TODO: wait for other handlers

        Ok(())
    }

    async fn handle_command(&self, cmd: MonitorCommand) {
        match cmd {
            MonitorCommand::Subscribe {
                user_id,
                channel_handle,
                response,
            } => {
                let result = self.subscribe_to_channel(user_id, channel_handle).await;
                let _ = response.send(result.map_err(|e| e.to_string()));
            }
            MonitorCommand::Unsubscribe {
                user_id,
                channel_handle,
                response,
            } => {
                let result = self.unsubscribe_from_channel(user_id, channel_handle).await;
                let _ = response.send(result.map_err(|e| e.to_string()));
            }
            MonitorCommand::ListSubscriptions { user_id, response } => {
                let result = self.list_subscriptions(user_id).await;
                let _ = response.send(result.map_err(|e| e.to_string()));
            }
            _ => (),
        }
    }
}
