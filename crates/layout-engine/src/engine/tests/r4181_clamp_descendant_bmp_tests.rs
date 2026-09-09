//! R4181（css-overflow-4 #line-clamp）：clamp 点落入带块向 bmp 的子盒时扣减其 padding/border。
//!
//! line-clamp-auto-020 assert「If line-clamp: auto has the clamp point in a descendant,
//! it should take its bmp into account」——约束 6lh=192px，子盒上下 bmp 各 17px
//!（padding 15 + border 2），盒内可见行数 = (192-34)/32 = 4 行 + 末行省略号。
//! 旧实现按 remaining 全额放行（6 行），首行未下移、整盒超出约束。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32, Option<usize>)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.y, b.height, b.line_clamp_cap));
        }
        stack.extend(b.children.iter());
    }
    None
}

/// auto-020 结构：clamp:auto + max-height:6lh 容器 > 带 padding:15 + border:2 的子 div
/// 持 6 行 pre 文本。bmp 扣减后盒内可见 4 行，盒高 = 34 + 4×32 = 162。
#[test]
fn r4181_descendant_bmp_deducted_from_line_budget() {
    let html = r#"<html><head><style>
.clamp {
  line-clamp: auto;
  max-height: 6lh;
  font: 16px / 32px serif;
  white-space: pre;
}
.clamp > div {
  padding: 15px;
  border: 2px solid black;
}
</style></head><body style="margin:0">
<div class="clamp"><div>Line 1
Line 2
Line 3
Line 4
Line 5
Line 6</div></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(
        ".clamp { line-clamp: auto; max-height: 6lh; font: 16px / 32px serif; white-space: pre; } .clamp > div { padding: 15px; border: 2px solid black; }",
    );
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let inner = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .nth(1)
        .expect("inner div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (_y, h, cap) = find_box(&result.root, inner).expect("inner box");
    assert_eq!(cap, Some(4), "R4181: 盒内可见行数应扣 bmp 后为 4（旧实现全额 6）");
    assert!(
        (h - (34.0 + 4.0 * 32.0)).abs() < 0.5,
        "R4181: 盒高应保留 bmp = 34 + 4×32 = 162，实际 {h}（旧实现按 visible_h 覆盖会切掉 padding/border）"
    );
}

/// R4181b（auto-042）：min-height 托底约束阻止内截——clamp 点本落 .min-height 盒内
///（行 5-8 + Line 9，剩余预算 2 行 < 盒 4 行内容），盒 min-height:3lh=96 > cap 收缩高
/// 64 → 内容驱动盒不可内截，clamp 点退到盒前（hide 盒 + 前兄末行省略号）。旧实现内截
/// 2 行，min-height 地板被违反。
#[test]
fn r4181b_min_height_content_driven_box_retreats_clamp_point() {
    let html = r#"<html><head><style>
.clamp {
  line-clamp: auto;
  max-height: 6lh;
  font: 16px / 32px serif;
  white-space: pre;
}
.min-height {
  min-height: 3lh;
  background-color: red;
}
</style></head><body style="margin:0">
<div class="clamp">Line 1
Line 2
Line 3
Line 4
<div class="min-height">Line 5
Line 6
Line 7
Line 8</div>
Line 9</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(
        ".clamp { line-clamp: auto; max-height: 6lh; font: 16px / 32px serif; white-space: pre; } .min-height { min-height: 3lh; background-color: red; }",
    );
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let inner = doc
        .get_elements_by_tag_name("div")
        .into_iter()
        .nth(1)
        .expect("min-height div");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let (_y, _h, _cap, hidden) = {
        let mut found = None;
        let mut stack: Vec<&crate::types::LayoutBox> = vec![&result.root];
        while let Some(b) = stack.pop() {
            if b.node_id == Some(inner) {
                found = Some((b.y, b.height, b.line_clamp_cap, b.line_clamp_hidden));
                break;
            }
            for c in &b.children {
                stack.push(c);
            }
        }
        found.expect("min-height box")
    };
    assert!(
        hidden,
        "R4181b: min-height 托底的内容驱动盒应整体隐藏（clamp 点退到盒前），cap={:?}",
        _cap
    );
}
