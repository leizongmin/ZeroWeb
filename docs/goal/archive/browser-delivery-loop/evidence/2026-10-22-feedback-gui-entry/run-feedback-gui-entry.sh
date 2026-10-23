#!/bin/sh
# FR-5 GUI 导出入口（zero://feedback 内部页）验收：真实进程两腿 + app 级测试映射。
#
#   leg A  GUI 面回归：GUI + --profile 启动 → SIGKILL 唯一 tab renderer 产生
#          真实崩溃报告（G3 leg B 模式）→ SIGINT 优雅退出（feedback.rs/pages.rs/
#          app.rs 改动被真实 GUI 进程整体加载，无启动/退出回归）。
#   leg B  GUI 导出同管线字节来源：同一 profile 上 CLI --export-format=zip 导出
#          （GUI export 动作与 CLI 共用 build_feedback_zip_bytes）→ python3
#          zipfile 真解压深校验（CRC/成员逐字节/version-commit 贯穿）。
#   动作层（页面渲染/删除/导出经下载管线落盘/穿越拒绝）由 app 级测试覆盖：
#   tests::feedback_page_lists_reports_deletes_and_exports_zip_via_download +
#   pages::generate_feedback_html_*（BrowserApp 真实 shell + CPU 渲染 + 真实
#   下载管线），见 run-feedback-gui-entry.log 的门禁段。
#
# 用法: run-feedback-gui-entry.sh <repo-root> <work-dir>
# 前置: make browser-build（target/release/{zero-browser,zero-renderer}）。
# 全程隔离：--profile 与环境变量兜底均指向 work-dir，真实 HOME 零接触。
# 任一断言 FAIL 退出码 1。
set -eu

REPO="$1"; WORK="$2"
BROWSER="$REPO/target/release/zero-browser"
PASS=0; FAIL=0

note() { echo "[GUI] $*"; }
ok()   { PASS=$((PASS+1)); echo "[GUI] PASS: $*"; }
bad()  { FAIL=$((FAIL+1)); echo "[GUI] FAIL: $*"; }

live_renderer() {
    for pid in $(pgrep -P "$1" -x zero-renderer 2>/dev/null || true); do
        state=$(awk '{print $3}' "/proc/$pid/stat" 2>/dev/null || true)
        [ -n "$state" ] && [ "$state" != "Z" ] && { echo "$pid"; return; }
    done
}

zip_deep_check() {
    python3 - "$1" "$2" "$3" "$4" <<'PYEOF'
import json, os, sys, zipfile

zip_path, reports_dir, ver, commit = sys.argv[1:5]
z = zipfile.ZipFile(zip_path)
assert z.testzip() is None, "CRC/integrity failure"
names = z.namelist()
assert "environment.json" in names, names
report_members = [n for n in names if n.startswith("reports/") and n != "reports/"]
assert report_members, f"no report members: {names}"
env = json.loads(z.read("environment.json"))
assert env["version"] == ver, env["version"]
assert env["git_commit"] == commit, env["git_commit"]
for n in report_members:
    with open(os.path.join(reports_dir, os.path.basename(n)), "rb") as f:
        assert z.read(n) == f.read(), f"member {n} differs"
print(f"deep-check OK: {len(report_members)} report member(s)")
PYEOF
}

mkdir -p "$WORK"
cd "$REPO"
for f in "$BROWSER" "$REPO/target/release/zero-renderer"; do
    [ -e "$f" ] || { echo "[GUI] missing $f (build first)"; exit 2; }
done

VERSION_OUT=$("$BROWSER" --version)
PROD_VER=$(echo "$VERSION_OUT" | awk '{print $2}')
COMMIT=$(echo "$VERSION_OUT" | sed -n 's/.*commit \([0-9a-f]*\).*/\1/p')
note "product identity: $VERSION_OUT"

# ---- leg A: 真实 GUI 进程回归 + 报告落盘 ----
note "leg A: GUI process loads new feedback code paths, crash report lands"
P1="$WORK/p1"
BREPORTS="$P1/zeroweb/crash-reports"
OUT="$WORK/out"
rm -rf "$P1" "$OUT" "$WORK/env-xdg" "$WORK/env-storage" "$WORK/env-cache"
mkdir -p "$WORK/env-xdg"
XDG_CONFIG_HOME="$WORK/env-xdg" \
ZERO_STORAGE_DIR="$WORK/env-storage" \
ZERO_CACHE_DIR="$WORK/env-cache" \
DISPLAY="${DISPLAY:-:0}" \
ZW_LOG_LEVEL=debug \
    "$BROWSER" --profile="$P1" zero://newtab \
    > "$WORK/legA-stdout.log" 2> "$WORK/legA-stderr.log" &
PID_A=$!
cleanup_a() {
    for child in $(pgrep -P "$PID_A" 2>/dev/null); do kill -9 "$child" 2>/dev/null || true; done
    kill -9 "$PID_A" 2>/dev/null || true
}
trap cleanup_a EXIT

REN_A=""
for _ in $(seq 1 60); do
    REN_A=$(live_renderer "$PID_A")
    [ -n "$REN_A" ] && break
    kill -0 "$PID_A" 2>/dev/null || break
    sleep 0.5
done
[ -n "$REN_A" ] && ok "legA live renderer pid=$REN_A" \
    || { bad "legA no live renderer child"; exit 1; }

kill -9 "$REN_A"
REPORT=""
for _ in $(seq 1 30); do
    REPORT=$(ls "$BREPORTS"/crash-*.json 2>/dev/null | head -1 || true)
    [ -n "$REPORT" ] && break
    sleep 0.5
done
[ -n "$REPORT" ] && ok "legA real crash report: $(basename "$REPORT")" \
    || { bad "legA no crash report under profile"; exit 1; }

kill -INT "$PID_A"
for _ in $(seq 1 40); do
    kill -0 "$PID_A" 2>/dev/null || break
    sleep 0.25
done
if kill -0 "$PID_A" 2>/dev/null; then
    bad "legA browser still alive 10s after SIGINT"
    cleanup_a
else
    wait "$PID_A" 2>/dev/null || true
    ok "legA GUI exited cleanly after SIGINT (no startup/exit regression)"
fi
trap - EXIT

# ---- leg B: GUI 导出同管线的 zip 字节来源（CLI 复现）----
note "leg B: shared build_feedback_zip_bytes pipeline verified via CLI export"
EXPORT_OUT=$("$BROWSER" --profile="$P1" --export-feedback="$OUT" --export-format=zip 2>/dev/null || true)
echo "$EXPORT_OUT" | grep -q "^FEEDBACK_EXPORTED .*\.zip$" \
    && ok "legB zip export: $EXPORT_OUT" \
    || { bad "legB zip export unexpected: $EXPORT_OUT"; exit 1; }
ZIP_PATH=$(echo "$EXPORT_OUT" | awk '{print $2}')
DEEP_OUT=$(zip_deep_check "$ZIP_PATH" "$BREPORTS" "$PROD_VER" "$COMMIT") \
    && ok "legB deep check (CRC/members byte-identity/version-commit): $DEEP_OUT" \
    || bad "legB deep check failed: $DEEP_OUT"

# ---- 汇总 ----
echo "[GUI] summary: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" = "0" ]
