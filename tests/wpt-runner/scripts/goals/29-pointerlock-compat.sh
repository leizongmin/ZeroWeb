#!/usr/bin/env bash
# Goal: pointerlock-compat（编号 29——推进顺序见同目录 README.md；**门控 goal**，学 99-webgl）
# Pointer Lock 面（requestPointerLock/pointerlockchange 语义；host 鼠标捕获依赖
# host-runtime 面——uievents 挂账记录）。
# 分工：M1 盘点（corpus census + 可执行子集甄别）= 自主切片；指针捕获管线 =
# **用户点名门控**。
# 用法：bash tests/wpt-runner/scripts/goals/29-pointerlock-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="pointerlock-compat"
DIRS=( "pointerlock" )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
