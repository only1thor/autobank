# Autobank: Rule-Based Banking Automation

## Current Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | sb1-api crate | **COMPLETE** |
| Phase 2 | autobank-server | **COMPLETE** |
| Phase 3 | SvelteKit frontend | **NEXT** |
| Phase 4 | Nix flake (already done) | **COMPLETE** |

### Recent Commits
```
107fded feat: add autobank-server with rule engine and REST API
b3093b9 refactor: extract sb1-api crate with async trait-based client
44c004d add plan to transform to rule engine
```

### Test Status
- **26 tests passing** (5 condition, 3 mock, 6 client, 8 model, 4 integration)

### Quick Start for Development
```bash
nix develop                    # Enter dev environment
cargo test                     # Run all tests
cargo run -p autobank-server   # Run server (needs config)
```

### Next Steps (Phase 3)
1. Initialize SvelteKit project in `web/` directory
2. Set up Tailwind CSS v4
3. Create API client
4. Build dashboard and account views
5. Build rule builder UI
6. Add execution history and audit views

---

## Project Overview

Transform the existing TUI banking app (Auox) into a **Rust backend** with a **SvelteKit web frontend** that enables rule-based automation of bank transfers based on transaction patterns from SpareBank 1's API.

### Key Decisions
- **Backend**: Rust (reuse existing API code, type safety, performance)
- **Frontend**: SvelteKit (simple, fast, modern)
- **Database**: SQLite (embedded, no separate server)
- **Triggers**: Polling interval (configurable)
- **API Style**: REST
- **User Model**: Single-user deployment
- **Original TUI**: Replaced entirely

---

## Architecture

```
autobank/
├── flake.nix                    # Nix flake for development
├── flake.lock
├── AGENTS.md                    # Documentation for AI assistants
├── Cargo.toml                   # Workspace root
├── crates/
│   ├── sb1-api/                 # Core API client library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs        # HTTP client with trait-based design
│   │       ├── auth.rs          # OAuth flow
│   │       ├── models/          # Data models (from existing)
│   │       └── error.rs         # Proper error types
│   └── autobank-server/         # Backend server
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs          # Axum server entry
│           ├── api/             # REST endpoints
│           ├── rules/           # Rule engine
│           ├── scheduler/       # Polling scheduler
│           ├── db/              # SQLite persistence
│           └── audit/           # Audit trail logging
└── web/                         # SvelteKit frontend
    ├── package.json
    ├── svelte.config.js
    └── src/
        ├── routes/
        └── lib/
```

---

## Phase 1: Extract and Refactor API Library (`sb1-api` crate)

### Goals
- Strip TUI code, keep only API-related functionality
- Make the API client testable via traits
- Add proper error handling (replace panics with `Result` types)
- Write comprehensive mocks and tests

### Tasks

#### 1.1 Create Cargo workspace and Nix flake
- Initialize `flake.nix` with rust toolchain, sqlite, nodejs, pnpm
- Set up `Cargo.toml` as workspace root
- Create `crates/sb1-api/` directory structure

#### 1.2 Extract and refactor API code
- Move `src/models/*.rs` → `crates/sb1-api/src/models/`
- Refactor `src/api.rs` → `crates/sb1-api/src/client.rs`
- Refactor `src/auth.rs` → `crates/sb1-api/src/auth.rs`
- Refactor `src/fileio.rs` → `crates/sb1-api/src/config.rs`
- Delete TUI-specific files: `src/main.rs`, `src/ui.rs`

#### 1.3 Make client trait-based for testability
```rust
// crates/sb1-api/src/client.rs
#[async_trait]
pub trait BankApiClient: Send + Sync {
    async fn get_accounts(&self) -> Result<AccountData, ApiError>;
    async fn get_transactions(&self, account_key: &str) -> Result<TransactionResponse, ApiError>;
    async fn create_transfer(&self, transfer: CreateTransferDTO) -> Result<TransferResponse, ApiError>;
    async fn create_credit_card_transfer(&self, transfer: TransferToCreditCardDTO) -> Result<TransferResponse, ApiError>;
}

pub struct SpareBank1Client {
    http_client: reqwest::Client,
    base_url: String,
    token_provider: Arc<dyn TokenProvider>,
}

impl BankApiClient for SpareBank1Client { /* real implementation */ }
```

