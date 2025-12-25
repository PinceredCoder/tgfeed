use teloxide::utils::command::BotCommands;

use crate::command::Command;

pub fn start() -> String {
    "👋 Hello! This is a Telegram channels aggregator. Run /help to see the available commands."
        .to_string()
}

pub fn help() -> String {
    Command::descriptions().to_string()
}

pub fn usage() -> String {
    "Usage: /unsubscribe @channelname".to_string()
}

pub fn unknown_command() -> String {
    "❌ Unknown command".to_string()
}

pub fn internal_server_error() -> String {
    "❌ Internal server error".to_string()
}
