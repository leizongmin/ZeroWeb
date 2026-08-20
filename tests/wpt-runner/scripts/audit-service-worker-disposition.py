#!/usr/bin/env python3
"""Build or verify the pinned Service Worker WPT disposition contract."""

from __future__ import annotations

import argparse
import csv
import io
import re
from collections import Counter
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[2]
EVIDENCE_DIR = REPO_ROOT / "docs/goal/service-workers/evidence"
INVENTORY = EVIDENCE_DIR / "2026-08-19-m0-wpt-case-inventory.tsv"
CANDIDATE_REVIEW = (
    EVIDENCE_DIR / "2026-08-19-m0-m1-candidate-resource-closure.tsv"
)
IDL_REVIEW = EVIDENCE_DIR / "2026-08-19-idlharness-review.md"
CONTRACT = EVIDENCE_DIR / "2026-08-19-wpt-disposition.tsv"
WPT_REVISION = "04067ce9c7c2165e71ad7d0dde10a4c5cb394a83"
IMPORTED_LEDGER = REPO_ROOT / "tests/wpt-runner/imported-testharness.txt"
CORE_ASSET_MANIFESTS = [
    EVIDENCE_DIR / "2026-08-19-m1-tier-a-assets.tsv",
    EVIDENCE_DIR / "2026-08-19-m1-next-wave-assets.tsv",
    EVIDENCE_DIR / "2026-08-19-worker-global-static-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-update-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-import-response-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-import-dynamic-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-import-event-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-module-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-module-bytecheck-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-module-cors-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-module-registration-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-module-type-update-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-module-request-metadata-assets.tsv",
    EVIDENCE_DIR / "2026-08-20-m3-update-via-cache-matrix-assets.tsv",
    EVIDENCE_DIR / "2026-08-21-m3-dynamic-import-update-assets.tsv",
    EVIDENCE_DIR / "2026-08-21-m3-update-failure-assets.tsv",
]
REVIEW_FILES = [
    EVIDENCE_DIR / "2026-08-19-m1-next-wave-review.tsv",
    EVIDENCE_DIR / "2026-08-19-m1-iframe-review.tsv",
    EVIDENCE_DIR / "2026-08-19-m1-message-review.tsv",
    EVIDENCE_DIR / "2026-08-19-m1-final-review.tsv",
    EVIDENCE_DIR / "2026-08-19-static-routing-review.tsv",
    EVIDENCE_DIR / "2026-08-19-worker-global-import-review.tsv",
    EVIDENCE_DIR / "2026-08-19-navigation-review.tsv",
    EVIDENCE_DIR / "2026-08-19-request-response-review.tsv",
    EVIDENCE_DIR / "2026-08-19-final-remaining-review.tsv",
]
IDL_SOURCE = "service-workers/idlharness.https.any.js"
EXPECTED_SOURCE_COUNT = 294
EXPECTED_URL_COUNT = 331
EXPECTED_LANES = Counter(core=30, defer=49, gated=173, skip=42)
SHA_PATTERN = re.compile(r"^[0-9a-f]{40}$")


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as stream:
        return list(csv.DictReader(stream, delimiter="\t"))


def unique_by(
    rows: list[dict[str, str]], key: str, label: str
) -> dict[str, dict[str, str]]:
    result: dict[str, dict[str, str]] = {}
    for row in rows:
        value = row[key]
        if value in result:
            raise ValueError(f"duplicate {label}: {value}")
        result[value] = row
    return result


def classify_review(decision: str) -> str:
    if decision.startswith("next-wave-") and decision.endswith("core"):
        return "core"
    for lane in ("defer", "gated", "skip"):
        if decision.startswith(f"{lane}-"):
            return lane
    raise ValueError(f"unrecognized review decision: {decision}")


def audit_core_inputs(
    core_sources: set[str], inventory: dict[str, dict[str, str]]
) -> None:
    imported: dict[str, str] = {}
    with IMPORTED_LEDGER.open(encoding="utf-8") as stream:
        for line_number, line in enumerate(stream, start=1):
            fields = line.split()
            if not fields or not fields[0].startswith("service-workers/"):
                continue
            if len(fields) != 4:
                raise ValueError(
                    f"malformed Service Worker import at line {line_number}"
                )
            source, revision, _date, _milestone = fields
            if source in imported:
                raise ValueError(f"duplicate Service Worker import: {source}")
            imported[source] = revision
    if set(imported) != core_sources:
        raise ValueError("runner imports do not match core disposition sources")
    for source, revision in imported.items():
        if revision != WPT_REVISION:
            raise ValueError(f"runner import has wrong revision: {source}")

    asset_cases: dict[str, str] = {}
    for path in CORE_ASSET_MANIFESTS:
        for row in read_tsv(path):
            if row["manifest_type"] != "testharness" or row["roles"] != "case":
                continue
            source = row["path"]
            if source in asset_cases:
                raise ValueError(f"duplicate core case asset: {source}")
            asset_cases[source] = row["git_blob_sha"]
    if set(asset_cases) != core_sources:
        raise ValueError("core case assets do not match core disposition sources")
    for source, blob_sha in asset_cases.items():
        if blob_sha != inventory[source]["manifest_sha"]:
            raise ValueError(f"core case asset SHA does not match inventory: {source}")


