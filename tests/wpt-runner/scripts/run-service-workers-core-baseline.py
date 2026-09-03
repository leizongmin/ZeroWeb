#!/usr/bin/env python3
"""Run and verify the pinned Service Worker core baseline twice."""

from __future__ import annotations

import argparse
import collections
import json
import subprocess
import sys
from pathlib import Path

EXPECTED_CASES = 65
EXPECTED_SUBTESTS = 249


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--runner", required=True, type=Path)
    parser.add_argument("--wpt-data", required=True, type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--summary", type=Path)
    return parser.parse_args()


def run_once(runner: Path, wpt_data: Path, filter: str | None = None) -> list:
    command = [
        str(runner),
        "testharness-service-workers",
        "--wpt-data",
        str(wpt_data),
        "--json",
    ]
    if filter is not None:
        command.append(filter)
    completed = subprocess.run(command, capture_output=True, text=True, check=False)
    if completed.returncode not in (0, 1):
        raise RuntimeError(
            f"runner exited {completed.returncode}: {completed.stderr.strip()}"
        )
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"runner emitted invalid JSON: {error}") from error


def load_manifest() -> list[str]:
    manifest_path = Path(__file__).resolve().parents[1] / "src" / "testharness.rs"
    source = manifest_path.read_text(encoding="utf-8")
    start = source.index("pub const SERVICE_WORKER_CORE_CASES")
    start = source.index("[", start)
    end = source.index("];", start)
    cases = []
    for line in source[start:end].splitlines():
        line = line.strip()
        if not line.startswith('"'):
            continue
        cases.append(line.split('"', 2)[1])
    if len(cases) != EXPECTED_CASES:
        raise RuntimeError(f"expected {EXPECTED_CASES} manifest cases, got {len(cases)}")
    if len(cases) != len(set(cases)):
        raise RuntimeError("manifest contains duplicate case paths")
    return cases


def run_isolated(runner: Path, wpt_data: Path) -> list:
    results = []
    for case in load_manifest():
        case_results = run_once(runner, wpt_data, case)
        if len(case_results) != 1 or case_results[0][0] != case:
            raise RuntimeError(f"runner emitted unexpected results for {case}: {case_results}")
        results.extend(case_results)
    return results


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
        short_cases = [
            f"{case}={len(results)}"
            for case, results in cases
            if len(results) <= 1
        ]
        failures = [
            f"{case}: {result['name']}={result['status']} {result.get('message') or ''}".strip()
            for case, results in cases
            for result in results
            if result.get("status") != "Pass"
        ]
        detail = "; ".join(short_cases[:8] + failures[:5])
        raise RuntimeError(f"expected {EXPECTED_SUBTESTS} subtests, got {subtests}: {detail}")
    paths = [case for case, _ in cases]
    if len(paths) != len(set(paths)):
        raise RuntimeError("runner emitted duplicate case paths")


def validate_all_pass(entries: list[tuple[str, str, str]]) -> None:
    failures = [entry for entry in entries if entry[2] != "Pass"]
    if failures:
        sample = "; ".join(
            f"{case}: {name}={status}" for case, name, status in failures[:5]
        )
        raise RuntimeError(
            f"expected all {len(entries)} subtests to Pass, "
            f"found {len(failures)} non-Pass: {sample}"
        )


def render_markdown(summary: dict) -> str:
    status = summary["status"]
    return "\n".join(
        [
            "# Service Worker Core WPT Baseline",
            "",
            "- Date: 2026-09-03",
            "- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`",
            f"- Cases: {summary['cases']}",
            f"- Subtests: {summary['subtests']}",
            f"- Pass: {status.get('Pass', 0)}",
            f"- Fail: {status.get('Fail', 0)}",
            f"- Timeout: {status.get('Timeout', 0)}",
            f"- Unsupported: {status.get('Unsupported', 0)}",
            f"- Deterministic: {str(summary['deterministic']).lower()}",
            "",
            "## Scope",
            "",
            "This pinned Service Worker core baseline covers registration, update, "
            "install/activate lifecycle, worker-global APIs, messaging, module "
            "script loading, skipWaiting, and updateViaCache cases. "
            "`ServiceWorkerGlobalScope/registration-attribute.https.html` extends "
            "the baseline with worker-global `registration.scope`, "
            "`registration.installing/waiting/active` slot visibility, "
            "registration and worker `EventTarget` methods, `updatefound`, and "
            "`statechange` ordering across initial and replacement workers. "
            "`ServiceWorkerGlobalScope/service-worker-error-event.https.html` "
            "extends it with worker-global `ErrorEvent` dispatch for message "
            "handler failures. "
            "`ServiceWorkerGlobalScope/error-message-event.https.html` extends "
            "it with worker-global `messageerror` dispatch for unserializable "
            "page messages. `controller-on-load.https.html` extends it with "
            "newly loaded controlled iframe controller projection and "
            "iframe-realm registration worker identity. "
            "`getregistration.https.html` extends it with same-origin "
            "document URL validation, fragment-insensitive lookup, and "
            "unregistered controlled iframe discovery. "
            "`registration-iframe.https.html` extends it with iframe-global "
            "scriptURL/scope parsing and immediate installing worker slot "
            "visibility. `installing.https.html` extends it with the top-level "
            "registration installing slot visibility and SameObject identity "
            "across `getRegistration()`. `waiting.https.html` extends it with "
            "top-level and iframe registration waiting slot visibility, "
            "controller nullability, and waiting/active SameObject identity. "
            "`controller-on-disconnect.https.html` extends it with clearing "
            "the Service Worker controller when an iframe global is detached. "
            "`oninstall-script-error.https.html` extends it with install "
            "listener exception reporting while preserving lifecycle success "
            "unless a `waitUntil()` promise rejects. "
            "`onactivate-script-error.https.html` extends it with activate "
            "listener exception reporting while preserving lifecycle success. "
            "`extendable-event-waituntil.https.html` extends it with "
            "ExtendableEvent lifetime promise all-settled ordering and "
            "activate waitUntil rejection non-blocking semantics. "
            "`extendable-event-async-waituntil.https.html` extends it with "
            "ExtendableEvent async waitUntil task/microtask eligibility and "
            "FetchEvent respondWith lifetime extension boundaries. "
            "`controller-on-reload.https.html` extends it with iframe reload "
            "controller acquisition and iframe-realm ServiceWorker identity.",
        ]
    ) + "\n"


def main() -> int:
    args = parse_args()
    if not args.runner.is_file():
        raise RuntimeError(f"runner does not exist: {args.runner}")
    if not args.wpt_data.is_dir():
        raise RuntimeError(f"WPT data root does not exist: {args.wpt_data}")

    first = run_isolated(args.runner, args.wpt_data)
    second = run_isolated(args.runner, args.wpt_data)
    validate_shape(first)
    validate_shape(second)
    first_normalized = normalized(first)
    second_normalized = normalized(second)
    if first_normalized != second_normalized:
        raise RuntimeError("case/subtest/status baseline changed between consecutive runs")
    validate_all_pass(first_normalized)

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
        args.summary.write_text(render_markdown(summary), encoding="utf-8")
    sys.stdout.write(rendered)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as error:
        print(f"service-worker-core-baseline: {error}", file=sys.stderr)
        raise SystemExit(1)
