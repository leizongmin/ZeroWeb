#!/bin/sh
# G4（headless 优雅关停）验收：真实产品 headless 模式的两条关停路径。
#
#   leg A  SIGINT：S4 起 shutdown_signal 全局置位 SIGINT flag，但 headless
#          mux 循环此前不消费——Ctrl+C 从"默认立即死"变成"永不退出"（renderer
#          与端口悬挂）。修复后：flag 命中 → 退出复用循环 → session Drop 经
#          RendererHandle::drop 回收 renderer → 退出码 0。
#   leg B  CDP Browser.close：此前为空壳 stub（只回 closing 不关停），自动化
#          标准停止路径失效。修复后：置位同一 flag，复用循环下一 tick 消费。
#
# 两腿共同断言：退出码 0、关停日志、无 renderer 孤儿（/proc exe 归属本次
# 构建目录）、不写 session.json（headless target 是自动化临时面，写
# clean_exit=true 会吞掉 GUI 侧待恢复的崩溃提示——会话语义仍归 GUI）。
#
# 用法: run-g4-headless-shutdown.sh <repo-root> <work-dir>
# 前置: make browser-build（target/release/{zero-browser,zero-renderer}）。
# 全程隔离：XDG_CONFIG_HOME/ZERO_STORAGE_DIR/ZERO_CACHE_DIR 均在 work-dir 内。
# 任一断言 FAIL 退出码 1。
set -eu

REPO="$1"; WORK="$2"
BROWSER="$REPO/target/release/zero-browser"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PASS=0; FAIL=0

note() { echo "[G4] $*"; }
ok()   { PASS=$((PASS+1)); echo "[G4] PASS: $*"; }
bad()  { FAIL=$((FAIL+1)); echo "[G4] FAIL: $*"; }

free_port() {
    python3 -c "import socket;s=socket.socket();s.bind(('127.0.0.1',0));print(s.getsockname()[1]);s.close()"
}

