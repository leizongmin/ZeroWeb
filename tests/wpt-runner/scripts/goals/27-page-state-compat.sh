#!/usr/bin/env bash
# Goal: page-state-compat（编号 27——推进顺序见同目录 README.md）
# beacon + page-visibility + online 三小套件打包（event-loop-spec 多套件先例）
# sendBeacon = net POST 底座薄封装；visibilitychange/online = 事件循环遗产消费。
# M1 资产预置：fetch 三子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/page-state-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/27-page-state-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="page-state-compat"
DIRS=( "beacon" "page-visibility" "online" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
