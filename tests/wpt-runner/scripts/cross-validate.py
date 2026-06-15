#!/usr/bin/env python3
"""交叉验证：ZeroWeb 同源通过 vs chromium 独立 Oracle（DC-14 anti-false-pass）。

输入：
  - ZeroWeb REFTEST_DUMP 产物（target/reftest-dump/{safe_id}-test.png / -ref.png）
  - chromium Oracle 截图（oracle-shots/{safe_id}.png）
对每个 safe_id 计算两个差异：
  - z_vs_ref : ZeroWeb-test vs ZeroWeb-ref（同源判定，即 runner 的通过依据）
  - z_vs_chr : ZeroWeb-test vs chromium-test（独立 Oracle）
污染 = 同源通过（z_vs_ref 小）但 chromium 不一致（z_vs_chr 大）——即「假通过」。

用法：
  python3 cross-validate.py --dump <dir> --oracle <dir> \\
                            [--chan-thresh 5] [--pass-ratio 0.005] [--oracle-ratio 0.01]

依赖：仅 Pillow（PIL）。不依赖 numpy。
"""
import argparse
import os
import sys
import glob
from PIL import Image, ImageChops


def diff_ratio(path_a, path_b, chan_thresh):
    """差异像素占比 + 最大通道差。像素任一通道差 > chan_thresh 视为差异。返回 (ratio, max_chan) 或 (None,None)（尺寸不符）。"""
    a = Image.open(path_a).convert("RGB")
    b = Image.open(path_b).convert("RGB")
    if a.size != b.size:
        return None, None
    diff = ImageChops.difference(a, b)
    r, g, bl = diff.split()
    m = ImageChops.lighter(ImageChops.lighter(r, g), bl)  # 每像素最大通道差（灰度）
    binmask = m.point(lambda x: 255 if x > chan_thresh else 0)
    hist = binmask.histogram()
    total = a.size[0] * a.size[1]
    differing = hist[255] if len(hist) > 255 else 0
    return differing / total, m.getextrema()[1]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dump", required=True, help="ZeroWeb REFTEST_DUMP 目录")
    ap.add_argument("--oracle", required=True, help="chromium Oracle 截图目录")
    ap.add_argument("--chan-thresh", type=int, default=5, help="通道差阈值（>视为差异像素）")
    ap.add_argument("--pass-ratio", type=float, default=0.005, help="同源通过判定：z_vs_ref < 此值（默认 0.5%%）")
    ap.add_argument("--oracle-ratio", type=float, default=0.01, help="Oracle 不一致判定：z_vs_chr > 此值（默认 1%%）")
    args = ap.parse_args()

    dump = os.path.abspath(args.dump)
    oracle = os.path.abspath(args.oracle)
    for d, n in [(dump, "dump"), (oracle, "oracle")]:
        if not os.path.isdir(d):
            print(f"ERROR: {n} dir not found: {d}", file=sys.stderr); sys.exit(1)

    def sid(p):
        return os.path.basename(p)[:-4]
    oracle_ids = {sid(p): p for p in glob.glob(os.path.join(oracle, "*.png"))}

    rows = []
    n_self_pass = n_polluted = n_compared = 0
    for s, opath in sorted(oracle_ids.items()):
        zt = os.path.join(dump, s + "-test.png")
        zr = os.path.join(dump, s + "-ref.png")
        if not (os.path.exists(zt) and os.path.exists(zr)):
            continue
        zr_r, zr_m = diff_ratio(zt, zr, args.chan_thresh)
        zc_r, zc_m = diff_ratio(zt, opath, args.chan_thresh)
        if zr_r is None or zc_r is None:
            rows.append((s, "SIZE-MISMATCH", None, None)); continue
        self_pass = zr_r < args.pass_ratio
        oracle_disagree = zc_r > args.oracle_ratio
        polluted = self_pass and oracle_disagree
        n_compared += 1
        n_self_pass += 1 if self_pass else 0
        n_polluted += 1 if polluted else 0
        rows.append((s, "POLLUTED" if polluted else ("self-pass" if self_pass else "self-fail"), zr_r, zc_r))

    print(f"# 交叉验证：ZeroWeb 同源 vs chromium Oracle（DC-14）")
    print(f"# dump: {dump}")
    print(f"# oracle: {oracle}")
    print(f"# 阈值: chan>{args.chan_thresh}, 同源通过<{args.pass_ratio*100:.2f}%, Oracle不一致>{args.oracle_ratio*100:.2f}%\n")
    print(f"| safe_id | 判定 | z_vs_ref | z_vs_chr |")
    print(f"|---------|------|----------|----------|")
    for s, v, zr, zc in rows:
        if zr is None:
            print(f"| {s} | {v} | — | — |")
        else:
            print(f"| {s} | {v} | {zr*100:.2f}% | {zc*100:.2f}% |")

    print(f"\n## 汇总")
    print(f"- 对比用例数: {n_compared}")
    print(f"- 同源通过（ZeroWeb vs ref）: {n_self_pass}")
    print(f"- **污染（同源通过但 chromium 不一致）: {n_polluted}**")
    if n_self_pass:
        print(f"- 污染率（占同源通过）: {n_polluted/n_self_pass*100:.1f}%")
    print(f"\n⚠️ 污染 = 被 ZeroWeb 判为通过、但与 chromium 独立 Oracle 不一致的用例。")
    print(f"   这是 436/490 同源通过率里水分的上界估计（抽样）。")


if __name__ == "__main__":
    main()
