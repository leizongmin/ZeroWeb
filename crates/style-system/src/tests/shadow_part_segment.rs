//! k133：`::part(<ident-list>)` 段级联测试（CSS Shadow §5.4，
//! https://drafts.csswg.org/css-shadow-1/#part）。
//!
//! 覆盖：命中翻转（文档表裸形 `::part(x)` 命中；shadow 树内裸形不命中——§5.4
//! originating element 须为 shadow host）、多名交集命中（§5.4 multiple idents =
//! 逐名全中）、ASCII 大小写不敏感比对、名单不命中零命中、缀后伪类（`checked`
//! 常规匹配生效）、`::part(x)::before` 伪元素尾对齐（`before_pseudo` 存储）、
//! host light 子零命中、文档表规则 host 复合/复杂前缀跨树命中、树上下文轴
//!（css-cascade-5 §6.1：normal 外树胜不问 specificity、important 内树反转胜）、
//! part 段 custom property 声明暂缓过滤（FIXME(k134+)）。
//! 匹配判据＝`part_selector_matches` 拆解＋`collect_part_declarations_with_media`
//! 段收集（shadow/文档表双通道）；解析面归 css-parser kc5r133_part_parse 钉。

use super::super::*;
use super::helpers::*;
use zero_css_parser::Parser;
use zero_dom::ShadowRootMode;

/// 组装基础 part 页：html > body > div#host（open shadow root；树内
/// `<span part="inner both">` ＋ `<input type="checkbox" part="cb" checked>`）。
/// 返回 (doc, host, span, cb)。shadow 树内无 slot（part 面不依赖分配）。
fn make_part_page() -> (Document, NodeId, NodeId, NodeId) {
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
    doc.set_attribute(span, "part", "inner both");
    doc.append_child(shadow, span).unwrap();
    let cb = doc.create_element("input");
    doc.set_attribute(cb, "type", "checkbox");
    doc.set_attribute(cb, "part", "cb");
    doc.set_attribute(cb, "checked", "");
    doc.append_child(shadow, cb).unwrap();
    (doc, host, span, cb)
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

/// 命中翻转：文档表裸形 `::part(inner)` 命中 `part="inner"` 元素（all-hosts 判据
/// 页形）；shadow 树内裸形不命中（§5.4 originating element 是 part 元素自身、非
/// shadow host，host-stylesheet 判据页形——同树 `span` 规则独胜）；宿主（无 part
/// 属性）与名单外形态零命中。
#[test]
fn test_part_hit_flip() {
    let (doc, host, span, _cb) = make_part_page();
    let mut sys = StyleSystem::new();
    // 文档表裸形命中。
    let styles = sys.compute_styles_with_shadow(&doc, &parse("::part(inner) { color: rgb(1, 2, 3); }"), &[]);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(1, 2, 3, 255),
        "文档表裸形 ::part(inner) 应命中 part=\"inner\" 元素"
    );
    assert_ne!(
        color_of(&styles, host),
        ColorValue::Rgba(1, 2, 3, 255),
        "宿主不吃 ::part 规则"
    );

    // shadow 树内裸形不命中：同树 span 规则独胜（host-stylesheet 判据页全形）。
    let scopes = vec![shadow_scope(
        &doc,
        host,
        "::part(inner) { color: rgb(255, 0, 0); } span { color: rgb(0, 128, 0); }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(0, 128, 0, 255),
        "树内裸形 originating element 非 host 不命中——span 规则独胜"
    );

    // 名单不命中：`::part(other)` 与 span 的 part 属性无交集。
    let styles = sys.compute_styles_with_shadow(&doc, &parse("#host::part(other) { color: rgb(1, 2, 3); }"), &[]);
    assert_ne!(color_of(&styles, span), ColorValue::Rgba(1, 2, 3, 255));
}

/// 多名交集命中（css-shadow §5.4：multiple idents = part element map 逐名交集，
/// 元素须具备名单全部名）＋ ASCII 大小写不敏感。
#[test]
fn test_part_name_list_and_case() {
    let (doc, host, span, _cb) = make_part_page();
    let mut sys = StyleSystem::new();
    // 名单全中：{inner, both} ⊆ span 的 part token 集。
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host::part(inner both) { color: rgb(4, 5, 6); }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(color_of(&styles, span), ColorValue::Rgba(4, 5, 6, 255), "名单全中命中");
    // 交集语义：名单含 span 缺失的名（missing）→ 不命中。
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host::part(both missing) { color: rgb(4, 5, 6); }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_ne!(
        color_of(&styles, span),
        ColorValue::Rgba(4, 5, 6, 255),
        "名单含缺失名不命中（交集语义）"
    );
    // 大小写：::part(INNER) 归一小写后命中 part="inner"。
    let scopes = vec![shadow_scope(&doc, host, ":host::part(INNER) { color: rgb(7, 8, 9); }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(7, 8, 9, 255),
        "part 名比对 ASCII 大小写不敏感"
    );
}

/// 缀后伪类 `::part(cb):checked` 常规匹配生效（grouping-with-checked 判据页形）；
/// 名单＋伪类不匹配形态（元素非 checked）零命中。
#[test]
fn test_part_with_checked_pseudo_class() {
    let (doc, host, _span, cb) = make_part_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host::part(cb):checked { color: rgb(10, 20, 30); }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, cb),
        ColorValue::Rgba(10, 20, 30, 255),
        "::part(cb):checked 应命中 checked 元素"
    );
    // 伪类不匹配：`:hover` 交互态无模拟 → 不命中（parser 放行、匹配面裁决）。
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host::part(cb):hover { color: rgb(10, 20, 30); }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_ne!(color_of(&styles, cb), ColorValue::Rgba(10, 20, 30, 255));
}

