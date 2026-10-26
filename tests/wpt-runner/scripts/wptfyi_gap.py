#!/usr/bin/env python3
"""wpt.fyi 差距分析工具（阶段 0：语料 revision 台账；阶段 1：三浏览器交集差距清单）。

只读接入 wpt.fyi 公开数据：拉取 chrome/firefox/safari aligned stable run 的
summary_v2 汇总，与 zero-wpt-runner `--json` 输出按测试路径交集对比，产出：

- gap_core    —— 三家浏览器全过、ZeroWeb 未全过的文件（最高价值修复清单）
- gap_partial —— 至少一家全过、ZeroWeb 未全过
- both_fail   —— 交集内没有任何一方全过（上游测试脆/规范歧义，降优先级）
- zw_strong   —— ZeroWeb 全过、至少一家浏览器未全过（信息面）
- both_pass   —— 双方都全过
- zw_env_limited —— ZeroWeb 侧未全过、且失败子测试信息全部命中环境受限标记
  （如 `zw-static-corpus`：静态语料无法执行的服务端行为），是语料边界而非
  引擎差距，单列不计入 gap。

口径与限制见 docs/compat/wptfyi-gap-analysis.md；子命令：

  revisions   打印语料 fetch 脚本的 WPT_REV pin 台账（无网络）
  fetch       下载/刷新三浏览器 aligned stable summary 到缓存目录
  gap         差距分析（读缓存 + ZeroWeb 结果目录，写 markdown/json 报告）
  self-test   离线自测（合成数据，不联网）

网络走 urllib，遵循 HTTPS_PROXY/HTTP_PROXY 环境变量（本机代理脚本
~/use-proxy-local.sh 导出后再运行）。全部逻辑只用标准库。
"""

from __future__ import annotations

import argparse
import gzip
import json
import re
import sys
import tempfile
import urllib.request
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

API_RUNS_URL = (
    "https://wpt.fyi/api/runs"
    "?products={products}&labels=stable&aligned=true&max-count=1"
)
DEFAULT_PRODUCTS = ["chrome", "firefox", "safari"]
DEFAULT_TIMEOUT = 120
DOWNLOAD_RETRIES = 3

# summary_v2 的 s 字段是单字符/短码状态（wpt.fyi 内部编码）：
#   O=OK(全部子测试通过) P=PASS(单测试文件通过，reftest 也算此列)
#   F=FAIL E=ERROR T=TIMEOUT S=SKIPPED C=CRASH PF=PRECONDITION_FAILED
# 文件级「全过」= s ∈ {O, P}；reftest 文件的 c 恒为 [0, 0]。
WPTFYI_FULL_PASS = frozenset({"O", "P"})

# zero-wpt-runner HarnessStatus：全过要求全部子测试 Pass；
# 中性状态（不通过率计账但也不算过）单列，报告里如实展示。
ZW_NEUTRAL_STATUS = frozenset({"Unsupported", "NotRun", "PreconditionFailed"})

# 环境受限标记：失败信息含这些子串的子测试是语料/宿主边界（非引擎差距）。
# runner 侧对静态语料无法执行的服务端行为打 `zw-static-corpus:` 前缀。
DEFAULT_ENV_MARKERS = ["zw-static-corpus"]

# 仓库相对的 fetch 脚本目录（本文件位于 tests/wpt-runner/scripts/ 下）。
SCRIPTS_DIR = Path(__file__).resolve().parent
# reftest 大语料走外部 repo（zeroweb-wpt-data，按 tag 固定），无上游 revision pin。
REFTEST_CORPUS_NOTE = "外部 repo zeroweb-wpt-data（tag 固定），无上游 revision pin"


class ToolError(SystemExit):
    """带用户可读消息的失败出口；不打印堆栈。"""

    def __init__(self, message: str, code: int = 1):
        super().__init__(code)
        self.message = message


# ---------------------------------------------------------------------------
# 阶段 0：WPT_REV pin 台账
# ---------------------------------------------------------------------------

_WPT_REV_RE = re.compile(r'^WPT_REV="([0-9a-f]{40})"\s*$', re.M)


