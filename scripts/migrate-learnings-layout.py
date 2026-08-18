#!/usr/bin/env python3
"""一次性迁移脚本：learnings 目录改为 <分类>/<YYYY-MM>/<YYYY-MM-DD>-<topic>.md。

日期事实源是 frontmatter date；文件名与目录由它派生。迁移后：
1. git mv 每个文件到新路径；
2. 全仓改写指向旧路径的引用（.rs/.md/.yaml/.sh/.py，跳过 target/、.git/、
   ../ZeroUI 外部路径引用）；
3. 交叉引用（learning 互相引用）同样改写。

用法：python3 scripts/migrate-learnings-layout.py [--dry-run]
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEARNINGS = ROOT / "docs" / "learnings"

# 改写引用时扫描的文件类型
EXTS = {".rs", ".md", ".yaml", ".yml", ".sh", ".py", ".toml"}
SKIP_DIRS = {"target", ".git", "node_modules"}

FM_DATE_RE = re.compile(r"\A---\ndate: (\d{4}-\d{2}-\d{2})\n")


def build_renames() -> list[tuple[Path, Path, str]]:
    """返回 [(old, new, date)]；date 为 frontmatter 日期。"""
    renames: list[tuple[Path, Path, str]] = []
    for f in sorted(LEARNINGS.glob("*/*.md")):
        if f.name == "INDEX.md":
            continue
        if f.parent.parent != LEARNINGS:  # 已在 <cat>/<YYYY-MM>/ 下则跳过
            continue
        m = FM_DATE_RE.match(f.read_text(encoding="utf-8"))
        if not m:
            print(f"SKIP (no frontmatter date): {f}", file=sys.stderr)
            continue
        date = m.group(1)
        new = f.parent / date[:7] / f"{date}-{f.name}"
        renames.append((f, new, date))
    return renames


def rewrite_refs(renames: list[tuple[Path, Path, str]], dry: bool) -> int:
    """全仓把旧相对路径改写为新路径，返回改写的文件数。"""
    # old_rel 如 docs/learnings/bugs/x.md；同时构造纯文件名 -> 新相对路径映射，
    # 以覆盖跨行折断的引用（见 tokenizer.rs 场景，用两次单行替换处理）
    mapping: dict[str, str] = {}
    for old, new, _ in renames:
        old_rel = str(old.relative_to(ROOT))
        new_rel = str(new.relative_to(ROOT))
        mapping[old_rel] = new_rel

    changed = 0
    for path in ROOT.rglob("*"):
        if not path.is_file() or path.suffix not in EXTS:
            continue
        rel_parts = path.relative_to(ROOT).parts
        if any(p in SKIP_DIRS for p in rel_parts):
            continue
        if LEARNINGS in path.parents and path.name == "INDEX.md":
            continue  # 生成物，重建
        try:
            text = orig = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, PermissionError):
            continue
        for old_rel, new_rel in mapping.items():
            text = text.replace(old_rel, new_rel)
        # learning 内部相对链接（../<cat>/x.md 或 <cat>/x.md 形式，含 INDEX 之外的正文互链）
        for old, new, _ in renames:
            cat = old.parent.name
            old_frag = f"learnings/{cat}/{old.name}"
            new_frag = f"learnings/{cat}/{new.parent.name}/{new.name}"
            text = text.replace(old_frag, new_frag)
        if text != orig:
            changed += 1
            if dry:
                print(f"[dry] rewrite {path.relative_to(ROOT)}")
            else:
                path.write_text(text, encoding="utf-8")
    return changed


def main() -> int:
    dry = "--dry-run" in sys.argv
    renames = build_renames()
    print(f"planned renames: {len(renames)}")
    for old, new, _ in renames[:3]:
        print(f"  e.g. {old.relative_to(ROOT)} -> {new.relative_to(ROOT)}")

    if dry:
        rewrite_refs(renames, dry=True)
        return 0

    for old, new, _ in renames:
        new.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "mv", str(old), str(new)], cwd=ROOT, check=True)
    changed = rewrite_refs(renames, dry=False)
    print(f"rewrote refs in {changed} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
