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

### Module Structure
- `src/main.rs`: Application entry point and TUI event loop
- `src/auth.rs`: OAuth authentication flow implementation
- `src/fileio.rs`: Configuration and token file management
- `src/api.rs`: SpareBank 1 API client functions
- `src/models/`: Data models for API responses
  - `accounts.rs`: Account data structures
  - `token.rs`: Token data structures
- `src/ui.rs`: TUI rendering logic

### OAuth Authentication Flow
The application implements a three-tiered OAuth authentication strategy in `src/auth.rs`:

1. **Access Token Check**: First attempts to use stored access token from `auth.json`
2. **Refresh Token Flow**: If access token invalid, attempts to refresh using stored refresh token
   - Makes POST request to `/oauth/token` with `grant_type=refresh_token`
   - Saves new token data to `auth.json` on success
3. **Full OAuth Flow**: If refresh fails, initiates full OAuth flow with SpareBank 1's API:
   - Spawns local HTTP server on port 8321 to receive OAuth callback
   - Opens browser to SpareBank 1's authorization endpoint with `finInst=fid-smn` parameter
   - Waits for authorization code via redirect
   - Exchanges code for access token (not yet implemented - see `get_access_token()` todo)

### File Management (`src/fileio.rs`)
- **Config file location**: `~/.config/auox/config.toml` (on macOS/Linux) or equivalent platform directory
- **Token file location**: `~/.local/share/auox/auth.json` (on Linux) or equivalent platform data directory
- Creates directories automatically if they don't exist
- **Required config fields**:
  - `client_id`: OAuth client ID for SpareBank 1 API
  - `client_secret`: OAuth client secret for SpareBank 1 API
- **Token file structure** (`auth.json`):
  - `access_token`: Current access token
  - `expires_in`: Access token expiry time (seconds)
  - `refresh_token`: Token for refreshing access
  - `refresh_token_expires_in`: Refresh token expiry time
  - `refresh_token_absolute_expires_in`: Absolute refresh token expiry
  - `token_type`: Token type (Bearer)

### TUI Architecture (`src/main.rs`, `src/ui.rs`)
- Built with `ratatui` and `crossterm` for terminal UI
- Main loop pattern: enable raw mode → render loop → cleanup
- Event handling for navigation:
  - `q`: quit application
  - Up/Down arrows: navigate account list with modulo wrap-around
- State management via `ListState` for tracking selected item
- UI rendered in `ui::draw()` with blue highlight style for selected items
- Displays actual bank accounts fetched from SpareBank 1 API

### API Client (`src/api.rs`)
- `get_accounts()`: Fetches account list from `/personal/banking/accounts`
- Uses Bearer token authentication
- Returns `AccountData` with list of accounts and errors

### Data Models (`src/models/`)
Defines SpareBank 1 API response structures:
- `AccountData`: Top-level response with accounts array and errors
- `Account`: Bank account with balance, IBAN, owner info, and account properties
- `AccountProperties`: Detailed flags for account capabilities (transfers, payments, special account types)
- `TokenData`: OAuth token response structure
- All structs use camelCase JSON serialization to match API format

## Key Dependencies
- `ratatui`: TUI framework for terminal interface
- `crossterm`: Cross-platform terminal manipulation
- `tiny_http`: Lightweight HTTP server for OAuth callback
- `reqwest`: HTTP client for API calls (with `blocking` and `json` features enabled)
- `serde`/`serde_json`/`toml`: Serialization and config parsing
- `color-eyre`: Enhanced error reporting
- `dirs`: Platform-agnostic directory paths

## Development Notes

### Current State
The application is functional with core features implemented:
- ✅ OAuth authentication with token refresh
- ✅ Token storage and retrieval from filesystem
- ✅ Account fetching from SpareBank 1 API
- ✅ TUI with account list navigation
- ✅ Modulo-based wrap-around navigation

### Incomplete Features
- `get_access_token()` in `src/auth.rs` - needs implementation to exchange OAuth code for tokens (full OAuth flow completion)
- Account details view - currently only shows list, no detailed view when selecting an account
- Error handling - many functions use `panic!` or `expect()` instead of proper error propagation
- Transaction history - not yet implemented
- Account operations (transfers, payments) - not yet implemented

### API Integration
- Base URL: `https://api.sparebank1.no`
- OAuth endpoint: `/oauth/authorize`
- Financial institution parameter: `finInst=fid-smn` (SpareBank 1 Midt-Norge)
- Redirect URI: `http://localhost:8321`
