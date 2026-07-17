//! R1619：嵌套 BFC（在非 BFC wrapper 内）旁 float 放不下时下沉到 float 底（CSS §9.5）回归测试。
//!
//! floats-wrap-bfc-with-margin-008/009 谱系：wrapper(100) 含 right/left float(50×50) +
//! 非 BFC margin-div（margin-right/left:50px）内嵌 bfc(overflow:hidden width:100 height:50)。
//! bfc declared 宽 100 > float 旁可用宽 50 → 不应被 shrink 到 50 并排重叠，而应**下沉**到
//! float 底（y=50），wrapper 高度随之增长到 100。
//!
//! R1619 fix（float_positioning.rs）：透传祖先 float 几何到非 BFC 后代（Slice 2）+
//! declared-width BFC（!declared_width_auto）放不下 float 旁可用宽时下沉（替代 R1369 的
//! `width < container_width` 代理，嵌套+margin 上下文下 BFC 溢出窄父失效）。env
//! `ZW_NESTED_BFC_FLOAT_AVOID=0` 关闭（kill-switch，default-on）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::{FloatValue, OverflowValue};
use zero_style_system::StyleSystem;

/// 找到 overflow:hidden 的 BFC 盒（模拟 .bfc）。
fn find_overflow_hidden_bfc<'a>(
    root: &'a LayoutBox,
    styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
) -> Option<&'a LayoutBox> {
    let is_bfc = root.node_id.is_some_and(|id| {
        styles.get(&id).is_some_and(|s| {
            matches!(
                s.overflow_x,
                OverflowValue::Hidden | OverflowValue::Scroll | OverflowValue::Clip
            )
        })
    });
    if is_bfc {
        return Some(root);
    }
    for c in &root.children {
        if let Some(b) = find_overflow_hidden_bfc(c, styles) {
            return Some(b);
        }
    }
    None
}

/// R1619-008（right float）：bfc declared width:100 在 margin-right:50px wrapper 内，
/// 旁 right float(50×50) 放不下（可用 50 < 100）→ 下沉到 float 底（y≈50）。
#[test]
fn test_nested_bfc_drops_below_right_float() {
    let html = r#"<html><body style="margin:0"><div style="width:100px;background:green"><div style="float:right;width:50px;height:50px;background:green"></div><div style="margin-right:50px"><div style="overflow:hidden;width:100px;height:50px;background:green"></div></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc = find_overflow_hidden_bfc(&result.root, &styles).expect("should find overflow:hidden bfc");
    assert!(
        bfc.y > 40.0,
        "nested declared-width BFC must drop below right float (y≈50), got y={}",
        bfc.y
    );
}

/// R1619-009（left float）：对称——left float 旁 declared-width BFC 下沉。
#[test]
fn test_nested_bfc_drops_below_left_float() {
    let html = r#"<html><body style="margin:0"><div style="width:100px;background:green"><div style="float:left;width:50px;height:50px;background:green"></div><div style="margin-left:50px"><div style="overflow:hidden;width:100px;height:50px;background:green"></div></div></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc = find_overflow_hidden_bfc(&result.root, &styles).expect("should find overflow:hidden bfc");
    assert!(
        bfc.y > 40.0,
        "nested declared-width BFC must drop below left float (y≈50), got y={}",
        bfc.y
    );
    let _ = FloatValue::None;
}
