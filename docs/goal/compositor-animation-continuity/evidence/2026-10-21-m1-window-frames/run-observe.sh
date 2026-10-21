#!/bin/sh
# M1 真实窗口观察（第二轮）：隔离 XDG 配置 + compositor 帧落盘。
# 墙钟上界 25s（timeout -s TERM）；输出到 $1 指定目录。
set -eu
# 脚本随证据归档于仓库内 5 层目录；ROOT 由脚本位置派生（同 comp01-probe.sh 风格）。
ROOT=$(CDPATH= cd -- "$(dirname "$0")/../../../../.." && pwd)
OUT="$1"
FIXDIR="$ROOT/tests/fixtures/compositor-animation"
HTTP_PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')

mkdir -p "$OUT/profile" "$OUT/frames"
cd "$FIXDIR"
python3 -m http.server "$HTTP_PORT" --bind 127.0.0.1 >"$OUT/http.log" 2>&1 &
HTTP_PID=$!
sleep 1

cd "$ROOT"
XDG_CONFIG_HOME="$OUT/profile" \
ZW_COMPOSITOR_BIN="$ROOT/target/release/zero-compositor" \
ZW_COMPOSITOR_FRAME_DUMP_DIR="$OUT/frames" \
ZW_COMPOSITOR_FRAME_DUMP_MAX=64 \
RUST_LOG=info \
timeout -s TERM 25 "$ROOT/target/release/zero-browser" \
    "http://127.0.0.1:$HTTP_PORT/comp01-transform-opacity-under-js-load.html" \
    >"$OUT/browser.log" 2>&1 || true

kill "$HTTP_PID" 2>/dev/null || true
pkill -P "$HTTP_PID" 2>/dev/null || true
echo "observation done: $OUT"
ls -la "$OUT/frames" | head -8
