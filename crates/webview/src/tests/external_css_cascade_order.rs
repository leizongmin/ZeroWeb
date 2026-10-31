//! s174（html-semantics-compat M2）：webview 静态外链 CSS entries 化——级联文档序。
//! s167 挂账收口：静态发布（`prepare_page_subresources` → `load_html`）此前把外链
//! 表合并为单 blob 整体前置（`set_active_external_css`），link 在 `<style>` 之后的
//! 页同特异度恒输（blob 无位次，CSS Cascade 5 author origin 按引入元素文档序）。
//! 条目化后走 engine collector 文档序交错装配（`collect_interleaved_external`）。
//! 探针取值在页内脚本同步完成（gCS 面与布局级联同线程同槽）。
//!
//! https://drafts.csswg.org/css-cascade-5/#cascade-sort

use std::collections::HashMap;
use std::sync::Arc;

use crate::{WebView, WebViewConfig};

const PAGE_URL: &str = "https://wpt.test/cascade-order/probe.html";

fn webview_with_css(files: HashMap<String, String>) -> WebView {
    WebView::new(WebViewConfig {
        image_source_fetcher: Some(Arc::new(move |url| files.get(url).map(|css| css.clone().into_bytes()))),
        ..WebViewConfig::default()
    })
}

/// 加载页面（含外链 css 供给）→ 跑页面脚本 → 读脚本内取值。
fn load_and_probe(webview: &mut WebView, html: &str) -> String {
    webview.prepare_document_state(PAGE_URL);
    webview.complete_fetched_page(html, PAGE_URL);
    webview.run_page_scripts_strict().unwrap();
    webview.execute_script("globalThis.__zwProbe").unwrap()
}

/// link 在文档 `<style>` **之后**：同特异度 link 规则后到后胜 → `fixed`。
/// 红态（blob 前置）：外链整体先于文档表 → 文档 `<style>` 恒胜 → `static`。
#[test]
fn link_after_style_wins_by_document_order() {
    let mut webview = webview_with_css(HashMap::from([(
        "https://wpt.test/cascade-order/late.css".to_string(),
        "#test { position: fixed }".to_string(),
    )]));
    assert_eq!(
        load_and_probe(
            &mut webview,
            r#"<html><head></head><body>
<style>#test { position: static }</style>
<link rel="stylesheet" href="late.css">
<div id="test">Test</div>
<script>globalThis.__zwProbe = getComputedStyle(document.getElementById('test')).position;</script>
</body></html>"#
        ),
        "fixed"
    );
}

/// link 在文档 `<style>` **之前**：文档表后到后胜 → `static`（blob/条目两形态
/// 同判——防过度矫正守卫）。
#[test]
fn link_before_style_loses_by_document_order() {
    let mut webview = webview_with_css(HashMap::from([(
        "https://wpt.test/cascade-order/late.css".to_string(),
        "#test { position: fixed }".to_string(),
    )]));
    assert_eq!(
        load_and_probe(
            &mut webview,
            r#"<html><head>
<link rel="stylesheet" href="late.css">
</head><body>
<style>#test { position: static }</style>
<div id="test">Test</div>
<script>globalThis.__zwProbe = getComputedStyle(document.getElementById('test')).position;</script>
</body></html>"#
        ),
        "static"
    );
}

/// 文档序交错精钉：linkA → `<style>` → linkB（三者在 body 内交错）——期望序
/// A、style、B，B 最后 → `absolute`。红态（blob 前置）：A、B、style → style 胜
/// → `static`。
#[test]
fn interleaved_link_style_link_document_order() {
    let mut webview = webview_with_css(HashMap::from([
        (
            "https://wpt.test/cascade-order/a.css".to_string(),
            "#test { position: fixed }".to_string(),
        ),
        (
            "https://wpt.test/cascade-order/b.css".to_string(),
            "#test { position: absolute }".to_string(),
        ),
    ]));
    assert_eq!(
        load_and_probe(
            &mut webview,
            r#"<html><head></head><body>
<link rel="stylesheet" href="a.css">
<style>#test { position: static }</style>
<link rel="stylesheet" href="b.css">
<div id="test">Test</div>
<script>globalThis.__zwProbe = getComputedStyle(document.getElementById('test')).position;</script>
</body></html>"#
        ),
        "absolute"
    );
}

