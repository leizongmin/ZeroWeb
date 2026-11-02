#!/usr/bin/env python3
"""webcrypto 锚对照：逐 (case, subtest) 比较两份 testharness-webcrypto --json 账面。

webcrypto-compat-v3 每轮域切片与 M 末收口重定基的标准验证步骤——对基线锚
逐键对照，核查三查：P→非P = 0（零回归）、变化精确落靶案、键集零漂移。
替代逐轮手写一次性对照（M2-3b/M2-5 先例同口径），供切片落账与 DC-5 重定基复用。

输入：runner JSON 锚（`[[case_path, [{name, status, message}], ...], ...]`），
`.json` 与 `.json.gz` 均可。

用法：python3 scripts/webcrypto-anchor-diff.py <base.json[.gz]> <head.json[.gz]> [--json <out>]

输出语义：
- REGRESSED（P→非P）       回归清单，阻断项（退出码 1）；
- FIXED（非P→P）           修复清单，按 case 聚合计数；
- FLIPPED（非P→非P 字面变） F/T 互转等，记账观察项，不阻断；
- DRIFT                    键集漂移：case 级 lost/gained 与 case 内 subtest 级
                           lost/gained（阻断项，退出码 2；与回归并存时退出 1）。

status 记法：runner 字面 `Pass`/`Fail`/`Timeout`（WPT testharness 惯例），
缩写 P/F/T；出现其他字面按 UNKNOWN 报出并退出 3（runner 契约变化，对照
语义失效，不得静默放行）。
"""
import gzip
import json
import sys

PASS = "Pass"
KNOWN = {"Pass": "P", "Fail": "F", "Timeout": "T"}


def load(path):
    opener = gzip.open if path.endswith(".gz") else open
    with opener(path, "rt", encoding="utf-8") as f:
        raw = json.load(f)
    book = {}
    for case, subtests in raw:
        book[case] = {s["name"]: s.get("status", "?") for s in subtests}
    return book


def short(status):
    return KNOWN.get(status, "?")


def main():
    args = [a for a in sys.argv[1:] if a != "--json"]
    out_json = sys.argv[sys.argv.index("--json") + 1] if "--json" in sys.argv else None
    if len(args) != 2:
        sys.exit(__doc__)
    base, head = load(args[0]), load(args[1])

    unknown = {s for book in (base, head) for subs in book.values() for s in subs.values()
               if s not in KNOWN}
    if unknown:
        print(f"UNKNOWN status literals (runner contract changed?): {sorted(unknown)}")
        sys.exit(3)

    regressed, fixed, flipped = [], [], []
    for case in sorted(set(base) & set(head)):
        for name in sorted(set(base[case]) & set(head[case])):
            old, new = base[case][name], head[case][name]
            if old == new:
                continue
            entry = {"case": case, "subtest": name, "from": old, "to": new}
            if old == PASS:
                regressed.append(entry)
            elif new == PASS:
                fixed.append(entry)
            else:
                flipped.append(entry)

    lost_cases = sorted(set(base) - set(head))
    gained_cases = sorted(set(head) - set(base))
    drift_subtests = []
    for case in sorted(set(base) & set(head)):
        lost = sorted(set(base[case]) - set(head[case]))
        gained = sorted(set(head[case]) - set(base[case]))
        if lost or gained:
            drift_subtests.append({"case": case, "lost": len(lost), "gained": len(gained)})

    fixed_by_case = {}
    for e in fixed:
        fixed_by_case[e["case"]] = fixed_by_case.get(e["case"], 0) + 1

    print(f"base = {args[0]}")
    print(f"head = {args[1]}")
    print(f"REGRESSED (P->nonP): {len(regressed)}")
    for e in regressed:
        print(f"  {e['case']} :: {e['subtest']}  [{short(e['from'])}{short(e['to'])}]")
    print(f"FIXED (nonP->P): {len(fixed)} in {len(fixed_by_case)} cases")
    for case, n in sorted(fixed_by_case.items()):
        print(f"  {case}: {n}")
    print(f"FLIPPED (nonP->nonP): {len(flipped)}")
    for e in flipped:
        print(f"  {e['case']} :: {e['subtest']}  [{short(e['from'])}{short(e['to'])}]")
    print(f"DRIFT cases: lost={len(lost_cases)} gained={len(gained_cases)}")
    for case in lost_cases:
        print(f"  lost case: {case}")
    for case in gained_cases:
        print(f"  gained case: {case}")
    print(f"DRIFT subtests (in shared cases): {len(drift_subtests)}")
    for d in drift_subtests:
        print(f"  {d['case']}: lost={d['lost']} gained={d['gained']}")

    if out_json:
        with open(out_json, "w", encoding="utf-8") as f:
            json.dump(
                {
                    "regressed": regressed,
                    "fixed": fixed,
                    "flipped": flipped,
                    "lost_cases": lost_cases,
                    "gained_cases": gained_cases,
                    "drift_subtests": drift_subtests,
                },
                f,
                ensure_ascii=False,
                indent=1,
            )
        print(f"json detail -> {out_json}")

    if regressed:
        sys.exit(1)
    if lost_cases or gained_cases or drift_subtests:
        sys.exit(2)
    print("ANCHOR DIFF CLEAN: no regression, no drift")


if __name__ == "__main__":
    main()
