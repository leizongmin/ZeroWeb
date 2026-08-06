#!/usr/bin/env bash
# WPT Reftest 导入脚本 — 从上游 WPT GitHub 仓库导入 CSS 2.1 核心 reftest
#
# 用法：
#   ./import-wpt-reftests.sh [--count N] [--category CATEGORY]  批量初始化
#   ./import-wpt-reftests.sh --add <test_path> <ref_path> [--note "备注"]  单条导入（测试资产化）
#   ./import-wpt-reftests.sh --add <test_path> <ref_path> --relation mismatch
#
# 从 https://raw.githubusercontent.com/web-platform-tests/wpt/master/ 下载
# reftest HTML 文件和参考文件到本地 wpt-data/ 目录。
#
# 单条导入（--add）是「测试资产化」的操作入口：文件本体进入 wpt-data/，
# 条目（test/ref/relation/日期/备注）追加到 imported-tests.txt 常驻账本，
# 并重新生成 wpt-data/reftest-manifest.json。Makefile 入口：make import-wpt。
#
# 依赖：curl, jq

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/../wpt-data"
WPT_BASE="https://raw.githubusercontent.com/web-platform-tests/wpt/master"

COUNT=60  # 默认导入 60 个（要求 ≥ 50）
CATEGORY="css/CSS2"  # 默认导入 CSS 2.1
ADD_MODE=false
ADD_TEST=""
ADD_REF=""
ADD_RELATION="=="
ADD_NOTE=""
LEDGER="${SCRIPT_DIR}/../imported-tests.txt"  # 常驻断言集账本（测试资产化）

# 解析参数
while [[ $# -gt 0 ]]; do
  case "$1" in
    --count) COUNT="$2"; shift 2 ;;
    --category) CATEGORY="$2"; shift 2 ;;
    --add) ADD_MODE=true; ADD_TEST="$2"; ADD_REF="$3"; shift 3 ;;
    --relation) ADD_RELATION="$2"; shift 2 ;;
    --note) ADD_NOTE="$2"; shift 2 ;;
    --help)
      echo "Usage: $0 [--count N] [--category CATEGORY]"
      echo "       $0 --add <test_path> <ref_path> [--relation ==|!=] [--note \"备注\"]"
      echo "  --count N       Number of reftest pairs to import (default: 60)"
      echo "  --category DIR  WPT directory prefix (default: css/CSS2)"
      echo "  --add P1 P2     单条导入（测试资产化）：下载 test+ref 到 wpt-data/，"
      echo "                  追加到 imported-tests.txt 账本并重新生成 manifest"
      exit 0
      ;;
    *) echo "Unknown argument: $1"; exit 1 ;;
  esac
done

if [[ "$ADD_MODE" == "true" && -z "$ADD_TEST" && -z "$ADD_REF" ]]; then
  echo "Error: --add requires <test_path> and <ref_path>"; exit 1
fi

# 从账本合并已有导入条目（去重，保留顺序；仅保留本地文件存在的）
declare -a LEDGER_ENTRIES=()
if [[ -f "$LEDGER" ]]; then
  while IFS= read -r line; do
    line="${line%%#*}"   # 去注释
    line="$(echo "$line" | xargs)"  # 去首尾空白
    [[ -z "$line" ]] && continue
    LEDGER_ENTRIES+=("$line")
  done < "$LEDGER"
fi

echo "WPT Reftest Importer"
echo "  Category: ${CATEGORY}"
echo "  Count:    ${COUNT}"
echo "  Output:   ${DATA_DIR}"
echo ""

# 设置代理（如果需要）
if [[ -f ~/use-proxy ]]; then
  source ~/use-proxy 2>/dev/null || true
fi

mkdir -p "${DATA_DIR}/${CATEGORY}"

