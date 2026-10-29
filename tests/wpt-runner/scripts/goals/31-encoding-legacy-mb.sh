#!/usr/bin/env bash
# Goal: encoding-legacy-mb（编号 31——推进顺序见同目录 README.md）
# 解锁 legacy-mb 文档解码面（coverage-2026-10-27 对账：全量分母 50% 所在）。
#
# 与 30-encoding-compat 的划界：30 已收口 JS API 面（TextDecoder/Encoder .any.js）
# 并预置了 legacy-mb 七编码目录语料（253 文件，decode 表参照用途）；本 goal 的
# 测试面是 legacy-mb `*_chars-<label>.html` 文档解码族——用例自带
# `<meta charset="<label>">`，要求 runner 文档加载管道按声明编码解码字节流。
# 该管道 M0 可行性未验证（runner 现按 UTF-8 加载文档），是 goal 第一关，契约见
# docs/goal/archive/encoding-legacy-mb.md DC-1；不可行则按契约挂账重估。
# （goal 已收口归档 2026-10-29——本脚本保留作语料续拉/盘点。）
#
# 语料现状（30 号预置）：euc-jp 36 / iso-2022-jp 31 / shift_jis 60 / euc-kr 66 /
# gbk 2 / gb18030 3 / big5 55 文件，为子集（上游全量 ~1,200 .html）。本脚本
# GOAL_FETCH_DEEP=1 续拉补全；因目录已含 .html 会被幂等快路径跳过，补全首轮需：
#   FORCE=1 bash tests/wpt-runner/scripts/goals/31-encoding-legacy-mb.sh
# （FORCE=1 重列目录但 fetch_raw 只下缺失文件；日常盘点不带 FORCE。）
# 30 号显式不拉面继续不拉（form-encode 族、.py 端点、.headers 语义 sidecar 除外
# 全拉——document 管道若支持 HTTP charset 头优先，sidecar 是用例依赖）。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="encoding-legacy-mb"
GOAL_FETCH_DEEP=1
DIRS=(
  "encoding/legacy-mb-japanese/euc-jp"
  "encoding/legacy-mb-japanese/iso-2022-jp"
  "encoding/legacy-mb-japanese/shift_jis"
  "encoding/legacy-mb-korean/euc-kr"
  "encoding/legacy-mb-schinese/gbk"
  "encoding/legacy-mb-schinese/gb18030"
  "encoding/legacy-mb-tchinese/big5"
)

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
