#!/usr/bin/env python3
"""DC-3（COMP-03）帧序列分析：从 frame_dump 产物提取命中/暂停/取消/恢复证据
（S3d 证据工具，零第三方依赖）。

用法：python3 scripts/compositor-animation-dc3-analyze.py <frames-dir> [> analysis.txt]

对 manifest.jsonl 每帧计算：
- hit-box（#hit-box 400×110 @(24,180)，蓝）非白 bbox 中心 x → 合成位移轨迹
  （动画中越出基线右缘 424 即为「视觉与基线分叉」的直接证据）；
- pause-box（#pause-box 120×110 @(24,330)，青）非白 bbox 中心 x → 位移轨迹与
  冻结段识别（连续帧 bbox 恒定段，页内 t=2.5s paused / t=6.5s running）；
- cancel-box（#cancel-box 96² @(500,320)，橙）非白 bbox → 旋转相位佐证，
  取消（t=5.0s）后 bbox 应回落基线正方形 500..596×320..416；
- 相位条带（#q-strip 128×16 @(24,470)，灰度 = 255 − round(shift/240×240)）
  → 反解 translateX 位移，冻结段恒定、恢复段续走；
- 回执五格（y∈[508,524)：hit-a x[24,88) / hit-b x[112,176) / hit-c x[200,264) /
  ev-cancel x[288,352) / phase-resume x[376,440)）→ none/pass/fail 分类与首见。
"""

import hashlib
import json
import sys
from pathlib import Path


def solid_mean(rgba, width, height, x0, y0, x1, y1):
    """区域全像素均值（实心 div/条带区域）。"""
    count = 0
    sr = sg = sb = 0
    for y in range(y0, min(y1, height)):
        row = y * width
        for x in range(x0, min(x1, width)):
            i = (row + x) * 4
            sr += rgba[i]
            sg += rgba[i + 1]
            sb += rgba[i + 2]
            count += 1
    if count == 0:
        return None
    return (sr / count, sg / count, sb / count)


def nonwhite_bbox(rgba, width, height, x0, y0, x1, y1):
    """区域内非白像素 bbox 与计数（位移/旋转轨迹口径）。"""
    count = 0
    min_x = min_y = 10**9
    max_x = max_y = -1
    for y in range(y0, min(y1, height)):
        row = y * width
        for x in range(x0, min(x1, width)):
            i = (row + x) * 4
            if rgba[i] >= 248 and rgba[i + 1] >= 248 and rgba[i + 2] >= 248:
                continue
            count += 1
            min_x = min(min_x, x)
            max_x = max(max_x, x)
            min_y = min(min_y, y)
            max_y = max(max_y, y)
    if count == 0:
        return None
    return (count, min_x, min_y, max_x, max_y)


def classify_cell(mean):
    """回执格分类：白=none（未发生），绿=pass，红=fail（落空/重复/不合格）。"""
    if mean is None:
        return "none"
    r, g, b = mean
    if r >= 248 and g >= 248 and b >= 248:
        return "none"
    if g >= 140 and r < 100:
        return "pass"
    if r >= 140 and g < 100 and b < 100:
        return "fail"
    return "other"


