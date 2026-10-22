#!/bin/sh
# M4 全链路串测：在"已安装产物"上串联 DC-1..DC-4 机制层 + DC-5 清理审计。
#
#   leg1  初始安装    dpkg-deb -x v1 → <root>/releases/26.10.22 + current 链接
#                    （dpkg -i 的无根前缀隔离代理，M1 基线模式）
#   leg2  版本追溯    安装内 --version = `ZeroBrowser 26.10.22 (commit <HEAD>)`（IF-1）
#   leg3  更新应用    安装内二进制 --check-updates（无 ZERO_UPDATE_INSTALL_ROOT，
#                    证 current_exe 默认安装根推导）→ v2 原子翻转 + rollback 就位
#   leg4  崩溃报告    安装内 v2 GUI 启动 → SIGKILL renderer → 报告落盘，
#                    报告内 version/commit = 更新后版本（版本贯穿）
#   leg5  崩溃循环    第二刀 → crash_count=2 即时落盘 → SIGINT 干净退出 →
#                    重启自动恢复跳过 → 观察窗 10s 计数清零 → 再次 SIGINT
#   leg6  反馈导出    list → 目录包导出（env 版本/commit=v2、reports 原样、
#                    默认无 URL、整包隐私零命中）→ 重复导出拒绝
#   leg7  卸载完整性  删安装根（dpkg -r 代理：只删包提供的文件）→
#                    安装面消失，用户数据（会话/报告/已导出包）完整
#   leg8  清理审计    无 staging 残留、无残留进程、真实 ~/.config/zeroweb
#                    零写入、仓库工作树零污染
#
# 用法: run-m4-full-chain.sh <repo-root> <work-dir> <deb-v1=26.10.22> <deb-v2=26.10.23>
# 前置: 两个 deb 由 scripts/package-linux.sh --deb 构建（ZERO_BUILD_VERSION 区分）。
# 全程隔离：XDG_CONFIG_HOME/ZERO_STORAGE_DIR/ZERO_CACHE_DIR 均在 work-dir 内；
# 任一断言 FAIL 退出码 1。
set -eu

REPO="$1"; WORK="$2"; DEB_V1="$3"; DEB_V2="$4"
ROOT="$WORK/install-root"
BROWSER="$ROOT/current/usr/bin/zero-browser"
CFG="$WORK/profile"
BREPORTS="$CFG/zeroweb/crash-reports"
SESSION_JSON="$CFG/zeroweb/session.json"
PASS=0; FAIL=0

note() { echo "[M4] $*"; }
ok()   { PASS=$((PASS+1)); echo "[M4] PASS: $*"; }
bad()  { FAIL=$((FAIL+1)); echo "[M4] FAIL: $*"; }

COMMIT="$(git -C "$REPO" rev-parse --short=12 HEAD)"
VERSION_V1="26.10.22"; VERSION_V2="26.10.23"

mkdir -p "$WORK"
: > "$WORK/.chain-start"
rm -rf "$ROOT" "$CFG" "$WORK/storage" "$WORK/cache" "$WORK/export"
cd "$REPO"

