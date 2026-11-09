#!/usr/bin/env bash
# 无人值守执行本目标。可传 --dry-run 预览。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/wptfyi-css.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    --yes \
    "$@"
