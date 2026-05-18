# Stargate Contracts

Stellar/Soroban smart contracts for Stargate Protocol.

This repository owns invoice escrow state, payment validation, multi-sig treasury settlement, and compliance gates.

## Workspace

- `contracts/invoice`: invoice state machine and payment marking
- `contracts/treasury`: 2-of-3 settlement approval workflow
- `contracts/compliance`: admin-managed allow/block list
- `abis`: committed ABI metadata consumed by `stargate-backend`

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
