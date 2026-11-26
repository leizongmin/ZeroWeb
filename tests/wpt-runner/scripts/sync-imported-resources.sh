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
  # fetch-wpt-data 整步失败。--connect-timeout/--max-time 限定单次尝试
  # 上限：无显式超时时半开连接会挂满系统级超时（windows-11-arm 实测
  # 单资源静默 ~9 分钟后整体 exit 4，run 45680976019），重试形同虚设
  curl --retry 5 --retry-all-errors --retry-delay 3 --connect-timeout 20 --max-time 120 -sSf -o "$target" "${WPT_BASE}/${resource_path}"
done < "$LEDGER"
