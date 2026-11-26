//! k136：异种伪元素槽表（css-shadow-1 §5.4 缀后伪元素 × css-pseudo-4 异种伪
//! 元素 gCS 读面）＋ first-line/first-letter part 通道 plumbing。
//!
//! 覆盖：静态名裸键 bg 存槽（`::backdrop`）、highlight 函数形键控（多实名各占
//! 键；实参归一同键——解析器 Functional 臂 `Token::Ident` 解码+小写）、
//! Transparent 误存守卫（元素自身 bg 相异不误存无差异键，R5052 同型）、
//! first-letter bg 位（part 通道，R5320 first-line 切片 2 镜像）、first-line
//! part 通道（interaction-with-pseudo-elements 页形）、非 part 元素探针 None
//! 臂（外树 normal 胜机制依赖槽只落 part 元素）。

use super::super::*;
use super::helpers::*;
use zero_css_parser::Parser;
use zero_dom::ShadowRootMode;

fn parse(css: &str) -> Vec<zero_css_parser::Stylesheet> {
    vec![Parser::parse_stylesheet(css)]
}

fn shadow_scope(doc: &Document, host: NodeId, css: &str) -> ShadowScopeStylesheets {
    ShadowScopeStylesheets {
        root: doc.shadow_root(host).unwrap(),
        sheets: parse(css),
    }
}

/// 组装页：html > body > div#host（open shadow root；树内 `<span part="inner">`）。
fn make_part_page() -> (Document, NodeId, NodeId) {
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

/// 组装普通文档页：html > body > div（无 shadow；异种裸键/键控页形）。
fn make_plain_page() -> (Document, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    (doc, div)
}

/// 静态名裸键 bg 存槽：`::backdrop` 规则 → other_pseudos 精确键 "backdrop"
/// 收 background 声明值。
#[test]
fn test_kc5r136_backdrop_bg_slot() {
    let (doc, div) = make_plain_page();
    let mut sys = StyleSystem::new();
    let styles =
        sys.compute_styles_with_shadow(&doc, &parse("::backdrop { background: rgb(1, 2, 3); }"), &[]);
    let slots = styles
        .get(&div)
        .expect("元素样式应在表内")
        .other_pseudos
        .as_ref()
        .expect("::backdrop bg 规则应存异种槽（k136 门）");
    let (key, st) = slots
        .iter()
        .find(|(k, _)| k.as_ref() == "backdrop")
        .expect("槽键应为精确裸名 backdrop");
    assert_eq!(key.as_ref(), "backdrop");
    assert_eq!(st.background_color, ColorValue::Rgba(1, 2, 3, 255));
}

/// highlight 函数形键控：两实名规则各占独立键、值互不串（exact-key 检索面）。
#[test]
fn test_kc5r136_highlight_slots_keyed() {
    let (doc, div) = make_plain_page();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse(
            "::highlight(foo) { color: rgb(1, 2, 3); } ::highlight(bar) { background: rgb(4, 5, 6); }",
        ),
        &[],
    );
    let slots = styles
        .get(&div)
        .expect("元素样式应在表内")
        .other_pseudos
        .as_ref()
        .expect("highlight 规则应存异种槽");
    assert_eq!(slots.len(), 2, "两实名各占一键");
    let foo = slots.iter().find(|(k, _)| k.as_ref() == "highlight(foo)").expect("foo 键在表");
    assert_eq!(foo.1.color, ColorValue::Rgba(1, 2, 3, 255));
    let bar = slots.iter().find(|(k, _)| k.as_ref() == "highlight(bar)").expect("bar 键在表");
    assert_eq!(bar.1.background_color, ColorValue::Rgba(4, 5, 6, 255));
}

/// highlight 实参归一同键：规则侧 `n\61me`（CSS escape 解码 ident "name"）＋
/// 大写 `NAME`（ascii_lowercase）→ 键同为 "highlight(name)"（读形归一与键同源，
/// getComputedStyle-pseudo-with-argument 别名面）。
#[test]
fn test_kc5r136_highlight_arg_normalized_key() {
    let (doc, div) = make_plain_page();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(
        &doc,
        // Rust 串 \\61 → CSS 文本 `\61 ` 形。
        &parse("::highlight(n\\61me) { color: rgb(9, 8, 7); }"),
        &[],
    );
    let slots = styles
        .get(&div)
        .expect("元素样式应在表内")
        .other_pseudos
        .as_ref()
        .expect("highlight 解码形规则应存槽");
    assert!(
        slots.iter().any(|(k, _)| k.as_ref() == "highlight(name)"),
        "escape 解码形实参应归一为 highlight(name)，实际键表 {:?}",
        slots.iter().map(|(k, _)| k.as_ref()).collect::<Vec<_>>()
    );
    let st = slots.iter().find(|(k, _)| k.as_ref() == "highlight(name)").unwrap();
    assert_eq!(st.1.color, ColorValue::Rgba(9, 8, 7, 255));
    assert_eq!(slots.len(), 1, "归一后不得残留原文键 highlight(n\\61me)");
}

