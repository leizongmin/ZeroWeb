//! R5250：relative inline 子 inset 视觉位移映射锚点。
//!
//! 纯 inline 非替换盒无 taffy 节点，其行内流中 relative inline 子的 inset 无人施加
//!（R5225 判例「display:inline 非替换盒 inset 由 taffy 单次施加」对纯 inline 不成立）。
//! compute_final 以真实 styles 构建 per-node 位移映射存盒（`inline_relpos_shifts`），
//! stored IFC fragment 与 paint Path B（空 styles 重跑）据表平移。
//! CSS2 §9.3.2 https://www.w3.org/TR/CSS22/visuren.html#relative-positioning

use super::*;

/// R5250 锚：位移映射构建（032 形态——嵌套 span relative top:25px，纵向）。
/// 容器 div 无 inset；外层 span 无 inset 不入表（零位移跳过）；span1 = (0, 25)。
#[test]
fn r5250_relpos_inline_shift_map_built() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font:20px/1 Ahem\"><span>AAA<span style=\"position:relative;top:25px\">BBB</span>CCC</span></div>\
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
    let outer = doc
        .child_nodes(div_id)
        .iter()
        .copied()
        .find(|&id| {
            doc.get(id)
                .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(_)))
        })
        .expect("外层 span 存在");
    let span1 = doc
        .child_nodes(outer)
        .iter()
        .copied()
        .find(|&id| {
            doc.get(id)
                .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(_)))
        })
        .expect("span1 存在");
    let shift = div
        .inline_relpos_shifts
        .as_deref()
        .and_then(|m| m.get(&span1))
        .copied()
        .expect("div 盒应存 span1 位移映射");
    assert!(
        (shift.0 - 0.0).abs() < 0.5 && (shift.1 - 25.0).abs() < 0.5,
        "span1 位移应 (0, 25)，实 {:?}",
        shift
    );
}

/// R5250 锚：stored IFC fragment 平移（pure-Ahem 容器存 IFC 路径，033 横向形态）。
/// span2 relative left:-6em @20px = -120px——其名下文本 fragment x 整体左移 120，
/// 外层无 inset 文本（span1 名下 AAA）保持静态位。
#[test]
fn r5250_relpos_inline_stored_fragments_shifted() {
    let html = "<html><body style=\"margin:0\">\
<div style=\"font:20px/1 Ahem\"><span>AAA<span style=\"position:relative;left:-6em\">BBB</span>CCC</span></div>\
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
    let ifc = div.inline_layout.as_ref().expect("pure-Ahem 容器须存 IFC");
    let frags: Vec<_> = ifc.iter().flat_map(|line| line.fragments.iter()).collect();
    let describe = || {
        frags
            .iter()
            .map(|f| format!("{{text={:?} x={} nid={:?}}}", f.text, f.x, f.node_id))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let bbb = frags
        .iter()
        .find(|f| f.text.contains("BBB"))
        .unwrap_or_else(|| panic!("BBB fragment 存在，实际 fragments: {}", describe()));
    let aaa = frags
        .iter()
        .find(|f| f.text.contains("AAA"))
        .unwrap_or_else(|| panic!("AAA fragment 存在，实际 fragments: {}", describe()));
    // 静态流：AAA@0 宽 60 → BBB@60；位移 -120 → BBB x = -60。
    assert!(
        (aaa.x - 0.0).abs() < 0.5,
        "AAA（无 inset 祖先链）应保持静态 x=0，实 {}",
        aaa.x
    );
    assert!(
        (bbb.x - (-60.0)).abs() < 0.5,
        "BBB fragment x 应 -60（静态 60 + inset -120），实 {}",
        bbb.x
    );
}