# 导入 reftest 文件对的函数
import_reftest_pair() {
  local test_path="$1"
  local ref_path="$2"

  local test_file="${DATA_DIR}/${test_path}"
  local ref_file="${DATA_DIR}/${ref_path}"

  # 跳过已存在的文件
  if [[ -f "${test_file}" ]] && [[ -f "${ref_file}" ]]; then
    return 0
  fi

  # 创建目录
  mkdir -p "$(dirname "${test_file}")"
  mkdir -p "$(dirname "${ref_file}")"

  # 下载测试文件
  local test_url="${WPT_BASE}/${test_path}"
  local ref_url="${WPT_BASE}/${ref_path}"

  if ! curl -sSf -o "${test_file}" "${test_url}" 2>/dev/null; then
    echo "  SKIP: ${test_path} (download failed)"
    rm -f "${test_file}"
    return 1
  fi

  # 下载参考文件
  if ! curl -sSf -o "${ref_file}" "${ref_url}" 2>/dev/null; then
    echo "  SKIP: ${ref_path} (download failed)"
    rm -f "${test_file}" "${ref_file}"
    return 1
  fi

  echo "  OK: ${test_path} -> ${ref_path}"
  return 0
}

# 硬编码的 CSS 2.1 核心 reftest 列表
# 选择不依赖外部资源（图片、特殊字体）的 reftest
# 格式：test_path ref_path

