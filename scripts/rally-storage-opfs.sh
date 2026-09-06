#!/usr/bin/env bash
# 用 rally run 持续推进 storage-opfs goal（docs/goal/storage-opfs.md）。
#
# 用途：长期无人值守推进 OPFS 真实化目标（页面 OPFS 从 JS shim 内存虚拟树接到
# zero-storage 真实实现 + per-origin 持久化；存储三件套之三），agent-command 用
# claude-glm（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-storage-opfs.sh                # 推进 storage-opfs goal
#   bash scripts/rally-storage-opfs.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/storage-opfs.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
