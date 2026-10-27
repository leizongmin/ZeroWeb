#!/usr/bin/env bash
# Goal: editing-compat（编号 33——推进顺序见同目录 README.md）
# execCommand/queryCommand/selection 编辑命令族（coverage-2026-10-27 对账：
# editing/run 96,745 + editing/other 16,339 子测试，0/2 文件覆盖）。
#
# 大分母长线 goal：编辑命令语义面广（execCommand 家族 + styleWithCSS 等），
# 预计初始通过率低——先语料预置与基线分类（Unsupported/Fail/Timeout 归簇），
# 再按簇立项收敛。与 uievents（50 号已收口）、selection（23 号二期）共享
# 编辑域路径，失败簇归口先按 run-rules §9 协调。
# 契约见 docs/goal/editing-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/33-editing-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="editing-compat"
GOAL_FETCH_DEEP=1
DIRS=(
  "editing/run"
  "editing/other"
)

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