def collect_wpt_revs(scripts_dir: Path = SCRIPTS_DIR) -> list[tuple[str, str]]:
    """扫描 fetch 脚本，返回 (相对路径, revision) 列表；无 pin 的脚本跳过。"""
    found: list[tuple[str, str]] = []
    for pattern in ("*.sh", "goals/*.sh"):
        for script in sorted(scripts_dir.glob(pattern)):
            text = script.read_text(encoding="utf-8", errors="replace")
            for rev in _WPT_REV_RE.findall(text):
                found.append((script.relative_to(scripts_dir).as_posix(), rev))
    return found


def cmd_revisions(_args: argparse.Namespace) -> None:
    rows = collect_wpt_revs()
    if not rows:
        raise ToolError(f"未在 {SCRIPTS_DIR} 下扫到 WPT_REV pin")
    by_rev: dict[str, int] = defaultdict(int)
    for script, rev in rows:
        print(f"{script:55s} {rev}")
        by_rev[rev] += 1
    print()
    for rev, count in sorted(by_rev.items(), key=lambda item: -item[1]):
        print(f"{rev}  ×{count} 个脚本")
    print(f"reftest 大语料: {REFTEST_CORPUS_NOTE}")


# ---------------------------------------------------------------------------
# 阶段 1：fetch + gap
# ---------------------------------------------------------------------------


def _http_get(url: str, timeout: int = DEFAULT_TIMEOUT) -> bytes:
    """带重试的 GET；urllib 遵循 HTTPS_PROXY 等代理环境变量。"""
    last_error: Exception | None = None
    for attempt in range(1, DOWNLOAD_RETRIES + 1):
        try:
            with urllib.request.urlopen(url, timeout=timeout) as response:
                return response.read()
        except Exception as error:  # 代理抖动/超时统一重试
            last_error = error
            if attempt < DOWNLOAD_RETRIES:
                print(f"  下载失败（第 {attempt} 次），重试: {error}", file=sys.stderr)
    raise ToolError(f"下载失败（已重试 {DOWNLOAD_RETRIES} 次）: {url}\n{last_error}")


def _atomic_write(path: Path, payload: bytes) -> None:
    """同目录临时文件 + replace；失败路径不触碰既有文件。"""
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_bytes(payload)
    tmp.replace(path)


def decode_summary(raw: bytes) -> dict:
    """summary_v2：实际分发是明文 JSON，文件名 .gz 名不副实；两种编码都兼容。"""
    if raw[:2] == b"\x1f\x8b":
        raw = gzip.decompress(raw)
    data = json.loads(raw.decode("utf-8"))
    if not isinstance(data, dict):
        raise ToolError("summary 顶层不是 JSON 对象")
    return data


def fetch_aligned_runs(products: list[str]) -> list[dict]:
    url = API_RUNS_URL.format(products=",".join(products))
    runs = json.loads(_http_get(url))
    if not isinstance(runs, list) or len(runs) != len(products):
        raise ToolError(f"aligned run 数量不符：期望 {len(products)}")
    if {r.get("browser_name") for r in runs} != set(products):
        raise ToolError(f"run 产品集不符：{[r.get('browser_name') for r in runs]}")
    for run in runs:
        if not run.get("results_url"):
            raise ToolError(f"run 缺 results_url: {run.get('browser_name')}")
    return runs


def cmd_fetch(args: argparse.Namespace) -> None:
    cache = Path(args.cache)
    cache.mkdir(parents=True, exist_ok=True)
    products = args.products.split(",")

    runs_path = cache / "runs.json"
    summaries_ok = all((cache / f"summary-{p}.json").exists() for p in products)
    if runs_path.exists() and summaries_ok and not args.refresh:
        print(f"缓存已存在（{cache}），--refresh 可强制刷新")
        return

    runs = fetch_aligned_runs(products)
    for run in runs:
        product = run["browser_name"]
        print(f"下载 {product} {run['browser_version']} @ {run['revision']} ...")
        raw = _http_get(run["results_url"], timeout=args.timeout)
        summary = decode_summary(raw)
        # 校验通过才落盘；既有缓存文件在任何失败路径下都不被删除或破坏。
        payload = json.dumps(summary, ensure_ascii=False).encode("utf-8")
        _atomic_write(cache / f"summary-{product}.json", payload)
    _atomic_write(
        runs_path, json.dumps(runs, ensure_ascii=False, indent=1).encode("utf-8")
    )
    for run in runs:
        print(
            f"  {run['browser_name']}: {run['browser_version']}"
            f" @ {run['revision']}（{run['created_at']}）"
        )


