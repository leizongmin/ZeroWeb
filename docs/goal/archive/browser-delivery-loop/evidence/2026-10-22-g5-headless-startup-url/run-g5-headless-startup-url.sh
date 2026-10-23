#!/bin/sh
# G5（headless 消费启动 URL）验收：真实 headless 进程三腿。
#
#   leg A  真实加载：本地 HTTP fixture + 位置参数 URL 启动 → /json/list 目标
#          URL 翻转、fixture 服务器收到该路径的 GET（真网络加载证明）、启动
#          导航日志且无失败告警、SIGINT 仍优雅关停（G4 机制不受影响）。
#   leg B  缺省回归：不带位置参数 → /json/list 恒 about:blank（缺省行为不变）。
#   leg C  规范化与失败不阻断：裸 host:port（无 scheme）按地址栏同一
#          normalize_url 升级 http://；不可达 URL 启动导航失败只告警、server
#          仍可发现可附接。
#
# 用法: run-g5-headless-startup-url.sh <repo-root> <work-dir>
# 前置: make browser-build（target/release/{zero-browser,zero-renderer}）。
# 全程隔离：XDG_CONFIG_HOME/ZERO_STORAGE_DIR/ZERO_CACHE_DIR 均在 work-dir 内。
# 任一断言 FAIL 退出码 1。
set -eu

REPO="$1"; WORK="$2"
BROWSER="$REPO/target/release/zero-browser"
PASS=0; FAIL=0

note() { echo "[G5] $*"; }
ok()   { PASS=$((PASS+1)); echo "[G5] PASS: $*"; }
bad()  { FAIL=$((FAIL+1)); echo "[G5] FAIL: $*"; }

free_port() {
    python3 -c "import socket;s=socket.socket();s.bind(('127.0.0.1',0));print(s.getsockname()[1]);s.close()"
}

# /json/list 第一个 page target 的 url 字段。
first_target_url() {
    curl -s "http://127.0.0.1:$1/json/list" 2>/dev/null \
        | python3 -c "import sys,json;print(json.load(sys.stdin)[0]['url'])" 2>/dev/null || echo ""
}

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

# 等 server 起来（/json/version 应答），输出实际端口；失败输出空。
wait_up() {
    for _ in $(seq 1 40); do
        curl -s "http://127.0.0.1:$1/json/version" 2>/dev/null | grep -q "ZeroWeb" && { echo 1; return; }
        kill -0 "$2" 2>/dev/null || break
        sleep 0.25
    done
    echo 0
}

# SIGINT 优雅关停按 G4 形态逐腿内联（不在函数/命令替换里 wait：子 shell
# 等不到非自身子进程，退出码失真）。/proc 状态判别退出：空=已回收、
# Z=僵尸待 wait；10s 窗口。

mkdir -p "$WORK"
cd "$REPO"
for f in "$BROWSER" "$REPO/target/release/zero-renderer"; do
    [ -e "$f" ] || { echo "[G5] missing $f (build first)"; exit 2; }
done

