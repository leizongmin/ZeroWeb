#!/bin/sh
# 反馈包 zip 形态（RFC §4.4/IF-4 原始形态，--export-format=zip）验收：真实进程三腿。
#
#   leg A  真实报告 + zip 导出深校验：GUI + --profile 启动 → SIGKILL 唯一 tab
#          renderer 产生真实崩溃报告（G3 leg B 模式）→ --export-format=zip
#          导出 → python3 zipfile 真解压语义深校验（testzip CRC 完整性、成员
#          名与目录形态一致、报告逐字节一致、environment.json version/commit
#          与 --version 贯穿、无 summary 成员除非显式勾选）→ 整包隐私 grep
#          全 0 → 重复导出拒绝且字节不变。
#   leg B  缺省回归：不带 --export-format 恒目录形态；目录树成员与 zip 成员
#          一一对应（同参数两形态内容一致）。
#   leg C  负例与参数面：空 profile 无报告 → FEEDBACK_REJECTED NO_REPORTS 且
#          零文件；--export-format=tab 非法值 → exit 2；勾选不存在报告 →
#          FEEDBACK_REJECTED。
#
# 用法: run-feedback-zip-acceptance.sh <repo-root> <work-dir>
# 前置: make browser-build（target/release/{zero-browser,zero-renderer}）。
# 全程隔离：--profile 与环境变量兜底均指向 work-dir，真实 HOME 零接触。
# 任一断言 FAIL 退出码 1。
set -eu

REPO="$1"; WORK="$2"
BROWSER="$REPO/target/release/zero-browser"
PASS=0; FAIL=0

note() { echo "[ZIP] $*"; }
ok()   { PASS=$((PASS+1)); echo "[ZIP] PASS: $*"; }
bad()  { FAIL=$((FAIL+1)); echo "[ZIP] FAIL: $*"; }

# 指定 browser pid 的一个非僵尸 renderer 子进程 pid（无则空）。
live_renderer() {
    for pid in $(pgrep -P "$1" -x zero-renderer 2>/dev/null || true); do
        state=$(awk '{print $3}' "/proc/$pid/stat" 2>/dev/null || true)
        [ -n "$state" ] && [ "$state" != "Z" ] && { echo "$pid"; return; }
    done
}

# 真解压语义深校验：CRC（testzip）、成员名、报告逐字节一致、version/commit
# 贯穿、summary 成员按需出现。输出差异即非零退出。
zip_deep_check() {
    python3 - "$1" "$2" "$3" "$4" "$5" <<'PYEOF'
import json, os, sys, zipfile

zip_path, reports_dir, ver, commit, want_summary = sys.argv[1:6]
z = zipfile.ZipFile(zip_path)
assert z.testzip() is None, "CRC/integrity failure"
names = z.namelist()
assert "environment.json" in names, f"missing environment.json: {names}"
report_members = [n for n in names if n.startswith("reports/") and n != "reports/"]
assert report_members, f"no report members: {names}"
env = json.loads(z.read("environment.json"))
assert env["schema_version"] == 1, env
assert env["version"] == ver, f"version {env['version']} != {ver}"
assert env["git_commit"] == commit, f"commit {env['git_commit']} != {commit}"
env_report_names = {f"reports/{r['file_name']}" for r in env["reports"]}
assert env_report_names == set(report_members), f"{env_report_names} != {set(report_members)}"
for n in report_members:
    orig = os.path.join(reports_dir, os.path.basename(n))
    with open(orig, "rb") as f:
        assert z.read(n) == f.read(), f"member {n} differs from source report"
assert ("session-summary.json" in names) == bool(int(want_summary)), names
if int(want_summary):
    summary = json.loads(z.read("session-summary.json"))
    assert "urls" not in summary, "summary must not carry URLs unless opted in"
print(f"deep-check OK: {len(report_members)} report member(s), summary={bool(int(want_summary))}")
PYEOF
}