def build_contract() -> tuple[str, Counter[str]]:
    if not IDL_REVIEW.is_file():
        raise ValueError(f"IDL review evidence not found: {IDL_REVIEW.name}")

    inventory_rows = read_tsv(INVENTORY)
    inventory = unique_by(inventory_rows, "source_path", "inventory source")
    if len(inventory) != EXPECTED_SOURCE_COUNT:
        raise ValueError(
            f"inventory source count: expected {EXPECTED_SOURCE_COUNT}, "
            f"found {len(inventory)}"
        )

    candidate_rows = read_tsv(CANDIDATE_REVIEW)
    candidates = unique_by(candidate_rows, "case_path", "candidate source")
    review: dict[str, tuple[str, str]] = {}
    for path in REVIEW_FILES:
        for row in read_tsv(path):
            source = row["source_path"]
            if source in review:
                raise ValueError(f"duplicate reviewed source: {source}")
            review[source] = (row["decision"], path.name)
    review[IDL_SOURCE] = ("defer-mixed-idlharness", IDL_REVIEW.name)

    expected_candidates = {
        source
        for source, row in inventory.items()
        if row["disposition"] == "candidate"
    }
    expected_review = {
        source
        for source, row in inventory.items()
        if row["disposition"] == "review"
    }
    if set(candidates) != expected_candidates:
        raise ValueError("candidate closure does not match inventory candidate sources")
    if set(review) != expected_review:
        raise ValueError("review ledgers do not match inventory review sources")

    candidate_lanes = {
        "keep-first": "core",
        "defer-after-core": "defer",
        "remove-first": "gated",
    }
    output_rows: list[dict[str, str]] = []
    lanes: Counter[str] = Counter()
    total_urls = 0
    for source, row in sorted(inventory.items()):
        manifest_sha = row["manifest_sha"]
        if not SHA_PATTERN.fullmatch(manifest_sha):
            raise ValueError(f"invalid manifest SHA for {source}: {manifest_sha}")
        try:
            generated_urls = int(row["generated_urls"])
        except ValueError as error:
            raise ValueError(f"invalid generated URL count for {source}") from error
        if generated_urls < 1:
            raise ValueError(f"generated URL count must be positive for {source}")

        disposition = row["disposition"]
        if disposition == "gated":
            lane = "gated"
            decision = "gated-initial-inventory"
            evidence = INVENTORY.name
        elif disposition == "candidate":
            candidate = candidates[source]
            decision = candidate["decision"]
            try:
                lane = candidate_lanes[decision]
            except KeyError as error:
                raise ValueError(
                    f"unrecognized candidate decision for {source}: {decision}"
                ) from error
            evidence = CANDIDATE_REVIEW.name
        elif disposition == "review":
            decision, evidence = review[source]
            lane = classify_review(decision)
        else:
            raise ValueError(f"unrecognized inventory disposition: {disposition}")

        lanes[lane] += 1
        total_urls += generated_urls
        output_rows.append(
            {
                "source_path": source,
                "revision": WPT_REVISION,
                "manifest_sha": manifest_sha,
                "generated_urls": str(generated_urls),
                "lane": lane,
                "decision": decision,
                "evidence": evidence,
            }
        )

    if total_urls != EXPECTED_URL_COUNT:
        raise ValueError(
            f"generated URL count: expected {EXPECTED_URL_COUNT}, found {total_urls}"
        )
    if lanes != EXPECTED_LANES:
        raise ValueError(f"lane counts: expected {EXPECTED_LANES}, found {lanes}")

    core_sources = {
        row["source_path"] for row in output_rows if row["lane"] == "core"
    }
    audit_core_inputs(core_sources, inventory)

    output = io.StringIO(newline="")
    writer = csv.DictWriter(
        output,
        fieldnames=[
            "source_path",
            "revision",
            "manifest_sha",
            "generated_urls",
            "lane",
            "decision",
            "evidence",
        ],
        delimiter="\t",
        lineterminator="\n",
    )
    writer.writeheader()
    writer.writerows(output_rows)
    return output.getvalue(), lanes


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--write",
        action="store_true",
        help="replace the committed contract with the derived content",
    )
    args = parser.parse_args()

    try:
        expected, lanes = build_contract()
        if args.write:
            CONTRACT.write_text(expected, encoding="utf-8")
        elif not CONTRACT.is_file():
            raise ValueError(f"contract not found: {CONTRACT.relative_to(REPO_ROOT)}")
        elif CONTRACT.read_text(encoding="utf-8") != expected:
            raise ValueError("committed disposition contract is stale")
    except (OSError, ValueError) as error:
        print(f"Service Worker WPT disposition audit: FAIL: {error}")
        return 1

    summary = " ".join(f"{lane}={lanes[lane]}" for lane in sorted(lanes))
    action = "wrote" if args.write else "verified"
    print(
        f"Service Worker WPT disposition audit: PASS "
        f"({action} {EXPECTED_SOURCE_COUNT} sources / "
        f"{EXPECTED_URL_COUNT} URLs; {summary})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