def normalize_path(path: str) -> str:
    """wpt.fyi 的 key 带前导斜杠，runner case 名不带；统一补齐。"""
    return path if path.startswith("/") else "/" + path


# runner 以 .js 源文件名上报，wpt.fyi 收录生成后的 HTML 变体；候选按 ZW 实际
# 执行 scope 排序——.any.js 的 window 面是 .any.html，worker 变体仅作兜底。
_VARIANT_RULES = [
    (".any.js", (".any.html", ".any.worker.html")),
    (".window.js", (".window.html",)),
    (".worker.js", (".worker.html",)),
]


def variant_candidates(path: str) -> list[str]:
    for suffix, variants in _VARIANT_RULES:
        if path.endswith(suffix):
            stem = path[: -len(suffix)]
            return [stem + variant for variant in variants]
    return []


def load_zw_results(path: Path, env_markers: list[str]) -> dict[str, dict]:
    """加载 zero-wpt-runner `--json` 输出（[[case, subtests], ...]）。

    返回 {规范路径: {passes, fails, neutral, total, full_pass, suspicious,
    env_fails, env_limited}}。
    """
    try:
        cases = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ToolError(f"读取 ZeroWeb 结果失败 {path}: {error}") from error
    if not isinstance(cases, list):
        raise ToolError(f"{path} 顶层不是数组——确认是 zero-wpt-runner --json 输出")

    files: dict[str, dict] = {}
    seen: set[str] = set()
    for entry in cases:
        if (
            not isinstance(entry, list)
            or len(entry) != 2
            or not isinstance(entry[0], str)
            or not isinstance(entry[1], list)
        ):
            raise ToolError(f"{path} 存在非法条目 {entry!r}")
        raw_path, subtests = entry
        name = normalize_path(raw_path)
        if name in seen:
            # 同名多 scope 的结果无法映射到单一上游变体，标可疑不入分类。
            files[name]["suspicious"] = True
            files[name]["total"] = 0
            continue
        seen.add(name)
        passes = fails = neutral = env_fails = 0
        for subtest in subtests:
            status = subtest.get("status") if isinstance(subtest, dict) else None
            message = subtest.get("message") if isinstance(subtest, dict) else None
            if status == "Pass":
                passes += 1
                continue
            if status in ZW_NEUTRAL_STATUS:
                neutral += 1
            else:
                fails += 1
            if isinstance(message, str) and any(
                marker in message for marker in env_markers
            ):
                env_fails += 1
        total = len(subtests)
        # 空 subtests 属 runner 侧可疑输出，不参与分类，单独计数。
        files[name] = {
            "passes": passes,
            "fails": fails,
            "neutral": neutral,
            "total": total,
            "full_pass": total > 0 and fails == 0 and neutral == 0,
            "suspicious": total == 0,
            "env_fails": env_fails,
            "env_limited": False,
        }
        if not files[name]["full_pass"] and env_fails > 0:
            not_pass = fails + neutral
            # 全部未过子测试都是环境受限 → 语料边界，不是引擎差距。
            files[name]["env_limited"] = env_fails == not_pass
    return files


def load_summary(path: Path) -> dict:
    try:
        return decode_summary(path.read_bytes())
    except (OSError, json.JSONDecodeError, UnicodeDecodeError) as error:
        raise ToolError(f"读取 wpt.fyi summary 失败 {path}: {error}") from error


