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
    @./scripts/generate_abi_metadata.sh abis

# Check dependencies for vulnerabilities and license issues
deny:
    cargo deny check

# Run format and lint checks (for CI)
check: fmt lint test deny
    @echo "✓ All checks passed"

# Default target
default: check
