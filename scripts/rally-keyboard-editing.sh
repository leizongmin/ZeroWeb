#!/usr/bin/env bash
# 用 rally run 持续推进键盘/编辑方向三个 goal（html-compat 编辑面三拆）：
#   - docs/goal/editing-contenteditable.md     编辑与 contenteditable（选区/键入/execCommand）
#   - docs/goal/keyboard-default-actions.md    键盘默认动作（Enter 提交/空格激活/Esc/select 导航）
#   - docs/goal/keyboard-page-scrolling.md     键盘页面滚动（PageUp/Space/Home/End/方向键滚动）
#
# 用途：长期无人值守推进键盘与编辑交互目标（WPT 驱动，三个入口文档按轮次
# 顺序推进），agent-command 用 claude-glm（GLM-5.1，bigmodel 通道）。
#
# 用法：
#   bash scripts/rally-keyboard-editing.sh                # 推进三个 goal
#   bash scripts/rally-keyboard-editing.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/editing-contenteditable.md \
    docs/goal/keyboard-default-actions.md \
    docs/goal/keyboard-page-scrolling.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
