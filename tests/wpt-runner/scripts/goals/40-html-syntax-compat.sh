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

# depth-2 support/ 资产补拉（fetch_dir_html 深度<1 不入子子目录）：运行面 RUN 案引用的
# helper——render-blocking support 脚本（utils.js / test-render-blocking.js / dummy-1.*）
# 与 parsing support 脚本（DOMContentLoaded-defer.js / svg-script-self-closing.js）。
# lib.sh 拉取模式不含 .mjs/.css（此前 corpus 无此形态）——dummy-1.mjs / target-red.css
# 显式补拉（/common/sab.js 显式补拉先例）。
fetch_dir_html "html/dom/render-blocking/support"
fetch_dir_html "html/syntax/parsing/support"
fetch_raw "html/dom/render-blocking/support/dummy-1.mjs"
fetch_raw "html/dom/render-blocking/support/target-red.css"

goals_inventory
goals_next_steps "${GOAL}"
