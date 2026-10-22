#!/bin/sh
# DC-3 后半 + DC-4（S4）验收：反馈导出与崩溃循环守卫。
#
#   leg A  崩溃循环守卫（真实产品，GUI）：连续两次 SIGKILL 同一 tab 的
#          renderer → session.json 的 crash_count 逐次递增并即时落盘；
#          SIGINT 干净退出（clean_exit=true）→ 重启自动恢复 → 达阈值 tab
#          置空 about:blank 不自动加载崩溃页；存活观察窗（10s）后计数清零。
#   leg B  反馈导出（真实二进制 CLI）：列举 → 同意导出（目录形态 IF-4，
#          reports/ 原样 + environment.json + 可选 session-summary）→
#          重复导出拒绝 → 删除 → URL 仅显式勾选时进入摘要 → 无报告拒绝。
#
# 用法: run-dc3b-dc4-acceptance.sh <repo-root> <work-dir>
# 前置: make browser-build（target/release/{zero-browser,zero-renderer}）。
# 全程隔离：XDG_CONFIG_HOME/ZERO_STORAGE_DIR/ZERO_CACHE_DIR 均在 work-dir 内；
# 任一断言 FAIL 退出码 1。
set -eu

REPO="$1"; WORK="$2"
BROWSER="$REPO/target/release/zero-browser"
PASS=0; FAIL=0

note() { echo "[DC4] $*"; }
ok()   { PASS=$((PASS+1)); echo "[DC4] PASS: $*"; }
bad()  { FAIL=$((FAIL+1)); echo "[DC4] FAIL: $*"; }

for f in "$BROWSER" "$REPO/target/release/zero-renderer"; do
    [ -x "$f" ] || { echo "[DC4] missing $f (run make browser-build first)"; exit 2; }
done

mkdir -p "$WORK"
cd "$REPO"

