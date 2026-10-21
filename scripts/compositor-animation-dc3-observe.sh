#!/bin/sh
# DC-3（COMP-03）真实窗口观察：命中当前几何 + 暂停冻结 + 取消恰一次 + 恢复续走
# 证据采集（goal：compositor-animation-continuity M2-S3d）。
#
# 用法：scripts/compositor-animation-dc3-observe.sh <out-dir> <on|off> [wall-seconds]
#   on  —— ZW_COMPOSITOR_ANIMATIONS=1：命中修正生效（hit-a/hit-b 应绿）
#   off —— flag 关对照：合成不采样、无导出（盒恒基线 → hit-a/hit-b 落空染红、
#          ev-cancel 无事件、帧序列收敛后冻结）
# 点击注入计划与 fixture 时间线配套：注入锚 = tab loaded 事件（title 级，较
# 页面 load 提前 0~2.5s，记 Δ），点击时刻 3000/4500/5500ms 与落点按 Δ 全区间
# 校准，且全部落在 512 帧采集窗（~8.2s）内。坐标 = 窗口物理像素、DPR=1 与
# fixture CSS 像素同尺度；窗口 → 文档坐标换算扣 chrome 高：TAB_STRIP 40 +
# ADDRESS_BAR 50 ≈ 91px（layout.rs TOOLBAR_HEIGHT=90，实测 doc = win − 91）。
# 落点按文档坐标反推窗口 y = doc + 91：
#   点击 1 @3000 文档 (436,220)——400px 宽 hit-box 基线右缘 424 外 12px、
#           「位移 ≥12px（t≥0.4s）即覆盖」窗口内（win 436,311）；
#   点击 2 @4500 文档 (170,385)——paused(1.5s) 冻结位移 45px 的盒 x 69..189 内、
#           基线右缘 144 外 26px（win 170,476）；
#   点击 3 @5500 文档 (548,368)——cancel(3.5s) 前后基线中心（旋转绕中心，
#           任意相位均命中；win 548,459）。
# 注意：renderer 对空白区点击（browser selector=None 且 renderer hit_test None）
# 静默不派发（预存在缺口，master 缺口表）——落点必须落在目标元素上，flag 关
# 时 hit-a 由「第三格占位失配」染红（详见 fixture 注释）。
#
# 前置：make browser-build 已产出 target/release/{zero-browser,zero-compositor}。
# 隔离与上界同 dc1/dc2-observe：一次性 XDG 配置、timeout 墙钟、systemd-run 内存
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
WALL="${3:-16}"
FIXDIR="$ROOT/tests/fixtures/compositor-animation"
FIXTURE="comp03-hit-test-pause-cancel.html"
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
ZW_INJECT_CLICKS="436,311@3000;170,476@4500;548,459@5500" \
ZW_RENDERER_STDERR_ECHO=1 \
ZW_LOG_LEVEL="${ZW_LOG_LEVEL:-info}" \
timeout -s TERM "$WALL" "$ROOT/target/release/zero-browser" \
    "http://127.0.0.1:$HTTP_PORT/$FIXTURE" \
    >"$OUT/browser.log" 2>&1 || true

kill "$HTTP_PID" 2>/dev/null || true
pkill -P "$HTTP_PID" 2>/dev/null || true
echo "observation done: $OUT (mode=$MODE)"
wc -l "$OUT/frames/manifest.jsonl" 2>/dev/null || echo "no manifest (0 frames dumped)"
echo "click inject log lines:"
grep -c "click_injector: 注入点击" "$OUT/browser.log" 2>/dev/null || echo 0
echo "animation event log lines:"
grep -cE "animation(cancel|end) dispatched" "$OUT/browser.log" 2>/dev/null || echo 0
