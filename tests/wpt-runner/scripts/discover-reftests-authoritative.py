#!/usr/bin/env python3
"""权威 reftest 发现工具（DC-14 分母去子集化）。

按 `<link rel="match" href="...">` 解析上游 WPT 测试文件，产出权威 test→ref 对
（区别于 discover-wpt-reftests.py 的文件名启发式，后者 miss 非标准 ref 命名）。

用法：
  python3 discover-reftests-authoritative.py --category css/css-grid [--max N] [--import]

不带 --import：仅输出发现的 test→ref 对（dry-run）。
带 --import：下载 test+ref 文件到 wpt-data/ 并更新 manifest。

依赖：curl（经 ~/use-proxy 代理）。GitHub raw 不限速；目录列表用 API（限速，单目录 1 调用）。
"""
import argparse, json, os, re, subprocess, sys, urllib.request

WPT_API = "https://api.github.com/repos/web-platform-tests/wpt/contents"
WPT_RAW = "https://raw.githubusercontent.com/web-platform-tests/wpt/master"
HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "..", "wpt-data")

LINK_RE = re.compile(r'<link[^>]*\brel\s*=\s*["\']match["\'][^>]*\bhref\s*=\s*["\']([^"\']+)["\']', re.I)
LINK_RE2 = re.compile(r'<link[^>]*\bhref\s*=\s*["\']([^"\']+)["\'][^>]*\brel\s*=\s*["\']match["\']', re.I)

def gh_api(path):
    """GitHub API 目录列表（单调用）。"""
    import urllib.request as u
    req = u.Request(f"{WPT_API}/{path}?ref=master", headers={"User-Agent": "zw-discover"})
    with u.urlopen(req, timeout=30) as r:
        return json.load(r)

def fetch_raw(path):
    """raw.githubusercontent 文件内容（不限速）。"""
    import urllib.request as u
    req = u.Request(f"{WPT_RAW}/{path}", headers={"User-Agent": "zw-discover"})
    try:
        with u.urlopen(req, timeout=20) as r:
            return r.read().decode("utf-8", "ignore")
    except Exception:
        return None

def discover(category, max_n=None):
    """返回 [(test_path, ref_path), ...] 权威对。"""
    entries = gh_api(category)
    test_files = [e["name"] for e in entries
                  if e["type"] == "file" and e["name"].endswith((".html", ".xht"))
                  and "-ref" not in e["name"] and "notref" not in e["name"] and "reference" not in e["name"]]
    if max_n:
        test_files = test_files[:max_n]
    pairs = []
    for name in test_files:
        test_path = f"{category}/{name}"
        content = fetch_raw(test_path)
        if content is None:
            continue
        m = LINK_RE.search(content) or LINK_RE2.search(content)
        if not m:
            continue
        ref_href = m.group(1)
        # 解析 ref 路径：`/foo` = 仓库根相对；其余 = test 文件目录相对
        if ref_href.startswith("/"):
            ref_path = ref_href.lstrip("/").lstrip("\\")
        else:
            ref_path = os.path.normpath(os.path.join(category, ref_href)).replace("\\", "/")
        pairs.append((test_path, ref_path))
    return pairs

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--category", required=True)
    ap.add_argument("--max", type=int, default=None)
    ap.add_argument("--import", dest="do_import", action="store_true")
    args = ap.parse_args()

    pairs = discover(args.category, args.max)
    print(f"{args.category}: 发现 {len(pairs)} 个权威 reftest 对（link rel=match 解析）")
    if not args.do_import:
        for t, r in pairs[:20]:
            print(f"  {t}  ->  {r}")
        if len(pairs) > 20:
            print(f"  ... ({len(pairs)} total)")
        return

    # 导入：下载 test+ref
    imported = 0
    for test_path, ref_path in pairs:
        for p in (test_path, ref_path):
            dst = os.path.join(DATA, p)
            if os.path.exists(dst):
                continue
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            content = fetch_raw(p)
            if content is None:
                print(f"  SKIP (fetch fail): {p}")
                continue
            with open(dst, "w", encoding="utf-8") as f:
                f.write(content)
        imported += 1
    print(f"导入 {imported} 对到 {DATA}")

if __name__ == "__main__":
    main()
