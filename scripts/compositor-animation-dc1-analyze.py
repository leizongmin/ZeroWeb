#!/usr/bin/env python3
"""DC-1（COMP-01）帧序列分析：从 frame_dump 产物生成归因摘要（S1e 证据工具）。

用法：python3 scripts/compositor-animation-dc1-analyze.py <frames-dir> [> analysis.txt]

对 manifest.jsonl 每帧计算：sha256、红盒（#opacity-box 区域）均值色（opacity 相位
归因：alpha 混白底，均值随采样值变化）、蓝盒（#transform-box）bbox（旋转相位归因）、
标记行（#load-marker 区域 y∈[288,306) x∈[24,250)）内容哈希（负载窗口定位：标记
文本只在片间更新）。零第三方依赖（hashlib/zlib/json 标准库）。
"""

import hashlib
import json
import sys
from pathlib import Path


def region_stats(rgba, width, height, x0, y0, x1, y1):
    """区域內「非白」像素统计：均值色 + bbox。白底判定 r/g/b 全 ≥248。"""
    count = 0
    sr = sg = sb = 0
    min_x = min_y = 10**9
    max_x = max_y = -1
    for y in range(y0, min(y1, height)):
        row = y * width
        for x in range(x0, min(x1, width)):
            i = (row + x) * 4
            r, g, b = rgba[i], rgba[i + 1], rgba[i + 2]
            if r >= 248 and g >= 248 and b >= 248:
                continue
            count += 1
            sr += r
            sg += g
            sb += b
            min_x = min(min_x, x)
            max_x = max(max_x, x)
            min_y = min(min_y, y)
            max_y = max(max_y, y)
    if count == 0:
        return "count=0"
    return (
        f"count={count} mean=({sr // count},{sg // count},{sb // count}) "
        f"bbox=({min_x},{min_y})-({max_x},{max_y})"
    )


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
    marker_hashes = {}
    print("seq frame_id wall_ms dt_ms sha12 red_box blue_box marker12")
    for entry in lines:
        pixels = (frames_dir / entry["file"]).read_bytes()
        sha12 = hashlib.sha256(pixels).hexdigest()[:12]
        hashes.add(sha12)
        width, height = entry["width"], entry["height"]
        dt = entry["wall_ms"] - prev_ms if prev_ms is not None else 0
        prev_ms = entry["wall_ms"]
        red = region_stats(pixels, width, height, 24, 32, 24 + 128, 32 + 128)
        # 蓝盒旋转 + translateX(36px) 外接可达中心 (264,144) ± ~104px；x 从 160 起
        # 避开红盒右缘（152），右/下留到 380/260 覆盖旋转包络。
        blue = region_stats(pixels, width, height, 160, 40, 380, 260)
        marker = pixels[(288 * width + 24) * 4 : (306 * width + 250) * 4]
        marker12 = hashlib.sha256(marker).hexdigest()[:12]
        marker_hashes.setdefault(marker12, entry["wall_ms"] - t0)
        print(
            f"{entry['seq']} {entry['frame_id']} {entry['wall_ms'] - t0} {dt} {sha12} "
            f"{red} | {blue} | {marker12}"
        )
    print(f"distinct_frame_hashes={len(hashes)} distinct_marker_signatures={len(marker_hashes)}")
    for marker12, at_ms in sorted(marker_hashes.items(), key=lambda kv: kv[1]):
        print(f"marker {marker12} first_seen_ms={at_ms}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
