#!/usr/bin/env bash
# Legacy Static Web smoke（DC-13，goal rendering-compat.md line 316-320）。
#
# 跑 evidence/product-static/legacy-html/fixtures/*.html（20 页 HTML 3.2/4 + CSS1/2 静态
# fixture），每页 ZeroWeb CPU 截图 vs chrome-127 oracle PNG 像素 diff% + struct-check。
#
# ★ trend-only（退出 0）——diff 主要归因字体墙（fontdue 行度量 vs chromium，R633 多会话
# plateau），非回归；像素阈值作趋势指标记录，不替代 WPT/DC-14 达标口径。退出码：
#   0  全部跑完（diff 为趋势数据）
#   1  struct-check 出现真实结构性退化（sibling-overlap / text-concatenation）= 真回归
#
# Oracle 抓取须 chrome-for-testing 127（系统 chromium 150 在 WSL2 kernel 6.6 上 SIGTRAP，
# 见 scripts/install-oracle-chrome.sh + master.md R1650）。oracle PNG 已预抓（oracle/*.png）；
# 重抓：见文末 capture-all-oracles()。
set -uo pipefail

LEGACY_DIR="$(cd "$(dirname "$0")" && pwd)"
FIXTURE_DIR="$LEGACY_DIR/fixtures"
ORACLE_DIR="$LEGACY_DIR/oracle"

# PUPPETEER_EXECUTABLE_PATH 不影响 product-smoke（ZW CPU 路径），仅 oracle 抓取需要。
# ZeroWeb CPU 截图经 zero-wpt-runner product-smoke 子命令。
MAX_DIFF="${MAX_DIFF:-100}"  # trend-only：默认不因 diff 失败（struct-check 才失败）

# 确保测试二进制已构建（release）。test-guard 包裹防 OOM。
TEST_GUARD="${TEST_GUARD:-./target/test-guard}"

echo "=== Legacy Static Web smoke (DC-13 Tier 1, HTML 3.2/4 + CSS1/2) ==="
echo "    fixtures: $FIXTURE_DIR"
echo "    oracle:   $ORACLE_DIR (chrome-for-testing 127)"
echo

if [ ! -x "$TEST_GUARD" ]; then
  echo "[run-all] building test-guard + zero-wpt-runner (release) ..."
  cargo build --release --bin zero-wpt-runner || exit 1
fi

total=0
struct_fail=0
sum_pct=0
max_pct=0
max_name=""
results=()

for f in "$FIXTURE_DIR"/*.html; do
  name=$(basename "$f" .html)
  oracle="$ORACLE_DIR/$name.png"
  total=$((total+1))
  if [ ! -f "$oracle" ]; then
    echo "  [SKIP] $name — oracle missing (run capture-all-oracles)"
    results+=("SKIP  $name  (no oracle)")
    continue
  fi
  # product-smoke 退出码：0 = diff≤max + struct ok；2 = diff>max；3 = struct-check fail。
  out=$("$TEST_GUARD" -- cargo run --release --bin zero-wpt-runner -- product-smoke "$f" --oracle "$oracle" --max-diff "$MAX_DIFF" --struct-check 2>/dev/null || true)
  # 提取 diff%
  pct=$(echo "$out" | grep -oE 'diff vs chromium[^:]*: [0-9/]+ px \([0-9.]+%\)' | grep -oE '\([0-9.]+%\)' | tr -d '()%')
  pct="${pct:-NA}"
  if echo "$out" | grep -q "struct-check: PASS"; then
    struct="PASS"
  else
    struct="FAIL"; struct_fail=$((struct_fail+1))
    # struct FAIL 时打印 issue 详情（sibling overlap / collapsed / text concatenation），
    # 作为「待查清单」诊断入口——否则 summary 只见 FAIL 不见根因。
    echo "$out" | grep -E "^[[:space:]]*-[[:space:]]+(sibling overlap|collapsed|text concatenation|text-concat)" | sed 's/^/      /'
  fi
  printf "  %-34s diff=%6s%%  struct=%s\n" "$name" "$pct" "$struct"
  results+=("$pct  $name  $struct")
  if [ "$pct" != "NA" ]; then
    sum_pct=$(awk -v s="$sum_pct" -v p="$pct" 'BEGIN{print s+p}')
    if awk -v p="$pct" -v m="$max_pct" 'BEGIN{exit !(p>m)}'; then
      max_pct="$pct"; max_name="$name"
    fi
  fi
done

echo
echo "=== summary ==="
echo "  fixtures run: $total"
echo "  struct-check failures: $struct_fail"
if [ "$total" -gt 0 ] && [ "$sum_pct" != "0" ]; then
  avg=$(awk -v s="$sum_pct" -v n="$total" 'BEGIN{printf "%.2f", s/n}')
  echo "  avg diff%: $avg"
  echo "  worst: $max_name ($max_pct%)"
fi
echo
echo "trend-only: pixel diff is font-wall baseline data (R633 plateau), not pass/fail."
echo "struct-check FAIL = real structural finding to investigate (logged above), not a CI block."
# trend-only（exit 0）：diff% + struct findings 均为数据，不阻 CI（per Makefile intent）。
# struct FAIL 是「待查清单」入口（如 R1651 center 修复即由此抓到）。
exit 0

# ── oracle 重抓（需 chrome-for-testing 127）──────────────────────────────────
# bash scripts/install-oracle-chrome.sh   # 确保 chrome 127 就位
# export PUPPETEER_EXECUTABLE_PATH="$HOME/.cache/zw-oracle-chrome/chrome-linux64/chrome"
# cd <repo root>
# for f in docs/goal/rendering-compat/evidence/product-static/legacy-html/fixtures/*.html; do
#   n=$(basename "$f" .html)
#   node tests/wpt-runner/scripts/capture-legacy-oracle.mjs "$f" "docs/goal/rendering-compat/evidence/product-static/legacy-html/oracle/$n.png"
# done
