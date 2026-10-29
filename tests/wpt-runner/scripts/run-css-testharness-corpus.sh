#!/usr/bin/env bash
# css-testharness-compat goal（docs/goal/css-testharness-compat.md M0 / DC-1）：
# css/values + css/selectors 语料分片运行器（M3 扩面目录按 CORPUS 参数复用）。
#
# 为何分片：runner 单进程跑全量（css-values ~600 案 / selectors ~700 案）时 RSS
# 线性累积（每案独立 WebView/V8 分配器驻留，~35MB/案——run-css-variables-corpus.sh
# 同根因），test-guard 8G 单进程上限必然杀树；单案 RSS <300MB。每案独立 runner
# 进程使内存天然有界，代价是每案 ~1s 进程启动开销——基线/回归记录场景可接受，
# 速度敏感场景待 M1 内存治理（契约 DC-2）。
#
# 用法：run-css-testharness-corpus.sh <corpus> [runner]
#   <corpus>  wpt-data/css/ 下的目录名（css-values | selectors）
# 退出码：0 = 至少跑了 1 案（记录模式，不设通过门禁——css-variables corpus
# 脚本先例：基线首跑通过率只记录）；2 = runner/语料缺失（基础设施错误）。
# 筛减规则单点在 runner 侧 css_testharness_case_skipped（fetch 脚本头注释同域
# 双保险），此处不复制规则——每案带相对路径 filter 透传，runner 谓词照常生效。
#
# 产物：文本 transcript（CSS_CORPUS_TRANSCRIPT，默认 /tmp）+ 汇总 JSON
# （CSS_CORPUS_JSON，设置时写出——逐案四态计数，供 evidence 与 M1 分簇消费）。
#
# 为何每案另有外部墙钟（CASE_WALL_S，默认 90s）：runner 内部 30s per-case 超时
# 在同步脚本死循环形态下拦不住（script timeout 只在 run_page_scripts 返回后
# tick——首跑实证 selectors/invalidation/has-complexity.html 100% CPU 自旋 >7min），
# 分片脚本按外部 timeout 兜底杀单案（exit 124 记为 Timeout 行，账面不缺案），
# 不连累整轮。

set -uo pipefail

CORPUS="${1:-}"
RUNNER="${2:-./target/release/zero-wpt-runner}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
DATA_DIR="${REPO_ROOT}/tests/wpt-runner/wpt-data/css/${CORPUS}"
CASE_WALL_S="${CASE_WALL_S:-90}"
# 断点续跑（CSS_CORPUS_RESUME=1）：transcript 已有 `==== <rel>` 的案跳过——
# 大目录全量超出单轮 test-guard 墙钟时的续跑通道（css-values 首跑实证 3600s
# 只走完 290/595 案）。分片（CSS_CORPUS_SHARD=k/N，k=0..N-1）：按列表序取模
# 交错分片，多 test-guard 实例并行消费同一名册（各写各的 transcript/JSON），
# 墙钟线性下降；合并口径在 evidence 侧按 transcript 相加。
RESUME="${CSS_CORPUS_RESUME:-0}"
SHARD="${CSS_CORPUS_SHARD:-}"

case "${CORPUS}" in
  css-values) SUBCMD=testharness-css-values ;;
  selectors) SUBCMD=testharness-selectors ;;
  *) echo "usage: $0 <css-values|selectors> [runner]" >&2; exit 2 ;;
esac

if [[ ! -x "${RUNNER}" || ! -d "${DATA_DIR}" ]]; then
  echo "css ${CORPUS} corpus: runner or data missing" >&2
  exit 2
fi

transcript="${CSS_CORPUS_TRANSCRIPT:-/tmp/css-${CORPUS}-corpus.log}"
# 续跑模式追加写（不截断既有 transcript——done 集与已完成案都靠它）；否则清空重跑。
[[ "${RESUME}" == "1" ]] || : > "${transcript}"

ran=0 pass=0 fail=0 timeout=0 unsupported=0 skipped_patho=0
done_cases=""
if [[ "${RESUME}" == "1" ]]; then
  done_cases="$(grep '^==== ' "${transcript}" 2>/dev/null | sed 's/^==== //')"
fi
shard_k=0 shard_n=1
if [[ "${SHARD}" =~ ^[0-9]+/[0-9]+$ ]]; then
  shard_k="${SHARD%/*}"; shard_n="${SHARD#*/}"
fi
idx=0
# process substitution 而非管道——管道 while 在子 shell 计数，退出后归零。
while read -r f; do
  rel="css/${CORPUS}/${f#"${DATA_DIR}"/}"
  name="$(basename "${f}")"
  # 病态案名级快筛（css-variables corpus 脚本先例：run-css-variables-corpus.sh
  # 跳 variable-exponential-blowup.html）：if-conditionals.html 单案 Oilpan
  # CagedHeap OOM 实证（fuzz-repro-guard 口径复现——CSS if() 条件面引擎半支持
  # 状态指数膨胀，4G 单进程守卫杀树连累整轮）。重入条件 = if() 求值内存语义
  # 修复；内容级筛减仍由 runner 侧谓词单点负责，此处不复制。
  if [[ "${name}" == "if-conditionals.html" ]]; then
    skipped_patho=$((skipped_patho + 1))
    continue
  fi
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
    # 外部墙钟兜底杀——内部 30s 超时未生效的同步死循环形态，账面记 Timeout 行。
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

echo "css ${CORPUS} corpus: cases=${ran} pass=${pass} fail=${fail} timeout=${timeout} unsupported=${unsupported} skipped-patho=${skipped_patho}"
echo "transcript: ${transcript}"

if [[ -n "${CSS_CORPUS_JSON:-}" ]]; then
  python3 - "${transcript}" "${CSS_CORPUS_JSON}" "${CORPUS}" <<'PYEOF'
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
summary = {"corpus": f"css/{corpus}", "totals": totals, "cases": cases}
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
