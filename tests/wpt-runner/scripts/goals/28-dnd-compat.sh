#!/usr/bin/env bash
# Goal: dnd-compat（编号 28——推进顺序见同目录 README.md；**门控 goal**，学 99-webgl）
# HTML5 拖放面（DataTransfer/DragEvent 构造器为纯 JS 面；真实拖放交互依赖 host
# drag-input 管道——web-api-batch2 挂账记录）。
# 分工：M1 盘点 + M2 构造器/DataTransfer 面 = 自主切片；拖放交互管线 = **用户点名
# 门控**（M1 盘点为唯一自主切片的规则见 README.md 99 行先例）。
# 用法：bash tests/wpt-runner/scripts/goals/28-dnd-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="dnd-compat"
DIRS=( "dnd" )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
