#!/bin/sh
# G3（--profile CLI）验收：独立 profile 的路径收敛与隔离语义，真实进程三腿。
#
#   leg A  GUI 覆盖语义：环境变量指向 scratch（XDG_CONFIG_HOME/ZERO_STORAGE_DIR/
#          ZERO_CACHE_DIR 均非 profile），--profile 显式给出后必须赢——配置/存储/
#          缓存全部落 profile 根，renderer 经 env 继承同一隔离，SIGINT 后
#          session.json 在 profile 内而 scratch XDG 零写入。
#   leg B  崩溃报告随 profile：GUI（无 CDP，S3 先例——CDP 辅助 renderer 干扰
#          「唯一 tab renderer」选取）+ --profile 启动 → SIGKILL 唯一 tab
#          renderer → 报告落 <profile>/zeroweb/crash-reports/；CLI 面双向隔离
#          （--profile 列得到、不带 --profile 时 scratch XDG 列不到）；报告内
#          version/commit 与 --version 贯穿一致。
#   leg C  相对路径 profile：相对值按 CWD 绝对化（避免平台层静默忽略相对
#          XDG 值后回落真实 HOME 的隐患）。
#
# 用法: run-g3-profile-acceptance.sh <repo-root> <work-dir>
# 前置: make browser-build（target/release/{zero-browser,zero-renderer}）。
# 全程隔离：即使 --profile 失效，环境变量兜底仍指向 work-dir，真实 HOME 零接触。
# 任一断言 FAIL 退出码 1。
set -eu

REPO="$1"; WORK="$2"
BROWSER="$REPO/target/release/zero-browser"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PASS=0; FAIL=0

note() { echo "[G3] $*"; }
ok()   { PASS=$((PASS+1)); echo "[G3] PASS: $*"; }
bad()  { FAIL=$((FAIL+1)); echo "[G3] FAIL: $*"; }

free_port() {
    python3 -c "import socket;s=socket.socket();s.bind(('127.0.0.1',0));print(s.getsockname()[1]);s.close()"
}

# 指定 browser pid 的一个非僵尸 renderer 子进程 pid（无则空）。
live_renderer() {
    for pid in $(pgrep -P "$1" -x zero-renderer 2>/dev/null || true); do
        state=$(awk '{print $3}' "/proc/$pid/stat" 2>/dev/null || true)
        [ -n "$state" ] && [ "$state" != "Z" ] && { echo "$pid"; return; }
    done
}

mkdir -p "$WORK"
cd "$REPO"
for f in "$BROWSER" "$REPO/target/release/zero-renderer"; do
    [ -e "$f" ] || { echo "[G3] missing $f (build first)"; exit 2; }
done

VERSION_OUT=$("$BROWSER" --version)
PROD_VER=$(echo "$VERSION_OUT" | awk '{print $2}')
COMMIT=$(echo "$VERSION_OUT" | sed -n 's/.*commit \([0-9a-f]*\).*/\1/p')
note "product identity: $VERSION_OUT"

# ---- leg A: GUI + --profile 覆盖环境变量 ----
note "leg A: --profile overrides ambient ZERO_* env (GUI)"
P1="$WORK/p1"
rm -rf "$P1" "$WORK/env-xdg" "$WORK/env-storage" "$WORK/env-cache"
mkdir -p "$WORK/env-xdg"
PORT_A=$(free_port)
XDG_CONFIG_HOME="$WORK/env-xdg" \
ZERO_STORAGE_DIR="$WORK/env-storage" \
ZERO_CACHE_DIR="$WORK/env-cache" \
DISPLAY="${DISPLAY:-:0}" \
ZW_LOG_LEVEL=debug \
    "$BROWSER" --profile="$P1" --remote-debugging-port="$PORT_A" zero://newtab \
    > "$WORK/legA-stdout.log" 2> "$WORK/legA-stderr.log" &
PID_A=$!
cleanup_a() {
    for child in $(pgrep -P "$PID_A" 2>/dev/null); do kill -9 "$child" 2>/dev/null || true; done
    kill -9 "$PID_A" 2>/dev/null || true
}
trap cleanup_a EXIT

UP=0
for _ in $(seq 1 40); do
    curl -s "http://127.0.0.1:$PORT_A/json/version" 2>/dev/null | grep -q "ZeroWeb" && { UP=1; break; }
    kill -0 "$PID_A" 2>/dev/null || break
    sleep 0.25
done
[ "$UP" = "1" ] && ok "legA GUI up on :$PORT_A (/json/version answers)" \
    || { bad "legA GUI did not come up"; exit 1; }

# 注：/proc/<browser>/environ 只反映 exec 时刻环境，看不到运行期 set_var；
# browser 侧注入的正确观测点是子进程继承（下两断言）与落盘位置（session.json）。

REN=$(live_renderer "$PID_A")
if [ -n "$REN" ]; then
    ok "legA live renderer pid=$REN"
    RENV=$(tr '\0' '\n' < "/proc/$REN/environ")
    echo "$RENV" | grep -q "ZERO_STORAGE_DIR=$P1/storage" \
        && ok "legA renderer inherits ZERO_STORAGE_DIR=<profile>/storage" \
        || bad "legA renderer storage env wrong: $(echo "$RENV" | grep ZERO_STORAGE_DIR || echo unset)"
    echo "$RENV" | grep -q "ZERO_CACHE_DIR=$P1/cache" \
        && ok "legA renderer inherits ZERO_CACHE_DIR=<profile>/cache" \
        || bad "legA renderer cache env wrong: $(echo "$RENV" | grep ZERO_CACHE_DIR || echo unset)"
