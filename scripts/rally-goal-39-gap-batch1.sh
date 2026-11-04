#!/usr/bin/env bash
# 用 rally run 持续推进 gap-batch1 goal（docs/goal/gap-batch1.md）。
#
# 用途：覆盖率缺口小面批量补齐 batch1——CONFIRMED 六面（quirks/domparsing/
# domxpath/subresource-integrity/cors/import-maps）通道五件套 + LIKELY 六面
# 甄别入账 + mixed-content 语料窗口全量。coverage-2026-11-02 对账：十二面
# 合计 28,394 子测试 0 通过，纯通道增量 ~0.6pp。
# M0 语料预置脚本：tests/wpt-runner/scripts/goals/39-gap-batch1.sh
# （M0 落地时创建；FORCE=1 续拉先例）。
#
# 用法：
#   bash scripts/rally-goal-39-gap-batch1.sh                # 推进 gap-batch1 goal
#   bash scripts/rally-goal-39-gap-batch1.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/gap-batch1.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
