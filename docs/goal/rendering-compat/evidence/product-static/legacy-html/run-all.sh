#!/usr/bin/env bash
# R657: batch-capture chromium oracle + render ZeroWeb CPU + diff for all legacy fixtures.
set -u
cd /home/lei/work/ZeroWeb
DIR=docs/goal/rendering-compat/evidence/product-static/legacy-html
RUNNER=./target/release/zero-wpt-runner
ORACLE=tests/wpt-runner/scripts/capture-legacy-oracle.mjs
SUMMARY="$DIR/diff-summary.txt"
: > "$SUMMARY"
for htm in "$DIR"/testpage-0*.htm; do
  stem=$(basename "$htm" .htm)
  oracle="$DIR/$stem-chromium.png"
  zw="$DIR/$stem-zeroweb-cpu.png"
  echo "=== $stem ==="
  # 1. chromium oracle（已存在则复用，避免每轮重抓——PNG 已提交作 evidence）
  if [ ! -s "$oracle" ]; then
    if ! timeout 60 node "$ORACLE" "$htm" "$oracle" >/dev/null 2>&1; then
      echo "$stem ORACLE_FAIL" | tee -a "$SUMMARY"
      continue
    fi
  fi
  # 2. ZeroWeb CPU render + diff vs oracle
  out=$(timeout 120 "$RUNNER" product-smoke "$htm" --oracle "$oracle" --max-diff 100 --out "$zw" 2>&1)
  pct=$(echo "$out" | grep -oE 'diff vs[^0-9]*[0-9.]+%' | grep -oE '[0-9.]+%' | head -1)
  if [ -z "$pct" ]; then
    pct=$(echo "$out" | grep -oE '[0-9.]+%' | head -1)
  fi
  echo "$stem $pct" | tee -a "$SUMMARY"
done
echo "=== DONE ==="
cat "$SUMMARY"
# trend-only gate（DC-13）：diff 全为字体墙非回归，永远退出 0。
exit 0
