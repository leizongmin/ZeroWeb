#!/usr/bin/env python3
"""BG goal 进程树资源采样器：CPU jiffies + RSS/HWM，零第三方依赖。

用途：background-resource-lifecycle goal M1 基线观测。按固定间隔枚举
--root-pid 的全部后代进程（/proc/<pid>/stat ppid 递归），逐进程记录：
  cpu_jiffies  utime+stime（Linux USER_HZ=100 假设，样本间差分即用量）
  rss_kib      /proc/<pid>/status VmRSS
  hwm_kib      /proc/<pid>/status VmHWM（内核高水位）
进程出现/消失自然体现在相邻样本集合差异（BG-04 renderer 退出观测面）。

输出 JSONL（--out）：每行 {"t_ms", "root_pid", "procs":[...]}。
SIGTERM/SIGINT 时写 {"event":"sampler_stop"} 后退出。

用法：scripts/background-resource-sampler.py --root-pid <pid> --out <jsonl>
      [--interval-ms 250]
"""
from __future__ import annotations

import argparse
import json
import os
import signal
import time

USER_HZ = os.sysconf("SC_CLK_TCK") if hasattr(os, "sysconf") else 100
_running = True


def _stop(_sig, _frame) -> None:
    global _running
    _running = False


def read_tree(root_pid: int) -> list[dict]:
    """返回 root_pid 后代进程（含自身）的资源快照列表。"""
    children: dict[int, list[int]] = {}
    comm_of: dict[int, str] = {}
    cpu_of: dict[int, int] = {}
    state_of: dict[int, str] = {}
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        pid = int(entry)
        try:
            with open(f"/proc/{entry}/stat", "rb") as f:
                raw = f.read().decode("utf-8", "replace")
            # comm 可含空格与括号：取最后一个 ')' 之后的字段
            rparen = raw.rfind(")")
            head = raw[: rparen + 1]
            fields = raw[rparen + 2 :].split()
            comm = head[head.find("(") + 1 : -1]
            state = fields[0]
            ppid = int(fields[1])
            utime = int(fields[11])
            stime = int(fields[12])
        except (OSError, IndexError, ValueError):
            continue
        children.setdefault(ppid, []).append(pid)
        comm_of[pid] = comm
        cpu_of[pid] = utime + stime
        state_of[pid] = state

    procs: list[dict] = []
    stack = [root_pid]
    seen: set[int] = set()
    while stack:
        pid = stack.pop()
        if pid in seen or pid not in cpu_of:
            continue
        seen.add(pid)
        stack.extend(children.get(pid, []))
        rss_kib = None
        hwm_kib = None
        try:
            with open(f"/proc/{pid}/status", encoding="utf-8") as f:
                for line in f:
                    if line.startswith("VmRSS:"):
                        rss_kib = int(line.split()[1])
                    elif line.startswith("VmHWM:"):
                        hwm_kib = int(line.split()[1])
                    elif line.startswith("VmRSS") and rss_kib is not None and hwm_kib is not None:
                        break
        except OSError:
            pass
        procs.append(
            {
                "pid": pid,
                "comm": comm_of[pid],
                "state": state_of[pid],
                "cpu_jiffies": cpu_of[pid],
                "rss_kib": rss_kib,
                "hwm_kib": hwm_kib,
            }
        )
    procs.sort(key=lambda p: p["pid"])
    return procs


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root-pid", type=int, required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--interval-ms", type=int, default=250)
    args = ap.parse_args()

    signal.signal(signal.SIGTERM, _stop)
    signal.signal(signal.SIGINT, _stop)

    interval = args.interval_ms / 1000.0
    with open(args.out, "w", encoding="utf-8") as out:
        out.write(json.dumps({"event": "sampler_start", "t_ms": int(time.time() * 1000), "root_pid": args.root_pid, "interval_ms": args.interval_ms, "user_hz": USER_HZ}) + "\n")
        out.flush()
        while _running:
            t0 = time.monotonic()
            procs = read_tree(args.root_pid)
            out.write(json.dumps({"t_ms": int(time.time() * 1000), "root_pid": args.root_pid, "procs": procs}) + "\n")
            out.flush()
            delay = interval - (time.monotonic() - t0)
            if delay > 0:
                time.sleep(delay)
        out.write(json.dumps({"event": "sampler_stop", "t_ms": int(time.time() * 1000)}) + "\n")


if __name__ == "__main__":
    main()