start_browser() {
    # 隔离环境启动"已安装"GUI 浏览器。不注入 ZERO_UPDATE_INSTALL_ROOT：
    # 安装根由 current_exe 上溯三级默认推导（串测目标之一）。
    XDG_CONFIG_HOME="$CFG" \
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

graceful_stop() {
    # SIGINT → graceful 路径（session.json clean_exit=true）→ 等退出。
    kill -INT "$1" 2>/dev/null || return 0
    for _ in $(seq 1 20); do
        kill -0 "$1" 2>/dev/null || break
        sleep 0.5
    done
}

crash_count() {
    python3 -c "
import json
try:
    s=json.load(open('$SESSION_JSON'))
    print(max((t.get('crash_count',0) for t in s['tabs']), default=0))
except Exception:
    print(-1)
" 2>/dev/null || echo -1
}

BPID="" ; BPID2=""
cleanup() {
    for pid in "$BPID" "$BPID2"; do
        [ -n "$pid" ] && [ "$pid" != "0" ] && kill_tree "$pid" 2>/dev/null || true
    done
}
trap cleanup EXIT

# ---- leg1: 初始安装（dpkg-deb -x 前缀隔离，dpkg -i 代理） ----
note "leg1: install v$VERSION_V1 (dpkg-deb -x prefix isolation)"
mkdir -p "$ROOT/releases"
dpkg-deb -x "$DEB_V1" "$ROOT/releases/$VERSION_V1"
[ -x "$ROOT/releases/$VERSION_V1/usr/bin/zero-browser" ] \
    && ok "leg1 deb extracted, zero-browser executable" \
    || { bad "leg1 zero-browser missing after extract"; exit 1; }
MISSING=0
for b in zero-renderer zero-compositor zero-image-decoder; do
    [ -x "$ROOT/releases/$VERSION_V1/usr/bin/$b" ] || MISSING=1
done
[ "$MISSING" = "0" ] && ok "leg1 payload: renderer/compositor/image-decoder present" \
    || bad "leg1 payload incomplete in usr/bin"
ls "$ROOT/releases/$VERSION_V1/usr/share/applications/"*.desktop >/dev/null 2>&1 \
    && ok "leg1 desktop entry present" || bad "leg1 desktop entry missing"
ln -s "releases/$VERSION_V1" "$ROOT/current"
[ "$(readlink "$ROOT/current")" = "releases/$VERSION_V1" ] \
    && ok "leg1 current -> releases/$VERSION_V1 (relative)" \
    || bad "leg1 current link wrong: $(readlink "$ROOT/current")"

# ---- leg2: 版本追溯（IF-1） ----
note "leg2: --version traceability from installed prefix"
VER_OUT="$("$BROWSER" --version 2>/dev/null)"
[ "$VER_OUT" = "ZeroBrowser $VERSION_V1 (commit $COMMIT)" ] \
    && ok "leg2 installed --version: $VER_OUT" \
    || bad "leg2 version line mismatch: '$VER_OUT' (want ZeroBrowser $VERSION_V1 (commit $COMMIT))"

# ---- leg3: 更新应用（DC-2 串测面；无 env 覆盖，默认安装根推导） ----
note "leg3: apply update to v$VERSION_V2 via installed binary (default install-root derivation)"
SRC="$WORK/update-source"
rm -rf "$SRC"; mkdir -p "$SRC"
cp "$DEB_V2" "$SRC/$(basename "$DEB_V2")"
SHA="$(sha256sum "$SRC/$(basename "$DEB_V2")" | cut -d' ' -f1)"
printf '{"schema_version":1,"version":"%s","artifact":"%s","sha256":"%s"}' \
    "$VERSION_V2" "$(basename "$DEB_V2")" "$SHA" > "$SRC/manifest.json"
UP_OUT="$(env -u ZERO_UPDATE_INSTALL_ROOT "$BROWSER" --check-updates="$SRC" 2>/dev/null)" \
    || { bad "leg3 check-updates exited nonzero: $UP_OUT"; }
[ "$UP_OUT" = "UPDATE_APPLIED $VERSION_V2 (previous $VERSION_V1)" ] \
    && ok "leg3 update applied: $UP_OUT" \
    || bad "leg3 unexpected output: $UP_OUT"
[ "$(readlink "$ROOT/current")" = "releases/$VERSION_V2" ] \
    && ok "leg3 current flipped to releases/$VERSION_V2" \
    || bad "leg3 current: $(readlink "$ROOT/current")"
[ "$(readlink "$ROOT/rollback")" = "releases/$VERSION_V1" ] \
    && ok "leg3 rollback -> releases/$VERSION_V1" \
    || bad "leg3 rollback: $(readlink "$ROOT/rollback" 2>/dev/null || echo missing)"
VER_OUT="$("$BROWSER" --version 2>/dev/null)"
[ "$VER_OUT" = "ZeroBrowser $VERSION_V2 (commit $COMMIT)" ] \
    && ok "leg3 post-update --version: $VER_OUT (commit unchanged)" \
    || bad "leg3 post-update version mismatch: '$VER_OUT'"
[ ! -e "$ROOT/staging" ] && ok "leg3 no staging residue" || bad "leg3 staging residue: $(ls "$ROOT/staging")"

# ---- leg4: 崩溃报告（DC-3 串测面：报告出自安装内 v2） ----
note "leg4: crash report from installed v$VERSION_V2 (renderer SIGKILL)"
ALOG="$WORK/leg4-browser.log"
BPID=$(start_browser "zero://newtab" "$ALOG")
R1=""
for _ in $(seq 1 60); do
    R1=$(live_renderer "$BPID" || true)
    [ -n "$R1" ] && break
    sleep 0.5
done
[ -n "$R1" ] || { bad "leg4 no live renderer child"; exit 1; }
ok "leg4 installed browser up (pid=$BPID, renderer pid=$R1)"
kill -9 "$R1"
REPORT1=""
for _ in $(seq 1 20); do
    REPORT1=$(ls "$BREPORTS"/crash-*-renderer.json 2>/dev/null | head -1 || true)
    [ -n "$REPORT1" ] && break
    sleep 0.5
done
[ -n "$REPORT1" ] && ok "leg4 renderer report landed: $(basename "$REPORT1")" \
    || { bad "leg4 no renderer report appeared"; exit 1; }
python3 - "$REPORT1" "$VERSION_V2" "$COMMIT" "$WORK" <<'PYEOF' \
    && ok "leg4 report fields: schema/role/version=v2/commit + privacy clean" \
    || bad "leg4 report field or privacy assertion failed"
import json, sys
path, version, commit, work = sys.argv[1:5]
r = json.load(open(path))
assert r["schema_version"] == 1, r
assert r["role"] == "renderer", r
assert r["version"] == version, (r["version"], version)
assert r["git_commit"] == commit, (r["git_commit"], commit)
raw = open(path, encoding="utf-8").read()
for pat in ("http", "file://", "/home/", work):
    assert pat not in raw, f"privacy leak {pat!r}"
PYEOF
COUNT=0
for _ in $(seq 1 20); do
    COUNT=$(crash_count)
    [ "$COUNT" = "1" ] && break
    sleep 0.5
done
[ "$COUNT" = "1" ] && ok "leg4 crash #1: session crash_count=1 persisted" \
    || bad "leg4 crash #1: crash_count=$COUNT (want 1)"

# ---- leg5: 崩溃循环守卫 + SIGINT + 恢复（DC-4 串测面） ----
note "leg5: crash-loop guard across restart (threshold 2 -> restore skip -> window reset)"
R2=$(live_renderer "$BPID" || true)
[ -n "$R2" ] || { bad "leg5 renderer did not respawn after crash #1"; exit 1; }
# 报告文件名含 unix 秒：等墙钟严格跨过报告 #1 的秒再补第二刀，避免同名覆盖。
REPORT1_STAMP=$(basename "$REPORT1" | sed 's/crash-//; s/-renderer\.json//')
while [ "$(date +%s)" -le "$REPORT1_STAMP" ]; do sleep 0.2; done
kill -9 "$R2"
COUNT=0
for _ in $(seq 1 20); do
    COUNT=$(crash_count)
    [ "$COUNT" = "2" ] && break
    sleep 0.5
done
[ "$COUNT" = "2" ] && ok "leg5 crash #2: session crash_count=2 persisted" \
    || bad "leg5 crash #2: crash_count=$COUNT (want 2)"
graceful_stop "$BPID"
CLEAN=$(python3 -c "
import json
try:
    print(json.load(open('$SESSION_JSON'))['clean_exit'])
except Exception:
    print(False)
" 2>/dev/null || echo False)
[ "$CLEAN" = "True" ] && ok "leg5 SIGINT: clean_exit=true" \
    || bad "leg5 clean_exit=$CLEAN (want True)"
note "leg5 restarting installed browser (auto-restore path)"
ALOG2="$WORK/leg5-browser.log"
BPID2=$(start_browser "zero://newtab" "$ALOG2")
SKIP=0
for _ in $(seq 1 20); do
    grep -q "skipping auto-load of crash-loop tab" "$ALOG2" 2>/dev/null && { SKIP=1; break; }
    sleep 0.5
done
[ "$SKIP" = "1" ] && ok "leg5 restore: crash-loop tab auto-load skipped" \
    || bad "leg5 restore: no skip log in leg5-browser.log"
RESET=0
for _ in $(seq 1 36); do
    COUNT=$(crash_count)
    [ "$COUNT" = "0" ] && { RESET=1; break; }
    sleep 0.5
done
[ "$RESET" = "1" ] && ok "leg5 observation window: crash_count reset to 0 after ~10s" \
    || bad "leg5 observation window: crash_count=$COUNT still non-zero after 18s"
graceful_stop "$BPID2"
CLEAN=$(python3 -c "
import json
try:
    print(json.load(open('$SESSION_JSON'))['clean_exit'])
except Exception:
    print(False)
" 2>/dev/null || echo False)
[ "$CLEAN" = "True" ] && ok "leg5 second SIGINT: clean_exit=true (repeatable)" \
    || bad "leg5 second clean_exit=$CLEAN (want True)"

# ---- leg6: 反馈导出（FR-5/IF-4 串测面：env 绑定安装内 v2） ----
note "leg6: feedback export from installed v$VERSION_V2 (consent / IF-4 layout)"
REPORT_COUNT=$(ls "$BREPORTS"/crash-*-renderer.json 2>/dev/null | wc -l)
LIST_OUT="$WORK/leg6-list.txt"
XDG_CONFIG_HOME="$CFG" "$BROWSER" --list-crash-reports > "$LIST_OUT" 2>"$WORK/leg6-list.err"
LINES=$(grep -c "^REPORT crash-.*-renderer\.json renderer [0-9]* [0-9]*$" "$LIST_OUT" || true)
[ "$LINES" = "$REPORT_COUNT" ] && [ "$REPORT_COUNT" -ge 2 ] \
    && ok "leg6 --list-crash-reports: $LINES well-formed lines" \
    || bad "leg6 list malformed: $(cat "$LIST_OUT")"
EXPORT="$WORK/export"
XDG_CONFIG_HOME="$CFG" "$BROWSER" \
    --export-feedback="$EXPORT" --include-session-summary > "$WORK/leg6-export.out" 2>&1
PACKAGE=$(grep -o "$EXPORT/zeroweb-feedback-[^ ]*" "$WORK/leg6-export.out" | head -1)
[ -n "$PACKAGE" ] && [ -d "$PACKAGE" ] \
    && ok "leg6 export: $(basename "$PACKAGE")" \
    || { bad "leg6 export failed: $(cat "$WORK/leg6-export.out")"; exit 1; }
python3 - "$PACKAGE" "$BREPORTS" "$VERSION_V2" "$COMMIT" "$WORK" <<'PYEOF' \
    && ok "leg6 IF-4: reports byte-identical + env(version=v2/commit) + summary w/o URLs + privacy clean" \
    || bad "leg6 package structure/environment/privacy assertion failed"
import json, sys, os
package, reports_dir, version, commit, work = sys.argv[1:6]
src = sorted(os.listdir(reports_dir))
packed = sorted(os.listdir(os.path.join(package, "reports")))
assert len(src) >= 2 and packed == src, (packed, src)
for name in packed:
    assert open(os.path.join(reports_dir, name), "rb").read() == \
           open(os.path.join(package, "reports", name), "rb").read(), name
env = json.load(open(os.path.join(package, "environment.json")))
assert env["schema_version"] == 1, env
assert env["version"] == version, (env["version"], version)
assert env["git_commit"] == commit, (env["git_commit"], commit)
assert env["os"] and env["arch"] and env["generated_at"] > 0, env
assert [r["file_name"] for r in env["reports"]] == src, env["reports"]
summary = json.load(open(os.path.join(package, "session-summary.json")))
assert summary["tab_count"] == 1, summary
assert "urls" not in summary, "URL 不得默认进入摘要"
for root, _, files in os.walk(package):
    for f in files:
        raw = open(os.path.join(root, f), "rb").read().decode("utf-8", "replace")
        for pat in ("http", "file://", "/home/", work):
            assert pat not in raw, f"privacy leak {pat!r} in {f}"
PYEOF
XDG_CONFIG_HOME="$CFG" "$BROWSER" \
    --export-feedback="$EXPORT" > "$WORK/leg6-dup.out" 2>&1 \
    && bad "leg6 duplicate export should fail" || true
grep -q "already exists" "$WORK/leg6-dup.out" \
    && ok "leg6 duplicate export rejected without clobber" \
    || bad "leg6 duplicate rejection missing: $(cat "$WORK/leg6-dup.out")"

# ---- leg7: 卸载完整性（DC-1 卸载机制面：dpkg -r 只删包文件、不动用户数据） ----
note "leg7: uninstall (remove install root) keeps user data intact"
rm -rf "$ROOT"
[ ! -e "$ROOT/current/usr/bin/zero-browser" ] \
    && ok "leg7 install face removed (binary path gone)" \
    || bad "leg7 installed binary still present after uninstall"
python3 - "$SESSION_JSON" <<'PYEOF' \
    && ok "leg7 user session intact (clean_exit=true, 1 tab)" \
    || bad "leg7 session.json damaged by uninstall"
import json, sys
s = json.load(open(sys.argv[1]))
assert s["clean_exit"] is True, s["clean_exit"]
assert len(s["tabs"]) == 1, s["tabs"]
PYEOF
[ "$(ls "$BREPORTS"/crash-*-renderer.json 2>/dev/null | wc -l)" = "$REPORT_COUNT" ] \
    && ok "leg7 crash reports survive uninstall ($REPORT_COUNT files)" \
    || bad "leg7 crash reports lost in uninstall"
[ -f "$PACKAGE/environment.json" ] && [ -d "$PACKAGE/reports" ] \
    && ok "leg7 exported package survives uninstall" \
    || bad "leg7 export package lost in uninstall"

# ---- leg8: 清理审计（DC-5 机制面） ----
note "leg8: cleanup audit (no residue processes / staging / real-config writes / repo dirt)"
[ ! -e "$ROOT/staging" ] && ok "leg8 no staging residue" || bad "leg8 staging residue: $(ls "$ROOT/staging")"
LEAK=""
for pid in $(pgrep -x zero-browser 2>/dev/null || true; pgrep -x zero-renderer 2>/dev/null || true); do
    tgt=$(readlink "/proc/$pid/exe" 2>/dev/null || true)
    case "$tgt" in
        "$WORK"/*|"$ROOT"/*) LEAK="$LEAK $pid($tgt)" ;;
    esac
done
[ -z "$LEAK" ] && ok "leg8 no residual browser/renderer processes from this run" \
    || bad "leg8 residual processes:$LEAK"
if [ -d "$HOME/.config/zeroweb" ]; then
    NEWER=$(find "$HOME/.config/zeroweb" -newer "$WORK/.chain-start" 2>/dev/null | wc -l)
    [ "$NEWER" = "0" ] && ok "leg8 real ~/.config/zeroweb untouched (0 files newer than chain start)" \
        || bad "leg8 real config got $NEWER new writes: $(find "$HOME/.config/zeroweb" -newer "$WORK/.chain-start" | head -5)"
else
    ok "leg8 real ~/.config/zeroweb absent (nothing to leak into)"
fi
DIRTY=$(git -C "$REPO" diff --name-only | wc -l)
[ "$DIRTY" = "0" ] && ok "leg8 repo tracked tree clean (no tracked file modified)" \
    || bad "leg8 tracked files modified: $(git -C "$REPO" diff --name-only | head -5)"

# ---- 汇总 ----
echo "[M4] summary: PASS=$PASS FAIL=$FAIL"
[ "$FAIL" = "0" ]
