#!/usr/bin/env bash
# R5114（site-optimizer P9-9 切片 2）：css-variables 语料分片运行器。
#
# 为何分片：runner 单进程跑全语料（~250 案）时 RSS 线性累积（每案独立 WebView/V8
# 的分配器驻留，~35MB/案），8GB 单进程守卫仍不足；单案 RSS <300MB。每案独立
# runner 进程使内存天然有界（探针记录见运行目录）。代价是每案 ~1s 的进程启动
# 开销——语料基线/回归记录场景可接受，速度敏感场景待 runner 进程模型演进。
#
# 退出码：0 = 至少跑了 1 案（记录模式，不设通过门禁——clipboard/csp 先例：
# 基线首跑通过率只记录）；2 = 语料为空/.runner 缺失（基础设施错误）。
# 名字级筛减（crash/manual/病态输入）在此快速跳过；内容级筛减（iframe 依赖等）由
# runner 侧 css_variables_case_skipped 单点负责，此处不复制规则。病态输入（crash、
# 指数膨胀）按仓库 fuzz-repro-guard 口径不入常驻运行面（首跑实证：exponential-
# blowup 单案把 runner 撑到 4.9GB 触发守卫杀树）。

set -uo pipefail

RUNNER="${1:-./target/release/zero-wpt-runner}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
DATA_DIR="${REPO_ROOT}/tests/wpt-runner/wpt-data/css/css-variables"

if [[ ! -x "${RUNNER}" || ! -d "${DATA_DIR}" ]]; then
  echo "css-variables corpus: runner or data missing" >&2
  exit 2
fi

ran=0 skipped=0 pass=0 fail=0 unsupported=0
transcript="${CSSVARS_TRANSCRIPT:-/tmp/css-variables-corpus.log}"
: > "${transcript}"

for f in "${DATA_DIR}"/*.html; do
  name="$(basename "${f}")"
  case "${name}" in
    *-crash.html|*-manual.html|variable-exponential-blowup.html) skipped=$((skipped + 1)); continue;;
  esac
  out="$("${RUNNER}" testharness-css-variables "css/css-variables/${name}" 2>&1)"
  echo "==== ${name}" >> "${transcript}"
  echo "${out}" >> "${transcript}"
  p="$(grep -c '^Pass' <<<"${out}" || true)"
  fl="$(grep -c '^Fail' <<<"${out}" || true)"
  un="$(grep -c '^Unsupported' <<<"${out}" || true)"
  pass=$((pass + p)); fail=$((fail + fl)); unsupported=$((unsupported + un))
  if [[ "${p}" -gt 0 || "${fl}" -gt 0 || "${un}" -gt 0 ]]; then
    ran=$((ran + 1))
  fi
done

echo "css-variables corpus: cases=${ran} skipped-by-name=${skipped} pass=${pass} fail=${fail} unsupported=${unsupported}"
echo "transcript: ${transcript}"
if [[ "${ran}" -eq 0 ]]; then
  exit 2
fi
exit 0
