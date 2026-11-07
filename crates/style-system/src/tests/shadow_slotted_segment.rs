//! 45 号 shadow-dom-rendering M2：`::slotted(<compound-selector>)` 段级联测试
//!（CSS Shadow §3.2.4/§3.3）。
//!
//! 覆盖：命中翻转（::slotted 规则作用于被分配 light 子）、参数化合物过滤（含
//! `:has` 入参）、树上下文排序轴（normal 外层胜 / !important 内层胜）、段内
//! specificity、`:host` / 树内祖先前缀组合子（flat tree 上行）、slot 余部化合物、
//! R5 继承切父（slotted 元素从 slot 继承样式与自定义属性）、`::slotted(x)::before`
//! 与 `:host::before` 伪元素路径、未分配元素零命中。

use super::super::*;
use super::helpers::*;
use zero_css_parser::Parser;
use zero_dom::ShadowRootMode;

/// 组装基础 slotted 页：
/// html > body > div#host（open shadow root；树内 slot#main 默认槽 +
/// 回退内容 `<p class="fb">`；host light 子 `<span class="s">`）。
/// 返回 (doc, host, default_slot, span)。已调 resolve_slots 完成分配。
fn make_slotted_page() -> (Document, NodeId, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let host = doc.create_element("div");
    doc.set_attribute(host, "id", "host");
    doc.append_child(body, host).unwrap();
    let span = doc.create_element("span");
    doc.set_attribute(span, "class", "s");
    doc.append_child(host, span).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let slot = doc.create_element("slot");
    doc.set_attribute(slot, "id", "main");
    doc.append_child(shadow, slot).unwrap();
    let fb = doc.create_element("p");
    doc.set_attribute(fb, "class", "fb");
    doc.append_child(slot, fb).unwrap();
    doc.resolve_slots(host);
    (doc, host, slot, span)
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

/// CSS Shadow §3.2.4 命中翻转：shadow 表 `::slotted(*)` 命中被分配 light 子；
/// 回退内容（slot 树内子）与宿主不吃该规则。
#[test]
fn test_slotted_universal_hit() {
    let (doc, host, _slot, span) = make_slotted_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, "::slotted(*) { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        "::slotted(*) 应命中被分配的 light 子"
    );
    assert_ne!(
        color_of(&styles, host),
        ColorValue::Rgba(255, 0, 0, 255),
        "宿主不吃 ::slotted 规则"
    );
}

/// 参数化合物过滤：`::slotted(span)` 命中 span、`::slotted(.no)` 不命中。
#[test]
fn test_slotted_arg_filter() {
    let (doc, host, _slot, span) = make_slotted_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, "::slotted(span) { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        "参数 span 命中被分配元素"
    );

    let scopes = vec![shadow_scope(&doc, host, "::slotted(.no) { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_ne!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        "参数不命中时 ::slotted 规则不得应用"
    );
}

/// 参数内 `:has()` 合法且须匹配（slotted-has-001..004 形态，csswg-drafts#10586）。
#[test]
fn test_slotted_has_argument() {
    let (mut doc, host, _slot, span) = make_slotted_page();
    // span 无子 → :has(b) 不命中。
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, "::slotted(:has(b)) { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_ne!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        ":has(b) 对无 b 后代的 span 不命中"
    );
    // 加 <b> 子后命中。
    let b = doc.create_element("b");
    doc.append_child(span, b).unwrap();
    doc.resolve_slots(doc.parent_node(span).unwrap());
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        ":has(b) 对含 b 后代的 span 命中"
    );
}

/// CSS Shadow §3.2.4 树上下文排序（css-cascade-5 §6.1）：normal 声明外层树胜——
/// 文档表 `span { color: blue }` 胜 shadow 表 `::slotted(*) { color: red }`。
#[test]
fn test_slotted_normal_outer_wins() {
    let (doc, host, _slot, span) = make_slotted_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, "::slotted(*) { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse("span { color: blue; }"), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(0, 0, 255, 255),
        "normal 声明外层树（文档表）应胜 shadow slotted 段"
    );
}

/// 树上下文排序：!important 内层树胜——shadow 表 `::slotted(*) { color: red
/// !important }` 胜文档表 normal。
#[test]
fn test_slotted_important_inner_wins() {
    let (doc, host, _slot, span) = make_slotted_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, "::slotted(*) { color: red !important; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse("span { color: blue; }"), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        "!important 内层树（shadow slotted 段）应胜外层 normal"
    );
}

/// 段内 specificity：`::slotted(.s)` (0,1,1) 胜 `::slotted(*)` (0,0,1)
///（slotted-specificity.html driving 形态）。
#[test]
fn test_slotted_specificity_within_segment() {
    let (doc, host, _slot, span) = make_slotted_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        "::slotted(*) { color: green; } ::slotted(.s) { color: red; }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        "段内高特异性 ::slotted(.s) 应胜 ::slotted(*)"
    );
}

/// `:host` 前缀组合子：`:host ::slotted(*)` 命中（前缀沿 slot 的 flat tree 上行，
/// host 边界 featureless 仅 :host 族命中）；参数不匹配的 `:host(.nope)` 前缀不命中。
#[test]
fn test_host_prefix_slotted() {
    let (doc, host, _slot, span) = make_slotted_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, ":host ::slotted(*) { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        ":host ::slotted(*) 应经 flat tree 前缀链命中"
    );

    let scopes = vec![shadow_scope(&doc, host, ":host(.nope) ::slotted(*) { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_ne!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        ":host(.nope) 前缀不命中宿主时不应用"
    );
}

