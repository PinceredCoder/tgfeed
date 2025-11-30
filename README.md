# tgfeed

A Rust-based Telegram bot that aggregates messages from multiple Telegram channels. Combines Telegram Bot API (teloxide) for user commands and MTProto client (grammers) for real-time channel monitoring.

## Features

- Subscribe to public Telegram channels and collect messages in real-time
- Persistent message storage in MongoDB with automatic deduplication
- Telegram bot interface with interactive commands
- Session persistence across restarts
- Full 2FA authentication support (phone, SMS, 2FA password)
- Message forwarding to subscribed users with source links

## Bot Commands

| Command | Description |
|---------|-------------|
| `/subscribe @channel` | Subscribe to a channel |
| `/unsubscribe @channel` | Unsubscribe from a channel |
| `/list` | Show all active subscriptions |
| `/help` or `/start` | Show help message |
| `/summarize` | Get AI summary (placeholder) |

## Quick Start

### Prerequisites

- Rust 1.70+ (Edition 2024)
- MongoDB running locally or remotely
- Telegram API credentials from [my.telegram.org](https://my.telegram.org/apps)
- Telegram Bot token from [@BotFather](https://t.me/BotFather)

### Installation

1. Clone and configure:
   ```bash
   git clone https://github.com/yourusername/tgfeed.git
   cd tgfeed
   cp Settings.toml.sample Settings.toml
   ```

2. Edit `Settings.toml` with your credentials:
   ```toml
   [monitor_config]
   api_id = 123456                           # From my.telegram.org
   api_hash = "your_api_hash_here"
   session_file = "session.sqlite"

   [bot_config]
   token = "your_bot_token_here"             # From @BotFather

   [repo_config]
   connection_string = "mongodb://127.0.0.1:27017"
   database_name = "tgfeed_db"
   ```

   Or use environment variables:
   ```bash
   export MONITOR_CONFIG__API_ID=123456
   export MONITOR_CONFIG__API_HASH="your_api_hash"
   export BOT_CONFIG__TOKEN="your_bot_token"
   export REPO_CONFIG__CONNECTION_STRING="mongodb://127.0.0.1:27017"
   ```

3. Start MongoDB and run:
   ```bash
   mongod                    # Start MongoDB
   cargo run                 # Build and run tgfeed
   ```

4. On first run, authenticate with your Telegram account (phone + SMS code + 2FA password if enabled)

## Usage

1. Message your bot on Telegram
2. Use `/subscribe @channelname` to follow channels
3. Messages from subscribed channels are automatically collected and forwarded
4. Use `/list` to view subscriptions, `/unsubscribe @channel` to stop following

## Architecture

Modular workspace with 5 crates:

```
tgfeed/                    # Main orchestrator - service spawning & shutdown
├── tgfeed-bot/            # Telegram Bot API (teloxide) - user commands
├── tgfeed-monitor/        # MTProto client (grammers) - channel monitoring
├── tgfeed-common/         # Shared types for inter-service communication
└── tgfeed-repo/           # MongoDB data access layer
```

**Communication flow:**
- Bot receives user commands → sends to Monitor via MPSC channels
- Monitor handles subscriptions and streams channel updates
- New messages stored in MongoDB and forwarded to subscribed users

## Database Collections

- **subscriptions** - User channel subscriptions (user_id, channel_id, channel_handle)
- **messages** - Collected messages (indexed on channel_id + message_id for deduplication)
- **summarize_state** - Summary tracking (for future AI integration)

## Development

```bash
cargo build              # Build all crates
cargo test               # Run tests
cargo fmt                # Format code
RUST_LOG=debug cargo run # Run with debug logging
```

## Troubleshooting

**MongoDB connection failed:** Ensure `mongod` is running and connection string is correct

**Authentication failed:** Verify API credentials, use correct phone format (+1234567890), delete `session.sqlite` to re-authenticate

**Bot not responding:** Check bot token, verify bot service is running, enable debug logs with `RUST_LOG=debug`

**Channel subscription failed:** Ensure channel is public and handle is correct (@channelname)

## License

MIT
