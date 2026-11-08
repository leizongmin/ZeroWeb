#!/usr/bin/env bash
# WPT 数据套件升级脚本（调研建议 A2）
#
# 用途：wpt-data 是独立 repo（leizongmin/zeroweb-wpt-data，tag 版本化）。
# 上游 web-platform-tests/wpt 持续新增测试（如 Wasm 3.0 单次 +100k 子测试），
# 套件必须随上游滚动，否则通过率数字无法与上游/Ladybird 对比。
#
# 用法：
#   bash scripts/update-wpt-data.sh v2.0        # 升级到指定 tag
#   bash scripts/update-wpt-data.sh --check     # 检查远端可用 tag（只读）
#
# 升级后：
#   1. 手动核对 Makefile 的 WPT_DATA_REF 并更新（本脚本只拉数据不改 Makefile，
#      避免误改其他配置；版本切换记录在 git 提交里）
#   2. 跑 make reftest-upstream FILTER=... 抽查新套件可运行
#   3. 记录 wpt-data ref 到 trend 记录（scripts/record-wpt-trend.sh 自动读取）
#
# 依赖：git, curl
#
# 已知套件打包缺口（升级时按需补，2026-08-10 状态）：
#   - css/css-variables、css/css-ruby：v1.10 未打包，reftest-smoke 依赖；
#     fetch-wpt-data 经 scripts/fetch-wpt-smoke-subdirs.sh 自动补齐
#   - css/filter-effects：已用 scripts/fetch-wpt-subdir.sh 补入本地 wpt-data
#     （652 文件；该子域不在 skip list，属于范围内）——升级新 ref 后需重新补
#   - css/css-masking、css/css-shapes 未打包但**在 reftest-skip-list.txt**
#     （SVG 渲染不在范围），无需补
#   - css/reference/ref-nothing-below.xht 等个别 ref 文件缺失（影响少量 case）
#   - css/support/swatch-yellow.png 缺失（active-selection-041 依赖；R5274 已本地补入
#     CSS2/support 副本——升级新 ref 后需重新补）
#   - css/support/black96x96.png 缺失（顶层 css/reference/ref-filled-black-96px-square.xht
#     引用；R5276 核实该顶层 ref 对 **corpus 是死路径**——150 个引用案全走各套件内
#     css/CSS2/reference/+support/ 完整副本，A/B 零翻转；已补 CSS2/support 副本
#     md5 ed89c5f9 使顶层 ref 自洽，防未来导入案引用死路径静默白板——升级新 ref 后需重新补）
#   - 其余 4 项维持缺：black_red.png（引用案无 match/mismatch 声明，无翻转面）、
#     css-overflow/support/common.js 与 numeric-testcommon.js（testharness JS 非 reftest
#     像素面）、swatch-blue.png（manual 案）
#   - fetch/api/cors/cors-preflight-redirect.any.js：已对齐上游 HEAD（2026-10-29，
#     net-api M2-S12——上游 651ffcb03 于 2026-09-12 把「after preflight failed」
#     五腿改写为「after preflight succeeded」（跟随 + 重定向跳重预检），本地
#     v1.10 快照早于该提交；docs/goal/net-api-compat/evidence/2026-10-29-m2-s12-
#     preflight-redirect-corpus-drift.md）——升级新 ref 后需重新核对（新 ref 若
#     快照 ≥ 2026-09-12 则无需补）
#     销账（2026-11-08，46 号 runner-net-infra M3 切片五核对）：现有 pin 树
#     已原生含「after preflight succeeded」腿，上述条件成立——升级新 ref 后
#     无需再补；redirect.py OPTIONS 补丁本体已随 46 号 M3 注册表收编
#     （route_fetch_redirect 臂）
#   - fetch/api/basic/request-upload.any.js：已对齐上游 HEAD（2026-10-29，
#     net-api M2-S13 语料巡检——上游删除「POST text body 421 retried once」
#     测试，本地残留假红腿；docs/goal/net-api-compat/evidence/2026-10-29-m2-s13-
#     corpus-drift-sweep.md）——升级新 ref 后无需补（新 ref 原生含删除）。
#     注意：request-disturbed.any.js 上游翻转期望（init body 不再扰动源
#     Request）已由 M2-S14 引擎+语料双侧同步对齐（2026-10-29，docs/goal/
#     net-api-compat/evidence/2026-10-29-m2-s14-request-disturbed-init-body.md）
#     ——升级新 ref 后该腿为绿，无需人工干预
#   - css/support/color-testcommon.js 缺失（html/semantics/forms/the-input-element/
#     color.window.js 依赖；s238 勘根定性该文件 1F「load support …」为语料 404
#     非引擎缺陷——META script 装载失败 → 顶层 set_up_fuzzy_color_test
#     ReferenceError → 全文件子测零注册；已从上游 wpt css/support/ 原件本地补入
#     169 行 JS（自包含无依赖），补后 color.window.js 全量子测起跑（color-well
#     序列化引擎面同切收口，见同日 m2s238 账本）——升级新 ref 后需重新补）
#   - images/smiley.png、images/fail.gif 缺失（html/semantics/embedded-content/
#     the-img-element/update-media.html 依赖；s227 勘根定性该文件级 Timeout 为
#     语料 404 非引擎缺陷——404 → error 代替 load → async_test 永不 done，探针
#     p8 fetch 观测 status=404 定谳；已从上游 wpt images/ 原件本地补入 300x150
#     PNG 1852B + 320x240 GIF 24480B，A/B 恰 1 Timeout→Pass 零其他位移；
#     docs/goal/html-semantics-compat/evidence/2026-11-02-m2s227-update-media-
#     corpus-gap.md——升级新 ref 后需重新补）
#   - html/semantics/scripting-1/the-script-element/css-module/resources/
#     redirect-target.css 缺失（css-module/redirect.html 依赖；s249 勘根定性
#     该 3F 为语料 404 非引擎缺陷——redirect.py 终点文件 404 → module 网络
#     错误 → 三条断言（load 应用/重复 import 同一对象/URL 与终点分 map 条目）
#     永假；已从上游 wpt 同路径原件本地补入 3 行 CSS（load-relative-url.css
#     经核与上游一致未动）——升级新 ref 后需重新补）
#   - html/semantics/scripting-1/the-script-element/text-module/file 缺失
#     （integrity.html matches 臂依赖——`import text from "./file" with
#     { type: "text" }`；s250 勘根定性为语料缺口：fetch 按扩展名过滤漏收
#     无扩展名文件 → 404 → matchesLog 空；已从上游 wpt 同路径原件本地补入
#     10 字节 `text file\n`——升级新 ref 后需重新补）
#   - html/semantics/scripting-1/the-script-element/css-module/resources/
#     {utf-8,windows-1250,bom-utf-8,bom-utf-16be,bom-utf-16le}.css 与
#     text-module/{utf-8.txt,windows-1250.txt} 缺失（charset 家族 5 页依赖；
#     s264 勘根定性该 5 页 file-Timeout 全为资产缺口——旧白名单注记「端点
#     落地时再补」，端点 serve-with-content-type.py 已于 s115/s249/s250 落地；
#     已按 lib.sh WPT_REV pin 从上游补入并入 25-html-semantics-compat.sh
#     白名单。字节勿用 master：bom-utf-16be.css 上游已重编码为 UTF-8 BOM+
#     纯文本，与 pin 版真 UTF-16BE 不符，corpus 判据按 pin 字节形写定——
#     升级新 ref 后 make fetch-wpt-html-semantics 白名单幂等自动补齐）
#   - images/green-1x1.png、images/green-16x16.png 与 html/semantics/
#     embedded-content/the-img-element/srcset/resources/image.png 缺失
#     （the-img-element sizes/img-load/srcset 家族依赖；s282 勘根定性
#     sizes-auto 族 6F、sizes-xhr-fragment 1F、adoption 等引用面为语料
#     404 非引擎缺陷——img onerror → 断言永假/文件级 Timeout；已从上游
#     wpt images/ 原件本地补入 1x1 PNG 82B + 16x16 PNG 92B + srcset
#     resources 100x100 PNG 268B（经 gh api contents 按 lib.sh WPT_REV
#     pin 315976933870b34d6ea30e3f6643403edae678ba 取件）；solo A/B
#     sizes-auto 6F 全转绿——升级新 ref 后需重新补）
#   - images/red-16x16.png 缺失（html/semantics/embedded-content/
#     the-img-element/sizes/sizes-auto-dynamic-attributes.html 依赖；s284
#     勘根定性该页 3F 为语料 404 非引擎缺陷——fetch fail → img onerror
#     载入断言永假；已从上游 wpt images/ 原件本地补入 16x16 PNG 130B
#     （同 gh api contents 按 pin 取件）——升级新 ref 后需重新补）
#   - images/rrgg-256x256.png 缺失（html/semantics/embedded-content/
#     the-img-element/historical-progress-event.window.js 依赖；s288 勘根
#     定性该文件级 Timeout 的悬挂腿为语料 404——async_test 等 load 永不
#     done；已从上游 wpt images/ 原件本地补入 256x256 PNG 120B（github.com
#     sparse clone 按 wpt pin 1eecaa0c30 取件）——升级新 ref 后需重新补。
#     同域：404-response-with-actual-image-data.py 端点为 runner 仿真臂
#     （testharness.rs s288，wptserve handler 不入语料），无语料资产）
#   - html/semantics/embedded-content/the-img-element/resources/
#     {animated.gif,animated.webp,red.png} 缺失（animated-image-update.
#     tentative.html、animated-webp-update.tentative.html 与 responsive-
#     image-select-print.html 依赖；s304 勘根定性该簇部分红页含语料 404
#     面——资产缺席时测试图走 error/占位形态，期望帧面根本不可观测；已
#     从上游 wpt 原件本地补入 animated GIF 292B（md5 ceb7ff0068d19fbef6
#     57cc0f09434e44）+ animated WebP 286B（md5 fe2ba7e9a9e024a91ee24e2
#     d6def97b2）+ 100x100 PNG 510B（md5 b8ae7929d842f20ada510d963b741e1c；
#     github.com sparse clone 按 wpt pin efcaa5bb87 取件，2026-11-03
#     master 快照）；补后 animated 两页转为可观测 2.08% 首帧差（10000/
#     480000，GIF/WebP 帧推进引擎面归 render-foundation decode 域）——
#     升级新 ref 后需重新补）
#   - common/reftest-wait.js 缺失（树级 reftest wait 机制脚本，html+css
#     树 354 页 `<script src="/common/reftest-wait.js">` 依赖；本地语料
#     common/ 为绕过本脚本补齐路径的手动填充残留——14 文件独缺此件，s304
#     勘根定性脚本 404 → takeScreenshot 未定义告警刷屏、wait 机制全树失效
#     面；已从上游 wpt sparse clone（pin efcaa5bb87）cat-file 原件补入
#     5616B（md5 fbe764e11b37f04436f6a470ebc418e8）。注记：本脚本
#     fetch_common_resources 走 unpinned master，该件内容随上游漂移，与
#     WPT_DATA_REF pin 语料存在版本错位风险——升级新 ref 后重跑本脚本
#     common/ 补齐即自动接管，或按本条手工补入）
#   - html/semantics/forms/the-textarea-element/support/placeholder.css 缺失
#     （multiline-placeholder{,-cr,-crlf}.html 与 multiline-placeholder-ref.html
#     4 页依赖；s311 勘根定性该簇 3F 为语料缺口非引擎缺陷——上游 CSS 对称
#     归一 test/ref 两侧着色（textarea::placeholder 与 .placeholder 正文同为
#     GrayText + opacity 1），缺席时 test 侧 placeholder 走引擎 UA 灰
#     #757575、ref 侧正文默认色，逐字形色差 5508/480000=1.15% 三页齐红；
#     已从上游 wpt 原件按 pin 取件补入 188B（md5 07368e84cb705a96ff176ec45
#     652a4db；gh api contents 按 wpt pin efcaa5bb87 取件，2026-11-06）——
#     升级新 ref 后需重新补）
#   - html/semantics/document-metadata/the-link-element/{stylesheet.css,
#     resources/stylesheet.css} 缺失（link-rel-attribute-ascii-case-insensitive
#     .html/notref 与 stylesheet-with-base.html 依赖；s315 勘根定性 ascii-case 页
#     mismatch 恒败为纯语料缺口（link1/link3 双 404 → test/notref 同白底）、
#     with-base 页为缺件+`<base>` 基准缺口复合（引擎侧 base 前移同提交）——上游
#     原件本地补入 36B 红（md5 4952ca73677541df4bacbe83cc3aae99）/ 38B 绿
#     （md5 56ae6412c0519859b25b032fb77a2270）（raw.githubusercontent wpt master
#     2026-11-06 取件）——升级新 ref 后需重新补）
#   common/ 已由本脚本自动补齐。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
WPT_DATA_DIR="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_DATA_REPO="https://github.com/leizongmin/zeroweb-wpt-data.git"

