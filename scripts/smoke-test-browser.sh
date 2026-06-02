#!/bin/bash
# Smoke test: 启动浏览器，验证不会立即崩溃
# 用法: ./scripts/smoke-test-browser.sh [wayland|x11]

set -e
BACKEND="${1:-wayland}"
TIMEOUT=8
RUST_LOG=info

echo "=== ZeroBrowser Smoke Test ($BACKEND) ==="

if [ "$BACKEND" = "wayland" ]; then
    echo "Testing Wayland backend..."
    timeout $TIMEOUT cargo run -p zero-browser -- --renderer=cpu 2>&1 | tee /tmp/zero-browser-smoke.log &
    PID=$!
elif [ "$BACKEND" = "x11" ]; then
    echo "Testing X11 backend..."
    WAYLAND_DISPLAY= WAYLAND_SOCKET= WINIT_UNIX_BACKEND=x11 \
        timeout $TIMEOUT cargo run -p zero-browser -- --renderer=cpu 2>&1 | tee /tmp/zero-browser-smoke.log &
    PID=$!
else
    echo "Unknown backend: $BACKEND"
    exit 1
fi

# 等它启动
sleep 5

# 检查进程还活着（没有崩溃）
if kill -0 $PID 2>/dev/null; then
    echo "✓ Browser still running after 5s — no crash"
    # 发送 SIGTERM，正常退出
    kill $PID 2>/dev/null || true
    wait $PID 2>/dev/null || true
    echo "✓ Browser exited cleanly"
else
    echo "✗ Browser crashed before 5s!"
    cat /tmp/zero-browser-smoke.log
    exit 1
fi

# 检查日志中没有 Broken pipe 或 panic
if grep -qE "Broken pipe|panicked|ERROR.*zero_browser" /tmp/zero-browser-smoke.log; then
    echo "✗ Found error in logs:"
    grep -E "Broken pipe|panicked|ERROR" /tmp/zero-browser-smoke.log
    exit 1
fi

echo "✓ No errors in logs"
echo "=== Smoke test PASSED ==="
