#!/usr/bin/env bash
# 用 rally run 持续推进 http-cache goal（docs/goal/http-cache.md）。
#
# 用途：引擎 HTTP 缓存层（用户 2026-11-04 拍板立项）——内存态语义层：
# freshness/条件请求 304/Vary + request.cache 六模式态，fetch/xhr 双通道
# 同一缓存层；net-api-v2 request-cache 98F standing exclusion 转正承接。
# 磁盘持久化与产品缓存策略排除；py 端点前置归 46 号 runner-net-infra。
# M0 语料预置脚本：tests/wpt-runner/scripts/goals/47-http-cache.sh
# （M0 落地时创建；FORCE=1 续拉先例）。
#
# 用法：
#   bash scripts/rally-goal-47-http-cache.sh                # 推进 http-cache goal
#   bash scripts/rally-goal-47-http-cache.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/http-cache.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
