#!/usr/bin/env bash
# Regenerate committed ABI metadata under abis/ from contract sources.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:-"$ROOT_DIR/abis"}"

cd "$ROOT_DIR"
export LC_ALL=C
export LANG=C

echo "Building contracts (workspace test build)..."
cargo test --no-run --workspace

mkdir -p "$OUT_DIR"
python3 "$ROOT_DIR/scripts/generate_abi_metadata.py" "$OUT_DIR"

echo "ABI metadata written to $OUT_DIR"
