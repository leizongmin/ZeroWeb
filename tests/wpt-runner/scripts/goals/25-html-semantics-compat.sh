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

# ③ 跨目录助资产（R4300）：the-dialog-element/the-details-element 语料以相对或
# `/` 根路径引用域外 resources——fetch_dir_html 只拉目录内文件，缺资产时相对
# 路径 script 标签被移除（inline_local_scripts 缺文件即删）→ clickOn/sendEscKey/
# blessTopLayer/createToggleEventSourceTest/waitUntilStableAutofocusState
# ReferenceError 假 Fail。清单=dialog closedby/canceling/toggle-source/autofocus
# 语料实际引用面。
fetch_raw "common/top-layer.js"
fetch_raw "close-watcher/resources/helpers.js"
fetch_raw "html/interaction/focus/the-autofocus-attribute/resources/utils.js"
fetch_raw "html/semantics/popovers/resources/popover-utils.js"
fetch_raw "html/semantics/popovers/resources/toggle-event-source-test.js"
# s404：顶层 /resources/ 助资产——object/embed-javascript-url 双案
# insertObjectNavigable 以 data|src=/resources/blank.html 导航（语料 resources/
# 仅 testharness 基建 5 件，blank.html 缺 404 → error 代替 load → await onload
# 恒挂 → 文件级 Timeout，s227 同族勘根）。
fetch_raw "resources/blank.html"

# ③b css/text 数据模块语料资源（s108 待决①销账，用户授权 2026-10-31）：css-module/
# text-module 用例引用面。缺口机制 = M1 首拉（2026-10-15）时扩展白名单尚无 .css
#（2026-10-29 才加）且 fetch_dir_html 的 .txt 为 opt-in，此后 css-module/ 已含
# .html 被幂等快路径整目录跳过——缺口固化（s158 补 link-rel-attribute.css 同坑）。
# not_found.css / not_found.txt 为负向用例，必须继续缺席，不拉。
# charset 家族资产（s264 补入）：css-module/charset{,-2,-bom}.html、
# text-module/charset{,-2}.html 五页 file-Timeout 勘根均为资产缺席——旧注记
# 「utf-8.css / utf-8.txt 仅经 serve-with-content-type.py?fn= 引用，runner 未
# 实现该端点，落地时再补」；端点已于 s115/s249/s250 落地，此销。字节源必须
# lib.sh WPT_REV pin 版（fetch_raw 同 pin）：上游 master 已重编 bom-utf-16be.css
# 为 UTF-8 BOM+纯文本，与 pin 版（真 UTF-16BE）不符；windows-1250.* pin 版本即
# 含 U+FFFD 序列，corpus 判据按该字节形写定。
# 基准变更注记：补语料后 5 页 Timeout→起跑（12 子测 Pass；charset-bom 残 2F =
# shim CSSOM 解析缺 CSS Syntax §3.3 NUL→FFFD 预处理，引擎面下切收口），下轮
# 全量 join 的对应 GAINED 按语料变更归因，非回归。
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/basic.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/basic-large.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/bad-import.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/malformed.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/parse-error.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/load-relative-url.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/atImported.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/utf-8.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/windows-1250.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/bom-utf-8.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/bom-utf-16be.css"
fetch_raw "html/semantics/scripting-1/the-script-element/css-module/resources/bom-utf-16le.css"
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/file.txt"
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/utf-8.txt"
fetch_raw "html/semantics/scripting-1/the-script-element/text-module/windows-1250.txt"

# ③c the-img-element 助资产（s225）：image-base-url 语料经 <base href="resources/">
# 重写引用 resources/green.png——重写后路径落在域内 resources/，缺席即假 Fail
# "The image must load"。utf-8.css 同坑变体（幂等快路径整目录跳过后补清单）。
fetch_raw "html/semantics/embedded-content/the-img-element/resources/green.png"