#### 1.4 Convert to async
- Replace `reqwest::blocking` with async `reqwest`
- Use `tokio` runtime

#### 1.5 Proper error handling
```rust
// crates/sb1-api/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("API error: {code} - {message}")]
    Api { code: String, message: String, trace_id: String },
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Config error: {0}")]
    Config(String),
}
```

#### 1.6 Create mock client for testing
```rust
// crates/sb1-api/src/mock.rs
pub struct MockBankClient {
    pub accounts: Vec<Account>,
    pub transactions: HashMap<String, Vec<Transaction>>,
    pub transfer_results: VecDeque<Result<TransferResponse, ApiError>>,
}

impl BankApiClient for MockBankClient { /* return preset data */ }
```

#### 1.7 Write unit tests
- Model serialization/deserialization tests with sample JSON
- Client method tests with mock HTTP responses (using `wiremock`)
- Auth flow tests

### Dependencies for `sb1-api`
```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
async-trait = "0.1"
tokio = { version = "1", features = ["full"] }
url = "2.5"
urlencoding = "2.1"
tracing = "0.1"

[dev-dependencies]
wiremock = "0.6"
tokio-test = "0.4"
```

### Deliverables
- [x] Working `sb1-api` crate that compiles
- [x] All existing API functionality preserved
- [x] Unit tests passing
- [x] Mock client available for testing

**Status: COMPLETE** - Committed as `refactor: extract sb1-api crate with async trait-based client`

---

## Phase 2: Build the Backend Server (`autobank-server` crate)

### Goals
- REST API server using Axum
- Rule engine for transaction-based automation
- SQLite persistence for rules, execution history, and audit logs
- Polling scheduler for periodic transaction checks
- Robust transaction deduplication

### 2.1 Database Schema

```sql
-- Rules table
CREATE TABLE rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    trigger_account_key TEXT NOT NULL,
    conditions TEXT NOT NULL,      -- JSON: array of conditions
    actions TEXT NOT NULL,         -- JSON: array of actions
    created_at INTEGER NOT NULL,   -- Unix timestamp
    updated_at INTEGER NOT NULL
);

-- Transaction tracking for deduplication
-- Stores a fingerprint of transaction state to detect updates
CREATE TABLE tracked_transactions (
    id TEXT PRIMARY KEY,                    -- Transaction ID from API
    account_key TEXT NOT NULL,
    fingerprint TEXT NOT NULL,              -- Hash of mutable fields
    first_seen_at INTEGER NOT NULL,
    last_updated_at INTEGER NOT NULL,
    settled INTEGER NOT NULL DEFAULT 0,     -- Whether transaction is final
    raw_data TEXT NOT NULL                  -- Full transaction JSON for debugging
);

-- Rule-transaction processing record
-- Tracks which rules have processed which transaction versions
CREATE TABLE rule_transaction_log (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES rules(id),
    transaction_id TEXT NOT NULL,
    transaction_fingerprint TEXT NOT NULL,  -- Fingerprint at time of processing
    action_taken TEXT NOT NULL,             -- 'executed', 'skipped', 'error'
    processed_at INTEGER NOT NULL,
    UNIQUE(rule_id, transaction_id, transaction_fingerprint)
);

-- Rule execution history (successful transfers)
CREATE TABLE rule_executions (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES rules(id),
    transaction_id TEXT NOT NULL,
    transfer_payment_id TEXT,               -- Payment ID from bank API
    amount REAL NOT NULL,
    from_account TEXT NOT NULL,
    to_account TEXT NOT NULL,
    status TEXT NOT NULL,                   -- 'success', 'failed'
    error_message TEXT,
    executed_at INTEGER NOT NULL
);

-- Audit log (comprehensive system audit trail)
CREATE TABLE audit_log (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    event_type TEXT NOT NULL,               -- See AuditEventType enum
    actor TEXT NOT NULL,                    -- 'system', 'user', 'scheduler'
    resource_type TEXT,                     -- 'rule', 'transfer', 'auth', etc.
    resource_id TEXT,
    details TEXT NOT NULL,                  -- JSON with event-specific details
    ip_address TEXT,
    user_agent TEXT
);

-- Indexes for common queries
CREATE INDEX idx_tracked_transactions_account ON tracked_transactions(account_key);
CREATE INDEX idx_tracked_transactions_settled ON tracked_transactions(settled);
CREATE INDEX idx_rule_transaction_log_rule ON rule_transaction_log(rule_id);
CREATE INDEX idx_rule_executions_rule ON rule_executions(rule_id);
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX idx_audit_log_event_type ON audit_log(event_type);
```

