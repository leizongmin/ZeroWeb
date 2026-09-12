#!/usr/bin/env bash
# Goal: navigation-compat（编号 60——推进顺序见同目录 README.md）
# M3 iframe 切片用户门控
# M1 资产预置：fetch history navigation-api html/browsers 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/navigation-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/60-navigation-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="navigation-compat"
DIRS=( "history" "navigation-api" "html/browsers"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
