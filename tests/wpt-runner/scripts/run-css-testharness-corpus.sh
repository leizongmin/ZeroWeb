#!/usr/bin/env bash
# css-testharness-compat goal（docs/goal/css-testharness-compat.md M0 / DC-1）：
# css/values + css/selectors + css/css-variables 语料分片运行器（M3 扩面目录按
# CORPUS 参数复用）。
#
# 为何每案独立进程：M1 归因实证单进程 RSS 无线性累积（0.06–0.82MB/案，
# 8.8GB 杀树实为病态单案 4.9GB 抬升笼水位后的平摊误读——goal evidence
# 2026-10-30-m1-rss-attribution.md），内存不是本通道的理由。真实价值是外部
# 兜底：病态内存案与原生同步死循环案（engine 侧自旋）内部超时拦不住，每案
# 独立进程 + 外部墙钟（CASE_WALL_S）保证单案失控不连累整轮；顺带获得断点
# 续跑与多分片并行。代价是每案 ~1s 进程启动开销。
#
# 用法：run-css-testharness-corpus.sh <corpus> [runner]
#   <corpus>  wpt-data/css/ 下的目录名（css-values | selectors | css-variables）
# 退出码：0 = 至少跑了 1 案（记录模式，不设通过门禁——css-variables corpus
# 脚本先例：基线首跑通过率只记录）；2 = runner/语料缺失（基础设施错误）。
# 筛减规则单点在 runner 侧 css_testharness_case_skipped（fetch 脚本头注释同域
# 双保险），此处不复制规则——每案带相对路径 filter 透传，runner 谓词照常生效。
#
# 产物：文本 transcript（CSS_CORPUS_TRANSCRIPT，默认 /tmp）+ 汇总 JSON
# （CSS_CORPUS_JSON，设置时写出——逐案四态计数，供 evidence 与 M1 分簇消费）。
#
# JSON 通道（CSS_CORPUS_JSON_DIR，css-testharness-compat M1 / DC-2）：设置时每案
# 改跑 `--json`（runner 原生 `[[case, [{name,status,message}...]]...]` 形状），块体
# 原样进 transcript，收尾合并为分片级同名数组
# `<JSON_DIR>/<corpus>[-shardKofN].json`——直接投喂 coverage 管道
# （wptfyi_gap.py merge_zw_results 按案取最完整，多分片文件可并存）。外部墙钟杀
# 的案以合成单子测 Timeout 条目入账（案面不缺账）。文本通道行为不变。
#
# 子集过滤（CSS_CORPUS_ONLY_SUBSTR）：只跑 rel 含该子串的案（等价性验证与定向
# 重跑用；病态案名级跳过优先于此过滤）。
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
JSON_DIR="${CSS_CORPUS_JSON_DIR:-}"
ONLY_SUBSTR="${CSS_CORPUS_ONLY_SUBSTR:-}"

