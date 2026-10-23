#!/bin/sh
# DC-2 验收：更新校验（损坏/错误版本）/ 下载与替换阶段中断 / 原子翻转与回滚。
#
# 用法: run-dc2-acceptance.sh <repo-root> <work-dir> <deb-v22> <deb-v23>
#   deb-v22: 当前版本包（装为初始安装，版本须为 26.10.22）
#   deb-v23: 更新包（版本须为 26.10.23）
# 全程隔离：安装根/更新源均在 work-dir 内，不触系统与真实 profile。
# 输出逐腿 PASS/FAIL；任一 FAIL 退出码 1。
set -eu

REPO="$1"; WORK="$2"; DEB22="$3"; DEB23="$4"
PASS=0; FAIL=0

note() { echo "[DC2] $*"; }
ok()   { PASS=$((PASS+1)); echo "[DC2] PASS: $*"; }
bad()  { FAIL=$((FAIL+1)); echo "[DC2] FAIL: $*"; }

sha_of() { sha256sum "$1" | cut -d' ' -f1; }

# fresh_root <dir>: 在 dir 建初始安装（releases/26.10.22 + current 链接）。
fresh_root() {
    rm -rf "$1"; mkdir -p "$1/releases"
    dpkg-deb -x "$DEB22" "$1/releases/26.10.22"
    ln -s releases/26.10.22 "$1/current"
}

# make_source <dir> <version-label> <artifact-file> [sha256-override]
make_source() {
    rm -rf "$1"; mkdir -p "$1"
    cp "$3" "$1/$(basename "$3")"
    if [ $# -ge 4 ]; then sha="$4"; else sha="$(sha_of "$1/$(basename "$3")")"; fi
    printf '{"schema_version":1,"version":"%s","artifact":"%s","sha256":"%s"}' \
        "$2" "$(basename "$3")" "$sha" > "$1/manifest.json"
}

# run_update <root> <source>: 经安装内二进制触发更新；echo 输出。
run_update() {
    ZERO_UPDATE_INSTALL_ROOT="$1" "$1/current/usr/bin/zero-browser" --check-updates="$2" 2>/dev/null
}

# assert_active <root> <expected-version>
assert_active() {
    got="$("$1/current/usr/bin/zero-browser" --version 2>/dev/null | sed 's/.*ZeroBrowser \([0-9.]*\).*/\1/')"
    [ "$got" = "$2" ] || { bad "active version: want $2 got '$got'"; return 1; }
    ok "active version = $2"
}

# ---- Leg 1: 正常更新（校验通过 → 翻转 → 健康） ----
ROOT="$WORK/leg1"; SRC="$WORK/src1"
fresh_root "$ROOT"
assert_active "$ROOT" "26.10.22"
make_source "$SRC" "26.10.23" "$DEB23"
out="$(run_update "$ROOT" "$SRC")" || { bad "leg1 update exited nonzero: $out"; }
case "$out" in
    "UPDATE_APPLIED 26.10.23 (previous 26.10.22)") ok "leg1 applied: $out" ;;
    *) bad "leg1 unexpected output: $out" ;;
esac
assert_active "$ROOT" "26.10.23"
[ "$(readlink "$ROOT/current")" = "releases/26.10.23" ] && ok "leg1 current link flipped" || bad "leg1 current link: $(readlink "$ROOT/current")"
[ "$(readlink "$ROOT/rollback")" = "releases/26.10.22" ] && ok "leg1 rollback set" || bad "leg1 rollback: $(readlink "$ROOT/rollback" 2>/dev/null || echo missing)"

# ---- Leg 1b: 显式回滚 ----
out="$(ZERO_UPDATE_INSTALL_ROOT="$ROOT" "$ROOT/current/usr/bin/zero-browser" --rollback-update 2>/dev/null)"
case "$out" in
    "UPDATE_ROLLBACK 26.10.22") ok "leg1b rollback: $out" ;;
    *) bad "leg1b unexpected: $out" ;;
esac
assert_active "$ROOT" "26.10.22"

