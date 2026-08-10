#!/usr/bin/env bash
# WPT Reftest Smoke 运行脚本（B2：PR CI 秒级分层门禁）
#
# 读取 tests/wpt-runner/reftest-smoke.txt 清单（每行一个 case id），
# 对每个 case 跑 reftest-upstream 并断言通过；任一失败 → 退出 1。
#
# 用法：
#   bash scripts/run-reftest-smoke.sh          # 跑全部清单
#   bash scripts/run-reftest-smoke.sh --list   # 只列清单
# 本地入口（test-guard 包裹）：make reftest-smoke

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SMOKE_LIST="${REPO_ROOT}/tests/wpt-runner/reftest-smoke.txt"
RUNNER="${REPO_ROOT}/target/release/zero-wpt-runner"

if [[ ! -f "$SMOKE_LIST" ]]; then
  echo "Error: smoke 清单不存在: ${SMOKE_LIST}"
  exit 1
fi

# 读取清单（去注释/空行）
mapfile -t CASES < <(grep -vE '^\s*(#|$)' "$SMOKE_LIST" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')

if [[ "${1:-}" == "--list" ]]; then
  printf '%s\n' "${CASES[@]:-（空清单）}"
  exit 0
fi

if [[ ${#CASES[@]} -eq 0 ]]; then
  echo "Smoke 清单为空（tests/wpt-runner/reftest-smoke.txt）——跳过 smoke 门禁。"
  echo "提示：从全量通过结果中填充代表性 case 后本门禁生效。"
  exit 0
fi

if [[ ! -x "$RUNNER" ]]; then
  echo "Error: ${RUNNER} 不存在，请先构建（cargo build --release --bin zero-wpt-runner）"
  exit 1
fi
if [[ ! -d "${REPO_ROOT}/tests/wpt-runner/wpt-data" ]]; then
  echo "Error: wpt-data 不存在，请先运行 make fetch-wpt-data"
  exit 1
fi

echo "Reftest Smoke（${#CASES[@]} cases）"
echo "═══════════════════════════════════════"

failed=0
missing_corpus=0
for case_id in "${CASES[@]}"; do
  output=$("$RUNNER" reftest-upstream "$case_id" 2>&1 || true)
  # 0-case 匹配 = wpt-data 缺语料，与像素失败不同（见 fetch-wpt-smoke-subdirs.sh）
  running=$(echo "$output" | awk '/Running [0-9]+ upstream reftest cases/{print $2; exit}')
  # 提取 `  Passed:  N` 摘要行的数值（runner format_reftest_report 输出）。用 awk 而非 `grep -oP`
  #（Perl 正则 \K）——后者在 BSD grep（macOS）不可用，会致 smoke 门禁在 macOS 恒误报 FAIL。
  passed=$(echo "$output" | awk '/^  Passed:/{print $2; exit}')
  if [[ "${running:-0}" == "0" ]]; then
    echo "  ⊘ ${case_id}（CORPUS_MISSING：wpt-data 无此 case，先 make fetch-wpt-data）"
    echo "$output" | tail -4 | sed 's/^/    /'
    missing_corpus=$((missing_corpus + 1))
    failed=$((failed + 1))
  elif [[ "$passed" == "1" ]]; then
    echo "  ✓ ${case_id}"
  else
    echo "  ✗ ${case_id}（Passed: ${passed:-0}）"
    echo "$output" | tail -8 | sed 's/^/    /'
    failed=$((failed + 1))
  fi
done

echo "═══════════════════════════════════════"
if [[ $failed -gt 0 ]]; then
  if [[ $missing_corpus -gt 0 ]]; then
    echo "Smoke 失败：${failed}/${#CASES[@]}（其中 ${missing_corpus} 条语料缺失，非渲染回归）。"
    echo "修复：make fetch-wpt-data（自动补齐 css-variables/css-ruby）或 bash scripts/fetch-wpt-smoke-subdirs.sh"
  else
    echo "Smoke 失败：${failed}/${#CASES[@]}。清单中的 case 必须保持通过——"
    echo "若属预期行为变化，先修回归；确需移除时更新 reftest-smoke.txt 并说明理由。"
  fi
  exit 1
fi
echo "Smoke 全部通过（${#CASES[@]}/${#CASES[@]}）"
