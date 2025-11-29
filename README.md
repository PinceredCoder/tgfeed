# tgfeed

A modular, Rust-based Telegram userbot system that aggregates messages from multiple Telegram channels and provides AI-powered summarization capabilities. Built with a modern async architecture using Tokio, it combines both Telegram Bot API (via teloxide) and MTProto client (via grammers) for comprehensive channel monitoring and user interaction.

## Features

- **Channel Subscription Management**: Subscribe to any public Telegram channel by handle with automatic resolution and membership
- **Real-time Message Collection**: Stream and capture messages from all subscribed channels with automatic deduplication
- **Persistent Storage**: MongoDB-based storage for messages, subscriptions, and summarization state
- **Dual Bot Interface**: Interactive Telegram bot commands via teloxide for user-friendly interaction
- **MTProto Client**: Direct Telegram client access via grammers for channel monitoring
- **Session Persistence**: SQLite-based session storage that survives application restarts
- **Two-Factor Authentication**: Full support for phone verification, SMS codes, and 2FA passwords
- **AI Summarization Framework**: Infrastructure ready for Claude API integration (pending)
- **Graceful Shutdown**: Coordinated service termination with proper state synchronization
- **Modular Architecture**: Workspace-based design with separated concerns across dedicated crates
- **Flexible Configuration**: Multi-source configuration via TOML files and environment variables
- **Structured Logging**: Comprehensive tracing integration throughout the application

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
   # Server configuration (for future health checks)
   server_addr = "0.0.0.0:8080"
   healthcheck_addr = "0.0.0.0:8081"

   # Telegram MTProto client configuration
   [monitor_config]
   api_id = 123456                           # Get from https://my.telegram.org/apps
   api_hash = "your_api_hash_here"           # Get from https://my.telegram.org/apps
   session_file = "session.sqlite"           # SQLite session persistence file

   # Telegram Bot configuration
   [bot_config]
   token = "your_bot_token_here"             # Get from @BotFather on Telegram

   # MongoDB configuration
   [repo_config]
   connection_string = "mongodb://127.0.0.1:27017"
   database_name = "tgfeed_db"
   ```

   Alternatively, use environment variables with `__` separator:
   ```bash
   export MONITOR_CONFIG__API_ID=123456
   export MONITOR_CONFIG__API_HASH="your_api_hash_here"
   export BOT_CONFIG__TOKEN="your_bot_token_here"
   export REPO_CONFIG__CONNECTION_STRING="mongodb://127.0.0.1:27017"
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

1. After authentication, the bot will start monitoring for messages
2. Open Telegram and message your bot (using the bot token you configured)
3. Use commands to interact:
   - `/subscribe @bbcnews` - Start following a channel
   - `/list` - View all active subscriptions
   - `/unsubscribe @bbcnews` - Stop following a channel
   - `/summarize` - Request AI summary (currently a placeholder)
4. Messages from subscribed channels are automatically collected and stored in MongoDB
5. Use CTRL+C to gracefully shutdown the application

### Service Communication Flow

```
User sends /subscribe @channel to Bot
         ↓
Bot parses command and sends MonitorCommand::Subscribe
         ↓
Monitor receives command via MPSC channel
         ↓
Monitor resolves channel, subscribes, stores in DB
         ↓
Monitor sends response back via oneshot channel
         ↓
Bot receives response and replies to user
```

## Architecture

The project uses a modular workspace architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                         Main Application                     │
│                    (tgfeed/src/main.rs)                     │
│                                                             │
│  • Service Orchestration                                    │
│  • Configuration Loading                                    │
│  • Graceful Shutdown Coordination                           │
└──────────────────┬──────────────────────┬───────────────────┘
                   │                      │
         ┌─────────▼──────────┐  ┌───────▼────────────┐
         │   tgfeed-bot      │  │  tgfeed-monitor   │
         │   (teloxide)      │  │   (grammers)      │
         │                   │  │                   │
         │ • Bot Commands    │  │ • MTProto Client  │
         │ • User Interface  │  │ • Channel Monitor │
         │ • Command Parsing │  │ • Auth & Session  │
         └────────┬──────────┘  │ • Update Handler  │
                  │             └─────────┬─────────┘
                  │                       │
                  └───────┬───────────────┘
                          │
                ┌─────────▼──────────┐
                │  tgfeed-common    │
                │                   │
                │ • MonitorCommand  │
                │ • Shared Types    │
                └─────────┬─────────┘
                          │
                ┌─────────▼──────────┐
                │   tgfeed-repo     │
                │   (MongoDB)       │
                │                   │
                │ • Subscriptions   │
                │ • Messages        │
                │ • Summarize State │
                └───────────────────┘
