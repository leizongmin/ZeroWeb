#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT="${1:-$PROJECT_ROOT/tests/benchmarks/results/form-input-latest.json}"

mkdir -p "$(dirname "$REPORT")"
cargo run --release --quiet -p zero-integration-tests --bin form-input-perf > "$REPORT"
bash "$SCRIPT_DIR/form-input-perf-gate.sh" "$REPORT"
