#!/usr/bin/env bash
# Goal: webgl-compat（编号 99——推进顺序见同目录 README.md）
# 远期门控（M2+ 全门控）
# M1 资产预置：fetch webgl 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/webgl-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/99-webgl-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="webgl-compat"
DIRS=( "webgl"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
