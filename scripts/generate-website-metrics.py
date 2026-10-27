#!/usr/bin/env python3
"""Build the small, deterministic data file used by the metrics web page."""

from __future__ import annotations

import argparse
import csv
import json
import re
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

# docs/compat/trends/wpt-suites.csv 的 15 列（顺序固定；字段内禁止半角逗号，见该文件头注释）
SUITES_COLUMNS = (
    "date", "suite_id", "name_en", "name_zh", "wpt_dirs", "harness", "unit",
    "total", "passed", "rate", "status", "goal", "evidence", "note_en", "note_zh",
)
STATUS_RANK = {"active": 0, "completed": 1, "planned": 2}
HARNESS_VALUES = {"reftest", "testharness", "mixed"}
UNIT_VALUES = {"tests", "subtests", "files"}


def parse_int(value: str, label: str) -> int | None:
    if not value:
        return None
    try:
        return int(value)
    except ValueError as error:
        raise ValueError(f"{label}: 期望整数，得 {value!r}") from error


def build_suites(path: Path) -> list[dict[str, object]]:
    latest: dict[str, tuple[int, list[str]]] = {}
    for index, row in enumerate(data_rows(path), start=1):
        if len(row) != len(SUITES_COLUMNS):
            raise ValueError(
                f"{path}: 数据行 {index}: 期望 {len(SUITES_COLUMNS)} 列，得 {len(row)}"
                "（字段内出现半角逗号会错位，见文件头注释第 4 条）"
            )
        record = dict(zip(SUITES_COLUMNS, row))
        date = record["date"]
        if date and not re.fullmatch(r"\d{4}-\d{2}-\d{2}", date):
            raise ValueError(f"{path}: 数据行 {index}: date 期望 YYYY-MM-DD，得 {date!r}")
        if record["status"] not in STATUS_RANK:
            raise ValueError(f"{path}: 数据行 {index}: status 非法 {record['status']!r}")
        if record["harness"] not in HARNESS_VALUES:
            raise ValueError(f"{path}: 数据行 {index}: harness 非法 {record['harness']!r}")
        if record["unit"] not in UNIT_VALUES:
            raise ValueError(f"{path}: 数据行 {index}: unit 非法 {record['unit']!r}")
        total = parse_int(record["total"], f"{path}: 数据行 {index} total")
        passed = parse_int(record["passed"], f"{path}: 数据行 {index} passed")
        if record["status"] == "planned":
            if total is not None or passed is not None:
                raise ValueError(f"{path}: 数据行 {index}: planned 行 total/passed 必须留空")
        else:
            if total is None or passed is None:
                raise ValueError(f"{path}: 数据行 {index}: {record['status']} 行 total/passed 必填")
            if not date:
                raise ValueError(f"{path}: 数据行 {index}: {record['status']} 行 date 必填")
            if passed > total:
                raise ValueError(f"{path}: 数据行 {index}: passed {passed} > total {total}")
            if record["rate"]:
                expected = round(passed / total * 100, 1)
                if abs(float(record["rate"]) - expected) > 0.05:
                    raise ValueError(
                        f"{path}: 数据行 {index}: rate {record['rate']} 与 passed/total 重算值 {expected} 不一致"
                    )
        # 同 suite_id 取最新一行（date 更晚优先；同日取文件序靠后者）
        if record["suite_id"] not in latest or (date, index) > latest[record["suite_id"]][:2]:
            latest[record["suite_id"]] = (date, index, row)

    suites = []
    for _, _, row in latest.values():
        record = dict(zip(SUITES_COLUMNS, row))
        total = parse_int(record["total"], record["suite_id"])
        passed = parse_int(record["passed"], record["suite_id"])
        suites.append(
            {
                "id": record["suite_id"],
                "name": {"en": record["name_en"], "zh": record["name_zh"]},
                "dirs": [d for d in record["wpt_dirs"].split(";") if d],
                "harness": record["harness"],
                "unit": record["unit"],
                "status": record["status"],
                "goal": parse_int(record["goal"], record["suite_id"]),
                "total": total,
                "passed": passed,
                "rate": round(passed / total * 100, 1) if total is not None and passed is not None else None,
                "date": record["date"] or None,
                "evidence": record["evidence"],
                "note": {"en": record["note_en"], "zh": record["note_zh"]}
                if record["note_en"] or record["note_zh"]
                else None,
            }
        )
    suites.sort(
        key=lambda suite: (
            STATUS_RANK[suite["status"]],
            -(suite["rate"] if suite["rate"] is not None else -1.0),
            suite["id"],
        )
    )
    return suites