### 2.2 Transaction Deduplication Strategy

Transactions from the API can change over time (description updates, status changes). We need to:

1. **Track transaction state changes**: Store a fingerprint (hash) of mutable fields
2. **Detect meaningful changes**: Only re-evaluate rules if relevant fields changed
3. **Prevent duplicate actions**: Track which rule+transaction+fingerprint combinations have been processed
4. **Handle settlement**: Once a transaction is "settled" (final), stop monitoring for changes

```rust
// Transaction fingerprinting
pub struct TransactionFingerprint {
    pub transaction_id: String,
    pub fingerprint: String,  // SHA256 of: description + amount + type_code + booking_status
}

impl TransactionFingerprint {
    pub fn from_transaction(tx: &Transaction) -> Self {
        let content = format!(
            "{}|{}|{}|{}|{}",
            tx.id,
            tx.cleaned_description.as_deref().unwrap_or(""),
            tx.amount,
            tx.type_code,
            tx.booking_status
        );
        Self {
            transaction_id: tx.id.clone(),
            fingerprint: sha256_hex(&content),
        }
    }
}

// Deduplication check
pub enum ProcessingDecision {
    Process,                    // New transaction or meaningful change
    Skip { reason: String },    // Already processed this version
    Wait { reason: String },    // Transaction not settled, wait for updates
}
```

**Fields considered "mutable" (part of fingerprint):**
- `cleaned_description` - Can be updated by bank's systems
- `amount` - Should be stable but include for safety
- `type_code` - Classification may change
- `booking_status` - Important: indicates if transaction is pending/booked

**Settlement detection:**
- `booking_status == "BOOKED"` indicates a settled transaction
- Settled transactions won't be re-evaluated for changes

### 2.3 Rule DSL Design

```rust
// Rule condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    // Match transaction description (merchant name, etc.)
    DescriptionMatches { 
        pattern: String,  // Regex pattern
        case_insensitive: bool,
    },
    
    // Amount comparisons
    AmountGreaterThan { value: f64 },
    AmountLessThan { value: f64 },
    AmountBetween { min: f64, max: f64 },
    AmountEquals { value: f64, tolerance: f64 },  // With floating point tolerance
    
    // Transaction type
    TransactionType { type_code: String },
    
    // Only trigger on settled transactions
    IsSettled,
    
    // Logical operators
    And { conditions: Vec<Condition> },
    Or { conditions: Vec<Condition> },
    Not { condition: Box<Condition> },
}

// Rule action types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Transfer {
        from_account: AccountRef,
        to_account: AccountRef,
        amount: AmountSpec,
        message: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountRef {
    ByKey { key: String },
    ByNumber { number: String },
    TriggerAccount,  // The account being monitored
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AmountSpec {
    Fixed { value: f64 },
    TransactionAmount,
    TransactionAmountAbs,  // Absolute value (for negative transactions)
    Percentage { of_transaction: f64 },
    Min { specs: Vec<AmountSpec> },  // Minimum of multiple specs
    Max { specs: Vec<AmountSpec> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub trigger_account_key: String,
    pub conditions: Vec<Condition>,  // AND'd together
    pub actions: Vec<Action>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### 2.4 Audit Trail System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Authentication events
    AuthStarted,
    AuthCompleted,
    AuthFailed,
    TokenRefreshed,
    
    // Rule management
    RuleCreated,
    RuleUpdated,
    RuleDeleted,
    RuleEnabled,
    RuleDisabled,
    
    // Rule execution
    RuleEvaluated,        // Rule was checked against a transaction
    RuleMatched,          // Rule conditions matched
    RuleSkipped,          // Transaction already processed
    TransferInitiated,    // Transfer API call made
    TransferSucceeded,    // Transfer completed
    TransferFailed,       // Transfer failed
    
    // Scheduler events
    SchedulerStarted,
    SchedulerStopped,
    PollStarted,
    PollCompleted,
    PollFailed,
    
    // System events
    ServerStarted,
    ServerStopped,
    ConfigChanged,
    DatabaseMigrated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: i64,
    pub event_type: AuditEventType,
    pub actor: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

// Audit service trait for testability
#[async_trait]
pub trait AuditService: Send + Sync {
    async fn log(&self, event: AuditEntry) -> Result<(), DbError>;
    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>, DbError>;
}
```

