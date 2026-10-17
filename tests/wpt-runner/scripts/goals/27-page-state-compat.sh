#!/usr/bin/env bash
# Goal: page-state-compat（编号 27——推进顺序见同目录 README.md）
# beacon + page-visibility + online 三小套件打包（event-loop-spec 多套件先例）
# sendBeacon = net POST 底座薄封装；visibilitychange/online = 事件循环遗产消费。
# M1 资产预置：fetch 三子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/page-state-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/27-page-state-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="page-state-compat"
# 上游无顶层 online/ 目录（2026-10-17 Contents API 实证 Not Found）——navigator.onLine
# 测试在 html/browsers/offline/browser-state/（navigator_online_online.https.html 等）。
DIRS=( "beacon" "page-visibility" "html/browsers/offline/browser-state" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
# 跨域 /common 依赖（headers/*.html 的 `<script src="/common/utils.js">` token() 与
# get-host-info.sub.js——webcrypto 先例单文件补拉）。
fetch_raw "common/utils.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "common/get-host-info.sub.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
goals_inventory
goals_next_steps "${GOAL}"
