#!/bin/sh
# DC-2（COMP-02）真实窗口观察：动画查询一致性 + 结束事件恰一次 + 导航换代
# 证据采集（goal：compositor-animation-continuity M2-S2c）。
#
# 用法：scripts/compositor-animation-dc2-observe.sh <out-dir> <on|off> [wall-seconds]
#   on  —— ZW_COMPOSITOR_ANIMATIONS=1（采样链路开启）
#   off —— flag 关闭对照（gCS 条带恒级联值、零事件、帧序列收敛后冻结）
#
# 前置：make browser-build 已产出 target/release/{zero-browser,zero-compositor}。
# 隔离与上界同 dc1-observe：一次性 XDG 配置、timeout 墙钟、systemd-run 内存
# scope 由调用方决定（无人值守请外层包 systemd-run scope）。
set -eu
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
OUT="$1"
# 脚本中途会 cd 进 fixture 目录，OUT 必须先定着为绝对路径。
case "$OUT" in
    /*) ;;
    *) OUT="$PWD/$OUT" ;;
esac
MODE="${2:-on}"
WALL="${3:-12}"
FIXDIR="$ROOT/tests/fixtures/compositor-animation"
FIXTURE="comp02-script-state-and-end-event.html"
HTTP_PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')

mkdir -p "$OUT/profile" "$OUT/frames"
cd "$FIXDIR"
python3 -m http.server "$HTTP_PORT" --bind 127.0.0.1 >"$OUT/http.log" 2>&1 &
HTTP_PID=$!
sleep 1

cd "$ROOT"
if [ "$MODE" = "on" ]; then
    ANIM_FLAG=1
else
    ANIM_FLAG=0
fi
echo "mode=$MODE flag=ZW_COMPOSITOR_ANIMATIONS=$ANIM_FLAG wall=${WALL}s fixture=$FIXTURE" >"$OUT/run-config.txt"

XDG_CONFIG_HOME="$OUT/profile" \
ZW_COMPOSITOR_BIN="$ROOT/target/release/zero-compositor" \
ZW_COMPOSITOR_ANIMATIONS="$ANIM_FLAG" \
ZW_COMPOSITOR_FRAME_DUMP_DIR="$OUT/frames" \
ZW_COMPOSITOR_FRAME_DUMP_MAX=512 \
ZW_RENDERER_STDERR_ECHO=1 \
ZW_LOG_LEVEL="${ZW_LOG_LEVEL:-info}" \
timeout -s TERM "$WALL" "$ROOT/target/release/zero-browser" \
    "http://127.0.0.1:$HTTP_PORT/$FIXTURE" \
    >"$OUT/browser.log" 2>&1 || true

kill "$HTTP_PID" 2>/dev/null || true
pkill -P "$HTTP_PID" 2>/dev/null || true
echo "observation done: $OUT (mode=$MODE)"
wc -l "$OUT/frames/manifest.jsonl" 2>/dev/null || echo "no manifest (0 frames dumped)"
echo "animationend log lines:"
grep -c "animationend dispatched" "$OUT/browser.log" 2>/dev/null || echo 0
