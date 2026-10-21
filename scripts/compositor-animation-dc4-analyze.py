#!/usr/bin/env python3
"""DC-4（COMP-04a）帧序列分析：从 frame_dump 产物提取忙窗期呈现连续性与
terminate 冻结证据（S1d 证据工具，零第三方依赖；dc3-analyze 同款结构化识别
纪律——不依赖 fixture 时间线常量）。

用法：python3 scripts/compositor-animation-dc4-analyze.py <frames-dir> [> analysis.txt]

判定区域：#xfer-canvas（240×120 @(24,24)）→ x[24,264) y[24,144)。对每帧计算：
- 区域像素 sha（区域哈希序列）→ 变化事件（连续性）与冻结段（连续同哈希 run）；
- 全帧 sha distinct 数（页面整体活性对照）。

输出语义：
- region_change_events        区域变化时间点序列（t_ms sha12）；
- freeze runs（≥300ms）       区域恒定段；首帧 run（pre-paint 白屏）不计；
- busy_freeze_candidate       最长非终末冻结段（off 预期 ≈ 忙窗 900ms；on 预期无）；
- terminal_freeze             末段冻结（on 预期存在——terminate @6500 后定格；
                              off 预期无（或仅采集窗边界截断的短段）；
- distinct hashes             区域/全帧去重哈希数（on 区域应显著多于冻结对照）。
"""

import hashlib
import json
import sys
from pathlib import Path

REGION = (24, 24, 264, 144)
MIN_RUN_MS = 300.0


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
    x0, y0, x1, y1 = REGION
    t0 = lines[0]["wall_ms"]
    full_hashes = set()
    region_hashes = set()
    events = []  # (t_ms, sha12) 区域哈希变化事件
    series = []  # (t_ms, sha12)
    prev_sha = None
    print("seq t_ms dt_ms region_sha12 changed")
    for entry in lines:
        pixels = (frames_dir / entry["file"]).read_bytes()
        width, height = entry["width"], entry["height"]
        full_hashes.add(hashlib.sha256(pixels).hexdigest()[:12])
        row_bytes = height * width * 4
        # 区域行拷贝（避免全图切片）：逐行取 [x0,x1) 的 RGBA 字节拼接后哈希。
        h = hashlib.sha256()
        for y in range(y0, min(y1, height)):
            start = (y * width + x0) * 4
            h.update(pixels[start : start + (x1 - x0) * 4])
        sha = h.hexdigest()[:12]
        t = entry["wall_ms"] - t0
        dt = t - series[-1][0] if series else 0
        changed = sha != prev_sha
        if changed:
            events.append((t, sha))
        print(f"{entry['seq']} {t} {dt} {sha} {'Y' if changed else '.'}")
        series.append((t, sha))
        region_hashes.add(sha)
        prev_sha = sha

    # 冻结段：连续同哈希 run；段起点 = 该哈希首帧 t，终点 = 下一变化帧 t。
    runs = []
    for i, (t, sha) in enumerate(events):
        end = events[i + 1][0] if i + 1 < len(events) else series[-1][0]
        if end - t >= MIN_RUN_MS:
            runs.append((t, end, end - t, sha))
    # 首帧 run（pre-paint 稳定白屏）不计入判定段。
    judged = [r for r in runs if r[0] > 0]
    terminal = judged[-1] if judged and judged[-1][1] >= series[-1][0] else None
    mid = [r for r in judged if terminal is None or r[0] < terminal[0]]
    print(f"distinct_region_hashes={len(region_hashes)}")
    print(f"distinct_frame_hashes={len(full_hashes)}")
    print(f"region_change_events={len(events)}")
    print(f"freeze_runs_ge_{int(MIN_RUN_MS)}ms={len(judged)}")
    for start, end, dur, sha in judged:
        kind = "terminal" if terminal is not None and (start, end) == (terminal[0], terminal[1]) else "mid"
        print(f"freeze_run {kind} window=({start},{end}) dur_ms={dur:.0f} sha={sha}")
    if mid:
        best = max(mid, key=lambda r: r[2])
        print(f"busy_freeze_candidate window=({best[0]},{best[1]}) dur_ms={best[2]:.0f}")
    else:
        print("busy_freeze_candidate=none")
    if terminal is not None:
        print(f"terminal_freeze onset_ms={terminal[0]} dur_ms={terminal[2]:.0f} sha={terminal[3]}")
    else:
        print("terminal_freeze=none")
    return 0


if __name__ == "__main__":
    sys.exit(main())
