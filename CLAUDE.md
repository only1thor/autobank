# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Auox (Aurum Oxydatum) is a terminal-based banking application written in Rust that interfaces with SpareBank 1's API. It provides a TUI (Terminal User Interface) for viewing and managing bank accounts using OAuth authentication.

## Commands

### Build and Run
```bash
cargo build          # Build the project
cargo run            # Run the application
cargo check          # Quick type checking without building
cargo clippy         # Run linter
```

### Testing
```bash
cargo test           # Run all tests
cargo test <name>    # Run a specific test
```

## Architecture

### OAuth Authentication Flow
The application implements a three-tiered OAuth authentication strategy in `src/auth.rs`:

1. **Access Token Check**: First attempts to use stored access token (`read_access_token()`)
2. **Refresh Token Flow**: If access token invalid, attempts to refresh using stored refresh token (not yet implemented)
3. **Full OAuth Flow**: If refresh fails, initiates full OAuth flow with SpareBank 1's API:
   - Spawns local HTTP server on port 8321 to receive OAuth callback
   - Opens browser to SpareBank 1's authorization endpoint with `finInst=fid-smn` parameter
   - Waits for authorization code via redirect
   - Exchanges code for access token (not yet implemented - see `get_access_token()` todo)

### Configuration Management (`src/config.rs`)
- Config file location: `~/.config/auox/config.toml` (on macOS/Linux) or equivalent platform directory
- Creates config directory automatically if it doesn't exist
- Required config fields:
  - `client_id`: OAuth client ID for SpareBank 1 API
  - `refresh_token`: Stored refresh token for re-authentication
- Access token storage mechanism not yet implemented (see `read_access_token()`)

### TUI Architecture (`src/main.rs`, `src/ui.rs`)
- Built with `ratatui` and `crossterm` for terminal UI
- Main loop pattern: enable raw mode → render loop → cleanup
- Event handling for navigation:
  - `q`: quit application
  - Up/Down arrows: navigate list (wraps around)
- State management via `ListState` for tracking selected item
- UI rendered in `ui::draw()` with blue highlight style for selected items

### API Data Models (`src/accounts.rs`)
Defines SpareBank 1 API response structures:
- `AccountResponse`: Top-level response with accounts array and errors
- `Account`: Bank account with balance, IBAN, owner info, and account properties
- `AccountProperties`: Detailed flags for account capabilities (transfers, payments, special account types)
- All structs use camelCase JSON serialization to match API format

## Key Dependencies
- `ratatui`: TUI framework for terminal interface
- `crossterm`: Cross-platform terminal manipulation
- `tiny_http`: Lightweight HTTP server for OAuth callback
- `reqwest`: HTTP client for API calls
- `serde`/`serde_json`/`toml`: Serialization and config parsing
- `color-eyre`: Enhanced error reporting

## Development Notes

### Current State
The application is in early development. Recent commits show work on auth flow and token management. The main TUI renders a placeholder list of names rather than actual bank accounts.

### Incomplete Features
- `get_access_token()` in `src/auth.rs:72` - needs implementation to exchange OAuth code for tokens
- `refresh_access_token()` in `src/auth.rs:76` - needs implementation for token refresh
- `test_token()` in `src/auth.rs:80` - currently returns true, needs actual token validation
- `read_access_token()` in `src/config.rs:55` - returns hardcoded string, needs proper token storage/retrieval
- Account data not yet integrated into UI (placeholder names used in main.rs:27)

### API Integration
- Base URL: `https://api.sparebank1.no`
- OAuth endpoint: `/oauth/authorize`
- Financial institution parameter: `finInst=fid-smn` (SpareBank 1 Midt-Norge)
- Redirect URI: `http://localhost:8321`
