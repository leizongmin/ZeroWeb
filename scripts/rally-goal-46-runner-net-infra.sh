#!/usr/bin/env bash
# 用 rally run 持续推进 runner-net-infra goal（docs/goal/runner-net-infra.md）。
#
# 用途：runner 网络 fixture 基建（用户 2026-11-04 拍板立项）——TLS/wss
# 端点 fixture（net-api P3 挂账转正）+ wptserve py 端点 shim 框架化
# （v2 逐案先例升级为路由表框架，S12 本地补丁收编）。下游：43 号
# websockets wss 面、39/47 号 py 端点目录；不碰引擎网络栈语义。
# M0 语料预置脚本：tests/wpt-runner/scripts/goals/46-runner-net-infra.sh
# （M0 落地时创建；FORCE=1 续拉先例）。
#
# 用法：
#   bash scripts/rally-goal-46-runner-net-infra.sh                # 推进 runner-net-infra goal
#   bash scripts/rally-goal-46-runner-net-infra.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/runner-net-infra.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
