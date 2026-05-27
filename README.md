# Stargate Contracts

Stellar/Soroban smart contracts for Stargate Protocol.

This repository owns invoice escrow state, payment validation, multi-sig treasury settlement, and compliance gates.

## Workspace

- `contracts/invoice`: invoice state machine and payment marking
- `contracts/treasury`: 2-of-3 settlement approval workflow
- `contracts/compliance`: admin-managed allow/block list
- `abis`: committed ABI metadata consumed by `stargate-backend`

## Local Development

### Starting the Local Environment

Requires Docker and Docker Compose.

```sh
docker-compose up -d
```

This starts:
- **Soroban Node**: Stellar quickstart (Horizon at `http://localhost:8000`)
- **Redis**: Event consumer backing service (port 6379)

Check service health:
```sh
docker-compose ps
curl http://localhost:8000/health
```

### Deploying Contracts Locally

```sh
cp .env.local.example .env.local
# Edit .env.local with your test keys
scripts/deploy_local.sh  # if available, or use stellar-cli with soroban
```

### Tearing Down

```sh
docker-compose down
# To also remove persistent data:
docker-compose down -v
```

## Development Tasks

This project uses `just` for common contract development commands. Install from https://github.com/casey/just.

```sh
# Format code
just fmt

# Run lints
just lint

# Run tests
just test

# Run all checks (format, lint, test)
just check

# Or use cargo directly
cargo fmt --all
cargo clippy -- -D warnings
cargo test
```

## Coverage

Generate coverage reports for contract tests:

```sh
scripts/coverage.sh
```

This produces an HTML report in `coverage/index.html` and an LCOV file for CI integration.

## Development

### VS Code Dev Container

A fully configured development environment is available as a VS Code Dev Container. It includes Rust, the Soroban CLI, and recommended extensions.

To use it:

1. Install [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
2. Open the repo in VS Code
3. Press `Ctrl+Shift+P` and select "Dev Containers: Reopen in Container"

See `docs/dev-environment.md` for full setup instructions.

## Deployment

```sh
cp .env.testnet.example .env.testnet
scripts/deploy_testnet.sh
```

Mainnet deployment is intentionally manual and must go through multi-sig governance.

See `docs/MAINNET_DEPLOYMENT.md` for the live deployment checklist and signing ceremony.

## License

MIT
