#!/bin/sh
# 桌面浏览器 M2 标签族/导航控制真实窗口演示流（docs/goal/desktop-browser.md M2）。
#
# 真实窗口（X11/Wayland；无人值守时经 xvfb-run 提供 display）内经真实输入路径驱动：
#   Ctrl+T 新标签 → 第二标签导航 → Ctrl+Tab 切回 → Alt+Left 后退 → Alt+Right 前进
#   → F5 刷新（epoch 前进断言）→ 标签拖拽重排（顺序 + 标签条可视变化断言）
#   → Ctrl+W 关闭（剩余标签断言）
# 状态断言（tab 顺序/活动标签/快照 URL/地址栏文本/导航 epoch）经 app_smoke_state
# 只读面在进程内完成，步骤截图落盘 .acceptance/ 供 evidence。
#
# 用法：bash scripts/desktop-browser-m2-smoke.sh
#   OUT_DIR=…     证据输出根目录（默认 .acceptance/desktop-browser-m2）
#   SKIP_BUILD=1  跳过 release 构建（二进制已就绪时）
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
OUT_DIR=${OUT_DIR:-"$ROOT/.acceptance/desktop-browser-m2"}
FIXTURE_DIR="$ROOT/examples/m2-tabs"
BIN="$ROOT/target/release/zero-browser"
GUARD="$ROOT/target/test-guard"
COMPOSITOR_BIN="$ROOT/target/release/zero-compositor"

log() { echo "desktop-browser-m2-smoke: $*"; }
die() { echo "desktop-browser-m2-smoke: FAIL: $*" >&2; exit 1; }

# --- 构建（经 test-guard 包裹，防失控 rustc 连累宿主） -----------------------
if [ "${SKIP_BUILD:-0}" != "1" ]; then
    log "building release product processes"
    "$GUARD" --time-limit 1200 -- cargo build --manifest-path "$ROOT/Cargo.toml" \
        --release -p zero-browser -p zero-renderer -p zero-compositor -p zero-image-decoder
fi
test -x "$BIN" || die "missing $BIN"
test -x "$GUARD" || die "missing $GUARD (run 'make test' once to build it)"

# --- 本地 HTTP 服务（/slow.html 延迟响应，供加载指示腿采样） -------------------
PORT_FILE=$(mktemp)
python3 - "$FIXTURE_DIR" "$PORT_FILE" <<'PYEOF' &
import functools
import http.server
import socketserver
import sys
import time

root, port_file = sys.argv[1], sys.argv[2]


class Handler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, fmt, *args):
        pass

    def do_GET(self):
        if self.path.split("?")[0] == "/slow.html":
            time.sleep(1.5)
        super().do_GET()

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
URL_ONE="http://127.0.0.1:$PORT/one.html"
URL_TWO="http://127.0.0.1:$PORT/two.html"

# --- display：无头环境自动补 Xvfb -------------------------------------------
USE_XVFB=0
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    command -v xvfb-run >/dev/null 2>&1 || die "no DISPLAY and xvfb-run unavailable"
    log "no display detected; using xvfb-run -a"
    USE_XVFB=1
fi

# --- 执行 ---------------------------------------------------------------------
# 断言锚：TAB_SMOKE_STEP ×7 passed + TAB_SMOKE_COMPLETE（状态断言在进程内，
# 截图与日志锚是 evidence 证据面）。
leg_dir="$OUT_DIR/tab-flow"
leg_log="$OUT_DIR/tab-flow.log"
rm -rf "$leg_dir"
mkdir -p "$leg_dir"

log "leg tab-flow: renderer=gpu url_one=$URL_ONE url_two=$URL_TWO"
LEG_ENV="RUST_LOG=info ZERO_BROWSER_PRODUCT_SMOKE=1 ZW_COMPOSITOR_BIN=$COMPOSITOR_BIN"
LEG_ARGS="--renderer=gpu --scale=1 --viewport-width=1024 --viewport-height=700 --tab-smoke-url-one=$URL_ONE --tab-smoke-url-two=$URL_TWO --tab-smoke-dir=$leg_dir"
if [ "$USE_XVFB" = "1" ]; then
    xvfb-run -a -s "-screen 0 1280x800x24" env $LEG_ENV \
        "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$leg_log" 2>&1
