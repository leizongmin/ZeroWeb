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


def update_card(item, language, copy, detail=False):
    path = route(f'/updates/{item["date"]}-{item["type"]}/', language)
    heading = "h1" if detail else "h2"
    title = esc(item["title"][language])
    if not detail:
        title = f'<a href="{path}">{title}</a>'
    sources = "".join(f'<a href="{esc(source["url"])}">{esc(source["label"])} ↗</a>' for source in item["sources"])
    completed = "".join(f"<li>{esc(entry)}</li>" for entry in item["completed"][language])
    return (
        f'<article class="update-card"><div class="update-meta"><time datetime="{item["date"]}">{item["date"]}</time>'
        f'<span class="update-type">{esc(copy[item["type"]])}</span></div><{heading}>{title}</{heading}>'
        f'<p>{esc(item["summary"][language])}</p><div class="update-detail"><div><h3>{copy["completed"]}</h3>'
        f'<ul>{completed}</ul></div><div><h3>{copy["inProgress"]}</h3><p>{esc(item["inProgress"][language])}</p>'
        f'</div></div><div class="source-links"><span>{copy["sources"]}:</span>{sources}</div></article>'
    )


def guide_links(guides, language):
    return "".join(
        f'<article class="update-summary"><h3><a href="{route("/guides/" + guide["slug"] + "/", language)}">'
        f'{esc(guide["title"][language])}</a></h3><p>{esc(guide["description"][language])}</p></article>'
        for guide in guides
    )


def page(template, language, path, copy, title, description, replacements=None, extra="", article_date=None):
    tree = PageParser(template).root
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
            node.children.append(Node(text=metadata(path, language, title, description, article_date) + '<link rel="stylesheet" href="/seo.css">'))
        if "language-switcher" in node.attrs.get("class", "").split():
            node.replace(language_links(path, language))
        for attr in ("href", "src"):
            value = node.attrs.get(attr, "")
            if value.startswith("index.html"):
                node.attrs[attr] = route("/", language) + value[len("index.html"):]
            elif value in ("progress.html", "updates.html"):
                node.attrs[attr] = route("/" + value, language)
            elif value and not value.startswith(("/", "#", "http", "mailto:")):
                node.attrs[attr] = "/" + value
        if replacements and node.attrs.get("id") in replacements:
            node.replace(replacements[node.attrs["id"]])
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
    metrics = metrics_module.build_metrics(
        ROOT / "docs/perf/trends/benchmark-trend.csv",
        ROOT / "docs/goal/rendering-compat/evidence/wpt-trends/trend.csv",
        ROOT / "docs/compat/trends/wpt-suites.csv",
    )
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
        extra = f'<section class="section"><div class="section-heading"><h2>{guide_title}</h2></div><div class="experiment-grid">{guide_links(guides, language)}</div></section>'
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
            f'<p>{esc(item["summary"][language])}</p></article>' for item in updates[:3])
        write("/", language, page(index, language, "/", rendered["home"], titles["/"][language == "zh"], descriptions["/"][language == "zh"], {"home-update-list": cards}, extra))
        progress = {key: value["innerHTML"] or esc(value["textContent"]) for key, value in rendered["elements"].items()}
        # Keep all benchmark histories readable before JS selects a single metric.
        histories = "".join(
            f'<details><summary>{esc(metric["label"][language])} ({esc(metric["unit"])})</summary>'
            f'<table><thead><tr><th scope="col">{"Date" if language == "en" else "日期"}</th><th scope="col">p95</th></tr></thead><tbody>' +
            "".join(f'<tr><td>{point["date"]}</td><td>{point["value"]} {esc(metric["unit"])}</td></tr>' for point in metric["points"]) +
            "</tbody></table></details>" for metric in metrics["performance"]["metrics"])
        note = f'<section class="chart-section"><h2>{"All benchmark histories" if language == "en" else "全部性能历史"}</h2>{histories}</section>'
        write("/progress.html", language, page((SITE / "progress.html").read_text(), language, "/progress.html", rendered["progress"], titles["/progress.html"][language == "zh"], descriptions["/progress.html"][language == "zh"], progress, note + extra))
        template = (SITE / "updates.html").read_text()
        write("/updates.html", language, page(template, language, "/updates.html", labels, titles["/updates.html"][language == "zh"], descriptions["/updates.html"][language == "zh"], {"update-list": "".join(update_card(item, language, labels) for item in updates)}, extra))
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
            body = update_card(item, language, labels, True) + links
            write(path, language, page(shell.render(), language, path, labels, item["title"][language] + " — ZeroWeb", item["summary"][language], {"article-content": body}, extra, item["date"]))
        for guide in guides:
            path = "/guides/" + guide["slug"] + "/"
            body = f'<article class="guide"><h1>{esc(guide["title"][language])}</h1><p class="intro">{esc(guide["description"][language])}</p>{guide["body"][language]}</article>'
            write(path, language, page(shell.render(), language, path, labels, guide["title"][language] + " — ZeroWeb", guide["description"][language], {"article-content": body}, extra))
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
