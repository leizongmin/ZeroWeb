#!/usr/bin/env python3
"""从 docs/learnings/<分类>/<YYYY-MM>/<YYYY-MM-DD>-<topic>.md 的 frontmatter 生成 INDEX.md。

同时校验布局契约：frontmatter date 是事实源，文件名日期前缀与月度目录必须由它派生，
不一致即报错退出（防漂移）。

用法：python3 scripts/gen-learnings-index.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEARNINGS = ROOT / "docs" / "learnings"
INDEX = LEARNINGS / "INDEX.md"

CATEGORIES = [
    ("bugs", "Bugs — 踩坑记录（根因 + 修复 + 如何避免）"),
    ("patterns", "Patterns — 可复用代码模式与最佳实践"),
    ("performance", "Performance — 性能优化经验"),
    ("platform", "Platform — 平台与环境相关经验"),
]

FM_RE = re.compile(
    r"\A---\ndate: (\d{4}-\d{2}-\d{2})\nmodules:[ \t]*(.*)\n---\n", re.DOTALL
)


def parse(f: Path) -> tuple[str, str, str] | None:
    """返回 (date, modules, title)；frontmatter 或标题缺失时返回 None。"""
    text = f.read_text(encoding="utf-8")
    m = FM_RE.match(text)
    if not m:
        return None
    title = ""
    for line in text[m.end() :].splitlines():
        if line.startswith("# "):
            title = line[2:].strip()
            break
    return m.group(1), m.group(2).strip(), title


def main() -> int:
    bad: list[str] = []
    sections: list[str] = []
    total = 0
    for cat, label in CATEGORIES:
        # 布局契约：<cat>/<YYYY-MM>/<YYYY-MM-DD>-<topic>.md
        files = sorted((LEARNINGS / cat).glob("*/*.md"))
        legacy = list((LEARNINGS / cat).glob("*.md"))
        for f in legacy:
            bad.append(f"{f.relative_to(ROOT)} (legacy flat layout)")
        rows = []
        for f in files:
            parsed = parse(f)
            if parsed is None:
                bad.append(f"{f.relative_to(ROOT)} (BAD frontmatter)")
                continue
            date, modules, title = parsed
            # 日期一致性：目录月度 = date[:7]，文件名前缀 = date
            if f.parent.name != date[:7] or not f.name.startswith(f"{date}-"):
                bad.append(
                    f"{f.relative_to(ROOT)} (date mismatch: fm={date})"
                )
                continue
            rows.append((date, f.name, title, modules))
        total += len(rows)
        lines = [f"## {label}（{len(rows)}）", ""]
        for date, name, title, modules in sorted(rows, reverse=True):
            disp = title or name.removesuffix(".md")
            mod = f" — {modules}" if modules else ""
            lines.append(f"- {date} [{disp}]({cat}/{date[:7]}/{name}){mod}")
        sections.append("\n".join(lines))

    header = (
        "# Learnings 索引\n\n"
        "> 本文件由 `scripts/gen-learnings-index.py` 生成，勿手改；\n"
        "> 新增 learning 后运行 `make learnings-index` 重建。\n"
        "> 布局契约：`<分类>/<YYYY-MM>/<YYYY-MM-DD>-<topic>.md`，日期以 frontmatter 为准。\n"
        "> 方法论蒸馏层见 `.agents/skills/zeroweb-guidelines/SKILL.md`。\n"
    )
    body = header + "\n" + "\n\n".join(sections) + "\n"
    INDEX.write_text(body, encoding="utf-8")
    print(f"INDEX.md: {total} entries")
    for b in bad:
        print(f"  BAD: {b}", file=sys.stderr)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
