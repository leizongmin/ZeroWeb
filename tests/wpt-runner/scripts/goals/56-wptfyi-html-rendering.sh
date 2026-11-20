#!/usr/bin/env bash
# Goal: wptfyi-html（docs/goal/wptfyi-html.md，五目标计划 03 号——html 顶级目录缺口）
# DC-2 批二：html/rendering UA 样式/呈现提示 testharness 语料（11-20 快照缺口
# 11,705 子测试 / 525 分母文件；testharness 形计分面 = 158 计分文件）。
#
# 计分机制：目录内 testharness 形 .html 页——UA 样式表/呈现提示断言主力为
# getComputedStyle 对照（support/test-ua-stylesheet.js 共享 helper，test 元素 vs
# fake clone ref 逐 CSS 属性比对）。runner 通道 testharness-html-rendering
# （testharness_html_rendering.rs）同域排除面：support/resources/reference/
# crashtests 助资产、-ref/-notref 参照页、.xhtml 真文档（无 XML 管道——xhtml-syntax
# 同款门控）、无 testharness 引用页（reftest 主体页——本口径 reftest 无子测试
# 不进分母）。
#
# 拉取面：目录清单按 pin 31597693 git trees html/rendering 子树全量枚举（51 目录
# 显式列出，fetch_dir_html 幂等标记防 API 限流重列）；5 个 .png 表格背景资产不在
# fetch_dir_html 拉取模式内，显式单文件；html/rendering 根级 5 文件不在子目录清单
# 内（dimension-attributes / pixel-length-attributes / unmapped-attributes 计分页 +
# cdata-001/002.xhtml 零执行记账面），显式单文件。.yml（元数据）/.py（wptserve
# 服务端）不拉——window 通道零消费。
# 用法：bash tests/wpt-runner/scripts/goals/56-wptfyi-html-rendering.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="wptfyi-html"
FAILED_FETCHES=0

fetch_dir_html "html/rendering/bidi-rendering"
fetch_dir_html "html/rendering/bindings/the-button-element"
fetch_dir_html "html/rendering/bindings/the-input-element-as-a-text-entry-widget"
fetch_dir_html "html/rendering/bindings/the-select-element-0"
fetch_dir_html "html/rendering/bindings/the-textarea-element-0"
fetch_dir_html "html/rendering/interactive-media/links-forms-and-navigation"
fetch_dir_html "html/rendering/non-replaced-elements"
fetch_dir_html "html/rendering/non-replaced-elements/flow-content-0"
fetch_dir_html "html/rendering/non-replaced-elements/flow-content-0/support"
fetch_dir_html "html/rendering/non-replaced-elements/form-controls"
fetch_dir_html "html/rendering/non-replaced-elements/lists"
fetch_dir_html "html/rendering/non-replaced-elements/margin-collapsing-quirks"
fetch_dir_html "html/rendering/non-replaced-elements/phrasing-content-0"
fetch_dir_html "html/rendering/non-replaced-elements/phrasing-content-0/br-wbr-content"
fetch_dir_html "html/rendering/non-replaced-elements/phrasing-content-0/font-element-text-decoration-color"
fetch_dir_html "html/rendering/non-replaced-elements/sections-and-headings"
fetch_dir_html "html/rendering/non-replaced-elements/tables"
fetch_dir_html "html/rendering/non-replaced-elements/tables/resources"
fetch_dir_html "html/rendering/non-replaced-elements/the-br-element"
fetch_dir_html "html/rendering/non-replaced-elements/the-fieldset-and-legend-elements"
fetch_dir_html "html/rendering/non-replaced-elements/the-fieldset-and-legend-elements/crashtests"
fetch_dir_html "html/rendering/non-replaced-elements/the-fieldset-and-legend-elements/resources"
fetch_dir_html "html/rendering/non-replaced-elements/the-frameset-and-frame-elements"
fetch_dir_html "html/rendering/non-replaced-elements/the-frameset-and-frame-elements/reference"
fetch_dir_html "html/rendering/non-replaced-elements/the-frameset-and-frame-elements/resources"
fetch_dir_html "html/rendering/non-replaced-elements/the-hr-element-0"
fetch_dir_html "html/rendering/non-replaced-elements/the-page"
fetch_dir_html "html/rendering/non-replaced-elements/the-page/crashtests"
fetch_dir_html "html/rendering/non-replaced-elements/the-page/support"
fetch_dir_html "html/rendering/replaced-elements/attributes-for-embedded-content-and-images"
fetch_dir_html "html/rendering/replaced-elements/attributes-for-embedded-content-and-images/resources"
fetch_dir_html "html/rendering/replaced-elements/embedded-content"
fetch_dir_html "html/rendering/replaced-elements/embedded-content-rendering-rules"
fetch_dir_html "html/rendering/replaced-elements/embedded-content/resources"
fetch_dir_html "html/rendering/replaced-elements/images"
fetch_dir_html "html/rendering/replaced-elements/resources"
fetch_dir_html "html/rendering/replaced-elements/svg-embedded-sizing"
fetch_dir_html "html/rendering/replaced-elements/svg-inline-sizing"
fetch_dir_html "html/rendering/replaced-elements/the-option-element"
fetch_dir_html "html/rendering/replaced-elements/the-select-element"
fetch_dir_html "html/rendering/replaced-elements/the-select-element/customizable-select"
fetch_dir_html "html/rendering/replaced-elements/the-textarea-element"
fetch_dir_html "html/rendering/sections"
fetch_dir_html "html/rendering/support"
fetch_dir_html "html/rendering/the-css-user-agent-style-sheet-and-presentational-hints"
fetch_dir_html "html/rendering/the-details-element"
fetch_dir_html "html/rendering/the-details-element/resources"
fetch_dir_html "html/rendering/widgets"
fetch_dir_html "html/rendering/widgets/appearance"
fetch_dir_html "html/rendering/widgets/button-layout"
fetch_dir_html "html/rendering/widgets/the-select-element"

# html/rendering 根级 5 文件（fetch_dir_html 清单只覆盖子目录，根级不在其中）
fetch_raw "html/rendering/cdata-001.xhtml" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/rendering/cdata-002.xhtml" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/rendering/dimension-attributes.html" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/rendering/pixel-length-attributes.html" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/rendering/unmapped-attributes.html" || FAILED_FETCHES=$((FAILED_FETCHES + 1))

# 5 个 .png 表格背景资产（fetch_dir_html 拉取模式不含图片，显式单文件）
fetch_raw "html/rendering/non-replaced-elements/tables/resources/aqua-yellow-32x32.png" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/rendering/non-replaced-elements/tables/resources/blue-16x20-green-16x20.png" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/rendering/non-replaced-elements/tables/resources/fuchsia-32x32.png" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/rendering/non-replaced-elements/tables/resources/red-32x32.png" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
fetch_raw "html/rendering/non-replaced-elements/tables/resources/yellow-32x32.png" || FAILED_FETCHES=$((FAILED_FETCHES + 1))

if [[ "${FAILED_FETCHES}" -gt 0 ]]; then
  echo "wptfyi-html html/rendering fetch incomplete: ${FAILED_FETCHES} failed" >&2
  exit 1
fi

echo "wptfyi-html html/rendering corpus ready (51 dirs + 5 root files + 5 png, WPT ${WPT_REV})"
