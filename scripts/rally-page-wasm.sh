#!/usr/bin/env bash
# 用 rally run 持续推进 page-wasm goal（docs/goal/page-wasm.md）。
#
# 用途：长期无人值守推进页面 WASM 深化目标（WasmValue 全类型映射 + 导出面真实化 +
# 实例化语义 + WPT jsapi 基线），agent-command 用 claude-glm（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-page-wasm.sh                # 推进 page-wasm goal
#   bash scripts/rally-page-wasm.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/page-wasm.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
