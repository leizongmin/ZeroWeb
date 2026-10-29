#!/usr/bin/env bash
# Goal: wasm-compat（编号 32——推进顺序见同目录 README.md）
# wasm/jsapi 已有通道（testharness-wasm，coverage-2026-10-27 干跑 31 文件
# 715/945 = 75.7%）；本 goal 扩面 wasm/core 全量 + wasm/webapi。
#
# wasm/core 258 文件（188,319 子测试，干跑口径 0 覆盖）是纯语料扩充面——用例
# 形态以 .any.js 为主（any_js_window_wrapper window 变体执行，fs/timing 先例），
# 埋在 core/ 二三层子目录，必须深挖模式。预计 75% 起步；缺口预计集中在
# threads/SIMD 等 proposal 面（wasm-sandbox feature gate 边界，逐簇挂账）。
# 契约见 docs/goal/archive/wasm-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/32-wasm-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="wasm-compat"
GOAL_FETCH_DEEP=1
GOAL_PULL_ANY_JS=1
# .wasm 二进制资产（wasm/webapi/resources incrementer*.wasm——contenttype/
# instantiateStreaming fetch 集成面依赖；lib.sh opt-in 同 .txt 先例）。
GOAL_PULL_WASM=1
DIRS=(
  "wasm/core"
  "wasm/webapi"
)

goals_fetch_all
# 顶层 wasm/ 散资产单点补拉（webapi fetch 集成面绝对/相对引用：'/wasm/incrementer.wasm'
# 与 '../incrementer.wasm' 都落 wasm/incrementer.wasm——DIRS 深挖不覆盖顶层散文件；
# contenttype/historical/wasm_stream_* 依赖。fetch_raw 由 lib.sh 提供）。
fetch_raw "wasm/incrementer.wasm" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
goals_inventory
goals_next_steps "${GOAL}"
