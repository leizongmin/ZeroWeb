#!/usr/bin/env bash
# Goal: html-syntax-compat（编号 40——推进顺序见同目录 README.md）
# html-compat（fixture 制）的 WPT 化续篇
# M1 资产预置：fetch html/syntax html/dom 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/html-syntax-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/40-html-syntax-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="html-syntax-compat"
DIRS=( "html/syntax" "html/dom"  )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
