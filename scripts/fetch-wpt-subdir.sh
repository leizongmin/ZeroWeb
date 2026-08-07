#!/usr/bin/env bash
# 从上游 wpt 拉取缺失子域到本地 wpt-data（C2 套件补齐）
#
# 用途：zeroweb-wpt-data 独立 repo 是裁剪子集，个别子域未打包
# （如 css/filter-effects）。本脚本按需从上游补到本地 wpt-data
# （gitignored 本地数据），供 reftest-upstream 使用。
#
# 用法：
#   bash scripts/fetch-wpt-subdir.sh css/filter-effects
#   bash scripts/fetch-wpt-subdir.sh --list    # 显示上游 css/ 顶层子域
#
# 依赖：curl, python3

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
WPT_DATA="${REPO_ROOT}/tests/wpt-runner/wpt-data"
API="https://api.github.com/repos/web-platform-tests/wpt/contents"

if [[ "${1:-}" == "--list" ]]; then
  curl -s --max-time 30 "${API}/css" | python3 -c "
import json, sys
d = json.load(sys.stdin)
for e in d:
    if e['type'] == 'dir':
        print(e['name'])
"
  exit 0
fi

SUBDIR="${1:?用法: $0 <wpt 子域>}"
echo "拉取上游子域: ${SUBDIR} → ${WPT_DATA}/${SUBDIR}"

# python 递归列出子域全部文件路径
python3 - "$SUBDIR" <<'PY' > /tmp/wpt-subdir-files.txt
import json
import sys
import urllib.request

subdir = sys.argv[1]
api = "https://api.github.com/repos/web-platform-tests/wpt/contents"
files = []

def walk(path):
    url = f"{api}/{path}"
    req = urllib.request.Request(url, headers={"User-Agent": "zero-web"})
    with urllib.request.urlopen(req, timeout=30) as r:
        entries = json.load(r)
    for e in entries:
        if e["type"] == "dir":
            walk(e["path"])
        elif e["type"] == "file":
            files.append(e["path"])

walk(subdir)
for f in files:
    print(f)
PY

total=$(wc -l < /tmp/wpt-subdir-files.txt)
echo "共 ${total} 个文件，开始下载..."
ok=0
while IFS= read -r f; do
  target="${WPT_DATA}/${f}"
  mkdir -p "$(dirname "$target")"
  if curl -s -o "$target" --max-time 30 "https://raw.githubusercontent.com/web-platform-tests/wpt/master/${f}"; then
    ok=$((ok + 1))
  else
    echo "  FAIL: ${f}"
  fi
done < /tmp/wpt-subdir-files.txt

echo "完成：${ok}/${total} 文件（缺失子域已补入本地 wpt-data）"
