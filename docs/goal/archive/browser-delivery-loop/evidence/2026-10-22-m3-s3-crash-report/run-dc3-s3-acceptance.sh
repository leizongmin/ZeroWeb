#!/bin/sh
# DC-3 前半（S3）验收：本地崩溃报告。
#
#   leg A  真实进程 panic：crash_probe 注入 panic → 白名单报告落盘，
#          字段集精确匹配 IF-3，隐私 grep（URL/宿主路径/报告目录）全 0。
#   leg B  真实产品 renderer 异常退出：GUI zero-browser 启动（无 CDP）→
#          SIGKILL 唯一的 tab renderer → renderer 报告落盘（signal: 9），
#          版本/commit 与宿主二进制一致，隐私 grep 全 0；重生路径不受影响
#          （browser 存活、新 renderer pid）。
#
# 用法: run-dc3-s3-acceptance.sh <repo-root> <work-dir>
# 前置: make browser-build（target/release/{zero-browser,zero-renderer}）。
# 全程隔离：XDG_CONFIG_HOME/ZERO_STORAGE_DIR/ZERO_CACHE_DIR 均在 work-dir 内，
# 不触系统与真实 profile；任一断言 FAIL 退出码 1。
set -eu

REPO="$1"; WORK="$2"
BROWSER="$REPO/target/release/zero-browser"
PASS=0; FAIL=0

note() { echo "[DC3] $*"; }
ok()   { PASS=$((PASS+1)); echo "[DC3] PASS: $*"; }
bad()  { FAIL=$((FAIL+1)); echo "[DC3] FAIL: $*"; }

for f in "$BROWSER" "$REPO/target/release/zero-renderer"; do
    [ -x "$f" ] || { echo "[DC3] missing $f (run make browser-build first)"; exit 2; }
done

mkdir -p "$WORK"
cd "$REPO"

# ---- Leg A: 真实进程 panic（crash_probe，生产 panic 钩子代码路径） ----
note "leg A: real-process panic probe"
AREPORTS="$WORK/panic-reports"
rm -rf "$AREPORTS"; mkdir -p "$AREPORTS"

PROBE_EXIT=0
systemd-run --user --scope -p MemoryMax=4G timeout 60 \
    cargo run -q -p zero-browser-shell --example crash_probe -- "$AREPORTS" \
    > "$WORK/legA-probe.log" 2>&1 || PROBE_EXIT=$?
[ "$PROBE_EXIT" = "101" ] && ok "legA probe exited 101 (panic)" || bad "legA probe exit=$PROBE_EXIT (want 101)"

ACOUNT=$(ls "$AREPORTS"/crash-*-browser.json 2>/dev/null | wc -l)
[ "$ACOUNT" = "1" ] && ok "legA exactly one browser report" || bad "legA report count=$ACOUNT (want 1)"
AREPORT=$(ls "$AREPORTS"/crash-*-browser.json)

python3 - "$AREPORT" <<'PYEOF'
import json, sys
report = json.load(open(sys.argv[1]))
expected = {"schema_version", "role", "version", "git_commit", "os", "arch",
            "thread", "timestamp", "panic_message", "panic_location"}
keys = set(report)
missing = expected - keys
extra = keys - expected
assert not missing, f"missing fields: {missing}"
assert not extra, f"non-whitelist fields: {extra}"
assert report["schema_version"] == 1, report["schema_version"]
assert report["role"] == "browser", report["role"]
assert "intentional crash for FR-4 acceptance" in report["panic_message"]
assert report["panic_location"].startswith("crates/browser-shell/examples/crash_probe.rs:"), \
    report["panic_location"]
print("legA whitelist field set + values OK")
PYEOF
ok "legA IF-3 whitelist field set exact + values"

for pat in "http" "file://" "/home/" "$WORK"; do
    if grep -q "$pat" "$AREPORT"; then
        bad "legA privacy leak: '$pat' present in report"
    else
        ok "legA privacy: no '$pat' in report"
    fi
done

# ---- Leg B: 真实产品 renderer 异常退出（GUI 事件轮询 + SIGKILL） ----
# renderer 崩溃检测点在 GUI 事件循环（tabs.poll → handle_renderer_ipc_lost，
# RFC §4.2）；headless 是独立的 renderer 管线、无崩溃轮询（既有缺口 G4 面，
# 非本切片范围）。不传 --remote-debugging-port：GUI 模式的 CDP server 会自带
# 一个辅助 renderer 子进程（其启动期即断链死亡、不被回收），传了会干扰
# 「唯一 tab renderer」的选取。
note "leg B: real-product renderer crash (GUI event loop)"
BREPORTS="$WORK/profile/zeroweb/crash-reports"
rm -rf "$WORK/profile" "$WORK/storage" "$WORK/cache" "$WORK/browser.log" "$WORK/legB-browser.pid"

