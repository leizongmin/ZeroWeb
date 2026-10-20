#!/bin/sh
# 推进 renderer 信任边界目标；加 --dry-run 仅预览 Rally 命令。
set -eu
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
PROJECT_ROOT="$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)"
exec rally run "$PROJECT_ROOT/docs/goal/renderer-trust-boundary.md" \
    -w "$PROJECT_ROOT" --agent-command claude-glm "$@"
