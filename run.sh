#!/usr/bin/env bash
# Regenerate vectors, judge with Agave, run every harness, write results/REPORT.md
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --manifest-path oracle/Cargo.toml
mkdir -p vectors results
./oracle/target/release/txconform-oracle gen --out vectors/vectors.jsonl --random "${RANDOM_VECTORS:-200}" --mutations "${MUTATIONS:-1000}"
./oracle/target/release/txconform-oracle judge --vectors vectors/vectors.jsonl --out vectors/oracle.jsonl
( cd harness/ts && [ -d node_modules ] || npm ci
  node harness_kit.js ../../vectors/vectors.jsonl ../../results/kit.jsonl
  node harness_web3.js ../../vectors/vectors.jsonl ../../results/web3js3.jsonl
  node harness_web3.js ../../vectors/vectors.jsonl ../../results/web3js1.jsonl web3js1 )
PY=${PYTHON:-python3}
$PY harness/py/harness_solders.py vectors/vectors.jsonl results/solders.jsonl
( cd harness/go && go build -o harness-go . && ./harness-go ../../vectors/vectors.jsonl ../../results/solana-go.jsonl )
( cd harness/go-gagliardetto && go build -o harness-go-gagliardetto . && ./harness-go-gagliardetto ../../vectors/vectors.jsonl ../../results/gagliardetto.jsonl )
( cd harness/rust-primitives && cargo build --release -q && ./target/release/txconform-harness-primitives ../../vectors/vectors.jsonl ../../results/primitives.jsonl )
python3 tools/rust_sdk_results.py vectors/oracle.jsonl results/rust-sdk.jsonl oracle/Cargo.lock
python3 txconform.py --results results/rust-sdk.jsonl results/kit.jsonl results/web3js3.jsonl results/web3js1.jsonl results/solders.jsonl results/solana-go.jsonl results/gagliardetto.jsonl results/primitives.jsonl > /dev/null
python3 tools/paykit_export.py vectors/vectors.jsonl vectors/oracle.jsonl vectors/paykit-wire-vectors.json
echo "report: results/REPORT.md"
