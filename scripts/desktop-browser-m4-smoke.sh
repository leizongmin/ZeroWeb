#!/bin/sh
# 桌面浏览器 M4 数据面真实窗口演示流（docs/goal/desktop-browser.md M4）。
#
# 真实窗口（X11/Wayland；无人值守时经 xvfb-run 提供 display）内经真实输入路径驱动：
#   书签栏开关（内部设置 URL）→ 工具栏菜单点击「添加书签」→ 书签栏条目点击打开
#   → 条目右键删除 → zero://history 呈现 → zero://history/clear 清空 → 设置
#   搜索引擎切 DuckDuckGo → 隐私 do_not_track 开关 → 设置 home_url 为本地
#   fixture → Alt+Home 主页导航落地（DC-2 主页按钮项收口）
# 自清洁：流程结束恢复搜索引擎/隐私/主页/书签栏设置为默认，不留痕。
#
# 用法：bash scripts/desktop-browser-m4-smoke.sh
#   OUT_DIR=…     证据输出根目录（默认 .acceptance/desktop-browser-m4）
#   SKIP_BUILD=1  跳过 release 构建（二进制已就绪时）
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
OUT_DIR=${OUT_DIR:-"$ROOT/.acceptance/desktop-browser-m4"}
FIXTURE_DIR="$ROOT/examples/m4-data"
BIN="$ROOT/target/release/zero-browser"
GUARD="$ROOT/target/test-guard"
COMPOSITOR_BIN="$ROOT/target/release/zero-compositor"

log() { echo "desktop-browser-m4-smoke: $*"; }
die() { echo "desktop-browser-m4-smoke: FAIL: $*" >&2; exit 1; }

# --- 构建（经 test-guard 包裹，防失控 rustc 连累宿主） -----------------------
if [ "${SKIP_BUILD:-0}" != "1" ]; then
    log "building release product processes"
    "$GUARD" --time-limit 1200 -- cargo build --manifest-path "$ROOT/Cargo.toml" \
        --release -p zero-browser -p zero-renderer -p zero-compositor -p zero-image-decoder
fi
test -x "$BIN" || die "missing $BIN"
test -x "$GUARD" || die "missing $GUARD (run 'make test' once to build it)"

# --- 本地 HTTP 服务 -----------------------------------------------------------
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
BASE="http://127.0.0.1:$PORT"

# --- display：无头环境自动补 Xvfb -------------------------------------------
USE_XVFB=0
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    command -v xvfb-run >/dev/null 2>&1 || die "no DISPLAY and xvfb-run unavailable"
    log "no display detected; using xvfb-run -a"
    USE_XVFB=1
fi

# --- 执行 ---------------------------------------------------------------------
leg_dir="$OUT_DIR/data-flow"
leg_log="$OUT_DIR/data-flow.log"
rm -rf "$leg_dir"
mkdir -p "$leg_dir"

log "leg data-flow: renderer=gpu base=$BASE"
# XDG_CONFIG_HOME 隔离：书签/历史/设置走演示流私有 profile，不读写用户真实数据。
LEG_ENV="RUST_LOG=info ZERO_BROWSER_PRODUCT_SMOKE=1 ZW_COMPOSITOR_BIN=$COMPOSITOR_BIN XDG_CONFIG_HOME=$leg_dir/profile"
LEG_ARGS="--renderer=gpu --scale=1 --viewport-width=1024 --viewport-height=700 --data-smoke-base=$BASE --data-smoke-dir=$leg_dir"
if [ "$USE_XVFB" = "1" ]; then
    xvfb-run -a -s "-screen 0 1280x800x24" env $LEG_ENV \
        "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$leg_log" 2>&1
else
    env $LEG_ENV "$GUARD" --time-limit 300 -- "$BIN" $LEG_ARGS >"$leg_log" 2>&1
fi
test $? -eq 0 || die "leg data-flow: browser exited non-zero (see $leg_log)"

for step in seed-load bookmark-added bar-open bookmark-deleted \
    history-page history-cleared search-engine do-not-track home-set home-nav; do
    grep -aq "DATA_SMOKE_STEP step=$step" "$leg_log" || die "step $step not recorded (see $leg_log)"
done
grep -aq "DATA_SMOKE_COMPLETE base=$BASE" "$leg_log" || die "flow did not complete"
grep -aqE "DATA_SMOKE_FAILURE|FIND_SMOKE_FAILURE|SMOKE_FAILURE|panicked at" "$leg_log" \
    && die "failure marker found in log"
grep -aq "Compositor disconnected" "$leg_log" \
    && die "compositor disconnected (fallback path engaged)"
for png in 01-bookmark-added.png 02-history-cleared.png 03-home-nav.png; do
    test -s "$leg_dir/$png" || die "missing screenshot $png"
done

log "PASS (leg: data-flow)"
log "screenshots under $leg_dir/"
