#!/bin/sh
# COMP-04a DC-4 探针（compositor-animation-continuity M3 / S1d 脚手架）。
#
# 用生产 headless 浏览器（zero-browser --headless）加载 COMP-04a fixture，采样
# 模式面 / worker 线程活性面 / canvas 转移面 / terminate 与错误路径面，产出：
# <out>/comp04a-probe.json、comp04a-s{0,1,2,3}.png、headless.log。
# 判定语义（消费方解读 JSON 的 verdict 字段）：
#   mode=shim   （预期，S1a 未落地）——wt 跨忙窗间隔 ≥800ms 复现 comp04 基线
#               （同 realm worker 与主线程同载），transfer=probe-only
#   mode=threaded（S1a 落地后）——wt 跨忙窗间隔 ≤200ms（worker 线程不被忙窗
#               阻塞）+ transfer ctxOk + terminate 后零 beat 回流 + 错误路径回流
#   呈现像素连续性不在本探针判定面：frame_dump 采集 + dc4-analyze.py 承载。
# 用法：scripts/compositor-animation-comp04a-probe.sh [OUT_DIR]
# 前置：make browser-build（本脚本不代跑长构建，缺二进制时退出 2）。
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
FIXTURE="$ROOT/tests/fixtures/compositor-animation/comp04a-worker-threaded-canvas.html"
DRIVER="$ROOT/scripts/compositor-animation-comp04a-probe.mjs"
BROWSER_BIN="$ROOT/target/release/zero-browser"
OUT_DIR=${1:-"$ROOT/target/compositor-animation-comp04a/$(date +%Y%m%d-%H%M%S)"}

test -x "$BROWSER_BIN" || {
    echo "comp04a-probe: missing $BROWSER_BIN — run 'make browser-build' first" >&2
    exit 2
}
test -f "$FIXTURE" || { echo "comp04a-probe: missing fixture $FIXTURE" >&2; exit 2; }
test -f "$DRIVER" || { echo "comp04a-probe: missing driver $DRIVER" >&2; exit 2; }
command -v node >/dev/null || { echo "comp04a-probe: node not found" >&2; exit 2; }

mkdir -p "$OUT_DIR"
LOG="$OUT_DIR/headless.log"

# 空闲回环端口（绑定后立即释放，存在竞窗；浏览器端口占用时报错可见即可）。
PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')

echo "comp04a-probe: launching headless browser on port $PORT (out: $OUT_DIR)"
ZW_HEADLESS_GPU_SCREENSHOT=1 RUST_LOG=info \
    "$BROWSER_BIN" --headless --remote-debugging-port="$PORT" \
    --viewport-width=480 --viewport-height=360 >"$LOG" 2>&1 &
BROWSER_PID=$!
cleanup() {
    # 只清理本次启动的进程树（浏览器 + 其 renderer 子进程）。
    kill "$BROWSER_PID" 2>/dev/null || true
    pkill -P "$BROWSER_PID" 2>/dev/null || true
    wait "$BROWSER_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

# 等服务端打印 ws:// 地址（最长 20s）。
i=0
while [ $i -lt 100 ]; do
    if grep -q "ws://" "$LOG" 2>/dev/null; then
        break
    fi
    if ! kill -0 "$BROWSER_PID" 2>/dev/null; then
        echo "comp04a-probe: browser exited during startup" >&2
        tail -20 "$LOG" >&2 || true
        exit 1
    fi
    sleep 0.2
    i=$((i + 1))
done
grep -q "ws://" "$LOG" || {
    echo "comp04a-probe: browser did not report ws endpoint in time" >&2
    tail -20 "$LOG" >&2 || true
    exit 1
}

echo "comp04a-probe: running dc4 threaded-worker sampling (bounded: 120s wall clock)"
timeout 120 node "$DRIVER" --url "ws://127.0.0.1:$PORT" --fixture "$FIXTURE" --out "$OUT_DIR"

test -s "$OUT_DIR/comp04a-probe.json" || {
    echo "comp04a-probe: missing probe JSON" >&2
    exit 1
}
echo "comp04a-probe: artifacts in $OUT_DIR"
