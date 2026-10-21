#!/bin/sh
# DC-4（COMP-04a）帧序列观察：主线程忙窗期 worker canvas 呈现连续性 + terminate
# 冻结证据采集（goal：compositor-animation-continuity M3 / S1d）。
#
# 用法：scripts/compositor-animation-dc4-observe.sh <out-dir> <on|off> [wall-seconds]
#   on  —— ZW_THREADED_WORKERS=1：真线程 worker 执行面（S1a 落地后呈现连续性判定腿）
#   off —— flag 关对照：现行同 realm 语义（canvas registry 共享修复后区域出图：
#     忙窗外动画 + 忙窗期帧静默——主线程同载；修前基线区域恒定不出图见
#     evidence/2026-10-22-m3-s1d-scaffold）
# fixture 时间线（页内钟）：忙窗 @3000–3900ms；threaded 模式 terminate @6500ms、
# 静默核查 @7500ms。frame_dump 512 帧（~8.2s @16ms）须覆盖忙窗与 terminate 点；
# capture t0 = 浏览器启动，页面导航晚 0.5~2s，分析按区域哈希结构化识别、不依赖
# 时间线常量（dc3-analyze 同纪律）。
# 呈现连续性判据：canvas 区域（24,24)-(264,144) 逐帧哈希——off 预期区域动画
#（distinct>1 + mod-232 回绕周期复现）+ 忙窗覆盖的帧静默段；on 预期忙窗期连续
# 变化 + terminate 后终末冻结段。
#
# 前置：make browser-build 已产出 target/release/{zero-browser,zero-compositor}。
# 隔离与上界同 dc1/dc3-observe：一次性 XDG 配置、timeout 墙钟、systemd-run 内存
# scope 由调用方决定（无人值守请外层包 systemd-run scope）。
set -eu
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
OUT="$1"
# 脚本中途会 cd 进 fixture 目录，OUT 必须先定着为绝对路径。
case "$OUT" in
    /*) ;;
    *) OUT="$PWD/$OUT" ;;
esac
MODE="${2:-off}"
WALL="${3:-16}"
FIXDIR="$ROOT/tests/fixtures/compositor-animation"
FIXTURE="comp04a-worker-threaded-canvas.html"
HTTP_PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')

mkdir -p "$OUT/profile" "$OUT/frames"
cd "$FIXDIR"
python3 -m http.server "$HTTP_PORT" --bind 127.0.0.1 >"$OUT/http.log" 2>&1 &
HTTP_PID=$!
sleep 1

cd "$ROOT"
if [ "$MODE" = "on" ]; then
    WORKER_FLAG=1
else
    WORKER_FLAG=0
fi
echo "mode=$MODE flag=ZW_THREADED_WORKERS=$WORKER_FLAG wall=${WALL}s fixture=$FIXTURE" >"$OUT/run-config.txt"

XDG_CONFIG_HOME="$OUT/profile" \
ZW_THREADED_WORKERS="$WORKER_FLAG" \
ZW_COMPOSITOR_BIN="$ROOT/target/release/zero-compositor" \
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
