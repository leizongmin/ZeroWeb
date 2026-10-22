#!/usr/bin/env python3
"""M4 GUI tab 切换腿分段 CPU 统计。

输入 sampler 全程采样（samples.jsonl）+ browser.log 的 UI_DRIVER_STEP log 段锚点
（phaseN-<name>），按锚点把墙钟切成段，统计各段各进程 CPU（jiffies 增量 → cores）。
多 renderer 实例按 sampler 首现顺序编号 renderer-1/2（与 browser.log [renderer-N]
前缀的启动序对应）。

用法：background-resource-phase-stats.py <samples.jsonl> <browser.log> [anchors.txt]
  anchors.txt（xdotool 形态）：每行 `<epoch_ms> <phase-name>`；缺省从 browser.log
  解析 UI_DRIVER_STEP log= 行（ui-driver 形态）。
"""
import json
import re
import sys
from datetime import datetime, timezone

ANSI = re.compile(r"\x1b\[[0-9;]*m")
ANCHOR = re.compile(r"UI_DRIVER_STEP step=\d+ log=(phase\d+-[\w-]+)")


def parse_anchor_ms(line: str) -> tuple[str, int] | None:
    m = ANCHOR.search(line)
    if not m:
        return None
    ts = ANSI.sub("", line).strip()[:24]  # RFC3339 带毫秒前缀
    dt = datetime.strptime(ts, "%Y-%m-%dT%H:%M:%S.%f").replace(tzinfo=timezone.utc)
    return m.group(1), int(dt.timestamp() * 1000)


def main(samples_path: str, log_path: str, anchors_path: str | None = None) -> None:
    anchors: list[tuple[str, int]] = []
    if anchors_path:
        with open(anchors_path, encoding="utf-8") as f:
            for line in f:
                parts = line.split()
                if len(parts) == 2:
                    anchors.append((parts[1], int(parts[0])))
    else:
        with open(log_path, encoding="utf-8", errors="replace") as f:
            for line in f:
                parsed = parse_anchor_ms(line)
                if parsed:
                    anchors.append(parsed)
    if not anchors:
        sys.exit("no phase anchors found")

    samples: list[dict] = []
    with open(samples_path, encoding="utf-8") as f:
        for line in f:
            row = json.loads(line)
            if row.get("procs"):
                samples.append(row)
    if not samples:
        sys.exit("no samples")

    # 进程身份：pid → 稳定标签（renderer 多实例按首现序编号）。
    labels: dict[int, str] = {}
    renderer_seq = 0
    for row in samples:
        for p in row["procs"]:
            if p["pid"] not in labels:
                if p["comm"] == "zero-renderer":
                    renderer_seq += 1
                    labels[p["pid"]] = f"renderer-{renderer_seq}"
                else:
                    labels[p["pid"]] = p["comm"]

    # 段边界：锚点 t_ms（含 wait_idle 加载段为 phase0 前置，不计段）。
    bounds = [(name, ms) for name, ms in anchors]
    phases = []
    for i, (name, start_ms) in enumerate(bounds):
        end_ms = bounds[i + 1][1] if i + 1 < len(bounds) else samples[-1]["t_ms"]
        phases.append((name, start_ms, end_ms))

    def in_phase(t_ms: int, start: int, end: int) -> bool:
        return start <= t_ms < end

    print(f"{'phase':28} {'proc':14} {'dur_s':>6} {'cpu_s':>7} {'cores':>7} {'n':>4}")
    for name, start_ms, end_ms in phases:
        if end_ms <= start_ms:
            continue
        # 段内首末 jiffies 差分（进程 jiffies 单调累积；段内任一样本缺失按 0 起算）。
        first: dict[int, int] = {}
        last: dict[int, int] = {}
        n = 0
        for row in samples:
            if not in_phase(row["t_ms"], start_ms, end_ms):
                continue
            n += 1
            seen = set()
            for p in row["procs"]:
                seen.add(p["pid"])
                if p["pid"] not in first:
                    first[p["pid"]] = p["cpu_jiffies"]
                last[p["pid"]] = p["cpu_jiffies"]
            for pid in first:
                if pid not in seen:
                    last[pid] = last.get(pid, first[pid])
        dur_s = (end_ms - start_ms) / 1000
        for pid in sorted(first, key=lambda x: labels[x]):
            delta = last[pid] - first[pid]
            user_hz = 100
            cpu_s = delta / user_hz
            cores = cpu_s / dur_s if dur_s else 0
            print(f"{name:28} {labels[pid]:14} {dur_s:6.1f} {cpu_s:7.2f} {cores:7.3f} {n:4}")


if __name__ == "__main__":
    if len(sys.argv) not in (3, 4):
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2], sys.argv[3] if len(sys.argv) == 4 else None)
