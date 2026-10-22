#!/bin/sh
# background-resource-lifecycle goal M4 GUI tab 切换腿编排（DC-1 timer 钳制 / DC-2
# 呈现链 hidden 门控的真实宿主端到端）：
#   fixture server + 真实窗口 zero-browser（X display，--ui-driver-script JSONL
#   经真实输入路径驱动 Ctrl+T/Ctrl+Tab 切 tab）+ 进程树采样器 + UI_DRIVER_STEP
#   段锚点日志。
#
# 用法：scripts/background-resource-gui-tab-switch.sh <scenario> <out-dir>
#   scenario ∈ bg01-timer（DC-1：tab1=1ms interval 页，后台钳 1s 的分段 CPU 差分；
#              ui-driver 形态——renderer 侧 timer 派发差分远大于强制重绘背景）
#            | bg02-anim（DC-2：tab1=视口内 CSS 动画页，tab 隐藏呈现链停止差分；
#              ui-driver 形态——注意每 tick 强制 needs_redraw 抬高 browser/compositor
#              背景 ~9/0.1 cores，呈现链差分被淹没，判别腿用 bg02-anim-xdo）
#            | bg02-anim-xdo（DC-2 判别腿：xdotool 注入 Ctrl+T/Ctrl+Tab，browser
#              正常事件循环无强制成帧——前台段动画呈现链活跃、后台段停不停的
#              直接观测；锚点 = browser.log 的 renderer visibility 翻转行）
#
# 时序（三段）：前台 10s → Ctrl+T 开静态页转后台 15s → Ctrl+Tab 切回前台 10s。
# 产物（out-dir）：run-config.txt、browser.log、samples.jsonl、http.jsonl、
#   ui-driver 形态另有 driver-script.jsonl/result.json/final.png、xdotool 形态另有
#   anchors.txt、frames/（bg02 组 compositor dump manifest）。分段 CPU 分析由
#   scripts/background-resource-phase-stats.py 完成。
#
# 无人值守请外层包 systemd-run 内存 scope（run-rules §6）。
#
# 前置：make browser-build 已产出 target/release/{zero-browser,zero-compositor}；
#   X display :20 存活。
set -eu
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
SCENARIO="${1:?scenario required}"
OUT="${2:?out-dir required}"