# 预检：本 repo 早轮残留的 zero-browser 会毒化计时与端口面（M4 轮曾遗留
# pre-G4 二进制的 headless 服务），发现即失败并提示清理。
STRAY=0
for pid in $(pgrep -x zero-browser 2>/dev/null || true); do
    exe=$(readlink "/proc/$pid/exe" 2>/dev/null || true)
    case "$exe" in
        "$REPO"/target/release/*|"$REPO"/target/packages/*) STRAY=$((STRAY+1)) ;;
    esac
done
[ "$STRAY" = "0" ] || { echo "[G5] FAIL: $STRAY stray zero-browser process(es) from earlier runs; clean before acceptance"; exit 1; }

# ---- leg A: fixture 真实加载 ----
note "leg A: startup URL is fetched and becomes the initial tab"
SITE="$WORK/site"; rm -rf "$SITE"; mkdir -p "$SITE"
cat > "$SITE/index.html" <<'EOF'
<!doctype html><html><head><title>G5 fixture page</title></head>
<body><p id="marker">zeroweb-g5-startup-loaded</p></body></html>
EOF
HTTPD_PORT=$(free_port)
python3 -m http.server "$HTTPD_PORT" --bind 127.0.0.1 --directory "$SITE" \
    > /dev/null 2> "$WORK/legA-httpd.log" &
HTTPD_PID=$!
STARTUP_URL="http://127.0.0.1:$HTTPD_PORT/index.html"

PORT_A=$(free_port)
XDG_CONFIG_HOME="$WORK/xdg" ZERO_STORAGE_DIR="$WORK/storage" ZERO_CACHE_DIR="$WORK/cache" \
ZW_LOG_LEVEL=debug \
    "$BROWSER" --headless --remote-debugging-port="$PORT_A" "$STARTUP_URL" \
    > "$WORK/legA-stdout.log" 2> "$WORK/legA-stderr.log" &
PID_A=$!
cleanup_a() { kill "$HTTPD_PID" 2>/dev/null || true; for c in $(pgrep -P "$PID_A" 2>/dev/null); do kill -9 "$c" 2>/dev/null || true; done; kill -9 "$PID_A" 2>/dev/null || true; }
trap cleanup_a EXIT

[ "$(wait_up "$PORT_A" "$PID_A")" = "1" ] \
    && ok "legA server up on :$PORT_A" || { bad "legA server did not come up"; exit 1; }

TARGET=""
for _ in $(seq 1 20); do
    TARGET=$(first_target_url "$PORT_A")
    [ "$TARGET" = "$STARTUP_URL" ] && break
    sleep 0.25
done
[ "$TARGET" = "$STARTUP_URL" ] \
    && ok "legA /json/list target url = startup URL" \
    || bad "legA target url '$TARGET' != '$STARTUP_URL'"

sleep 0.5
grep -q "GET /index.html" "$WORK/legA-httpd.log" \
    && ok "legA fixture server received GET /index.html (real network load)" \
    || bad "legA fixture server saw no GET (log: $(tail -2 "$WORK/legA-httpd.log" | tr '\n' ';'))"

grep -q "Headless startup navigation: $STARTUP_URL" "$WORK/legA-stderr.log" \
    && ok "legA startup navigation log present" \
    || bad "legA startup navigation log missing"
grep -q "Headless startup navigation failed" "$WORK/legA-stderr.log" \
    && bad "legA unexpected navigation failure warn" \
    || ok "legA no navigation failure warn"

kill -INT "$PID_A"
STATE="R"
for _ in $(seq 1 100); do
    STATE=$(awk '{print $3}' "/proc/$PID_A/stat" 2>/dev/null || true)
    [ -z "$STATE" ] && break
    [ "$STATE" = "Z" ] && break
    sleep 0.1
done
CODE=0
if [ -n "$STATE" ] && [ "$STATE" != "Z" ]; then
    bad "legA still alive 10s after SIGINT"
    kill -9 "$PID_A" 2>/dev/null || true
    wait "$PID_A" 2>/dev/null || true
    CODE=999
else
    wait "$PID_A" 2>/dev/null || CODE=$?
fi
[ "$CODE" = "0" ] && ok "legA SIGINT exit code 0 (G4 graceful path intact)" \
    || bad "legA exit code $CODE (want 0)"
[ "$(orphans_of_run)" = "0" ] && ok "legA no orphan renderer" || bad "legA orphan renderer"
trap - EXIT
kill "$HTTPD_PID" 2>/dev/null || true

# ---- leg B: 缺省 about:blank 回归 ----
note "leg B: no positional URL keeps about:blank default"
PORT_B=$(free_port)
XDG_CONFIG_HOME="$WORK/xdg" ZERO_STORAGE_DIR="$WORK/storage" ZERO_CACHE_DIR="$WORK/cache" \
    "$BROWSER" --headless --remote-debugging-port="$PORT_B" \
    > "$WORK/legB-stdout.log" 2> "$WORK/legB-stderr.log" &
PID_B=$!
cleanup_b() { for c in $(pgrep -P "$PID_B" 2>/dev/null); do kill -9 "$c" 2>/dev/null || true; done; kill -9 "$PID_B" 2>/dev/null || true; }
trap cleanup_b EXIT

[ "$(wait_up "$PORT_B" "$PID_B")" = "1" ] \
    && ok "legB server up" || { bad "legB server did not come up"; exit 1; }
B_URL=$(first_target_url "$PORT_B")
[ "$B_URL" = "about:blank" ] \
    && ok "legB default initial tab is still about:blank" \
    || bad "legB default tab url '$B_URL' != about:blank"
kill -INT "$PID_B"
STATE="R"
for _ in $(seq 1 100); do
    STATE=$(awk '{print $3}' "/proc/$PID_B/stat" 2>/dev/null || true)
    [ -z "$STATE" ] && break
    [ "$STATE" = "Z" ] && break
    sleep 0.1
done
CODE=0
if [ -n "$STATE" ] && [ "$STATE" != "Z" ]; then
    bad "legB still alive 10s after SIGINT"
    kill -9 "$PID_B" 2>/dev/null || true
    wait "$PID_B" 2>/dev/null || true
    CODE=999
else
    wait "$PID_B" 2>/dev/null || CODE=$?
fi
[ "$CODE" = "0" ] && ok "legB clean exit" || bad "legB exit code $CODE"
trap - EXIT

# ---- leg C: 规范化 + 失败不阻断 ----
note "leg C: bare host normalization + unreachable URL does not block server"
PORT_C=$(free_port)
DEAD_PORT=$(free_port)
XDG_CONFIG_HOME="$WORK/xdg" ZERO_STORAGE_DIR="$WORK/storage" ZERO_CACHE_DIR="$WORK/cache" \
ZW_LOG_LEVEL=debug \
    "$BROWSER" --headless --remote-debugging-port="$PORT_C" "127.0.0.1:$DEAD_PORT/gone.html" \
    > "$WORK/legC-stdout.log" 2> "$WORK/legC-stderr.log" &
PID_C=$!
cleanup_c() { for c in $(pgrep -P "$PID_C" 2>/dev/null); do kill -9 "$c" 2>/dev/null || true; done; kill -9 "$PID_C" 2>/dev/null || true; }
trap cleanup_c EXIT

[ "$(wait_up "$PORT_C" "$PID_C")" = "1" ] \
    && ok "legC server up despite unreachable startup URL" \
    || { bad "legC unreachable URL blocked server startup"; exit 1; }
C_URL=$(first_target_url "$PORT_C")
case "$C_URL" in
    "http://127.0.0.1:$DEAD_PORT/gone.html") ok "legC bare host:port normalized to http:// (address-bar parity)" ;;
    *) bad "legC normalized url '$C_URL' unexpected" ;;
esac
grep -q "Headless startup navigation failed" "$WORK/legC-stderr.log" \
    && ok "legC navigation failure warn present (non-fatal)" \
    || bad "legC expected failure warn missing"
kill -INT "$PID_C"
STATE="R"
for _ in $(seq 1 100); do
    STATE=$(awk '{print $3}' "/proc/$PID_C/stat" 2>/dev/null || true)
    [ -z "$STATE" ] && break
    [ "$STATE" = "Z" ] && break
    sleep 0.1
done
CODE=0
if [ -n "$STATE" ] && [ "$STATE" != "Z" ]; then
    bad "legC still alive 10s after SIGINT"
    kill -9 "$PID_C" 2>/dev/null || true
    wait "$PID_C" 2>/dev/null || true
    CODE=999
else
    wait "$PID_C" 2>/dev/null || CODE=$?
fi
[ "$CODE" = "0" ] && ok "legC clean exit after failed startup nav" || bad "legC exit code $CODE"
trap - EXIT

# ---- 汇总 ----
echo "[G5] summary: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" = "0" ]
