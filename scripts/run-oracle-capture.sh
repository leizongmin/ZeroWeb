#!/bin/bash
# R1253: 抓取 chromium oracle-shots（DC-14 reftest-oracle 用的参考截图）。
#
# 背景：WSL2 + chromium 150（无 /dev/dri）headless 渲染路径 SIGTRAP，puppeteer.launch
# (headless) 崩。改用非 headless chromium（GUI 渲染路径：--user-data-dir 独立 profile
# + --remote-debugging-port + --ozone-platform=x11），capture-oracle-per-dir.mjs 经
# ORACLE_CDP_URL 连接它截图。headless 崩，GUI 路径能渲染。
#
# 用法:
#   ./scripts/run-oracle-capture.sh --category css/css-flexbox [--category css/css-grid ...] [--skip-existing]
#   make capture-oracle DIR=css/css-flexbox
#
# 抓完后 oracle-shots 存 tests/wpt-runner/oracle-shots/，再 make reftest-oracle DIR=...
# 跑 A/B（reftest-oracle 读存 PNG，不需 chromium）。
set -euo pipefail

PORT="${ORACLE_CDP_PORT:-9227}"
CHROME="${PUPPETEER_EXECUTABLE_PATH:-/usr/bin/chromium}"
TMP="${TMPDIR:-/tmp}"
USER_DATA="$TMP/zeroweb-chrome-oracle"
LOG="$TMP/zeroweb-chrome-oracle.log"
export DISPLAY="${DISPLAY:-:0}"

# 若已有 chromium 在该端口跑（复用），则不重启
if curl -s --max-time 2 "http://localhost:$PORT/json/version" >/dev/null 2>&1; then
  echo "[capture-oracle] 复用已运行的 chromium (CDP :$PORT)"
  CHROME_PID=""
else
  echo "[capture-oracle] 启动非 headless chromium (CDP :$PORT)..."
  "$CHROME" --user-data-dir="$USER_DATA" --no-sandbox --disable-setuid-sandbox \
    --ozone-platform=x11 --remote-debugging-port="$PORT" about:blank \
    >"$LOG" 2>&1 &
  CHROME_PID=$!
fi
cleanup() {
  if [ -n "$CHROME_PID" ]; then kill "$CHROME_PID" 2>/dev/null || true; wait "$CHROME_PID" 2>/dev/null || true; fi
}
trap cleanup EXIT

# 等 CDP 就绪（最多 ~15s）
ready=0
for i in $(seq 1 30); do
  if curl -s --max-time 2 "http://localhost:$PORT/json/version" >/dev/null 2>&1; then ready=1; break; fi
  sleep 0.5
done
if [ "$ready" -ne 1 ]; then
  echo "[capture-oracle] ❌ chromium 启动失败（见 $LOG）" >&2
  exit 1
fi
[ -n "$CHROME_PID" ] && echo "[capture-oracle] chromium 就绪 (PID $CHROME_PID)"

# 确保 puppeteer-core 装好
if [ ! -d node_modules/puppeteer-core ]; then
  echo "[capture-oracle] 装 puppeteer-core..."
  npm install puppeteer-core --no-save >/dev/null 2>&1 || { echo "❌ npm install puppeteer-core 失败" >&2; exit 1; }
fi

export ORACLE_CDP_URL="http://localhost:$PORT"
echo "[capture-oracle] ORACLE_CDP_URL=$ORACLE_CDP_URL，抓取: $*"
node tests/wpt-runner/scripts/capture-oracle-per-dir.mjs "$@"
