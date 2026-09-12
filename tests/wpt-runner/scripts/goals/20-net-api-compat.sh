#!/usr/bin/env bash
# Goal: net-api-compat（编号 20——推进顺序见同目录 README.md）
# 主攻切片（WebSocket 二期挂账）
# M1 资产预置：fetch fetch xhr url mimesniff streams eventsource 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/net-api-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/20-net-api-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="net-api-compat"
DIRS=( "fetch" "xhr" "url" "mimesniff" "streams" "eventsource"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
