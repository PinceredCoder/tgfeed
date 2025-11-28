# tgfeed

A Telegram userbot that aggregates messages from multiple channels and provides AI-powered summaries.

## Features

- Subscribe to any public Telegram channel
- Automatically collect and store messages in MongoDB
- Generate AI summaries of recent messages (framework implemented, Claude integration pending)
- Simple command interface via Telegram DMs
- Session persistence with SQLite
- Graceful shutdown handling

## Commands

| Command | Description |
|---------|-------------|
| `/subscribe @channel` | Subscribe to a channel |
| `/unsubscribe @channel` | Unsubscribe from a channel |
| `/list` | Show all active subscriptions |
| `/summarize` | Get AI summary of messages since last summary (placeholder) |
| `/help` or `/start` | Show help message |

## Setup

### Prerequisites

- Rust 1.70+ (Edition 2024)
- MongoDB running locally or accessible remotely
- Telegram API credentials (from [my.telegram.org](https://my.telegram.org/apps))
- Claude API key (from [console.anthropic.com](https://console.anthropic.com/)) - for future summarization feature

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/tgfeed.git
   cd tgfeed
   ```

2. Copy and edit the config:
   ```bash
   cp Settings.toml.sample Settings.toml
   ```

3. Fill in your credentials in `Settings.toml`:
   ```toml
   # Server configuration
   server_addr = "0.0.0.0:8080"
   healthcheck_addr = "0.0.0.0:8081"

   # Telegram API credentials (get from https://my.telegram.org/apps)
   api_id = 123456
   api_hash = "your_api_hash_here"
   session_file = "session.sqlite"

   # MongoDB configuration
   [repo_config]
   connection_string = "mongodb://127.0.0.1:27017"
   database_name = "tgfeed_db"
   ```

4. Start MongoDB:
   ```bash
   mongod
   ```

5. Build and run:
   ```bash
   cargo run
   ```

6. On first run, you'll be prompted to authenticate with your Telegram account:
   - Phone number (with country code, e.g., +1234567890)
   - SMS verification code
   - Two-factor authentication password (if enabled)

## Usage

1. Open Telegram and message your bot account (the phone number you authenticated with)
2. Use `/subscribe @bbcnews` to start following a channel
3. Messages from subscribed channels will be automatically collected and stored
4. Use `/list` to view all your active subscriptions
5. Use `/summarize` to request a summary (currently returns a placeholder)

## Project Structure

This is a Cargo workspace with two crates:

```
tgfeed/                           # Root workspace
├── src/                          # Main application crate
│   ├── main.rs                   # Entry point and initialization
│   ├── config.rs                 # Configuration management (TOML + env vars)
│   ├── utils.rs                  # Utilities (command parsing, prompts)
│   └── service/
│       ├── mod.rs                # TgFeedService (client, auth, event loop)
│       └── handlers.rs           # Command handlers
│
└── tgfeed-repo/                  # Data access layer crate
    └── src/
        ├── lib.rs                # Repo struct and MongoDB client
        ├── config.rs             # Database configuration
        ├── error.rs              # Error types
        ├── models.rs             # Data models
        ├── subscription.rs       # Subscription operations
        ├── message.rs            # Message storage/retrieval
        └── summarize.rs          # Summarize state tracking
```

## Database Schema

**Database**: `tgfeed_db` (configurable)

**Collections**:

- `subscriptions` - Channel subscriptions
  - `channel_id` (i64)
  - `channel_handle` (String)
  - `subscribed_at` (DateTime)

- `messages` - Collected messages
  - `channel_id` (i64)
  - `channel_handle` (String)
  - `message_id` (i32)
  - `text` (String)
  - `date` (DateTime)
  - Indexed on (channel_id, message_id) for deduplication

- `summarize_state` - Summary tracking
  - `user_id` (i64)
  - `last_summarized_at` (DateTime)

## Dependencies

### Main Application
- `grammers-client` (0.8.1) - Telegram MTProto client
- `grammers-session` (0.8.0) - SQLite session storage
- `mongodb` (3.4.1) - Database driver
- `tokio` (1.48.0) - Async runtime with signal handling
- `config` (0.15.19) - Configuration management
- `serde` (1.0.228) - Serialization/deserialization
- `chrono` (0.4.42) - Date/time handling
- `tracing` / `tracing-subscriber` - Structured logging
- `anyhow` (1.0.100) - Error handling

### Repository Crate
- `mongodb` (3.4.1) - Database operations
- `bson` (2.15) - BSON serialization
- `chrono` (0.4.42) - Date/time handling
- `thiserror` (2.0.17) - Error type derivation

## Configuration

Configuration can be provided via:
1. `Settings.toml` file
2. Environment variables (using `__` separator, e.g., `API_ID`, `REPO_CONFIG__CONNECTION_STRING`)

Environment variables override TOML settings.

## Implementation Status

**Fully Implemented**:
- Telegram client initialization and authentication (with 2FA support)
- Session persistence via SQLite
- Real-time message streaming from Telegram
- Channel subscription management (`/subscribe`, `/unsubscribe`, `/list`)
- Message capture and MongoDB storage with deduplication
- Command parsing and routing
- Graceful shutdown on CTRL+C
- Configuration via TOML and environment variables
- Structured logging with tracing

**Partially Implemented**:
- Summarization infrastructure (framework ready, Claude integration is a placeholder)

**Planned**:
- Full Claude API integration for AI-powered summaries
- Message filtering and search capabilities
- Channel statistics
- Advanced subscription management

## License

MIT