case "$OUT" in
    /*) ;;
    *) OUT="$PWD/$OUT" ;;
esac
FIXDIR="$ROOT/tests/fixtures/background-resource"
BROWSER="$ROOT/target/release/zero-browser"
COMPOSITOR="$ROOT/target/release/zero-compositor"
[ -x "$BROWSER" ] || { echo "missing $BROWSER (run make browser-build)" >&2; exit 2; }
[ -n "${DISPLAY:-}" ] || { echo "DISPLAY required (GUI leg)" >&2; exit 2; }

case "$SCENARIO" in
    bg01-timer)    START_URL="bg01-interval-timer.html"; COMP_ANIM=0; FRAME_DUMP=0; DRIVER=ui ;;
    bg02-anim)     START_URL="bg02-onscreen-anim.html";  COMP_ANIM=1; FRAME_DUMP=1; DRIVER=ui ;;
    bg02-anim-xdo) START_URL="bg02-onscreen-anim.html";  COMP_ANIM=1; FRAME_DUMP=1; DRIVER=xdo ;;
    *) echo "unknown scenario: $SCENARIO" >&2; exit 2 ;;
esac

mkdir -p "$OUT"
HTTP_PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')

echo "scenario=$SCENARIO driver=$DRIVER start_url=$START_URL ZW_COMPOSITOR_ANIMATIONS=$COMP_ANIM frame_dump=$FRAME_DUMP display=$DISPLAY port=$HTTP_PORT" >"$OUT/run-config.txt"

python3 "$ROOT/scripts/background-resource-fixture-server.py" \
    --dir "$FIXDIR" --port "$HTTP_PORT" --log "$OUT/http.jsonl" \
    >"$OUT/http.log" 2>&1 &
HTTP_PID=$!
sleep 1

# 一次性 profile 目录隔离（同 observe.sh GUI 分支惯例）。
export XDG_CONFIG_HOME="$OUT/profile"
mkdir -p "$XDG_CONFIG_HOME"
export ZW_COMPOSITOR_ANIMATIONS=$COMP_ANIM
export ZW_RENDERER_STDERR_ECHO=1
export ZW_COMPOSITOR_BIN="$COMPOSITOR"
if [ "$FRAME_DUMP" = "1" ]; then
    export ZW_COMPOSITOR_FRAME_DUMP_DIR="$OUT/frames"
    export ZW_COMPOSITOR_FRAME_DUMP_MAX=512
fi

if [ "$DRIVER" = "ui" ]; then
    # ui_driver JSONL：Ctrl+T 后新 tab 地址栏自动聚焦（app.rs new_blank_tab_focused），
    # type 逐字符真实键盘路径；Enter 触发地址栏导航；段边界落 UI_DRIVER_STEP log 行
    # （browser.log 时间戳对齐 sampler t_ms 分桶）。总步时 35s + 加载余量，远低于
    # DRIVER_TIMEOUT 600s 硬上限。
    DRIVER="$OUT/driver-script.jsonl"
    cat >"$DRIVER" <<EOF
{"op":"wait_idle","ms":8000}
{"op":"log","msg":"phase1-foreground-start"}
{"op":"wait_ms","ms":10000}
{"op":"key","key":"t","mods":["ctrl"]}
{"op":"assert_addressbar_focus","focused":true}
{"op":"type","text":"http://127.0.0.1:$HTTP_PORT/bg02-static.html"}
{"op":"key","key":"Enter"}
{"op":"log","msg":"phase2-background-start"}
{"op":"wait_ms","ms":15000}
{"op":"key","key":"Tab","mods":["ctrl"]}
{"op":"log","msg":"phase3-restore-start"}
{"op":"wait_ms","ms":10000}
{"op":"shot","name":"final.png"}
EOF

    # ui-driver 会话：--scale=1 强制、smoke 模式互斥、需 --ui-driver-out。每 tick 强制
    # needs_redraw——失焦不抑制呈现链（ui_driver.rs 白名单）；驱动完成 exit(0)。
    timeout -s TERM 120 "$BROWSER" \
        --scale=1 \
        --ui-driver-script="$DRIVER" \
        --ui-driver-out="$OUT" \
        "http://127.0.0.1:$HTTP_PORT/$START_URL" \
        >"$OUT/browser.log" 2>&1 &
else
    # xdotool 形态：browser 正常事件循环（无强制成帧），X 层键盘注入。窗口失焦会
    # 抑制 needs_redraw（main.rs:1653-1665）——windowactivate 保持 ZeroBrowser 前窗，
    # tab 切换不产生失焦；:20 上无其他窗口争抢。
    timeout -s TERM 120 "$BROWSER" \
        "http://127.0.0.1:$HTTP_PORT/$START_URL" \
        >"$OUT/browser.log" 2>&1 &
fi
BROWSER_PID=$!

python3 "$ROOT/scripts/background-resource-sampler.py" \
    --root-pid "$BROWSER_PID" --out "$OUT/samples.jsonl" --interval-ms 250 &
SAMPLER_PID=$!

if [ "$DRIVER" = "xdo" ]; then
    # 窗口 title 随页面加载变为「<页面标题> - ZeroBrowser」（app.rs:3300）——子串
    # 匹配 + 轮询等待（冷启动 + GPU 初始化慢于 4s 的轮次）。
    WID=""
    for _ in $(seq 1 20); do
        WID=$(xdotool search --name ZeroBrowser 2>/dev/null | head -1)
        [ -n "$WID" ] && break
        sleep 1
    done
    [ -n "$WID" ] || { echo "ZeroBrowser window not found" >&2; kill "$BROWSER_PID" 2>/dev/null || true; exit 3; }
    xdotool windowactivate --sync "$WID"
    ANCH="$OUT/anchors.txt"; : >"$ANCH"
    step() { echo "$(date +%s%3N) $1" >>"$ANCH"; }
    step load-done
    sleep 10
    step phase1-foreground-start
    xdotool key ctrl+t
    sleep 1
    xdotool type "http://127.0.0.1:$HTTP_PORT/bg02-static.html"
    xdotool key Return
    sleep 1
    step phase2-background-start
    sleep 15
    xdotool key ctrl+Tab
    step phase3-restore-start
    sleep 10
    step leg-end
    kill "$BROWSER_PID" 2>/dev/null || true
fi

wait "$BROWSER_PID" || true
kill "$SAMPLER_PID" 2>/dev/null || true
wait "$SAMPLER_PID" 2>/dev/null || true
kill "$HTTP_PID" 2>/dev/null || true
wait "$HTTP_PID" 2>/dev/null || true

echo "observation done: $OUT (scenario=$SCENARIO)"
