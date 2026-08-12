#!/usr/bin/env bash
# Form-input smoothness gate: deterministic hard budgets plus an optional fixed-platform baseline.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT="${1:-}"

if [ -z "$REPORT" ] || [ ! -f "$REPORT" ]; then
    echo "form-input-perf-gate: missing report path" >&2
    exit 2
fi

if command -v python >/dev/null 2>&1 && python --version >/dev/null 2>&1; then
    PYTHON=python
else
    PYTHON=python3
fi
"$PYTHON" "$SCRIPT_DIR/form-input-perf-gate.py" "$REPORT" "$PROJECT_ROOT/docs/perf/baselines"
