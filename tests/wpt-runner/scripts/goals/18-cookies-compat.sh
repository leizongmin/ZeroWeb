#!/usr/bin/env bash
# Goal: cookies-compat（编号 18——推进顺序见同目录 README.md）
# 快赢切片（document.cookie 语义面；net crate cookie jar 既有实现的 JS 面首次 goal 化度量）
# M1 资产预置：fetch cookies 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/cookies-compat.md DC-1。
# 注意：依赖 HTTP 响应头（Set-Cookie via .headers/.py）的用例在静态语料 runner 上
# 大概率不可执行——M1 盘点时按可执行子集甄别，skip 规则落 runner 侧。
# 用法：bash tests/wpt-runner/scripts/goals/18-cookies-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="cookies-compat"
DIRS=( "cookies" )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
