#!/usr/bin/env bash
# Goal: fileapi-compat（编号 16——推进顺序见同目录 README.md）
# Blob/File/FileReader/objectURL 面（net-api M2 打过 Blob.type 底，从未 goal 化度量）
# M1 资产预置：fetch FileAPI 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/fileapi-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/16-fileapi-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="fileapi-compat"
DIRS=( "FileAPI" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
