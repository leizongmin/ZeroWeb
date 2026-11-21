#!/usr/bin/env bash
# Goal: wptfyi-html（docs/goal/wptfyi-html.md，五目标计划 03 号）DC-2 批八：
# html/syntax 解析簇 html5lib 语料补齐——解析驱动三件套
#（html/syntax/parsing/html5lib_url.html（60 variant）/ html5lib_write.html（55）/
# html5lib_write_single.html（55），合计 170 变体键）以
# `<script src="resources/test.js" data-run-type="…">` 承载运行形态，test.js 按
# variant 查询参数 fetch(`resources/${file}.dat`) 取测试数据。本地语料此前面
# （resources/ 只有 3 个 .js，61 个 .dat 全缺）→ 170 键整批落
# 「html5lib-runner setup」synthetic Fail（runType=undefined + fetch 404 双缺口，
# 对账分母 5,417 子测试 chrome 过 5,376 未入账）。
#
# 拉取面：parsing/resources/*.dat 61 文件（pin 31597693 git trees 全集）显式
# fetch_raw——数据资产非 .html/.js 扩展，fetch_dir_html 扩展名分流不覆盖；清单
# 按 pin 固定（上游 .dat 在该 rev 冻结），逐文件幂等（-s 在盘跳过）。
# runner 侧配套：testharness.rs 投递改写其余原标签属性透传（data-run-type 面板，
# open_tag_passthrough_attrs）——驱动页 `<script data-run-type=…>` 承载的运行形态
# 此前被投递改写丢弃，runType 读空。
# pin 缺失挂账（den 4b26db8978 分母有、pin 315976933 git trees 无此文件，实证
# 26 键 53 子测试 chrome 过 49）：parsing/adoption-agency-reparenting 族 7 +
# custom-element-reparenting 族 3 + crashtests 3（0 子测试）+ foster-parenting
# template 面 3 + parsing 顶层散键 6 + parsing-html-fragments 3 +
# parsing-xml-fragments 1——上游晚于 pin 的新增面，按批六先例（语料单 pin 不混版）
# 不拉，随周跑另报。html5lib 驱动变体侧另有 3 键 scripted_foster01（url/write/
# write_single）的 .dat 晚于 pin 不在本地清单，驱动页 variant 表即 pin 侧证据；
# sanitizer-api/html5lib-basics.html 属 sanitizer 域非本通道扫描面。
# 用法：bash tests/wpt-runner/scripts/goals/61-wptfyi-html-syntax-dat.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="wptfyi-html"
FAILED_FETCHES=0

RES="html/syntax/parsing/resources"
for dat in \
  adoption01.dat adoption02.dat blocks.dat comments01.dat doctype01.dat \
  domjs-unsafe.dat entities01.dat entities02.dat foreign-fragment.dat \
  html5test-com.dat inbody01.dat isindex.dat main-element.dat math.dat \
  menuitem-element.dat namespace-sensitivity.dat noscript01.dat \
  pending-spec-changes-plain-text-unsafe.dat pending-spec-changes.dat \
  plain-text-unsafe.dat processing-instructions.dat quirks01.dat ruby.dat \
  scriptdata01.dat scripted_adoption01.dat scripted_ark.dat \
  scripted_webkit01.dat search-element.dat svg.dat tables01.dat template.dat \
  tests1.dat tests2.dat tests3.dat tests4.dat tests5.dat tests6.dat \
  tests7.dat tests8.dat tests9.dat tests10.dat tests11.dat tests12.dat \
  tests14.dat tests15.dat tests16.dat tests17.dat tests18.dat tests19.dat \
  tests20.dat tests21.dat tests22.dat tests23.dat tests24.dat tests25.dat \
  tests26.dat tests_innerHTML_1.dat tricky01.dat void-in-phrasing.dat \
  webkit01.dat webkit02.dat; do
  fetch_raw "${RES}/${dat}" || FAILED_FETCHES=$((FAILED_FETCHES + 1))
done

echo "== html5lib .dat 拉取完成（失败 ${FAILED_FETCHES}，pin ${WPT_REV:0:9}）=="
ls "${WPT_DATA}/${RES}"/*.dat 2>/dev/null | wc -l
