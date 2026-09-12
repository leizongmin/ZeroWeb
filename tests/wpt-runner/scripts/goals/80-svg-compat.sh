#!/usr/bin/env bash
# Goal: svg-compat（编号 80——推进顺序见同目录 README.md）
# SVG 文档模型（渲染面归 rendering-compat）
# M1 资产预置：fetch svg 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/svg-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/80-svg-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="svg-compat"
DIRS=( "svg"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
