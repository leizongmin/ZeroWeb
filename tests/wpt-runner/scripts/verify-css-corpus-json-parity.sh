#!/usr/bin/env bash
# css-testharness-compat M1 / DC-2 等价判据复验：分片通道（每案独立进程）合并结果
# ≡ 单进程 `--json` 输出，按 (case, 子测名→status) 集合恒等（perf/wpt-runner
# testharness-speed A/B 先例口径）。M3 扩面目录按 CORPUS/FILTER 换子集复用。
#
# 用法：verify-css-corpus-json-parity.sh <corpus> <filter-substr> [runner]
#   <corpus>        wpt-data/css/ 下的目录名（css-values | selectors）
#   <filter-substr> 子集过滤（选无外部墙钟 Timeout 案的子集——两通道的 Timeout
#                   语义固有差异：分片=外部 90s 杀+合成条目，单进程=内部超时，
#                   不在恒等判据内）
# 退出码：0 = 集合恒等；1 = 有差异；2 = 基础设施错误（runner/语料缺失、空结果）。
set -uo pipefail

CORPUS="${1:-}"
FILTER="${2:-}"
RUNNER="${3:-./target/release/zero-wpt-runner}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

case "${CORPUS}" in
  css-values) SUBCMD=testharness-css-values ;;
  selectors) SUBCMD=testharness-selectors ;;
  *) echo "usage: $0 <css-values|selectors> <filter-substr> [runner]" >&2; exit 2 ;;
esac

WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT

# A 臂：单进程一次跑整个子集（RSS 归因实证无线性累积，子集量级守卫内安全）。
"${RUNNER}" "${SUBCMD}" --json "${FILTER}" > "${WORK}/single.json" 2>"${WORK}/single.err"

# B 臂：分片通道同子集（每案独立进程 + JSON 合并）。
CSS_CORPUS_JSON_DIR="${WORK}/shards" CSS_CORPUS_ONLY_SUBSTR="${FILTER}" \
  CSS_CORPUS_TRANSCRIPT="${WORK}/shard-transcript.log" \
  bash "${SCRIPT_DIR}/run-css-testharness-corpus.sh" "${CORPUS}" "${RUNNER}" \
  > "${WORK}/shard-run.out" 2>&1

python3 - "${WORK}/single.json" "${WORK}/shards" "${CORPUS}" "${FILTER}" <<'PYEOF'
import glob, json, sys

single_path, shards_dir, corpus, filter_substr = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]

def load(path):
    with open(path, encoding="utf-8") as fh:
        data = json.load(fh)
    # (case, 子测名→status) 集合形态；message 不入判据（分片 Timeout 合成条目的
    # message 与单进程内部超时文案不同，属通道固有差异）。
    return {case: {sub["name"]: sub["status"] for sub in subtests} for case, subtests in data}

single = load(single_path)
shard_files = sorted(glob.glob(f"{shards_dir}/*.json"))
if not shard_files:
    sys.exit(f"no merged shard json under {shards_dir}")
sharded = {}
for path in shard_files:
    for case, subtests in load(path).items():
        sharded.setdefault(case, {}).update(subtests)

if not single:
    sys.exit(f"single-process run empty for filter={filter_substr!r} — check filter/corpus")
if not sharded:
    sys.exit("sharded run empty — corpus script produced no entries")

missing = sorted(set(single) - set(sharded))
extra = sorted(set(sharded) - set(single))
status_diffs = sorted(
    (case, name, single[case][name], sharded[case][name])
    for case in set(single) & set(sharded)
    for name in set(single[case]) & set(sharded[case])
    if single[case][name] != sharded[case][name]
)
subtest_only_a = sorted(
    (case, name) for case in set(single) & set(sharded)
    for name in set(single[case]) - set(sharded[case])
)
subtest_only_b = sorted(
    (case, name) for case in set(single) & set(sharded)
    for name in set(sharded[case]) - set(single[case])
)

ok = not (missing or extra or status_diffs or subtest_only_a or subtest_only_b)
print(f"parity[{corpus} filter={filter_substr!r}]: cases single={len(single)} sharded={len(sharded)} -> "
      f"{'IDENTICAL' if ok else 'DIFF'}")
for label, rows in (("missing-in-shards", missing), ("extra-in-shards", extra),
                    ("subtest-only-in-single", subtest_only_a), ("subtest-only-in-shards", subtest_only_b)):
    for row in rows[:10]:
        print(f"  {label}: {row}")
    if len(rows) > 10:
        print(f"  {label}: ... {len(rows)} total")
for case, name, sa, sb in status_diffs[:10]:
    print(f"  status-diff: {case} :: {name}: single={sa} sharded={sb}")
if len(status_diffs) > 10:
    print(f"  status-diff: ... {len(status_diffs)} total")
sys.exit(0 if ok else 1)
PYEOF
exit $?
