#!/bin/sh
# 桌面浏览器 M1 真窗口主链路演示流（docs/goal/desktop-browser.md M1）。
#
# 在 Linux 真实窗口（X11/Wayland；无人值守时经 xvfb-run 提供 display）内端到端
# 验收：启动 → 加载 URL → 渲染 → 输入交互（滚动 / 缩放 / 刷新），三腿覆盖：
#   A. cpu renderer + compositor     —— 主链路 + compositor 帧事件链
#   B. gpu renderer（默认旗标）      —— GPU 窗口 surface 呈现 + compositor 帧消费
#   C. gpu renderer + GPU 导入腿     —— ZW_COMPOSITOR_SCROLL_TRANSFORM=0 时
#                                       compositor GPU 纹理导出 → browser 导入
#                                       （compositor_dmabuf_adopted / gpu_direct）
# 每腿由 gui_smoke 状态机驱动：加载帧可视内容断言 + 滚动/缩放可视变化断言 + 步骤
# 截图落盘（.acceptance/ 下），日志锚断言在本脚本内复核。
#
# 用法：bash scripts/desktop-browser-m1-smoke.sh
#   OUT_DIR=…     证据输出根目录（默认 .acceptance/desktop-browser-m1）
#   SKIP_BUILD=1  跳过 release 构建（二进制已就绪时）
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
OUT_DIR=${OUT_DIR:-"$ROOT/.acceptance/desktop-browser-m1"}
FIXTURE_DIR="$ROOT/examples/m1-real-window"
BIN="$ROOT/target/release/zero-browser"
GUARD="$ROOT/target/test-guard"
COMPOSITOR_BIN="$ROOT/target/release/zero-compositor"

log() { echo "desktop-browser-m1-smoke: $*"; }
die() { echo "desktop-browser-m1-smoke: FAIL: $*" >&2; exit 1; }

# --- 构建（经 test-guard 包裹，防失控 rustc 连累宿主） -----------------------
if [ "${SKIP_BUILD:-0}" != "1" ]; then
    log "building release product processes"
    "$GUARD" --time-limit 1200 -- cargo build --manifest-path "$ROOT/Cargo.toml" \
        --release -p zero-browser -p zero-renderer -p zero-compositor -p zero-image-decoder
fi
test -x "$BIN" || die "missing $BIN"
test -x "$GUARD" || die "missing $GUARD (run 'cargo build --release -p test-guard' or make test once)"

# --- 本地 HTTP 服务（file:// 会短路输入交互腿，滚动/缩放/刷新需 http） -------
PORT_FILE=$(mktemp)
python3 - "$FIXTURE_DIR" "$PORT_FILE" <<'PYEOF' &
import functools
import http.server
import socketserver
import sys

root, port_file = sys.argv[1], sys.argv[2]


class Handler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, fmt, *args):
        pass

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()


