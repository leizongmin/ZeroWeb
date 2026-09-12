#!/usr/bin/env bash
# Goal: uievents-compat（编号 50——推进顺序见同目录 README.md）
# 合成事件语义先行
# M1 资产预置：fetch uievents pointerevents 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/uievents-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/50-uievents-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="uievents-compat"
DIRS=( "uievents" "pointerevents"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
