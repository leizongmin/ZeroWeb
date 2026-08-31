#!/usr/bin/env bash
# 用 rally run 持续推进 storage-cache-api goal（docs/goal/storage-cache-api.md）。
#
# 用途：长期无人值守推进 Cache API / 存储目标（与 service-workers 的 sw 环境用例
# 面有边界划分——cache-storage/sw 类归 SW 流），agent-command 用 claude-glm
#（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-storage-cache-api.sh                # 推进 storage-cache-api goal
#   bash scripts/rally-storage-cache-api.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/storage-cache-api.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
