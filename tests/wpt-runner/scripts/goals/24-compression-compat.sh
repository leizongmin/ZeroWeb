#!/usr/bin/env bash
# Goal: compression-compat（编号 24——推进顺序见同目录 README.md）
# CompressionStream/DecompressionStream 面（streams 底座为 net-api M4 修齐的遗产）
# M1 资产预置：fetch compression 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/compression-compat.md DC-1。
# 注意：API 缺失时基线即 0%——实现型小套件（非纯语义修齐），M2 为实现轮。
# 用法：bash tests/wpt-runner/scripts/goals/24-compression-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="compression-compat"
DIRS=( "compression" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
# 用例运行依赖（fetch_dir_html 子目录只递归一层，third_party/pako 在两层深处）：
# - compression/third_party/pako/pako_inflate.min.js——compression-stream /
#   multiple-chunks / including-empty-chunk / large-flush-output 经
#   resources/decompress.js 用 pako 交叉解压实现输出（pako_inflate.min.js 22.7KB）。
# - media/foo.vtt + media/test-av-384k-*.webm——compression-stream / output-length
#   的 fetch('/media/...') 输入（空数据外的另两档数据量；单文件 fetch 不整拉 media/）。
fetch_raw "compression/third_party/pako/pako_inflate.min.js"
fetch_raw "media/foo.vtt"
fetch_raw "media/test-av-384k-44100Hz-1ch-320x240-30fps-10kfr.webm"
goals_inventory
goals_next_steps "${GOAL}"
