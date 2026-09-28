#!/usr/bin/env bash
# Goal: net-api-compat（编号 20——推进顺序见同目录 README.md）
# 主攻切片（WebSocket 二期挂账）
# M1 资产预置：fetch xhr url mimesniff streams eventsource 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/net-api-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/20-net-api-compat.sh（FORCE=1 强制重拉）
#
# **拉取面（2026-09-28 M1 rally 轮定稿，照 fetch-dom-subset.sh SUBDIRS 先例显式列目录）**：
# 逐 corpus 对准 M2/M3 语义簇 + 自包含面；`.any.js` window 变体拉取（GOAL_PULL_ANY_JS=1，
# lib.sh opt-in——streams/ 以 .any.js 为唯一形态）。resources/ 目录只拉 helper .js（运行面
# skip 规则排除，双保险）。
#
# **显式不拉（记账，重入条件见括号）**：
# - fetch/api/cors|redirect|urls|scheme（wptserve .py 端点重依赖——runner fixture 通道扩展后）
# - fetch/api/policies|metadata|cross-origin-resource-policy|local-network-access|security
#   （策略/头面——security-hardening goal 域）
# - fetch/http-cache|stale-while-revalidate|content-encoding|connection-pool|range|nosniff|orb|corb
#   （HTTP 栈本体/内容编码/ORB 读取算法——goal 排除「HTTP 栈本身」；nosniff 策略面归 security）
# - fetch/compression-dictionary|fetch-later（新 API + 服务器基建，二期评估）
# - fetch|streams|xhr 等域 crashtests/（崩溃回归面，非语义簇）
# - streams/transferable（postMessage transfer 面——runner 无跨窗/worker 结构化克隆管道）
# - mimesniff/sniffing（字节嗅探 .py 支持面——M3 重评估）
# - eventsource/dedicated-worker（worker 面）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="net-api-compat"
GOAL_PULL_ANY_JS=1
DIRS=(
  # ── fetch：M2 主簇（Headers/Request/Response/Body/fetch 方法语义/abort/credentials）+ 自包含面 ──
  "fetch/api/headers"
  "fetch/api/request"
  "fetch/api/response"
  "fetch/api/body"
  "fetch/api/basic"
  "fetch/api/abort"
  "fetch/api/credentials"
  "fetch/api/resources"
  "fetch/data-urls"
  "fetch/content-type"
  "fetch/content-length"
  "fetch/h1-parsing"
  "fetch/images"
  "fetch/redirects"
  # ── xhr：状态机 + 事件序 + FormData（M3 主簇）──
  "xhr"
  "xhr/formdata"
  "xhr/resources"
  # ── url：URL/URLSearchParams 边缘语义（P1a 已落面修齐对象）──
  "url"
  "url/resources"
  # ── mimesniff：MIME 解析对齐 WHATWG 规则（sniffing 字节嗅探面记账不拉）──
  "mimesniff/mime-types"
  "mimesniff/mime-types/resources"
  # ── streams：Readable/Writable/Transform 底座（M4；transferable 记账不拉）──
  "streams"
  "streams/piping"
  "streams/readable-byte-streams"
  "streams/readable-streams"
  "streams/transform-streams"
  "streams/writable-streams"
  "streams/resources"
  # ── eventsource：SSE 解析/重连语义（M3；dedicated-worker 记账不拉）──
  "eventsource"
  "eventsource/resources"
)

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
