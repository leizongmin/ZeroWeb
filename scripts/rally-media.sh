#!/usr/bin/env bash
# 用 rally run 持续推进媒体方向三个 goal（media 三件套，依赖链 media-playback → media-audio）：
#   - docs/goal/media-playback.md       媒体播放（解码选型 RFC 门控 + 静音视频管线）
#   - docs/goal/media-elements.md       HTMLMediaElement 语义面（事件序列/canPlayType/track）
#   - docs/goal/media-audio.md          音频（双重门控：解码选型 + M0 音频环境验证）
#
# 用途：长期无人值守推进媒体目标。media-audio 源码改动受 media-playback M0 选型 +
# 自身 M0 音频环境双重门控，rally 遇门控项按 goal 契约记「待用户决策」并转零碰撞面
# （调研/环境探测/WPT 导入），不会卡死。
#
# 用法：
#   bash scripts/rally-media.sh                # 推进三个媒体 goal
#   bash scripts/rally-media.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/media-playback.md \
    docs/goal/media-elements.md \
    docs/goal/media-audio.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
