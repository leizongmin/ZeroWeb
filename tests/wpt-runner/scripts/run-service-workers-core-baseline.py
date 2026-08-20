#!/usr/bin/env python3
"""Run and verify the pinned Service Worker core baseline twice."""

from __future__ import annotations

import argparse
import collections
import json
import subprocess
import sys
from pathlib import Path

EXPECTED_CASES = 14
EXPECTED_SUBTESTS = 38


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--runner", required=True, type=Path)
    parser.add_argument("--wpt-data", required=True, type=Path)
    parser.add_argument("--output", type=Path)
    return parser.parse_args()


def run_once(runner: Path, wpt_data: Path) -> list:
    command = [
        str(runner),
        "testharness-service-workers",
        "--wpt-data",
        str(wpt_data),
        "--json",
    ]
    completed = subprocess.run(command, capture_output=True, text=True, check=False)
    if completed.returncode not in (0, 1):
        raise RuntimeError(
            f"runner exited {completed.returncode}: {completed.stderr.strip()}"
        )
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"runner emitted invalid JSON: {error}") from error


def normalized(cases: list) -> list[tuple[str, str, str]]:
    return [
        (case, result["name"], result["status"])
        for case, results in cases
        for result in results
    ]


def validate_shape(cases: list) -> None:
    if len(cases) != EXPECTED_CASES:
        raise RuntimeError(f"expected {EXPECTED_CASES} cases, got {len(cases)}")
    subtests = sum(len(results) for _, results in cases)
    if subtests != EXPECTED_SUBTESTS:
        raise RuntimeError(f"expected {EXPECTED_SUBTESTS} subtests, got {subtests}")
    paths = [case for case, _ in cases]
    if len(paths) != len(set(paths)):
        raise RuntimeError("runner emitted duplicate case paths")


def main() -> int:
    args = parse_args()
    if not args.runner.is_file():
        raise RuntimeError(f"runner does not exist: {args.runner}")
    if not args.wpt_data.is_dir():
        raise RuntimeError(f"WPT data root does not exist: {args.wpt_data}")

    first = run_once(args.runner, args.wpt_data)
    second = run_once(args.runner, args.wpt_data)
    validate_shape(first)
    validate_shape(second)
    first_normalized = normalized(first)
    second_normalized = normalized(second)
    if first_normalized != second_normalized:
        raise RuntimeError("case/subtest/status baseline changed between consecutive runs")

    counts = collections.Counter(status for _, _, status in first_normalized)
    summary = {
        "cases": len(first),
        "subtests": len(first_normalized),
        "status": dict(sorted(counts.items())),
        "deterministic": True,
        "results": first,
    }
    rendered = json.dumps(summary, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    sys.stdout.write(rendered)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as error:
        print(f"service-worker-core-baseline: {error}", file=sys.stderr)
        raise SystemExit(1)