/// `::part(x)::before` 伪元素尾对齐：主元素样式零命中（规则带尾），伪元素样式
/// 存 `before_pseudo`（pseudo-elements-after-part 判据页 gCS 形的计算面）。
#[test]
fn test_part_pseudo_element_tail() {
    let (doc, host, span, _cb) = make_part_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host::part(inner) { color: rgb(0, 0, 0); } :host::part(inner)::before { color: rgb(33, 44, 55); }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(0, 0, 0, 255),
        "::part(inner) 裸规则命中元素自身"
    );
    let style = styles.get(&span).expect("元素样式应在表内");
    let before = style
        .before_pseudo
        .as_ref()
        .expect("::part(x)::before 应算入 before_pseudo");
    assert_eq!(
        before.color,
        ColorValue::Rgba(33, 44, 55, 255),
        "before 伪元素样式取 ::part(x)::before 规则值"
    );

    // 文档表专属形：仅文档表 `::part(x)::before`（shadow 表无任何匹配规则）——
    // 伪元素通道收集只含尾对齐声明，主匹配段恒空，早返门不得吞掉 doc 段。
    let styles = sys.compute_styles_with_shadow(&doc, &parse("::part(inner)::before { color: rgb(66, 77, 88); }"), &[]);
    let style = styles.get(&span).expect("元素样式应在表内");
    let before = style
        .before_pseudo
        .as_ref()
        .expect("文档表专属 ::part(x)::before 应算入 before_pseudo");
    assert_eq!(
        before.color,
        ColorValue::Rgba(66, 77, 88, 255),
        "文档表专属 before 尾声明应进入伪元素样式"
    );
}

/// host light 子带 part 属性零命中（css-shadow-parts §2：文档侧 part 属性不吃
/// shadow 表规则——part 段收集只服务 shadow 树内元素）。
#[test]
fn test_part_light_child_miss() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let host = doc.create_element("div");
    doc.set_attribute(host, "id", "host");
    doc.append_child(body, host).unwrap();
    let light = doc.create_element("span");
    doc.set_attribute(light, "part", "inner");
    doc.append_child(host, light).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let slot = doc.create_element("slot");
    doc.append_child(shadow, slot).unwrap();
    doc.resolve_slots(host);

    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(&doc, host, "::part(inner) { color: rgb(1, 2, 3); }")];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_ne!(
        color_of(&styles, light),
        ColorValue::Rgba(1, 2, 3, 255),
        "light 子（即使 part 属性同名）不吃 shadow 表 ::part 规则"
    );
}

