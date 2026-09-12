#!/usr/bin/env bash
# Goal: timing-animation-compat（编号 10——推进顺序见同目录 README.md）
# 轻量热身切片（纯 JS API 面）
# M1 资产预置：fetch hr-time performance-timeline user-timing web-animations 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/timing-animation-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/10-timing-animation-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="timing-animation-compat"
DIRS=( "hr-time" "performance-timeline" "user-timing" "web-animations"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