else
    env $LEG_ENV "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$leg_log" 2>&1
fi
test $? -eq 0 || die "leg tab-flow: browser exited non-zero (see $leg_log)"

for step in first-load new-tab tab-two switch-back in-tab-nav go-back go-forward reload tab-drag close-tab; do
    grep -aq "TAB_SMOKE_STEP step=$step" "$leg_log" \
        || die "step $step not recorded (see $leg_log)"
done
grep -aq "TAB_SMOKE_COMPLETE url_one=$URL_ONE url_two=$URL_TWO" "$leg_log" \
    || die "flow did not complete"
grep -aq "SMOKE_EVENT component=compositor_client status=Healthy" "$leg_log" \
    || die "compositor client not healthy"
grep -aqE "TAB_SMOKE_FAILURE|GUI_SMOKE_FAILURE|SMOKE_FAILURE|panicked at" "$leg_log" \
    && die "failure marker found in log"
grep -aq "Compositor disconnected" "$leg_log" \
    && die "compositor disconnected (fallback path engaged)"
for png in 01-new-tab.png 02-tab-two.png 03-switched-back.png 04-went-back.png \
    05-went-forward.png 06-reloaded.png 07-reordered.png 08-closed.png; do
    test -s "$leg_dir/$png" || die "missing screenshot $png"
done

log "PASS (leg: tab-flow)"
log "screenshots under $leg_dir/"

# --- 腿 2：地址栏（键入导航 / 自动补全 / 加载指示） ---------------------------
# 断言锚：ADDR_SMOKE_STEP（seed-load/typed-url/typed-nav/suggest-popup/suggest-nav/
# loading-indicator/slow-loaded）+ ADDR_SMOKE_COMPLETE + 零 fallback。
addr_dir="$OUT_DIR/addressbar-flow"
addr_log="$OUT_DIR/addressbar-flow.log"
rm -rf "$addr_dir"
mkdir -p "$addr_dir"

log "leg addressbar-flow: renderer=gpu base=http://127.0.0.1:$PORT"
LEG_ENV="RUST_LOG=info ZERO_BROWSER_PRODUCT_SMOKE=1 ZW_COMPOSITOR_BIN=$COMPOSITOR_BIN"
LEG_ARGS="--renderer=gpu --scale=1 --viewport-width=1024 --viewport-height=700 --addressbar-smoke-base=http://127.0.0.1:$PORT --addressbar-smoke-dir=$addr_dir"
if [ "$USE_XVFB" = "1" ]; then
    xvfb-run -a -s "-screen 0 1280x800x24" env $LEG_ENV \
        "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$addr_log" 2>&1
else
    env $LEG_ENV "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$addr_log" 2>&1
fi
test $? -eq 0 || die "leg addressbar-flow: browser exited non-zero (see $addr_log)"

for step in seed-load typed-url typed-nav suggest-popup suggest-nav loading-indicator slow-loaded; do
    grep -aq "step=$step" "$addr_log" || die "step $step not recorded (see $addr_log)"
done
grep -aq "ADDR_SMOKE_COMPLETE base=http://127.0.0.1:$PORT" "$addr_log" \
    || die "addressbar flow did not complete (see $addr_log)"
grep -aqE "ADDR_SMOKE_FAILURE|TAB_SMOKE_FAILURE|GUI_SMOKE_FAILURE|SMOKE_FAILURE|panicked at" "$addr_log" \
    && die "failure marker found in addressbar log"
grep -aq "Compositor disconnected" "$addr_log" \
    && die "compositor disconnected (fallback path engaged)"
for png in 01-typed-url.png 02-suggest-popup.png 03-suggest-loaded.png 04-loading.png 05-slow-loaded.png; do
    test -s "$addr_dir/$png" || die "missing screenshot $png"
done

log "PASS (leg: addressbar-flow)"
log "screenshots under $addr_dir/"