def classify(zw_files: dict, summaries: dict[str, dict]) -> dict:
    """按完整交集口径分类：路径必须在全部浏览器 summary 中都出现。

    单家浏览器缺该路径 → coverage_missing（不对齐的测试不强行判定）；
    ZeroWeb 侧空 subtests 条目 → skipped_suspicious，同样不入分类。
    """
    buckets: dict[str, list] = {
        k: []
        for k in (
            "gap_core", "gap_partial", "both_fail", "zw_strong", "both_pass",
            "zw_env_limited",
        )
    }
    coverage_missing = 0
    skipped_suspicious = 0
    variant_resolved = 0

    for name, zw in zw_files.items():
        if zw["suspicious"]:
            skipped_suspicious += 1
            continue
        # 完整交集口径：exact 三家齐用 exact；否则尝试 .js→HTML 变体，仍要三家齐。
        # 不做单家混合（exact + 变体）匹配，避免跨浏览器比对到不同文件。
        entries = {product: s.get(name) for product, s in summaries.items()}
        resolved_by_variant = False
        if any(entry is None for entry in entries.values()):
            candidates = variant_candidates(name)
            if not candidates:
                coverage_missing += 1
                continue
            entries = {}
            for product, summary in summaries.items():
                entry = next(
                    (summary[c] for c in candidates if c in summary), None
                )
                if entry is None:
                    entries = None
                    break
                entries[product] = entry
            if entries is None:
                coverage_missing += 1
                continue
            resolved_by_variant = True
        if resolved_by_variant:
            variant_resolved += 1
        browser_ok = {
            product: entry["s"] in WPTFYI_FULL_PASS
            for product, entry in entries.items()
        }
        all_ok = all(browser_ok.values())
        any_ok = any(browser_ok.values())
        item = {
            "path": name,
            "zw": {
                k: zw[k] for k in ("passes", "fails", "neutral", "total")
            },
            "browsers": {
                product: [entry["c"][0], entry["c"][1]]
                for product, entry in entries.items()
            },
        }
        if zw["full_pass"] and all_ok:
            buckets["both_pass"].append(item)
        elif zw["env_limited"]:
            # 全部失败都命中环境受限标记：语料边界，先于 gap 分类。
            buckets["zw_env_limited"].append(item)
        elif not zw["full_pass"] and all_ok:
            buckets["gap_core"].append(item)
        elif not zw["full_pass"] and any_ok:
            buckets["gap_partial"].append(item)
        elif not zw["full_pass"]:
            buckets["both_fail"].append(item)
        else:
            buckets["zw_strong"].append(item)

    return {
        "buckets": buckets,
        "coverage_missing": coverage_missing,
        "skipped_suspicious": skipped_suspicious,
        "variant_resolved": variant_resolved,
    }


def topdir(path: str) -> str:
    return path.lstrip("/").split("/", 1)[0]


BUCKET_KEYS = (
    "gap_core", "gap_partial", "both_fail", "zw_strong", "both_pass",
    "zw_env_limited",
)


def empty_stat() -> dict:
    return {key: 0 for key in ("intersection", *BUCKET_KEYS)}


def aggregate(buckets: dict, by_key) -> dict[str, dict]:
    stats: dict[str, dict] = defaultdict(empty_stat)
    for bucket, items in buckets.items():
        for item in items:
            key = by_key(item)
            stats[key][bucket] += 1
            stats[key]["intersection"] += 1
    return dict(stats)


def build_meta(runs: list[dict]) -> dict:
    return {
        "generated_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "runs": [
            {
                "browser": r["browser_name"],
                "version": r["browser_version"],
                "revision": r["revision"],
                "created_at": r["created_at"],
            }
            for r in runs
        ],
        "corpus_revs": sorted({rev for _, rev in collect_wpt_revs()}),
        "reftest_corpus": REFTEST_CORPUS_NOTE,
    }


