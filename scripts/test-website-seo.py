#!/usr/bin/env python3
"""验证部署产物中的语言互链、静态正文、元信息及全部本地资源。"""

from html.parser import HTMLParser
import html
import json
from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlsplit
import xml.etree.ElementTree as ET


class Document(HTMLParser):
    """采集搜索与用户可见的 HTML 结构，不执行 JavaScript。"""

    def __init__(self, source):
        super().__init__()
        self.tags = []
        self.head_titles = 0
        self.in_head = False
        self.text = []
        self.feed(source)

    def handle_data(self, data):
        self.text.append(data)

    def handle_starttag(self, tag, attrs):
        if tag == "head":
            self.in_head = True
        if tag == "title" and self.in_head:
            self.head_titles += 1
        self.tags.append((tag, dict(attrs)))

    def handle_endtag(self, tag):
        if tag == "head":
            self.in_head = False

    def select(self, tag, **attrs):
        return [data for name, data in self.tags if name == tag and all(data.get(key) == value for key, value in attrs.items())]


def verify(root):
    """验证所有首选页面及译本，确保发布不依赖爬虫执行脚本。"""
    origin = "https://zeroweb.leizm.com"
    pages = sorted(root.rglob("*.html"))
    sitemap = ET.parse(root / "sitemap.xml")
    locations = {node.text for node in sitemap.findall(".//{*}loc")}
    assert len(locations) == len(pages), "Sitemap must cover every canonical page"
    updates = json.loads((root / "updates.json").read_text())
    guides = json.loads((root / "guides.json").read_text())
    for file in pages:
        source = file.read_text()
        doc = Document(source)
        canonical = doc.select("link", rel="canonical")
        assert len(canonical) == 1, file
        url = canonical[0]["href"]
        assert url in locations and url.startswith(origin), file
        path = urlsplit(url).path
        expected = root / path.lstrip("/")
        if path.endswith("/"):
            expected /= "index.html"
        assert expected == file, (file, url)
        chinese = path.startswith("/zh/")
        language = "zh" if chinese else "en"
        visible = re.sub(r"\s+", "", "".join(doc.text))
        ids = {attrs["id"] for _, attrs in doc.tags if "id" in attrs}
        assert doc.select("html", lang="zh-CN" if chinese else "en"), file
        assert len(doc.select("h1")) == 1, file
        assert doc.head_titles == 1, file
        descriptions = doc.select("meta", name="description")
        assert len(descriptions) == 1 and descriptions[0]["content"], file
        assert doc.select("meta", property="og:url")[0]["content"] == url, file
        assert doc.select("meta", name="twitter:card"), file
        assert doc.select("script", type="application/ld+json"), file
        if chinese:
            assert any("\u4e00" <= char <= "\u9fff" for char in descriptions[0]["content"]), file
        base_path = path[3:] if chinese else path
        for alternate_language in ("en", "zh"):
            alternate = doc.select("link", rel="alternate", hreflang=alternate_language)
            target = origin + ("/zh" if alternate_language == "zh" else "") + base_path
            assert len(alternate) == 1 and alternate[0]["href"] == target, file
            assert doc.select("a", hreflang=alternate_language), file
        for tag, attrs in doc.tags:
            for attr in ("src", "href"):
                value = attrs.get(attr)
                if not value or urlsplit(value).scheme:
                    continue
                if value.startswith("#"):
                    assert unquote(value[1:]) in ids, (file, value)
                    continue
                assert value.startswith("/"), (file, value)
                local = root / unquote(urlsplit(value).path).lstrip("/")
                if local.is_dir():
                    local /= "index.html"
                assert local.is_file(), (file, value)
            if tag == "meta" and attrs.get("property") == "og:image":
                image = root / urlsplit(attrs["content"]).path.lstrip("/")
                assert image.is_file() and image.read_bytes().startswith(b"\x89PNG\r\n\x1a\n"), image
        if base_path == "/":
            for item in updates[:3]:
                assert item["title"]["zh" if chinese else "en"] in html.unescape(source), file
                assert html.escape(item.get("teaser", item["summary"])[language]) in source, file
                if "teaser" in item:
                    assert html.escape(item["summary"][language]) not in source, file
        if base_path == "/updates.html":
            assert len(doc.select("article", **{"class": "update-card"})) == len(updates), file
            months = {item["date"][:7] for item in updates}
            assert {f"month-{month}" for month in months} <= ids, file
            assert not doc.select("div", **{"class": "update-detail"}), file
        if base_path.startswith("/updates/"):
            item = next(item for item in updates if base_path == f'/updates/{item["date"]}-{item["type"]}/')
            for text in [item["summary"][language], *item["completed"][language], item["inProgress"][language]]:
                assert re.sub(r"\s+", "", text) in visible, (file, "Missing full update text")
            assert {"overview", "completed", "in-progress", "sources"} <= ids, file
            assert doc.select("nav", **{"class": "article-toc"}), file
        if base_path == "/guides/":
            for guide in guides:
                assert doc.select("a", href=("/zh" if chinese else "") + f'/guides/{guide["slug"]}/'), file
        elif base_path.startswith("/guides/"):
            assert doc.select("nav", **{"class": "article-toc"}), file
            assert len(doc.select("a", href=path)) == 1, (file, "Guide must not recommend itself")
        assert doc.select("a", href=("/zh" if chinese else "") + "/guides/"), file
        if base_path == "/progress.html":
            assert len(doc.select("table")) >= 3 and doc.select("option"), file
            assert "self-source reftest" in source, file
            assert len(doc.select("details")) >= 6, file
            assert doc.select("div", id="wptfyi-gap-preview"), file
        assert "data-i18n" not in source, file
    assert f"Sitemap: {origin}/sitemap.xml" in (root / "robots.txt").read_text()
    print(f"PASS: {len(pages)} pages, static content, language pairs, metadata and local links")


if __name__ == "__main__":
    verify(Path(sys.argv[1]).resolve())
