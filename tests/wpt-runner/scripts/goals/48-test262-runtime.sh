#!/usr/bin/env bash
# Goal: wptfyi-runtime（编号 48——五目标之「Test262、Encoding 与 Wasm」，2026-11-09 立项）
# third_party/test262 上游语料 vendoring（契约见 docs/goal/wptfyi-runtime.md DC-1）。
#
# 与 lib.sh 逐文件 fetch 不同：test262 是 5.3 万文件的整目录 vendored 子树
# （上游 web-platform-tests/wpt 内 third_party/test262，见 vendored.toml——
# tc39/test262 @ b66872a9），逐文件 raw 拉取不可行，改走 git partial clone +
# cone sparse-checkout（blob 按需取，仅下载 test262 子树 ~225MB）。
#
# 语料形态：
# - third_party/test262/test/**/*.js——test262 用例（YAML frontmatter：
#   flags [module/onlyStrict/async/raw]、includes、negative {phase,type}）；
#   无 frontmatter 的 .js 为 support 文件（上游 manifest 同判据，runner skip 记账）
# - third_party/test262/harness/*.js——用例 includes 引用的 harness 库
#   （assert/sta/doneprintHandle 等常驻 + include 列表按文件引用）
# - resources/test262/{test262-reporter,test262-provider}.js——上游 serve 包装器
#   脚本（reporter 负例分类 + $262 provider），runner 折叠包装页内联消费
#
# 变体记账（上游 serve.py + tools/manifest/sourcefile.py 逐字核对）：
#   flags 含 module   → <name>.test262-module.html（dynamic import 包装）
#   flags 含 onlyStrict（非 module）→ <name>.test262.strict.html（"use strict" 前缀）
#   其余              → <name>.test262.html
# 每用例恰一变体（test262 语料无 META: variant——全库 grep 零命中实证）。
#
# runner 通道：zero-wpt-runner testharness-test262（testharness_test262.rs）；
# Makefile target：fetch-wpt-test262 / testharness-test262。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

GOAL="wptfyi-runtime"
VENDOR_DIR="${WPT_DATA}/third_party/test262"

# Vendoring 幂等：vendored.toml 在位即视为已拉过（同 pin 文件集稳定；FORCE=1 重做）。
if [[ "${FORCE:-0}" != "1" && -f "${VENDOR_DIR}/vendored.toml" ]]; then
  echo "== third_party/test262 已 vendored（${VENDOR_DIR}），跳过（FORCE=1 重做）=="
else
  echo "== vendoring third_party/test262（pin ${WPT_REV:0:9}，partial clone + sparse）=="
  clone_root="$(mktemp -d /tmp/wpt-test262-vendor.XXXXXX)"
  repo="${clone_root}/repo"
  git clone --filter=blob:none --sparse --no-checkout \
    https://github.com/web-platform-tests/wpt.git "${repo}"
  git -C "${repo}" sparse-checkout init --cone
  git -C "${repo}" sparse-checkout set third_party/test262
  git -C "${repo}" fetch --depth 1 origin "${WPT_REV}"
  git -C "${repo}" checkout "${WPT_REV}"
  mkdir -p "${WPT_DATA}/third_party"
  # 目录换入：旧 vendored 目录先挪进本次临时区（同 pin 内容恒等，仅 pin 变更时才有残页风险），
  # 新目录原子 mv 入位，临时区整体回收——rm 严格限定在 mktemp 目录内。
  if [[ -e "${VENDOR_DIR}" ]]; then
    mv "${VENDOR_DIR}" "${clone_root}/old-vendor"
  fi
  mv "${repo}/third_party/test262" "${VENDOR_DIR}"
  rm -rf "${clone_root}"
fi

# 上游 serve 包装器脚本（reporter/provider）——runner 折叠包装页按同 pin 内联。
for resource in resources/test262/test262-reporter.js resources/test262/test262-provider.js; do
  fetch_raw "${resource}"
done

echo "== 语料盘点 =="
if [[ -d "${VENDOR_DIR}/test" ]]; then
  n_test=$(find "${VENDOR_DIR}/test" -name '*.js' ! -name '*_FIXTURE.js' | wc -l)
  n_harness=$(find "${VENDOR_DIR}/harness" -name '*.js' | wc -l)
  printf '  third_party/test262/test    %6d 用例（含 support 无 frontmatter 文件）\n' "${n_test}"
  printf '  third_party/test262/harness %6d harness 库\n' "${n_harness}"
else
  echo "  ⚠ ${VENDOR_DIR}/test 缺失——vendoring 未完成" >&2
  exit 1
fi

goals_next_steps "${GOAL}"
