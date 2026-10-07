#!/usr/bin/env bash
# Goal: html-semantics-compat（编号 25——推进顺序见同目录 README.md；主攻大域）
# html/semantics 语义面（forms/embedded-content/tabular-data 等——form-validation
# 打过样，全语义面从未系统圈过）
# M1 资产预置：fetch html/semantics 主要子域；基线运行（runner 通道 + Makefile target
# + skip 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/html-semantics-compat.md
# DC-1。
# 注意：大域——fetch_dir_html 只递归一层，更深子域（如 forms/ 下的深层目录）按
# web-animations 先例在 M1 rally 轮以显式 SUBDIRS 追加；渲染差异面（table 布局等）
# 归 rendering-compat，本 goal 只做 DOM/语义/API 面。
# 用法：bash tests/wpt-runner/scripts/goals/25-html-semantics-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="html-semantics-compat"
DIRS=( "html/semantics/forms"
       "html/semantics/embedded-content"
       "html/semantics/document-metadata"
       "html/semantics/grouping-content"
       "html/semantics/text-level-semantics"
       "html/semantics/tabular-data"
       "html/semantics/interactive-elements"
       "html/semantics/scripting-1"
       "html/semantics/links"
       "html/semantics/microdata" )

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
