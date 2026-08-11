#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/../wpt-data"
LEDGER="${SCRIPT_DIR}/../imported-resources.txt"
WPT_BASE="https://raw.githubusercontent.com/web-platform-tests/wpt/master"

[[ -f "$LEDGER" ]] || exit 0

while IFS= read -r resource_path; do
  resource_path="${resource_path%%#*}"
  resource_path="$(echo "$resource_path" | xargs)"
  [[ -z "$resource_path" ]] && continue

  target="${DATA_DIR}/${resource_path}"
  [[ -f "$target" ]] && continue
  mkdir -p "$(dirname "$target")"
  echo "fetch imported WPT resource: ${resource_path}"
  curl -sSf -o "$target" "${WPT_BASE}/${resource_path}"
done < "$LEDGER"
