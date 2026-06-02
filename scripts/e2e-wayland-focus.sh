#!/bin/bash
# e2e 测试：Wayland 下切换窗口不崩溃
# 连接到当前 Wayland compositor，通过启动/关闭第二个浏览器实例
# 来触发焦点切换，验证主浏览器进程不崩溃。
set -euo pipefail

BROWSER_LOG="/tmp/zero-browser-e2e-$$.log"
TIMEOUT=30
SWITCHES=5

cleanup() {
    kill %1 %2 2>/dev/null || true
    rm -f "$BROWSER_LOG"
}
trap cleanup EXIT

echo "=== ZeroBrowser Wayland Focus E2E ==="
echo "WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-wayland-0}"

# 1. 启动主浏览器进程
echo "[1/3] Starting browser..."
cargo run -p zero-browser -- --renderer=gpu 2>&1 | tee "$BROWSER_LOG" &
MAIN_PID=$!
sleep 3
if ! kill -0 $MAIN_PID 2>/dev/null; then
    echo "FAIL: Browser crashed on startup"
    exit 1
fi
echo "  PID=$MAIN_PID OK"

# 2. 反复切换焦点：启动第二个浏览器 → 关闭它 → 检查主浏览器还活着
echo "[2/3] Focus switches ($SWITCHES rounds)..."
for i in $(seq 1 $SWITCHES); do
    timeout 5 cargo run -p zero-browser -- --renderer=gpu 2>/dev/null &
    SWITCH_PID=$!
    sleep 2
    kill $SWITCH_PID 2>/dev/null || true
    wait $SWITCH_PID 2>/dev/null || true

    if ! kill -0 $MAIN_PID 2>/dev/null; then
        echo "FAIL: Browser crashed on switch #$i"
        grep -E "Broken pipe|ERROR|panicked" "$BROWSER_LOG" | tail -10
        exit 1
    fi
    echo "  Round $i: OK"
done

# 3. 检查
echo "[3/3] Checking logs..."
if grep -qE "Broken pipe|panicked" "$BROWSER_LOG"; then
    echo "FAIL: Errors in log:"
    grep -E "Broken pipe|panicked|ERROR.*event loop" "$BROWSER_LOG"
    exit 1
fi

kill $MAIN_PID 2>/dev/null || true
wait $MAIN_PID 2>/dev/null || true
echo "=== PASS ==="
