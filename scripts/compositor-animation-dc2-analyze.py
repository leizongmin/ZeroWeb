#!/usr/bin/env python3
"""DC-2（COMP-02）帧序列分析：从 frame_dump 产物提取查询一致性与事件回执证据
（S2c 证据工具，零第三方依赖）。

用法：python3 scripts/compositor-animation-dc2-analyze.py <frames-dir> [> analysis.txt]

对 manifest.jsonl 每帧计算：
- 红盒（#opacity-box 128² @(24,32)）均值绿通道 → 反解合成侧不透明度
  o_pixel = (255 − mean_g) / 225（alpha 混白底线性混合）；
- gCS 条带（#q-strip 128×16 @(24,340)，fixture 每 50ms 把查询值编码为
  灰度 255 − round(opacity × 240)）→ 反解查询侧不透明度 o_strip；
- 同帧相位差 dphase = |o_strip − o_pixel|（RFC §3：断言用相位区间非逐帧相等，
  容差 0.15 覆盖 Get 往返 + 条带发布滞后 + 边缘反锯齿偏差）；
- 事件回执条四格（y∈[400,416)：busy x[24,88) / idle x[112,176) /
  replay x[200,264) / own x[288,352)）→ none/pass/fail 分类与首见时刻；
- 有限动画方块（#end-busy @(24,440) 64²、#end-idle @(112,440) 64²）均值
  （淡出可见性佐证）；蓝盒非白 bbox（transform 连续性佐证，同 DC-1）；
- 负载 marker 行与 nav-marker 行内容哈希（负载窗 / 导航换代定位）。
"""

import hashlib
import json
import sys
from pathlib import Path

PHASE_TOL = 0.15


def solid_mean(rgba, width, height, x0, y0, x1, y1):
    """区域全像素均值（实心 div 区域，不做白底过滤）。"""
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
    """区域内非白像素 bbox（旋转相位佐证，同 DC-1 口径）。"""
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
        return "count=0"
    return f"count={count} bbox=({min_x},{min_y})-({max_x},{max_y})"


