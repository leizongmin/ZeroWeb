#!/bin/sh
# background-resource-lifecycle goal M1 单腿观测编排：
# fixture server + 隔离 profile headless zero-browser + 进程树采样器 + CDP 场景驱动。
#
# 用法：scripts/background-resource-observe.sh <scenario> <out-dir> [wall-seconds]
#   scenario ∈ bg01-foreground | bg01-background | bg01-interval |
#              bg02-anim | bg02-static | bg02-rafstub | bg02-media |
#              bg03-nav | bg04-cycle
#
# 产物（out-dir）：run-config.txt、browser.log、http.log、samples.jsonl、
#   probe.json、frames/（compositor frame dump，bg02 组）。
# 墙钟：浏览器 timeout = probe wall + 25s 余量。隔离：一次性 XDG_CONFIG_HOME。
# 无人值守请外层包 systemd-run 内存 scope（run-rules §6）。
#
# 前置：make browser-build 已产出 target/release/{zero-browser,zero-compositor}。
set -eu
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
SCENARIO="${1:?scenario required}"
OUT="${2:?out-dir required}"
WALL="${3:-30}"

case "$OUT" in
    /*) ;;
    *) OUT="$PWD/$OUT" ;;
esac
FIXDIR="$ROOT/tests/fixtures/background-resource"
BROWSER="$ROOT/target/release/zero-browser"
COMPOSITOR="$ROOT/target/release/zero-compositor"
[ -x "$BROWSER" ] || { echo "missing $BROWSER (run make browser-build)" >&2; exit 2; }

HTTP_PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')
CDP_PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')

# 场景 → 启动 URL / env
case "$SCENARIO" in
    bg01-foreground) START_URL="bg01-nested-timer.html";  RAF_DRIVEN=0 ;;
    bg01-background) START_URL="bg01-nested-timer.html";  RAF_DRIVEN=0 ;;
    bg01-interval)   START_URL="bg01-interval-timer.html"; RAF_DRIVEN=0 ;;
    bg02-anim)       START_URL="bg02-offscreen-anim.html"; RAF_DRIVEN=1 ;;
    bg02-static)     START_URL="bg02-static.html";        RAF_DRIVEN=1 ;;
    bg02-rafstub)    START_URL="bg02-offscreen-anim.html"; RAF_DRIVEN=0 ;;
    bg02-anim-gui)   START_URL="bg02-offscreen-anim.html"; RAF_DRIVEN=1; GUI=1 ;;
    bg02-static-gui) START_URL="bg02-static.html";        RAF_DRIVEN=1; GUI=1 ;;
    bg02-anim-compgui) START_URL="bg02-offscreen-anim.html"; RAF_DRIVEN=1; GUI=1; COMP_ANIM=1 ;;
    bg02-onscreen-compgui) START_URL="bg02-onscreen-anim.html"; RAF_DRIVEN=1; GUI=1; COMP_ANIM=1 ;;
    bg02-media)      START_URL="bg02-media.html";         RAF_DRIVEN=0 ;;
    bg02-media-autoplay) START_URL="bg02-media-autoplay.html"; RAF_DRIVEN=0 ;;
    bg03-nav)        START_URL="bg03-nav-source.html";    RAF_DRIVEN=0; WALL="${3:-45}" ;;
    bg04-cycle)      START_URL="bg02-static.html";        RAF_DRIVEN=0; WALL=40 ;;
    *) echo "unknown scenario: $SCENARIO" >&2; exit 2 ;;
esac
GUI="${GUI:-0}"
COMP_ANIM="${COMP_ANIM:-0}"

mkdir -p "$OUT" "$OUT/profile" "$OUT/frames"
cd "$ROOT"

echo "scenario=$SCENARIO wall=${WALL}s cdp=127.0.0.1:$CDP_PORT http=127.0.0.1:$HTTP_PORT" >"$OUT/run-config.txt"
echo "start_url=$START_URL ZW_RAF_FRAME_DRIVEN=$RAF_DRIVEN ZW_COMPOSITOR_ANIMATIONS=$COMP_ANIM gui=$GUI viewport=800x600 display=${DISPLAY:-none}" >>"$OUT/run-config.txt"

# 1) fixture server
python3 "$ROOT/scripts/background-resource-fixture-server.py" \
    --dir "$FIXDIR" --port "$HTTP_PORT" --log "$OUT/http.jsonl" \
    --media-dir "$ROOT/tests/fixtures/media" \
    >"$OUT/http.log" 2>&1 &
HTTP_PID=$!
sleep 1

# 2) browser（隔离 profile；墙钟上限 = 观测 + 余量）
# GUI=1：真实窗口腿（呈现次数权威面 = compositor frame dump；headless 无
# compositor 进程）。probe 场景名回落到对应 headless 名（bg02-anim/static）。
if [ "$GUI" = "1" ]; then
    PROBE_SCENARIO="${SCENARIO%-gui}"
    XDG_CONFIG_HOME="$OUT/profile" \
    DISPLAY="${DISPLAY:-:20}" \
    ZW_RAF_FRAME_DRIVEN="$RAF_DRIVEN" \
    ZW_COMPOSITOR_ANIMATIONS="$COMP_ANIM" \
    ZW_COMPOSITOR_BIN="$COMPOSITOR" \
    ZW_COMPOSITOR_FRAME_DUMP_DIR="$OUT/frames" \
    ZW_COMPOSITOR_FRAME_DUMP_MAX=2048 \
    ZW_RENDERER_STDERR_ECHO=1 \
    ZW_LOG_LEVEL="${ZW_LOG_LEVEL:-info}" \
    timeout -s TERM $((WALL + 25)) "$BROWSER" \
        "http://127.0.0.1:$HTTP_PORT/$START_URL" \
        >"$OUT/browser.log" 2>&1 &
else
    # probe 场景名回落：autoplay 变体复用 bg02-media 采样臂（-gui 先例）。
    PROBE_SCENARIO="${SCENARIO%-autoplay}"
    XDG_CONFIG_HOME="$OUT/profile" \
    ZW_RAF_FRAME_DRIVEN="$RAF_DRIVEN" \
    ZW_COMPOSITOR_BIN="$COMPOSITOR" \
    ZW_COMPOSITOR_FRAME_DUMP_DIR="$OUT/frames" \
    ZW_COMPOSITOR_FRAME_DUMP_MAX=2048 \
    ZW_RENDERER_STDERR_ECHO=1 \
    ZW_LOG_LEVEL="${ZW_LOG_LEVEL:-info}" \
    timeout -s TERM $((WALL + 25)) "$BROWSER" --headless \
        --remote-debugging-port="$CDP_PORT" --viewport-width=800 --viewport-height=600 \
        "http://127.0.0.1:$HTTP_PORT/$START_URL" \
        >"$OUT/browser.log" 2>&1 &
fi
BROWSER_PID=$!

# 3) 采样器（先于 probe 覆盖全程）
python3 "$ROOT/scripts/background-resource-sampler.py" \
    --root-pid "$BROWSER_PID" --out "$OUT/samples.jsonl" --interval-ms 250 &
SAMPLER_PID=$!

# 4) 等 CDP 就绪（最多 20s），起 probe（GUI 腿无 CDP 端点，跳过驱动只采样）
PROBE_OK=0
if [ "$GUI" = "1" ]; then
    sleep "$WALL"
else
    for _ in $(seq 1 40); do
        if python3 - "$CDP_PORT" <<'EOF' 2>/dev/null
import sys, urllib.request
try:
    urllib.request.urlopen(f"http://127.0.0.1:{sys.argv[1]}/json/version", timeout=1).read()
    sys.exit(0)
except Exception:
    sys.exit(1)
EOF
        then break; fi
        sleep 0.5
    done
    if node "$ROOT/scripts/background-resource-probe.mjs" \
        --scenario "$PROBE_SCENARIO" --cdp-port "$CDP_PORT" --http-port "$HTTP_PORT" \
        --start-url "http://127.0.0.1:$HTTP_PORT/$START_URL" \
        ${BG_NAV_SRC:+--nav-src "$BG_NAV_SRC"} ${BG_NAV_DST:+--nav-dst "$BG_NAV_DST"} \
        ${BG_NAV_CYCLES:+--nav-cycles "$BG_NAV_CYCLES"} \
        --wall-ms "$((WALL * 1000))" --out "$OUT/probe.json"; then
        PROBE_OK=1
    fi
fi

# 5) 收尾：停采样器（SIGTERM → sampler_stop 行）、停浏览器与 server
kill "$SAMPLER_PID" 2>/dev/null || true
wait "$SAMPLER_PID" 2>/dev/null || true
kill "$BROWSER_PID" 2>/dev/null || true
wait "$BROWSER_PID" 2>/dev/null || true
kill "$HTTP_PID" 2>/dev/null || true
wait "$HTTP_PID" 2>/dev/null || true

echo "observation done: $OUT (scenario=$SCENARIO probe_ok=$PROBE_OK)"
