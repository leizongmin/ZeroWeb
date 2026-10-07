#!/usr/bin/env bash
# Goal: touch-events-compat（编号 22——推进顺序见同目录 README.md）
# TouchEvent 构造器/属性面（uievents 挂账面之一；uievents M2 driver actions
# 重写为注入底座）
# M1 资产预置：fetch touch-events 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/touch-events-compat.md DC-1。
# 注意：构造器/属性面（Touch/TouchEvent/ForceRelation 纯 JS）应可直接执行；
# 依赖宿主触摸注入的用例 M1 甄别可执行子集。
# 用法：bash tests/wpt-runner/scripts/goals/22-touch-events-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="touch-events-compat"
DIRS=( "touch-events" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
