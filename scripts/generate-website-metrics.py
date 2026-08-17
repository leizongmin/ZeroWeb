#!/usr/bin/env python3
"""Build the small, deterministic data file used by the metrics web page."""

from __future__ import annotations

import argparse
import csv
import json
from collections import OrderedDict
from pathlib import Path


PERFORMANCE_METRICS = OrderedDict(
    (
        ("page/welcome/total_ms", ("Welcome page", "欢迎页总耗时")),
        ("page/medium/total_ms", ("Medium page", "中型页面总耗时")),
        ("page/morning/total_ms", ("Morning test page", "Morning 测试页总耗时")),
        ("startup_ms", ("Browser startup", "浏览器启动耗时")),
        ("resource/peak_rss_mb", ("Peak memory", "峰值内存")),
    )
)


def data_rows(path: Path) -> list[list[str]]:
    with path.open(encoding="utf-8", newline="") as source:
        return [row for row in csv.reader(line for line in source if not line.startswith("#")) if row]


def build_metrics(performance_csv: Path, wpt_csv: Path) -> dict[str, object]:
    wpt_by_month: dict[str, dict[str, object]] = {}
    for row in data_rows(wpt_csv):
        if len(row) < 8:
            continue
        date, mode, ref, total, passed, rate, _extra, sha, *_ = row
        if mode != "upstream":
            continue
        wpt_by_month[date[:7]] = {
            "period": date[:7],
            "date": date,
            "total": int(total),
            "passed": int(passed),
            "rate": float(rate),
            "ref": ref,
            "sha": sha,
        }

    performance_by_metric: dict[str, dict[str, dict[str, object]]] = {
        metric_id: {} for metric_id in PERFORMANCE_METRICS
    }
    units: dict[str, str] = {}
    for row in data_rows(performance_csv):
        if len(row) < 8:
            continue
        date, platform, metric_id, _p50, p95, _maximum, unit, sha, *_ = row
        if platform != "github-ubuntu-latest" or metric_id not in performance_by_metric or not p95:
            continue
        performance_by_metric[metric_id][date] = {"date": date, "value": float(p95), "sha": sha}
        units[metric_id] = unit

    performance = []
    for metric_id, labels in PERFORMANCE_METRICS.items():
        points = performance_by_metric[metric_id]
        performance.append(
            {
                "id": metric_id,
                "label": {"en": labels[0], "zh": labels[1]},
                "unit": units.get(metric_id, ""),
                "lower_is_better": True,
                "points": [points[date] for date in sorted(points)],
            }
        )

    all_dates = [item["date"] for item in wpt_by_month.values()]
    all_dates.extend(point["date"] for metric in performance for point in metric["points"])
    return {
        "schema_version": 1,
        "latest_data_date": max(all_dates, default=None),
        "wpt": [wpt_by_month[month] for month in sorted(wpt_by_month)],
        "performance": {"platform_class": "github-ubuntu-latest", "metrics": performance},
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--performance", type=Path, default=Path("docs/perf/trends/benchmark-trend.csv"))
    parser.add_argument("--wpt", type=Path, default=Path("docs/goal/rendering-compat/evidence/wpt-trends/trend.csv"))
    parser.add_argument("--output", type=Path, default=Path("website/metrics.json"))
    args = parser.parse_args()
    payload = build_metrics(args.performance, args.wpt)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