```

## Project Structure

This is a Cargo workspace with five specialized crates:

```
tgfeed/
├── Cargo.toml                    # Workspace configuration
├── Settings.toml                 # Active configuration (gitignored)
├── Settings.toml.sample          # Configuration template
├── rustfmt.toml                  # Code formatting rules
│
├── src/                          # Main orchestrator crate
│   ├── main.rs                   # Entry point, service spawning, shutdown
│   └── config.rs                 # Multi-source config loading (TOML + env)
│
├── tgfeed-bot/                   # Telegram Bot interface crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # TgFeedBot struct and run method
│       ├── config.rs             # Bot configuration
│       ├── command.rs            # BotCommands enum definitions
│       └── handler.rs            # Command handlers with user interaction
│
├── tgfeed-common/                # Shared types crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Module exports
│       └── command.rs            # MonitorCommand enum for IPC
│
├── tgfeed-monitor/               # Telegram MTProto client crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # MonitorService orchestration
│       ├── config.rs             # Monitor configuration
│       ├── command.rs            # Command handlers
│       ├── update.rs             # Telegram update processing
│       ├── error.rs              # Error types and conversions
│       └── utils.rs              # Utility functions (prompts)
│
└── tgfeed-repo/                  # Data access layer crate
    ├── Cargo.toml
    └── src/
        ├── lib.rs                # Repo struct and collection accessors
        ├── config.rs             # Database configuration
        ├── models.rs             # Data models (Subscription, Message, State)
        ├── error.rs              # Error types
        ├── subscription.rs       # Subscription CRUD operations
        ├── message.rs            # Message storage and retrieval
        └── summarize.rs          # Summarization state tracking
```

### Crate Responsibilities

| Crate | Purpose | Key Dependencies |
|-------|---------|------------------|
| **tgfeed** (root) | Orchestrates all services, manages configuration and shutdown | tokio, config, anyhow, tracing |
| **tgfeed-bot** | Provides Telegram Bot API interface for user commands | teloxide, tgfeed-common |
| **tgfeed-monitor** | Implements MTProto client for channel monitoring and auth | grammers-client, grammers-session, sqlite |
| **tgfeed-common** | Defines shared types for inter-service communication | serde |
| **tgfeed-repo** | Abstracts MongoDB operations for all data persistence | mongodb, bson, chrono |

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

### Core Technologies
- **Rust**: Edition 2024 with latest async/await features
- **Tokio**: Async runtime (v1.48.0) with signal handling and task spawning
- **MongoDB**: Document database (v3.4.1) for scalable data persistence
- **Telegram APIs**:
  - Teloxide (v0.17.0) - Bot API for user commands
  - Grammers (v0.8.1) - MTProto client for channel monitoring

### By Crate

| Crate | Key Dependencies | Purpose |
|-------|------------------|---------|
| **tgfeed** | tokio, config, anyhow, tracing | Async runtime, config management, logging |
| **tgfeed-bot** | teloxide, tgfeed-common, tokio, thiserror | Bot API integration, command parsing |
| **tgfeed-monitor** | grammers-client, grammers-session, grammers-mtsender, sqlite, tokio | MTProto client, session storage |
| **tgfeed-common** | serde, tokio | Shared types, async channels |
| **tgfeed-repo** | mongodb, bson, chrono, futures, thiserror | Database operations, data models |

### Complete Dependency List
- `tokio` (1.48.0) - Async runtime with macros, RT-multi-thread, signal support
- `teloxide` (0.17.0) - Telegram Bot API framework with macros
- `grammers-client` (0.8.1) - Telegram MTProto client implementation
- `grammers-session` (0.8.0) - Session storage for MTProto
- `grammers-mtsender` (0.8.1) - MTProto message sender
- `mongodb` (3.4.1) - Official MongoDB Rust driver
- `bson` (2.15) - BSON serialization for MongoDB
- `serde` (1.0.228) - Serialization framework
- `chrono` (0.4.42) - Date and time library
- `tracing` (0.1.41) - Structured logging framework
- `tracing-subscriber` (0.3.19) - Tracing log output
- `config` (0.15.19) - Configuration management with TOML/env support
- `thiserror` (2.0.17) - Ergonomic error type derivation
- `anyhow` (1.0.100) - Flexible error handling
- `futures` (0.3.31) - Stream utilities for async operations
- `sqlite` (0.37.0) - SQLite bindings for session persistence
- `cargo-husky` (0.0.3) - Git hooks for development

## Configuration

Configuration can be provided via:
1. `Settings.toml` file
2. Environment variables (using `__` separator, e.g., `API_ID`, `REPO_CONFIG__CONNECTION_STRING`)

Environment variables override TOML settings.

## Implementation Status

### Fully Implemented ✅
- **Telegram MTProto Client**: Full grammers integration with authorization flow
- **Two-Factor Authentication**: Phone verification, SMS codes, and 2FA password support
- **Session Persistence**: SQLite-based session storage that survives restarts
- **Real-time Message Streaming**: Continuous monitoring of subscribed channels
- **Channel Subscription Management**: Subscribe, unsubscribe, and list operations
- **Message Storage**: MongoDB persistence with automatic deduplication via composite indexing
- **Telegram Bot Interface**: teloxide-based command handling with 6 commands
- **Service Orchestration**: Concurrent service spawning with coordinated shutdown
- **Configuration Management**: Multi-source config (TOML + environment variables)
- **Inter-Service Communication**: MPSC channels with request-response pattern using oneshot channels
- **Graceful Shutdown**: CTRL+C signal handling with proper cleanup
- **Structured Logging**: tracing integration throughout the application
- **Error Handling**: Custom error types with thiserror and proper conversion chains

### Partially Implemented 🚧
- **AI Summarization**: Database layer complete, Claude API integration pending
- **Health Check Endpoints**: Configuration present, implementation pending

### Planned Features 📋
- Full Claude API integration for intelligent message summarization
- Message filtering by keywords, time ranges, and content type
- Channel statistics and analytics dashboard
- Advanced subscription management (categories, tags, priorities)
- Web dashboard for monitoring and configuration
- Export functionality (JSON, CSV, PDF reports)
- Message search with full-text indexing
- Rate limiting and quota management
- Multi-user support with permissions

## Development

### Building the Project

```bash
# Build all workspace crates
cargo build

