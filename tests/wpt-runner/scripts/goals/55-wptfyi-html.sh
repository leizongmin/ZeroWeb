#!/usr/bin/env bash
# Goal: wptfyi-html（docs/goal/wptfyi-html.md，五目标计划 03 号——html 顶级目录缺口）
# DC-1 首批：html/the-xhtml-syntax XHTML 实体矩阵语料（缺口 25,607 子测试，15 文件
# 中 12 个实体矩阵 .htm 贡献 25,606）。
#
# 实体矩阵机制（上游 pin 31597693 实读）：外层 .htm 页经 iframe 载入
# support/xhtml-mathml-dtd-entity.htm，调其 run([mime, publicId, systemId, label])；
# support 页 XHR 拉 entities.json（2,231 项命名实体），逐实体
# DOMParser.parseFromString(doctype+含实体串, mime) 断言 Text 节点 charCode——
# application/xhtml+xml 走真实 MIME 的 XML 文档解析路径，systemId 恒 "foo"
# （不存在的相对 URL——外部实体不得发起网络取回的安全边界锚）。
#
# 语料极小（15 文件 + support 三资产），显式清单拉取，不走 API 列目录。
# 另两形态零执行记账（runner 通道侧同源规则）：
#   - data-xhtml-with-dtd.html / adopt-while-parsing-001.html：rel=match reftest 形
#     （本口径 c=[0,0] 不贡献子测试分母；ref/子文档一并拉取保语料完整）
#   - xml-stylesheet-blocking.xhtml：真 .xhtml 文档（XML MIME 载入路径）——runner
#     无 XML 文档管道，通道零执行记账，DC-2 门控
# 用法：bash tests/wpt-runner/scripts/goals/55-wptfyi-html.sh（FORCE=1 强制重拉）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="wptfyi-html"
BASE="html/the-xhtml-syntax/parsing-xhtml-documents"

FILES=(
  # 12 实体矩阵外层页（DC-1 试跑计分面）
  "${BASE}/xhtml-mathml-dtd-entity-1.htm"
  "${BASE}/xhtml-mathml-dtd-entity-2.htm"
  "${BASE}/xhtml-mathml-dtd-entity-3.htm"
  "${BASE}/xhtml-mathml-dtd-entity-4.htm"
  "${BASE}/xhtml-mathml-dtd-entity-5.htm"
  "${BASE}/xhtml-mathml-dtd-entity-6.htm"
  "${BASE}/xhtml-mathml-dtd-entity-7.htm"
  "${BASE}/xhtml-mathml-dtd-entity-8.htm"
  "${BASE}/xhtml-mathml-dtd-entity-9.htm"
  "${BASE}/xhtml-mathml-dtd-entity-10.htm"
  "${BASE}/xhtml-mathml-dtd-entity-11.htm"
  "${BASE}/xhtml-mathml-dtd-entity-12.htm"
  # support 资产（实体机器页 + 实体表 + xml-stylesheet 样式）
  "${BASE}/support/xhtml-mathml-dtd-entity.htm"
  "${BASE}/support/entities.json"
  "${BASE}/support/simple-style.css"
  # reftest 形与 .xhtml（零执行记账，语料完整性）
  "${BASE}/data-xhtml-with-dtd.html"
  "${BASE}/data-xhtml-with-dtd-ref.html"
  "${BASE}/adopt-while-parsing-001.html"
  "${BASE}/adopt-while-parsing-001-ref.html"
  "${BASE}/adopt-while-parsing.xhtml"
  "${BASE}/xml-stylesheet-blocking.xhtml"
)

FAILED_FETCHES=0
for relative in "${FILES[@]}"; do
  if ! fetch_raw "${relative}"; then
    FAILED_FETCHES=$((FAILED_FETCHES + 1))
  fi
done

if [[ "${FAILED_FETCHES}" -gt 0 ]]; then
  echo "wptfyi-html the-xhtml-syntax fetch incomplete: ${FAILED_FETCHES} failed" >&2
  exit 1
fi

echo "wptfyi-html the-xhtml-syntax corpus ready (${#FILES[@]} files, WPT ${WPT_REV})"