# 整包隐私 grep：解包全部成员内容后匹配红线词（URL/凭据类），命中即泄漏。
zip_privacy_scan() {
    python3 - "$1" <<'PYEOF'
import re, sys, zipfile

z = zipfile.ZipFile(sys.argv[1])
leak = re.compile(rb"https?://|cookie|token|authorization", re.IGNORECASE)
hits = [n for n in z.namelist() if leak.search(z.read(n))]
assert not hits, f"privacy red-line hit in: {hits}"
print("privacy scan OK: 0 hits")
PYEOF
}

mkdir -p "$WORK"
cd "$REPO"
for f in "$BROWSER" "$REPO/target/release/zero-renderer"; do
    [ -e "$f" ] || { echo "[ZIP] missing $f (build first)"; exit 2; }
done

VERSION_OUT=$("$BROWSER" --version)
PROD_VER=$(echo "$VERSION_OUT" | awk '{print $2}')
COMMIT=$(echo "$VERSION_OUT" | sed -n 's/.*commit \([0-9a-f]*\).*/\1/p')
note "product identity: $VERSION_OUT"

# ---- leg A: 真实报告 + zip 导出深校验 ----
note "leg A: real crash report exported as zip, deep-checked"
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
note "legA SIGKILL delivered to renderer $REN_A"
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
    ok "legA browser exited cleanly after SIGINT"
fi
trap - EXIT

# 注意：set -eu 下命令替换非零退出会静默终止脚本（首轮曾因此只看到 4 个
# PASS 就断流），关键 CLI 捕获一律 || true 再显式断言输出。
EXPORT_OUT=$("$BROWSER" --profile="$P1" --export-feedback="$OUT" --export-format=zip 2>/dev/null || true)
echo "$EXPORT_OUT" | grep -q "^FEEDBACK_EXPORTED .*\.zip$" \
    && ok "legA --export-format=zip exports: $EXPORT_OUT" \
    || { bad "legA zip export unexpected output: $EXPORT_OUT"; exit 1; }
ZIP_PATH=$(echo "$EXPORT_OUT" | awk '{print $2}')
[ -f "$ZIP_PATH" ] && ok "legA zip file on disk" || bad "legA zip missing: $ZIP_PATH"
ZIP_NAME=$(basename "$ZIP_PATH")
case "$ZIP_NAME" in
    zeroweb-feedback-"$PROD_VER"-*.zip) ok "legA zip name follows <version>-<date>: $ZIP_NAME" ;;
    *) bad "legA zip name unexpected: $ZIP_NAME" ;;
esac

DEEP_OUT=$(zip_deep_check "$ZIP_PATH" "$BREPORTS" "$PROD_VER" "$COMMIT" 0) \
    && ok "legA deep check (CRC/members/byte-identity/version-commit/summary-gated): $DEEP_OUT" \
    || bad "legA deep check failed: $DEEP_OUT"
PRIV_OUT=$(zip_privacy_scan "$ZIP_PATH") \
    && ok "legA privacy scan: $PRIV_OUT" \
    || bad "legA privacy scan FAILED: $PRIV_OUT"

# 显式勾选 --include-session-summary 后才有 summary 成员（NFR-1 分级同意）。
OUT_S="$WORK/out-summary"
SUM_OUT=$("$BROWSER" --profile="$P1" --export-feedback="$OUT_S" --export-format=zip --include-session-summary 2>/dev/null || true)
ZIP_S=$(echo "$SUM_OUT" | awk '{print $2}')
[ -n "$ZIP_S" ] && [ -f "$ZIP_S" ] || { bad "legA summary export failed: $SUM_OUT"; exit 1; }
ZIP_S=$(echo "$SUM_OUT" | awk '{print $2}')
zip_deep_check "$ZIP_S" "$BREPORTS" "$PROD_VER" "$COMMIT" 1 > /dev/null \
    && ok "legA summary opt-in export passes deep check (session-summary.json present)" \
    || bad "legA summary opt-in export failed deep check"

# 重复导出拒绝且既有 zip 字节不变。
HASH_BEFORE=$(sha256sum "$ZIP_PATH" | awk '{print $1}')
DUP_OUT=$("$BROWSER" --profile="$P1" --export-feedback="$OUT" --export-format=zip 2>&1 || true)
echo "$DUP_OUT" | grep -q "FEEDBACK_REJECTED" \
    && ok "legA duplicate export rejected: $(echo "$DUP_OUT" | head -1)" \
    || bad "legA duplicate export not rejected: $DUP_OUT"
