#!/usr/bin/env python3
"""一次性迁移脚本：归一化 docs/learnings 的文件名与 frontmatter。

- git mv tools/stale-renderer-binary-browser-tests.md -> platform/
- git mv 去掉文件名日期后缀（如 cjk-raster-face-cache-2026-08-07.md）
- 将 6 种既有 header 变体统一改写为 YAML frontmatter：
    ---
    date: YYYY-MM-DD
    modules: 模块列表（纯文本，去反引号/顿号分隔）
    ---
  标题行（# ...）保留在 frontmatter 之后。

用法：python3 scripts/normalize-learnings.py [--dry-run]
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEARNINGS = ROOT / "docs" / "learnings"

# 匹配既有多变体日期/模块行，如：
#   - 日期：2026-08-17            **日期**: 2026-08-13（R3354）
#   Date: 2026-08-13              日期：2026-08-14 ｜ 模块：a、b
DATE_RE = re.compile(
    r"^(?:[-*]+\s*)?\*{0,2}(?:日期|Date)\*{0,2}\s*[：:]\s*(\d{4}-\d{2}-\d{2})"
)
MODULES_RE = re.compile(
    r"^(?:[-*]+\s*)?\*{0,2}(?:相关模块|Modules?|模块)\*{0,2}\s*[：:]\s*(.+)$"
)


def parse_header(text: str) -> tuple[str, str, str, str] | None:
    """返回 (title, date, modules, rest) 或 None（无法解析时）。

    title 为首个 `# ` 标题行（含换行），无标题时为空串。
    """
    date = None
    modules = None
    lines = text.splitlines(keepends=True)
    title = ""
    i = 0
    if lines and lines[0].lstrip().startswith("# "):
        title = lines[0]
        i = 1
    while i < len(lines) and i < 10:
        line = lines[i]
        if date is None:
            m = DATE_RE.match(line.strip())
            if m:
                date = m.group(1)
                i += 1
                continue
        if modules is None:
            m = MODULES_RE.match(line.strip())
            if m:
                modules = m.group(1)
                i += 1
                continue
        if line.strip() == "":
            i += 1
            continue
        break
    if date is None:
        return None
    return title, date, modules or "", "".join(lines[i:])


def clean_modules(raw: str) -> str:
    """顿号/加号分隔的模块列表 -> 逗号分隔纯文本。

    注意不按 `/` 拆分：模块引用多为路径（apps/browser/src/x.rs）。
    括号补充说明（如 `x.rs（CanvasContext::new）`）保留原样——信息有用，
    强行剥除容易误伤路径本身的括号。
    """
    parts = re.split(r"[、､,，;；]+|\s+\+\s+", raw.replace("`", "").strip())
    seen: list[str] = []
    for p in parts:
        p = p.strip().rstrip("。.").strip()
        if p and p not in seen:
            seen.append(p)
    return ", ".join(seen)


def main() -> int:
    dry = "--dry-run" in sys.argv
    moved, rewritten, skipped = [], [], []

    # 1) tools/ 目录并入 platform/
    tools_dir = LEARNINGS / "tools"
    if tools_dir.is_dir():
        for f in sorted(tools_dir.glob("*.md")):
            dest = LEARNINGS / "platform" / f.name
            if dry:
                print(f"[dry] git mv {f.relative_to(ROOT)} -> {dest.relative_to(ROOT)}")
            else:
                subprocess.run(["git", "mv", str(f), str(dest)], cwd=ROOT, check=True)
            moved.append(dest)
        if not dry:
            (tools_dir).rmdir()

    # 2) 去掉文件名日期后缀
    for f in sorted(LEARNINGS.glob("*/*.md")):
        if f.name == "INDEX.md":
            continue
        new_name = re.sub(r"-\d{4}-\d{2}-\d{2}(?=\.md$)", "", f.name)
        if new_name != f.name:
            dest = f.with_name(new_name)
            if dest.exists():
                skipped.append(f"{f} -> {dest} (exists)")
                continue
            if dry:
                print(f"[dry] git mv {f.name} -> {new_name}")
                f = dest  # dry-run 下后续步骤仅打印，dest 可能不存在
            else:
                subprocess.run(["git", "mv", str(f), str(dest)], cwd=ROOT, check=True)
                f = dest

        # 3) frontmatter 改写
        if not f.exists():
            skipped.append(f"{f.relative_to(ROOT)} (dry-run rename)")
            continue
        text = f.read_text(encoding="utf-8")
        if text.startswith("---\n"):
            continue
        parsed = parse_header(text)
        if parsed is None:
            # 日期也可能写在标题行尾部括号里，如「# …（M1-S1，2026-08-08）」
            first = text.splitlines()[0] if text.splitlines() else ""
            m = re.search(r"(\d{4}-\d{2}-\d{2})", first)
            if not m:
                skipped.append(f"{f.relative_to(ROOT)} (no date header)")
                continue
            # 正文原样保留（含标题行），日期取自标题
            rest_lines = text.splitlines(keepends=True)
            title = rest_lines[0] if rest_lines and rest_lines[0].lstrip().startswith("# ") else ""
            body = "".join(rest_lines[1:]) if title else text
            parsed = (title, m.group(1), "", body)
        title, date, modules, rest = parsed
        fm = f"---\ndate: {date}\nmodules: {clean_modules(modules)}\n---\n\n"
        out = fm + title
        if title and not rest.startswith("\n"):
            out += "\n"
        out += rest
        if dry:
            print(f"[dry] rewrite {f.relative_to(ROOT)}")
        else:
            f.write_text(out, encoding="utf-8")
        rewritten.append(f)

    print(f"\nmoved={len(moved)} rewritten={len(rewritten)} skipped={len(skipped)}")
    for s in skipped:
        print(f"  SKIP {s}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
