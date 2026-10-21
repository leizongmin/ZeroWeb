#!/bin/sh
# COMP-01 帧连续性基线探针（compositor-animation-continuity M1）。
#
# 用生产 headless 浏览器（zero-browser --headless，ZW_HEADLESS_GPU_SCREENSHOT=1 走
# 生产 GPU 光栅）加载 COMP-01 fixture，跨时间采样截图 + 脚本状态，产出：
#   <out>/comp01-probe.json、comp01-s{0..6}.png、headless.log
# 判定语义（消费方解读 JSON 的 verdict 字段）：
#   anim_idle=FROZEN       —— 静止页面下动画帧不推进（当前生产基线预期）
#   mutation_control=OK    —— DOM mutation 能产出新帧（发布链路活跃正控）
# 用法：scripts/compositor-animation-comp01-probe.sh [OUT_DIR]
# 前置：make browser-build（本脚本不代跑长构建，缺二进制时退出 2）。
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
FIXTURE="$ROOT/tests/fixtures/compositor-animation/comp01-transform-opacity-under-js-load.html"
DRIVER="$ROOT/scripts/compositor-animation-comp01-probe.mjs"
BROWSER_BIN="$ROOT/target/release/zero-browser"
OUT_DIR=${1:-"$ROOT/target/compositor-animation-comp01/$(date +%Y%m%d-%H%M%S)"}

test -x "$BROWSER_BIN" || {
    echo "comp01-probe: missing $BROWSER_BIN — run 'make browser-build' first" >&2
    exit 2
}
test -f "$FIXTURE" || { echo "comp01-probe: missing fixture $FIXTURE" >&2; exit 2; }
test -f "$DRIVER" || { echo "comp01-probe: missing driver $DRIVER" >&2; exit 2; }
command -v node >/dev/null || { echo "comp01-probe: node not found" >&2; exit 2; }

mkdir -p "$OUT_DIR"
LOG="$OUT_DIR/headless.log"

# 空闲回环端口（绑定后立即释放，存在竞窗；浏览器端口占用时报错可见即可）。
PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')

echo "comp01-probe: launching headless browser on port $PORT (out: $OUT_DIR)"
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
        echo "comp01-probe: browser exited during startup" >&2
        tail -20 "$LOG" >&2 || true
        exit 1
    fi
    sleep 0.2
    i=$((i + 1))
done
grep -q "ws://" "$LOG" || {
    echo "comp01-probe: browser did not report ws endpoint in time" >&2
    tail -20 "$LOG" >&2 || true
    exit 1
}

echo "comp01-probe: running frame-continuity sampling (bounded: 120s wall clock)"
timeout 120 node "$DRIVER" --url "ws://127.0.0.1:$PORT" --fixture "$FIXTURE" --out "$OUT_DIR"

test -s "$OUT_DIR/comp01-probe.json" || {
    echo "comp01-probe: missing probe JSON" >&2
    exit 1
}
echo "comp01-probe: artifacts in $OUT_DIR"
