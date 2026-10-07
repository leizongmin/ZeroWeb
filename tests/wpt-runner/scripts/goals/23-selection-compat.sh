#!/usr/bin/env bash
# Goal: selection-compat（编号 23——推进顺序见同目录 README.md）
# Selection API 面（js-dom 已做 Range、editing 已做 contenteditable，selection 面
# 从未 goal 化度量）
# M1 资产预置：fetch selection 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/selection-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/23-selection-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="selection-compat"
DIRS=( "selection" )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
