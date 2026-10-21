#!/usr/bin/env bash
# Goal: workers-compat（编号 70——推进顺序见同目录 README.md）
# 运行时工程量大，契约先行
# M1 资产预置：fetch workers dedicated-workers 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/archive/workers-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/70-workers-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="workers-compat"
DIRS=( "workers" "dedicated-workers"  )
# M3 补充依赖目录：semantics/structured-clone/dedicated.html 的页面级 META script
# （structured-clone-battery-of-tests*.js）与 /common/sab.js——该案脚本经 <script src>
# 由 runner 页面通道抓取，语料缺文件即 ReferenceError（2026-10-21 记账后补）。
# 仅作支撑资产：runner workers 通道扫描面（WORKERS_TEST_SUBDIRS）不含该目录，
# 不引入新执行案。
DIRS+=( "html/webappapis/structured-clone" )
# 语料三层嵌套（interfaces/WorkerGlobalScope/close/ 等），shallow Contents API 只递归
# 一层拉不全——走 git trees 深挖（lib.sh fetch_dir_deep，全量树；WebCrypto 先例）。
# .any.js window 变体 opt-in（Worker-base64/WorkerNavigator/importScripts error 面
# 等 31 案是 window 可执行面成员——webstorage/net-api 先例；M2 起 dedicated 变体
# 经 testharness_workers 变体分道走 dedicated worker 通道）。
# .worker.js in-worker harness 形态 opt-in（M2 worker harness 通道重入——
# fetch_tests_from_worker 聚合，canvas worker 通道先例）。
# 注：上游 pin 无 dedicated-workers 目录（dedicated worker 用例 2026 起并入
# workers/）——跳过属预期。
GOAL_FETCH_DEEP=1
GOAL_PULL_ANY_JS=1
GOAL_PULL_WORKER_JS=1
GOAL_PULL_HEADERS_TARGETS=1

goals_fetch_all
# 单文件支撑资产（不整目录拉——common/ 为全语料共享根，只取本 goal 依赖）。
fetch_raw "common/sab.js" || echo "  common/sab.js 拉取失败（瞬时错误重跑续拉）" >&2
goals_inventory
goals_next_steps "${GOAL}"
