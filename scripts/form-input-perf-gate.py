"""Validate a retained form-input performance report."""

from __future__ import annotations

import json
import pathlib
import sys


def fail(message: str) -> None:
    print(f"form-input-perf-gate: FAIL - {message}", file=sys.stderr)
    raise SystemExit(1)


if len(sys.argv) != 3:
    print("usage: form-input-perf-gate.py REPORT BASELINE_DIR", file=sys.stderr)
    raise SystemExit(2)

report_path = pathlib.Path(sys.argv[1])
baseline_dir = pathlib.Path(sys.argv[2])
if not report_path.is_file():
    print("form-input-perf-gate: missing report path", file=sys.stderr)
    raise SystemExit(2)

report = json.loads(report_path.read_text(encoding="utf-8"))
if report.get("profile") != "release":
    fail("report must use the release profile")

timing = report["input_to_publish_ms"]
counts = report["max_counts_per_input"]
hard_budget_ok = (
    timing["p95"] <= 20.0
    and report["jank_20ms_ratio"] <= 0.05
    and counts == {"parse": 0, "style": 0, "full_layout": 0, "paint": 1, "publish": 1}
)
if not hard_budget_ok:
    fail(f"hard smoothness or pipeline-count budget exceeded: timing={timing}, counts={counts}")

platform = report["platform_class"]
baseline_path = baseline_dir / f"form-input-{platform}.json"
if not baseline_path.is_file():
    print(f"form-input-perf-gate: PASS - {platform} hard budgets passed; no fixed-platform baseline yet")
    raise SystemExit(0)

baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
p95_budget = baseline["input_to_publish_ms"]["p95"] * 1.10 + 2.0
jank_budget = min(baseline["jank_20ms_ratio"] + 0.02, 0.05)
if timing["p95"] > p95_budget or report["jank_20ms_ratio"] > jank_budget:
    fail(
        f"platform baseline regression (p95={timing['p95']}/{p95_budget} ms, "
        f"jank={report['jank_20ms_ratio']}/{jank_budget})"
    )

print(
    f"form-input-perf-gate: PASS - {platform} p95={timing['p95']} ms "
    f"(budget={p95_budget}), jank={report['jank_20ms_ratio']} (budget={jank_budget}), "
    f"counts={counts['parse']}/{counts['style']}/{counts['full_layout']}/{counts['publish']}"
)
