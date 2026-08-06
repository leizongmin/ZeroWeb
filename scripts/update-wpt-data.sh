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

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
WPT_DATA_DIR="${REPO_ROOT}/tests/wpt-runner/wpt-data"
WPT_DATA_REPO="https://github.com/leizongmin/zeroweb-wpt-data.git"

# 从 Makefile 读取当前 ref
current_ref() {
  grep -oP 'WPT_DATA_REF\s*\?=\s*\K.*' "${REPO_ROOT}/Makefile" | tr -d ' '
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
