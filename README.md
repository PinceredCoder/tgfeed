# tgfeed

Telegram bot that aggregates messages from multiple channels into a single stream with AI-powered summarization.

## Commands

- `/subscribe @channel` - Subscribe to a channel
- `/unsubscribe @channel` - Unsubscribe from a channel
- `/list` - Show subscriptions
- `/summarize` - Get AI summary (once per hour)

## Quick Start

```bash
# 1. Copy config template
cp Settings.toml.sample Settings.toml

# 2. Add your credentials to Settings.toml:
#    - Telegram API ID/hash: https://my.telegram.org/apps
#    - Bot token: @BotFather
#    - MongoDB connection string
#    - Claude API key: https://console.anthropic.com

# 3. Run
cargo run

# 4. Authenticate with your Telegram phone number (first run only)
```

## Requirements

- Rust 1.70+
- MongoDB
- Telegram API credentials
- Claude API key

## Architecture

```
tgfeed/              # Orchestrator
├── tgfeed-bot/      # User interface (teloxide)
├── tgfeed-monitor/  # Channel monitoring (MTProto)
├── tgfeed-ai/       # Summarization (Claude)
├── tgfeed-repo/     # Database (MongoDB)
└── tgfeed-common/   # Shared types
```

## License

MIT
