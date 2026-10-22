#!/usr/bin/env python3
"""DC-4 条带解卷：从 frame_dump 的 canvas 蓝条位置恢复 worker tick 序列，
度量 dump 空洞（忙窗期呈现采样停摆）两侧 worker 是否持续绘制——数据面
连续性证据（S1c patch 呈现面判定；dc4-analyze 的区域哈希只见呈现面，
呈现面受既有 OPEN 缺口「忙窗 keep-alive tick 挂 Wayland 阻塞读」影响，
忙窗内零 dump，须以条带位置解卷跨空洞核对 tick 速率）。

用法：python3 scripts/compositor-animation-dc4-bar-unwrap.py <frames-dir>
          --origin <ISO8601|epoch-ms> [--busy <ms>-<ms>] [--table]

原理：fixture 蓝条 x=(seq*4)%232（8×104，240×120 canvas @(24,24)），每
16ms 一次 tick。对逐 dump 解码条 x → seq mod 58；相邻 dump 间隔内按
elapsed/16 选取最近一圈数确定整圈数（解卷）。空洞跨度上的解卷 tick 数
÷ 跨度 ≈16-17ms/tick 即 worker 绘制未停（数据面存活）；≈0 或远大于
17ms/tick 则绘制停摆。呈现面占空另见 dc4-analyze（busy 窗内 dump 数）。

输出语义：
- dumps in busy window        忙窗内 dump 数（呈现面占空对照，需 --busy）；
- straddle 表                 跨 >300ms 空洞的条位置跃迁与解卷 tick 数、
                              折算 ms/tick（数据面连续性判定行）。
"""

import argparse
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

# 与 comp04a-worker-threaded-canvas.html 对齐：条 8×104 @y8、canvas
# 240×120 @(24,24)、tick 16ms、x 步进 4、回绕 232/4=58。
CANVAS_X = 24
SCAN_Y = 80  # 条纵向中线（页面坐标 24+8+52）
SCAN_X0, SCAN_X1 = CANVAS_X, CANVAS_X + 240
BAR_RGB = (30, 90, 220)
BAR_TOL = 30
TICK_MS = 16.0
X_STEP = 4
WRAP = 58
STRADDLE_MIN_MS = 300.0


def parse_origin(text: str) -> int:
    """--origin 接收 ISO8601（UTC）或 epoch 毫秒，统一为 epoch 毫秒。"""
    if text.isdigit():
        return int(text)
    dt = datetime.fromisoformat(text.replace("Z", "+00:00"))
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return int(dt.timestamp() * 1000)


def bar_x(pixels: bytes, width: int) -> int | None:
    """在扫描行上找蓝条左缘（canvas 内坐标）；无条返回 None。"""
    run = 0
    start = None
    for x in range(SCAN_X0, SCAN_X1):
        o = (SCAN_Y * width + x) * 4
        r, g, b = pixels[o], pixels[o + 1], pixels[o + 2]
        if abs(r - BAR_RGB[0]) < BAR_TOL and abs(g - BAR_RGB[1]) < BAR_TOL and abs(b - BAR_RGB[2]) < BAR_TOL:
            if run == 0:
                start = x
            run += 1
        else:
            if run >= 5:
                return start - CANVAS_X
            run = 0
    return start - CANVAS_X if run >= 5 else None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("frames_dir")
    ap.add_argument("--origin", required=True, help="页面钟原点（ISO8601 UTC 或 epoch 毫秒）")
    ap.add_argument("--busy", help="忙窗页内时刻区间，如 3000-3900")
    ap.add_argument("--table", action="store_true", help="逐 dump 条位置表")
    args = ap.parse_args()
    frames_dir = Path(args.frames_dir)
    manifest = frames_dir / "manifest.jsonl"
    if not manifest.is_file():
        print(f"no manifest at {manifest}")
        return 1
    origin = parse_origin(args.origin)
    rows = []  # (page_ms, bar_x|None, sha12)
    for line in manifest.read_text().splitlines():
        if not line.strip():
            continue
        entry = json.loads(line)
        pixels = (frames_dir / entry["file"]).read_bytes()
        width = entry["width"]
        h = hashlib.sha256()
        for y in range(24, min(144, entry["height"])):
            s = (y * width + CANVAS_X) * 4
            h.update(pixels[s : s + (264 - CANVAS_X) * 4])
        page_ms = entry["wall_ms"] - origin
        rows.append((page_ms, bar_x(pixels, width), h.hexdigest()[:12]))
    rows.sort(key=lambda r: r[0])
    bars = [(pt, bx) for pt, bx, _ in rows if bx is not None]
    print(f"frames={len(rows)} bar_dumps={len(bars)} page_span=({rows[0][0]},{rows[-1][0]})")
    if args.busy:
        lo, hi = (int(v) for v in args.busy.split("-"))
        inside = [r for r in rows if lo <= r[0] <= hi]
        print(f"dumps in busy({lo}-{hi}): {len(inside)}")
    if args.table:
        print("page_ms bar_x sha12")
        for pt, bx, sha in rows:
            print(f"{pt} {bx if bx is not None else '-'} {sha}")
    print("straddle_from to elapsed_ms bar_x_from bar_x_to ticks_unwrapped ms_per_tick")
    prev = None
    for pt, bx in bars:
        if prev is not None:
            ppt, pbx = prev
            el = pt - ppt
            if el > STRADDLE_MIN_MS:
                est = el / TICK_MS
                base = (bx - pbx) // X_STEP % WRAP
                best = min((base + WRAP * k for k in (-1, 0, 1, 2)), key=lambda c: abs(c - est))
                # 同 x 跨长空洞：冻结与整圈回绕不可分（终末冻结判读须结合
                # Destroy/静默核查面，见 evidence README），显式标注。
                note = " same-x:frozen-or-wrap" if bx == pbx else (
                    "" if abs(best - est) <= WRAP / 4 else " ambiguous")
                print(f"{ppt} {pt} {el} {pbx} {bx} {best} {el / best:.1f}{note}")
        prev = (pt, bx)
    return 0


if __name__ == "__main__":
    sys.exit(main())
