# tgfeed

A Telegram userbot that aggregates messages from multiple channels and provides AI-powered summaries

## Features

- Subscribe to any public Telegram channel
- Automatically collect and store messages in MongoDB
- Generate AI summaries of recent news
- Simple command interface via Telegram DMs

## Commands

| Command | Description |
|---------|-------------|
| `/subscribe @channel` | Subscribe to a channel |
| `/unsubscribe @channel` | Unsubscribe from a channel |
| `/list` | Show all active subscriptions |
| `/summarize` | Get AI summary of messages since last summary |
| `/help` | Show help message |

## Setup

### Prerequisites

- Rust 1.70+
- MongoDB
- Telegram API credentials (from [my.telegram.org](https://my.telegram.org/apps))
- Claude API key (from [console.anthropic.com](https://console.anthropic.com/))

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/tgfeed.git
   cd tgfeed
   ```

2. Copy and edit the config:
   ```bash
   cp config.toml.example config.toml
   ```

3. Fill in your credentials in `config.toml`:
   ```toml
   [telegram]
   api_id = 123456
   api_hash = "your_api_hash"
   session_file = "tgfeed.session"

   [mongodb]
   uri = "mongodb://localhost:27017"
   database = "tgfeed"

   [claude]
   api_key = "sk-ant-..."
   model = "claude-sonnet-4-20250514"
   ```

4. Start MongoDB:
   ```bash
   mongod
   ```

5. Build and run:
   ```bash
   cargo run
   ```

6. On first run, you'll be prompted to authenticate with your Telegram account (phone number + code).

## Usage

1. Open Telegram and message your bot account
2. Use `/subscribe @bbcnews` to start following a channel
3. Wait for messages to accumulate
4. Use `/summarize` to get an AI-generated summary

## Project Structure

```
src/
├── main.rs         # Entry point and event loop
├── config.rs       # Configuration loading
├── client.rs       # Telegram client setup and auth
├── handlers.rs     # Command handlers
├── db.rs           # MongoDB operations
├── models.rs       # Data structures
└── summarizer.rs   # Claude API integration
```

## Dependencies

- `grammers-client` - Telegram MTProto client
- `mongodb` - Database driver
- `tokio` - Async runtime
- `reqwest` - HTTP client for Claude API
- `serde` / `toml` - Configuration parsing
- `chrono` - Date/time handling
- `tracing` - Logging

## License

MIT