BREPORTS="$WORK/profile/zeroweb/crash-reports"
SESSION_JSON="$WORK/profile/zeroweb/session.json"
rm -rf "$WORK/profile" "$WORK/storage" "$WORK/cache" "$WORK/export" "$WORK/export-urls" "$WORK/export-empty"
rm -f "$WORK"/*.log "$WORK"/leg*.pid "$WORK"/legB-*.out "$WORK"/legB-*.txt "$WORK"/legB-*.err

start_browser() {
    # 隔离环境启动 GUI 浏览器（无 CDP；S3 验收确认 CDP 会引入干扰性辅助
    # renderer）。日志转 debug 供守卫断言。
    XDG_CONFIG_HOME="$WORK/profile" \
    ZERO_STORAGE_DIR="$WORK/storage" \
    ZERO_CACHE_DIR="$WORK/cache" \
    DISPLAY="${DISPLAY:-:0}" \
    ZW_LOG_LEVEL=debug \
        "$BROWSER" "$1" >> "$2" 2>&1 &
    echo $!
}

live_renderer() {
    # 首个非僵尸 renderer 子进程（启动期死亡 renderer 残留为 zombie）。
    for pid in $(pgrep -P "$1" -x zero-renderer 2>/dev/null); do
        state=$(awk '{print $3}' "/proc/$pid/stat" 2>/dev/null || true)
        [ -n "$state" ] && [ "$state" != "Z" ] && { echo "$pid"; return; }
    done
}

kill_tree() {
    for child in $(pgrep -P "$1" 2>/dev/null); do
        kill -9 "$child" 2>/dev/null || true
    done
    kill -9 "$1" 2>/dev/null || true
}

wait_for_python() {
    # 轮询执行 python 断言片段，最多 $1 秒。
    deadline=$(( $(date +%s) + $1 ))
    while [ "$(date +%s)" -lt "$deadline" ]; do
        python3 - "$@" >/dev/null 2>&1 && return 0
        sleep 0.5
    done
    return 1
}

# ---- Leg A: 崩溃循环守卫（FR-6 / DC-4） ----
note "leg A: crash-loop guard (double kill + clean restart + restore skip)"
ALOG="$WORK/legA-browser.log"
BPID=$(start_browser "zero://newtab" "$ALOG")
echo "$BPID" > "$WORK/legA.pid"
cleanup() {
    for pid in "$BPID" "$BPID2"; do
        [ -n "$pid" ] && [ "$pid" != "0" ] && kill_tree "$pid" 2>/dev/null || true
    done
}
trap cleanup EXIT

R1=""
for _ in $(seq 1 60); do
    R1=$(live_renderer "$BPID" || true)
    [ -n "$R1" ] && break
    sleep 0.5
done
[ -n "$R1" ] || { bad "legA no live renderer child"; exit 1; }
CHILD_COUNT=$(pgrep -P "$BPID" -x zero-renderer 2>/dev/null | wc -l)
[ "$CHILD_COUNT" = "1" ] || { bad "legA expected exactly 1 renderer, got $CHILD_COUNT"; exit 1; }
ok "legA browser up, exactly 1 tab renderer pid=$R1"

# 第一次崩溃：计数 1（即时落盘）。
kill -9 "$R1"
CRASH_COUNT=0
for _ in $(seq 1 20); do
    CRASH_COUNT=$(python3 -c "
import json,sys
try:
    s=json.load(open('$SESSION_JSON'))
    print(max((t.get('crash_count',0) for t in s['tabs']), default=0))
except Exception:
    print(0)
" 2>/dev/null || echo 0)
    [ "$CRASH_COUNT" = "1" ] && break
    sleep 0.5
done
[ "$CRASH_COUNT" = "1" ] && ok "legA crash #1: session.json crash_count=1 persisted" \
    || bad "legA crash #1: crash_count=$CRASH_COUNT (want 1)"

# 第二次崩溃（观察窗内）：计数 2。
R2=$(live_renderer "$BPID" || true)
[ -n "$R2" ] || { bad "legA renderer did not respawn after crash #1"; exit 1; }
ok "legA renderer respawned pid=$R2"
# 报告文件名含 unix 秒：等墙钟严格跨过报告 #1 的秒再补第二刀，否则两份
# 报告同名互相覆盖（肯候锚点是报告落盘时刻，不是 kill 发出时刻）。
REPORT1_STAMP=$(ls "$BREPORTS"/crash-*-renderer.json | head -1 | sed 's/.*crash-//; s/-renderer\.json//')
while [ "$(date +%s)" -le "$REPORT1_STAMP" ]; do sleep 0.2; done
kill -9 "$R2"
CRASH_COUNT=0
for _ in $(seq 1 20); do
    CRASH_COUNT=$(python3 -c "
import json,sys
try:
    s=json.load(open('$SESSION_JSON'))
    print(max((t.get('crash_count',0) for t in s['tabs']), default=0))
except Exception:
    print(0)
" 2>/dev/null || echo 0)
    [ "$CRASH_COUNT" = "2" ] && break
    sleep 0.5
done
[ "$CRASH_COUNT" = "2" ] && ok "legA crash #2: session.json crash_count=2 persisted" \
    || bad "legA crash #2: crash_count=$CRASH_COUNT (want 2)"

# 干净退出（SIGINT → Ctrl+C handler → clean_exit=true），再重启自动恢复。
kill -INT "$BPID"
for _ in $(seq 1 20); do
    kill -0 "$BPID" 2>/dev/null || break
    sleep 0.5
done
CLEAN=$(python3 -c "
import json
try:
    print(json.load(open('$SESSION_JSON'))['clean_exit'])
except Exception:
    print(False)
" 2>/dev/null || echo False)
[ "$CLEAN" = "True" ] && ok "legA clean exit: session.json clean_exit=true" \
    || bad "legA clean_exit=$CLEAN (want True)"

note "legA restarting browser (auto-restore path, clean_exit=true)"
ALOG2="$WORK/legA2-browser.log"
BPID2=$(start_browser "zero://newtab" "$ALOG2")
SKIP=0
for _ in $(seq 1 20); do
    grep -q "skipping auto-load of crash-loop tab" "$ALOG2" 2>/dev/null && { SKIP=1; break; }
    sleep 0.5
done
[ "$SKIP" = "1" ] && ok "legA restore: crash-loop tab auto-load skipped (about:blank)" \
    || bad "legA restore: no skip log line in legA2-browser.log"

# 观察窗（10s）内存活无新崩溃 → 计数清零并落盘。
RESET=0
for _ in $(seq 1 36); do
    CRASH_COUNT=$(python3 -c "
import json
try:
    s=json.load(open('$SESSION_JSON'))
    print(max((t.get('crash_count',0) for t in s['tabs']), default=0))
except Exception:
    print(-1)
" 2>/dev/null || echo -1)
    [ "$CRASH_COUNT" = "0" ] && { RESET=1; break; }
    sleep 0.5
done
[ "$RESET" = "1" ] && ok "legA observation window: crash_count reset to 0 after ~10s" \
    || bad "legA observation window: crash_count=$CRASH_COUNT still non-zero after 18s"

# ---- Leg B: 反馈导出（FR-5 / IF-4 目录形态 / DC-3 后半） ----
note "leg B: feedback export (list / consent-export / dup-reject / delete / urls opt-in)"
REPORT_COUNT=$(ls "$BREPORTS"/crash-*-renderer.json 2>/dev/null | wc -l)
[ "$REPORT_COUNT" -ge 2 ] && ok "legB precondition: $REPORT_COUNT renderer reports from leg A" \
    || bad "legB precondition: only $REPORT_COUNT renderer reports"

LIST_OUT="$WORK/legB-list.txt"
XDG_CONFIG_HOME="$WORK/profile" "$BROWSER" --list-crash-reports > "$LIST_OUT" 2>"$WORK/legB-list.err"
grep -q "^REPORT crash-.*-renderer\.json renderer [0-9]* [0-9]*$" "$LIST_OUT" \
    && ok "legB --list-crash-reports: preview lines well-formed" \
    || bad "legB list output malformed: $(cat "$LIST_OUT")"

HOST_VERSION=$("$BROWSER" --version 2>/dev/null)
EXPORT_ROOT="$WORK/export"
OUT_DIR="$WORK/legB-export.out"
XDG_CONFIG_HOME="$WORK/profile" "$BROWSER" \
    --export-feedback="$EXPORT_ROOT" --include-session-summary > "$OUT_DIR" 2>&1
PACKAGE=$(grep -o "$EXPORT_ROOT/zeroweb-feedback-[^ ]*" "$OUT_DIR" | head -1)
[ -n "$PACKAGE" ] && [ -d "$PACKAGE" ] && ok "legB export: $PACKAGE" || { bad "legB export failed: $(cat "$OUT_DIR")"; exit 1; }

python3 - "$PACKAGE" "$BREPORTS" "$HOST_VERSION" "$WORK" <<'PYEOF'
import json, sys, os
package, reports_dir, host_line, work = sys.argv[1:5]
version = host_line.split()[1]
commit = host_line.rsplit("commit ", 1)[1].rstrip(")")
# reports/ 原样（与源逐字节一致）。
src = sorted(os.listdir(reports_dir))
packed = sorted(os.listdir(os.path.join(package, "reports")))
assert packed == src, (packed, src)
for name in packed:
    assert open(os.path.join(reports_dir, name), "rb").read() == open(os.path.join(package, "reports", name), "rb").read(), name
env = json.load(open(os.path.join(package, "environment.json")))
assert env["schema_version"] == 1, env
assert env["version"] == version, (env["version"], version)
assert env["git_commit"] == commit, (env["git_commit"], commit)
assert env["os"] and env["arch"] and env["generated_at"] > 0, env
assert [r["file_name"] for r in env["reports"]] == src, env["reports"]
summary = json.load(open(os.path.join(package, "session-summary.json")))
assert summary["tab_count"] == 1, summary
assert "urls" not in summary, "URL 不得默认进入摘要"
# 隐私：整包无 URL/宿主路径串。
for root, _, files in os.walk(package):
    for f in files:
        raw = open(os.path.join(root, f), "rb").read().decode("utf-8", "replace")
        for pat in ("http", "file://", "/home/", work):
            assert pat not in raw, f"privacy leak {pat!r} in {f}"
print("legB package structure + environment + privacy OK")
PYEOF
ok "legB IF-4 layout: reports byte-identical + env(version/commit/os/arch/manifest) + summary w/o URLs + privacy clean"

XDG_CONFIG_HOME="$WORK/profile" "$BROWSER" \
    --export-feedback="$EXPORT_ROOT" > "$WORK/legB-dup.out" 2>&1 \
    && bad "legB duplicate export should fail" || true
grep -q "already exists" "$WORK/legB-dup.out" \
    && ok "legB duplicate export rejected without clobber" \
    || bad "legB duplicate rejection message missing: $(cat "$WORK/legB-dup.out")"

VICTIM=$(ls "$BREPORTS"/crash-*-renderer.json | head -1 | xargs basename)
XDG_CONFIG_HOME="$WORK/profile" "$BROWSER" "--delete-crash-report=$VICTIM" > "$WORK/legB-del.out" 2>&1
grep -q "REPORT_DELETED $VICTIM" "$WORK/legB-del.out" && [ ! -f "$BREPORTS/$VICTIM" ] \
    && ok "legB --delete-crash-report removes the file" \
    || bad "legB delete failed: $(cat "$WORK/legB-del.out")"
XDG_CONFIG_HOME="$WORK/profile" "$BROWSER" "--delete-crash-report=../../etc/passwd" > "$WORK/legB-del2.out" 2>&1 \
    && bad "legB traversal delete should fail" || true
grep -q "REPORT_DELETE_REJECTED" "$WORK/legB-del2.out" \
    && ok "legB traversal delete rejected" || bad "legB traversal not rejected: $(cat "$WORK/legB-del2.out")"

# URL 显式勾选：第二个包的摘要含 URL。
URLS_ROOT="$WORK/export-urls"
XDG_CONFIG_HOME="$WORK/profile" "$BROWSER" \
    --export-feedback="$URLS_ROOT" --include-session-urls > "$WORK/legB-urls.out" 2>&1
URLS_PACKAGE=$(grep -o "$URLS_ROOT/zeroweb-feedback-[^ ]*" "$WORK/legB-urls.out" | head -1)
python3 - "$URLS_PACKAGE" <<'PYEOF'
import json, sys
summary = json.load(open(sys.argv[1] + "/session-summary.json"))
assert summary.get("urls") == ["zero://newtab"], summary
print("legB explicit urls opt-in OK")
PYEOF
ok "legB --include-session-urls: URLs enter summary only on explicit opt-in"

# 无报告拒绝：清空报告目录后导出 → 拒绝且不产生任何文件。
rm -rf "$BREPORTS"
EMPTY_ROOT="$WORK/export-empty"
XDG_CONFIG_HOME="$WORK/profile" "$BROWSER" "--export-feedback=$EMPTY_ROOT" > "$WORK/legB-empty.out" 2>&1 \
    && bad "legB export without reports should fail" || true
grep -q "FEEDBACK_REJECTED NO_REPORTS" "$WORK/legB-empty.out" \
    && ok "legB no-reports export rejected" || bad "legB no-reports message missing: $(cat "$WORK/legB-empty.out")"
if [ ! -e "$EMPTY_ROOT" ]; then
    ok "legB rejection leaves no files (refusal = nothing sent)"
elif [ -z "$(ls -A "$EMPTY_ROOT" 2>/dev/null)" ]; then
    ok "legB rejection leaves no files (empty dir only)"
else
    bad "legB rejection created files: $(ls -A "$EMPTY_ROOT" 2>/dev/null)"
fi

# ---- 汇总 ----
echo "[DC4] summary: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" = "0" ]