def cmd_gap(args: argparse.Namespace) -> None:
    cache = Path(args.cache)
    runs_path = cache / "runs.json"
    if not runs_path.exists():
        raise ToolError(f"缓存缺失（{runs_path}），先运行 fetch 子命令")
    runs = json.loads(runs_path.read_text(encoding="utf-8"))
    products = [run["browser_name"] for run in runs]
    summaries = {
        product: load_summary(cache / f"summary-{product}.json")
        for product in products
    }

    results_dir = Path(args.results)
    result_files = sorted(results_dir.glob("*.json"))
    if not result_files:
        raise ToolError(f"{results_dir} 下没有 *.json 结果文件")

    per_suite: dict[str, dict] = {}
    all_buckets = {key: [] for key in BUCKET_KEYS}
    total_missing = 0
    total_suspicious = 0
    total_variant = 0
    for path in result_files:
        files = load_zw_results(path, args.env_marker)
        result = classify(files, summaries)
        per_suite[path.stem] = {
            "files": len(files),
            "intersection": sum(len(v) for v in result["buckets"].values()),
            "coverage_missing": result["coverage_missing"],
            "skipped_suspicious": result["skipped_suspicious"],
            "variant_resolved": result["variant_resolved"],
            **{k: len(v) for k, v in result["buckets"].items()},
        }
        for bucket, items in result["buckets"].items():
            for item in items:
                item["suite"] = path.stem
            all_buckets[bucket].extend(items)
        total_missing += result["coverage_missing"]
        total_suspicious += result["skipped_suspicious"]
        total_variant += result["variant_resolved"]

    out_md = Path(args.out)
    out_md.parent.mkdir(parents=True, exist_ok=True)
    out_md.write_text(
        render_markdown(
            runs, per_suite, all_buckets, total_missing, total_suspicious, total_variant
        ),
        encoding="utf-8",
    )
    out_json = out_md.with_suffix(".json")
    out_json.write_text(
        json.dumps(
            {
                "meta": build_meta(runs),
                "suites": per_suite,
                "topdirs": aggregate(all_buckets, lambda i: topdir(i["path"])),
                **all_buckets,
            },
            ensure_ascii=False,
            indent=1,
        ),
        encoding="utf-8",
    )
    print(f"报告: {out_md}\n数据: {out_json}")
    counts = " ".join(f"{k}={len(all_buckets[k])}" for k in BUCKET_KEYS)
    print(
        f"{counts} coverage_missing={total_missing}"
        f" suspicious={total_suspicious} variant_resolved={total_variant}"
    )


