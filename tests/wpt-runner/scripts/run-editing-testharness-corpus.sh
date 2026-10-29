#!/usr/bin/env bash
# editing-compat goal（docs/goal/editing-compat.md M1 / DC-2）：
# editing/run + editing/other 语料分片运行器（run-corpus 参数二选一）。
#
# 为何分片：runner 单进程跑全量时 RSS 线性累积（每案独立 WebView/V8 分配器
# 驻留，~35MB/案——run-css-testharness-corpus.sh 同根因同结论），每案独立
# runner 进程使内存天然有界，代价是每案 ~1s 进程启动开销——基线/回归记录
# 场景可接受，速度敏感场景待套件内存治理（css-testharness-compat DC-2 共享）。
#
# 用法：run-editing-testharness-corpus.sh <run|other> [runner]
# 退出码：0 = 至少跑了 1 案（记录模式，不设通过门禁——css corpus 脚本先例：
# 基线首跑通过率只记录）；2 = runner/语料缺失（基础设施错误）。
# 筛减规则单点在 runner 侧 editing_case_skipped（fetch 侧 33 号脚本头注释同域
# 双保险），此处不复制规则——每案带相对路径 filter 透传，runner 谓词照常生效。
#
# 产物：文本 transcript（EDITING_CORPUS_TRANSCRIPT，默认 /tmp）+ 汇总 JSON
# （EDITING_CORPUS_JSON，设置时写出——逐案四态计数，供 evidence 与 M2 分簇消费）。
# 断点续跑（EDITING_CORPUS_RESUME=1）与分片（EDITING_CORPUS_SHARD=k/N）语义
# 同 run-css-testharness-corpus.sh；分片实例并行消费同一名册时各写各的
# transcript/JSON，合并口径在 evidence 侧按 transcript 相加。
# editing-compat M2 首轮实测两条操作坑（2026-10-30）：
# 1. 分片 k **从 0 起**（`idx % shard_n != shard_k`——k=N 恒不命中，0 案退出）；
# 2. 并行分片**必须**各设 EDITING_CORPUS_TRANSCRIPT——JSON 汇总从 transcript
#    解析（共享路径时后启实例启动即 `: > transcript` 截断，先启实例的账丢失、
#    双方 JSON 互相掺案）。
#
# 为何每案另有外部墙钟（CASE_WALL_S，run 通道建议 ≥150）：runner 内部
# per-case 超时在同步脚本死循环形态下拦不住（css selectors 首跑实证），
# 外部 timeout 兜底杀单案（rc 124/137 记 Timeout 行，账面不缺案）。

set -uo pipefail

CORPUS="${1:-}"
RUNNER="${2:-./target/release/zero-wpt-runner}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
DATA_DIR="${REPO_ROOT}/tests/wpt-runner/wpt-data/editing/${CORPUS}"
CASE_WALL_S="${CASE_WALL_S:-120}"
RESUME="${EDITING_CORPUS_RESUME:-0}"
SHARD="${EDITING_CORPUS_SHARD:-}"

case "${CORPUS}" in
  run) SUBCMD=testharness-editing-run ;;
  other) SUBCMD=testharness-editing-other ;;
  *) echo "usage: $0 <run|other> [runner]" >&2; exit 2 ;;
esac

if [[ ! -x "${RUNNER}" || ! -d "${DATA_DIR}" ]]; then
  echo "editing ${CORPUS} corpus: runner or data missing" >&2
  exit 2
fi

transcript="${EDITING_CORPUS_TRANSCRIPT:-/tmp/editing-${CORPUS}-corpus.log}"
[[ "${RESUME}" == "1" ]] || : > "${transcript}"

ran=0 pass=0 fail=0 timeout=0 unsupported=0
done_cases=""
if [[ "${RESUME}" == "1" ]]; then
  done_cases="$(grep '^==== ' "${transcript}" 2>/dev/null | sed 's/^==== //')"
fi
shard_k=0 shard_n=1
if [[ "${SHARD}" =~ ^[0-9]+/[0-9]+$ ]]; then
  shard_k="${SHARD%/*}"; shard_n="${SHARD#*/}"
fi
idx=0
while read -r f; do
  rel="editing/${CORPUS}/${f#"${DATA_DIR}"/}"
  if [[ "${RESUME}" == "1" && "${done_cases}" == *"${rel}"* ]]; then
    continue
  fi
  if [[ "${shard_n}" -gt 1 ]]; then
    if (( idx % shard_n != shard_k )); then
      idx=$((idx + 1))
      continue
    fi
  fi
  idx=$((idx + 1))
  out="$(timeout --signal=KILL "${CASE_WALL_S}" "${RUNNER}" "${SUBCMD}" "${rel}" 2>&1)"
  rc=$?
  echo "==== ${rel}" >> "${transcript}"
  echo "${out}" >> "${transcript}"
  if [[ "${rc}" -eq 124 || "${rc}" -eq 137 ]]; then
    echo "Timeout ${rel} :: external wall-clock kill (${CASE_WALL_S}s, rc=${rc})" >> "${transcript}"
    t=1; p=0; fl=0; un=0
  else
    p="$(grep -c '^Pass ' <<<"${out}" || true)"
    fl="$(grep -c '^Fail ' <<<"${out}" || true)"
    t="$(grep -c '^Timeout ' <<<"${out}" || true)"
    un="$(grep -c '^Unsupported ' <<<"${out}" || true)"
  fi
  if [[ "${p}" -gt 0 || "${fl}" -gt 0 || "${t}" -gt 0 || "${un}" -gt 0 ]]; then
    ran=$((ran + 1))
  fi
  pass=$((pass + p)); fail=$((fail + fl)); timeout=$((timeout + t)); unsupported=$((unsupported + un))
done < <(find "${DATA_DIR}" \( -name '*.html' -o -name '*.htm' \) | sort)

echo "editing ${CORPUS} corpus: cases=${ran} pass=${pass} fail=${fail} timeout=${timeout} unsupported=${unsupported}"
echo "transcript: ${transcript}"

if [[ -n "${EDITING_CORPUS_JSON:-}" ]]; then
  python3 - "${transcript}" "${EDITING_CORPUS_JSON}" "editing/${CORPUS}" <<'PYEOF'
import json, re, sys

transcript, out_path, corpus = sys.argv[1], sys.argv[2], sys.argv[3]
line_re = re.compile(r"^(Pass|Fail|Timeout|Unsupported) (\S+) :: ")
totals, cases, current = {}, [], None
with open(transcript, encoding="utf-8", errors="replace") as fh:
    for line in fh:
        case = re.match(r"^==== (\S+)$", line.rstrip("\n"))
        if case:
            current = {"case": case.group(1), "pass": 0, "fail": 0, "timeout": 0, "unsupported": 0}
            cases.append(current)
            continue
        m = line_re.match(line)
        if m and current is not None:
            key = {"Pass": "pass", "Fail": "fail", "Timeout": "timeout", "Unsupported": "unsupported"}[m.group(1)]
            current[key] += 1
            totals[key] = totals.get(key, 0) + 1
summary = {"corpus": corpus, "totals": totals, "cases": cases}
with open(out_path, "w", encoding="utf-8") as fh:
    json.dump(summary, fh, ensure_ascii=False, indent=1)
    fh.write("\n")
print(f"json summary: {out_path} ({len(cases)} cases)")
PYEOF
fi

if [[ "${ran}" -eq 0 ]]; then
  exit 2
fi
exit 0
