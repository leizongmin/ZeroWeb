#!/usr/bin/env python3
"""M1 基线分析：汇总各观测腿的进程 CPU 率、RSS 趋势、呈现帧数与场景序列。

输入：本目录下各 <scenario>/ 子目录（observe.sh 产物）。
输出：summary.json（机器可读）——判定与叙述由 ANALYSIS.md 承载。
"""
from __future__ import annotations

import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
USER_HZ = os.sysconf("SC_CLK_TCK")


def load_samples(leg: str) -> list[dict]:
    path = os.path.join(HERE, leg, "samples.jsonl")
    out = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            d = json.loads(line)
            if "procs" in d:
                out.append(d)
    return out


def proc_series(samples: list[dict], comm: str) -> dict | None:
    """取指定 comm 进程（取 cpu_jiffies 最大的 pid 为主实例）的时间序列。"""
    per_pid: dict[int, list[tuple[int, int, int | None]]] = {}
    for s in samples:
        for p in s["procs"]:
            if p["comm"] == comm:
                per_pid.setdefault(p["pid"], []).append((s["t_ms"], p["cpu_jiffies"], p["rss_kib"]))
    if not per_pid:
        return None
    pid, series = max(per_pid.items(), key=lambda kv: kv[1][-1][1])
    if len(series) < 2:
        return None
    t0, c0, r0 = series[0]
    t1, c1, r1 = series[-1]
    span_s = (t1 - t0) / 1000.0
    peak_rss = max((r for _, _, r in series if r is not None), default=None)
    return {
        "pid": pid,
        "instances_seen": len(per_pid),
        "window_s": round(span_s, 2),
        "cpu_jiffies": c1 - c0,
        "cpu_cores": round((c1 - c0) / USER_HZ / span_s, 4) if span_s > 0 else None,
        "rss_first_kib": r0,
        "rss_last_kib": r1,
        "rss_peak_kib": peak_rss,
    }


def frame_count(leg: str) -> int | None:
    path = os.path.join(HERE, leg, "frames", "manifest.jsonl")
    if not os.path.exists(path):
        return None
    with open(path, encoding="utf-8") as f:
        return sum(1 for line in f if line.strip())


def probe_json(leg: str) -> dict | None:
    path = os.path.join(HERE, leg, "probe.json")
    if not os.path.exists(path):
        return None
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def main() -> None:
    legs = sorted(
        d for d in os.listdir(HERE)
        if os.path.isdir(os.path.join(HERE, d)) and os.path.exists(os.path.join(HERE, d, "samples.jsonl"))
    )
    summary: dict = {"legs": {}, "user_hz": USER_HZ}
    for leg in legs:
        samples = load_samples(leg)
        entry: dict = {
            "renderer": proc_series(samples, "zero-renderer"),
            "browser": proc_series(samples, "zero-browser"),
            "compositor": proc_series(samples, "zero-compositor"),
            "renderer_instances": len({p["pid"] for s in samples for p in s["procs"] if p["comm"] == "zero-renderer"}),
            "frames_dumped": frame_count(leg),
        }
        probe = probe_json(leg)
        if probe:
            if "samples" in probe:
                entry["page_samples"] = probe["samples"]
            if "nav_series" in probe:
                nav = probe["nav_series"]
                entry["nav_cycles"] = len(nav)
                entry["nav_ok"] = sum(1 for n in nav if n.get("nav_ok"))
                entry["stats_first"] = nav[0].get("stats") if nav else None
                entry["stats_last"] = nav[-1].get("stats") if nav else None
                entry["stall_active_series"] = [n.get("stats", {}).get("stall_active") for n in nav if n.get("stats")]
                entry["stall_closed_series"] = [n.get("stats", {}).get("stall_closed_by_peer") for n in nav if n.get("stats")]
            if "reopen_series" in probe:
                ro = probe["reopen_series"]
                entry["reopen_cycles"] = len(ro)
                entry["reopen_destroyed"] = sum(1 for r in ro if r.get("destroyed_event"))
                entry["reopen_errors"] = sum(1 for r in ro if r.get("error") or r.get("close_error"))
                entry["final_targets"] = probe.get("final_targets")
        summary["legs"][leg] = entry

    # bg03：renderer RSS vs 导航 cycle 时间对齐
    bg03 = summary["legs"].get("bg03-nav", {})
    if bg03.get("nav_cycles"):
        probe = probe_json("bg03-nav")
        cycles = probe["nav_series"]
        samples = load_samples("bg03-nav")
        renderer_ids = {p["pid"] for s in samples for p in s["procs"] if p["comm"] == "zero-renderer"}
        rss_at = {}
        for n in cycles:
            t = n.get("nav_ms") and None  # nav_ms 是相对量，用 cycle 顺序对齐采样尾部
        # 简化：renderer RSS 采样序列首/每 1/4/尾
        rss_seq = [p["rss_kib"] for s in samples for p in s["procs"] if p["comm"] == "zero-renderer" and p["rss_kib"]]
        if rss_seq:
            n = len(rss_seq)
            entry = {
                "rss_first": rss_seq[0],
                "rss_q1": rss_seq[n // 4],
                "rss_half": rss_seq[n // 2],
                "rss_q3": rss_seq[3 * n // 4],
                "rss_last": rss_seq[-1],
                "rss_max": max(rss_seq),
            }
            bg03["renderer_rss_progression_kib"] = entry
    summary["legs"]["bg03-nav"] = bg03

    out = os.path.join(HERE, "summary.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=1, ensure_ascii=False)
    print(f"summary written: {out}")
    print(f"legs: {', '.join(legs)}")


if __name__ == "__main__":
    sys.exit(main())