def render_markdown(
    runs: list[dict],
    per_suite: dict,
    buckets: dict,
    total_missing: int,
    total_suspicious: int,
    total_variant: int,
) -> str:
    lines: list[str] = []
    lines.append("# ZeroWeb vs wpt.fyi 三浏览器差距分析")
    lines.append("")
    lines.append(f"生成时间：{datetime.now(timezone.utc).isoformat(timespec='seconds')}")
    lines.append("")
    lines.append("## 口径")
    lines.append("")
    lines.append("- 浏览器基准：wpt.fyi aligned stable run（同一上游 revision）：")
    for run in runs:
        lines.append(
            f"  - {run['browser_name']} {run['browser_version']} @ {run['revision']}"
            f"（{run['created_at']}）"
        )
    corpus_revs = sorted({rev for _, rev in collect_wpt_revs()})
    lines.append(
        f"- ZeroWeb 语料 pin：{', '.join(corpus_revs)}；reftest 大语料{REFTEST_CORPUS_NOTE}"
        "——与 run revision 存在时间漂移，同路径下测试内容可能已变。"
    )
    lines.append(
        "- 文件级「全过」：ZeroWeb 侧全部子测试 Pass；浏览器侧 summary `s ∈ {O, P}`。"
    )
    lines.append(
        "- 只对三家浏览器 summary 都收录的路径判定（完整交集），"
        f"交集外路径 {total_missing} 条不计入任何分类。"
    )
    lines.append(
        f"- ZeroWeb 侧空 subtests 可疑条目共 {total_suspicious} 条，不参与分类。"
    )
    lines.append(
        f"- 变体解析：{total_variant} 条 .js 源名按生成变体（.any.html 等）匹配，"
        "候选取 ZW 实际执行的 window scope 优先。"
    )
    lines.append("")
    lines.append("## 分套件汇总")
    lines.append("")
    lines.append(
        "| 套件 | 文件 | 交集 | gap_core | gap_partial | both_fail"
        " | zw_strong | both_pass | 环境受限 | 交集外 | 可疑 |"
    )
    lines.append("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for suite in sorted(per_suite):
        s = per_suite[suite]
        lines.append(
            f"| {suite} | {s['files']} | {s['intersection']} | {s['gap_core']} "
            f"| {s['gap_partial']} | {s['both_fail']} | {s['zw_strong']} "
            f"| {s['both_pass']} | {s['zw_env_limited']} "
            f"| {s['coverage_missing']} | {s['skipped_suspicious']} |"
        )
    lines.append("")

    topdir_stats = aggregate(buckets, lambda i: topdir(i["path"]))
    lines.append("## 分目录汇总（交集内）")
    lines.append("")
    lines.append(
        "| 目录 | 交集 | gap_core | gap_partial | both_fail | zw_strong | both_pass |"
    )
    lines.append("|---|---:|---:|---:|---:|---:|---:|")
    for td, s in sorted(topdir_stats.items(), key=lambda kv: -kv[1]["gap_core"]):
        lines.append(
            f"| {td} | {s['intersection']} | {s['gap_core']} | {s['gap_partial']} "
            f"| {s['both_fail']} | {s['zw_strong']} | {s['both_pass']} |"
        )
    lines.append("")

    sections = [
        ("gap_core", "gap_core：三家全过、ZeroWeb 未全过（最高价值）", 200),
        ("gap_partial", "gap_partial：至少一家全过、ZeroWeb 未全过", 200),
        (
            "zw_env_limited",
            "zw_env_limited：环境受限（失败全命中标记，语料边界不计入 gap）",
            100,
        ),
        ("zw_strong", "zw_strong：ZeroWeb 全过、至少一家未全过", 100),
        ("both_fail", "both_fail：交集内没有一方全过（降优先级）", 100),
    ]
    for bucket, title, limit in sections:
        items = sorted(buckets[bucket], key=lambda i: (-i["zw"]["fails"], i["path"]))
        lines.append(f"## {title}（{len(items)} 条）")
        lines.append("")
        for item in items[:limit]:
            zw = item["zw"]
            neutral = f" neutral={zw['neutral']}" if zw["neutral"] else ""
            lines.append(
                f"- `{item['path']}`（{item['suite']}："
                f"{zw['passes']}/{zw['total']}{neutral}）"
            )
        if len(items) > limit:
            lines.append(f"- … 其余 {len(items) - limit} 条见同名 .json")
        lines.append("")
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# self-test
# ---------------------------------------------------------------------------


def run_self_test() -> None:
    """合成数据离线自测：分类、口径与 summary 解码，不联网。"""
    ok_entry = {"s": "O", "c": [2, 2]}
    fail_entry = {"s": "F", "c": [1, 2]}
    reftest_ok = {"s": "P", "c": [0, 0]}
    summaries = {
        "chrome": {
            "/a/one.html": ok_entry,
            "/b/two.html": ok_entry,
            "/c/three.html": ok_entry,
            "/d/four.html": fail_entry,
            "/e/five.html": fail_entry,
            "/f/six.html": reftest_ok,
            "/env/marker.html": ok_entry,
            "/k/ten.any.html": ok_entry,
        },
        "firefox": {
            "/a/one.html": ok_entry,
            "/b/two.html": ok_entry,
            "/c/three.html": fail_entry,
            "/d/four.html": fail_entry,
            "/e/five.html": {"s": "F", "c": [0, 3]},
            "/f/six.html": reftest_ok,
            "/env/marker.html": ok_entry,
            "/g/seven.html": ok_entry,  # 仅 firefox 有 → 不影响 zw 侧交集
            "/k/ten.any.html": ok_entry,
        },
        "safari": {
            "/a/one.html": ok_entry,
            "/b/two.html": ok_entry,
            "/c/three.html": ok_entry,
            "/d/four.html": fail_entry,
            "/e/five.html": fail_entry,
            "/f/six.html": reftest_ok,
            "/env/marker.html": ok_entry,
            "/k/ten.any.html": ok_entry,
            "/k/ten.any.worker.html": fail_entry,
        },
    }

    def zw(statuses: list[str]) -> dict:
        env_fails = 0  # 合成用例不携带标记信息；env 场景单独构造
        return {
            "passes": statuses.count("Pass"),
            "fails": statuses.count("Fail"),
            "neutral": statuses.count("NotRun"),
            "total": len(statuses),
            "full_pass": len(statuses) > 0
            and "Fail" not in statuses
            and "NotRun" not in statuses,
            "suspicious": len(statuses) == 0,
            "env_fails": env_fails,
            "env_limited": False,
        }

    zw_files = {
        "/a/one.html": zw(["Pass", "Pass"]),      # 双方全过 → both_pass
        "/b/two.html": zw(["Pass", "Fail"]),      # 三家全过、zw 挂 → gap_core
        "/c/three.html": zw(["Pass", "NotRun"]),  # 部分全过、zw 挂 → gap_partial
        "/d/four.html": zw(["Pass", "Pass"]),     # 无人全过、zw 过 → zw_strong
        "/e/five.html": zw(["Fail"]),             # 无人全过、zw 挂 → both_fail
        "/f/six.html": zw(["Pass"]),              # reftest P 算全过 → both_pass
        "/h/eight.html": zw(["Pass"]),            # 浏览器侧全缺 → coverage_missing
        "/i/nine.html": zw([]),                   # 空 subtests → 可疑跳过
        # .js 源名 → window 变体解析（.any.html 优先，三家一致取它）→ both_pass
        "/k/ten.any.js": zw(["Pass", "Pass"]),
        # 失败全部命中环境受限标记 → 单列，即使三家全过也不进 gap_core
        "/env/marker.html": {
            "passes": 1, "fails": 1, "neutral": 0, "total": 2,
            "full_pass": False, "suspicious": False,
            "env_fails": 1, "env_limited": True,
        },
    }
    result = classify(zw_files, summaries)
    assert sorted(i["path"] for i in result["buckets"]["both_pass"]) == [
        "/a/one.html",
        "/f/six.html",
        "/k/ten.any.js",
    ], result["buckets"]
    assert [i["path"] for i in result["buckets"]["gap_core"]] == ["/b/two.html"]
    assert [i["path"] for i in result["buckets"]["gap_partial"]] == ["/c/three.html"]
    assert [i["path"] for i in result["buckets"]["zw_strong"]] == ["/d/four.html"]
    assert [i["path"] for i in result["buckets"]["both_fail"]] == ["/e/five.html"]
    assert [i["path"] for i in result["buckets"]["zw_env_limited"]] == [
        "/env/marker.html"
    ]
    assert result["coverage_missing"] == 1
    assert result["skipped_suspicious"] == 1
    assert result["variant_resolved"] == 1
    # 变体解析取 window scope：三家比对的都是 .any.html，不是 safari 独有的
    # .any.worker.html（那里是 FAIL，若误取会翻成 gap）。
    ten = next(i for i in result["buckets"]["both_pass"] if i["path"] == "/k/ten.any.js")
    assert all(entry == [2, 2] for entry in ten["browsers"].values()), ten

    # 分目录聚合
    stats = aggregate(result["buckets"], lambda i: topdir(i["path"]))
    assert stats["a"]["intersection"] == 1
    assert stats["a"]["both_pass"] == 1
    assert not any(stats["a"][k] for k in BUCKET_KEYS if k != "both_pass")
    assert stats["b"]["gap_core"] == 1
    assert "g" not in stats  # 仅浏览器侧存在的路径不进聚合

    # 明文与 gzip 两种 summary 编码
    assert decode_summary(b'{"a": 1}') == {"a": 1}
    assert decode_summary(gzip.compress(b'{"a": 1}')) == {"a": 1}

    # 加载侧：同名多 scope 的重复 case 标可疑，不静默覆盖
    with tempfile.TemporaryDirectory() as tmp:
        dup = Path(tmp) / "dup.json"
        dup.write_text(
            json.dumps(
                [
                    ["x/one.any.js", [{"name": "a", "status": "Pass", "message": None}]],
                    ["x/one.any.js", [{"name": "b", "status": "Fail", "message": None}]],
                ]
            )
        )
        loaded = load_zw_results(dup, DEFAULT_ENV_MARKERS)
        assert loaded["/x/one.any.js"]["suspicious"] is True
        assert loaded["/x/one.any.js"]["total"] == 0

    # 加载侧：失败信息全命中环境标记 → env_limited；混合真实失败则不标记
    with tempfile.TemporaryDirectory() as tmp:
        envcase = Path(tmp) / "env.json"
        envcase.write_text(
            json.dumps(
                [
                    [
                        "y/marker.html",
                        [
                            {
                                "name": "a",
                                "status": "Fail",
                                "message": "zw-static-corpus: no server",
                            },
                            {
                                "name": "b",
                                "status": "Fail",
                                "message": "assert_true failed",
                            },
                        ],
                    ],
                    [
                        "y/pure.html",
                        [
                            {
                                "name": "a",
                                "status": "Fail",
                                "message": "zw-static-corpus: no server",
                            }
                        ],
                    ],
                ]
            )
        )
        loaded = load_zw_results(envcase, DEFAULT_ENV_MARKERS)
        assert loaded["/y/marker.html"]["env_limited"] is False
        assert loaded["/y/pure.html"]["env_limited"] is True

    # WPT_REV 解析（合成脚本目录）
    with tempfile.TemporaryDirectory() as tmp:
        scripts = Path(tmp)
        (scripts / "demo.sh").write_text(
            'WPT_REV="315976933870b34d6ea30e3f6643403edae678ba"\n'
        )
        goals = scripts / "goals"
        goals.mkdir()
        (goals / "lib.sh").write_text(
            'WPT_REV="04067ce9c7c2165e71ad7d0dde10a4c5cb394a83"\n'
        )
        rows = collect_wpt_revs(scripts)
        assert dict(rows) == {
            "demo.sh": "315976933870b34d6ea30e3f6643403edae678ba",
            "goals/lib.sh": "04067ce9c7c2165e71ad7d0dde10a4c5cb394a83",
        }, rows

    print("self-test: 全部通过")


# ---------------------------------------------------------------------------


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("revisions", help="打印语料 WPT_REV pin 台账")

    p_fetch = sub.add_parser("fetch", help="下载三浏览器 aligned stable summary 到缓存")
    p_fetch.add_argument("--cache", default="tests/wpt-runner/wptfyi-cache")
    p_fetch.add_argument("--products", default=",".join(DEFAULT_PRODUCTS))
    p_fetch.add_argument("--refresh", action="store_true", help="忽略既有缓存强制刷新")
    p_fetch.add_argument("--timeout", type=int, default=DEFAULT_TIMEOUT)
    p_fetch.set_defaults(func=cmd_fetch)

    p_gap = sub.add_parser("gap", help="差距分析")
    p_gap.add_argument("--cache", default="tests/wpt-runner/wptfyi-cache")
    p_gap.add_argument("--results", required=True, help="zero-wpt-runner --json 结果目录")
    p_gap.add_argument("--out", default="wptfyi-gap-report.md", help="输出 markdown 路径")
    p_gap.add_argument(
        "--env-marker",
        action="append",
        default=DEFAULT_ENV_MARKERS,
        help="环境受限标记子串（可重复）；失败信息全命中标记的文件单列不计入 gap",
    )
    p_gap.set_defaults(func=cmd_gap)

    sub.add_parser("self-test", help="离线自测")

    args = parser.parse_args()
    if args.command == "revisions":
        cmd_revisions(args)
    elif args.command == "fetch":
        cmd_fetch(args)
    elif args.command == "gap":
        cmd_gap(args)
    else:
        run_self_test()


if __name__ == "__main__":
    try:
        main()
    except ToolError as error:
        print(f"错误: {error.message}", file=sys.stderr)
        raise SystemExit(1) from None