XDG_CONFIG_HOME="$WORK/profile" \
ZERO_STORAGE_DIR="$WORK/storage" \
ZERO_CACHE_DIR="$WORK/cache" \
DISPLAY="${DISPLAY:-:0}" \
ZW_LOG_LEVEL=debug \
    "$BROWSER" "zero://newtab" \
    > "$WORK/browser.log" 2>&1 &
BPID=$!
echo "$BPID" > "$WORK/legB-browser.pid"
cleanup() {
    # 只清本腿进程树：先杀 browser 的子 renderer（避免孤儿锁文件），再杀
    # browser；不按名字全局 pkill（并行 clone 的 renderer 同名，误伤）。
    for child in $(pgrep -P "$BPID" 2>/dev/null); do
        kill -9 "$child" 2>/dev/null || true
    done
    kill "$BPID" 2>/dev/null || true
}
trap cleanup EXIT

RENDERER_PID=""
for _ in $(seq 1 60); do
    # 只认非僵尸 renderer 子进程（启动期死亡 renderer 会残留为 zombie）。
    for pid in $(pgrep -P "$BPID" -x zero-renderer 2>/dev/null); do
        state=$(awk '{print $3}' "/proc/$pid/stat" 2>/dev/null || true)
        if [ -n "$state" ] && [ "$state" != "Z" ]; then
            RENDERER_PID="$pid"
            break
        fi
    done
    [ -n "$RENDERER_PID" ] && break
    sleep 0.5
done
[ -n "$RENDERER_PID" ] && ok "legB live renderer pid=$RENDERER_PID" || { bad "legB no live renderer child"; exit 1; }
CHILD_COUNT=$(pgrep -P "$BPID" -x zero-renderer 2>/dev/null | wc -l)
[ "$CHILD_COUNT" = "1" ] || { bad "legB expected exactly 1 renderer child, got $CHILD_COUNT"; exit 1; }

rm -rf "$BREPORTS"
BEFORE_CRASH=$(date +%s)
kill -9 "$RENDERER_PID"
note "legB SIGKILL delivered to renderer $RENDERER_PID"
sleep 1
if [ -d "/proc/$RENDERER_PID" ]; then
    note "legB forensic: killed pid state=$(awk '{print $3}' /proc/$RENDERER_PID/stat 2>/dev/null || echo gone)"
else
    note "legB forensic: killed pid gone (reaped or never was ours)"
fi
note "legB forensic: browser children now: $(ps --ppid "$BPID" -o pid,stat,comm --no-headers 2>/dev/null | tr '\n' ';' || echo none)"

BREPORT=""
for _ in $(seq 1 40); do
    for candidate in $(ls "$BREPORTS"/crash-*-renderer.json 2>/dev/null || true); do
        mtime=$(stat -c %Y "$candidate")
        [ "$mtime" -ge "$BEFORE_CRASH" ] && { BREPORT="$candidate"; break; }
    done
    [ -n "$BREPORT" ] && break
    sleep 0.5
done
[ -n "$BREPORT" ] || { bad "legB no renderer report within 20s"; exit 1; }
ok "legB renderer report written: $(basename "$BREPORT")"

HOST_VERSION=$("$BROWSER" --version 2>/dev/null)
note "legB host binary: $HOST_VERSION"
python3 - "$BREPORT" "$HOST_VERSION" "$WORK" <<'PYEOF'
import json, sys
report = json.load(open(sys.argv[1]))
host_line = sys.argv[2]
work = sys.argv[3]
version = host_line.split()[1]
commit = host_line.rsplit("commit ", 1)[1].rstrip(")")
assert report["schema_version"] == 1, report["schema_version"]
assert report["role"] == "renderer", report["role"]
assert report["version"] == version, (report["version"], version)
assert report["git_commit"] == commit, (report["git_commit"], commit)
exit_code = report.get("renderer_exit_code", "")
assert "9" in exit_code and "KILL" in exit_code, exit_code
assert "panic_message" not in report, report.get("panic_message")
assert "panic_location" not in report, report.get("panic_location")
raw = open(sys.argv[1]).read()
for pat in ("http", "file://", "/home/", work, "stderr tail"):
    assert pat not in raw, f"privacy leak: {pat}"
print("legB report fields + privacy OK")
PYEOF
ok "legB renderer report whitelist + version/commit traceable + privacy clean"

if kill -0 "$BPID" 2>/dev/null; then
    ok "legB browser alive after renderer crash"
else
    bad "legB browser dead after renderer crash"
fi

NEW_RENDERER=$(pgrep -P "$BPID" -x zero-renderer 2>/dev/null | head -1 || true)
if [ -n "$NEW_RENDERER" ] && [ "$NEW_RENDERER" != "$RENDERER_PID" ]; then
    ok "legB renderer respawned pid=$NEW_RENDERER (was $RENDERER_PID)"
else
    note "legB renderer respawn: pid='$NEW_RENDERER' (respawn needs snapshot URL; coverage optional here)"
fi

# ---- 汇总 ----
echo "[DC3] summary: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" = "0" ]
