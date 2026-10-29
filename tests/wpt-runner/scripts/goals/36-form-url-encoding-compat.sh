#!/usr/bin/env bash
# Goal: form-url-encoding-compat（编号 36——推进顺序见同目录 README.md）
# legacy-mb encode 面：form 提交编码序列化 + href URL encoding-parse
# （31 号 encoding-legacy-mb M3 甄别归口，2026-10-29 立项）。
#
# 测试面：`*-encode-form-*.html`（GET + acceptCharset=<meta charset> +
# target=iframe，断言 query 出非 UTF-8 字节序列）与 `*-encode-href-*.html`
# / `*-encoder.html`（a.href 以文档编码 fallback 解析，读 a.search 比对）。
# 契约见 docs/goal/archive/form-url-encoding-compat.md（form 面缺 acceptCharset
# 序列化消费，href 面缺 encoding-parse a URL——M0/M1/M2 逐面解锁）。
#
# 语料现状：68 文件已全量在位（31 号 DC-3 对账上游 pin 315976933
# missing=0；此前「31 号不拉 form-encode 族」的注释为误记——GOAL_FETCH_DEEP
# 深拉已随目录带齐）。本脚本仅作幂等盘点与缺口校验，无 fetch 依赖；
# 日常盘点不带 FORCE。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="form-url-encoding-compat"
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
