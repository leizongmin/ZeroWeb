#!/usr/bin/env bash
# Goal: webmessaging-compat（编号 17——推进顺序见同目录 README.md）
# postMessage/MessageChannel/BroadcastChannel 面（多进程 IPC 有底子，window 面未度量）
# M1 资产预置：fetch webmessaging 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/webmessaging-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/17-webmessaging-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="webmessaging-compat"
DIRS=( "webmessaging" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
