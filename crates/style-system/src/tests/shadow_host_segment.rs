//! 45 号 shadow-dom-rendering M1：`:host` 族宿主段级联测试（CSS Scoping §3/§5）。
//!
//! 覆盖：宿主段收集（shadow sheets 的 `:host` 族规则只作用于 host 元素自身）、
//! 树上下文排序轴（normal 外层胜 / !important 内层胜）、`:host(args)` /
//! `:host-context(args)` 匹配、文档表规则不经宿主段通道、非 shadow 页零变化。

use super::super::*;
use super::helpers::*;
use zero_css_parser::Parser;
use zero_dom::ShadowRootMode;

/// 组装 host 页：html > body > div#host（挂 open shadow root，树内 `<div class="inner">`）。
fn make_host_page() -> (Document, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let host = doc.create_element("div");
    doc.set_attribute(host, "id", "host");
    doc.append_child(body, host).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let inner = doc.create_element("div");
    doc.set_attribute(inner, "class", "inner");
    doc.append_child(shadow, inner).unwrap();
    (doc, host, inner)
}

fn parse(css: &str) -> Vec<zero_css_parser::Stylesheet> {
    vec![Parser::parse_stylesheet(css)]
}

fn shadow_scope(doc: &Document, host: NodeId, css: &str) -> ShadowScopeStylesheets {
    ShadowScopeStylesheets {
        root: doc.shadow_root(host).unwrap(),
        sheets: parse(css),
    }
}

fn color_of(styles: &HashMap<NodeId, ComputedStyle>, id: NodeId) -> ColorValue {
    styles.get(&id).expect("元素样式应在表内").color.clone()
}

/// CSS Scoping §5（https://drafts.csswg.org/css-scoping/#cascading）：normal 声明外层
/// 树胜——文档表 `div { color: blue }` 胜 shadow 表 `:host { color: red }`。
#[test]
fn test_host_segment_normal_outer_wins() {
    let (doc, host, _inner) = make_host_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, ":host { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse("div { color: blue; }"), &scopes);
    assert_eq!(
        color_of(&styles, host),
        ColorValue::Rgba(0, 0, 255, 255),
        "normal 声明外层树（文档表）应胜 shadow 宿主段"
    );
}

/// CSS Scoping §5：!important 内层树胜——shadow 表 `:host { color: red !important }`
/// 胜文档表 normal。
#[test]
fn test_host_segment_important_inner_wins() {
    let (doc, host, _inner) = make_host_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, ":host { color: red !important; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse("div { color: blue; }"), &scopes);
    assert_eq!(
        color_of(&styles, host),
        ColorValue::Rgba(255, 0, 0, 255),
        "!important 内层树（shadow 宿主段）应胜外层 normal"
    );
}

/// 宿主段内特异性参与比较：`:host(#host)` (1,1,0) 胜 `:host` (0,1,0)。
#[test]
fn test_host_segment_specificity_within_segment() {
    let (doc, host, _inner) = make_host_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host { color: red; } :host(#host) { color: blue; }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, host),
        ColorValue::Rgba(0, 0, 255, 255),
        "段内高特异性 :host(#host) 应胜 :host"
    );
}

/// 文档表规则不经宿主段通道：文档表的 `:host` 规则不生效（matcher 常规路径恒不匹配），
/// 同形态的 shadow 表 `:host` 才作用（封装边界）。
#[test]
fn test_document_sheet_host_rule_inert() {
    let (doc, host, _inner) = make_host_page();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(&doc, &parse(":host { color: red; }"), &[]);
    assert_ne!(
        color_of(&styles, host),
        ColorValue::Rgba(255, 0, 0, 255),
        "文档表 :host 规则不得命中宿主（封装边界）"
    );
}

/// `:host(<compound-selector>)` 参数匹配：类命中生效、类不命中不生效。
#[test]
fn test_host_arg_match() {
    let (mut doc, host, _inner) = make_host_page();
    doc.set_attribute(host, "class", "card");
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host(.card) { color: red; } :host(.other) { color: green; }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, host),
        ColorValue::Rgba(255, 0, 0, 255),
        ":host(.card) 应命中 class=card 宿主，:host(.other) 不命中"
    );
}

/// `:host-context(<compound-selector>)`：light 域祖先命中（穿出 shadow 边界上行）。
#[test]
fn test_host_context_ancestor_match() {
    let (doc, host, _inner) = make_host_page();
    let mut sys = StyleSystem::new();
    // body 是 host 的 shadow-including 祖先。
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host-context(body) { color: red; } :host-context(.nope) { color: green; }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, host),
        ColorValue::Rgba(255, 0, 0, 255),
        ":host-context(body) 应经祖先链命中"
    );
}

/// 非 host 元素不吃宿主段：shadow 树内元素不受 `:host` 规则影响（吃树内普通规则）。
#[test]
fn test_shadow_inner_not_affected_by_host_segment() {
    let (doc, host, inner) = make_host_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host { color: red; } .inner { color: blue; }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, inner),
        ColorValue::Rgba(0, 0, 255, 255),
        "shadow 树内元素应吃树内普通规则而非宿主段"
    );
}

/// 空 scopes（非 shadow 页）：[`StyleSystem::compute_styles`] 与
/// [`StyleSystem::compute_styles_with_shadow`] 逐位一致——tree_context 轴为常量
///（同值比较落入后轴），常规页排序零变化（回归门）。
#[test]
fn test_plain_path_unchanged() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let d = doc.create_element("div");
    doc.append_child(body, d).unwrap();
    let mut sys = StyleSystem::new();
    let styles_a = sys.compute_styles(&doc, &parse("div { color: blue; }"));
    let styles_b = sys.compute_styles_with_shadow(&doc, &parse("div { color: blue; }"), &[]);
    assert_eq!(
        color_of(&styles_a, d),
        color_of(&styles_b, d),
        "空 scopes 两路径计算结果应逐位一致"
    );
}
