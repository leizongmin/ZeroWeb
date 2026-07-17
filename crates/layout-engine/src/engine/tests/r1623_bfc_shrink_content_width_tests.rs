//! R1623：BFC 被 float 排斥收缩 width 后同步 content_width 回归测试。
//!
//! floats-bfc-003 谱系：BFC（overflow:hidden, auto width）旁有 float，BFC float 排斥
//! 把 BFC width 收缩到「float 旁可用宽」。旧实现只更 child.width 不更 child.content_width，
//! 致内层 adjust_float_positions 递归用旧（大）content_width 作 container_width，BFC 内
//! float 不按收缩后宽换行/堆叠 → 溢出 BFC。R1623 fix：收缩 width 时同步 content_width
//!（= width - frame）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::{FloatValue, OverflowValue};
use zero_style_system::StyleSystem;

fn find_beside_float_bfc<'a>(
    root: &'a LayoutBox,
    styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
) -> Option<&'a LayoutBox> {
    let is_bfc = root.node_id.is_some_and(|id| {
        styles
            .get(&id)
            .is_some_and(|s| matches!(s.overflow_x, OverflowValue::Hidden | OverflowValue::Scroll))
    });
    if is_bfc {
        return Some(root);
    }
    for c in &root.children {
        if let Some(b) = find_beside_float_bfc(c, styles) {
            return Some(b);
        }
    }
    None
}

/// R1623：BFC（overflow:hidden, auto width）旁有 left float，BFC width 被收缩到可用宽后，
/// content_width 须同步收缩——验证内层 float（2×60 在 100 宽 BFC 内放不下）堆叠而非并排。
#[test]
fn test_bfc_shrink_syncs_content_width_inner_floats_stack() {
    // 容器 200 宽；left float 100×10；BFC（overflow:hidden, auto width）含 2 个 60×40 float。
    // BFC 被 float 排斥收缩到 100 宽（=200-100）后，content_width 应=100，内层 2×60 float
    //（120 > 100 放不下并排）应堆叠（h=80）。无 fix（content_width=200 旧值）则并排 h=40。
    let html = r#"<html><body style="margin:0"><div style="width:200px"><div style="float:left;width:100px;height:10px"></div><div style="overflow:hidden"><div style="float:left;width:60px;height:40px"></div><div style="float:left;width:60px;height:40px"></div></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc = find_beside_float_bfc(&result.root, &styles).expect("should find overflow:hidden bfc");
    // content_width 应与收缩后 width 一致（≈100，frame=0）。
    assert!(
        (bfc.content_width - bfc.width).abs() < 1.0,
        "bfc content_width must sync with shrunk width; got width={} content_width={}",
        bfc.width,
        bfc.content_width
    );
    // 内层 2×50 float 在 100 宽内应堆叠 → BFC 高度 ≈80（并排溢出则 ≈40）。
    assert!(
        bfc.height > 70.0,
        "inner floats must stack in shrunk bfc (h≈80), got bfc height={}",
        bfc.height
    );
    let _ = FloatValue::None;
}