# 本进程树无残留：exe 归属本次构建目录的 renderer 存活数。
orphans_of_run() {
    count=0
    for pid in $(pgrep -x zero-renderer 2>/dev/null || true); do
        exe=$(readlink "/proc/$pid/exe" 2>/dev/null || true)
        case "$exe" in
            "$REPO"/target/release/*) count=$((count+1)) ;;
        esac
    done
    echo "$count"
}

# 指定 browser pid 的非僵尸 renderer 子进程数（启动期死亡残留为 zombie）。
live_children() {
    for pid in $(pgrep -P "$1" -x zero-renderer 2>/dev/null || true); do
        state=$(awk '{print $3}' "/proc/$pid/stat" 2>/dev/null || true)
        [ -n "$state" ] && [ "$state" != "Z" ] && { echo 1; return; }
    done
    echo 0
}

mkdir -p "$WORK"
cd "$REPO"
for f in "$BROWSER" "$REPO/target/release/zero-renderer" "$SCRIPT_DIR/ws-browser-close.js"; do
    [ -e "$f" ] || { echo "[G4] missing $f (build first)"; exit 2; }
done

# ---- leg A: SIGINT 优雅关停 ----
note "leg A: SIGINT graceful shutdown"
PROFILE_A="$WORK/profile-a"
rm -rf "$PROFILE_A"; mkdir -p "$PROFILE_A"
PORT_A=$(free_port)
OUT_A="$WORK/legA-stdout.log"; ERR_A="$WORK/legA-stderr.log"
XDG_CONFIG_HOME="$PROFILE_A" \
ZERO_STORAGE_DIR="$WORK/storage-a" \
ZERO_CACHE_DIR="$WORK/cache-a" \
ZW_LOG_LEVEL=debug \
    "$BROWSER" --headless --remote-debugging-port="$PORT_A" > "$OUT_A" 2> "$ERR_A" &
PID_A=$!

UP=0
for _ in $(seq 1 40); do
    grep -q "DevTools URL" "$OUT_A" 2>/dev/null && { UP=1; break; }
    kill -0 "$PID_A" 2>/dev/null || break
    sleep 0.25
done
[ "$UP" = "1" ] || { bad "legA headless server did not announce DevTools URL"; kill -9 "$PID_A" 2>/dev/null || true; exit 1; }
curl -s "http://127.0.0.1:$PORT_A/json/version" | grep -q "ZeroWeb" \
    && ok "legA server up on :$PORT_A (/json/version answers)" \
    || bad "legA /json/version did not answer"

REN=0
for _ in $(seq 1 20); do
    REN=$(live_children "$PID_A")
    [ "$REN" = "1" ] && break
    sleep 0.25
done
[ "$REN" = "1" ] && ok "legA renderer child spawned (1 live)" \
    || bad "legA expected 1 renderer child, got $REN"

kill -INT "$PID_A"
CODE=0
for _ in $(seq 1 40); do
    kill -0 "$PID_A" 2>/dev/null || break
    sleep 0.25
done
if kill -0 "$PID_A" 2>/dev/null; then
    bad "legA still alive 10s after SIGINT (pre-fix behavior: flag unconsumed)"
    kill -9 "$PID_A" 2>/dev/null || true
    wait "$PID_A" 2>/dev/null || true
    CODE=999
else
    wait "$PID_A" 2>/dev/null || CODE=$?
fi
[ "$CODE" = "0" ] && ok "legA exit code 0 within 10s" \
    || bad "legA exit code $CODE (want 0)"
grep -q "Headless server shutting down" "$ERR_A" \
    && ok "legA graceful shutdown log present" \
    || bad "legA shutdown log missing"
[ "$(orphans_of_run)" = "0" ] \
    && ok "legA no orphan renderer (reaped via session Drop)" \
    || bad "legA orphan renderer survived"
[ ! -e "$PROFILE_A/zeroweb/session.json" ] \
    && ok "legA no session.json written (headless leaves GUI session semantics alone)" \
    || bad "legA session.json unexpectedly written"

# ---- leg B: CDP Browser.close ----
note "leg B: CDP Browser.close stops the server"
PROFILE_B="$WORK/profile-b"
rm -rf "$PROFILE_B"; mkdir -p "$PROFILE_B"
PORT_B=$(free_port)
OUT_B="$WORK/legB-stdout.log"; ERR_B="$WORK/legB-stderr.log"
XDG_CONFIG_HOME="$PROFILE_B" \
ZERO_STORAGE_DIR="$WORK/storage-b" \
ZERO_CACHE_DIR="$WORK/cache-b" \
ZW_LOG_LEVEL=debug \
    "$BROWSER" --headless --remote-debugging-port="$PORT_B" > "$OUT_B" 2> "$ERR_B" &
PID_B=$!

UP=0
for _ in $(seq 1 40); do
    grep -q "DevTools URL" "$OUT_B" 2>/dev/null && { UP=1; break; }
    kill -0 "$PID_B" 2>/dev/null || break
    sleep 0.25
done
[ "$UP" = "1" ] || { bad "legB headless server did not announce DevTools URL"; kill -9 "$PID_B" 2>/dev/null || true; exit 1; }
curl -s "http://127.0.0.1:$PORT_B/json/version" | grep -q "ZeroWeb" \
    && ok "legB server up on :$PORT_B" || bad "legB /json/version did not answer"

WS_OUT="$WORK/legB-ws.out"
if node "$SCRIPT_DIR/ws-browser-close.js" "$PORT_B" > "$WS_OUT" 2>&1; then
    grep -q "closing" "$WS_OUT" \
        && ok "legB WS response carries closing result: $(cat "$WS_OUT")" \
        || ok "legB WS finished without response frame: $(cat "$WS_OUT")"
else
    bad "legB WS driver failed: $(cat "$WS_OUT")"
fi

CODE=0
for _ in $(seq 1 40); do
    kill -0 "$PID_B" 2>/dev/null || break
    sleep 0.25
done
if kill -0 "$PID_B" 2>/dev/null; then
    bad "legB still alive 10s after Browser.close (stub behavior)"
    kill -9 "$PID_B" 2>/dev/null || true
    wait "$PID_B" 2>/dev/null || true
    CODE=999
else
    wait "$PID_B" 2>/dev/null || CODE=$?
fi
[ "$CODE" = "0" ] && ok "legB exit code 0 within 10s" \
    || bad "legB exit code $CODE (want 0)"
grep -q "Headless server shutting down" "$ERR_B" \
    && ok "legB graceful shutdown log present" \
    || bad "legB shutdown log missing"
[ "$(orphans_of_run)" = "0" ] \
    && ok "legB no orphan renderer" \
    || bad "legB orphan renderer survived"

# ---- 汇总 ----
echo "[G4] summary: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" = "0" ]