# Build in release mode
cargo build --release

# Build specific crate
cargo build -p tgfeed-bot

# Check without building
cargo check
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p tgfeed-repo

# Run with output
cargo test -- --nocapture
```

### Code Formatting

The project uses `rustfmt` with custom configuration:

```bash
# Format all code
cargo fmt

# Check formatting without applying
cargo fmt -- --check
```

### Workspace Structure Benefits

- **Independent Development**: Each crate can be developed and tested independently
- **Clean Dependencies**: Explicit dependency graph prevents circular dependencies
- **Faster Compilation**: Cargo can parallelize compilation of independent crates
- **Code Reusability**: Shared types in tgfeed-common prevent duplication
- **Clear Boundaries**: Separation of concerns enforced at the crate level

## Troubleshooting

### Common Issues

**"Failed to connect to MongoDB"**
- Ensure MongoDB is running: `mongod` or check your service manager
- Verify connection string in Settings.toml
- Check network connectivity and firewall rules

**"Authentication failed"**
- Verify API credentials from https://my.telegram.org/apps
- Ensure phone number includes country code (e.g., +1234567890)
- Check 2FA password if enabled on your account
- Delete `session.sqlite` to re-authenticate

**"Failed to subscribe to channel"**
- Verify channel handle is correct (e.g., @channelname)
- Ensure channel is public or you have access
- Check Telegram API rate limits

**"Bot not responding to commands"**
- Verify bot token is correct from @BotFather
- Ensure bot service is running alongside monitor service
- Check logs for error messages with `RUST_LOG=debug`

### Logging

Enable detailed logging with environment variables:

```bash
# Debug level for all modules
RUST_LOG=debug cargo run

# Specific crate logging
RUST_LOG=tgfeed_monitor=debug,tgfeed_bot=info cargo run

# Trace level for everything
RUST_LOG=trace cargo run
```

## Performance Considerations

### MongoDB Indexing

The application automatically creates these indexes:
- `messages` collection: Compound index on `(channel_id, message_id)` for deduplication
- `subscriptions` collection: Upsert operations ensure uniqueness

### Memory Usage

- SQLite session file: ~1-5 MB depending on contact list size
- Grammers client: Maintains in-memory state for active connections
- MongoDB connections: Pooled connections managed by the driver

### Scalability

- **Horizontal**: MongoDB supports sharding for large message volumes
- **Vertical**: Async I/O ensures efficient resource usage
- **Concurrent**: Multiple channels monitored simultaneously via async streams

## Security Considerations

### Credential Management

- Never commit `Settings.toml` to version control (included in .gitignore)
- Use environment variables for production deployments
- Rotate bot tokens and API credentials periodically
- Store session files securely with appropriate file permissions

### Database Security

- Use authentication for MongoDB in production
- Configure network access controls
- Enable MongoDB audit logging
- Implement backup strategies for critical data

### Telegram Security

- Session files contain authentication tokens - protect them
- Enable 2FA on your Telegram account
- Monitor for unusual API activity
- Use bot token only for authorized bot operations

## Contributing

Contributions are welcome! Here's how to get started:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes following the existing code style
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Commit with conventional commits style (feat/fix/docs/refactor)
7. Push and create a pull request

### Commit Message Format

Follow conventional commits:
- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `refactor:` Code refactoring
- `test:` Test additions or modifications
- `chore:` Maintenance tasks

## Roadmap

### Phase 1: Core Functionality (Completed)
- ✅ Telegram client integration
- ✅ Channel subscription management
- ✅ Message collection and storage
- ✅ Bot command interface

### Phase 2: AI Integration (In Progress)
- 🚧 Claude API integration
- 📋 Smart message filtering
- 📋 Context-aware summarization
- 📋 Custom summary templates

### Phase 3: Advanced Features (Planned)
- 📋 Web dashboard
- 📋 Multi-user support
- 📋 Analytics and statistics
- 📋 Export functionality
- 📋 Message search

### Phase 4: Enterprise Features (Future)
- 📋 Role-based access control
- 📋 Audit logging
- 📋 High availability setup
- 📋 API for external integrations

## License

MIT
