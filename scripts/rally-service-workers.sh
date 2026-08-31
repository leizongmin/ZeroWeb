#!/usr/bin/env bash
# 用 rally run 持续推进 service-workers goal（docs/goal/service-workers.md）。
#
# 用途：长期无人值守推进 Service Worker 目标（方案 C RFC 已批准；M1 已完成，
# M2 fetch/cache 与 M3 控制语义持续推进），agent-command 用 claude-glm
#（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-service-workers.sh                # 推进 service-workers goal
#   bash scripts/rally-service-workers.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/service-workers.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
