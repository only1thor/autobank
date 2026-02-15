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
