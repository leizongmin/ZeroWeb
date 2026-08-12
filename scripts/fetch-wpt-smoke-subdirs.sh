#!/usr/bin/env bash
# 补齐 reftest-smoke.txt 依赖、但 zeroweb-wpt-data v1.10 未打包的子域/资源。
#
# v1.10 裁剪子集不含 css/css-variables、css/css-ruby；smoke 清单 6 条
# 依赖这些目录。fresh clone 后按需从上游 WPT 补入（同 fetch-wpt-subdir.sh）。
# 另有单文件资源（reftest 字体测试依赖的 ahem-ex-*.otf）同样按需补入。
#
# 用法：由 make fetch-wpt-data / update-wpt-data.sh 自动调用；也可手动执行。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"

if [[ ! -d "$WPT_DATA" ]]; then
  echo "Error: wpt-data 不存在（先 make fetch-wpt-data）"
  exit 1
fi

ensure_subdir() {
  local subdir="$1"
  local marker="$2"
  if [[ -f "${WPT_DATA}/${marker}" ]]; then
    echo "  smoke 子域已就绪: ${subdir}"
    return 0
  fi
  echo "  smoke 子域缺失 ${subdir}，从上游 WPT 补齐..."
  bash "${SCRIPT_DIR}/fetch-wpt-subdir.sh" "${subdir}"
}

echo "检查 reftest-smoke 依赖子域..."
ensure_subdir css/css-variables css/css-variables/css-vars-custom-property-case-sensitive-001.html
ensure_subdir css/css-ruby css/css-ruby/abs-in-ruby-base.html

# R3254：reftest_fonts 单元测试（wpt-runner）依赖的字体资源——v1.10 裁剪子集
# 未打包 css/css-fonts/resources 的 Ahem 变体（ahem-ex-*.otf），缺失时
# `cargo test -p zero-wpt-runner` 直接失败。按需从上游 WPT 单文件补入。
ensure_font_file() {
  local rel="$1"
  local path="${WPT_DATA}/${rel}"
  if [[ -f "$path" ]]; then
    echo "  font 资源已就绪: ${rel}"
    return 0
  fi
  echo "  font 资源缺失 ${rel}，从上游 WPT 补齐..."
  mkdir -p "$(dirname "$path")"
  curl -fsSL --max-time 60 "https://raw.githubusercontent.com/web-platform-tests/wpt/master/${rel}" -o "$path"
}

echo "检查 reftest 字体资源..."
ensure_font_file css/css-fonts/resources/ahem-ex-500.otf
ensure_font_file css/css-fonts/resources/ahem-ex-250.otf
