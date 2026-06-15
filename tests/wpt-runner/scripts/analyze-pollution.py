#!/usr/bin/env python3
"""污染分类分析（DC-14 Phase 1B）。

解析 cross-validate 全量 evidence，把「污染」（同源通过但 chromium 不一致）用例
按 z_vs_chr 量级 × 目录交叉分类，产出「真 bug 候选」优先清单，区分：
  - 真 bug 候选（高信度）：布局目录 + 高量级（z_vs_chr > 5%）—— 差异不太可能纯字体
  - 字体/噪声嫌疑：文字目录（fonts/text-decor/writing-modes）或低量级（< 3%）

⚠️ 这是启发式优先级排序，非精确判定。最终是否真 bug 需逐用例看 Z_test/C_test/Z_ref 像素。

用法：
  python3 analyze-pollution.py --evidence <cross-validate-full-*.txt> [--hi 5] [--lo 3]
"""
import argparse
import os
import re
import sys
from collections import defaultdict

LAYOUT_DIRS = {"css/CSS2", "css/css-flexbox", "css/css-grid",
               "css/css-multicol", "css/css-tables", "css/css-position"}
TEXT_DIRS = {"css/css-fonts", "css/css-text-decor", "css/css-writing-modes"}

ROW_RE = re.compile(r"^\|\s*([^|]+?)\s*\|\s*(POLLUTED|self-pass|self-fail|SIZE-MISMATCH)\s*\|\s*([\d.]+)%\s*\|\s*([\d.]+)%\s*\|\s*$")


def category_of(sid):
    parts = sid.split('_')
    if len(parts) >= 2 and parts[0] == 'css':
        return 'css/CSS2' if parts[1] == 'CSS2' else 'css/' + parts[1]
    return '(other)'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--evidence", required=True)
    ap.add_argument("--hi", type=float, default=5.0, help="真 bug 候选量级下限（z_vs_chr > hi 且布局目录）")
    ap.add_argument("--lo", type=float, default=3.0, help="低量级噪声上限（z_vs_chr < lo）")
    args = ap.parse_args()

    polluted = []  # (sid, cat, z_vs_ref, z_vs_chr)
    with open(args.evidence) as f:
        for line in f:
            m = ROW_RE.match(line)
            if not m:
                continue
            sid, verdict, zr, zc = m.group(1).strip(), m.group(2), float(m.group(3)), float(m.group(4))
            if verdict != "POLLUTED":
                continue
            polluted.append((sid, category_of(sid), zr, zc))

    print(f"# 污染分类分析（DC-14 Phase 1B）")
    print(f"# evidence: {args.evidence}")
    print(f"# 污染用例总数: {len(polluted)}\n")

    # 量级桶
    def bucket(v):
        if v < args.lo:
            return f"<{args.lo:.0f}%"
        if v < 8:
            return f"{args.lo:.0f}-8%"
        if v < 20:
            return "8-20%"
        return ">20%"

    buckets = ["<3%", "3-8%", "8-20%", ">20%"]
    cats = sorted({c for _, c, _, _ in polluted})
    cross = defaultdict(lambda: defaultdict(int))
    for sid, cat, zr, zc in polluted:
        cross[cat][bucket(zc)] += 1

    print(f"## 量级 × 目录交叉（污染用例数）")
    print(f"| 目录 | " + " | ".join(buckets) + " | 总 |")
    print(f"|------|" + "|".join(["------"] * (len(buckets) + 1)) + "|")
    for cat in cats:
        row = [cross[cat][b] for b in buckets]
        print(f"| {cat} | " + " | ".join(str(x) for x in row) + f" | {sum(row)} |")
    tot = [sum(cross[c][b] for c in cats) for b in buckets]
    print(f"| **总计** | " + " | ".join(str(x) for x in tot) + f" | {sum(tot)} |")

    # 分类
    true_bug = [(s, c, zr, zc) for s, c, zr, zc in polluted if c in LAYOUT_DIRS and zc > args.hi]
    text_suspect = [(s, c, zr, zc) for s, c, zr, zc in polluted if c in TEXT_DIRS]
    low_noise = [(s, c, zr, zc) for s, c, zr, zc in polluted if zc < args.lo]
    # 低量级噪声优先于文字嫌疑扣除（避免重复计数感）
    text_suspect = [t for t in text_suspect if t[3] >= args.lo]
    layout_low = [(s, c, zr, zc) for s, c, zr, zc in polluted
                  if c in LAYOUT_DIRS and zc <= args.hi and zc >= args.lo]

    print(f"\n## 分类统计")
    print(f"- **真 bug 候选（布局目录 + z_vs_chr>{args.hi:.0f}%）: {len(true_bug)}**")
    print(f"- 文字目录嫌疑（fonts/text-decor/writing-modes, >{args.lo:.0f}%）: {len(text_suspect)}（含字体度量噪声，需逐个甄别）")
    print(f"- 低量级噪声（<{args.lo:.0f}%）: {len(low_noise)}（多为字体/亚像素）")
    print(f"- 布局目录中量级（{args.lo:.0f}-{args.hi:.0f}%）: {len(layout_low)}（边界，需逐个看）")

    print(f"\n## 真 bug 候选清单（按 z_vs_chr 降序，优先修复）")
    print(f"这些是布局目录中 chromium 与 ZeroWeb 大幅不一致、却被同源验证判为「通过」的用例 ——")
    print(f"最可能是被同源假通过掩盖的真实渲染缺口。\n")
    print(f"| safe_id | 目录 | z_vs_ref | z_vs_chr |")
    print(f"|---------|------|----------|----------|")
    for s, c, zr, zc in sorted(true_bug, key=lambda x: -x[3]):
        print(f"| {s} | {c} | {zr:.2f}% | {zc:.2f}% |")

    print(f"\n⚠️ 启发式提醒：CSS2 部分用例用 Ahem 字体，>5% 差异仍可能含 Ahem 度量成分；")
    print(f"   writing-modes 的垂直文字差异难分字体 vs 轴交换 bug，已整体归入「文字嫌疑」需逐个甄别。")


if __name__ == "__main__":
    main()