### 2.5 REST API Endpoints

```
Authentication:
  POST   /api/auth/init              # Start OAuth flow, returns auth URL
  GET    /api/auth/callback          # OAuth callback handler
  GET    /api/auth/status            # Check auth status
  POST   /api/auth/refresh           # Force token refresh

Accounts:
  GET    /api/accounts               # List all accounts
  GET    /api/accounts/:key          # Get single account
  GET    /api/accounts/:key/transactions  # Get transactions (with pagination)

Rules:
  GET    /api/rules                  # List all rules
  POST   /api/rules                  # Create rule
  GET    /api/rules/:id              # Get rule details
  PUT    /api/rules/:id              # Update rule
  DELETE /api/rules/:id              # Delete rule
  POST   /api/rules/:id/enable       # Enable rule
  POST   /api/rules/:id/disable      # Disable rule

Rule Executions:
  GET    /api/executions             # List recent executions (all rules)
  GET    /api/rules/:id/executions   # Executions for specific rule

Audit:
  GET    /api/audit                  # Query audit log (with filters)
  GET    /api/audit/:id              # Get specific audit entry

System:
  GET    /api/status                 # Server status, last poll time, stats
  POST   /api/poll                   # Trigger immediate poll
  GET    /api/config                 # Get current configuration
  PUT    /api/config                 # Update configuration
```

### 2.6 Scheduler Design

```rust
pub struct SchedulerConfig {
    pub poll_interval_seconds: u64,  // Default: 300 (5 minutes)
    pub enabled: bool,
}

pub struct Scheduler {
    config: Arc<RwLock<SchedulerConfig>>,
    api_client: Arc<dyn BankApiClient>,
    db: SqlitePool,
    audit: Arc<dyn AuditService>,
}

impl Scheduler {
    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) {
        loop {
            tokio::select! {
                _ = shutdown.recv() => break,
                _ = tokio::time::sleep(self.get_interval()) => {
                    if self.is_enabled().await {
                        self.poll_and_evaluate().await;
                    }
                }
            }
        }
    }
    
    async fn poll_and_evaluate(&self) {
        self.audit.log(PollStarted).await;
        
        // 1. Get all enabled rules grouped by trigger_account
        let rules_by_account = self.get_enabled_rules_grouped().await?;
        
        for (account_key, rules) in rules_by_account {
            // 2. Fetch recent transactions for this account
            let transactions = self.api_client.get_transactions(&account_key).await?;
            
            for tx in transactions {
                // 3. Compute fingerprint and check for changes
                let fingerprint = TransactionFingerprint::from_transaction(&tx);
                let decision = self.check_processing_decision(&tx, &fingerprint).await;
                
                match decision {
                    ProcessingDecision::Skip { reason } => continue,
                    ProcessingDecision::Wait { reason } => continue,
                    ProcessingDecision::Process => {
                        // 4. Update tracked transaction state
                        self.update_tracked_transaction(&tx, &fingerprint).await?;
                        
                        // 5. Evaluate each rule
                        for rule in &rules {
                            self.evaluate_and_execute(rule, &tx, &fingerprint).await?;
                        }
                    }
                }
            }
        }
        
        self.audit.log(PollCompleted).await;
    }
    
    async fn evaluate_and_execute(
        &self, 
        rule: &Rule, 
        tx: &Transaction, 
        fingerprint: &TransactionFingerprint
    ) -> Result<()> {
        // Check if already processed this rule+tx+fingerprint
        if self.already_processed(rule, fingerprint).await? {
            return Ok(());
        }
        
        self.audit.log(RuleEvaluated { rule_id: &rule.id, tx_id: &tx.id }).await;
        
        // Evaluate conditions
        if !rule.evaluate_conditions(tx) {
            self.record_processing(rule, fingerprint, "skipped").await?;
            return Ok(());
        }
        
        self.audit.log(RuleMatched { rule_id: &rule.id, tx_id: &tx.id }).await;
        
        // Execute actions
        for action in &rule.actions {
            self.execute_action(rule, tx, action, fingerprint).await?;
        }
        
        Ok(())
    }
}
```

