#!/usr/bin/env bash
# 媒体三 goal（media-playback / media-elements / media-audio）已于 2026-09-05 完成收口
# 并整树归档至 docs/goal/archive/（模式 A）。本启动器指向归档后的入口文档，仅在需要按
# 归档 goal 的余项挂账（H.264 分发前法务复核、Mixer N→1 桌面可选切片、stss 索引加速
# 随切片 3 评估）复开 rally 时使用；日常推进不再经过本脚本。
#   - docs/goal/archive/media-playback.md       媒体播放（已 Done，DC-1~5 ✅）
#   - docs/goal/archive/media-elements.md       HTMLMediaElement 语义面（已 Done，DC-1~4 ✅）
#   - docs/goal/archive/media-audio.md          音频（已 Done，DC-1~5 ✅）
#
# 用法：
#   bash scripts/rally-media.sh                # 推进三个媒体 goal
#   bash scripts/rally-media.sh --dry-run      # 只打印命令
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

exec rally run docs/goal/archive/media-playback.md \
    docs/goal/archive/media-elements.md \
    docs/goal/archive/media-audio.md \
    -w "$PROJECT_ROOT" \
    --agent-command claude-glm \
    "$@"
