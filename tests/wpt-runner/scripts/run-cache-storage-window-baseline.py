#!/usr/bin/env python3
"""Run and verify the pinned CacheStorage window baseline twice."""

from __future__ import annotations

import argparse
import collections
import json
import subprocess
import sys
from pathlib import Path

EXPECTED_CASES = 4


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--runner", required=True, type=Path)
    parser.add_argument("--wpt-data", required=True, type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--summary", type=Path)
    return parser.parse_args()


def run_once(runner: Path, wpt_data: Path) -> list:
    command = [
        str(runner),
        "testharness-cache-storage",
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
    if subtests == 0:
        raise RuntimeError("runner emitted zero subtests")
    paths = [case for case, _ in cases]
    if len(paths) != len(set(paths)):
        raise RuntimeError("runner emitted duplicate case paths")


def markdown_summary(summary: dict) -> str:
    lines = [
        "# CacheStorage Window WPT Baseline",
        "",
        "- Date: 2026-08-22",
        "- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`",
        f"- Cases: {summary['cases']}",
        f"- Subtests: {summary['subtests']}",
        f"- Deterministic double run: {str(summary['deterministic']).lower()}",
        "",
        "## Status Counts",
        "",
        "| Status | Count |",
        "|---|---:|",
    ]
    for status, count in summary["status"].items():
        lines.append(f"| {status} | {count} |")
    lines.extend(
        [
            "",
            "## Notes",
            "",
            "This is the first pinned window-environment CacheStorage baseline. "
            "Failures are preserved as baseline data for follow-up semantic work; "
            "the script only requires the case set and status mapping to be stable "
            "between consecutive runs.",
            "",
            "## Non-Pass Subtests",
            "",
            "| Case | Subtest | Status | Message |",
            "|---|---|---|---|",
        ]
    )
    non_pass = [
        (case, result)
        for case, results in summary["results"]
        for result in results
        if result["status"] != "Pass"
    ]
    if non_pass:
        for case, result in non_pass:
            message = str(result.get("message") or "").replace("\n", " ")
            message = message.replace("|", "\\|")
            lines.append(
                f"| `{case}` | {result['name']} | {result['status']} | {message} |"
            )
    else:
        lines.append("| _none_ | _none_ | Pass |  |")
    lines.append("")
    return "\n".join(lines)


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
    if args.summary:
        args.summary.parent.mkdir(parents=True, exist_ok=True)
        args.summary.write_text(markdown_summary(summary), encoding="utf-8")
    sys.stdout.write(rendered)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as error:
        print(f"cache-storage-window-baseline: {error}", file=sys.stderr)
        raise SystemExit(1)