### Dependencies for `autobank-server`
```toml
[dependencies]
sb1-api = { path = "../sb1-api" }
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
regex = "1.10"
sha2 = "0.10"
hex = "0.4"

[dev-dependencies]
axum-test = "16"
```

### Deliverables
- [x] Axum server with all REST endpoints
- [x] SQLite database with migrations
- [x] Rule engine with condition evaluation
- [x] Transaction deduplication system
- [x] Audit trail logging
- [x] Polling scheduler
- [x] Integration tests

**Status: COMPLETE** - Committed as `feat: add autobank-server with rule engine and REST API`

### Implemented API Endpoints
```
Health:
  GET    /api/health               # Health check
  GET    /api/status               # Basic status

Accounts:
  GET    /api/accounts             # List all accounts
  GET    /api/accounts/:key        # Get single account
  GET    /api/accounts/:key/transactions  # Get transactions

Rules:
  GET    /api/rules                # List all rules
  POST   /api/rules                # Create rule
  GET    /api/rules/:id            # Get rule details
  PUT    /api/rules/:id            # Update rule
  DELETE /api/rules/:id            # Delete rule
  POST   /api/rules/:id/enable     # Enable rule
  POST   /api/rules/:id/disable    # Disable rule
  GET    /api/rules/:id/executions # Executions for specific rule

Executions:
  GET    /api/executions           # List recent executions (all rules)
  GET    /api/executions/:id       # Get specific execution

Audit:
  GET    /api/audit                # Query audit log
  GET    /api/audit/:id            # Get specific audit entry

System:
  GET    /api/system/status        # Server status with stats
  POST   /api/system/poll          # Trigger immediate poll
  POST   /api/system/scheduler/enable   # Enable scheduler
  POST   /api/system/scheduler/disable  # Disable scheduler
```

---

## Phase 3: SvelteKit Web Frontend (NEXT)

### Goals
- Clean, responsive UI for managing rules
- Account overview with balances
- Transaction history viewer
- Rule builder with visual condition/action editors
- Execution history and audit logs

### 3.1 Pages Structure

```
/                           # Dashboard: account overview, recent executions
/accounts                   # Account list with balances
/accounts/[key]             # Account details + transactions
/rules                      # Rule list with enable/disable toggles
/rules/new                  # Create new rule wizard
/rules/[id]                 # Edit rule
/rules/[id]/executions      # Rule execution history
/audit                      # Audit log viewer with filters
/settings                   # Polling interval, auth management
```

### 3.2 Key Components

