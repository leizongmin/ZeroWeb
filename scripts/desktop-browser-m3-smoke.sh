#!/bin/sh
# 桌面浏览器 M3 下载管理器真实窗口演示流（docs/goal/desktop-browser.md M3）。
#
# 真实窗口（X11/Wayland；无人值守时经 xvfb-run 提供 display）内经真实输入路径驱动：
#   页面链接点击 → attachment 响应拦截 → 文件落盘 + 下载管理器记账（Pending →
#   Downloading → Completed）→ 面板自动弹出 → 浏览器内下载页呈现 → 面板
#   「Show in folder」按钮真实点击 → 打开下载目录动作
# 下载落盘隔离到演示流目录（流程结束恢复用户设置空值，不写真实 Downloads）。
#
# 用法：bash scripts/desktop-browser-m3-smoke.sh
#   OUT_DIR=…     证据输出根目录（默认 .acceptance/desktop-browser-m3）
#   SKIP_BUILD=1  跳过 release 构建（二进制已就绪时）
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
OUT_DIR=${OUT_DIR:-"$ROOT/.acceptance/desktop-browser-m3"}
FIXTURE_DIR="$ROOT/examples/m3-downloads"
BIN="$ROOT/target/release/zero-browser"
GUARD="$ROOT/target/test-guard"
COMPOSITOR_BIN="$ROOT/target/release/zero-compositor"

log() { echo "desktop-browser-m3-smoke: $*"; }
die() { echo "desktop-browser-m3-smoke: FAIL: $*" >&2; exit 1; }

# --- 构建（经 test-guard 包裹，防失控 rustc 连累宿主） -----------------------
if [ "${SKIP_BUILD:-0}" != "1" ]; then
    log "building release product processes"
    "$GUARD" --time-limit 1200 -- cargo build --manifest-path "$ROOT/Cargo.toml" \
        --release -p zero-browser -p zero-renderer -p zero-compositor -p zero-image-decoder
fi
test -x "$BIN" || die "missing $BIN"
test -x "$GUARD" || die "missing $GUARD (run 'make test' once to build it)"

# --- 本地 HTTP 服务（/file.zip 带 attachment 头） ------------------------------
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

    def do_GET(self):
        self._is_attachment = self.path.split("?")[0] == "/file.zip"
        super().do_GET()

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        if getattr(self, "_is_attachment", False):
            self.send_header("Content-Disposition", 'attachment; filename="file.zip"')
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
BASE="http://127.0.0.1:$PORT"

# --- display：无头环境自动补 Xvfb -------------------------------------------
USE_XVFB=0
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    command -v xvfb-run >/dev/null 2>&1 || die "no DISPLAY and xvfb-run unavailable"
    log "no display detected; using xvfb-run -a"
    USE_XVFB=1
fi

# --- 执行 ---------------------------------------------------------------------
leg_dir="$OUT_DIR/download-flow"
leg_log="$OUT_DIR/download-flow.log"
rm -rf "$leg_dir"
mkdir -p "$leg_dir"

log "leg download-flow: renderer=gpu base=$BASE"
LEG_ENV="RUST_LOG=info ZERO_BROWSER_PRODUCT_SMOKE=1 ZW_COMPOSITOR_BIN=$COMPOSITOR_BIN"
LEG_ARGS="--renderer=gpu --scale=1 --viewport-width=1024 --viewport-height=700 --download-smoke-base=$BASE --download-smoke-dir=$leg_dir"
if [ "$USE_XVFB" = "1" ]; then
    xvfb-run -a -s "-screen 0 1280x800x24" env $LEG_ENV \
        "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$leg_log" 2>&1
else
    env $LEG_ENV "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$leg_log" 2>&1
fi
test $? -eq 0 || die "leg download-flow: browser exited non-zero (see $leg_log)"

for step in page-load download-done downloads-page show-in-folder; do
    grep -aq "DL_SMOKE_STEP step=$step" "$leg_log" || die "step $step not recorded (see $leg_log)"
done
grep -aq "ADDR_SMOKE\|TAB_SMOKE" "$leg_log" && die "unrelated smoke markers in log"
grep -aq "DL_SMOKE_COMPLETE base=$BASE" "$leg_log" || die "flow did not complete"
grep -aq "SMOKE_EVENT component=browser event=download_completed" "$leg_log" \
    || die "download_completed event not recorded"
grep -aq "SMOKE_EVENT component=browser event=download_show_in_folder" "$leg_log" \
    || die "download_show_in_folder event not recorded"
grep -aqE "DL_SMOKE_FAILURE|ADDR_SMOKE_FAILURE|TAB_SMOKE_FAILURE|SMOKE_FAILURE|panicked at" "$leg_log" \
    && die "failure marker found in log"
grep -aq "Compositor disconnected" "$leg_log" \
    && die "compositor disconnected (fallback path engaged)"
for png in 01-download-panel.png 02-downloads-page.png 03-show-in-folder.png; do
    test -s "$leg_dir/$png" || die "missing screenshot $png"
done

# 落盘字节与 fixture 源文件逐字节一致（确定性内容）。
cmp -s "$leg_dir/files/file.zip" "$FIXTURE_DIR/file.zip" \
    || die "downloaded file bytes differ from fixture source"

log "PASS (leg: download-flow)"
log "screenshots under $leg_dir/"

# --- 腿 2：右键上下文菜单 + 缩放联动 -------------------------------------------
# 断言锚：MZ_SMOKE_STEP（context-menu/menu-reload/inspect-tab/selection-copy/zoom-in）
# + MZ_SMOKE_COMPLETE + 零 fallback。
mz_dir="$OUT_DIR/menu-zoom-flow"
mz_log="$OUT_DIR/menu-zoom-flow.log"
rm -rf "$mz_dir"
mkdir -p "$mz_dir"

log "leg menu-zoom-flow: renderer=gpu base=$BASE"
LEG_ENV="RUST_LOG=info ZERO_BROWSER_PRODUCT_SMOKE=1 ZW_COMPOSITOR_BIN=$COMPOSITOR_BIN"
LEG_ARGS="--renderer=gpu --scale=1 --viewport-width=1024 --viewport-height=700 --menu-zoom-smoke-base=$BASE --menu-zoom-smoke-dir=$mz_dir"
if [ "$USE_XVFB" = "1" ]; then
    xvfb-run -a -s "-screen 0 1280x800x24" env $LEG_ENV \
        "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$mz_log" 2>&1
else
    env $LEG_ENV "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$mz_log" 2>&1
fi
test $? -eq 0 || die "leg menu-zoom-flow: browser exited non-zero (see $mz_log)"

for step in seed-load context-menu menu-reload inspect-tab selection-copy zoom-in; do
    grep -aq "MZ_SMOKE_STEP step=$step" "$mz_log" || die "step $step not recorded (see $mz_log)"
done
grep -aq "MZ_SMOKE_COMPLETE base=$BASE" "$mz_log" || die "menu/zoom flow did not complete"
grep -aqE "MZ_SMOKE_FAILURE|DL_SMOKE_FAILURE|SMOKE_FAILURE|panicked at" "$mz_log" \
    && die "failure marker found in menu/zoom log"
grep -aq "Compositor disconnected" "$mz_log" \
    && die "compositor disconnected (fallback path engaged)"
for png in 01-context-menu.png 02-inspect-tab.png 03-zoomed.png; do
    test -s "$mz_dir/$png" || die "missing screenshot $png"
done

log "PASS (leg: menu-zoom-flow)"
log "screenshots under $mz_dir/"
