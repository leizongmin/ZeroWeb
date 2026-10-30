#!/usr/bin/env bash
# Goal: encoding-compat-v2（编号 37——推进顺序见同目录 README.md）
# encoding 二期：根面 labels/API 残余 + streams 收口（2026-10-31 立项，
# coverage-2026-10-30 对账驱动；契约见 docs/goal/encoding-compat-v2.md）。
#
# 测试面（M0 对账甄别定稿，见 docs/goal/encoding-compat-v2/evidence/）：
# - encoding 根面 .any.js API 窗口腿（labels 列表 / TextDecoder/TextEncoder /
#   encodeInto / 异常分支——30 号通道已收 21 案窗口子集，本 goal 补拉上游滚动
#   新增面）与 .window.js 形态（unsupported-labels / single-byte-decoder-iframe，
#   webcrypto `.window.js` 通道先例）
# - encoding/streams 残余（37 文件：11 .any.js 窗口腿 + realms/invalid-realm
#   window 形态）
#
# 划界（M0 甄别记账，逐族 disposition）：
# - legacy-mb 三目录+schinese 无未导入案面：decode 族 34 文件 360,431（31 号
#   100%P）+ encode 族 68 文件 766,959（36 号 99.9999%）+ gbk/gb18030-decoder
#   .any 窗口腿（30 号通道在收）= 上游 1,206 文件全对账闭合；coverage-2026-10-30
#   缺口 573,911 系 CI form-encode 分片超时假低（2167d9ca6/3be29eb75 已修），
#   真残余 = worker 腿 714（挂账）
# - worker/serviceworker/sharedworker 腿（根面 ~8.8k + streams ~1.5k + legacy 714
#   子测试）——runner 无 SW/shared 桥；dedicated 腿 re-entry 见 goal 挂账清单
# - 文档级嗅探面（bom-handling/sniffing/utf-32*/remove-only-one-bom/eof-*/
#   big5-encoder/iso-2022-jp-encoder/json-document-utf8）——30 号 M4 划界挂账
#   html-syntax-compat，本 goal 不重开
# - idlharness.any.js（WebIDLParser build 期生成资产）——net-api/timing 先例挂账
#
# 与 30 号划界：30 号通道（testharness-encoding）与语料资产不动（零回归门）；
# 本脚本只作根面/streams 语料续拉与盘点（FORCE=1 对齐 pin 上游滚动），DC-2
# 可导入集（big5/iso-2022-jp-encoder，M0 甄别）经 36 号 form-url 通道扫描域
# 扩展接线，不新增 target。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="encoding-compat-v2"
GOAL_PULL_ANY_JS=1
DIRS=(
  "encoding"
  "encoding/resources"
  "encoding/streams"
  "encoding/streams/resources"
)

goals_fetch_all
# /common 数据依赖（30 号同款显式补拉）：encodeInto.any.js 与
# encoding/streams/decode-utf8.any.js 经 `META: script=/common/sab.js` 消费；
# api-invalid-label.any.js / textdecoder-fatal-single-byte.any.js 经
# `META: script=/common/subset-tests.js` 消费（缺失时整案塌缩为 load-support
# Fail——2026-10-19 复跑实测 3421 分母消失先例）。
fetch_raw "common/sab.js"
fetch_raw "common/subset-tests.js"
goals_inventory
goals_next_steps "${GOAL}"