```
src/lib/components/
├── accounts/
│   ├── AccountCard.svelte       # Display account with balance
│   ├── AccountList.svelte       # Grid/list of accounts
│   └── AccountSelect.svelte     # Dropdown for selecting account
├── transactions/
│   ├── TransactionTable.svelte  # Filterable transaction list
│   ├── TransactionRow.svelte    # Single transaction display
│   └── TransactionFilters.svelte
├── rules/
│   ├── RuleCard.svelte          # Rule summary with toggle
│   ├── RuleList.svelte          # List of rules
│   ├── RuleBuilder.svelte       # Full rule editor
│   ├── ConditionEditor.svelte   # Add/edit conditions
│   ├── ConditionDisplay.svelte  # Read-only condition view
│   ├── ActionEditor.svelte      # Configure transfer actions
│   └── AmountSpecEditor.svelte  # Amount specification input
├── executions/
│   ├── ExecutionList.svelte     # Execution history
│   └── ExecutionDetail.svelte   # Single execution details
├── audit/
│   ├── AuditLog.svelte          # Audit log table
│   ├── AuditFilters.svelte      # Event type, date filters
│   └── AuditEntry.svelte        # Single audit entry
└── common/
    ├── StatusBadge.svelte       # Success/error/pending badges
    ├── TimeAgo.svelte           # Relative time display
    ├── JsonViewer.svelte        # Collapsible JSON display
    └── ConfirmDialog.svelte     # Confirmation modal
```

### 3.3 API Client

```typescript
// src/lib/api/client.ts
class AutobankClient {
  constructor(private baseUrl: string) {}
  
  // Accounts
  async getAccounts(): Promise<Account[]>
  async getAccount(key: string): Promise<Account>
  async getTransactions(accountKey: string, params?: TransactionParams): Promise<Transaction[]>
  
  // Rules
  async getRules(): Promise<Rule[]>
  async getRule(id: string): Promise<Rule>
  async createRule(rule: CreateRuleDTO): Promise<Rule>
  async updateRule(id: string, rule: UpdateRuleDTO): Promise<Rule>
  async deleteRule(id: string): Promise<void>
  async enableRule(id: string): Promise<void>
  async disableRule(id: string): Promise<void>
  
  // Executions
  async getExecutions(params?: ExecutionParams): Promise<Execution[]>
  async getRuleExecutions(ruleId: string): Promise<Execution[]>
  
  // Audit
  async getAuditLog(params?: AuditParams): Promise<AuditEntry[]>
  
  // System
  async getStatus(): Promise<SystemStatus>
  async triggerPoll(): Promise<void>
  async getConfig(): Promise<Config>
  async updateConfig(config: Partial<Config>): Promise<Config>
}
```

### 3.4 Tech Stack

```json
{
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0",
    "@sveltejs/kit": "^2.0",
    "svelte": "^5.0",
    "@tailwindcss/vite": "^4.0",
    "tailwindcss": "^4.0",
    "typescript": "^5.0",
    "vite": "^6.0"
  },
  "dependencies": {
    "bits-ui": "^1.0",
    "lucide-svelte": "^0.400"
  }
}
```

### Deliverables
- [ ] SvelteKit project with routing
- [ ] Dashboard with account overview
- [ ] Rule CRUD with visual builder
- [ ] Transaction viewer
- [ ] Execution history
- [ ] Audit log viewer
- [ ] Settings page
- [ ] Responsive design

---

## Phase 4: Nix Flake Setup

### `flake.nix`

```nix
{
  description = "Autobank - Rule-based banking automation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            cargo-watch
            cargo-nextest
            
            # Database
            sqlite
            sqlx-cli
            
            # Node.js for frontend
            nodejs_22
            pnpm
            
            # Development tools
            just
            openssl
            pkg-config
            
            # Optional: for testing
            httpie
            jq
          ];
          
          env = {
            DATABASE_URL = "sqlite:./autobank.db";
            RUST_LOG = "info,autobank=debug";
          };
          
          shellHook = ''
            echo "Autobank development environment"
            echo "Commands:"
            echo "  just dev      - Run backend in dev mode"
            echo "  just web      - Run frontend in dev mode"
            echo "  just test     - Run all tests"
            echo "  just migrate  - Run database migrations"
          '';
        };
        
        packages = {
          default = self.packages.${system}.autobank-server;
          
          autobank-server = pkgs.rustPlatform.buildRustPackage {
            pname = "autobank-server";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl pkgs.sqlite ];
          };
        };
      }
    );
}
```

