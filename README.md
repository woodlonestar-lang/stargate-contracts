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

## Verification

```sh
cargo fmt --all
cargo clippy -- -D warnings
cargo test
```

## Deployment

```sh
cp .env.testnet.example .env.testnet
scripts/deploy_testnet.sh
```

Mainnet deployment is intentionally manual and must go through multi-sig governance.

See `docs/MAINNET_DEPLOYMENT.md` for the live deployment checklist and signing ceremony.

## License

MIT