RENDERING_TESTS=(
  # ── 颜色 ──
  "css/CSS2/colors/color-001.xht css/CSS2/colors/color-001-ref.xht"
  "css/CSS2/colors/color-002.xht css/CSS2/colors/color-002-ref.xht"
  "css/CSS2/colors/color-003.xht css/CSS2/colors/color-003-ref.xht"
  "css/CSS2/colors/color-004.xht css/CSS2/colors/color-004-ref.xht"
  "css/CSS2/colors/color-005.xht css/CSS2/colors/color-005-ref.xht"

  # ── 背景 ──
  "css/CSS2/backgrounds/background-001.xht css/CSS2/backgrounds/background-001-ref.xht"
  "css/CSS2/backgrounds/background-002.xht css/CSS2/backgrounds/background-002-ref.xht"
  "css/CSS2/backgrounds/background-003.xht css/CSS2/backgrounds/background-003-ref.xht"
  "css/CSS2/backgrounds/background-004.xht css/CSS2/backgrounds/background-004-ref.xht"
  "css/CSS2/backgrounds/background-005.xht css/CSS2/backgrounds/background-005-ref.xht"

  # ── 边框 ──
  "css/CSS2/borders/border-001.xht css/CSS2/borders/border-001-ref.xht"
  "css/CSS2/borders/border-002.xht css/CSS2/borders/border-002-ref.xht"
  "css/CSS2/borders/border-003.xht css/CSS2/borders/border-003-ref.xht"
  "css/CSS2/borders/border-bottom-color-001.xht css/CSS2/borders/border-bottom-color-001-ref.xht"
  "css/CSS2/borders/border-bottom-width-001.xht css/CSS2/borders/border-bottom-width-001-ref.xht"

  # ── 盒模型 ──
  "css/CSS2/box-model/box-model-001.xht css/CSS2/box-model/box-model-001-ref.xht"
  "css/CSS2/box-model/margin-001.xht css/CSS2/box-model/margin-001-ref.xht"
  "css/CSS2/box-model/margin-002.xht css/CSS2/box-model/margin-002-ref.xht"
  "css/CSS2/box-model/margin-collapse-001.xht css/CSS2/box-model/margin-collapse-001-ref.xht"
  "css/CSS2/box-model/padding-001.xht css/CSS2/box-model/padding-001-ref.xht"

  # ── 定位 ──
  "css/CSS2/positioning/position-001.xht css/CSS2/positioning/position-001-ref.xht"
  "css/CSS2/positioning/position-002.xht css/CSS2/positioning/position-002-ref.xht"
  "css/CSS2/positioning/position-absolute-001.xht css/CSS2/positioning/position-absolute-001-ref.xht"
  "css/CSS2/positioning/position-relative-001.xht css/CSS2/positioning/position-relative-001-ref.xht"
  "css/CSS2/positioning/position-fixed-001.xht css/CSS2/positioning/position-fixed-001-ref.xht"

  # ── 显示 ──
  "css/CSS2/display/display-001.xht css/CSS2/display/display-001-ref.xht"
  "css/CSS2/display/display-002.xht css/CSS2/display/display-002-ref.xht"
  "css/CSS2/display/display-003.xht css/CSS2/display/display-003-ref.xht"
  "css/CSS2/display/visibility-001.xht css/CSS2/display/visibility-001-ref.xht"
  "css/CSS2/display/visibility-002.xht css/CSS2/display/visibility-002-ref.xht"

  # ── 浮动 ──
  "css/CSS2/floats/floats-001.xht css/CSS2/floats/floats-001-ref.xht"
  "css/CSS2/floats/floats-002.xht css/CSS2/floats/floats-002-ref.xht"
  "css/CSS2/floats/floats-003.xht css/CSS2/floats/floats-003-ref.xht"
  "css/CSS2/floats/clear-001.xht css/CSS2/floats/clear-001-ref.xht"
  "css/CSS2/floats/clear-002.xht css/CSS2/floats/clear-002-ref.xht"

  # ── 行内 ──
  "css/CSS2/inline/inline-001.xht css/CSS2/inline/inline-001-ref.xht"
  "css/CSS2/inline/inline-002.xht css/CSS2/inline/inline-002-ref.xht"
  "css/CSS2/line-height/line-height-001.xht css/CSS2/line-height/line-height-001-ref.xht"
  "css/CSS2/line-height/line-height-002.xht css/CSS2/line-height/line-height-002-ref.xht"
  "css/CSS2/line-height/line-height-003.xht css/CSS2/line-height/line-height-003-ref.xht"

  # ── 文本 ──
  "css/CSS2/text/text-align-001.xht css/CSS2/text/text-align-001-ref.xht"
  "css/CSS2/text/text-align-002.xht css/CSS2/text/text-align-002-ref.xht"
  "css/CSS2/text/text-decoration-001.xht css/CSS2/text/text-decoration-001-ref.xht"
  "css/CSS2/text/text-indent-001.xht css/CSS2/text/text-indent-001-ref.xht"
  "css/CSS2/text/text-transform-001.xht css/CSS2/text/text-transform-001-ref.xht"

  # ── 尺寸 ──
  "css/CSS2/sizes/width-001.xht css/CSS2/sizes/width-001-ref.xht"
  "css/CSS2/sizes/height-001.xht css/CSS2/sizes/height-001-ref.xht"
  "css/CSS2/sizes/min-width-001.xht css/CSS2/sizes/min-width-001-ref.xht"
  "css/CSS2/sizes/max-width-001.xht css/CSS2/sizes/max-width-001-ref.xht"
  "css/CSS2/sizes/min-height-001.xht css/CSS2/sizes/min-height-001-ref.xht"

  # ── 溢出 ──
  "css/CSS2/overflow/overflow-001.xht css/CSS2/overflow/overflow-001-ref.xht"
  "css/CSS2/overflow/overflow-002.xht css/CSS2/overflow/overflow-002-ref.xht"
  "css/CSS2/overflow/overflow-003.xht css/CSS2/overflow/overflow-003-ref.xht"

  # ── z-index ──
  "css/CSS2/z-index/z-index-001.xht css/CSS2/z-index/z-index-001-ref.xht"
  "css/CSS2/z-index/z-index-002.xht css/CSS2/z-index/z-index-002-ref.xht"
  "css/CSS2/z-index/z-index-003.xht css/CSS2/z-index/z-index-003-ref.xht"

  # ── 生成内容 ──
  "css/CSS2/generated-content/content-001.xht css/CSS2/generated-content/content-001-ref.xht"
  "css/CSS2/generated-content/content-002.xht css/CSS2/generated-content/content-002-ref.xht"

  # ── 表格（属性存储但布局未完整实现） ──
  "css/CSS2/tables/table-001.xht css/CSS2/tables/table-001-ref.xht"
  "css/CSS2/tables/table-border-001.xht css/CSS2/tables/table-border-001-ref.xht"
)

