#!/bin/sh
# 推进增量渲染对照目标；加 --dry-run 仅预览 Rally 命令。
set -eu
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
PROJECT_ROOT="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)"
exec rally run "$PROJECT_ROOT/docs/goal/incremental-rendering-oracle.md" \
    -w "$PROJECT_ROOT" --agent-command claude-glm "$@"
