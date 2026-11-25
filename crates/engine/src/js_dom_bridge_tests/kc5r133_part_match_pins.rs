// wptfyi-css k133 刀 常驻钉测（2026-11-26）：part-match——`::part(<ident-list>)`
// 解析＋匹配＋级联面端到端（css-shadow-parts §3.2.2）。钉面三向：
//   P1 shadow 表 `:host::part(inner)` 命中 shadow 树内 part 元素（zwsh 选择器 gCS
//      green——parser Part 变体＋matcher Part 臂＋1.3 part 段收集全链；树内裸形
//      `::part(inner)` 按 §5.4 不命中，shadow_part_segment 钉覆盖）；
//   P2 `::part(x)::before` 伪元素尾对齐 → before_pseudo 存储＋
//      serialize_pseudo_element_property 投影读值（gCS 第二参投影面）；
//   P3 投影阴性/回退分界：名单外伪元素（::first-line）无存储槽→None；
//      名单内未计算槽（::marker 非 li）→槽 miss 回退 originating 元素自身
//      （css-pseudo-4 §2 Chrome 形，values 判据页对齐）。
// https://drafts.csswg.org/css-shadow-parts/#part

const K133_PART_HTML: &str = "<html><body><div id=\"host\"><template shadowrootmode=\"open\">\
<style>:host::part(inner) { color: rgb(0, 128, 0); }</style><span id=\"p1\" part=\"inner\">x</span></template></div></body></html>";

const K133_PART_BEFORE_HTML: &str = "<html><body><div id=\"host\"><template shadowrootmode=\"open\">\
<style>:host::part(pb) { color: rgb(0, 0, 4); } :host::part(pb)::before { color: rgb(11, 22, 33); }</style>\
<span id=\"p2\" part=\"pb\">x</span></template></div></body></html>";

/// P1：shadow 表 `:host::part(inner)` 规则经 1.3 part 段收集命中 `part="inner"` 元素。
#[test]
fn test_kc5r133_part_rule_styles_part_element() {
    let (doc, styles) = crate::js_dom_bridge::compute_document_styles(K133_PART_HTML);
    let color = crate::js_dom_bridge::lookup_computed_property(&doc, &styles, "zwsh5:#host|#p1", "color");
    assert_eq!(color, "rgb(0, 128, 0)", "P1 ::part rule styles part element, got {color:?}");
}

/// P2：`:host::part(pb)::before` 剥尾框架 → before_pseudo 计算＋投影 fn 读值。
#[test]
fn test_kc5r133_part_before_pseudo_projection() {
    let (doc, styles) = crate::js_dom_bridge::compute_document_styles(K133_PART_BEFORE_HTML);
    // 元素自身吃 :host::part(pb) 规则（非默认色，证规则真命中）。
    let color = crate::js_dom_bridge::lookup_computed_property(&doc, &styles, "zwsh5:#host|#p2", "color");
    assert_eq!(color, "rgb(0, 0, 4)", "P2 :host::part rule on element, got {color:?}");
    // before_pseudo 存储＋serialize_pseudo_element_property 投影。
    let node = crate::js_dom_bridge::find_by_selector(&doc, "zwsh5:#host|#p2").expect("P2 element found");
    let style = styles.get(&node).expect("P2 element style present");
    let before = style.before_pseudo.as_ref().expect("P2 ::part(pb)::before computes before_pseudo");
    assert_eq!(before.color, zero_style_system::property::ColorValue::Rgba(11, 22, 33, 255));
}

/// P3 投影阴性/回退分界：::first-line 名单外无存储槽 → None；::marker 名单内
/// 但仅 li 计算（R3867 门）→ span 槽 miss 回退 originating 元素自身
///（`:host::part(pb)` 规则值 rgb(0, 0, 4)）。
#[test]
fn test_kc5r133_pseudo_projection_negative() {
    let (doc, styles) = crate::js_dom_bridge::compute_document_styles(K133_PART_BEFORE_HTML);
    let node = crate::js_dom_bridge::find_by_selector(&doc, "zwsh5:#host|#p2").expect("P3 element found");
    let style = styles.get(&node).expect("P3 element style present");
    assert!(
        crate::js_dom_bridge::serialize_pseudo_element_property(style, "first-line", "color", None).is_none(),
        "P3 ::first-line has no storage slot"
    );
    assert_eq!(
        crate::js_dom_bridge::serialize_pseudo_element_property(style, "marker", "color", None),
        Some("rgb(0, 0, 4)".to_string()),
        "P3 ::marker slot miss falls back to originating element color"
    );
}

const K133_PART_DOCTABLE_HTML: &str = "<html><body><style>#host::part(inner) { color: rgb(0, 128, 0); }</style>\
<div id=\"host\"><template shadowrootmode=\"open\">\
<style>span { color: rgb(255, 0, 0); }</style><span id=\"p4\" part=\"inner\">x</span></template></div></body></html>";

/// P4：文档表 host 复合 `::part` 规则跨树命中 shadow 内 part 元素（all-hosts
/// 判据页形）＋级联序：part 段声明（文档表源）胜 shadow 表常规规则。
#[test]
fn test_kc5r133_part_document_table_rule() {
    let (doc, styles) = crate::js_dom_bridge::compute_document_styles(K133_PART_DOCTABLE_HTML);
    let color = crate::js_dom_bridge::lookup_computed_property(&doc, &styles, "zwsh5:#host|#p4", "color");
    assert_eq!(
        color, "rgb(0, 128, 0)",
        "P4 document-table ::part rule styles part element across tree, got {color:?}"
    );
}
