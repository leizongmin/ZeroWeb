#!/usr/bin/env bash
# Goal: webmessaging-compat（编号 17——推进顺序见同目录 README.md）
# postMessage/MessageChannel/BroadcastChannel 面（多进程 IPC 有底子，window 面未度量）
# M1 资产预置：fetch webmessaging 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/archive/webmessaging-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/17-webmessaging-compat.sh（FORCE=1 强制重拉）
#
# **拉取面（2026-10-12 M1 rally 轮定稿）**："webmessaging" 递归一层覆盖顶层
# （Channel_postMessage_*/MessageEvent/MessagePort 的 .any.js、with-options/、
# without-ports/、with-ports/、broadcastchannel/、message-channels/）；
# 两层深的 close-event 与 resources（lib.sh 只递归一层）显式追加。
# `.any.js` window 变体拉取（GOAL_PULL_ANY_JS=1）。
#
# **显式不拉（记账，重入条件见括号）**：
# - `*.worker.js`（lib.sh 默认排除——dedicated worker harness；worker 面归
#   workers-compat 管道）
# - `.htm` 扩展（lib.sh 模式外——event.*.sub.htm 等七案全为 .sub. 子域前提或
#   iframe 跨文档面；MultiGlobal/subdomain 前提 runner 无，均在排除面）
# - `*.yml`/`README.md`（lib.sh 模式外——META/清单资产，非用例）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="webmessaging-compat"
# 两层深的目录（lib.sh 只递归一层）显式追加；"webmessaging" 递归覆盖其余。
DIRS=(
  "webmessaging"
  "webmessaging/message-channels/close-event"
  "webmessaging/message-channels/resources"
  "webmessaging/broadcastchannel/resources"
)

GOAL_PULL_ANY_JS=1
goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
