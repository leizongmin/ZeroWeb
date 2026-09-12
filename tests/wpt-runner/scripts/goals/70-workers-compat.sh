#!/usr/bin/env bash
# Goal: workers-compat（编号 70——推进顺序见同目录 README.md）
# 运行时工程量大，契约先行
# M1 资产预置：fetch workers dedicated-workers 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/workers-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/70-workers-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="workers-compat"
DIRS=( "workers" "dedicated-workers"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
