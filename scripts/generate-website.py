#!/usr/bin/env python3
"""生成官网静态正文、语言版本与搜索元信息，复用已有指标和翻译。"""

import argparse
import html
from html.parser import HTMLParser
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
SITE = ROOT / "website"
ORIGIN = "https://zeroweb.leizm.com"
VOID = set("area base br col embed hr img input link meta param source track wbr".split())


class Node:
    """保留 HTML 结构，让翻译及静态内容替换不依赖嵌套标签正则。"""

    def __init__(self, tag="", attrs=(), text=""):
        self.tag, self.attrs, self.text = tag, dict(attrs), text
        self.children = []

    def walk(self):
        yield self
        for child in self.children:
            yield from child.walk()

    def render(self):
        if not self.tag:
            return self.text + "".join(child.render() for child in self.children)
        attrs = "".join(f' {key}="{html.escape(value or "", quote=True)}"' for key, value in self.attrs.items())
        body = "".join(child.render() for child in self.children)
        return f"<{self.tag}{attrs}>" + ("" if self.tag in VOID else body + f"</{self.tag}>")

    def replace(self, markup):
        self.children = [Node(text=markup)]


class PageParser(HTMLParser):
    """解析仓库模板，保留实体并校验元素闭合。"""

    def __init__(self, source):
        super().__init__(convert_charrefs=False)
        self.root = Node()
        self.stack = [self.root]
        self.feed(source)

    def handle_starttag(self, tag, attrs):
        node = Node(tag, attrs)
        self.stack[-1].children.append(node)
        if tag not in VOID:
            self.stack.append(node)

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        if tag not in VOID:
            self.handle_endtag(tag)

    def handle_endtag(self, tag):
        if self.stack[-1].tag != tag:
            raise ValueError(f"Unbalanced HTML: {tag}")
        self.stack.pop()

    def handle_data(self, data):
        self.stack[-1].children.append(Node(text=data))

    def handle_entityref(self, name):
        self.handle_data(f"&{name};")

    def handle_charref(self, name):
        self.handle_data(f"&#{name};")

    def handle_decl(self, decl):
        self.handle_data(f"<!{decl}>")

    def handle_comment(self, data):
        self.handle_data(f"<!--{data}-->")


def esc(value):
    return html.escape(str(value), quote=True)


def route(path, language):
    return ("/zh" if language == "zh" else "") + path


def language_links(path, language):
    """输出可抓取语言链接；每个译本拥有自身 canonical。"""
    # https://developers.google.com/search/docs/specialty/international/localized-versions
    current = ' aria-current="page"'
    return "".join(
        f'<a href="{route(path, lang)}" hreflang="{lang}" lang="{lang}"'
        f'{current if lang == language else ""}>{label}</a>'
        for lang, label in (("en", "EN"), ("zh", "中文"))
    )


def metadata(path, language, title, description, article_date=None):
    """在原始 HTML 中声明首选网址、分享摘要和与可见正文一致的结构化数据。"""
    url = ORIGIN + route(path, language)
    schema = {
        "@context": "https://schema.org", "@type": "WebPage",
        "name": title, "description": description, "url": url,
        "inLanguage": language, "isPartOf": {"@id": ORIGIN + "/#website"},
    }
    if path == "/":
        schema.update({"@type": "WebSite", "@id": ORIGIN + "/#website", "name": "ZeroWeb"})
    if article_date:
        schema.update({
            "@type": "Article", "headline": title, "datePublished": article_date,
            "author": {"@type": "Person", "name": "leizongmin", "url": "https://github.com/leizongmin"},
        })
    image = ORIGIN + f"/social-{language}.png"
    tags = [
        f"<title>{esc(title)}</title>",
        f'<meta name="description" content="{esc(description)}">',
        f'<link rel="canonical" href="{url}">',
        *(f'<link rel="alternate" hreflang="{lang}" href="{ORIGIN + route(path, lang)}">' for lang in ("en", "zh")),
        f'<link rel="alternate" hreflang="x-default" href="{ORIGIN + path}">',
    ]
    for key, value in {
        "og:type": "article" if article_date else "website", "og:site_name": "ZeroWeb",
        "og:title": title, "og:description": description, "og:url": url,
        "og:locale": "zh_CN" if language == "zh" else "en_US",
        "og:image": image, "og:image:width": "1200", "og:image:height": "630",
        "og:image:alt": "ZeroWeb — AI autonomous engineering · Rust browser",
    }.items():
        tags.append(f'<meta property="{key}" content="{esc(value)}">')
    for key, value in {"twitter:card": "summary_large_image", "twitter:title": title,
                       "twitter:description": description, "twitter:image": image}.items():
        tags.append(f'<meta name="{key}" content="{esc(value)}">')
    for env, name in (("GOOGLE_SITE_VERIFICATION", "google-site-verification"), ("BING_SITE_VERIFICATION", "msvalidate.01")):
        if os.environ.get(env):
            tags.append(f'<meta name="{name}" content="{esc(os.environ[env])}">')
    tags.append('<script type="application/ld+json">' + json.dumps(schema, ensure_ascii=False).replace("<", "\\u003c") + "</script>")
    return "\n".join(tags)


