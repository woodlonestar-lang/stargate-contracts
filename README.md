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
make check-abi-snapshots
```

## ABI snapshots

Committed ABI metadata in `abis/` is generated from contract sources. Before opening a PR that changes contract behavior, refresh snapshots:

```sh
make update-abi-snapshots
```

Confirm the tree is clean:

```sh
make check-abi-snapshots
git diff --exit-code abis/
```

The generator sets `LC_ALL=C` and `LANG=C` so output is identical across machines.

See `CONTRIBUTING.md` for local pre-commit hooks.

## Deployment

```sh
cp .env.testnet.example .env.testnet
scripts/deploy_testnet.sh
```

After deployment, contract IDs are exported to `artifacts/addresses.json` (gitignored; environment-specific). See `artifacts/addresses.json.example` for the schema:

- `network`: Stellar network name (for example `testnet`)
- `contracts[]`: `name` and deployed `address` for each contract

Override the output path with `DEPLOYED_ADDRESSES_FILE` when calling `scripts/export_deployed_addresses.sh`.

Mainnet deployment is intentionally manual and must go through multi-sig governance.

See `docs/MAINNET_DEPLOYMENT.md` for the live deployment checklist and signing ceremony.

## License

MIT
