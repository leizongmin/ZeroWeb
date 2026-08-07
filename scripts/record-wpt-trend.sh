#!/usr/bin/env bash
# WPT 度量趋势记录脚本（调研 P2：绝对数 + 周期追踪）
#
# 跑 WPT reftest 全量，把「绝对数」追加到
#   docs/goal/rendering-compat/evidence/wpt-trends/trend.csv
# 并保留 JSON 快照，形成可回溯的趋势基线。
#
# 口径（与 docs/goal/rendering-compat.md 的诚实度量原则一致）：
#   - upstream 模式（默认）：同源 reftest 绝对数（自一致性参考，非达标依据）
#   - oracle 模式：Chromium Oracle 真一致率（DC-14，credible pass），
#     需先 make capture-oracle 生成 oracle-shots（gitignored）
#
# 用法：
#   bash scripts/record-wpt-trend.sh                    # upstream 全量
#   bash scripts/record-wpt-trend.sh --oracle           # DC-14 credible 口径
#   bash scripts/record-wpt-trend.sh --note "R21xx 修复后"
#   bash scripts/record-wpt-trend.sh --filter css-grid  # 局部记录（调试用，不写趋势文件）
#   bash scripts/record-wpt-trend.sh --dry-run          # 只打印将执行的命令
#
# 本地入口（带 test-guard OOM 包裹）：make reftest-trend / make reftest-trend-oracle

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TREND_DIR="${REPO_ROOT}/docs/goal/rendering-compat/evidence/wpt-trends"
TREND_CSV="${TREND_DIR}/trend.csv"

MODE="upstream"
NOTE=""
FILTER=""
DRY_RUN=false

# wpt-data 套件版本（C3：从 Makefile WPT_DATA_REF 读取，随每次记录带上，
# 保证不同套件版本间的绝对数可比性）
WPT_DATA_REF="$(grep -oP 'WPT_DATA_REF\s*\?=\s*\K.*' "${REPO_ROOT}/Makefile" | tr -d ' ' || echo "unknown")"

usage() {
  sed -n '2,16p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --oracle) MODE="oracle"; shift ;;
    --note) NOTE="$2"; shift 2 ;;
    --filter) FILTER="$2"; shift 2 ;;
    --dry-run) DRY_RUN=true; shift ;;
    --help) usage; exit 0 ;;
    *) echo "Unknown argument: $1"; exit 1 ;;
  esac
done

if [[ "$FILTER" != "" ]]; then
  echo "filter 模式（调试用，不写趋势文件）：$FILTER"
fi

if [[ "$MODE" == "oracle" && ! -d "${REPO_ROOT}/tests/wpt-runner/oracle-shots" ]]; then
  echo "Error: oracle-shots 不存在，请先运行 make capture-oracle 生成（gitignored，可再生）。"
  exit 1
fi

# ── 构造命令 ──
if [[ "$MODE" == "oracle" ]]; then
  CMD=("${REPO_ROOT}/target/release/zero-wpt-runner" "reftest-oracle" "${FILTER}")
else
  CMD=("${REPO_ROOT}/target/release/zero-wpt-runner" "reftest-upstream" "${FILTER}")
fi

if [[ "$DRY_RUN" == "true" ]]; then
  echo "Dry-run: cd ${REPO_ROOT} && ${CMD[*]}"
  exit 0
fi

if [[ ! -x "${REPO_ROOT}/target/release/zero-wpt-runner" ]]; then
  echo "Error: target/release/zero-wpt-runner 不存在，请先构建：make build 或 cargo build --release"
  exit 1
fi

if [[ ! -d "${REPO_ROOT}/tests/wpt-runner/wpt-data" ]]; then
  echo "Error: wpt-data 不存在，请先运行 make fetch-wpt-data"
  exit 1
fi

# ── 执行并捕获输出（报告在 stderr）──
OUTPUT_FILE=$(mktemp)
trap 'rm -f "$OUTPUT_FILE"' EXIT
set +e
(cd "${REPO_ROOT}" && "${CMD[@]}") > "$OUTPUT_FILE" 2>&1
RC=$?
set -e
cat "$OUTPUT_FILE" | tail -40