/// 树内祖先前缀组合子：`div.wrap ::slotted(*)` 仅当 slot 包裹在 shadow 树内
/// div.wrap 时命中（前缀链 slot → wrap → …）。
#[test]
fn test_wrapped_slot_prefix() {
    // 包裹形态：shadow 树 div.wrap > slot。
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let host = doc.create_element("div");
    doc.set_attribute(host, "id", "host");
    doc.append_child(body, host).unwrap();
    let span = doc.create_element("span");
    doc.append_child(host, span).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let wrap = doc.create_element("div");
    doc.set_attribute(wrap, "class", "wrap");
    doc.append_child(shadow, wrap).unwrap();
    let slot = doc.create_element("slot");
    doc.append_child(wrap, slot).unwrap();
    doc.resolve_slots(host);
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, "div.wrap ::slotted(*) { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        "div.wrap ::slotted(*) 应经树内前缀链命中"
    );

    // 无包裹形态（make_slotted_page：slot 直挂 shadow root）不命中。
    let (doc, host, _slot, span) = make_slotted_page();
    let scopes = vec![shadow_scope(&doc, host, "div.wrap ::slotted(*) { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_ne!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        "slot 无 div.wrap 祖先时不应用"
    );
}

/// slot 余部化合物：`slot.named::slotted(em)` 仅命中分配到 name=named 槽的元素。
#[test]
fn test_slot_rest_compound() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let host = doc.create_element("div");
    doc.append_child(body, host).unwrap();
    let em = doc.create_element("em");
    doc.set_attribute(em, "slot", "named");
    doc.append_child(host, em).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let named = doc.create_element("slot");
    doc.set_attribute(named, "name", "named");
    // `slot.named` 为类选择器（余部化合物走类匹配），槽位须带同名 class。
    doc.set_attribute(named, "class", "named");
    doc.append_child(shadow, named).unwrap();
    let other = doc.create_element("slot");
    doc.set_attribute(other, "name", "other");
    doc.set_attribute(other, "class", "other");
    doc.append_child(shadow, other).unwrap();
    doc.resolve_slots(host);
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        "slot.named::slotted(em) { color: red; } slot.other::slotted(em) { color: green; }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, em),
        ColorValue::Rgba(255, 0, 0, 255),
        "余部 slot.named 匹配分配槽位时应命中"
    );
    assert_ne!(
        color_of(&styles, em),
        ColorValue::Rgba(0, 128, 0, 255),
        "余部 slot.other 不匹配分配槽位时不得命中"
    );
}

/// R5 继承切父（CSS Shadow §3.3）：slotted 元素从 originating slot 继承——
/// shadow 表 `slot { color: green }` 使无自身规则的 slotted 子呈绿色（宿主
/// 颜色黑不得泄漏）。
#[test]
fn test_slotted_inherits_from_slot() {
    let (doc, host, _slot, span) = make_slotted_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, "slot { color: green; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(0, 128, 0, 255),
        "slotted 子应从 slot 继承 color 而非宿主"
    );
}

/// R5 继承切父（自定义属性）：`slot { --accent: ... }` + `::slotted(*) { color:
/// var(--accent) }`——var 求值取 slot 的自定义属性继承链。
#[test]
fn test_slotted_inherits_slot_custom_property() {
    let (doc, host, _slot, span) = make_slotted_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        "slot { --accent: rgb(0, 128, 0); } ::slotted(*) { color: var(--accent); }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(0, 128, 0, 255),
        "slotted 子 var(--accent) 应取 slot 链的自定义属性"
    );
}

/// 伪元素路径：`::slotted(*)::before` 生成被分配元素的 ::before 伪样式。
#[test]
fn test_slotted_before_pseudo() {
    let (doc, host, _slot, span) = make_slotted_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        "::slotted(*)::before { content: 'X'; color: red; }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    let before = styles
        .get(&span)
        .and_then(|s| s.before_pseudo.as_ref())
        .expect("::slotted(*)::before 应生成伪样式");
    assert_eq!(
        before.color,
        ColorValue::Rgba(255, 0, 0, 255),
        "伪样式 color 应取 slotted 段声明"
    );
}

/// 伪元素路径（宿主段）：`:host::before` 生成宿主 ::before 伪样式（M2 宿主段
/// 伪元素尾接线）。
#[test]
fn test_host_before_pseudo() {
    let (doc, host, _slot, _span) = make_slotted_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, ":host::before { content: 'H'; color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    let before = styles
        .get(&host)
        .and_then(|s| s.before_pseudo.as_ref())
        .expect(":host::before 应生成宿主伪样式");
    assert_eq!(
        before.color,
        ColorValue::Rgba(255, 0, 0, 255),
        "宿主伪样式 color 应取宿主段声明"
    );
}

/// 未分配元素零命中：light 子未被任何 slot 分配（shadow 树只有 named 槽）时，
/// `::slotted(*)` 不得命中。
#[test]
fn test_unassigned_light_child_not_hit() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let host = doc.create_element("div");
    doc.append_child(body, host).unwrap();
    let span = doc.create_element("span");
    doc.append_child(host, span).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let named = doc.create_element("slot");
    doc.set_attribute(named, "name", "named");
    doc.append_child(shadow, named).unwrap();
    doc.resolve_slots(host);
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, "::slotted(*) { color: red; }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_ne!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        "未分配元素不得命中 ::slotted 规则"
    );
}
