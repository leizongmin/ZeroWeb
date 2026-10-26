#!/usr/bin/env python3
"""background-resource-lifecycle goal M4 资源门禁：腿指标提取 + 阈值配对导出 + 门禁检查。

输入是 background-resource-observe.sh 产物腿目录（run-config.txt + samples.jsonl +
probe.json [+ fixture-stats.json]）。三种模式：

  report  <leg-dir>...            打印各腿指标（JSON）
  derive  <baseline.json> <leg-dir>...
                                  按场景分组从配对腿导出阈值基线（配对腿清单写入
                                  provenance；n=1 场景标 pairing_insufficient）
  check   <baseline.json> <leg-dir>...
                                  腿指标对照基线逐项判定；任一 FAIL 退出 1，结构
                                  错误（场景/数据缺失）退出 2

阈值导出规则（与 docs/perf/baselines 记录纪律同族）：
  CPU/进度率类上界 = 配对腿 max × 1.5；RSS 上界 = max + 64 MiB；
  「合法工作不误停」下界（probe 计数/率）= min × 0.5。
  修改基线须更新配对腿并写明理由，禁止为通过门禁调阈值（config 同族政策）。

环境一致性：check 模式比对 run-config 的 ZW_* 开关与 gui 标志（display 除外），
与基线 env 不一致 FAIL——防错版门禁静默放行；不一致不短路指标判定（两类失败
并列输出）。run-config schema v1（M1/dc1 时代）未记录 ZW_COMPOSITOR_ANIMATIONS
与 gui 两字段，当时运行时默认即 0——env 归一化对缺失键回填默认值。
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

HZ_DEFAULT = 100.0
CPU_MARGIN = 1.5
RSS_MARGIN_MIB = 64.0
WORK_FLOOR_MARGIN = 0.5
KIB = 1024.0
ENV_DEFAULTS = {"ZW_COMPOSITOR_ANIMATIONS": "0", "gui": "0"}


def leg_env(config: dict) -> dict:
    """run-config 字段 → 门禁 env 快照（ZW_* + gui，display 除外；v1 缺失键回填默认）。"""
    env = {k: v for k, v in config.items() if k.startswith("ZW_") or k == "gui"}
    return {**ENV_DEFAULTS, **env}


def parse_run_config(path: Path) -> dict:
    fields: dict[str, str] = {}
    for line in path.read_text().splitlines():
        for token in line.split():
            if "=" in token:
                key, value = token.split("=", 1)
                fields[key] = value
    return fields


def read_samples(leg: Path) -> tuple[float, list[dict]]:
    """返回 (采样窗口时长 s, 进程样本行列表)。"""
    rows: list[dict] = []
    hz = HZ_DEFAULT
    for line in (leg / "samples.jsonl").read_text().splitlines():
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        if row.get("event") == "sampler_start":
            hz = float(row.get("user_hz", HZ_DEFAULT))
            continue
        if "procs" in row:
            rows.append(row)
    if len(rows) < 2:
        raise SystemExit(f"{leg}: samples.jsonl 进程样本不足")
    t0, t1 = rows[0]["t_ms"], rows[-1]["t_ms"]
    dt_s = (t1 - t0) / 1000.0
    return dt_s, [{**row, "_hz": hz} for row in rows]


def primary_pid(rows: list[dict], comm: str) -> int | None:
    """首现该 comm 的 pid（renderer 主实例口径，同历史分析）。"""
    for row in rows:
        for proc in row["procs"]:
            if proc["comm"] == comm:
                return proc["pid"]
    return None


def process_stats(rows: list[dict], pid: int) -> dict:
    first_jiffies = last_jiffies = None
    first_rss = last_rss = hwm = None
    t_first = t_last = None
    for row in rows:
        for proc in row["procs"]:
            if proc["pid"] != pid:
                continue
            if first_jiffies is None:
                first_jiffies, first_rss, t_first = proc["cpu_jiffies"], proc["rss_kib"], row["t_ms"]
            last_jiffies, last_rss, t_last = proc["cpu_jiffies"], proc["rss_kib"], row["t_ms"]
            hwm = proc["hwm_kib"] if hwm is None else max(hwm, proc["hwm_kib"])
    dt_s = (t_last - t_first) / 1000.0
    hz = rows[0]["_hz"]
    return {
        "cpu_cores": (last_jiffies - first_jiffies) / hz / dt_s if dt_s > 0 else 0.0,
        "rss_first_mib": first_rss / KIB,
        "rss_last_mib": last_rss / KIB,
        "rss_hwm_mib": hwm / KIB,
    }


def probe_metrics(probe: dict) -> tuple[dict, int]:
    """probe.json → (计数/率指标, 退出结构错误码 0/2)。"""
    metrics: dict[str, float] = {}
    series = probe.get("samples") or []
    by_label: dict[str, list[float]] = {}
    for sample in series:
        by_label.setdefault(sample.get("label", "?"), []).append(float(sample.get("value", 0)))
    for label, values in by_label.items():
        if len(values) >= 2:
            delta = values[-1] - values[0]
            metrics[f"probe_delta:{label}"] = delta
            # bg02-media v2（media-runtime-isolation）：回卷安全行程指标——时钟
            # 序列（bg02-media-audio-clock 等）跨流末回绕时末减首净位移可为 0/
            # 负（采样相位落点），对「时钟推进」判定盲；路径长度（相邻差绝对值
            # 累计）冻结序列恒 0、推进/回卷序列随时长增长，工作下界设此口径。
            travel = sum(abs(b - a) for a, b in zip(values, values[1:]))
            metrics[f"probe_travel:{label}"] = round(travel, 4)
    nav = probe.get("nav_series") or []
    if nav:
        ok = sum(1 for item in nav if item.get("nav_ok"))
        metrics["nav_ok_ratio"] = ok / len(nav)
        # stall 连接统计 = fixture server 累计计数，末次快照即终态（M2-S1 验收信号：
        # 导航放弃挂起 fetch 后连接是否被对端关闭）。
        stats = nav[-1].get("stats") or {}
        total = stats.get("stall_total")
        closed = stats.get("stall_closed_by_peer")
        if total:
            metrics["stall_closed_ratio"] = closed / total
    reopen = probe.get("reopen_series") or []
    if reopen:
        metrics["close_error_count"] = float(
            sum(1 for item in reopen if item.get("close_error"))
        )
    return metrics, 0


def leg_metrics(leg: Path) -> tuple[dict, dict, int]:
    """腿目录 → (指标, run-config 字段, 结构错误码)。"""
    config = parse_run_config(leg / "run-config.txt")
    scenario = config.get("scenario")
    if not scenario:
        return {}, config, 2
    dt_s, rows = read_samples(leg)
    metrics: dict[str, float] = {}
    for comm, prefix in (("zero-renderer", "renderer"), ("zero-browser", "browser"), ("zero-compositor", "compositor")):
        pid = primary_pid(rows, comm)
        if pid is None:
            continue
        stats = process_stats(rows, pid)
        metrics[f"{prefix}_cpu_cores"] = round(stats["cpu_cores"], 4)
        metrics[f"{prefix}_rss_last_mib"] = round(stats["rss_last_mib"], 1)
        metrics[f"{prefix}_rss_hwm_mib"] = round(stats["rss_hwm_mib"], 1)
    probe_path = leg / "probe.json"
    if probe_path.is_file():
        probe = json.loads(probe_path.read_text())
        probe_m, code = probe_metrics(probe)
        metrics.update(probe_m)
        if code:
            return metrics, config, code
        # 计数 → 率（墙钟归一），供跨墙钟配对（probe wall 与 run-config wall 同源）。
        wall_s = float(probe.get("wall_ms") or 0) / 1000.0
        if wall_s > 0:
            for name, value in list(probe_m.items()):
                if name.startswith("probe_delta:"):
                    metrics[name.replace("probe_delta:", "probe_rate:") + "/s"] = round(value / wall_s, 2)
    return metrics, config, 0


def derive_bounds(values: list[float], kind: str) -> dict:
    if kind == "rss":
        return {"max": round(max(values) + RSS_MARGIN_MIB, 1)}
    if kind == "floor":
        floor = min(values) * WORK_FLOOR_MARGIN
        return {"min": round(floor, 4)} if floor > 0 else {"min": 0.0001}
    return {"max": round(max(values) * CPU_MARGIN, 4)}


# 场景 → (指标名, 导出类别)。cpu=上界 max×1.5 / rss=上界 max+64MiB / floor=下界 min×0.5。
METRIC_KINDS = {
    "renderer_cpu_cores": "cpu",
    "browser_cpu_cores": "cpu",
    "compositor_cpu_cores": "cpu",
    "renderer_rss_last_mib": "rss",
    "browser_rss_last_mib": "rss",
    "compositor_rss_last_mib": "rss",
    "renderer_rss_hwm_mib": "rss",
    "nav_ok_ratio": "floor",
    "stall_closed_ratio": "floor",
}
# 布尔/计数类不经导出，check 时按固定界判定。
FIXED_BOUNDS = {
    "close_error_count": {"max": 0},
}


def cmd_report(legs: list[Path]) -> int:
    report = {}
    for leg in legs:
        metrics, config, code = leg_metrics(leg)
        report[str(leg)] = {
            "scenario": config.get("scenario"),
            "wall_s": config.get("wall", "").rstrip("s"),
            "env": leg_env(config),
            "metrics": metrics,
            "error": code,
        }
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0


def cmd_derive(out_path: Path, legs: list[Path], *, sha: str, date: str, note: str) -> int:
    grouped: dict[str, list[tuple[str, dict, dict]]] = {}
    for leg in legs:
        metrics, config, code = leg_metrics(leg)
        scenario = config.get("scenario")
        if code or not scenario:
            print(f"derive: 跳过结构错误腿 {leg} (code={code})", file=sys.stderr)
            continue
        grouped.setdefault(scenario, []).append((str(leg), metrics, config))

    baseline: dict = {
        "schema_version": 1,
        "kind": "background-resource-baseline",
        "platform_class": "linux-x86_64",
        "recorded_at": date,
        "git_sha": sha,
        "justification": note,
        "platform": "linux-x86_64",
        "policy": (
            f"CPU/率上界=配对腿max×{CPU_MARGIN}；RSS上界=max+{RSS_MARGIN_MIB:.0f}MiB；"
            f"工作下界=min×{WORK_FLOOR_MARGIN}。修改配对须更新 pairing 清单并写明理由，"
            "禁止为通过门禁调阈值。"
        ),
        "scenarios": {},
    }
    for scenario, entries in sorted(grouped.items()):
        names = sorted({k for _, m, _ in entries for k in m})
        bounds: dict = {}
        for name in names:
            values = [m[name] for _, m, _ in entries if name in m]
            if name in FIXED_BOUNDS:
                bounds[name] = dict(FIXED_BOUNDS[name])
                continue
            if name.startswith("probe_delta:") and f"probe_travel:{name.split(':', 1)[1]}" in names:
                # 回卷安全 travel 界存在时取代 delta 界：单调计数器两者数值恒等
                # （无损失）；时钟序列跨流末回绕时末减首净位移随采样相位可为 0/
                # 负（autoplay-b 实测 -0.003）——delta 界对该序列不健全。
                continue
            if name.startswith("probe_delta:") or name.startswith("probe_travel:"):
                # probe 计数差值是「合法工作不误停」信号（rAF 泵/迭代计数），
                # 墙钟同源配对下设下界——跌破即工作被误停。travel（路径长度）
                # 同口径：时钟序列的回卷安全推进下界（bg02-media v2）。
                bounds[name] = derive_bounds(values, "floor")
                continue
            kind = METRIC_KINDS.get(name)
            if kind is None:
                continue  # probe_rate 等派生率指标只记录不设界（界设在 probe_delta 口径）
            bounds[name] = derive_bounds(values, kind)
        baseline["scenarios"][scenario] = {
            "pairing": [leg for leg, _, _ in entries],
            "n": len(entries),
            "observed": {leg: m for leg, m, _ in entries},
            "env": leg_env(entries[0][2]),
            "bounds": bounds,
            "pairing_insufficient": len(entries) < 2,
        }
    out_path.write_text(json.dumps(baseline, ensure_ascii=False, indent=2) + "\n")
    print(f"derive: {len(baseline['scenarios'])} 场景 → {out_path}")
    for scenario, entry in baseline["scenarios"].items():
        flag = " [pairing_insufficient]" if entry["pairing_insufficient"] else ""
        print(f"  {scenario}: n={entry['n']}{flag} bounds={sorted(entry['bounds'])}")
    return 0


def cmd_check(baseline_path: Path, legs: list[Path]) -> int:
    baseline = json.loads(baseline_path.read_text())
    failures: list[str] = []
    for leg in legs:
        metrics, config, code = leg_metrics(leg)
        scenario = config.get("scenario")
        entry = baseline["scenarios"].get(scenario) if scenario else None
        if code or entry is None:
            failures.append(f"FAIL {leg}: 结构错误或基线无场景 {scenario} (code={code})")
            continue
        env = leg_env(config)
        insufficient = entry.get("pairing_insufficient")
        leg_failures: list[str] = []
        if env != entry.get("env"):
            # 错版门禁（开关组合与基线不同版）不短路指标判定——两类失败并列，
            # 负例腿（历史修复前数据）的指标突破仍然可见。
            leg_failures.append(
                f"env 与基线不一致 {env} vs {entry.get('env')}（错版门禁）"
            )
        for name, bound in entry["bounds"].items():
            if name not in metrics:
                leg_failures.append(f"{name}: 腿数据缺失")
                continue
            value = metrics[name]
            if "min" in bound and value < bound["min"]:
                leg_failures.append(f"{name}={value} < min {bound['min']}")
            if "max" in bound and value > bound["max"]:
                leg_failures.append(f"{name}={value} > max {bound['max']}")
        verdict = "FAIL" if leg_failures else "PASS"
        note = " [pairing_insufficient——阈值仅参考]" if insufficient else ""
        print(f"{verdict} {leg}{note}")
        for failure in leg_failures:
            print(f"    {failure}")
        if leg_failures:
            failures.append(f"{leg}: {'; '.join(leg_failures)}")
    if failures:
        print(f"check: {len(failures)} 项未过（详见上）", file=sys.stderr)
        return 1
    print("check: 全部通过")
    return 0


def main() -> int:
    argv = sys.argv[1:]
    if len(argv) >= 2 and argv[0] == "report":
        return cmd_report([Path(p) for p in argv[1:]])
    if len(argv) >= 3 and argv[0] == "derive":
        rest = argv[2:]
        sha = date = ""
        note = ""
        i = 0
        pos: list[str] = []
        while i < len(rest):
            if rest[i] == "--sha":
                sha = rest[i + 1]; i += 2
            elif rest[i] == "--date":
                date = rest[i + 1]; i += 2
            elif rest[i] == "--note":
                note = rest[i + 1]; i += 2
            else:
                pos.append(rest[i]); i += 1
        if not (sha and date and note):
            print("derive: --sha/--date/--note 必填（基线记录纪律：justification 必填）", file=sys.stderr)
            return 2
        return cmd_derive(Path(argv[1]), [Path(p) for p in pos], sha=sha, date=date, note=note)
    if len(argv) >= 3 and argv[0] == "check":
        return cmd_check(Path(argv[1]), [Path(p) for p in argv[2:]])
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
