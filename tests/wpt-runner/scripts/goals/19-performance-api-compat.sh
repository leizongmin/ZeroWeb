#!/usr/bin/env bash
# Goal: performance-api-compat（编号 19——推进顺序见同目录 README.md）
# 资源/导航/事件时序扩面（10-timing 已圈 hr-time/timeline/user-timing/WAAPI，
# 本 goal 圈 performance-api 四件套）
# M1 资产预置：fetch performance-api 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/performance-api-compat.md DC-1。
# 注意：resource/navigation-timing 需要 host 侧资源装载与导航生命周期数据管道——
# M1 盘点甄别可执行子集，缺面部分如实挂账（分母口径注记）。
# 用法：bash tests/wpt-runner/scripts/goals/19-performance-api-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="performance-api-compat"
DIRS=( "performance-api" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
