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

# TBD-1 已决（RFC §10，s4 落地 2026-10-19）：pointerlock_shadow 依赖的 shadow-dom
# 测试 helper（createTestTree 等）——单文件补拉（20-net-api-compat 资产先例；不入
# DIRS 扫描面，runner 只扫 pointerlock/）。实跑 shadow 案翻绿（M3 Commit 5）。
fetch_raw "shadow-dom/resources/shadow-dom.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
