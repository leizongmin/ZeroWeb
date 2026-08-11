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
  # --retry-all-errors：raw.githubusercontent.com 在 CI runner 上偶发
  # DNS/连接失败（macos-15-intel 实测 curl (6)/(56)），重试吸收，避免
  # fetch-wpt-data 整步失败
  curl --retry 5 --retry-all-errors --retry-delay 3 -sSf -o "$target" "${WPT_BASE}/${resource_path}"
done < "$LEDGER"
