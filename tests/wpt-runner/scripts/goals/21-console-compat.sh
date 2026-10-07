#!/usr/bin/env bash
# Goal: console-compat（编号 21——推进顺序见同目录 README.md）
# 极小快赢切片（console IDL/格式化语义面；PR #101 console 错误序列化后的首次 goal 化度量）
# M1 资产预置：fetch console 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/console-compat.md DC-1。
# 注意：依赖 console 检查器（ConsoleRecorder/inspector 回调）的用例需 runner 侧
# 提供 console 消息捕获通道——M1 盘点甄别可执行子集。
# 用法：bash tests/wpt-runner/scripts/goals/21-console-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="console-compat"
DIRS=( "console" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
