#!/usr/bin/env bash
# 用 rally run 持续推进 webcrypto-compat goal（docs/goal/webcrypto-compat.md）。
#
# 用途：长期无人值守推进。M1 盘点/基线自主；subtle 实现轮按契约切片自主推进（依赖选型落 evidence/）。
# M1 语料预置脚本：tests/wpt-runner/scripts/goals/26-webcrypto-compat.sh
#   （rally 轮内也可直接跑；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/26-webcrypto-compat.sh                # 推进 webcrypto-compat goal
#   bash scripts/26-webcrypto-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/webcrypto-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