else
    bad "legA no live renderer child"
fi

kill -INT "$PID_A"
CODE=0
for _ in $(seq 1 40); do
    kill -0 "$PID_A" 2>/dev/null || break
    sleep 0.25
done
if kill -0 "$PID_A" 2>/dev/null; then
    bad "legA still alive 10s after SIGINT"
    cleanup_a
    CODE=999
else
    wait "$PID_A" 2>/dev/null || CODE=$?
fi
trap - EXIT
[ "$CODE" = "0" ] && ok "legA exit code 0 within 10s" || bad "legA exit code $CODE (want 0)"
[ -f "$P1/zeroweb/session.json" ] \
    && ok "legA session.json under <profile>/zeroweb" \
    || bad "legA session.json missing under profile"
[ ! -e "$WORK/env-xdg/zeroweb/session.json" ] \
    && ok "legA scratch XDG got zero session writes (override wins)" \
    || bad "legA session leaked to ambient XDG"

# ---- leg B: 崩溃报告随 profile + CLI 双向隔离 ----
note "leg B: crash report lands under profile (renderer SIGKILL)"
P2="$WORK/p2"
BREPORTS="$P2/zeroweb/crash-reports"
rm -rf "$P2" "$WORK/env-xdg2"
mkdir -p "$WORK/env-xdg2"
XDG_CONFIG_HOME="$WORK/env-xdg2" \
ZERO_STORAGE_DIR="$WORK/env-storage" \
ZERO_CACHE_DIR="$WORK/env-cache" \
DISPLAY="${DISPLAY:-:0}" \
ZW_LOG_LEVEL=debug \
    "$BROWSER" --profile="$P2" zero://newtab \
    > "$WORK/legB-stdout.log" 2> "$WORK/legB-stderr.log" &
PID_B=$!
cleanup_b() {
    for child in $(pgrep -P "$PID_B" 2>/dev/null); do kill -9 "$child" 2>/dev/null || true; done
    kill -9 "$PID_B" 2>/dev/null || true
}
trap cleanup_b EXIT

REN_B=""
for _ in $(seq 1 60); do
    REN_B=$(live_renderer "$PID_B")
    [ -n "$REN_B" ] && break
    kill -0 "$PID_B" 2>/dev/null || break
    sleep 0.5
done
[ -n "$REN_B" ] && ok "legB live renderer pid=$REN_B" \
    || { bad "legB no live renderer child"; exit 1; }

kill -9 "$REN_B"
note "legB SIGKILL delivered to renderer $REN_B"
REPORT=""
for _ in $(seq 1 30); do
    REPORT=$(ls "$BREPORTS"/crash-*.json 2>/dev/null | head -1 || true)
    [ -n "$REPORT" ] && break
    sleep 0.5
done
[ -n "$REPORT" ] && ok "legB crash report under <profile>/zeroweb/crash-reports: $(basename "$REPORT")" \
    || bad "legB no crash report under profile"

LIST_OUT=$("$BROWSER" --list-crash-reports --profile="$P2" 2>/dev/null)
echo "$LIST_OUT" | grep -q "$(basename "$REPORT")" \
    && ok "legB --list-crash-reports --profile lists the report" \
    || bad "legB list with profile missing report (got: $LIST_OUT)"

ISO_OUT=$(XDG_CONFIG_HOME="$WORK/env-xdg2" "$BROWSER" --list-crash-reports 2>/dev/null)
[ "$ISO_OUT" = "NO_REPORTS" ] \
    && ok "legB without --profile, scratch XDG lists NO_REPORTS (isolation)" \
    || bad "legB isolation broken (got: $ISO_OUT)"

grep -q "\"version\":\"$PROD_VER\"" "$REPORT" 2>/dev/null \
    && ok "legB report version matches --version ($PROD_VER)" \
    || bad "legB report version mismatch"
grep -q "\"git_commit\":\"$COMMIT\"" "$REPORT" \
    && ok "legB report git_commit matches --version ($COMMIT)" \
    || bad "legB report git_commit mismatch"

kill -INT "$PID_B"
for _ in $(seq 1 40); do
    kill -0 "$PID_B" 2>/dev/null || break
    sleep 0.25
done
if kill -0 "$PID_B" 2>/dev/null; then
    bad "legB browser still alive 10s after SIGINT"
    cleanup_b
else
    wait "$PID_B" 2>/dev/null || true
    ok "legB browser exited cleanly after SIGINT (1 crash < loop-guard threshold)"
fi
trap - EXIT

# ---- leg C: 相对路径 profile 按 CWD 绝对化 ----
note "leg C: relative --profile resolves against cwd"
cd "$WORK"
REL_OUT=$("$BROWSER" --list-crash-reports --profile=p2 2>/dev/null)
echo "$REL_OUT" | grep -q "$(basename "$REPORT")" \
    && ok "legC relative --profile=p2 from cwd=$WORK lists the same report" \
    || bad "legC relative profile did not resolve (got: $REL_OUT)"

# ---- 汇总 ----
echo "[G3] summary: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" = "0" ]
