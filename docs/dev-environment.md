# Development Environment Setup

This guide covers setting up a full local development environment for the Stargate protocol, spanning the contracts, backend, and frontend repositories.

## Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target:

  ```sh
  rustup install stable
  rustup target add wasm32-unknown-unknown
  ```

- Soroban CLI: `cargo install soroban-cli`
- Node.js 18+ (for frontend)
- Docker (for local Soroban sandbox)
- Stellar testnet account with funded testnet USDC

## Directory Layout

Create a workspace directory and clone in this order:

```
~/stargate/
  ├── stargate-contracts/     # This repo (contracts)
  ├── stargate-backend/       # Backend API
  └── stargate-frontend/      # Frontend UI
```

```sh
mkdir ~/stargate && cd ~/stargate
git clone https://github.com/dreamgeneX/stargate-contracts.git
git clone https://github.com/dreamgeneX/stargate-backend.git
git clone https://github.com/dreamgeneX/stargate-frontend.git
```

## Local Soroban Sandbox

Start a local Soroban sandbox for testing without testnet:

```sh
soroban-cli start --standalone
```

This runs Soroban RPC on `http://localhost:8000` and Horizon on `http://localhost:8001`.

## Environment Setup

### Contracts

Copy the testnet configuration and generate a test account:

```sh
cd stargate-contracts
cp .env.testnet.example .env.testnet
```

Generate a new testnet keypair for local testing:

```sh
soroban config identity generate dev
soroban config set --scope testnet RPC_URL http://localhost:8000
soroban config set --scope testnet NETWORK_PASSPHRASE "Standalone Network ; February 2025"
```

Export your account ID for use in backend/frontend configuration:

```sh
ADMIN_PUBLIC_KEY=$(soroban config identity show dev)
echo "ADMIN_PUBLIC_KEY=$ADMIN_PUBLIC_KEY"
```

### Deploy Contracts Locally

```sh
cd stargate-contracts

# Build WASM artifacts
cargo build --target wasm32-unknown-unknown --release

# Deploy to local sandbox
./scripts/deploy_testnet.sh
```

This outputs contract IDs. Save them:

```sh
export INVOICE_CONTRACT_ID=<id>
export TREASURY_CONTRACT_ID=<id>
export COMPLIANCE_CONTRACT_ID=<id>
```

### Backend

```sh
cd stargate-backend

# Create .env from the contracts output
cat > .env <<EOF
STELLAR_NETWORK=testnet
SOROBAN_RPC_URL=http://localhost:8000
HORIZON_URL=http://localhost:8001
ADMIN_PUBLIC_KEY=$ADMIN_PUBLIC_KEY
INVOICE_CONTRACT_ID=$INVOICE_CONTRACT_ID
TREASURY_CONTRACT_ID=$TREASURY_CONTRACT_ID
COMPLIANCE_CONTRACT_ID=$COMPLIANCE_CONTRACT_ID
USDC_CONTRACT_ID=CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4
EOF

# Install and run
cargo build
cargo run
```

Backend listens on `http://localhost:3000`.

### Frontend

```sh
cd stargate-frontend

# Create .env
cat > .env <<EOF
VITE_API_URL=http://localhost:3000
VITE_SOROBAN_RPC=http://localhost:8000
VITE_HORIZON_URL=http://localhost:8001
VITE_NETWORK_PASSPHRASE=Standalone Network ; February 2025
EOF

# Install and run
npm install
npm run dev
```

Frontend runs on `http://localhost:5173`.

## Running Contract Tests

```sh
cd stargate-contracts

# Run all contract tests
cargo test

# Generate coverage report
scripts/coverage.sh
```

## Development Workflow

1. **Make contract changes** in `stargate-contracts/contracts/*/src/`
2. **Rebuild and redeploy**:

   ```sh
   cd stargate-contracts
   cargo build --target wasm32-unknown-unknown --release
   ./scripts/deploy_testnet.sh
   ```

3. **Restart backend** to reload new contract IDs (if changed)
4. **Test in frontend** UI

## Troubleshooting

- **"Soroban RPC not reachable"**: Ensure sandbox is running with `soroban-cli start --standalone`
- **"Contract not found"**: Verify contract IDs in `.env` match deployed IDs from deployment script
- **"USDC balance insufficient"**: Fund your testnet account at [Stellar Lab](https://laboratory.stellar.org/#create-account)
- **Port already in use**: Change the port in backend/frontend env files if 3000 or 5173 are taken

## Further Reading

- [Soroban Docs](https://developers.stellar.org/soroban)
- [Mainnet Deployment](./MAINNET_DEPLOYMENT.md)
