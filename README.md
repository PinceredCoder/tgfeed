# tgfeed

A Telegram bot that aggregates and forwards messages from multiple channels in real-time.

## Features

- Subscribe to public Telegram channels
- Real-time message collection and forwarding
- AI-powered message summarization with Claude
- MongoDB storage with automatic deduplication
- User-based rate limiting

## Bot Commands

- `/subscribe @channel` - Subscribe to a channel
- `/unsubscribe @channel` - Unsubscribe from a channel
- `/list` - Show your subscriptions
- `/summarize` - Get AI summary (once per hour)
- `/help` - Show help message

## Setup

**Prerequisites:**
- Rust 1.70+
- MongoDB
- Telegram API credentials ([my.telegram.org](https://my.telegram.org/apps))
- Bot token ([@BotFather](https://t.me/BotFather))
- Claude API key ([Anthropic](https://console.anthropic.com))

**Install:**

```bash
cp Settings.toml.sample Settings.toml
# Edit Settings.toml with your credentials
cargo run
```

**Configuration:**

```toml
[monitor_config]
api_id = 123456
api_hash = "your_api_hash"

[bot_config]
token = "your_bot_token"

[repo_config]
connection_string = "mongodb://127.0.0.1:27017"

[ai_config.claude]
api_key = "your_claude_api_key"
```

On first run, authenticate with your Telegram phone number.

## Architecture

```
tgfeed/                  # Service orchestrator
├── tgfeed-bot/          # Bot commands (teloxide)
├── tgfeed-monitor/      # Channel monitoring (grammers MTProto)
├── tgfeed-ai/           # Claude API integration
├── tgfeed-repo/         # MongoDB data layer
└── tgfeed-common/       # Shared types
```

Services communicate via async MPSC channels.

## License

MIT
