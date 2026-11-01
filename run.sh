#!/usr/bin/env bash
# Regenerate vectors, judge with Agave, run every harness, write results/REPORT.md
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --manifest-path oracle/Cargo.toml
mkdir -p vectors results
./oracle/target/release/txconform-oracle gen --out vectors/vectors.jsonl --random "${RANDOM_VECTORS:-200}"
./oracle/target/release/txconform-oracle judge --vectors vectors/vectors.jsonl --out vectors/oracle.jsonl
( cd harness/ts && [ -d node_modules ] || npm ci
  node harness_kit.js ../../vectors/vectors.jsonl ../../results/kit.jsonl
  node harness_web3.js ../../vectors/vectors.jsonl ../../results/web3js3.jsonl
  node harness_web3.js ../../vectors/vectors.jsonl ../../results/web3js1.jsonl web3js1 )
PY=${PYTHON:-python3}
$PY harness/py/harness_solders.py vectors/vectors.jsonl results/solders.jsonl
( cd harness/go && go build -o harness-go . && ./harness-go ../../vectors/vectors.jsonl ../../results/solana-go.jsonl )
python3 txconform.py --results results/kit.jsonl results/web3js3.jsonl results/web3js1.jsonl results/solders.jsonl results/solana-go.jsonl > /dev/null
echo "report: results/REPORT.md"
