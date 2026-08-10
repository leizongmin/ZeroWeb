#!/usr/bin/env bash
# 补齐 reftest-smoke.txt 依赖、但 zeroweb-wpt-data v1.10 未打包的子域。
#
# v1.10 裁剪子集不含 css/css-variables、css/css-ruby；smoke 清单 6 条
# 依赖这些目录。fresh clone 后按需从上游 WPT 补入（同 fetch-wpt-subdir.sh）。
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