def data_rows(path: Path) -> list[list[str]]:
    with path.open(encoding="utf-8", newline="") as source:
        return [row for row in csv.reader(line for line in source if not line.startswith("#")) if row]


# coverage 快照 groups 的累计字段 → 网站缺口表字段（coverage-*.json 由
# tests/wpt-runner/scripts/wptfyi_gap.py coverage 产出，口径见
# docs/compat/wptfyi-gap-analysis.md「全量对账」节）
COVERAGE_GROUP_FIELDS = (
    "den_files", "covered_files", "full_pass_files",
    "den_subtests", "zw_passes", "zw_env_fails",
)
COVERAGE_GAP_TOP = 30


def build_coverage(wptfyi_dir: Path) -> dict[str, object] | None:
    """读 docs/compat/wptfyi/ 最新 coverage-*.json 快照（全量 WPT 对账口径）。

    metrics.json 只带总量、浏览器对照、topdir 聚合与缺口 top 30；二级目录
    的完整缺口表留在快照里，页面给原始数据链接。无快照时返回 None
    （首次部署 coverage job 未跑过属正常，页面隐藏对应区块）。
    """
    snaps = sorted(wptfyi_dir.glob("coverage-*.json"))
    if not snaps:
        return None
    data = json.loads(snaps[-1].read_text(encoding="utf-8"))
    totals = data.get("totals")
    if not isinstance(totals, dict) or not totals:
        return None

    topdirs: dict[str, dict[str, int]] = {}
    for key, stats in data.get("groups", {}).items():
        target = topdirs.setdefault(
            key.split("/")[0], {field: 0 for field in COVERAGE_GROUP_FIELDS}
        )
        for field in COVERAGE_GROUP_FIELDS:
            target[field] += int(stats.get(field, 0))
    gap_top = sorted(
        topdirs.items(),
        key=lambda kv: -(kv[1]["den_subtests"] - kv[1]["zw_passes"]),
    )[:COVERAGE_GAP_TOP]
    return {
        "date": snaps[-1].stem.removeprefix("coverage-"),
        "totals": totals,
        "browsers": data.get("browsers", {}),
        "topdirs": dict(sorted(topdirs.items())),
        "gap_top": [dict(dir_name=key, **stats) for key, stats in gap_top],
    }


def build_metrics(performance_csv: Path, wpt_csv: Path, suites_csv: Path) -> dict[str, object]:
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

    suites = build_suites(suites_csv)
    all_dates = [item["date"] for item in wpt_by_month.values()]
    all_dates.extend(point["date"] for metric in performance for point in metric["points"])
    all_dates.extend(suite["date"] for suite in suites if suite["date"])
    return {
        "schema_version": 2,
        "latest_data_date": max(all_dates, default=None),
        "wpt": [wpt_by_month[month] for month in sorted(wpt_by_month)],
        "suites": suites,
        "performance": {"platform_class": "github-ubuntu-latest", "metrics": performance},
    }


def build_metrics_with_coverage(
    performance_csv: Path, wpt_csv: Path, suites_csv: Path, wptfyi_dir: Path
) -> dict[str, object]:
    payload = build_metrics(performance_csv, wpt_csv, suites_csv)
    coverage = build_coverage(wptfyi_dir)
    if coverage is not None:
        payload["wptfyi"] = coverage
        if coverage["date"] > (payload["latest_data_date"] or ""):
            payload["latest_data_date"] = coverage["date"]
    return payload


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--performance", type=Path, default=Path("docs/perf/trends/benchmark-trend.csv"))
    parser.add_argument("--wpt", type=Path, default=Path("docs/goal/rendering-compat/evidence/wpt-trends/trend.csv"))
    parser.add_argument("--suites", type=Path, default=Path("docs/compat/trends/wpt-suites.csv"))
    parser.add_argument("--wptfyi-dir", type=Path, default=Path("docs/compat/wptfyi"))
    parser.add_argument("--output", type=Path, default=Path("website/metrics.json"))
    args = parser.parse_args()
    payload = build_metrics_with_coverage(args.performance, args.wpt, args.suites, args.wptfyi_dir)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