def main():
    frames_dir = Path(sys.argv[1])
    manifest = frames_dir / "manifest.jsonl"
    if not manifest.is_file():
        print(f"no manifest at {manifest}")
        return 1
    lines = [json.loads(line) for line in manifest.read_text().splitlines() if line.strip()]
    print(f"frames={len(lines)}")
    if not lines:
        return 0
    t0 = lines[0]["wall_ms"]
    prev_ms = None
    hashes = set()
    cell_first = {}
    cells_x = ((24, 88, "hit-a"), (112, 176, "hit-b"), (200, 264, "hit-c"),
               (288, 352, "ev-cancel"), (376, 440, "phase-resume"))
    shift_series = []  # (t_ms, shift_px)
    pause_series = []  # (t_ms, center_x or None)
    hit_series = []  # (t_ms, center_x or None)
    cancel_series = []  # (t_ms, bbox tuple or None)
    print("seq frame_id wall_ms dt_ms sha12 hit_cx pause_cx cancel_bbox strip_px "
          "cells(hit-a/hit-b/hit-c/ev-cancel/phase-resume)")
    for entry in lines:
        pixels = (frames_dir / entry["file"]).read_bytes()
        sha12 = hashlib.sha256(pixels).hexdigest()[:12]
        hashes.add(sha12)
        width, height = entry["width"], entry["height"]
        dt = entry["wall_ms"] - prev_ms if prev_ms is not None else 0
        prev_ms = entry["wall_ms"]
        t = entry["wall_ms"] - t0
        hit = nonwhite_bbox(pixels, width, height, 0, 170, 700, 310)
        pause = nonwhite_bbox(pixels, width, height, 0, 320, 460, 450)
        cancel = nonwhite_bbox(pixels, width, height, 460, 300, 700, 430)
        strip = solid_mean(pixels, width, height, 24, 470, 152, 486)
        shift = None
        if strip:
            shift = max(-10.0, min(250.0, (255.0 - strip[1]) / 240.0 * 240.0))
            shift_series.append((t, shift))
        hit_cx = None
        if hit:
            hit_cx = (hit[1] + hit[3]) / 2
            hit_series.append((t, hit_cx))
        pause_cx = None
        if pause:
            pause_cx = (pause[1] + pause[3]) / 2
            pause_series.append((t, pause_cx))
        cancel_series.append((t, cancel))
        cells = []
        for x0, x1, name in cells_x:
            cls = classify_cell(solid_mean(pixels, width, height, x0, 508, x1, 524))
            cells.append(cls)
            key = (name, cls)
            if cls != "none" and key not in cell_first:
                cell_first[key] = t
        hc = "-" if hit_cx is None else f"{hit_cx:.0f}"
        pc = "-" if pause_cx is None else f"{pause_cx:.0f}"
        cb = "-" if cancel is None else f"{cancel[1]}..{cancel[3]}x{cancel[2]}..{cancel[4]}"
        sp = "-" if shift is None else f"{shift:.0f}"
        print(f"{entry['seq']} {entry['frame_id']} {t} {dt} {sha12} {hc} {pc} {cb} {sp} | {'/'.join(cells)}")
    print(f"distinct_frame_hashes={len(hashes)}")
    for (name, cls), at_ms in sorted(cell_first.items(), key=lambda kv: kv[1]):
        print(f"cell {name} {cls} first_seen_ms={at_ms}")
    # hit-box 轨迹：越出基线右缘 424 的帧计数（视觉分叉证据）。
    beyond = [t for t, cx in hit_series if cx and cx > 428]
    if hit_series:
        cxs = [cx for _, cx in hit_series if cx]
        print(f"hit_center_min={min(cxs):.0f} max={max(cxs):.0f} frames_beyond_baseline={len(beyond)}")
    # pause-box 冻结段：连续帧 bbox 中心 x 恒定（±1px）的最长段。
    runs = []
    start = None
    last = None
    for t, cx in pause_series:
        if cx is None:
            continue
        if last is not None and abs(cx - last) <= 1.0:
            if start is None:
                start = prev_t
        else:
            if start is not None:
                runs.append((start, prev_t))
            start = None
        last = cx
        prev_t = t
    if start is not None:
        runs.append((start, prev_t))
    if runs:
        best = max(runs, key=lambda r: r[1] - r[0])
        print(f"pause_frozen_longest_run_ms={best[1] - best[0]:.0f} window=({best[0]:.0f},{best[1]:.0f})")
    else:
        print("pause_frozen_longest_run_ms=0")
    # 条带位移：冻结段值与恢复段首值（页内 phase-resume 格的旁路复核）。
    # 时间窗取结构化识别的 bbox 恒定段（best run），恢复段 = 段尾后 800ms——
    # 不依赖 fixture 时间线常量（时间线变更无需改本脚本）。
    frozen_samples = []
    resumed_samples = []
    if runs:
        best_run = max(runs, key=lambda r: r[1] - r[0])
        frozen_samples = [s for t, s in shift_series if best_run[0] <= t <= best_run[1]]
        resumed_samples = [s for t, s in shift_series if best_run[1] < t <= best_run[1] + 800]
    if frozen_samples and resumed_samples:
        frozen_end = frozen_samples[-1]
        resumed_first = resumed_samples[0]
        print(f"strip_frozen_end={frozen_end:.0f}px strip_resumed_first={resumed_first:.0f}px "
              f"strip_resume_delta={abs(resumed_first - frozen_end):.0f}px")
    else:
        print("strip_resume_delta=unavailable")
    return 0


if __name__ == "__main__":
    sys.exit(main())
