//! k135：伪元素存储槽 background 判据扩面（css-shadow-parts §3.2.2，
//! https://drafts.csswg.org/css-shadow-1/#part）。
//!
//! 覆盖：bg-only `::part(x)::before/::after` 规则存槽（pseudo-elements-after-part
//! 判据页 `background` gCS 读形的计算面——k133 门只有 content/色差异两臂，bg-only
//! 不存槽＝gCS 第二参读恒空）、`::placeholder` bg-only 存槽（宿主门判据扩面）、
//! 误存守卫（ Transparent 值探针——伪计算回落 initial＝Transparent，元素自身
//! background 非 Transparent 时旧 elem-diff 判据会误存，R5052 UA 表单默认陷阱
//! 同型；k135 判据不比 elem_style）、k133 color-only 臂零漂移对照。

use super::super::*;
use super::helpers::*;
use zero_css_parser::Parser;
use zero_dom::ShadowRootMode;

/// 组装 bg-only 判据页：html > body > div#host（open shadow root；树内
/// `<span part="inner">`，树内规则给 span 自身元素背景——误存守卫的陷阱构造）。
fn make_bg_probe_page() -> (Document, NodeId, NodeId) {
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
    let span = doc.create_element("span");
    doc.set_attribute(span, "part", "inner");
    doc.append_child(shadow, span).unwrap();
    (doc, host, span)
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

/// bg-only `::part(x)::before` 存槽：`background` 声明（shorthand 归一 background-color）
/// 进 before_pseudo.background_color——k133 门（content/色差异）漏收的判据页读形。
#[test]
fn test_kc5r135_before_bg_only_slot() {
    let (doc, host, span) = make_bg_probe_page();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("::part(inner)::before { background: rgb(1, 2, 3); }"),
        &[],
    );
    let before = styles
        .get(&span)
        .expect("元素样式应在表内")
        .before_pseudo
        .as_ref()
        .expect("bg-only ::part(x)::before 应存 before_pseudo（k135 门扩面）");
    assert_eq!(
        before.background_color,
        ColorValue::Rgba(1, 2, 3, 255),
        "before 槽收 background 声明值"
    );
    let _ = host;
}

/// bg-only `::part(x)::after` 存槽（before 门镜像臂）。
#[test]
fn test_kc5r135_after_bg_only_slot() {
    let (doc, host, span) = make_bg_probe_page();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("::part(inner)::after { background: rgb(4, 5, 6); }"),
        &[],
    );
    let after = styles
        .get(&span)
        .expect("元素样式应在表内")
        .after_pseudo
        .as_ref()
        .expect("bg-only ::part(x)::after 应存 after_pseudo（k135 门扩面）");
    assert_eq!(
        after.background_color,
        ColorValue::Rgba(4, 5, 6, 255),
        "after 槽收 background 声明值"
    );
    let _ = host;
}

/// 误存守卫：span 自身元素背景非 Transparent（树内规则）＋页面只有 ::after bg
/// 规则——before 无规则伪计算回落 initial（Transparent）。误存形态（比
/// elem_style）会把 Transparent 伪背景当成「与元素相异」而存槽（R5052 UA 表单
/// 默认陷阱同型）；k135 Transparent 值探针不存。after 臂照常存槽作判据活性对照。
#[test]
fn test_kc5r135_bg_no_rule_no_misstore() {
    let (doc, host, span) = make_bg_probe_page();
    let mut sys = StyleSystem::new();
    // 树内规则给 span 元素自身背景（非 Transparent）。
    let scopes = vec![shadow_scope(&doc, host, "span { background: rgb(8, 7, 6); }")];
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("::part(inner)::after { background: rgb(1, 2, 3); }"),
        &scopes,
    );
    let style = styles.get(&span).expect("元素样式应在表内");
    assert!(
        style.before_pseudo.is_none(),
        "before 无规则时 Transparent 伪背景不得因元素自身背景相异而误存"
    );
    let after = style
        .after_pseudo
        .as_ref()
        .expect("::after bg 规则照常存槽（判据活性对照）");
    assert_eq!(after.background_color, ColorValue::Rgba(1, 2, 3, 255));
}

/// `::placeholder` bg-only 存槽：宿主门（input/textarea＋placeholder 属性）判据
/// 扩面——after-part 判据页 `<input type='text' placeholder>` 形。
#[test]
fn test_kc5r135_placeholder_bg_only_slot() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let input = doc.create_element("input");
    doc.set_attribute(input, "type", "text");
    doc.set_attribute(input, "placeholder", "enter text");
    doc.append_child(body, input).unwrap();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(&doc, &parse("input::placeholder { background: rgb(4, 5, 6); }"), &[]);
    let ph = styles
        .get(&input)
        .expect("元素样式应在表内")
        .placeholder_pseudo
        .as_ref()
        .expect("bg-only ::placeholder 应存 placeholder_pseudo（k135 门扩面）");
    assert_eq!(
        ph.background_color,
        ColorValue::Rgba(4, 5, 6, 255),
        "placeholder 槽收 background 声明值"
    );
}

/// k133 color-only 臂零漂移对照：`::part(x)::before { color }` 无 content 形
/// 照常存槽（k135 门扩面不回流既有臂）。
#[test]
fn test_kc5r135_color_only_undrifted() {
    let (doc, _host, span) = make_bg_probe_page();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("::part(inner)::before { color: rgb(33, 44, 55); }"),
        &[],
    );
    let before = styles
        .get(&span)
        .expect("元素样式应在表内")
        .before_pseudo
        .as_ref()
        .expect("color-only ::part(x)::before 照 k133 臂存槽");
    assert_eq!(before.color, ColorValue::Rgba(33, 44, 55, 255));
}