/// Transparent 误存守卫：div 自身规则 bg 非 Transparent（R5052 UA 表单默认陷阱
/// 同型）＋ `::backdrop` 仅 background: transparent——伪计算回落 Transparent，
/// 旧 elem-diff 判据（bg red ≠ Transparent）会误存无差异键；k136 门（color 相异
/// 或 bg 非 Transparent）不存。gCS 读回落元素值同效。
#[test]
fn test_kc5r136_no_diff_no_misstore() {
    let (doc, div) = make_plain_page();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("div { background: rgb(9, 9, 9); } ::backdrop { background: transparent; }"),
        &[],
    );
    let style = styles.get(&div).expect("元素样式应在表内");
    assert!(
        style.other_pseudos.is_none(),
        "Transparent 伪背景不得因元素自身 bg 相异而误存（R5052 同型守卫）"
    );
    assert_eq!(style.background_color, ColorValue::Rgba(9, 9, 9, 255), "元素本体 bg 不受扰");
}

/// first-letter bg 位（part 通道，R5320 first-line 切片 2 镜像）：
/// `::part(inner)::first-letter { background }` → first_letter_pseudo 槽收 bg
///（pseudo-elements-after-part gCS 读形，k136 前 color-only 门不存）。
#[test]
fn test_kc5r136_first_letter_part_bg_arm() {
    let (doc, _host, span) = make_part_page();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("::part(inner)::first-letter { background: rgb(7, 8, 9); }"),
        &[],
    );
    let fl = styles
        .get(&span)
        .expect("元素样式应在表内")
        .first_letter_pseudo
        .as_ref()
        .expect("bg-only ::part(x)::first-letter 应存 first_letter_pseudo（k136 bg 位）");
    assert_eq!(fl.background_color, ColorValue::Rgba(7, 8, 9, 255));
}

/// first-line part 通道 plumbing：`::part(inner)::first-line { color }` →
/// first_line_pseudo 槽（interaction-with-pseudo-elements 页形；k136 shadow_view
/// 透传前 part 缀后形不可达）。
#[test]
fn test_kc5r136_first_line_part_channel() {
    let (doc, _host, span) = make_part_page();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("::part(inner)::first-line { color: rgb(2, 3, 4); }"),
        &[],
    );
    let fline = styles
        .get(&span)
        .expect("元素样式应在表内")
        .first_line_pseudo
        .as_ref()
        .expect("::part(x)::first-line 应经 part 通道存 first_line_pseudo");
    assert_eq!(fline.color, ColorValue::Rgba(2, 3, 4, 255));
}

/// 非 part 元素探针 None 臂：span 无 part 属性＋页面仅 `::part(x)::first-line`
/// 规则——part body 不匹配（或探针早退）→ 不存 first_line_pseudo（外树 normal
/// 胜机制依赖槽只落 part 元素）。
#[test]
fn test_kc5r136_first_line_no_part_attr_none() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let host = doc.create_element("div");
    doc.append_child(body, host).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let span = doc.create_element("span"); // 无 part 属性
    doc.append_child(shadow, span).unwrap();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("::part(other)::first-line { color: rgb(2, 3, 4); }"),
        &[],
    );
    let style = styles.get(&span).expect("元素样式应在表内");
    assert!(
        style.first_line_pseudo.is_none(),
        "非 part 元素不得存 ::part 缀后 first-line 槽"
    );
    assert!(style.other_pseudos.is_none(), "::part 键不入异种槽表（名单外尾）");
}

/// shadow 表规则同样入键表（页扫描 shadow 并集）：树内 `::target-text { color }`
/// 规则经 shadow scope 收键、树内元素存槽（8 suite 页两表混合形）。
#[test]
fn test_kc5r136_shadow_scope_rule_collected() {
    let (doc, host, span) = make_part_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, "::target-text { color: rgb(5, 6, 7); }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    let slots = styles
        .get(&span)
        .expect("元素样式应在表内")
        .other_pseudos
        .as_ref()
        .expect("shadow 表 ::target-text 规则应入键表并存槽");
    let st = slots
        .iter()
        .find(|(k, _)| k.as_ref() == "target-text")
        .expect("槽键应为 target-text");
    assert_eq!(st.1.color, ColorValue::Rgba(5, 6, 7, 255));
}
