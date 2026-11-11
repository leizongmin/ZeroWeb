//! R5503 kill-switch 回退锚（独立进程，R3858 纪律）：`ZW_PARA_BIDI_BOX_LEVEL=0`
//! 时竖排 rtl 行内轴推进臂与水平臂同关，复现旧行为——竖排 rtl 盒按 DOM 序自顶
//! 装配（R5502 探针 ZW 实测：ga 8/gb 108；文本带 [0, 168] 自顶起）。
//! 主判例 r5503_vertical_rtl_inline_order.rs 断言相反方向。

use std::sync::Arc;

use zero_layout_engine::{LayoutEngine, types::LayoutBox};
use zero_style_system::StyleSystem;

fn compute(css: &str, body_inner: &str) -> (Arc<LayoutBox>, zero_dom::Document) {
    let html =
        format!(r#"<html><head><style>body {{ margin: 8px; }} {css}</style></head><body>{body_inner}</body></html>"#);
    let doc = zero_dom::parse_html(&html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(f64::from(800.0_f32), f64::from(600.0_f32));
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let root = engine.compute(&doc, &styles).root;
    (root, doc)
}

fn find_by_id(root: &LayoutBox, doc: &zero_dom::Document, id: &str) -> (f32, f32, f32, f32) {
    fn walk(b: &LayoutBox, doc: &zero_dom::Document, id: &str, px: f32, py: f32) -> Option<(f32, f32, f32, f32)> {
        let ax = px + b.padding_left + b.border_left + b.x;
        let ay = py + b.padding_top + b.border_top + b.y;
        let hit = b
            .node_id
            .and_then(|nid| doc.get(nid))
            .and_then(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => Some(e.get_attribute("id").as_deref() == Some(id)),
                _ => None,
            })
            .unwrap_or(false);
        if hit {
            return Some((ax, ay, b.width, b.height));
        }
        for c in &b.children {
            if let Some(f) = walk(c, doc, id, ax, ay) {
                return Some(f);
            }
        }
        None
    }
    walk(root, doc, id, 0.0, 0.0).unwrap_or_else(|| panic!("#{id} 盒不在树"))
}

const VBOXES: &str = r#"<div id="d"><span id="ga" style="display:inline-block;width:50px;height:100px;background:#0f0"></span><span id="gb" style="display:inline-block;width:50px;height:200px;background:#f00"></span></div>"#;

#[test]
fn kill_switch_restores_vertical_dom_order() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；pass 入口首读 env 前生效
    //（R5501 同款模式）。
    unsafe { std::env::set_var("ZW_PARA_BIDI_BOX_LEVEL", "0") };
    // V1 旧世界：竖排 rtl 盒按 DOM 序自顶装配（R5502 探针 ZW 实测 ga 8/gb 108）。
    let (root, doc) = compute(
        "div { writing-mode: vertical-rl; direction: rtl; height: 600px; width: 108px; }",
        VBOXES,
    );
    let (_, gy, _, _) = find_by_id(&root, &doc, "ga");
    let (_, by, _, _) = find_by_id(&root, &doc, "gb");
    assert_eq!(gy, 8.0, "kill-switch 关闭后竖排 rtl 须回 DOM 序自顶装配（旧行为锚）");
    assert_eq!(by, 108.0);
}
