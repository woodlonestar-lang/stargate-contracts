# Stargate Contracts task runner

# Compile all contracts
build:
    cargo build

# Run all tests
test:
    cargo test

# Format all code
fmt:
    cargo fmt --all

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Deploy contracts to local Soroban node
deploy:
    @echo "Deployment script not yet implemented."
    @echo "See scripts/ for deployment helpers or MAINNET_DEPLOYMENT.md for manual process."

# Regenerate ABI snapshots
snapshot:
    @echo "ABI snapshot generation not yet implemented."
    @echo "Wire to snapshot command once available."

# Run format and lint checks (for CI)
check: fmt lint test
    @echo "✓ All checks passed"

# Default target
default: check
