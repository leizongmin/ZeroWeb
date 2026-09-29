//! display:none 子树文本不泄入父 IFC（CSS2 §9.3 / CSS Display 3 §2.1）。
//!
//! layout 趟有 styles 直判（collect_inline_items 的 display:none 分支）；paint Path B
//! （空 styles 重跑 IFC）靠 `display_none_walk_nodes` 注入集判定——本文件锚定两条
//! 路径与缺省零回归（不注入 = 旧行为）。

use super::super::InlineFormattingContext;
use crate::NodeIdSet;
use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, PositionValue};
use zero_dom::{Document, NodeId, NodeKind, parse_html};
use zero_style_system::ComputedStyle;

fn style(display: DisplayValue, position: PositionValue) -> ComputedStyle {
    let mut s = ComputedStyle::default();
    s.display = display;
    s.position = position;
    s
}

fn body_paragraph(doc: &Document) -> NodeId {
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    doc.first_child(body).unwrap()
}

fn first_element_child(doc: &Document, id: NodeId) -> Option<NodeId> {
    doc.child_nodes(id)
        .iter()
        .copied()
        .find(|&c| doc.get(c).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))))
}

fn fragments_text(ctx: &InlineFormattingContext) -> String {
    ctx.all_fragments().iter().map(|f| f.text.clone()).collect()
}

/// Path B（空 styles）+ 注入集：隐藏 span 文本不收集，相邻文本不受影响。
#[test]
fn display_none_inline_child_text_excluded_via_walk_nodes() {
    let doc = parse_html("<p>VIS<span>X-SECRET</span>IBLE</p>");
    let p = body_paragraph(&doc);
    let span = first_element_child(&doc, p).expect("span 应存在");

    let mut hidden = NodeIdSet::default();
    hidden.insert(span);
    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.display_none_walk_nodes = hidden;
    ctx.layout(&doc, p, &HashMap::new());

    let text = fragments_text(&ctx);
    assert!(!text.contains("X-SECRET"), "隐藏 span 文本不应收集，实际: {text}");
    assert!(text.contains("VIS"), "相邻前段文本应保留，实际: {text}");
    assert!(text.contains("IBLE"), "相邻后段文本应保留，实际: {text}");
}

/// layout 趟（styles 非空）：display:none 直判路径不受注入集影响。
#[test]
fn display_none_inline_child_text_excluded_with_styles() {
    let doc = parse_html("<p>VIS<span>X-SECRET</span>IBLE</p>");
    let p = body_paragraph(&doc);
    let span = first_element_child(&doc, p).expect("span 应存在");

    let mut styles = HashMap::new();
    styles.insert(p, style(DisplayValue::Block, PositionValue::Static));
    styles.insert(span, style(DisplayValue::None, PositionValue::Static));

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &styles);

    let text = fragments_text(&ctx);
    assert!(
        !text.contains("X-SECRET"),
        "styles 直判隐藏 span 不应收集，实际: {text}"
    );
    assert!(
        text.contains("VIS") && text.contains("IBLE"),
        "相邻文本应保留，实际: {text}"
    );
}

/// 缺省零回归：不注入 + 空 styles（旧 Path B 行为）仍收集全部文本。
#[test]
fn no_injection_keeps_legacy_collection() {
    let doc = parse_html("<p>VIS<span>X-SECRET</span>IBLE</p>");
    let p = body_paragraph(&doc);

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, p, &HashMap::new());

    let text = fragments_text(&ctx);
    assert!(
        text.contains("X-SECRET"),
        "缺省（未注入）应保持旧行为收集，实际: {text}"
    );
}