# 从 Makefile 读取当前 ref
current_ref() {
  grep -oP 'WPT_DATA_REF\s*\?=\s*\K.*' "${REPO_ROOT}/Makefile" | tr -d ' '
}

# 补齐套件缺的 common/ 基础设施资源（资源缺口，2026-08-07 修复）：
# wpt-data 独立 repo 不打包 WPT 的 common/（blank.html / reftest-wait.js 等），
# 缺失会导致大量 reftest 加载失败。从上游 wpt 仓库拉取到本地 wpt-data/common/。
WPT_UPSTREAM_BASE="https://raw.githubusercontent.com/web-platform-tests/wpt/master"
fetch_common_resources() {
  local target="${WPT_DATA_DIR}/common"
  mkdir -p "$target"
  local listing
  listing=$(curl -s --max-time 30 "https://api.github.com/repos/web-platform-tests/wpt/contents/common" || true)
  local ok=0 fail=0
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    local code
    code=$(curl -s -o "${target}/${f}" -w "%{http_code}" --max-time 20 "${WPT_UPSTREAM_BASE}/common/${f}" || true)
    if [[ "$code" == "200" ]]; then
      ok=$((ok + 1))
    else
      rm -f "${target}/${f}"
      fail=$((fail + 1))
    fi
  done < <(echo "$listing" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
except Exception:
    sys.exit(0)
if isinstance(d, list):
    print('\n'.join(e['name'] for e in d if e.get('type') == 'file'))
" 2>/dev/null || true)
  echo "  common/ 资源：成功 ${ok}，失败 ${fail}"
  # 完整性校验：关键文件必须存在
  for key in blank.html reftest-wait.js; do
    if [[ ! -f "${target}/${key}" ]]; then
      echo "Error: 关键资源缺失 ${target}/${key}——套件不完整，请检查网络后重试"
      return 1
    fi
  done
  echo "  common/ 完整性校验通过（blank.html / reftest-wait.js 在）"
}

usage() {
  sed -n '3,20p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

if [[ $# -lt 1 || "$1" == "--help" ]]; then
  usage
  exit 0
fi

CURRENT_REF="$(current_ref)"

if [[ "$1" == "--check" ]]; then
  echo "当前 wpt-data ref: ${CURRENT_REF}（Makefile WPT_DATA_REF）"
  echo "远端可用 tags:"
  git ls-remote --tags "${WPT_DATA_REPO}" | grep -oP 'refs/tags/\K.*' | sort -V | tail -8
  exit 0
fi

NEW_REF="$1"
if [[ -z "$NEW_REF" ]]; then
  echo "Error: 需要提供目标 tag（如 v2.0）"; exit 1
fi

echo "wpt-data 升级：${CURRENT_REF} → ${NEW_REF}"
echo "  数据目录：${WPT_DATA_DIR}（将删除重建）"

# 校验远端 tag 存在
if ! git ls-remote --tags "${WPT_DATA_REPO}" | grep -q "refs/tags/${NEW_REF}$"; then
  echo "Error: 远端不存在 tag ${NEW_REF}（先 bash scripts/update-wpt-data.sh --check 查看可用 tag）"
  exit 1
fi

# 备份旧套件统计（供升级报告）
if [[ -d "$WPT_DATA_DIR" ]]; then
  OLD_COUNT=$(find "$WPT_DATA_DIR" -type f | wc -l)
  OLD_MANIFEST_ENTRIES=$(python3 -c "import json,sys; d=json.load(open('${WPT_DATA_DIR}/reftest-manifest.json')); print(len(d.get('reftest_entries',[])))" 2>/dev/null || echo "?")
else
  OLD_COUNT=0
  OLD_MANIFEST_ENTRIES="?"
fi

# 拉取新套件（先临时目录再替换，避免半成品状态）
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
git clone --depth=1 --branch "${NEW_REF}" "${WPT_DATA_REPO}" "${TMP_DIR}/wpt-data" >/dev/null 2>&1
rm -rf "${TMP_DIR}/wpt-data/.git"

rm -rf "$WPT_DATA_DIR"
mv "${TMP_DIR}/wpt-data" "$WPT_DATA_DIR"

# 补齐 common/ 资源并校验完整性
fetch_common_resources || exit 1
bash "${SCRIPT_DIR}/fetch-wpt-smoke-subdirs.sh"

NEW_COUNT=$(find "$WPT_DATA_DIR" -type f | wc -l)
NEW_MANIFEST_ENTRIES=$(python3 -c "import json,sys; d=json.load(open('${WPT_DATA_DIR}/reftest-manifest.json')); print(len(d.get('reftest_entries',[])))" 2>/dev/null || echo "?")

echo ""
echo "═══════════════════════════════════════"
echo "  升级完成 ${CURRENT_REF} → ${NEW_REF}"
echo "  文件数:    ${OLD_COUNT} → ${NEW_COUNT}"
echo "  manifest:  ${OLD_MANIFEST_ENTRIES} → ${NEW_MANIFEST_ENTRIES} 条"
echo "═══════════════════════════════════════"
echo ""
echo "下一步："
echo "  1. 更新 Makefile 的 WPT_DATA_REF 为 ${NEW_REF}"
echo "  2. make reftest-upstream FILTER=css/CSS2/backgrounds 抽查新套件"
echo "  3. 趋势记录将自动带上新 ref（scripts/record-wpt-trend.sh）"