### `justfile` (Task Runner)

```just
# Development commands

# Run backend in development mode with auto-reload
dev:
    cargo watch -x 'run -p autobank-server'

# Run frontend development server
web:
    cd web && pnpm dev

# Run all tests
test:
    cargo nextest run

# Run database migrations
migrate:
    sqlx migrate run --source crates/autobank-server/migrations

# Create a new migration
migrate-new name:
    sqlx migrate add -r {{name}} --source crates/autobank-server/migrations

# Check code formatting and lints
check:
    cargo fmt --check
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Build release binaries
build:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean
    cd web && rm -rf .svelte-kit node_modules
```

---

## Implementation Order & Commits

### Phase 1: API Library (~6 commits)
1. `chore: initialize nix flake and cargo workspace`
   - Create `flake.nix` with all dependencies
   - Set up `Cargo.toml` workspace
   - Add `justfile`
   - Create directory structure

2. `refactor: extract sb1-api crate with models`
   - Move models to `crates/sb1-api/src/models/`
   - Set up crate with `lib.rs`
   - Remove TUI files

3. `refactor: convert api client to async with traits`
   - Create `BankApiClient` trait
   - Implement `SpareBank1Client`
   - Update auth module

4. `feat: add proper error handling with thiserror`
   - Create `ApiError` enum
   - Replace all panics with `Result` returns
   - Add context to errors

5. `test: add mock client and unit tests`
   - Create `MockBankClient`
   - Add model serialization tests
   - Add client tests with `wiremock`

6. `docs: move CLAUDE.md to AGENTS.md and update`
   - Rename and update documentation
   - Document new architecture

### Phase 2: Backend Server (~8 commits)
1. `feat: initialize autobank-server with axum`
   - Create crate structure
   - Set up Axum router
   - Add basic health endpoint

2. `feat: add sqlite database with sqlx migrations`
   - Create migration files
   - Set up connection pool
   - Add database module

3. `feat: implement rule model and CRUD endpoints`
   - Define rule types (conditions, actions)
   - Create rule repository
   - Add REST endpoints

4. `feat: implement transaction tracking for deduplication`
   - Add fingerprint computation
   - Create tracked_transactions table
   - Implement processing decision logic

5. `feat: implement rule engine with condition evaluation`
   - Add condition evaluator
   - Add action executor
   - Wire up to scheduler

6. `feat: add polling scheduler`
   - Create scheduler module
   - Add configuration
   - Implement poll loop

7. `feat: add audit trail system`
   - Create audit types
   - Add audit repository
   - Integrate throughout codebase

8. `test: add integration tests for rule engine`
   - Test rule evaluation
   - Test deduplication
   - Test transfer execution

### Phase 3: Frontend (~7 commits)
1. `feat: initialize sveltekit project with tailwind`
   - Create SvelteKit app
   - Configure Tailwind
   - Set up API client

2. `feat: add dashboard and account views`
   - Create account components
   - Add dashboard page
   - Add account detail page

3. `feat: add transaction viewer`
   - Create transaction table
   - Add filtering
   - Add pagination

4. `feat: add rule list and CRUD`
   - Create rule components
   - Add list page
   - Add create/edit pages

5. `feat: implement visual rule builder`
   - Create condition editor
   - Create action editor
   - Add validation

6. `feat: add execution history and audit views`
   - Create execution list
   - Create audit log viewer
   - Add filters

7. `feat: add settings page and polish UI`
   - Create settings page
   - Add responsive design
   - Polish styling

---

## Example Rule Configurations