# 账本写入函数：追加一行（test/ref/relation/日期/备注），文件不存在时先写头注释
append_ledger() {
  local entry="$1"
  mkdir -p "$(dirname "$LEDGER")"
  if [[ ! -f "$LEDGER" ]]; then
    cat > "$LEDGER" <<'LEDGER_HEADER'
# WPT Reftest 常驻断言集账本（测试资产化）— 由 import-wpt-reftests.sh 维护
# 格式：<test_path> <ref_path> <relation> <导入日期> <备注>
LEDGER_HEADER
  fi
  echo "$entry" >> "$LEDGER"
}

if [[ "$ADD_MODE" == "true" ]]; then
  echo "Single import (测试资产化): ${ADD_TEST} => ${ADD_REF} (${ADD_RELATION})"
  if ! import_reftest_pair "$ADD_TEST" "$ADD_REF"; then
    echo "Error: import failed for ${ADD_TEST}"; exit 1
  fi
  if grep -q "^${ADD_TEST} " "$LEDGER" 2>/dev/null; then
    echo "  LEDGER: ${ADD_TEST} 已在账本中，跳过追加"
  else
    append_ledger "${ADD_TEST} ${ADD_REF} ${ADD_RELATION} $(date +%F) ${ADD_NOTE}"
    echo "  LEDGER: 已追加到 imported-tests.txt"
    LEDGER_ENTRIES+=("${ADD_TEST} ${ADD_REF} ${ADD_RELATION}")
  fi
else
  echo "Importing ${#RENDERING_TESTS[@]} reftest pairs..."
  echo ""

  imported=0
  failed=0

for entry in "${RENDERING_TESTS[@]}"; do
  test_path=$(echo "$entry" | awk '{print $1}')
  ref_path=$(echo "$entry" | awk '{print $2}')

  if import_reftest_pair "$test_path" "$ref_path"; then
    ((imported++))
  else
    ((failed++))
  fi

  if [[ $imported -ge $COUNT ]]; then
    break
  fi
done

  echo ""
  echo "Import complete: ${imported} pairs imported, ${failed} failed"
fi

# 合并硬编码列表 + 账本条目（按 test_path 去重，保留先出现者）
declare -a ENTRIES=()
declare -A SEEN=()
for entry in "${RENDERING_TESTS[@]}" "${LEDGER_ENTRIES[@]}"; do
  test_path=$(echo "$entry" | awk '{print $1}')
  [[ -z "$test_path" ]] && continue
  if [[ -z "${SEEN[$test_path]:-}" ]]; then
    SEEN[$test_path]=1
    ENTRIES+=("$entry")
  fi
done

# 生成导入清单
MANIFEST="${DATA_DIR}/reftest-manifest.json"
echo "Generating manifest: ${MANIFEST}"

echo '{"reftest_entries": [' > "$MANIFEST"
first=true
for entry in "${ENTRIES[@]}"; do
  test_path=$(echo "$entry" | awk '{print $1}')
  ref_path=$(echo "$entry" | awk '{print $2}')
  relation=$(echo "$entry" | awk '{print $3}')
  relation="${relation:-==}"
  test_file="${DATA_DIR}/${test_path}"

  if [[ -f "${test_file}" ]]; then
    if [[ "$first" == "true" ]]; then
      first=false
    else
      echo "," >> "$MANIFEST"
    fi
    echo -n "  {\"test_path\": \"${test_path}\", \"ref_path\": \"${ref_path}\", \"relation\": \"${relation}\"}" >> "$MANIFEST"
  fi
done
echo "" >> "$MANIFEST"
echo ']}' >> "$MANIFEST"

echo "Done. Manifest written to ${MANIFEST}"
