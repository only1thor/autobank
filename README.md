# Autobank

Rule-based banking automation for SpareBank 1. Monitor transactions and automatically execute transfers based on customizable rules.

## Features

- **Rule Engine** - Create rules that trigger transfers based on transaction patterns
- **Condition Matching** - Match by description (regex), amount ranges, transaction type
- **Automatic Transfers** - Execute transfers between accounts when conditions match
- **Web Dashboard** - Modern SvelteKit frontend for managing rules and viewing history
- **Audit Trail** - Complete logging of all rule evaluations and transfers
- **Transaction Deduplication** - Smart fingerprinting to avoid duplicate actions

## Architecture

```
autobank/
├── crates/
│   ├── sb1-api/           # SpareBank 1 API client library
│   └── autobank-server/   # Axum REST API + rule engine + scheduler
├── web/                   # SvelteKit frontend
└── flake.nix              # Nix development environment
```

## Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled (recommended)
- Or: Rust, Node.js 22+, pnpm, SQLite
- SpareBank 1 API credentials ([developer portal](https://developer.sparebank1.no/#/))

## Quick Start

```bash
# Enter development environment
nix develop

# Start the backend server (creates database automatically)
cargo run -p autobank-server

# In another terminal, start the frontend
cd web && pnpm dev

# Open http://localhost:5173
```

## Configuration

Create a config file at:
- **macOS**: `~/Library/Application Support/autobank/config.toml`
- **Linux**: `~/.config/autobank/config.toml`
- **Windows**: `%APPDATA%\autobank\config.toml`

```toml
client_id = "your-client-id"
client_secret = "your-client-secret"
financial_institution = "fid-smn"  # e.g., fid-smn, fid-snn
```

## Example Rules

### Auto-cover Netflix subscription
When Netflix charges your checking account, automatically transfer the amount from savings:

```json
{
  "name": "Netflix auto-transfer",
  "trigger_account_key": "checking-account-key",
  "conditions": [
    { "type": "description_matches", "pattern": "netflix", "case_insensitive": true },
    { "type": "is_settled" }
  ],
  "actions": [{
    "type": "transfer",
    "from_account": { "type": "by_key", "key": "savings-account-key" },
    "to_account": { "type": "trigger_account" },
    "amount": { "type": "transaction_amount_abs" }
  }]
}
```

### Save on small purchases
Transfer a fixed amount to savings for every small purchase:

```json
{
  "name": "Small purchase savings",
  "trigger_account_key": "checking-account-key",
  "conditions": [
    { "type": "amount_less_than", "value": 0 },
    { "type": "amount_greater_than", "value": -100 },
    { "type": "is_settled" }
  ],
  "actions": [{
    "type": "transfer",
    "from_account": { "type": "trigger_account" },
    "to_account": { "type": "by_key", "key": "savings-account-key" },
    "amount": { "type": "fixed", "value": 20 }
  }]
}
```

## API Endpoints

### Accounts
- `GET /api/accounts` - List all accounts
- `GET /api/accounts/:key` - Get single account
- `GET /api/accounts/:key/transactions` - Get transactions

### Rules
- `GET /api/rules` - List all rules
- `POST /api/rules` - Create rule
- `GET /api/rules/:id` - Get rule details
- `PUT /api/rules/:id` - Update rule
- `DELETE /api/rules/:id` - Delete rule
- `POST /api/rules/:id/enable` - Enable rule
- `POST /api/rules/:id/disable` - Disable rule

### Executions & Audit
- `GET /api/executions` - List recent executions
- `GET /api/audit` - Query audit log

### System
- `GET /api/health` - Health check
- `GET /api/system/status` - Server status and stats
- `POST /api/system/poll` - Trigger immediate poll
- `POST /api/system/scheduler/enable` - Enable scheduler
- `POST /api/system/scheduler/disable` - Disable scheduler

## Development

```bash
# Run all tests
cargo test

# Run backend with auto-reload
just dev

# Run frontend dev server
just web

# Check code
cargo clippy
cargo fmt --check

# Build for production
cargo build --release
cd web && pnpm build
```

## Tech Stack

**Backend:**
- Rust with Axum web framework
- SQLite with sqlx for persistence
- Tokio async runtime

**Frontend:**
- SvelteKit 2 with Svelte 5
- Tailwind CSS v4
- TypeScript

**Development:**
- Nix flakes for reproducible environment
- Just task runner

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Disclaimer

This is an unofficial application and is not affiliated with or endorsed by SpareBank 1. Use at your own risk. Automated transfers can move real money - test thoroughly and monitor executions.
