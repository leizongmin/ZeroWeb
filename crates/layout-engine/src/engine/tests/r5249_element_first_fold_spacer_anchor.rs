//! R5249：element-first 首槽 spacer 锚点。
//!
//! walk 站位上首内容为元素子时 flush 无产出，旧实现无条件置 emitted_text_run
//! 消耗首槽——外层自身 ml+pl（R4332 折入含 border advance）整体丢失，内容
//! 左偏 frame（033-ref 形态）。修后以空文本零宽 run 归因外层自身落首槽：
//! break_lines 空 inline 分支消费 ml/pl 推进（CSS2 §10.2/§8.4），fold 仍落
//! **自名** run——盒 x 主臂 `first_ml − raw_ml` 检测式与链臂口径逐字节不变
//!（外盒 x 前后恒 0：min_x 与 fold 同步移动），可观察面 = 流内推进与内层盒。
//!
//! 锚：Ahem 50px span(border-left:10px) > span(color 分歧) "1234" + nbsp——
//! 修前签名：内容 run x=0（fold 悬流外）、nbsp x=200、外盒宽 60（自名仅
//! nbsp 段，并集丢 content 前缘）；修后：spacer x=10/ml=10、"1234" x=10
//!（fold 落流）、nbsp x=210、外盒 x=0 宽 260（border 10 + 内容 250）。

use super::*;

/// R5249 锚：element-first 折入落流（spacer 落首槽，内容随 frame 右移）。
#[test]
fn r5249_element_first_fold_spacer_lands_in_flow() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font:50px/1 Ahem; width:410px\"><span style=\"border-left:10px solid\">\
<span style=\"color:red\">1234</span>&nbsp;</span></div>\
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
    let outer_id = doc
        .child_nodes(div_id)
        .iter()
        .copied()
        .find(|&id| {
            doc.get(id)
                .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "span"))
        })
        .expect("外层 span 存在");

    // IFC 层：fold 落流签名。pure-Ahem 容器 stored IFC（fragments 含 spacer）。
    let ifc = div.inline_layout.as_ref().expect("pure-Ahem 容器须存 IFC");
    let frags: Vec<_> = ifc.iter().flat_map(|line| line.fragments.iter()).collect();
    let spacer = frags
        .iter()
        .find(|f| f.text.is_empty() && f.node_id == Some(outer_id))
        .expect("element-first 首槽 spacer（空文本、归因外层）应存在");
    assert!(
        (spacer.x - 10.0).abs() < 0.5,
        "spacer fragment x 应 10（cursor 0 + 折入 ml 10），实 {}",
        spacer.x
    );
    let content = frags.iter().find(|f| f.text == "1234").expect("内容 fragment 存在");
    assert!(
        (content.x - 10.0).abs() < 0.5,
        "内容 run x 应 10（fold 落流推进；修前 0 = fold 悬流外签名），实 {}",
        content.x
    );
    let nbsp = frags
        .iter()
        .find(|f| f.text.contains('\u{a0}'))
        .expect("nbsp fragment 存在");
    assert!(
        (nbsp.x - 210.0).abs() < 0.5,
        "nbsp run x 应 210（10 + 内容 200），实 {}",
        nbsp.x
    );

    // 盒层：外盒 x = margin 盒 0（min_x 与 fold 同步移动，前后恒 0），宽 = 框 10
    // + 内容 250（spacer 0 + "1234" 200 + nbsp 50）。
    let outer_box = div
        .children
        .iter()
        .find(|b| b.node_id == Some(outer_id))
        .expect("外层 span 树盒存在");
    assert!(
        (outer_box.x - 0.0).abs() < 0.5,
        "外盒 x 应 0（cursor + raw ml；fold 已在流内不扣），实 {}",
        outer_box.x
    );
    assert!(
        (outer_box.width - 260.0).abs() < 0.5,
        "外盒宽应 260（border 10 + 内容 250；修前自名仅 nbsp 段并集 60 签名），实 {}",
        outer_box.width
    );
}
