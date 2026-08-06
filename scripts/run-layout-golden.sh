#!/usr/bin/env bash
# 布局树 dump golden 对比（B1/P3 布局回归测试）
#
# 渲染上游 WPT reftest 测试页并 dump 布局树，与期望文件（golden）对比。
# 布局树格式为固定 1 位小数的 LAYOUT_DUMP 输出，结构回归（盒层次/尺寸/
# 位置/边距）可被精确捕获——比像素 diff 定位更快，比 struct-check 更细。
#
# 用法：
#   bash scripts/run-layout-golden.sh --update [filter]   # 生成/更新 golden
#   bash scripts/run-layout-golden.sh [filter]            # 对比（diff/缺失 → 退出 1）
# 示例：
#   bash scripts/run-layout-golden.sh --update css/CSS2/text/text-align-white-space-001
#   bash scripts/run-layout-golden.sh css/CSS2/backgrounds
#
# golden 存 tests/wpt-runner/layout-golden/（提交进 git，测试资产化）——
# 每个 case 一个文件，文件名 = case id 的 / → __ 转换。
# 本地入口（test-guard 包裹）：make layout-golden / make layout-golden-update

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
GOLDEN_DIR="${REPO_ROOT}/tests/wpt-runner/layout-golden"
RUNNER="${REPO_ROOT}/target/release/zero-wpt-runner"

MODE="check"
if [[ "${1:-}" == "--update" ]]; then
  MODE="update"
  shift
fi
FILTER="${1:-}"

if [[ ! -x "$RUNNER" ]]; then
  echo "Error: ${RUNNER} 不存在，请先构建（cargo build --release --bin zero-wpt-runner）"
  exit 1
fi
if [[ ! -d "${REPO_ROOT}/tests/wpt-runner/wpt-data" ]]; then
  echo "Error: wpt-data 不存在，请先运行 make fetch-wpt-data"
  exit 1
fi
mkdir -p "$GOLDEN_DIR"

OUT=$(mktemp)
trap 'rm -f "$OUT"' EXIT

echo "Layout golden ${MODE}（filter: ${FILTER:-全量}）"
echo "═══════════════════════════════════════"
(cd "$REPO_ROOT" && "$RUNNER" layout-dump $FILTER) 2>"$OUT" || true

# 用 python3 切分块并对比（项目脚本已有 python3 先例）
python3 - "$OUT" "$GOLDEN_DIR" "$MODE" <<'PY'
import os
import re
import sys

raw, gdir, mode = sys.argv[1], sys.argv[2], sys.argv[3]

blocks = {}
cur = None
buf = []
for line in open(raw, errors="replace"):
    m = re.match(r"^##### (.+) #####$", line.rstrip())
    if m:
        if cur is not None:
            blocks[cur] = "".join(buf)
        cur = m.group(1).strip()
        buf = []
    elif cur is not None:
        buf.append(line)
if cur is not None:
    blocks[cur] = "".join(buf)

if not blocks:
    print("  （无 layout-dump 输出——检查 filter 是否命中用例）")
    sys.exit(1 if mode == "check" else 0)

failed = 0
for cid, content in blocks.items():
    fname = re.sub(r"[^a-zA-Z0-9_.-]", "_", cid.replace("/", "__")) + ".txt"
    path = os.path.join(gdir, fname)
    if mode == "update":
        with open(path, "w") as f:
            f.write(content)
        print(f"  [update] {cid}")
    else:
        if not os.path.exists(path):
            print(f"  ✗ MISSING golden: {cid}（先 --update 生成）")
            failed += 1
        else:
            with open(path) as f:
                expect = f.read()
            if expect == content:
                print(f"  ✓ {cid}")
            else:
                print(f"  ✗ DIFF: {cid}")
                for a, b in zip(expect.splitlines(), content.splitlines()):
                    if a != b:
                        print(f"      expect: {a}")
                        print(f"      actual: {b}")
                        break
                else:
                    print(f"      （行数不同：expect {len(expect.splitlines())} / actual {len(content.splitlines())}）")
                failed += 1

print("═══════════════════════════════════════")
if mode == "check":
    print(f"结果：{len(blocks) - failed}/{len(blocks)} 一致，{failed} 处 diff/缺失")
    sys.exit(1 if failed else 0)
print(f"golden 更新完成：{len(blocks)} 个 case → {gdir}")
PY