def words(language, en, zh):
    """选择分别编辑的中英文界面文案。"""
    return zh if language == "zh" else en


def paragraphs(text, language):
    """沿句子边界分段，保留完整周报文字，不以短摘要替代详情。"""
    sentences = re.split(r"(?<=。)" if language == "zh" else r"(?<=[.!?])\s+(?=[A-Z])", text)
    chunks, current = [], ""
    for sentence in sentences:
        if current and len(current) + len(sentence) > (220 if language == "zh" else 520):
            chunks.append(current)
            current = ""
        current += (" " if current and language == "en" else "") + sentence
    if current:
        chunks.append(current)
    return "".join(f"<p>{esc(chunk)}</p>" for chunk in chunks)


def contents(entries, language, label=None):
    """输出无需脚本即可使用的文章目录。"""
    label = label or words(language, "On this page", "本页目录")
    return (
        f'<nav class="article-toc" aria-label="{esc(label)}">'
        f'<strong>{esc(label)}</strong><ul>' +
        "".join(f'<li><a href="#{esc(anchor)}">{esc(title)}</a></li>' for anchor, title in entries) +
        "</ul></nav>"
    )


def update_card(item, language, copy, detail=False):
    """列表使用独立摘要，详情按阅读顺序呈现全文、任务和来源。"""
    path = route(f'/updates/{item["date"]}-{item["type"]}/', language)
    heading = "h1" if detail else "h2"
    title = esc(item["title"][language])
    if not detail:
        title = f'<a href="{path}">{title}</a>'
    sources = "".join(f'<a href="{esc(source["url"])}">{esc(source["label"])} ↗</a>' for source in item["sources"])
    header = (
        f'<article class="update-card"><div class="update-meta"><time datetime="{item["date"]}">{item["date"]}</time>'
        f'<span class="update-type">{esc(copy[item["type"]])}</span></div><{heading}>{title}</{heading}>'
    )
    teaser = item.get("teaser", item["summary"])[language]
    if not detail:
        return header + f'<p>{esc(teaser)}</p><a class="text-link read-update" href="{path}">{words(language, "Read update", "阅读全文")} →</a></article>'
    entries = [("overview", words(language, "Overview", "本期概览")),
               ("completed", copy["completed"]), ("in-progress", copy["inProgress"]), ("sources", copy["sources"])]
    completed = "".join(f"<li>{paragraphs(entry, language)}</li>" for entry in item["completed"][language])
    return (
        header + f'<p class="intro">{esc(teaser)}</p>' + contents(entries, language) +
        f'<section id="overview"><h2>{entries[0][1]}</h2>{paragraphs(item["summary"][language], language)}</section>'
        f'<div class="update-detail"><section id="completed"><h2>{copy["completed"]}</h2><ul>{completed}</ul></section>'
        f'<section id="in-progress"><h2>{copy["inProgress"]}</h2>{paragraphs(item["inProgress"][language], language)}</section></div>'
        f'<section id="sources"><h2>{copy["sources"]}</h2><div class="source-links">{sources}</div></section></article>'
    )


