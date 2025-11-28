mod handlers;

use std::mem::MaybeUninit;
use std::sync::Arc;

use tgfeed_repo::models::StoredMessage;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinSet;

use crate::config::Config;
use crate::utils::{parse_command, prompt};

pub struct TgFeedService {
    client: grammers_client::Client,
    api_hash: String,
    // need to store to keep session alive
    handle: grammers_mtsender::SenderPoolHandle,
    updates: MaybeUninit<UnboundedReceiver<grammers_session::updates::UpdatesLike>>,
    repo: tgfeed_repo::Repo,
}

impl TgFeedService {
    pub fn new(config: &Config, repo: tgfeed_repo::Repo) -> anyhow::Result<Arc<Self>> {
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

        Ok(Arc::new(TgFeedService {
            client,
            handle,
            updates: MaybeUninit::new(updates),
            api_hash: config.api_hash.clone(),
            repo,
        }))
    }

    pub async fn authorize(&self) -> anyhow::Result<()> {
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

    async fn log_credentials(&self) -> anyhow::Result<()> {
        let me = self.client.get_me().await?;
        tracing::info!(
            "Logged in as: {} (ID: {})",
            me.username().unwrap_or("N/A"),
            me.bare_id()
        );
        Ok(())
    }

    pub async fn run(self: Arc<Self>) -> anyhow::Result<()> {
        let mut updates = self.client.stream_updates(
            unsafe { self.updates.assume_init_read() },
            grammers_client::UpdatesConfiguration {
                catch_up: true,
                update_queue_limit: Some(100),
            },
        );

        let mut handler_tasks = JoinSet::new();

        tracing::info!("Start listening for updates...");
        loop {
            // Empty finished handlers
            while let Some(_handle) = handler_tasks.try_join_next() {}

            tokio::select! {
                _ = tokio::signal::ctrl_c() => break,
                update = updates.next() => {
                    let update = update?;
                    let this = Arc::clone(&self);
                    handler_tasks.spawn(async move {if let Err(e) = this.handle_update(update).await {
                        tracing::error!(%e, "failed handling update");
                    }});
                }
            }
        }

        tracing::info!("Saving session file...");
        updates.sync_update_state();

        self.handle.quit();

        // TODO: wait for other handlers

        Ok(())
    }

    async fn handle_update(&self, update: grammers_client::Update) -> anyhow::Result<()> {
        match update {
            grammers_client::Update::NewMessage(message) if !message.outgoing() => {
                let peer = message
                    .peer()
                    .map_err(|_| anyhow::anyhow!("unable to get message's peer"))?;
                let peer_handle = peer.username().map(String::from).unwrap_or_default();
                let text = message.text();
                let sender_id = message
                    .sender()
                    .map(|s| s.id().bare_id())
                    .ok_or_else(|| anyhow::anyhow!("undefined sender"))?;

                tracing::debug!(
                    "Message from {sender_id} in {}: {text}",
                    peer.id().bare_id(),
                );

                match peer {
                    grammers_client::types::Peer::Channel(channel) => {
                        // Check if this is from a subscribed channel
                        if self.repo.is_subscribed(channel.bare_id()).await? {
                            // Store the message
                            let stored = StoredMessage {
                                id: None,
                                channel_id: channel.bare_id(),
                                channel_handle: peer_handle.clone(),
                                message_id: message.id(),
                                text: text.to_string(),
                                date: chrono::Utc::now(),
                            };
                            self.repo.store_message(stored).await?;
                            tracing::debug!("Stored message from @{}", peer_handle,);
                        }
                    }
                    // TODO: make Enum for commands
                    grammers_client::types::Peer::User(_user) => {
                        if let Some((command, args)) = parse_command(text) {
                            let response = match command {
                                "subscribe" => self.handle_subscribe(args).await?,
                                "unsubscribe" => self.handle_unsubscribe(args).await?,
                                "list" => self.handle_list().await?,
                                "summarize" => self.handle_summarize(sender_id).await?,
                                "help" | "start" => self.handle_help(),
                                _ => "Unknown command. Use /help to see available commands."
                                    .to_string(),
                            };

                            self.client.send_message(peer, response).await?;
                        }
                    }
                    _ => (),
                }
            }
            _ => {}
        }

        Ok(())
    }
}