/// media 门在条目形态保持：media 不匹配的 link 不取回不出条目 → 文档 `<style>`
/// 独胜 → `static`。
#[test]
fn media_gated_link_stays_out_of_entries() {
    let mut webview = webview_with_css(HashMap::from([(
        "https://wpt.test/cascade-order/print.css".to_string(),
        "#test { position: fixed }".to_string(),
    )]));
    assert_eq!(
        load_and_probe(
            &mut webview,
            r#"<html><head></head><body>
<style>#test { position: static }</style>
<link rel="stylesheet" href="print.css" media="print">
<div id="test">Test</div>
<script>globalThis.__zwProbe = getComputedStyle(document.getElementById('test')).position;</script>
</body></html>"#
        ),
        "static"
    );
}

/// @import 子表随根 link 桶参与文档序：`<style>`(static) 在 link(import→base
/// fixed) **之前** → link 位次在后 → 子表规则后到后胜 → `fixed`。红态（blob
/// 前置）：外链整体先于文档表 → 文档 `<style>` 恒胜 → `static`。
#[test]
fn import_expansion_stays_with_root_link_bucket() {
    let mut webview = webview_with_css(HashMap::from([
        (
            "https://wpt.test/cascade-order/import.css".to_string(),
            "@import url(\"base.css\")".to_string(),
        ),
        (
            "https://wpt.test/cascade-order/base.css".to_string(),
            "#test { position: fixed }".to_string(),
        ),
    ]));
    assert_eq!(
        load_and_probe(
            &mut webview,
            r#"<html><head></head><body>
<style>#test { position: static }</style>
<link rel="stylesheet" href="import.css">
<div id="test">Test</div>
<script>globalThis.__zwProbe = getComputedStyle(document.getElementById('test')).position;</script>
</body></html>"#
        ),
        "fixed"
    );
}

/// s177：`external_css_entries` getter 镜像 load_html 实际发布形态——静态外链页
/// 发布后条目非空（多进程 renderer runtime 咽喉读取源，随 SetDomSnapshot 钉版
/// 下发 js worker）；锚选择器非空、表文本逐字保真。
#[test]
fn external_css_entries_getter_mirrors_published_entries_s177() {
    let mut webview = webview_with_css(HashMap::from([(
        "https://wpt.test/cascade-order/late.css".to_string(),
        "#test { position: fixed }".to_string(),
    )]));
    load_and_probe(
        &mut webview,
        r#"<html><head></head><body>
<style>#test { position: static }</style>
<link rel="stylesheet" href="late.css">
<div id="test">Test</div>
</body></html>"#,
    );
    let entries = webview.external_css_entries();
    assert_eq!(entries.len(), 1, "静态外链发布后条目表恰一条");
    assert!(!entries[0].0.is_empty(), "锚选择器非空");
    assert_eq!(entries[0].1, "#test { position: fixed }\n", "表文本逐字保真（含源文件尾换行）");
}

/// s177：getter 回落半面——css 参数（嵌入者 custom-css 通道）回落 blob 发布，
/// 条目表为空；无链接页发布后亦空（Some 空表臂同清）。
#[test]
fn external_css_entries_getter_empty_on_blob_fallback_s177() {
    let mut webview = webview_with_css(HashMap::new());
    webview.load_html(
        "<html><body><div id=\"test\">t</div></body></html>",
        Some("#test { color: red }"),
    );
    assert!(
        webview.external_css_entries().is_empty(),
        "css 参数回落 blob 发布，条目表为空"
    );
}
