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
# run/ + other/ 是用例面；include/（implementation.js/tests.js/editor-test-utils.js/
# reset.css）与 data/（per-command 期望矩阵 bold.js 等）是 run/ 用例的共享依赖——
# DC-1「include/ 共享 helper 随目录拉取」。两目录无 .html，fetch_dir_deep 幂等
# 快路径不会跳过（每次重列 2 次 API，可接受）。
DIRS=(
  "editing/run"
  "editing/other"
  "editing/include"
  "editing/data"
)
# 注：run/createlink-with-selecting-img.html 引用的 /img/lion.svg 在 pin 快照
# 上游就不存在（根树无 img/ 目录，404）——不补拉，该案相关子测试按缺资源
# 计失败，基线分类记账。

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