# ③d import-meta resolve-importmap 语料资源（s254）：import-meta-resolve-importmap.html
# 的链式 map 用例第四键 fetch 面（import("./start") → export-1.mjs，import(resolve)
# → export-2.mjs）——M1 首拉无 resources/ 子目录（同 ③b 幂等快路径固化坑）。
# 基准变更注记：补语料后 import-meta 引用面预期 F→P 位移，下轮全量 join 的对应
# GAINED 按语料变更归因，非回归。
fetch_raw "html/semantics/scripting-1/the-script-element/module/import-meta/resources/export-1.mjs"
fetch_raw "html/semantics/scripting-1/the-script-element/module/import-meta/resources/export-2.mjs"

# ③e the-img-element sizes/srcset 族助资产（s282）：sizes-auto.html 六子测断言
# 面引用 /images/green-1x1.png?…、/images/green-16x16.png?…（404 → onerror →
# 断言永假 6F）；implicit-sizes-ignores-width.html 引用
# srcset/resources/image.png（404 → onerror，真断言面被加载错误遮蔽）。
# 基准变更注记：补语料后 sizes-auto 六子测预期 F→P、resize-observer-removes-auto
# 预期文件级 Timeout→子测起跑（真面 ResizeObserver 首报，转候选），下轮全量
# join 的对应位移按语料变更归因，非回归（2026-11-04 s282 账本 A/B 实证）。
fetch_raw "images/green-1x1.png"
fetch_raw "images/green-16x16.png"
fetch_raw "html/semantics/embedded-content/the-img-element/srcset/resources/image.png"
# ③f the-img-element sizes-auto-dynamic-attributes 助资产（s284）：该页断言面
# 引用 /images/red-16x16.png（404 → onerror → load 断言永假 3F）。
# 基准变更注记：补语料后该页 3F 预期转子测真面（sizes 重解析/eager flip/
# stale auto-width 非持久），下轮全量 join 对应位移按语料变更归因（s284 账本）。
fetch_raw "images/red-16x16.png"

# ③g the-img-element 端点仿真字节出处助资产（s295）：image-alternate-no-store.py /
# image-and-stash.py 两端点不在语料（runner 臂内嵌字节回体，s295 账本）——语料页只
# 引用 .py 不引用这些 .png，运行时不消费；此处拉取仅为 testharness.rs 常量字节出处
# 可复验（fresh clone 可对上游 pin 逐字节 cmp）。green.png 已在 ③c、image.png 以
# resources/（③c 同目录）与 srcset/resources/ 两副本在本地；顶层
# the-img-element/image.png 是**另一路径**（update-src-complete.html 相对引用面，
# 运行时消费，③i 补），勿与两副本混读。仅 blue-10.png 缺席。
fetch_raw "html/semantics/embedded-content/the-img-element/resources/blue-10.png"
# ③h the-img-element 运行时消费缺资产（s295）：not-rendered-dimension-getter.html
# 引用 image-1.jpg（389×590，IDL width/height 未渲染回落自然维断言面）；natural-
# size-orientation.html 引用 images/arrow-oriented-upright.jpg（EXIF upright 方向
# 元数据，naturalWidth/Height 重定向值断言面）。缺资产 404 → onerror → load 断言
# 永不到 → 文件级 Timeout（s227 先验判据）。基准变更注记：补语料后两页预期
# Timeout → 子测真面，下轮全量 join 对应位移按语料变更归因，非回归（s295 账本）。
fetch_raw "html/semantics/embedded-content/the-img-element/image-1.jpg"
fetch_raw "images/arrow-oriented-upright.jpg"

# ③i the-img-element 运行时消费缺资产（s295）：update-src-complete.html 两段引用
# 顶层 image.png（`<img src="image.png">` + 换源 `image.png?<random>`）。缺资产时
# 两段均 404 → decode 失败 → error：首载 resourceState=broken 使 complete getter
# 假绿（assert_true 过）、二次换源派 error 非 load → 新 onload 永不到 → 文件级
# Timeout（s227 同型——勘根先绕 shim 全链后验资产，教训录账本）。
# 基准变更注记：补语料后该页预期 Timeout → 子测起跑（complete 翻转 + 二次
# onload 两断言面，勘根确认 shim restart/fetch 链健全），下轮全量 join 对应
# 位移按语料变更归因，非回归（s295 账本）。
fetch_raw "html/semantics/embedded-content/the-img-element/image.png"

goals_fetch_all
goals_inventory
goals_next_steps "${GOAL}"
