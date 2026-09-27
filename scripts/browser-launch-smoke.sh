#!/bin/sh
# ZeroBrowser 无头启动冒烟（DC-1「三平台可编译启动」CI 矩阵佐证 + 本地可跑）。
#
# 流程：后台启动 zero-browser --headless（不开窗，runner 无需 display）→ 轮询
# /json/version 断言 Browser: ZeroWeb/ 与 webSocketDebuggerUrl → /json 断言
# page target 枚举 → 杀进程收尾。renderer/compositor/image-decoder 子进程 bin
# 须已与 zero-browser 同目录（CI 矩阵既有步骤已保证；resolve_renderer_binary 按
# current_exe 上溯查找）。
#
# 用法：bash scripts/browser-launch-smoke.sh <zero-browser 所在目录>
#   ZW_LAUNCH_SMOKE_PORT=9333  探活端口（默认 9333，runner 隔离环境无碰撞面）
set -eu

BIN_DIR=$1
[ -n "$BIN_DIR" ] || { echo "launch-smoke: usage: $0 <bin-dir>" >&2; exit 2; }

BIN="$BIN_DIR/zero-browser"
[ -f "$BIN" ] || BIN="$BIN_DIR/zero-browser.exe"
[ -f "$BIN" ] || { echo "launch-smoke: zero-browser not found in $BIN_DIR" >&2; exit 2; }

PORT=${ZW_LAUNCH_SMOKE_PORT:-9333}
LOG=$(mktemp)
cleanup() {
    [ -n "${BPID:-}" ] && kill "$BPID" 2>/dev/null || true
    sleep 1
    [ -n "${BPID:-}" ] && kill -9 "$BPID" 2>/dev/null || true
    rm -f "$LOG"
}
trap cleanup EXIT INT TERM

echo "launch-smoke: booting $BIN --headless on port $PORT"
"$BIN" --headless "--remote-debugging-port=$PORT" >"$LOG" 2>&1 &
BPID=$!

VERSION_BODY=""
i=0
while [ "$i" -lt 60 ]; do
    if VERSION_BODY=$(curl -fsS --max-time 2 "http://127.0.0.1:$PORT/json/version" 2>/dev/null); then
        break
    fi
    i=$((i + 1))
    sleep 1
done
[ -n "$VERSION_BODY" ] || {
    echo "launch-smoke: FAIL: /json/version unreachable after ${i}s" >&2
    sed 's/^/  log| /' "$LOG" >&2
    exit 1
}
echo "$VERSION_BODY" | grep -q '"Browser":"ZeroWeb/' \
    || { echo "launch-smoke: FAIL: Browser field missing: $VERSION_BODY" >&2; exit 1; }
echo "$VERSION_BODY" | grep -q '"webSocketDebuggerUrl"' \
    || { echo "launch-smoke: FAIL: webSocketDebuggerUrl missing: $VERSION_BODY" >&2; exit 1; }

TARGETS=$(curl -fsS --max-time 2 "http://127.0.0.1:$PORT/json" 2>/dev/null || true)
echo "$TARGETS" | grep -q '"type":"page"' \
    || { echo "launch-smoke: FAIL: no page target in /json: $TARGETS" >&2; exit 1; }

echo "launch-smoke: PASS ($VERSION_BODY)"
