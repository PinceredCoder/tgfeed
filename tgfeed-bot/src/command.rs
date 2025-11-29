use teloxide::utils::command::BotCommands;

#[derive(BotCommands)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub(crate) enum Command {
    #[command(description = "Show this help message")]
    Help,
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Subscribe to a channel: /subscribe @channel")]
    Subscribe(String),
    #[command(description = "Unsubscribe from a channel: /unsubscribe @channel")]
    Unsubscribe(String),
    #[command(description = "List all subscriptions")]
    List,
    #[command(description = "Get AI summary of recent messages")]
    Summarize,
}
