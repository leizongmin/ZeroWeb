#!/usr/bin/env python3
"""权威 reftest 发现工具（DC-14 分母去子集化）。

按 `<link rel="match" href="...">` 解析上游 WPT 测试文件，产出权威 test→ref 对
（区别于 discover-wpt-reftests.py 的文件名启发式，后者 miss 非标准 ref 命名）。

用法：
  python3 discover-reftests-authoritative.py --category css/css-grid [--max N] [--import]

不带 --import：仅输出发现的 test→ref 对（dry-run）。
带 --import：下载 test+ref 文件到 wpt-data/ 并更新 manifest。

依赖：curl（经 ~/use-proxy 代理）。GitHub raw 不限速；子目录枚举用 git/trees?recursive=1
（每目录 2 调用：contents(parent) 取 tree SHA + 递归取整棵子树），绕开 60/hr 限速。
"""
import argparse, json, os, re, subprocess, sys, urllib.request
from concurrent.futures import ThreadPoolExecutor

WPT_API = "https://api.github.com/repos/web-platform-tests/wpt/contents"
WPT_RAW = "https://raw.githubusercontent.com/web-platform-tests/wpt/master"
HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "..", "wpt-data")

LINK_RE = re.compile(r'<link[^>]*\brel\s*=\s*["\']match["\'][^>]*\bhref\s*=\s*["\']([^"\']+)["\']', re.I)
LINK_RE2 = re.compile(r'<link[^>]*\bhref\s*=\s*["\']([^"\']+)["\'][^>]*\brel\s*=\s*["\']match["\']', re.I)

def gh_api(path):
    """GitHub API 目录列表（单调用）。可选 GITHUB_TOKEN 环境变量提升限速（60→5000/hr）。"""
    import urllib.request as u
    headers = {"User-Agent": "zw-discover"}
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"token {token}"
    req = u.Request(f"{WPT_API}/{path}?ref=master", headers=headers)
    with u.urlopen(req, timeout=30) as r:
        return json.load(r)

def gh_tree(tree_sha):
    """Git Trees API recursive=1：单次调用返回整棵子树（含所有子目录 blob）。

    比逐子目录 gh_api(contents) 递归（css-text ~30 subdir / CSS2 ~43 subdir 各需 1 调用）
    省 30-75× 调用，绕开 60/hr 限速——这是 css-text / CSS2 全量导入能不靠 token 完成的关键。
    """
    import urllib.request as u
    headers = {"User-Agent": "zw-discover"}
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"token {token}"
    url = f"https://api.github.com/repos/web-platform-tests/wpt/git/trees/{tree_sha}?recursive=1"
    req = u.Request(url, headers=headers)
    with u.urlopen(req, timeout=30) as r:
        return json.load(r)

def get_tree_sha(category):
    """取 category 目录自身的 git tree SHA（contents(parent) 中找 leaf 条目的 sha）。"""
    if "/" in category:
        parent, leaf = category.rsplit("/", 1)
    else:
        parent, leaf = "", category
    for e in gh_api(parent):
        if e["name"] == leaf and e["type"] == "dir":
            return e["sha"]
    raise RuntimeError(f"{category} 目录未在 {parent or '(root)'} 下找到")

def fetch_raw(path):
    """raw.githubusercontent 文件内容（不限速）。"""
    import urllib.request as u
    req = u.Request(f"{WPT_RAW}/{path}", headers={"User-Agent": "zw-discover"})
    try:
        with u.urlopen(req, timeout=20) as r:
            return r.read().decode("utf-8", "ignore")
    except Exception:
        return None

# raw.githubusercontent 不限速，瓶颈是网络 RTT × 文件数。并发抓取把大目录
# （css-writing-modes ~400 test / css-text / CSS2）的发现+导入从顺序超时降到可接受。
# ThreadPoolExecutor.map 保持输入顺序，结果与 paths 对齐。
MAX_FETCH_WORKERS = 16

def fetch_raw_many(paths):
    """并行抓取多个 raw 文件。返回与 paths 同序的 [content|None]。"""
    if not paths:
        return []
    with ThreadPoolExecutor(max_workers=MAX_FETCH_WORKERS) as ex:
        return list(ex.map(fetch_raw, paths))

def is_test_file(name):
    return (name.endswith((".html", ".xht"))
            and "-ref" not in name and "notref" not in name and "reference" not in name)

def collect_test_paths(category):
    """收集 category 下所有 test 文件全路径（递归子目录，单次 git/trees 调用）。

    css-text / CSS2 等 test 散落在子目录（white-space/、box/...），git/trees?recursive=1
    一次性返回整棵子树。截断时（WPT 任一 css 目录子树 <100k 条目不会发生）立即报错，
    拒绝静默取部分以保 DC-14 分母完整性。
    """
    tree = gh_tree(get_tree_sha(category))
    if tree.get("truncated"):
        raise RuntimeError(f"{category} 子树被截断（>100k 条目）——分母不完整，拒绝")
    paths = []
    for e in tree["tree"]:
        if e["type"] == "blob" and is_test_file(os.path.basename(e["path"])):
            paths.append(f"{category}/{e['path']}")
    return paths

def discover(category, max_n=None):
    """返回 [(test_path, ref_path), ...] 权威对（递归子目录）。"""
    test_paths = collect_test_paths(category)
    if max_n:
        test_paths = test_paths[:max_n]
    contents = fetch_raw_many(test_paths)
    pairs = []
    for test_path, content in zip(test_paths, contents):
        if content is None:
            continue
        m = LINK_RE.search(content) or LINK_RE2.search(content)
        if not m:
            continue
        ref_href = m.group(1)
        # 解析 ref 路径：`/foo` = 仓库根相对；其余 = test 文件所在目录相对（子目录 test 关键）
        if ref_href.startswith("/"):
            ref_path = ref_href.lstrip("/").lstrip("\\")
        else:
            test_dir = os.path.dirname(test_path)
            ref_path = os.path.normpath(os.path.join(test_dir, ref_href)).replace("\\", "/")
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

    # 导入：收集所有需下载的唯一路径（跳过已存在），并行抓取后写入。
    paths_to_fetch = []
    seen = set()
    for test_path, ref_path in pairs:
        for p in (test_path, ref_path):
            if p in seen:
                continue
            seen.add(p)
            if not os.path.exists(os.path.join(DATA, p)):
                paths_to_fetch.append(p)
    fetched = fetch_raw_many(paths_to_fetch)
    for p, content in zip(paths_to_fetch, fetched):
        if content is None:
            print(f"  SKIP (fetch fail): {p}")
            continue
        dst = os.path.join(DATA, p)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        with open(dst, "w", encoding="utf-8") as f:
            f.write(content)
    # 计数两端均已落盘（本次或之前）的 test→ref 对
    imported = sum(
        1 for t, r in pairs
        if os.path.exists(os.path.join(DATA, t)) and os.path.exists(os.path.join(DATA, r))
    )
    print(f"导入 {imported} 对到 {DATA}（{len(paths_to_fetch)} 文件新下载）")

if __name__ == "__main__":
    main()
