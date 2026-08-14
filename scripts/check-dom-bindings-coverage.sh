#!/usr/bin/env bash
# check-dom-bindings-coverage.sh — dom_bindings 子模块独立覆盖率口径（js-dom goal M0 项 4）
#
# 背景：dom_bindings 是 zero-engine 的**子模块**（crates/engine/src/dom_bindings/），非独立 crate。
# scripts/check-coverage.sh 的 --summary-only 仅按 crate 报告，dom_bindings 被 fold 进 zero-engine，
# 无独立数字。本脚本经 cargo-llvm-cov 生成 lcov，再解析 dom_bindings/ 文件，给出子模块行覆盖率
# + 逐文件明细 + 源码/测试分离口径（DC-4：dom_bindings 覆盖率持续提升、不退化）。
#
# 用法：
#   scripts/check-dom-bindings-coverage.sh           # 默认 v8 feature（dom_bindings 测试点全在此矩阵）
#   scripts/check-dom-bindings-coverage.sh --no-default-features --features quickjs
#   scripts/check-dom-bindings-coverage.sh --json     # 机器可读 JSON（供 evidence 持久化）
#
# 环境前提：cargo-llvm-cov + llvm-tools-preview（缺则提示安装并退出 0，不阻断 CI）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

OUTPUT_JSON=0
# 解析透传给 cargo 的 feature 参数 + 本脚本自有 flag
PASS_ARGS=()
while [ $# -gt 0 ]; do
  case "$1" in
    --json) OUTPUT_JSON=1; shift;;
    *) PASS_ARGS+=("$1"); shift;;
  esac
done
# 默认（无透传 feature）时显式锁 v8——dom_bindings 测试点全在 v8 矩阵（tests_* 均 #[cfg(feature="v8")]）
if [ ${#PASS_ARGS[@]} -eq 0 ]; then
  PASS_ARGS=("--features" "v8")
fi

# 工具可用性（缺则提示，不报错退出——覆盖率口径非 CI 硬门禁）
if ! command -v cargo-llvm-cov &>/dev/null; then
  echo "dom_bindings coverage: cargo-llvm-cov 未安装（跳过）。安装：cargo install cargo-llvm-cov && rustup component add llvm-tools-preview" >&2
  exit 0
fi
if ! rustup component list --installed 2>/dev/null | grep -q "llvm-tools"; then
  echo "dom_bindings coverage: llvm-tools-preview 未安装（跳过）。安装：rustup component add llvm-tools-preview" >&2
  exit 0
fi

TMP_LCOV="$(mktemp -t dom_bindings_cov.XXXXXX.lcov)"
trap 'rm -f "$TMP_LCOV"' EXIT

# 生成 lcov（--lib 聚焦库测试；test-threads=4 平衡速度与 render-foundation SIGSEGV 风险，
# 这里只测 zero-engine 不含 GPU 测试，可放宽）
# shellcheck disable=SC2086
cargo llvm-cov -p zero-engine "${PASS_ARGS[@]}" --lib --lcov --output-path "$TMP_LCOV" \
  -- --test-threads=4 > /dev/null 2>&1 || {
    echo "dom_bindings coverage: cargo-llvm-cov 运行失败（见上方输出）" >&2
    exit 0
  }

# 解析 lcov → dom_bindings 行覆盖率（源码/测试分离 + 逐文件）。lcov 字段：SF:<path> / DA:<line>,<count>。
python3 - "$TMP_LCOV" "$OUTPUT_JSON" <<'PYEOF'
import json, sys, os
lcov_path, output_json = sys.argv[1], int(sys.argv[2])
files = []
cur = None
for line in open(lcov_path):
    line = line.rstrip("\n")
    if line.startswith("SF:"):
        cur = {"path": line[3:], "hit": 0, "found": 0}
    elif line == "end_of_record":
        if cur:
            files.append(cur); cur = None
    elif line.startswith("DA:") and cur is not None:
        cnt = line[3:].split(",")[1]
        cur["found"] += 1
        if cnt != "0":
            cur["hit"] += 1

def is_db(p): return "dom_bindings" in p
def is_test(p): return "/tests_" in p
def summ(pred):
    h = f = n = 0
    for c in files:
        if pred(c["path"]):
            h += c["hit"]; f += c["found"]; n += 1
    return {"files": n, "hit": h, "found": f, "rate": round(100.0 * h / f, 2) if f else 0.0}

db_files = [c for c in files if is_db(c["path"])]
per_file = []
for c in db_files:
    short = c["path"].split("dom_bindings")[-1]
    per_file.append({
        "file": short,
        "kind": "test" if is_test(c["path"]) else "src",
        "hit": c["hit"], "found": c["found"],
        "rate": round(100.0 * c["hit"] / c["found"], 2) if c["found"] else 0.0,
    })
per_file.sort(key=lambda r: (r["kind"], -r["rate"]))

result = {
    "module": "dom_bindings (crates/engine/src/dom_bindings/)",
    "source": summ(lambda p: is_db(p) and not is_test(p)),
    "test": summ(lambda p: is_db(p) and is_test(p)),
    "all": summ(is_db),
    "per_file": per_file,
}

if output_json:
    print(json.dumps(result, indent=2))
else:
    s, t, a = result["source"], result["test"], result["all"]
    print("=== dom_bindings 子模块覆盖率（独立口径，js-dom goal M0 项 4）===")
    print(f"  源码行覆盖（src，{s['files']} 文件）：{s['hit']}/{s['found']} = {s['rate']}%")
    print(f"  测试文件（tests_*，{t['files']} 文件）：{t['hit']}/{t['found']} = {t['rate']}%")
    print(f"  全部 dom_bindings（{a['files']} 文件）：{a['hit']}/{a['found']} = {a['rate']}%")
    print("")
    print("  逐文件明细（源码）：")
    for r in per_file:
        if r["kind"] == "src":
            print(f"    {r['rate']:5.1f}%  {r['hit']:4d}/{r['found']:<4d}  {r['file']}")
    src_low = [r for r in per_file if r["kind"] == "src" and r["rate"] < 90.0]
    if src_low:
        print("")
        print(f"  ⚠ <90% 源码文件（提升候选）：{', '.join(r['file'] for r in src_low)}")
PYEOF