# reftest-upstream 在存在失败 case 时退出 1（门禁语义，2026-08-07 CI 暴露）；
# trend 记录是报告语义——RC==1 且输出含完整报告即视为「跑完」，继续记录
#（失败 case 是数据而非错误）。RC 为其他值（崩溃/中断）才是真失败。
if [[ $RC -ne 0 && $RC -ne 1 ]]; then
  echo "Error: zero-wpt-runner 退出码 $RC（上方为输出尾部）"
  exit $RC
fi

# ── 解析绝对数 ──
if [[ "$MODE" == "oracle" ]]; then
  total=$(grep -oP 'with chromium oracle:\s*\K[0-9]+' "$OUTPUT_FILE" || echo 0)
  passed=$(grep -oP 'credible pass \(排除退化\):\s*\K[0-9]+' "$OUTPUT_FILE" || echo 0)
  rate=$(grep -oP 'credible pass \(排除退化\): [0-9]+ \(\K[0-9.]+' "$OUTPUT_FILE" || echo 0)
  oracle_pass=$(grep -oP 'oracle-pass \(z_vs_chr < [0-9.]+%\):\s*\K[0-9]+' "$OUTPUT_FILE" || echo 0)
  strict_pass=$(grep -oP '真通过 \(z_vs_chr < 布局0.1%/文字0.5%\):\s*\K[0-9]+' "$OUTPUT_FILE" || echo 0)
  extra="oracle_pass=${oracle_pass};strict_pass=${strict_pass}"
else
  total=$(grep -oP '^  Total:\s*\K[0-9]+' "$OUTPUT_FILE" || echo 0)
  passed=$(grep -oP '^  Passed:\s*\K[0-9]+' "$OUTPUT_FILE" || echo 0)
  failed=$(grep -oP '^  Failed:\s*\K[0-9]+' "$OUTPUT_FILE" || echo 0)
  skipped=$(grep -oP '^  Skipped:\s*\K[0-9]+' "$OUTPUT_FILE" || echo 0)
  rate=$(grep -oP '^  Pass Rate:\s*\K[0-9.]+' "$OUTPUT_FILE" || echo 0)
  extra="failed=${failed};skipped=${skipped}"
fi

DATE=$(date +%F)
SHA=$(cd "${REPO_ROOT}" && git rev-parse --short HEAD 2>/dev/null || echo "unknown")
NOTE_SAFE=$(echo "$NOTE" | sed 's/,/，/g')  # CSV 逗号转义

if [[ "$FILTER" != "" ]]; then
  echo "filter 模式：total=$total passed=$passed rate=${rate}%（不写趋势文件）"
  exit 0
fi

if [[ "$total" == "0" ]]; then
  echo "Error: 未能解析出测试总数，趋势文件未更新。检查上方输出。"
  exit 1
fi

# ── 追加趋势 CSV（绝对数，非百分比；百分比只在同口径快照下比较）──
mkdir -p "$TREND_DIR"
if [[ ! -f "$TREND_CSV" ]]; then
  echo "# WPT 趋势基线（绝对数口径，见 docs/goal/rendering-compat.md「测试资产化」）" > "$TREND_CSV"
  echo "# date,mode,wpt_data_ref,total,passed,rate_pct,extra,git_sha,note" >> "$TREND_CSV"
fi
echo "${DATE},${MODE},${WPT_DATA_REF},${total},${passed},${rate},${extra},${SHA},${NOTE_SAFE}" >> "$TREND_CSV"

# ── JSON 快照 ──
SNAPSHOT="${TREND_DIR}/${DATE}-${MODE}.json"
cat > "$SNAPSHOT" <<EOF
{
  "date": "${DATE}",
  "mode": "${MODE}",
  "wpt_data_ref": "${WPT_DATA_REF}",
  "total": ${total},
  "passed": ${passed},
  "rate_pct": ${rate},
  "extra": "${extra}",
  "git_sha": "${SHA}",
  "note": "${NOTE}"
}
EOF

echo ""
echo "═══════════════════════════════════════"
echo "  趋势已记录（绝对数）"
echo "  mode:   ${MODE}"
echo "  total:  ${total}"
echo "  passed: ${passed}"
echo "  rate:   ${rate}%"
echo "  csv:    ${TREND_CSV}"
echo "  json:   ${SNAPSHOT}"
echo "═══════════════════════════════════════"
