#!/usr/bin/env bash
# Goal: http-cache（编号 47——引擎 HTTP 缓存层 goal 的 corpus 消费面，控制面
# docs/goal/archive/http-cache.md，2026-11-09 已收口归档；脚本保留供语料续拉）
#
# 拉取 fetch/http-cache 语料（M0 对账档 §三盘点：20 文件，Chrome window
# 变体 417/554=75.3%；DC-4 corpus 门按批次对基准，批次排期对齐 46 号
# runner-net-infra M3）。
#
# **只消费不代做（划界）**：resources/ 下 wptserve .py 端点（http-cache.py
# stash 状态机、cached_pragma_*_rand.py、securedimage.py）由 46 号解锁，
# lib.sh 扩展名过滤器天然不拉 .py——本脚本零触碰。
#
# 语料以 .any.js 为主要形态（window 变体由 runner any_js_window_wrapper
# 执行；入窗时间点 = fetch/http-cache 加入 testharness.rs
# NET_API_CORPUS_SUBDIRS，届时需 46 号 py 端点前置就位）。
#
# 用法：bash tests/wpt-runner/scripts/goals/47-http-cache.sh（FORCE=1 强制重拉；
# 建议带 GITHUB_TOKEN=$(gh auth token) 跑——目录列举走 GitHub contents API，
# 匿名 60 req/h 易烧光）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="http-cache"
GOAL_PULL_ANY_JS=1
DIRS=(
  # ── fetch/http-cache：RFC 9111 语义簇语料（304-update/cache-mode/freshness/
  #    no-vary-search/status/cc-request/heuristic/invalidate/partial/vary/
  #    pragma-no-cache；split-cache 多窗面排除、basic-auth 零分母排除——
  #    M0 对账档 §三记账）──
  "fetch/http-cache"
)

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