def guide_links(guides, language, exclude=None):
    return "".join(
        f'<article class="update-summary"><h3><a href="{route("/guides/" + guide["slug"] + "/", language)}">'
        f'{esc(guide["title"][language])}</a></h3><p>{esc(guide["description"][language])}</p></article>'
        for guide in guides if guide["slug"] != exclude
    )


def guide_body(guide, language):
    """为指南章节提供稳定锚点，并把已有管线清单呈现为流程。"""
    tree = PageParser(guide["body"][language]).root
    entries = []
    for node in tree.walk():
        if node.tag == "h2":
            anchor = f"section-{len(entries) + 1}"
            node.attrs["id"] = anchor
            label = html.unescape("".join(child.render() for child in node.children))
            entries.append((anchor, label))
        if guide["slug"] == "rust-browser-architecture" and node.tag == "ol":
            node.attrs["class"] = "guide-pipeline"
    return contents(entries, language) + tree.render()


def update_archive(updates, language, labels):
    """按月份分组，让归档保留可抓取的紧凑列表。"""
    months = {}
    for item in updates:
        months.setdefault(item["date"][:7], []).append(item)
    links = contents([(f"month-{month}", month) for month in months], language, words(language, "Browse by month", "按月份浏览"))
    return links + "".join(
        f'<section class="update-month" id="month-{month}"><h2>{month}</h2>' +
        "".join(update_card(item, language, labels) for item in items) + "</section>"
        for month, items in months.items()
    )


def hero_wptfyi(data, language, copy):
    """首页 hero 的全量通过率大数字：数字取 coverage 快照，文案复用 script.js 翻译。"""
    totals = data["totals"]
    rate = f"{totals['pass_rate'] * 100:.2f}%"
    subtests = f"{totals['zw_passes']:,} / {totals['den_subtests']:,}"
    return (
        f'<small class="stat-label">{esc(copy["wptfyiHeroLabel"])}</small><strong>{esc(rate)}</strong>'
        f'<span>{esc(subtests)} {esc(copy["wptfyiHeroNote"])} · '
        f'<time datetime="{esc(data["date"])}">{esc(data["date"])}</time></span>'
        f'<a class="stat-link" href="{route("/progress.html", language)}">{esc(copy["wptfyiHeroCta"])} →</a>'
    )


