#!/usr/bin/env bash
# Goal: webstorage-compat（编号 15——推进顺序见同目录 README.md）
# 快赢切片（纯 JS API 面；zero-storage localStorage/sessionStorage 既有实现首次 goal 化度量）
# M1 资产预置：fetch webstorage 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/webstorage-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/15-webstorage-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="webstorage-compat"
# webstorage 顶层套件（localstorage/sessionStorage 语义 + 事件面）；fetch_dir_html
# 默认递归一层覆盖 event/ 子目录。
DIRS=( "webstorage" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
