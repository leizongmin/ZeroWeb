//! R5248：inline 树盒 x 双扣 border 修正锚点。
//!
//! R4297-F x 式（内容首缘 − ml − pl − bl）写于 margin_left 尚为 raw 值时；R4332 把
//! border advance 折入 fragment.margin_left 并推进流后，独立 border_left 项与 −ml
//! 项对折入双扣——有框 inline 树盒 x 左偏 border_w，box-level 边框/背景同步左移
//!（multicol-rule-dashed-000 span border-left 段 198 vs 208 实证）。R5248 终版：
//! x = 流内内容首缘 − **流内落账** frame（折入落流与否按首 run ml−raw 检测；
//! R4332 bidi 控制符清零与 element-first 首槽消耗两态不落流不扣）。
//!
//! 锚：Ahem 50px "1234" + span(border-left:10px) "5678"（div 410px）——span 树盒
//! x = cursor("1234" 末尾) + raw ml 0 = 200（= 内容首缘 210 − 落流折入 10），修前
//! 190（双扣 10）。

use super::*;

/// R5248 锚：有框 inline 树盒 x = 流内内容首缘 − 流内落账 frame（text-first 折入
/// 落流 → 扣 bl；raw ml 已在流内不再扣）。
#[test]
fn r5248_inline_box_x_no_border_double_count() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font:50px Ahem; width:410px\">1234<span style=\"border-left:10px solid\">5678</span></div>\
</body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = result
        .root
        .children
        .iter()
        .flat_map(|c| &c.children)
        .find(|b| b.is_block_level)
        .expect("块容器存在");
    let div_id = div.node_id.expect("div 有 node id");
    let span_id = doc
        .child_nodes(div_id)
        .iter()
        .copied()
        .find(|&id| {
            doc.get(id)
                .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "span"))
        })
        .expect("span 元素存在");

    // span 树盒（inline_finalization R4297-F 重写 x）挂 div 盒 children。
    let span_box = div
        .children
        .iter()
        .find(|b| b.node_id == Some(span_id))
        .expect("span 树盒存在");
    assert!(
        (span_box.x - 200.0).abs() < 0.5,
        "span 树盒 x 应 200（border 盒左缘 = \"1234\" 末尾；双扣时 190），实 {}",
        span_box.x
    );
    // 宽 = 内容 200 + 左 border 10 = 210（右缘 410 = div 内容右缘）。
    assert!(
        (span_box.width - 210.0).abs() < 0.5,
        "span 树盒宽应 210（内容 + 左框），实 {}",
        span_box.width
    );
}
