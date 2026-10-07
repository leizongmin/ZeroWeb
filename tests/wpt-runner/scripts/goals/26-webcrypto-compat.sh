#!/usr/bin/env bash
# Goal: webcrypto-compat（编号 26——推进顺序见同目录 README.md）
# WebCrypto API 面（getRandomValues/randomUUID 已实现于 part01b；subtle 全系待实现）
# M1 资产预置：fetch WebCryptoAPI 子集；基线运行（runner 通道 + Makefile target + skip
# 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/webcrypto-compat.md DC-1。
# 注意：subtle（digest/importKey/encrypt/decrypt）为实现轮——Rust crypto 依赖选型
# 须落 evidence/（复用既有依赖优先，参照 compression-compat 纪律）。
# 用法：bash tests/wpt-runner/scripts/goals/26-webcrypto-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="webcrypto-compat"
DIRS=( "WebCryptoAPI" )

GOAL_PULL_ANY_JS=1
goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