### Rule 1: "When Netflix charges, transfer from savings"
```json
{
  "name": "Netflix auto-transfer",
  "description": "Automatically cover Netflix subscription from savings",
  "trigger_account_key": "checking-account-key",
  "conditions": [
    { 
      "type": "description_matches", 
      "pattern": "netflix",
      "case_insensitive": true
    },
    { "type": "is_settled" }
  ],
  "actions": [
    {
      "type": "transfer",
      "from_account": { "type": "by_key", "key": "savings-account-key" },
      "to_account": { "type": "trigger_account" },
      "amount": { "type": "transaction_amount_abs" },
      "message": "Netflix refund"
    }
  ]
}
```

### Rule 2: "Small purchases trigger savings top-up"
```json
{
  "name": "Small purchase savings",
  "description": "Save 20 NOK for every small purchase",
  "trigger_account_key": "checking-account-key",
  "conditions": [
    { "type": "amount_less_than", "value": 0 },
    { "type": "amount_greater_than", "value": -100 },
    { 
      "type": "not",
      "condition": { "type": "amount_greater_than", "value": -50 }
    },
    { "type": "is_settled" }
  ],
  "actions": [
    {
      "type": "transfer",
      "from_account": { "type": "trigger_account" },
      "to_account": { "type": "by_key", "key": "savings-account-key" },
      "amount": { "type": "fixed", "value": 20 },
      "message": "Auto-save"
    }
  ]
}
```

### Rule 3: "Round up all purchases to nearest 10"
```json
{
  "name": "Round-up savings",
  "description": "Round up purchases and save the difference",
  "trigger_account_key": "checking-account-key",
  "conditions": [
    { "type": "amount_less_than", "value": 0 },
    { "type": "is_settled" }
  ],
  "actions": [
    {
      "type": "transfer",
      "from_account": { "type": "trigger_account" },
      "to_account": { "type": "by_key", "key": "savings-account-key" },
      "amount": { 
        "type": "round_up_to",
        "nearest": 10
      },
      "message": "Round-up"
    }
  ]
}
```

---

## Future Roadmap (Not in Scope)

These features are explicitly deferred for later:

- [ ] **Dry-run mode**: Evaluate rules without executing transfers
- [ ] **Rate limiting**: Respect SpareBank 1 API rate limits with backoff
- [ ] **Multi-user support**: OAuth per user, session management
- [ ] **Notifications**: Email/push when rules execute
- [ ] **Scheduled rules**: Time-based triggers (monthly, weekly)
- [ ] **Balance-based conditions**: "If account X balance < Y"
- [ ] **Mobile app**: React Native or Flutter frontend
- [ ] **Rule templates**: Pre-built rules for common scenarios
- [ ] **Import/export**: Backup and restore rules
- [ ] **Webhooks**: Notify external services on events

---

## Notes

### Transaction Deduplication Complexity

The SpareBank 1 API returns transactions that may be updated over time:
- `cleaned_description` can change as backend systems process
- `booking_status` changes from pending to booked
- Other metadata may be enriched

Our strategy:
1. **Fingerprint mutable fields** to detect meaningful changes
2. **Track processing per rule+transaction+fingerprint** to avoid duplicate actions
3. **Store raw transaction data** for debugging and auditing
4. **Consider settlement status** - only process settled transactions by default (configurable per rule)

### Security Considerations

- OAuth tokens stored securely (file permissions, encryption at rest in future)
- All transfers logged in audit trail
- No sensitive data in logs (token values redacted)
- API calls made server-side only (frontend never sees bank credentials)

### Error Handling

- Failed transfers logged but don't crash scheduler
- Retries with exponential backoff for transient failures
- Clear error messages in audit log for debugging
- User-visible error states in frontend

---

## Getting Started (After Implementation)

```bash
# Enter development environment
nix develop

# Initialize database
just migrate

# Configure API credentials
cp config.example.toml ~/.config/autobank/config.toml
# Edit with your SpareBank 1 API credentials

# Run backend
just dev

# In another terminal, run frontend
just web

# Open http://localhost:5173
```