# ---- Leg 2: 损坏包（哈希不符）拒绝，旧版可启动 ----
ROOT="$WORK/leg2"; SRC="$WORK/src2"
fresh_root "$ROOT"
make_source "$SRC" "26.10.23" "$DEB23" "0000000000000000000000000000000000000000000000000000000000000000"
out="$(run_update "$ROOT" "$SRC")" || true
case "$out" in
    "UPDATE_REJECTED HASH_MISMATCH") ok "leg2 corrupted rejected" ;;
    *) bad "leg2 unexpected: $out" ;;
esac
assert_active "$ROOT" "26.10.22"
[ ! -e "$ROOT/staging" ] && ok "leg2 staging cleaned" || bad "leg2 staging residue: $(ls "$ROOT/staging")"

# ---- Leg 3: 降级版本拒绝 ----
ROOT="$WORK/leg3"; SRC="$WORK/src3"
fresh_root "$ROOT"
make_source "$SRC" "26.10.21" "$DEB23"
out="$(run_update "$ROOT" "$SRC")" || true
case "$out" in
    "UPDATE_REJECTED DOWNGRADE") ok "leg3 downgrade rejected" ;;
    *) bad "leg3 unexpected: $out" ;;
esac
assert_active "$ROOT" "26.10.22"

# ---- Leg 4: 版本标签与包内容不符（暂存自检拒绝） ----
ROOT="$WORK/leg4"; SRC="$WORK/src4"
fresh_root "$ROOT"
make_source "$SRC" "26.10.24" "$DEB23"
out="$(run_update "$ROOT" "$SRC")" || true
case "$out" in
    "UPDATE_REJECTED STAGING_CHECK_FAILED") ok "leg4 self-check rejected" ;;
    *) bad "leg4 unexpected: $out" ;;
esac
assert_active "$ROOT" "26.10.22"

# ---- Leg 5: 复制阶段中断（SIGKILL），旧版存活，重跑成功 ----
ROOT="$WORK/leg5"; SRC="$WORK/src5"
fresh_root "$ROOT"
make_source "$SRC" "26.10.23" "$DEB23"
ZERO_UPDATE_INSTALL_ROOT="$ROOT" "$ROOT/current/usr/bin/zero-browser" --check-updates="$SRC" >/dev/null 2>&1 &
PID=$!
i=0
while [ ! -f "$ROOT/staging/$(basename "$DEB23")" ] && [ $i -lt 600 ]; do
    kill -0 $PID 2>/dev/null || break
    sleep 0.01; i=$((i+1))
done
kill -9 $PID 2>/dev/null || true
wait $PID 2>/dev/null || true
ok "leg5 killed during copy (iterations=$i)"
assert_active "$ROOT" "26.10.22"
out="$(run_update "$ROOT" "$SRC")"
case "$out" in
    "UPDATE_APPLIED 26.10.23"*) ok "leg5 retry applied" ;;
    *) bad "leg5 retry unexpected: $out" ;;
esac
assert_active "$ROOT" "26.10.23"

# ---- Leg 6: 解包阶段中断（SIGKILL），旧版存活，重跑成功 ----
ROOT="$WORK/leg6"; SRC="$WORK/src6"
fresh_root "$ROOT"
make_source "$SRC" "26.10.23" "$DEB23"
ZERO_UPDATE_INSTALL_ROOT="$ROOT" "$ROOT/current/usr/bin/zero-browser" --check-updates="$SRC" >/dev/null 2>&1 &
PID=$!
i=0
while [ ! -d "$ROOT/staging/prefix/usr" ] && [ $i -lt 1200 ]; do
    kill -0 $PID 2>/dev/null || break
    sleep 0.01; i=$((i+1))
done
kill -9 $PID 2>/dev/null || true
wait $PID 2>/dev/null || true
ok "leg6 killed during extract (iterations=$i)"
assert_active "$ROOT" "26.10.22"
out="$(run_update "$ROOT" "$SRC")"
case "$out" in
    "UPDATE_APPLIED 26.10.23"*) ok "leg6 retry applied" ;;
    *) bad "leg6 retry unexpected: $out" ;;
esac
assert_active "$ROOT" "26.10.23"

echo "[DC2] ================================"
echo "[DC2] PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
