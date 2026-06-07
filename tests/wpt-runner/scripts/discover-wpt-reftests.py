#!/usr/bin/env python3
"""WPT Reftest 动态发现和导入工具。

从上游 WPT GitHub 仓库动态发现并下载 reftest 文件对。

用法：
  python3 discover-wpt-reftests.py [--category DIR] [--max N] [--output DIR]

依赖：requests (或 curl 回退)
"""

import argparse
import json
import os
import subprocess
import sys
import urllib.request
import urllib.error

WPT_API = "https://api.github.com/repos/web-platform-tests/wpt/contents"
WPT_RAW = "https://raw.githubusercontent.com/web-platform-tests/wpt/master"

# 默认导入的目录
DEFAULT_CATEGORIES = [
    "css/CSS2/colors",
    "css/CSS2/backgrounds",
    "css/CSS2/borders",
    "css/CSS2/box",
    "css/CSS2/abspos",
    "css/CSS2/floats",
    "css/CSS2/floats-clear",
    "css/CSS2/fonts",
    "css/CSS2/linebox",
    "css/CSS2/normal-flow",
]

# 需要排除的文件模式（引用外部资源）
SKIP_PATTERNS = [
    "svg", "canvas", "webgl", "video", "audio", "animation",
    "Ahem", "testharness", ".svg", ".png", ".jpg",
]


def setup_proxy():
    """设置代理（如果存在 ~/use-proxy 文件）"""
    proxy_file = os.path.expanduser("~/use-proxy")
    if os.path.exists(proxy_file):
        with open(proxy_file) as f:
            for line in f:
                line = line.strip()
                if line.startswith("export "):
                    line = line[len("export "):]
                if "=" in line:
                    key, value = line.split("=", 1)
                    os.environ[key] = value


def fetch_json(url):
    """从 GitHub API 获取 JSON"""
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "ZeroWeb-WPT-Importer"})
        with urllib.request.urlopen(req, timeout=15) as resp:
            return json.loads(resp.read())
    except Exception as e:
        print(f"  ERROR fetching {url}: {e}", file=sys.stderr)
        return None


def download_file(url, dest_path):
    """下载文件到本地"""
    os.makedirs(os.path.dirname(dest_path), exist_ok=True)
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "ZeroWeb-WPT-Importer"})
        with urllib.request.urlopen(req, timeout=15) as resp:
            with open(dest_path, "wb") as f:
                f.write(resp.read())
        return True
    except Exception as e:
        print(f"  SKIP: {url} ({e})", file=sys.stderr)
        if os.path.exists(dest_path):
            os.remove(dest_path)
        return False


def discover_reftest_pairs(category_dir):
    """从 WPT 目录发现 reftest 文件对"""
    url = f"{WPT_API}/{category_dir}"
    data = fetch_json(url)
    if data is None:
        return []

    files = [f for f in data if f.get("type") == "file" and f["name"].endswith(".xht") or f.get("type") == "file" and f["name"].endswith(".html")]

    # 分离测试文件和参考文件
    test_files = {}
    ref_files = set()
    for f in data:
        if f.get("type") != "file":
            continue
        name = f["name"]
        if name.endswith(("-ref.xht", "-ref.html")):
            ref_files.add(name)
        elif name.endswith((".xht", ".html")):
            test_files[name] = f

    # 匹配测试和参考文件对
    pairs = []
    for test_name, test_info in sorted(test_files.items()):
        # 尝试多种参考文件命名模式
        base = test_name.rsplit(".", 1)[0]
        ref_candidates = [
            f"{base}-ref.xht",
            f"{base}-ref.html",
            base.replace("-001", "-ref").replace("-002", "-ref").replace("-003", "-ref") + ".xht",
            base.replace("-001", "-ref").replace("-002", "-ref").replace("-003", "-ref") + ".html",
        ]
        for ref_name in ref_candidates:
            if ref_name in ref_files:
                pairs.append((test_name, ref_name, test_info.get("download_url", "")))
                break

    return pairs


def should_skip(test_name):
    """检查是否应跳过该测试"""
    name_lower = test_name.lower()
    for pattern in SKIP_PATTERNS:
        if pattern.lower() in name_lower:
            return True
    return False


def main():
    parser = argparse.ArgumentParser(description="WPT Reftest 动态发现和导入")
    parser.add_argument("--category", action="append", help="WPT 目录（可多次指定）")
    parser.add_argument("--max", type=int, default=60, help="每个目录最大导入数量")
    parser.add_argument("--output", default=None, help="输出目录")
    args = parser.parse_args()

    setup_proxy()

    script_dir = os.path.dirname(os.path.abspath(__file__))
    output_dir = args.output or os.path.join(script_dir, "..", "wpt-data")
    output_dir = os.path.normpath(output_dir)

    categories = args.category or DEFAULT_CATEGORIES

    print(f"WPT Reftest Dynamic Importer")
    print(f"  Categories: {len(categories)}")
    print(f"  Max/category: {args.max}")
    print(f"  Output: {output_dir}")
    print()

    total_imported = 0
    total_skipped = 0

    for category in categories:
        print(f"Scanning {category}...")
        pairs = discover_reftest_pairs(category)
        print(f"  Found {len(pairs)} reftest pairs")

        imported = 0
        skipped = 0

        for test_name, ref_name, _ in pairs:
            if imported >= args.max:
                break

            if should_skip(test_name):
                skipped += 1
                continue

            test_path = os.path.join(output_dir, category, test_name)
            ref_path = os.path.join(output_dir, category, ref_name)

            # 跳过已存在的文件
            if os.path.exists(test_path) and os.path.exists(ref_path):
                imported += 1
                continue

            test_url = f"{WPT_RAW}/{category}/{test_name}"
            ref_url = f"{WPT_RAW}/{category}/{ref_name}"

            if download_file(test_url, test_path) and download_file(ref_url, ref_path):
                print(f"  OK: {category}/{test_name}")
                imported += 1
            else:
                skipped += 1

        total_imported += imported
        total_skipped += skipped
        print(f"  Imported: {imported}, Skipped: {skipped}")

    print(f"\nTotal imported: {total_imported}")
    print(f"Total skipped: {total_skipped}")

    # 生成 manifest
    manifest = {"type": "reftest-manifest", "version": 1, "entries": []}
    for root, dirs, files in os.walk(output_dir):
        for f in files:
            if f.endswith((".xht", ".html")) and not f.endswith(("-ref.xht", "-ref.html", "-ref.html")):
                test_rel = os.path.relpath(os.path.join(root, f), output_dir)
                base = f.rsplit(".", 1)[0]
                ref_candidates = [f"{base}-ref.xht", f"{base}-ref.html"]
                ref_rel = None
                for rc in ref_candidates:
                    rp = os.path.join(root, rc)
                    if os.path.exists(rp):
                        ref_rel = os.path.relpath(rp, output_dir)
                        break
                if ref_rel:
                    manifest["entries"].append({
                        "test": test_rel.replace("\\", "/"),
                        "ref": ref_rel.replace("\\", "/"),
                        "relation": "=="
                    })

    manifest_path = os.path.join(output_dir, "reftest-manifest.json")
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)
    print(f"Manifest written: {manifest_path} ({len(manifest['entries'])} entries)")


if __name__ == "__main__":
    main()
