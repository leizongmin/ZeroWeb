#!/usr/bin/env bash
# Goal: workers-compat（编号 70——推进顺序见同目录 README.md）
# 运行时工程量大，契约先行
# M1 资产预置：fetch workers dedicated-workers 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/workers-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/70-workers-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="workers-compat"
DIRS=( "workers" "dedicated-workers"  )
# 语料三层嵌套（interfaces/WorkerGlobalScope/close/ 等），shallow Contents API 只递归
# 一层拉不全——走 git trees 深挖（lib.sh fetch_dir_deep，全量树；WebCrypto 先例）。
# .any.js window 变体 opt-in（Worker-base64/WorkerNavigator/importScripts error 面
# 等 31 案是 window 可执行面成员——webstorage/net-api 先例）。
# 注：上游 pin 无 dedicated-workers 目录（dedicated worker 用例 2026 起并入 workers/，
# .worker.js in-worker harness 形态 21 案 fetch 层排除，M2 评估重入）——跳过属预期。
GOAL_FETCH_DEEP=1
GOAL_PULL_ANY_JS=1

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