def classify_bar(mean):
    """回执条分类：白=none（未发生），绿=pass（恰一次且 elapsedTime 合格），
    红=fail（重复/不合格），品红=qfail（查询异常），其余=other。"""
    if mean is None:
        return "none"
    r, g, b = mean
    if r >= 248 and g >= 248 and b >= 248:
        return "none"
    if g >= 140 and r < 100:
        return "pass"
    if r >= 140 and g < 100 and b < 100:
        return "fail"
    if r >= 140 and b >= 140 and g < 100:
        return "qfail"
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
    marker_hashes = {}
    nav_hashes = {}
    bar_first = {}
    deltas = []
    steady_deltas = []
    print(
        "seq frame_id wall_ms dt_ms sha12 o_pixel o_strip dphase "
        "end_busy end_idle bars(busy/idle/replay/own) blue marker12 nav12"
    )
    for entry in lines:
        pixels = (frames_dir / entry["file"]).read_bytes()
        sha12 = hashlib.sha256(pixels).hexdigest()[:12]
        hashes.add(sha12)
        width, height = entry["width"], entry["height"]
        dt = entry["wall_ms"] - prev_ms if prev_ms is not None else 0
        prev_ms = entry["wall_ms"]
        # 红盒 → 合成侧 opacity（绿通道摆幅 225 最大，抗边缘偏差最稳）。
        red = solid_mean(pixels, width, height, 24, 32, 152, 160)
        o_pixel = max(-0.05, min(1.05, (255.0 - red[1]) / 225.0)) if red else None
        # gCS 条带 → 查询侧 opacity（灰度编码 255 − 240×o）。
        strip = solid_mean(pixels, width, height, 24, 340, 152, 356)
        o_strip = max(-0.05, min(1.05, (255.0 - strip[1]) / 240.0)) if strip else None
        dphase = abs(o_strip - o_pixel) if (o_strip is not None and o_pixel is not None) else None
        if dphase is not None:
            deltas.append(dphase)
            # 稳态对：o_pixel < 0.999 排除内容帧提交瞬态（S1c 语义：新内容帧先以
            # 基线栅格呈现一 tick、下一 Get 才重采样——实测表现为 o_pixel=1.000
            # 尖刺，与条带扫相无关）；动画相位断言取稳态采样帧。
            if o_pixel < 0.999:
                steady_deltas.append(dphase)
        end_busy = solid_mean(pixels, width, height, 24, 440, 88, 504)
        end_idle = solid_mean(pixels, width, height, 112, 440, 176, 504)
        bars = []
        for x0, x1 in ((24, 88), (112, 176), (200, 264), (288, 352)):
            cls = classify_bar(solid_mean(pixels, width, height, x0, 400, x1, 416))
            bars.append(cls)
            key = (x0, cls)
            if cls != "none" and key not in bar_first:
                bar_first[key] = entry["wall_ms"] - t0
        blue = nonwhite_bbox(pixels, width, height, 160, 40, 380, 260)
        marker = pixels[(288 * width + 24) * 4 : (306 * width + 250) * 4]
        marker12 = hashlib.sha256(marker).hexdigest()[:12]
        marker_hashes.setdefault(marker12, entry["wall_ms"] - t0)
        nav = pixels[(520 * width + 24) * 4 : (538 * width + 150) * 4]
        nav12 = hashlib.sha256(nav).hexdigest()[:12]
        nav_hashes.setdefault(nav12, entry["wall_ms"] - t0)
        eb = "-" if end_busy is None else f"rgb({end_busy[0]:.0f},{end_busy[1]:.0f},{end_busy[2]:.0f})"
        ei = "-" if end_idle is None else f"rgb({end_idle[0]:.0f},{end_idle[1]:.0f},{end_idle[2]:.0f})"
        op = "-" if o_pixel is None else f"{o_pixel:.3f}"
        ost = "-" if o_strip is None else f"{o_strip:.3f}"
        dp = "-" if dphase is None else f"{dphase:.3f}"
        print(
            f"{entry['seq']} {entry['frame_id']} {entry['wall_ms'] - t0} {dt} {sha12} "
            f"{op} {ost} {dp} {eb} {ei} {'/'.join(bars)} | {blue} | {marker12} {nav12}"
        )
    print(f"distinct_frame_hashes={len(hashes)}")
    print(f"distinct_marker_signatures={len(marker_hashes)} distinct_nav_signatures={len(nav_hashes)}")
    for marker12, at_ms in sorted(marker_hashes.items(), key=lambda kv: kv[1]):
        print(f"marker {marker12} first_seen_ms={at_ms}")
    for nav12, at_ms in sorted(nav_hashes.items(), key=lambda kv: kv[1]):
        print(f"nav {nav12} first_seen_ms={at_ms}")
    for (x0, cls), at_ms in sorted(bar_first.items(), key=lambda kv: kv[1]):
        slot = {24: "busy", 112: "idle", 200: "replay", 288: "own"}[x0]
        print(f"bar {slot} {cls} first_seen_ms={at_ms}")
    if deltas:
        deltas.sort()
        median = deltas[len(deltas) // 2]
        within = sum(1 for d in deltas if d <= PHASE_TOL)
        print(
            f"phase_pairs={len(deltas)} median_dphase={median:.3f} max_dphase={max(deltas):.3f} "
            f"within_{PHASE_TOL}={within}/{len(deltas)}"
        )
    else:
        print("phase_pairs=0 (no frame with both red box and strip derivable)")
    if steady_deltas:
        steady_deltas.sort()
        smedian = steady_deltas[len(steady_deltas) // 2]
        swithin = sum(1 for d in steady_deltas if d <= PHASE_TOL)
        print(
            f"steady_pairs={len(steady_deltas)} median_dphase={smedian:.3f} max_dphase={max(steady_deltas):.3f} "
            f"within_{PHASE_TOL}={swithin}/{len(steady_deltas)}"
        )
    else:
        print("steady_pairs=0")
    return 0


if __name__ == "__main__":
    sys.exit(main())