class Server(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


srv = Server(("127.0.0.1", 0), functools.partial(Handler, directory=root))
with open(port_file, "w") as fh:
    fh.write(str(srv.server_address[1]))
srv.serve_forever()
PYEOF
SERVER_PID=$!
cleanup() { kill "$SERVER_PID" 2>/dev/null || true; rm -f "$PORT_FILE"; }
trap cleanup EXIT INT TERM

for _ in $(seq 1 50); do
    [ -s "$PORT_FILE" ] && break
    sleep 0.1
done
PORT=$(cat "$PORT_FILE")
[ -n "$PORT" ] || die "local http server did not report a port"
URL="http://127.0.0.1:$PORT/index.html"

# --- display：无头环境自动补 Xvfb -------------------------------------------
USE_XVFB=0
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    command -v xvfb-run >/dev/null 2>&1 || die "no DISPLAY and xvfb-run unavailable"
    log "no display detected; using xvfb-run -a"
    USE_XVFB=1
fi

# --- 腿执行 -----------------------------------------------------------------
# 断言说明：
#   GUI_SMOKE_STEP ×4 passed        加载/滚动/缩放/刷新步骤各自拿到可视帧
#   GUI_SMOKE_ASSERT visual_change  滚动与缩放确实改变了页面像素签名
#   GUI_SMOKE_COMPLETE              状态机走到终点
#   compositor 事件链               Healthy → submitted → committed → completed → adopted
run_leg() {
    leg=$1
    renderer=$2
    shift 2
    leg_dir="$OUT_DIR/$leg"
    leg_log="$OUT_DIR/$leg.log"
    rm -rf "$leg_dir"
    mkdir -p "$leg_dir"

    log "leg $leg: renderer=$renderer url=$URL"
    LEG_ENV="RUST_LOG=info ZERO_BROWSER_PRODUCT_SMOKE=1 ZW_COMPOSITOR_BIN=$COMPOSITOR_BIN ${1:-}"
    LEG_ARGS="--renderer=$renderer --scale=1 --viewport-width=1024 --viewport-height=700 --gui-smoke-url=$URL --gui-smoke-dir=$leg_dir"
    if [ "$USE_XVFB" = "1" ]; then
        xvfb-run -a -s "-screen 0 1280x800x24" env $LEG_ENV \
            "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$leg_log" 2>&1
    else
        env $LEG_ENV "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$leg_log" 2>&1
    fi
    test $? -eq 0 || die "leg $leg: browser exited non-zero (see $leg_log)"

    for step in loaded scrolled zoomed reloaded; do
        grep -aq "GUI_SMOKE_STEP step=$step status=passed" "$leg_log" \
            || die "leg $leg: step $step not passed (see $leg_log)"
    done
    grep -aq "GUI_SMOKE_ASSERT action=scroll visual_change=passed" "$leg_log" \
        || die "leg $leg: scroll visual change not asserted"
    grep -aq "GUI_SMOKE_ASSERT action=zoom_in visual_change=passed" "$leg_log" \
        || die "leg $leg: zoom visual change not asserted"
    grep -aq "GUI_SMOKE_COMPLETE url=$URL steps=load,scroll,zoom_in,reload" "$leg_log" \
        || die "leg $leg: flow did not complete"
    grep -aq "SMOKE_EVENT component=compositor_client status=Healthy" "$leg_log" \
        || die "leg $leg: compositor client not healthy"
    grep -aq "SMOKE_EVENT component=compositor_client event=frame_submitted" "$leg_log" \
        || die "leg $leg: no compositor frame submitted"
    grep -aq "SMOKE_EVENT component=zero-compositor event=frame_committed" "$leg_log" \
        || die "leg $leg: no compositor frame committed"
    grep -aq "SMOKE_EVENT component=compositor_client event=frame_completed" "$leg_log" \
        || die "leg $leg: no compositor frame completed"
    grep -aqE "GUI_SMOKE_FAILURE|SMOKE_FAILURE|panicked at" "$leg_log" \
        && die "leg $leg: failure marker found in log"
    grep -aq "Compositor disconnected" "$leg_log" \
        && die "leg $leg: compositor disconnected (fallback path engaged)"
    grep -aq "legacy_view_painted" "$leg_log" \
        && die "leg $leg: legacy frame consumed (compositor path bypassed)"
    grep -aq "fallback=true" "$leg_log" \
        && die "leg $leg: capture fell back (fallback=true)"
    for png in 01-loaded.png 02-scrolled.png 03-zoomed.png 04-reloaded.png; do
        test -s "$leg_dir/$png" || die "leg $leg: missing screenshot $png"
    done
}

run_leg a-cpu-compositor cpu ""
grep -aq "SMOKE_EVENT component=browser event=compositor_bitmap_adopted" "$OUT_DIR/a-cpu-compositor.log" \
    || die "leg a: compositor bitmap not adopted"

run_leg b-gpu-window gpu ""
grep -aq "GPU renderer initialized" "$OUT_DIR/b-gpu-window.log" \
    || die "leg b: GPU renderer did not initialize"
grep -aq "SMOKE_EVENT component=browser event=compositor_bitmap_adopted" "$OUT_DIR/b-gpu-window.log" \
    || die "leg b: compositor frame not consumed in GPU window"

# 腿 C：关 compositor 侧滚动变换，走 GPU 纹理导出 → browser 导入（gpu_direct）。
run_leg c-gpu-direct gpu ZW_COMPOSITOR_SCROLL_TRANSFORM=0
grep -aq "SMOKE_EVENT component=browser event=compositor_dmabuf_adopted" "$OUT_DIR/c-gpu-direct.log" \
    || die "leg c: GPU dma-buf frame not adopted"

log "PASS (legs: a-cpu-compositor, b-gpu-window, c-gpu-direct)"
log "screenshots under $OUT_DIR/<leg>/"