def page(template, language, path, copy, title, description, replacements=None, extra="", article_date=None, reveal=()):
    tree = PageParser(template).root
    if not any(node.attrs.get("class") == "skip-link" for node in tree.walk()):
        target = "#article-content" if path.startswith(("/guides/", "/updates/")) else "#main"
        for node in tree.walk():
            if node.tag == "body":
                node.children.insert(0, Node(text=f'<a class="skip-link" href="{target}">{words(language, "Skip to content", "跳至主要内容")}</a>'))
                break
    for node in list(tree.walk()):
        if node.tag == "html":
            node.attrs["lang"] = "zh-CN" if language == "zh" else "en"
        for key in ("data-i18n", "data-i18n-html"):
            if key in node.attrs:
                value = copy[node.attrs.pop(key)]
                node.replace(value if key.endswith("-html") else esc(value))
        if node.tag == "head":
            node.children = [
                child for child in node.children
                if child.tag != "title" and not (child.tag == "meta" and child.attrs.get("name") == "description")
            ]
            node.children.append(Node(text=metadata(path, language, title, description, article_date) + '<link rel="stylesheet" href="/seo.css"><link rel="stylesheet" href="/layout.css">'))
        if "language-switcher" in node.attrs.get("class", "").split():
            node.replace(language_links(path, language))
        if node.tag == "nav" and node.attrs.get("aria-label") == "Primary navigation":
            for child in node.children:
                child.attrs.pop("aria-current", None)
                href = child.attrs.get("href", "")
                if (path.startswith("/updates/") or path == "/updates.html") and href == "updates.html":
                    child.attrs["aria-current"] = "page"
                if path == "/progress.html" and href == "progress.html":
                    child.attrs["aria-current"] = "page"
            current = ' aria-current="page"' if path.startswith("/guides/") else ""
            node.children.append(Node(text=f'<a href="{route("/guides/", language)}"{current}>'
                                      f'{words(language, "Guides", "指南")}</a>'))
        if node.tag == "main":
            if not node.attrs.get("id"):
                node.attrs["id"] = "main"
            if path.startswith(("/guides/", "/updates/")) and path != "/guides/":
                node.attrs["class"] = "updates-page article-page"
        if node.tag == "a" and node.attrs.get("class") == "skip-link" and path.startswith(("/guides/", "/updates/")):
            node.attrs["href"] = "#article-content"
        for attr in ("href", "src"):
            value = node.attrs.get(attr, "")
            if value.startswith("index.html"):
                node.attrs[attr] = route("/", language) + value[len("index.html"):]
            elif value in ("progress.html", "updates.html"):
                node.attrs[attr] = route("/" + value, language)
            elif value.startswith("guides/"):
                node.attrs[attr] = route("/" + value, language)
            elif value and not value.startswith(("/", "#", "http", "mailto:")):
                node.attrs[attr] = "/" + value
        if replacements and node.attrs.get("id") in replacements:
            node.replace(replacements[node.attrs["id"]])
        if node.attrs.get("id") in reveal:
            node.attrs.pop("hidden", None)
        if node.tag == "main" and extra:
            node.children.append(Node(text=extra))
        if node.tag == "script" and node.attrs.get("src") in ("/script.js", "/updates.js"):
            node.tag = ""
            node.text = ""
            node.children = []
    return tree.render()


