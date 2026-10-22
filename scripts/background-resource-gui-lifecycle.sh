#!/bin/sh
# background-resource-lifecycle goal M4 GUI 生命周期腿编排（DC-3 重复导航 /
# DC-4 close/reopen 恢复的真实宿主端到端）：
#   fixture server + 真实窗口 zero-browser（X display，xdotool 注入 Ctrl+L/
#   Ctrl+T/Ctrl+W/Ctrl+Tab 真实键盘路径）+ 进程树采样器 + anchors.txt 段锚点。
#
# 用法：scripts/background-resource-gui-lifecycle.sh <scenario> <out-dir>
#   scenario ∈ bg03-nav-gui（DC-3：同 tab 地址栏导航 source/target 交替 ×N，
#              renderer RSS 逐周期趋势 + stall 连接随导航关闭——判别面是 RSS，
#              xdotool 形态）
#            | bg04-close-gui（DC-4：Ctrl+T 开页 + Ctrl+W 关 tab ×N + 动画页
#              前台恢复 + 地址栏输入恢复——判别面是 RSS/帧恢复/可见性翻转，
#              xdotool 形态）
#
# 双形态取舍（同 M4 tab 切换判别腿方法学，见
# evidence/2026-10-23-m4-gui-tab-switch/ANALYSIS.md）：本两条腿判别面是
# RSS/连接生命周期/帧恢复，ui-driver 每 tick 强制 needs_redraw 会抬高呈现链
# 背景并扭曲 renderer 分配行为，故只用 xdotool 形态（正常事件循环，
# windowactivate 保焦点）。
#
# 时序：
#   bg03-nav-gui：load 8s → source 前台保持 8s（stall 活跃窗）→ 导航交替 ×12
#     （奇数次→target、偶数次→source，每周期 ~5s）→ settle 10s。
#   bg04-close-gui：load 8s → 动画前台 6s（帧基线）→ Ctrl+T 开轻量页 +
#     Ctrl+W 关闭 ×8（每周期 ~6.5s，tab1 每周期 hidden↔visible 翻转）→
#     动画恢复 6s → Ctrl+L 地址栏导航输入检查 → settle 5s。
# 产物（out-dir）：run-config.txt、browser.log、samples.jsonl、http.jsonl、
#   anchors.txt、frames/（bg04 组 compositor dump manifest）。分段 CPU/RSS
#   分析由 scripts/background-resource-phase-stats.py 完成。
#
# 无人值守请外层包 systemd-run 内存 scope（run-rules §6）。
#
# 前置：make browser-build 已产出 target/release/{zero-browser,zero-compositor}；
#   X display :20 存活；xdotool 可用。
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
command -v xdotool >/dev/null || { echo "xdotool required" >&2; exit 2; }

case "$SCENARIO" in
    bg03-nav-gui)
        START_URL="bg03-nav-source.html"; COMP_ANIM=0; FRAME_DUMP=0
        CYCLES="${BG_NAV_GUI_CYCLES:-12}"
        TIMEOUT=$((90 + CYCLES * 7))
        ;;
    bg04-close-gui)
        START_URL="bg02-onscreen-anim.html"; COMP_ANIM=1; FRAME_DUMP=1
        CYCLES="${BG_CLOSE_GUI_CYCLES:-8}"
        TIMEOUT=$((100 + CYCLES * 8))
        ;;
    *) echo "unknown scenario: $SCENARIO" >&2; exit 2 ;;
esac

mkdir -p "$OUT"
HTTP_PORT=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')

echo "scenario=$SCENARIO start_url=$START_URL ZW_COMPOSITOR_ANIMATIONS=$COMP_ANIM frame_dump=$FRAME_DUMP display=$DISPLAY port=$HTTP_PORT" >"$OUT/run-config.txt"

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

# xdotool 形态：browser 正常事件循环（无强制成帧）。窗口失焦会抑制 needs_redraw
# （main.rs:1653-1665）——windowactivate 保持 ZeroBrowser 前窗；:20 上无其他窗口
# 争抢。窗口 title 随页面加载变为「<页面标题> - ZeroBrowser」（app.rs:3300）——
# 子串匹配 + 轮询等待（冷启动 + GPU 初始化慢于 4s 的轮次）。
timeout -s TERM "$TIMEOUT" "$BROWSER" \
    "http://127.0.0.1:$HTTP_PORT/$START_URL" \
    >"$OUT/browser.log" 2>&1 &
BROWSER_PID=$!

python3 "$ROOT/scripts/background-resource-sampler.py" \
    --root-pid "$BROWSER_PID" --out "$OUT/samples.jsonl" --interval-ms 250 &
SAMPLER_PID=$!

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

NAV_TARGET="http://127.0.0.1:$HTTP_PORT/bg03-nav-target.html"

case "$SCENARIO" in
    bg03-nav-gui)
        step load-done
        sleep 8
        step phase1-hold-start
        sleep 8
        step phase2-nav-cycles-start
        # 奇数次→target（轻量）、偶数次→source（stall+超长 timer 重新持有）；
        # 初始页为 source，故每个 source→target 导航都应触发一次 stall 关闭。
        i=1
        while [ "$i" -le "$CYCLES" ]; do
            step "cycle-$(printf '%02d' "$i")"
            xdotool key ctrl+l
            sleep 1
            if [ $((i % 2)) -eq 1 ]; then
                xdotool type "$NAV_TARGET"
            else
                xdotool type "http://127.0.0.1:$HTTP_PORT/bg03-nav-source.html"
            fi
            xdotool key Return
            sleep 4
            i=$((i + 1))
        done
        step phase3-settle-start
        sleep 10
        step leg-end
        ;;
    bg04-close-gui)
        step load-done
        sleep 6
        step phase1-foreground-start
        sleep 6
        step phase2-close-reopen-start
        # Ctrl+T 新 tab（地址栏自动聚焦）→ 导航轻量页 → tab1 转 hidden →
        # Ctrl+W 关闭 → tab1 回前台。每周期一对可见性翻转。
        i=1
        while [ "$i" -le "$CYCLES" ]; do
            step "cycle-$(printf '%02d' "$i")"
            xdotool key ctrl+t
            sleep 1
            xdotool type "$NAV_TARGET"
            xdotool key Return
            sleep 3
            xdotool key ctrl+w
            sleep 2
            i=$((i + 1))
        done
        step phase3-recovery-start
        sleep 6
        step phase4-input-check-start
        # 前台输入恢复：地址栏聚焦 + 真实键盘导航静态页（加载完成即输入路径成立）。
        xdotool key ctrl+l
        sleep 1
        xdotool type "http://127.0.0.1:$HTTP_PORT/bg02-static.html"
        xdotool key Return
        sleep 5
        step leg-end
        ;;
esac

kill "$BROWSER_PID" 2>/dev/null || true
wait "$BROWSER_PID" || true
kill "$SAMPLER_PID" 2>/dev/null || true
wait "$SAMPLER_PID" 2>/dev/null || true
# 终态计数落盘：非交互 shell 后台作业继承 SIG_IGN（POSIX），SIGINT 不会触发
# fixture-server 的 KeyboardInterrupt 收尾——先 curl /__stats 快照再 SIGTERM。
curl -s "http://127.0.0.1:$HTTP_PORT/__stats" >"$OUT/fixture-stats.json" || true
kill -TERM "$HTTP_PID" 2>/dev/null || true
wait "$HTTP_PID" 2>/dev/null || true

echo "observation done: $OUT (scenario=$SCENARIO)"