/// 文档表 `::part` 规则（css-shadow §5.4 判据主力）：host 复合形 `#host::part(inner)`
/// 与复杂前缀形 `body #host::part(inner)` 剥臂对 host 求值后命中 shadow 树内 part
/// 元素（all-hosts/complex-matching 判据页形）；host 不匹配形态零命中。
#[test]
fn test_part_document_table_host_compound() {
    let (doc, host, span, _cb) = make_part_page();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles_with_shadow(&doc, &parse("#host::part(inner) { color: rgb(8, 9, 10); }"), &[]);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(8, 9, 10, 255),
        "文档表 host 复合 ::part 规则应命中 shadow 树内 part 元素"
    );
    let styles =
        sys.compute_styles_with_shadow(&doc, &parse("body #host::part(inner) { color: rgb(11, 12, 13); }"), &[]);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(11, 12, 13, 255),
        "文档表复杂前缀 ::part 规则应命中"
    );
    let styles = sys.compute_styles_with_shadow(&doc, &parse("#other::part(inner) { color: rgb(8, 9, 10); }"), &[]);
    assert_ne!(
        color_of(&styles, span),
        ColorValue::Rgba(8, 9, 10, 255),
        "host 不匹配（id 错）零命中"
    );
}

/// `:host::part(x)` 树内复合形（host-part-001 判据页形）：`:host` 臂剥臂后对 host
/// 求值命中（part 元素上行找 ShadowRoot host）。
#[test]
fn test_part_shadow_inner_host_compound() {
    let (doc, host, span, _cb) = make_part_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host::part(inner) { color: rgb(14, 15, 16); }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(14, 15, 16, 255),
        "树内 :host::part 复合形应命中"
    );
}

/// part 段 custom property 声明暂缓收集（FIXME(k134+)：@container unknown 语义域
/// 未建——function-shadow-container 判据页 `::part(t){--actual:--b()}` 形回归
/// 回退）；常规属性同规则不受过滤影响。
#[test]
fn test_part_custom_property_decls_filtered() {
    let (doc, host, span, _cb) = make_part_page();
    let mut sys = StyleSystem::new();
    let scopes = vec![shadow_scope(
        &doc,
        host,
        ":host::part(inner) { --actual: 10px; color: rgb(3, 4, 5); }",
    )];
    let styles = sys.compute_styles_with_shadow(&doc, &parse(""), &scopes);
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(3, 4, 5, 255),
        "同规则常规属性照常收集"
    );
    let style = styles.get(&span).expect("元素样式应在表内");
    assert!(
        style.custom_properties.iter().all(|(k, _)| k.as_str() != "--actual"),
        "--* 声明应被过滤（FIXME(k134+) 暂缓）"
    );
}

/// 树上下文轴（css-cascade-5 §6.1，https://drafts.csswg.org/css-cascade-5/#shadow-tree）：
/// normal 声明外树胜内树、important 反转内树胜外树，specificity 不越轴。
/// simple.html（normal 外树胜）/simple-important-important（important 内树反转）
/// 判据页形。
#[test]
fn test_part_tree_context_cascade_axes() {
    let (doc, host, span, _cb) = make_part_page();
    let mut sys = StyleSystem::new();
    // normal：文档表 `#host::part(inner)` green 胜 shadow 树内 `span` red（外树胜）。
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("#host::part(inner) { color: rgb(0, 128, 0); }"),
        &[shadow_scope(&doc, host, "span { color: rgb(255, 0, 0); }")],
    );
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(0, 128, 0, 255),
        "normal 轴外树 part 声明胜内树"
    );
    // normal 轴不问 specificity：文档表裸形（低 specificity）胜 shadow 内高
    // specificity `span[part]`。
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("::part(inner) { color: rgb(0, 128, 0); }"),
        &[shadow_scope(&doc, host, "span[part] { color: rgb(255, 0, 0); }")],
    );
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(0, 128, 0, 255),
        "normal 外树轴先行于 specificity 轴"
    );
    // important 反转：文档表 `#host::part(inner)` green!important 负于 shadow 树内
    // `span` red!important（important 轴内树胜，simple-important-important 判据页形）。
    let styles = sys.compute_styles_with_shadow(
        &doc,
        &parse("#host::part(inner) { color: rgb(0, 128, 0) !important; }"),
        &[shadow_scope(&doc, host, "span { color: rgb(255, 0, 0) !important; }")],
    );
    assert_eq!(
        color_of(&styles, span),
        ColorValue::Rgba(255, 0, 0, 255),
        "important 轴内树反转胜外树 part 声明"
    );
}
