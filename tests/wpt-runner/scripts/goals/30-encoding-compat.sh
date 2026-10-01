#!/usr/bin/env bash
# Goal: encoding-compat（编号 30——推进顺序见同目录 README.md）
# 小快赢（legacy 编码标签表）
# M1 资产预置：fetch encoding corpus；基线运行（runner 通道 + Makefile target + skip
# 规则）= M1 rally 轮落地，契约见 docs/goal/encoding-compat.md DC-1。
# 用法：bash tests/wpt-runner/scripts/goals/30-encoding-compat.sh（FORCE=1 强制重拉）
#
# **拉取面（2026-10-01 M1 rally 轮定稿，照 20-net-api-compat.sh 先例显式列目录）**：
# JS API 面（textdecoder/textencoder/api-*/encodeInto .any.js 为主形态，GOAL_PULL_ANY_JS=1）
# + encoding/streams（TextDecoderStream/TextEncoderStream 变体）+ legacy-mb 七编码资产
# （decode 表参照——jis0208/euc-kr/gbk/gb18030/big5 index .js 与 decoder helper，M2
# 解码表数据化的事实源）+ encoding/resources（decode-common/encodings.js 等 META
# script 依赖，运行面 skip 规则排除，双保险）。
#
# **显式不拉/不跑（记账，重入条件见括号）**：
# - 文档级编码嗅探面（bom-handling/eof-*/utf-32*/remove-only-one-bom/sniffing/
#   resources/*.html 子文档——`<meta charset`/BOM 嗅探 → 文档解码，html-syntax-compat
#   划界挂账，M4 定稿；运行面 encoding_case_skipped 排除）
# - form 提交编码面（legacy-mb `*-encode-form-*.html` + big5-encoder/iso-2022-jp-encoder
#   的 <a href> URL 序列化——document 编码耦合，非 TextEncoder 面；运行面排除）
# - legacy-mb decode 族 .html（iframe 子文档解码面——runner 无多 frame 文档管道；
#   net-api 先例。index .js/decoder helper 照拉，M2 数据化参照）
# - .py 服务器端点（single-byte-raw/text-html-meta-charset/text-plain-charset——
#   lib.sh 拉取面不含 .py）+ *.headers（runner 不消费）
# - idlharness.any.js（/resources/WebIDLParser.js build 期生成资产，repo 内不存在）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="encoding-compat"
GOAL_PULL_ANY_JS=1
DIRS=(
  "encoding"
  "encoding/resources"
  "encoding/streams"
  "encoding/streams/resources"
  "encoding/legacy-mb-japanese/euc-jp"
  "encoding/legacy-mb-japanese/iso-2022-jp"
  "encoding/legacy-mb-japanese/shift_jis"
  "encoding/legacy-mb-korean/euc-kr"
  "encoding/legacy-mb-schinese/gbk"
  "encoding/legacy-mb-schinese/gb18030"
  "encoding/legacy-mb-tchinese/big5"
)

goals_fetch_all
# /common 数据依赖显式补拉（fetch-media-subset.sh 先例）：encodeInto.any.js 与
# encoding/streams/decode-utf8.any.js 经 `META: script=/common/sab.js` 消费。
fetch_raw "common/sab.js"
goals_inventory
goals_next_steps "${GOAL}"
