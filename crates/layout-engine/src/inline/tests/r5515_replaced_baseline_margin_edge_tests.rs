//! R5515（CSS2 §10.8.1）：行内替换元素（img）基线 = margin box 底边——baseline 字段
//! 口径是 border box 顶到基线的距离（R3904：margin_top 由 ascent 循环另加），故补
//! margin_bottom。驱动案 css-ui/box-sizing-009（R5418 挂账①残差 ✗2.50%）：t01
//! （mb +10）/ t02（mb −10）同带时 margin box 底锚行基线 → t02 border 底悬基线下
//! 10px（旧：字段取 border box 底 → 两图同底、行基线偏低 10）。mb=0 值恒等零扰动。

use crate::types::{InlineLayoutLine, LayoutBox};
use std::sync::Arc;
use zero_dom::{Document, NodeKind};

fn layout(html: &str, css: &str) -> crate::LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 收集（首个）存树 IFC 行。
fn find_ifc(node: &crate::LayoutBox) -> Option<&Vec<InlineLayoutLine>> {
    if let Some(lines) = &node.inline_layout {
        return Some(lines);
    }
    node.children.iter().find_map(find_ifc)
}

/// 找指定 id 的 LayoutBox（深度优先）。
fn find_id<'a>(b: &'a LayoutBox, doc: &'a Document, id: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_id(c, doc, id))
}

/// 驱动案 t02 形（mb −10 单图，Ahem strut 12 不参与）：行基线 = mt + (atom_h + mb)
/// = 10 + 40 = 50——margin box 底锚基线，border 底悬基线下 10。旧：字段取 border box
/// 底 → 基线 60。
#[test]
fn r5515_img_negative_mb_baseline_margin_box_bottom() {
    let css = "div { font: 10px/20px Ahem; } \
               img { width: 50px; height: 50px; margin-top: 10px; margin-bottom: -10px; }";
    let html = format!(r#"<html><head><style>{css}</style></head><body><div><img /></div></body></html>"#);
    let result = layout(&html, css);
    let lines = find_ifc(&result.root).expect("div 应有存树 IFC");
    assert_eq!(lines.len(), 1, "应单行，实得 {}", lines.len());
    assert!(
        (lines[0].baseline_y - 50.0).abs() < 0.01,
        "行基线应 = mt10 + (50−10) = 50（margin box 底边锚定），实得 {}",
        lines[0].baseline_y
    );
}

/// 同带双图（box-sizing-009 行 2 形：a = t01 mb+10 / b = t02 mb−10）：基线 =
/// max(mt+field) = 10+60 = 70；margin box 底各锚基线（a.y+50+10=70、b.y+50−10=70）
/// → b border 底悬基线下 10，b.y − a.y = 20。旧：字段取 border box 底 → 基线 60、
/// 两图同底（y 差 0）。
#[test]
fn r5515_same_line_mixed_mb_margin_box_anchoring() {
    let css = "div { font: 10px/20px Ahem; } \
               img { width: 50px; height: 50px; margin: 10px 0; } \
               #b { margin-bottom: -10px; }";
    let html =
        format!(r#"<html><head><style>{css}</style></head><body><div><img id=a /><img id=b /></div></body></html>"#);
    let doc = zero_dom::parse_html(&html);
    let result = layout(&html, css);
    let lines = find_ifc(&result.root).expect("div 应有存树 IFC");
    assert_eq!(lines.len(), 1, "应单行，实得 {}", lines.len());
    assert!(
        (lines[0].baseline_y - 70.0).abs() < 0.01,
        "行基线应 = 10 + (50+10) = 70（a 的 margin box 底锚定），实得 {}",
        lines[0].baseline_y
    );
    let baseline_y = lines[0].baseline_y;
    let root = Arc::try_unwrap(result.root).unwrap_or_else(|arc| (*arc).clone());
    let a = find_id(&root, &doc, "a").expect("img a 存在");
    let b = find_id(&root, &doc, "b").expect("img b 存在");
    assert!(
        (a.y + 50.0 + 10.0 - baseline_y).abs() < 0.01,
        "a margin box 底应锚基线（y={}+50+10=70），实得 {}",
        a.y,
        baseline_y
    );
    assert!(
        (b.y + 50.0 - 10.0 - baseline_y).abs() < 0.01,
        "b margin box 底应锚基线（border 底悬基线下 10），y={} 实得 {}",
        b.y,
        baseline_y
    );
    assert!(
        (b.y - a.y - 20.0).abs() < 0.01,
        "b.y − a.y 应 20（mb 差 ±10 的 border 位差），实得 {}",
        b.y - a.y
    );
}

/// mb=0 守卫：基线 = mt + atom_h = 60、y = 10——与旧实现逐字节同值（零扰动）。
#[test]
fn r5515_zero_mb_unchanged() {
    let css = "div { font: 10px/20px Ahem; } \
               img { width: 50px; height: 50px; margin-top: 10px; }";
    let html = format!(r#"<html><head><style>{css}</style></head><body><div><img /></div></body></html>"#);
    let result = layout(&html, css);
    let lines = find_ifc(&result.root).expect("div 应有存树 IFC");
    assert!(
        (lines[0].baseline_y - 60.0).abs() < 0.01,
        "mb=0 基线应 = mt10 + 50 = 60 不变，实得 {}",
        lines[0].baseline_y
    );
}

/// 正 mb 单图占行：行 ascent 计入全 margin box（A = mt+atom_h+mb = 70，旧 60）——
/// margin box 参与行盒围合的基线面（R3904/R5394 同语境）。
#[test]
fn r5515_positive_mb_line_ascent_includes_margin_box() {
    let css = "div { font: 10px/20px Ahem; } \
               img { width: 50px; height: 50px; margin: 10px 0; }";
    let html = format!(r#"<html><head><style>{css}</style></head><body><div><img /></div></body></html>"#);
    let result = layout(&html, css);
    let lines = find_ifc(&result.root).expect("div 应有存树 IFC");
    assert!(
        (lines[0].baseline_y - 70.0).abs() < 0.01,
        "正 mb 行基线应 = 10 + (50+10) = 70（margin box 全高参与 ascent），实得 {}",
        lines[0].baseline_y
    );
}