case "${CORPUS}" in
  css-values) SUBCMD=testharness-css-values ;;
  selectors) SUBCMD=testharness-selectors ;;
  css-variables) SUBCMD=testharness-css-variables ;;
  *) echo "usage: $0 <css-values|selectors|css-variables> [runner]" >&2; exit 2 ;;
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
  # 病态案名级快筛（避免无谓进程启动；规则单点在 runner 侧谓词
  # `css_testharness_case_skipped` PATHO_CASES——本表只是同名单的快路径，
  # 两侧同源防漂移）：if-conditionals.html 单案 Oilpan CagedHeap OOM 实证
  # （fuzz-repro-guard 口径复现——CSS if() 条件面引擎半支持状态指数膨胀，
  # 4G 单进程守卫杀树连累整轮）。variable-exponential-blowup.html（css-variables，
  # 单案 4.9GB）由 runner 谓词排除，进程启动后即时跳过，无需此快路径。
  if [[ "${name}" == "if-conditionals.html" ]]; then
    skipped_patho=$((skipped_patho + 1))
    continue
  fi
  if [[ -n "${ONLY_SUBSTR}" && "${rel}" != *"${ONLY_SUBSTR}"* ]]; then
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
  if [[ -n "${JSON_DIR}" ]]; then
    # JSON 通道：stderr 丢弃（诊断 trace 会破坏 transcript 块体的 JSON 可解析性）。
    out="$(timeout --signal=KILL "${CASE_WALL_S}" "${RUNNER}" "${SUBCMD}" --json "${rel}" 2>/dev/null)"
  else
    out="$(timeout --signal=KILL "${CASE_WALL_S}" "${RUNNER}" "${SUBCMD}" "${rel}" 2>&1)"
  fi
  rc=$?
  echo "==== ${rel}" >> "${transcript}"
  if [[ "${rc}" -eq 124 || "${rc}" -eq 137 ]]; then
    # 外部墙钟兜底杀——内部 30s 超时未生效的同步死循环形态，账面记 Timeout。
    t=1; p=0; fl=0; un=0
    if [[ -n "${JSON_DIR}" ]]; then
      # 合成单子测条目：runner 原生形状，coverage 管道按 Fail 口径计。
      printf '[["%s",[{"name":"%s","status":"Timeout","message":"external wall-clock kill (%ss, rc=%s)"}]]]\n' \
        "${rel}" "${rel}" "${CASE_WALL_S}" "${rc}" >> "${transcript}"
    else
      echo "Timeout ${rel} :: external wall-clock kill (${CASE_WALL_S}s, rc=${rc})" >> "${transcript}"
    fi
  else
    echo "${out}" >> "${transcript}"
    p="$(grep -c '^Pass ' <<<"${out}" || true)"
    fl="$(grep -c '^Fail ' <<<"${out}" || true)"
    t="$(grep -c '^Timeout ' <<<"${out}" || true)"
    un="$(grep -c '^Unsupported ' <<<"${out}" || true)"
  fi
  if [[ -n "${JSON_DIR}" ]]; then
    # JSON 通道按尝试案计（含空输出案——谓词筛减案产出 `[]`，merge 时零条目）。
    ran=$((ran + 1))
  elif [[ "${p}" -gt 0 || "${fl}" -gt 0 || "${t}" -gt 0 || "${un}" -gt 0 ]]; then
    ran=$((ran + 1))
  fi
  pass=$((pass + p)); fail=$((fail + fl)); timeout=$((timeout + t)); unsupported=$((unsupported + un))
done < <(find "${DATA_DIR}" \( -name '*.html' -o -name '*.htm' \) | sort)

if [[ -n "${JSON_DIR}" ]]; then
  # 分片 JSON 合并：transcript 各 `==== <rel>` 块体即该案 runner --json（数组），
  # 逐块解析拼接为分片级数组；形状非法即失败退出（账面静默缺案比失败更糟——
  # load_zw_results 同款严格口径）。
  mkdir -p "${JSON_DIR}"
  if python3 - "${transcript}" "${JSON_DIR}" "${CORPUS}" "${SHARD}" <<'PYEOF'
import json, re, sys

transcript, out_dir, corpus, shard = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
suffix = f"-shard{shard.replace('/', 'of')}" if re.fullmatch(r"\d+/\d+", shard) else ""
out_path = f"{out_dir}/{corpus}{suffix}.json"
entries, buf, current, line_no = [], [], None, 0
with open(transcript, encoding="utf-8", errors="replace") as fh:
    for line in fh:
        line_no += 1
        marker = re.match(r"^==== (\S+)$", line.rstrip("\n"))
        if marker:
            if buf:
                body = "\n".join(buf).strip()
                if body:
                    try:
                        data = json.loads(body)
                    except json.JSONDecodeError as err:
                        sys.exit(f"{transcript}:{line_no}: {current} 块体不是合法 runner --json: {err}")
                    if not isinstance(data, list) or not all(
                        isinstance(e, list) and len(e) == 2 and isinstance(e[0], str) and isinstance(e[1], list)
                        for e in data
                    ):
                        sys.exit(f"{transcript}: {current} 块体形状非法（须 [[case, subtests]]）")
                    entries.extend(data)
            buf, current = [], marker.group(1)
            continue
        buf.append(line.rstrip("\n"))
if buf:
    body = "\n".join(buf).strip()
    if body:
        try:
            entries.extend(json.loads(body))
        except json.JSONDecodeError as err:
            sys.exit(f"{transcript}: 尾块不是合法 runner --json: {err}")
with open(out_path, "w", encoding="utf-8") as fh:
    json.dump(entries, fh, ensure_ascii=False, indent=1)
    fh.write("\n")
counts = {"Pass": 0, "Fail": 0, "Timeout": 0, "Unsupported": 0}
for _, subtests in entries:
    for sub in subtests:
        status = sub.get("status")
        if status in counts:
            counts[status] += 1
print(
    f"css {corpus} corpus(json): cases={len(entries)} pass={counts['Pass']} "
    f"fail={counts['Fail']} timeout={counts['Timeout']} unsupported={counts['Unsupported']}"
)
print(f"merged: {out_path}")
PYEOF
  then
    exit 0
  fi
  exit 2
fi

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
