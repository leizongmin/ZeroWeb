#!/usr/bin/env bash
# Goal: html-semantics-compat（编号 25——推进顺序见同目录 README.md；主攻大域）
# html/semantics 语义面（forms/embedded-content/tabular-data 等——form-validation
# 打过样，全语义面从未系统圈过）
# M1 资产预置：fetch html/semantics 主要子域；基线运行（runner 通道 + Makefile target
# + skip 规则）= 各 goal M1 rally 轮落地，契约见 docs/goal/html-semantics-compat.md
# DC-1。
# 域边界（goal 契约）：只做 DOM/语义/API 面；渲染差异归 rendering-compat。
# 排除面（①段叶子清单不列；②段顶层递归仍会把下一级 excluded 目录带落盘——
# gitignored 不入库，零执行记账由 runner skip 规则兜底）：
#   - media-elements/（媒体元素 corpus 已由归档 media goal 通道资产化，209 案 640/653
#     收口账在 docs/compat/trends/wpt-suites.csv——防重复计账，本通道零执行记账）
#   - the-iframe-element/ + the-frame-element/（frame 树深结构面挂 navigation-compat
#     M3 门控；其通道 M1 已资产化 the-iframe-element 并零执行记账）
#   - the-template-element/（web-components 通道 corpus：WC_TEST_SUBDIRS 第三目录）
#   - crashtests/ bfcache/（崩溃面 / 多文档回退缓存面——runner 单文档通道外）
#   - microdata/：pin 31597693 上游不存在（html/microdata 已移除，404 实测 2026-10-15）
#     ——立项清单十域之一记空域，M1 盘点记账，不列目录。
# 注意：大域——fetch_dir_html 只递归一层；①段叶子目录拉各子目录 support/resources
# 助资产（缺资产会假 Fail "load support"）。叶子段必须排在②段顶层域**之前**：lib
# 幂等快路径按「目录已含 .html」跳过整目录列目录，顶层域先跑会把叶子 .html 拉齐，
# 叶子的助资产列目录从此被跳过。
# 用法：bash tests/wpt-runner/scripts/goals/25-html-semantics-compat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="html-semantics-compat"
DIRS=(
  # ① 叶子目录：support/resources 助资产 + the-script-element depth-3 用例目录
  # forms 各子目录（约 22 个）
  "html/semantics/forms/attributes-common-to-form-controls"
  "html/semantics/forms/constraints"
  "html/semantics/forms/customizable-combobox"
  "html/semantics/forms/form-control-infrastructure"
  "html/semantics/forms/form-submission-0"
  "html/semantics/forms/form-submission-target"
  "html/semantics/forms/resetting-a-form"
  "html/semantics/forms/textfieldselection"
  "html/semantics/forms/the-button-element"
  "html/semantics/forms/the-datalist-element"
  "html/semantics/forms/the-fieldset-element"
  "html/semantics/forms/the-form-element"
  "html/semantics/forms/the-input-element"
  "html/semantics/forms/the-label-element"
  "html/semantics/forms/the-legend-element"
  "html/semantics/forms/the-meter-element"
  "html/semantics/forms/the-optgroup-element"
  "html/semantics/forms/the-option-element"
  "html/semantics/forms/the-output-element"
  "html/semantics/forms/the-progress-element"
  "html/semantics/forms/the-select-element"
  "html/semantics/forms/the-textarea-element"
  # embedded-content（排除面外八叶）
  "html/semantics/embedded-content/image-maps"
  "html/semantics/embedded-content/the-area-element"
  "html/semantics/embedded-content/the-audio-element"
  "html/semantics/embedded-content/the-canvas-element"
  "html/semantics/embedded-content/the-embed-element"
  "html/semantics/embedded-content/the-img-element"
  "html/semantics/embedded-content/the-object-element"
  "html/semantics/embedded-content/the-video-element"
  # document-metadata 七叶
  "html/semantics/document-metadata/interactions-of-styling-and-scripting"
  "html/semantics/document-metadata/styling"
  "html/semantics/document-metadata/the-base-element"
  "html/semantics/document-metadata/the-link-element"
  "html/semantics/document-metadata/the-meta-element"
  "html/semantics/document-metadata/the-style-element"
  "html/semantics/document-metadata/the-title-element"
  # grouping-content 十二叶
  "html/semantics/grouping-content/the-dd-element"
  "html/semantics/grouping-content/the-div-element"
  "html/semantics/grouping-content/the-dl-element"
  "html/semantics/grouping-content/the-dt-element"
  "html/semantics/grouping-content/the-figcaption-element"
  "html/semantics/grouping-content/the-figure-element"
  "html/semantics/grouping-content/the-hr-element"
  "html/semantics/grouping-content/the-li-element"
  "html/semantics/grouping-content/the-ol-element"
  "html/semantics/grouping-content/the-p-element"
  "html/semantics/grouping-content/the-pre-element"
  "html/semantics/grouping-content/the-ul-element"
  # text-level-semantics 八叶（crashtests 不拉）
  "html/semantics/text-level-semantics/the-a-element"
  "html/semantics/text-level-semantics/the-b-element"
  "html/semantics/text-level-semantics/the-bdi-element"
  "html/semantics/text-level-semantics/the-bdo-element"
  "html/semantics/text-level-semantics/the-br-element"
  "html/semantics/text-level-semantics/the-ruby-element"
  "html/semantics/text-level-semantics/the-time-element"
  "html/semantics/text-level-semantics/the-wbr-element"
  # tabular-data 八叶
  "html/semantics/tabular-data/attributes-common-to-td-and-th-elements"
  "html/semantics/tabular-data/processing-model-1"
  "html/semantics/tabular-data/the-caption-element"
  "html/semantics/tabular-data/the-table-element"
  "html/semantics/tabular-data/the-tbody-element"
  "html/semantics/tabular-data/the-tfoot-element"
  "html/semantics/tabular-data/the-thead-element"
  "html/semantics/tabular-data/the-tr-element"
  # interactive-elements 四叶
  "html/semantics/interactive-elements/commands"
  "html/semantics/interactive-elements/the-details-element"
  "html/semantics/interactive-elements/the-dialog-element"
  "html/semantics/interactive-elements/the-summary-element"
  # scripting-1：the-script-element 用例在 depth-3 子目录，显式列叶子
  "html/semantics/scripting-1/the-noscript-element"
  "html/semantics/scripting-1/the-script-element"
  "html/semantics/scripting-1/the-script-element/css-module"
  "html/semantics/scripting-1/the-script-element/execution-timing"
  "html/semantics/scripting-1/the-script-element/fetch-src"
  "html/semantics/scripting-1/the-script-element/import-attributes"
  "html/semantics/scripting-1/the-script-element/json-module"
  "html/semantics/scripting-1/the-script-element/microtasks"
  "html/semantics/scripting-1/the-script-element/module"
  "html/semantics/scripting-1/the-script-element/moving-between-documents"
  "html/semantics/scripting-1/the-script-element/text-module"
  # links 五叶
  "html/semantics/links/downloading-resources"
  "html/semantics/links/following-hyperlinks"
  "html/semantics/links/hyperlink-auditing"
  "html/semantics/links/links-created-by-a-and-area-elements"
  "html/semantics/links/linktypes"
  # ② 九域顶层（递归一层补域顶散案 *.html——historical.html 等与 depth-1 子目录
  #   用例；上列叶子已拉过的目录此处幂等快路径跳过）
  "html/semantics/forms"
  "html/semantics/embedded-content"
  "html/semantics/document-metadata"
  "html/semantics/grouping-content"
  "html/semantics/text-level-semantics"
  "html/semantics/tabular-data"
  "html/semantics/interactive-elements"
  "html/semantics/scripting-1"
  "html/semantics/links"
)

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
