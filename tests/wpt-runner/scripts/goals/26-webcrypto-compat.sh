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
# 语料埋在二三层子目录（sign_verify/ import_export/ encrypt_decrypt/…），且
# 未认证 Contents API 逐目录列不完——走 git trees 深挖（lib.sh fetch_dir_deep）。
GOAL_FETCH_DEEP=1
goals_fetch_all
# 跨域 /common 依赖（用例 META 引用，不在 DIRS 内，单独补拉）；
# /resources/idlharness.js + WebIDLParser.js 不拉——idlharness 用例 runner 侧记账排除
#（compression 先例，WebIDLParser build 期资产 repo 内不可执行）。
fetch_raw "common/subset-tests.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "common/gc.js" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
goals_inventory
goals_next_steps "${GOAL}"
