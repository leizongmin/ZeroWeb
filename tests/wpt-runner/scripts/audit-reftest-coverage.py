#!/usr/bin/env python3
"""Reftest 分母覆盖率对账工具（DC-14 分母真实性 / anti-false-pass）。

统计本地已导入的上游 WPT reftest 数量（N_imported），揭露「每目录只取子集」
导致的分母缩水问题——`import-wpt-reftests.sh` 默认 `COUNT=60`，当前 490/503
是每目录约 60 个的**子集**，远非 `rendering-compat.md` 要求的「上游该目录全部
范围内 reftest」。本工具输出 N_imported 表，供 DC-14 分母真实性审计使用。

N_full（上游全量）需对比上游 WPT MANIFEST.json，本工具暂不联网获取（避免网络
依赖与 GitHub API 限流）；接入 `--upstream` 时可扩展为 N_imported / N_full 覆盖率。

用法：
  python3 audit-reftest-coverage.py            # 本地统计 N_imported（零依赖）

依赖：仅 Python 3 标准库。
"""
import collections
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MANIFEST = os.path.join(HERE, "..", "wpt-data", "reftest-manifest.json")

# DC-2~5 覆盖的目标目录（与 rendering-compat.md 一致）
TARGET_DIRS = [
    "css/CSS2",
    "css/css-flexbox",
    "css/css-grid",
    "css/css-position",
    "css/css-tables",
    "css/css-multicol",
    "css/css-writing-modes",
    "css/css-fonts",
    "css/css-text-decor",
]


def stat_dir(test_path):
    """从 test 路径提取聚合目录：CSS2 聚合到一级，其余取 css/<top>。"""
    parts = test_path.split("/")
    for i, seg in enumerate(parts):
        if seg == "css" and i + 1 < len(parts):
            return "css/" + parts[i + 1]
    return "(other)"


def count_imported():
    with open(MANIFEST) as f:
        m = json.load(f)
    c = collections.Counter()
    for e in m.get("entries", []):
        c[stat_dir(e.get("test", ""))] += 1
    return c


def main():
    if not os.path.exists(MANIFEST):
        print(f"ERROR: manifest not found: {MANIFEST}", file=sys.stderr)
        sys.exit(1)

    imported = count_imported()
    total = sum(imported.values())

    print("# Reftest 分母覆盖率对账（DC-14 分母真实性）")
    print(f"# 数据源: {os.path.relpath(MANIFEST, HERE)}")
    print(f"# 性质: 每目录 N_imported 来自 import 脚本（默认 COUNT=60），是**子集**非上游全量\n")
    print("| 目录 | N_imported（本地子集） |")
    print("|------|----------------------|")
    for d in sorted(imported):
        flag = " ⚠️ 子集" if imported[d] <= 65 else ""
        print(f"| {d} | {imported[d]}{flag} |")
    print(f"| **总计** | **{total}** |")

    subset = sum(1 for n in imported.values() if n <= 65)
    print(
        f"\n⚠️ {subset}/{len(imported)} 个目录 N_imported ≤ 65，符合 import 脚本默认 COUNT=60 的子集特征。"
    )
    print(
        "   DC-14 要求分母 = 上游每目录**全部**范围内 reftest；当前覆盖率 N_imported/N_full 未知，"
        "需扩展 --upstream 模式（fetch 上游 MANIFEST.json）补全。"
    )


if __name__ == "__main__":
    main()
