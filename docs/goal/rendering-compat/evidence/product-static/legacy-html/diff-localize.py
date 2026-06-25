#!/usr/bin/env python3
# R657: pixel-level diff localization (pure PIL, no numpy/scipy) to distinguish
# font/AA noise (scattered thin edge diff) from layout bugs (contiguous blocks).
#
# 用法: python3 diff-localize.py <zeroweb.png> <chromium.png> [--heat <out.png>]
import sys
from PIL import Image

def main():
    zw_path = sys.argv[1]
    chr_path = sys.argv[2]
    heat_out = None
    if "--heat" in sys.argv:
        heat_out = sys.argv[sys.argv.index("--heat") + 1]

    zw = Image.open(zw_path).convert("RGB")
    chr_ = Image.open(chr_path).convert("RGB")
    w = min(zw.width, chr_.width)
    h = min(zw.height, chr_.height)
    zpix = zw.load()
    cpix = chr_.load()

    total = w * h
    ndiff = 0
    sum_x = 0
    sum_y = 0
    min_x, min_y, max_x, max_y = w, h, 0, 0
    # store a 2D boolean mask as bytearray rows for neighbor/coherence analysis
    mask = bytearray(h * w)
    for y in range(h):
        base = y * w
        for x in range(w):
            zr, zg, zb = zpix[x, y]
            cr, cg, cb = cpix[x, y]
            if zr != cr or zg != cg or zb != cb:
                mask[base + x] = 1
                ndiff += 1
                sum_x += x
                sum_y += y
                if x < min_x: min_x = x
                if x > max_x: max_x = x
                if y < min_y: min_y = y
                if y > max_y: max_y = y

    pct = 100.0 * ndiff / total
    print(f"size={w}x{h} total={total} diff_px={ndiff} ({pct:.2f}%)")
    if ndiff == 0:
        print("NO_DIFF"); return

    cx, cy = sum_x / ndiff, sum_y / ndiff
    print(f"diff centroid=({cx:.0f},{cy:.0f}) bbox=x[{min_x}..{max_x}] y[{min_y}..{max_y}] "
          f"(bbox area={100.0*(max_x-min_x+1)*(max_y-min_y+1)/total:.1f}% of canvas)")

    # coherence: diff px with >=1 4-conn diff neighbor
    with_neighbor = 0
    for y in range(h):
        base = y * w
        for x in range(w):
            if mask[base + x]:
                if (x > 0 and mask[base + x - 1]) or (x + 1 < w and mask[base + x + 1]) \
                   or (y > 0 and mask[base - w + x]) or (y + 1 < h and mask[base + w + x]):
                    with_neighbor += 1
    coh = 100.0 * with_neighbor / ndiff
    print(f"coherence(diff px w/ >=1 diff neighbor)={coh:.1f}% "
          f"[>55% contiguous = possible layout bug; <40% scattered = AA/font noise]")

    # thick-core: 5x5 erosion survivor (solid filled regions, not thin edges)
    thick = 0
    for y in range(2, h - 2):
        base = y * w
        for x in range(2, w - 2):
            if not mask[base + x]:
                continue
            # check all 8 neighbors in 5x5 must be diff (approx erosion)
            ok = True
            for dy in (-2, 0, 2):
                rb = base + dy * w
                for dx in (-2, 0, 2):
                    if not mask[rb + x + dx]:
                        ok = False; break
                if not ok: break
            if ok:
                thick += 1
    print(f"thick_core_px(5x5 erosion survivor)={thick} ({100.0*thick/ndiff:.1f}% of diff) "
          f"[high → solid regions differ → layout/color bug; low → thin edges → font]")

    if heat_out:
        heat = zw.copy()
        hp = heat.load()
        for y in range(h):
            base = y * w
            for x in range(w):
                if mask[base + x]:
                    hp[x, y] = (255, 0, 0)
                else:
                    r, g, b = hp[x, y]
                    hp[x, y] = (r * 2 // 5, g * 2 // 5, b * 2 // 5)
        heat.save(heat_out)
        print(f"wrote heatmap {heat_out}")

if __name__ == "__main__":
    main()
