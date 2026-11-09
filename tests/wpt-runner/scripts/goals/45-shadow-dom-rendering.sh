#!/usr/bin/env bash
# Goal: shadow-dom-rendering（编号 45——推进顺序见同目录 README.md；契约
# docs/goal/archive/shadow-dom-rendering.md，M0 盘点 + 数值门见
# docs/goal/archive/shadow-dom-rendering/evidence/2026-11-07-m0-hookpoint-audit-and-corpus.md）。
#
# 渲染级语料预置：shadow-dom reftest 池 20 案 + refs + 补充池 10 案的显式清单拉取
# （非整目录——shadow-dom/ 的 JS 语义面归 42 号 shadow-dom-expansion 通道，
# fetch-web-components-subset.sh 头注释 skip 域同源两侧防漂移）。
#
# 与 44 号 sync 模式的差异：池小且封闭（20 案 + 15 refs/support + 10 补充池案），逐文件显式
# 列表即可，不走 git trees 对账；missing=0 断言保 DC-4 分批门分母真实性。
# 共享依赖（/css/reference/ref-filled-green-100px-square.xht、/fonts/ahem.css）
# 归 css/fonts 语料清单所有，只断言在场不重复拉取。
#
# 用法：bash tests/wpt-runner/scripts/goals/45-shadow-dom-rendering.sh
#   FORCE=1 强制重拉（幂等缺省：文件在场即跳过）
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

# -- 渲染级 reftest 池（M0 盘点 20 案；refs 与 reference/empty.html 随行）----
REFTEST_POOL=(
  shadow-dom/directionality-001.tentative.html
  shadow-dom/directionality-001-ref.html
  shadow-dom/directionality-002.tentative.html
  shadow-dom/directionality-002-ref.html
  shadow-dom/focus/focus-pseudo-on-shadow-host-1.html
  shadow-dom/focus/focus-pseudo-on-shadow-host-2.html
  shadow-dom/focus/focus-pseudo-on-shadow-host-3.html
  shadow-dom/imperative-slot-layout-invalidation-001.html
  shadow-dom/imperative-slot-layout-invalidation-001-ref.html
  shadow-dom/invalidate-sibling-different-slots.html
  shadow-dom/invalidate-sibling-different-slots-ref.html
  shadow-dom/layout-slot-no-longer-assigned.html
  shadow-dom/layout-slot-no-longer-fallback.html
  shadow-dom/manual-slot-assignment-no-wrong-unassign.html
  shadow-dom/manual-slot-assignment-no-wrong-unassign-ref.html
  shadow-dom/nested-hover-pseudo-class-removal.html
  shadow-dom/shadow-style-invalidation-vw-units.html
  shadow-dom/shadow-style-invalidation-vw-units-ref.html
  shadow-dom/slot-fallback-content-001.html
  shadow-dom/slot-fallback-content-001-ref.html
  shadow-dom/slot-fallback-content-002.html
  shadow-dom/slot-fallback-content-002-ref.html
  shadow-dom/slot-fallback-content-003.html
  shadow-dom/slot-fallback-content-003-ref.html
  shadow-dom/slot-fallback-content-004.html
  shadow-dom/slot-fallback-content-004-ref.html
  shadow-dom/slot-fallback-content-005.html
  shadow-dom/slot-fallback-content-005-ref.html
  shadow-dom/slot-fallback-content-006.html
  shadow-dom/slot-fallback-content-006-ref.html
  shadow-dom/slot-fallback-content-007.html
  shadow-dom/slot-fallback-content-007-ref.html
  shadow-dom/slot-fallback-content-008.html
  shadow-dom/slot-fallback-content-008-ref.html
  shadow-dom/reference/empty.html
)

# -- testharness 渲染耦合补充池（42 号 skip 规则排除归 45 号，M1 通道计门）----
# 与 tests/wpt-runner/src/testharness.rs SHADOW_RENDER_POOL_CASES 两侧同源
# （runner 通道 testharness-shadow-dom 消费）：增删案两侧必同步。
SUPPLEMENT_POOL=(
  shadow-dom/offsetParent-across-shadow-boundaries.html
  shadow-dom/offsetTop-offsetLeft-across-shadow-boundaries.html
  shadow-dom/focus-within-shadow.html
  shadow-dom/scroll-restore-shadow.html
  shadow-dom/scroll-to-the-fragment-in-shadow-tree.html
  shadow-dom/Document-caretPositionFromPoint.tentative.html
  shadow-dom/DocumentOrShadowRoot-prototype-elementFromPoint.html
  shadow-dom/HighlightRegistry-highlightsFromPoint.html
  shadow-dom/MouseEvent-prototype-offsetX-offsetY.html
  shadow-dom/accesskey.tentative.html
)

# -- 共享依赖（css/fonts 语料所有，只断言在场）------------------------------
SHARED_DEPS=(
  css/reference/ref-filled-green-100px-square.xht
  fonts/ahem.css
)

echo "== 45-shadow-dom-rendering 语料预置（pin ${WPT_REV}）=="
missing=0
for relative in "${REFTEST_POOL[@]}" "${SUPPLEMENT_POOL[@]}"; do
  if ! fetch_raw "${relative}"; then
    echo "  MISSING: ${relative}"
    missing=$((missing + 1))
  fi
done

for relative in "${SHARED_DEPS[@]}"; do
  if [[ ! -s "${WPT_DATA}/${relative}" ]]; then
    echo "  SHARED-DEP-MISSING: ${relative}（归 css/fonts 语料清单，请跑对应 fetch 目标）"
    missing=$((missing + 1))
  fi
done

pool_count=${#REFTEST_POOL[@]}
supp_count=${#SUPPLEMENT_POOL[@]}
echo "== 盘点：reftest 池 ${pool_count} 文件（20 案 + 14 refs + empty.html）+ 补充池 ${supp_count} 案 =========="
echo "   reftest 基线门：20 案分五批（B1 slot 回退 8 / B2 布局失效 5 / B3 样式继承 3 / B4 :host 伪类 3 / B5 交互耦合 1 静态面），M4 = 20/20 逐批不回落"
echo "   基线实测（2026-11-07）：16P/4F——失败 slot-fallback-content-004/006、shadow-style-invalidation-vw-units、focus/focus-pseudo-on-shadow-host-1"

if [[ "${missing}" -gt 0 ]]; then
  echo "FAIL: ${missing} 个语料文件缺失（missing=0 断言未过——DC-4 分母真实性）" >&2
  exit 1
fi
echo "missing=0 断言通过。"

cat <<'NEXT'
== 下一步检查单（契约 docs/goal/archive/shadow-dom-rendering.md）==
- M1：:host 族 + scoped sheets 进渲染管线（evidence §6 改动面 1-5）；
  验收 = B3/B4 红案翻绿 + B1/B2 零回落 + make reftest 全量零回归
- M2：::slotted() 合法化 + matcher arm + flat-tree 组合子（DC-2 收口）
- M3：mutation apply 臂 resolve_slots 触发 + shadow Text 布局通道 + B6 补充池通道
- M4：20/20 逐批不回落（+selection/shadow-dom 邻域 9/9 并跑对账） + rendering-compat 全量零回归 + make test + 头条不回落
NEXT
