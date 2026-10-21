#!/bin/sh
# COMP-04 worker canvas 基线探针（compositor-animation-continuity M3）。
#
# 用生产 headless 浏览器（zero-browser --headless）加载 COMP-04 fixture，采样
# worker OffscreenCanvas 能力面 / transferControlToOffscreen API 面 / 主线程忙窗
# 并发面，产出：<out>/comp04-probe.json、comp04-s{0..3}.png、headless.log。
# 判定语义（消费方解读 JSON 的 verdict 字段，基线预期）：
#   worker_probe ctx=null           —— worker 内 OffscreenCanvas 无 host 桥（同 realm shim）
#   transfer_control_api=undefined  —— canvas→worker 转移入口未实现
#   main_offscreen_2d_subset=pass   —— 主线程 OffscreenCanvas 2D 子集可用
#   beat_gap_in_load_ms >> 50       —— 忙窗内 worker beat 停顿（worker 与主线程同载）
#   draws_in_load=0                 —— 忙窗内主线程 canvas 绘制停摆
# 用法：scripts/compositor-animation-comp04-probe.sh [OUT_DIR]
# 前置：make browser-build（本脚本不代跑长构建，缺二进制时退出 2）。
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
FIXTURE="$ROOT/tests/fixtures/compositor-animation/comp04-worker-canvas-transfer.html"
DRIVER="$ROOT/scripts/compositor-animation-comp04-probe.mjs"
BROWSER_BIN="$ROOT/target/release/zero-browser"
OUT_DIR=${1:-"$ROOT/target/compositor-animation-comp04/$(date +%Y%m%d-%H%M%S)"}

test -x "$BROWSER_BIN" || {
    echo "comp04-probe: missing $BROWSER_BIN — run 'make browser-build' first" >&2
    exit 2
}
test -f "$FIXTURE" || { echo "comp04-probe: missing fixture $FIXTURE" >&2; exit 2; }
test -f "$DRIVER" || { echo "comp04-probe: missing driver $DRIVER" >&2; exit 2; }
command -v node >/dev/null || { echo "comp04-probe: node not found" >&2; exit 2; }

mkdir -p "$OUT_DIR"
LOG="$OUT_DIR/headless.log"

# 空闲回环端口（绑定后立即释放，存在竞窗；浏览器端口占用时报错可见即可）。
PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')

echo "comp04-probe: launching headless browser on port $PORT (out: $OUT_DIR)"
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
        echo "comp04-probe: browser exited during startup" >&2
        tail -20 "$LOG" >&2 || true
        exit 1
    fi
    sleep 0.2
    i=$((i + 1))
done
grep -q "ws://" "$LOG" || {
    echo "comp04-probe: browser did not report ws endpoint in time" >&2
    tail -20 "$LOG" >&2 || true
    exit 1
}

echo "comp04-probe: running worker-canvas baseline sampling (bounded: 120s wall clock)"
timeout 120 node "$DRIVER" --url "ws://127.0.0.1:$PORT" --fixture "$FIXTURE" --out "$OUT_DIR"

test -s "$OUT_DIR/comp04-probe.json" || {
    echo "comp04-probe: missing probe JSON" >&2
    exit 1
}
echo "comp04-probe: artifacts in $OUT_DIR"