def build(output):
    """生成独立部署目录；拒绝覆盖模板及已有非生成资产。"""
    output = output.resolve()
    if output == ROOT or output == SITE or ROOT.is_relative_to(output) or output.is_relative_to(SITE):
        raise ValueError("Output must be outside the website source directory")
    marker = output / ".zeroweb-site"
    if output.exists() and any(output.iterdir()) and not marker.exists():
        raise ValueError("Refusing to overwrite a non-generated directory")
    output.mkdir(parents=True, exist_ok=True)
    marker.write_text("Generated by scripts/generate-website.py\n")
    for asset in SITE.iterdir():
        if asset.is_file() and asset.suffix in (".css", ".svg", ".png", ".js", ".json"):
            shutil.copy2(asset, output / asset.name)
    spec = importlib.util.spec_from_file_location("metrics", ROOT / "scripts/generate-website-metrics.py")
    metrics_module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(metrics_module)
    metrics = metrics_module.build_metrics_with_coverage(
        ROOT / "docs/perf/trends/benchmark-trend.csv",
        ROOT / "docs/goal/rendering-compat/evidence/wpt-trends/trend.csv",
        ROOT / "docs/compat/trends/wpt-suites.csv",
        ROOT / "docs/compat/wptfyi",
    )
    wptfyi_totals = (metrics.get("wptfyi") or {}).get("totals")
    (output / "metrics.json").write_text(json.dumps(metrics, ensure_ascii=False), encoding="utf8")
    updates = sorted(json.loads((SITE / "updates.json").read_text()), key=lambda item: item["date"], reverse=True)
    if len({(item["date"], item["type"]) for item in updates}) != len(updates):
        raise ValueError("Update URLs must be unique")
    for item in updates:
        if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", item["date"]) or item["type"] not in ("weekly", "launch"):
            raise ValueError("Invalid update date or type")
        for source in item["sources"]:
            if not source["url"].startswith("https://"):
                raise ValueError("Update sources must use HTTPS")
    guides = json.loads((SITE / "guides.json").read_text())
    urls = []

    def write(path, language, source):
        target = output / route(path, language).lstrip("/")
        if path.endswith("/"):
            target = target / "index.html"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(source, encoding="utf8")
        urls.append((path, language))

    for language in ("en", "zh"):
        rendered = json.loads(subprocess.check_output(
            ["node", str(ROOT / "scripts/website-render.mjs"), language, str(output / "metrics.json")], text=True))
        labels = rendered["updates"]
        guide_title = "Engineering guides" if language == "en" else "工程指南"
        def guide_section(exclude=None):
            return f'<section class="section guide-links"><div class="section-heading"><h2>{guide_title}</h2></div><div class="guide-grid">{guide_links(guides, language, exclude)}</div></section>'
        extra = guide_section()
        titles = {
            "/": ("ZeroWeb — AI-Autonomous Engineering with a Rust Browser", "ZeroWeb — 以 Rust 浏览器验证 AI 自主软件工程"),
            "/progress.html": ("ZeroWeb WPT Results & Performance Benchmarks", "ZeroWeb — WPT 测试结果与性能基准"),
            "/updates.html": ("ZeroWeb Development Updates", "ZeroWeb — Rust 浏览器开发进展"),
        }
        descriptions = {
            "/": ("ZeroWeb explores AI-autonomous software engineering through an experimental Rust browser and embeddable ZeroWebView, with public code, tests and progress.", "ZeroWeb 以实验性 Rust 浏览器和可嵌入的 ZeroWebView 验证 AI 自主软件工程，公开代码、测试结果和开发进展。"),
            "/progress.html": ("Explore ZeroWeb's WPT results, corpus versions and performance trends. CSS self-source reftests measure self-consistency, not cross-browser compatibility.", "查看 ZeroWeb 的 WPT 测试结果、语料版本与性能趋势。CSS self-source reftest 衡量自身一致性，不等同于跨浏览器兼容率。"),
            "/updates.html": ("Follow ZeroWeb's Rust browser development: completed work, remaining tasks and source-linked weekly updates from the AI engineering experiment.", "查看 ZeroWeb Rust 浏览器每周开发进展、已完成工作与剩余任务，每条更新均链接到可核查的来源。"),
        }
        index = (SITE / "index.html").read_text()
        cards = "".join(
            f'<article class="update-summary"><time datetime="{item["date"]}">{item["date"]}</time>'
            f'<h3><a href="{route("/updates/" + item["date"] + "-" + item["type"] + "/", language)}">{esc(item["title"][language])}</a></h3>'
            f'<p>{esc(item.get("teaser", item["summary"])[language])}</p>'
            f'<a class="text-link read-update" href="{route("/updates/" + item["date"] + "-" + item["type"] + "/", language)}">{words(language, "Read update", "阅读全文")} →</a></article>' for item in updates[:3])
        index_replacements = {"home-update-list": cards, "hero-wptfyi-stat": hero_wptfyi(metrics["wptfyi"], language, rendered["home"]) if wptfyi_totals else ""}
        write("/", language, page(index, language, "/", rendered["home"], titles["/"][language == "zh"], descriptions["/"][language == "zh"], index_replacements, extra))
        progress = {key: value["innerHTML"] or esc(value["textContent"]) for key, value in rendered["elements"].items()}
        # 结构容器（如 wptfyi-section）在渲染器里只有空 innerHTML 条目；按 id 替换会把
        # 模板子元素清成空壳，运行时再取其子节点即抛错且被静默吞掉。空内容不替换。
        progress = {key: markup for key, markup in progress.items() if markup}
        # Keep all benchmark histories readable before JS selects a single metric.
        histories = "".join(
            f'<details><summary>{esc(metric["label"][language])} ({esc(metric["unit"])})</summary>'
            f'<div class="chart-table" tabindex="0" role="region" aria-label="{esc(metric["label"][language])}"><table><thead><tr><th scope="col">{"Date" if language == "en" else "日期"}</th><th scope="col">p95</th></tr></thead><tbody>' +
            "".join(f'<tr><td>{point["date"]}</td><td>{point["value"]} {esc(metric["unit"])}</td></tr>' for point in metric["points"]) +
            "</tbody></table></div></details>" for metric in metrics["performance"]["metrics"])
        note = f'<section class="chart-section"><details><summary>{words(language, "All benchmark histories", "全部性能历史")}</summary>{histories}</details></section>'
        write("/progress.html", language, page((SITE / "progress.html").read_text(), language, "/progress.html", rendered["progress"], titles["/progress.html"][language == "zh"], descriptions["/progress.html"][language == "zh"], progress, note + extra, reveal=("wptfyi-section",) if wptfyi_totals else ()))
        template = (SITE / "updates.html").read_text()
        write("/updates.html", language, page(template, language, "/updates.html", labels, titles["/updates.html"][language == "zh"], descriptions["/updates.html"][language == "zh"], {"update-list": update_archive(updates, language, labels)}, extra))
        # Reuse the archive shell for focused articles, preserving navigation.
        shell = PageParser(template).root
        for node in shell.walk():
            if node.tag == "main":
                node.attrs["id"] = "article-content"
            if node.tag == "script":
                node.tag, node.text, node.children = "", "", []
        for i, item in enumerate(updates):
            path = f'/updates/{item["date"]}-{item["type"]}/'
            adjacent = [updates[j] for j in (i - 1, i + 1) if 0 <= j < len(updates)]
            links = "".join(f'<p><a class="text-link" href="{route("/updates/" + entry["date"] + "-" + entry["type"] + "/", language)}">{esc(entry["title"][language])}</a></p>' for entry in adjacent)
            body = update_card(item, language, labels, True) + f'<nav class="adjacent-updates" aria-label="{words(language, "Adjacent updates", "相邻更新")}">{links}<p><a class="text-link" href="{route("/updates.html", language)}">{words(language, "All updates", "全部更新")} →</a></p></nav>'
            write(path, language, page(shell.render(), language, path, labels, item["title"][language] + " — ZeroWeb", item.get("teaser", item["summary"])[language], {"article-content": body}, extra, item["date"]))
        directory = f'<h1>{guide_title}</h1><p class="intro">{words(language, "Explore the experiment, architecture, local setup and measurement methods.", "了解实验方法、浏览器架构、本地运行与数据口径。")}</p><div class="guide-grid">{guide_links(guides, language)}</div>'
        write("/guides/", language, page(shell.render(), language, "/guides/", labels, guide_title + " — ZeroWeb", words(language, "Engineering guides for ZeroWeb and ZeroWebView.", "ZeroWeb 与 ZeroWebView 工程指南。"), {"article-content": directory}))
        for guide in guides:
            path = "/guides/" + guide["slug"] + "/"
            body = f'<article class="guide"><h1>{esc(guide["title"][language])}</h1><p class="intro">{esc(guide["description"][language])}</p>{guide_body(guide, language)}</article>'
            write(path, language, page(shell.render(), language, path, labels, guide["title"][language] + " — ZeroWeb", guide["description"][language], {"article-content": body}, guide_section(guide["slug"])))
    # Sitemaps list canonical URLs only; no fabricated lastmod dates.
    # https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap
    ET.register_namespace("", "http://www.sitemaps.org/schemas/sitemap/0.9")
    ET.register_namespace("xhtml", "http://www.w3.org/1999/xhtml")
    sitemap = ET.Element("{http://www.sitemaps.org/schemas/sitemap/0.9}urlset")
    for path, language in urls:
        node = ET.SubElement(sitemap, "{http://www.sitemaps.org/schemas/sitemap/0.9}url")
        ET.SubElement(node, "{http://www.sitemaps.org/schemas/sitemap/0.9}loc").text = ORIGIN + route(path, language)
        for lang in ("en", "zh"):
            ET.SubElement(node, "{http://www.w3.org/1999/xhtml}link", rel="alternate", hreflang=lang, href=ORIGIN + route(path, lang))
    ET.ElementTree(sitemap).write(output / "sitemap.xml", encoding="utf-8", xml_declaration=True)
    (output / "robots.txt").write_text(f"User-agent: *\nAllow: /\nSitemap: {ORIGIN}/sitemap.xml\n")
    (output / ".nojekyll").touch()
    print(f"Generated {len(urls)} pages")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    build(parser.parse_args().output)