HASH_AFTER=$(sha256sum "$ZIP_PATH" | awk '{print $1}')
[ "$HASH_BEFORE" = "$HASH_AFTER" ] && ok "legA existing zip untouched after rejection" \
    || bad "legA zip bytes changed"

# ---- leg B: 缺省目录形态回归 + 两形态成员一致 ----
note "leg B: default export stays directory form; members match zip"
OUT_D="$WORK/out-dir"
DIR_OUT=$("$BROWSER" --profile="$P1" --export-feedback="$OUT_D" 2>/dev/null || true)
echo "$DIR_OUT" | grep -q "^FEEDBACK_EXPORTED " \
    && ok "legB default (no --export-format) exports directory: $DIR_OUT" \
    || { bad "legB default export unexpected: $DIR_OUT"; exit 1; }
DIR_PATH=$(echo "$DIR_OUT" | awk '{print $2}')
[ -d "$DIR_PATH" ] && ok "legB exported path is a directory" || bad "legB not a directory: $DIR_PATH"

MEMBER_CMP=$(python3 - "$ZIP_PATH" "$DIR_PATH" <<'PYEOF'
import os, sys, zipfile

zip_path, dir_path = sys.argv[1], sys.argv[2]
z = zipfile.ZipFile(zip_path)
zip_names = sorted(n for n in z.namelist() if not n.endswith("/"))
dir_names = sorted(
    os.path.relpath(os.path.join(root, f), dir_path)
    for root, _, files in os.walk(dir_path)
    for f in files
)
assert zip_names == dir_names, f"{zip_names} != {dir_names}"
for n in zip_names:
    with open(os.path.join(dir_path, n), "rb") as f:
        assert z.read(n) == f.read(), f"member {n} content differs"
print(f"members identical: {len(zip_names)} file(s)")
PYEOF
)
[ -n "$MEMBER_CMP" ] && ok "legB zip members byte-identical to directory package: $MEMBER_CMP" \
    || bad "legB member comparison failed"

# ---- leg C: 负例与参数面 ----
note "leg C: negatives — no reports, bad format, missing report"
EMPTY="$WORK/empty-profile"
rm -rf "$EMPTY" "$WORK/out-empty"
"$BROWSER" --profile="$EMPTY" --export-feedback="$WORK/out-empty" --export-format=zip > "$WORK/legC-stdout.log" 2> "$WORK/legC-stderr.log" \
    && bad "legC export without reports should fail" \
    || true
grep -q "FEEDBACK_REJECTED NO_REPORTS" "$WORK/legC-stderr.log" \
    && ok "legC NO_REPORTS rejected on empty profile" \
    || bad "legC expected FEEDBACK_REJECTED NO_REPORTS (got: $(cat "$WORK/legC-stderr.log"))"
[ ! -e "$WORK/out-empty" ] && ok "legC zero files produced on rejection" \
    || bad "legC files created despite rejection"

"$BROWSER" --profile="$P1" --export-feedback="$WORK/out-bad" --export-format=tar > "$WORK/legC2.log" 2>&1 \
    && bad "legC invalid format should exit nonzero" || true
grep -q "must be dir or zip" "$WORK/legC2.log" \
    && ok "legC invalid --export-format rejected with usage" \
    || bad "legC invalid format not rejected: $(head -1 "$WORK/legC2.log")"

BAD_OUT=$("$BROWSER" --profile="$P1" --export-feedback="$WORK/out-bad" --export-format=zip --feedback-report=crash-9999999999-browser.json 2>&1 || true)
echo "$BAD_OUT" | grep -q "FEEDBACK_REJECTED" \
    && ok "legC selected missing report rejected: $(echo "$BAD_OUT" | head -1)" \
    || bad "legC missing report not rejected: $BAD_OUT"
[ ! -e "$WORK/out-bad" ] && ok "legC no package on rejected selection" \
    || bad "legC package created despite rejection"

# ---- 汇总 ----
echo "[ZIP] summary: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" = "0" ]
