#!/usr/bin/env bash
# 用 rally run 持续推进 form-url-encoding-compat goal（docs/goal/form-url-encoding-compat.md）。
#
# 用途：legacy-mb encode 面（form 提交编码序列化 + href URL encoding-parse）
# ——31 号 encoding-legacy-mb M3 甄别归口；全量 WPT 通过率到 50%+ 的唯一
# 未立项杠杆（encode 面 767,673 子测试）。
# M0/M1 语料盘点脚本：tests/wpt-runner/scripts/goals/36-form-url-encoding-compat.sh
#   （语料已全量在位，无需 FORCE；编号索引见该目录 README.md）。
#
# 用法：
#   bash scripts/rally-goal-36-form-url-encoding-compat.sh                # 推进 form-url-encoding-compat goal
#   bash scripts/rally-goal-36-form-url-encoding-compat.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/form-url-encoding-compat.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
