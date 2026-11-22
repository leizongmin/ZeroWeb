#!/usr/bin/env bash
# Goal: wptfyi-html（docs/goal/wptfyi-html.md，五目标计划 03 号）DC-2 批九：
# html/infrastructure + 小簇搭车（obsolete/links/meta/scripting/select）语料——
# 11-20 快照 covered_files=0 的未接入簇（html/interaction 另簇不在本批），
# den 193 键 / 1,736 子测试 / chrome 过 1,636（summary-chrome @4b26db8978：
# infrastructure 160/1,664/1,566 + obsolete 20/55/54 + links 7/5/4 + meta 1/3/3 +
# scripting 3/4/4 + select 2/5/5）。
#
# 计分面：目录内 testharness 形 .html 页 + .window.js/.any.js window 变体源
# （生成形分母键 15 + 8）；infrastructure 变体键 91 个（urls/resolving-urls/
# query-encoding/ 9 基页 `<meta name="variant">` 矩阵：?include=<面> ×80 +
# ?encoding=<enc> ×11 文档编码切换面），runner 侧 case_variants 展开 +
# 主文档 {{GET[...]}} 模板替换（testharness_html_infra.rs 模块注记）。
# 零执行记账面同 rendering 通道：support/resources/crashtests 助资产、-ref/-notref
# 参照页、.xhtml 真文档（historical.sub.xhtml，den 20 子测试）、manual 页、
# 无 testharness 引用页。
#
# 拉取面：26 目录显式列出（fetch_dir_html 幂等标记防 API 限流重列；.headers
# sidecar 走 lib.sh 扩展名分支自动拉取——structured-data 响应头面；.window.js
# 走 .js 扩展名分支自动拉取）；根级散文件显式 fetch_raw（meta/refresh-time.html +
# select 3 文件——select 无子目录）；GOAL_PULL_ANY_JS=1——safe-passing-of-structured-data
# 4 个 .any.js 源唯一形态（messagechannel / no-coop-coep.https / serialization-via-idb /
# serialization-via-notifications-api，生成形分母 8 键 306 子测试，批四/批五先例）；
# 非 fetch_dir_html 扩展名的测试资产显式拉取：manifest .webmanifest ×6（links
# manifest MIME 面被测对象）、plugins sample.txt（object data 面）、stylesheet
# quirk-stylesheet.css.txt（quirk 载入面）、obsolete original-id.json（document-all
# 断言面）。.py（wptserve 端点）/WEB_FEATURES.yml/META.yml 不拉——runner 通道零消费。
# 用法：bash tests/wpt-runner/scripts/goals/62-wptfyi-html-infra.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="wptfyi-html"
GOAL_PULL_ANY_JS=1
FAILED_FETCHES=0

for dir in \
  html/infrastructure/common-dom-interfaces/collections \
  html/infrastructure/common-microsyntaxes/colours \
  html/infrastructure/conformance-requirements/extensibility \
  html/infrastructure/fetching-resources \
  html/infrastructure/safe-passing-of-structured-data \
  html/infrastructure/safe-passing-of-structured-data/resources \
  html/infrastructure/safe-passing-of-structured-data/shared-array-buffers \
  html/infrastructure/safe-passing-of-structured-data/shared-array-buffers/resources \
  html/infrastructure/terminology/plugins \
  html/infrastructure/urls/base-url \
  html/infrastructure/urls/base-url/resources \
  html/infrastructure/urls/dynamic-changes-to-base-urls \
  html/infrastructure/urls/resolving-urls/query-encoding \
  html/infrastructure/urls/resolving-urls/query-encoding/resources \
  html/infrastructure/urls/terminology-0 \
  html/infrastructure/urls/terminology-0/resources \
  html/links/icon \
  html/links/manifest \
  html/links/manifest/link-relationship \
  html/links/stylesheet \
  html/links/stylesheet/resources \
  html/meta/resources \
  html/obsolete/requirements-for-implementations/other-elements-attributes-and-apis \
  html/obsolete/requirements-for-implementations/the-marquee-element-0 \
  html/scripting/the-script-element \
  html/scripting/the-script-element/resources \
  ; do
  fetch_dir_html "${dir}"
done

# 根级散文件（无子目录簇：meta 主页 + select 全部；crashtests 目录不拉——零执行记账面）
for file in \
  html/meta/refresh-time.html \
  html/select/options-length-too-large.html \
  html/select/select-capitalize-sizing-ref.html \
  html/select/select-capitalize-sizing.html \
  html/links/manifest/mime-type-application-json.webmanifest \
  html/links/manifest/mime-type-application-manifest+json.webmanifest \
  html/links/manifest/mime-type-invalid.webmanifest \
  html/links/manifest/mime-type-none.webmanifest \
  html/links/manifest/mime-type-text-json.webmanifest \
  html/links/manifest/wrong-mime-type-text-plain.webmanifest \
  html/infrastructure/terminology/plugins/sample.txt \
  html/links/stylesheet/resources/quirk-stylesheet.css.txt \
  html/obsolete/requirements-for-implementations/other-elements-attributes-and-apis/original-id.json \
  ; do
  fetch_raw "${file}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
done

# query-encoding resource.py png 面资产（images/ 在 html/ 子树外；green-2x2.png
# 为 q=%C3%A5 臂映射、未随先期 goal 入盘——green-1x1/16x16/256x256 已在盘）。
fetch_raw "images/green-2x2.png" || FAILED_FETCHES=$((FAILED_FETCHES + 1))

# query-encoding resource.py video 面资产（media/ 在 html/ 子树外；5 文件为
# q 臂映射缺档补齐——A4/movie_5/movie_300 .webm 已在盘；上游 pin 无 .ogv
# 档，ogv MIME 改名臂为上游死分支，resolve-url.js canPlayType 仅取 webm/mp4）。
for media in green-at-15.webm A4.mp4 movie_5.mp4 green-at-15.mp4 movie_300.mp4; do
  fetch_raw "media/${media}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
done

# 批十四：suite 外支撑脚本补档。CI 周跑按分片只跑本 goal 的 fetch（本地这些
# 文件由 workers/html-syntax/encoding 等其他 goal 通道带入，分片作业拿不到），
# 缺档形＝structured-clone 大簇 load support 报 os error 2（messagechannel/
# window-postmessage/SAB 族，本地 266+152 子测试面）＋ query-encoding 变体页
# subsetTestByKey 未定义 → 16 include × 5 页 0 注册 TO（CI 11-21 快照
# html/infrastructure zw_passes 82 vs 本地 674 的主构成）。META script 链
# 传递闭包扫描实证 9 文件；另补 links/icon no-load-event.svg（本地同缺、
# testharness 页）。
for support in \
  common/sab.js \
  common/utils.js \
  common/get-host-info.sub.js \
  common/dispatcher/dispatcher.js \
  common/subset-tests-by-key.js \
  IndexedDB/resources/support.js \
  html/webappapis/structured-clone/structured-clone-battery-of-tests.js \
  html/webappapis/structured-clone/structured-clone-battery-of-tests-with-transferables.js \
  html/webappapis/structured-clone/structured-clone-battery-of-tests-harness.js \
  images/wpt-logo/wpt-logo-darkblue-bg.svg \
  ; do
  fetch_raw "${support}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
done

echo "== infrastructure+小簇拉取完成（失败 ${FAILED_FETCHES}，pin ${WPT_REV:0:9}）=="
for top in html/infrastructure html/links html/meta html/obsolete html/scripting html/select; do
  printf "  %s: %s 案\n" "${top}" "$(find "${WPT_DATA}/${top}" -name '*.html' 2>/dev/null | wc -l)"
done
