//! R5331：column flex 容器内在主轴定尺的 item 内容贡献探针（css-flexbox-1 §9.9.1）。
//!
//! 根因：determine_container_main_size 的内在臂用 `SizingMode::InherentSize` 测 item
//! 内容贡献，taffy ComputeSize 的 style 注入（compute_flexbox_layout clamped_style_size）
//! 把 item 显式主轴定尺直接顶替 content 贡献——§9.9.1 容器内在主尺寸 = Σ item
//! hypothetical main size，specified size 只能经 automatic minimum
//!（min(specified, content)，§4.5）进入。修复 = `flex_intrinsic_main_probe` 旗标
//!（仅 column 主轴 ∧ item 有 specified main size ∧ flex-basis definite），探针测量后
//! 再被 specified main size 钳。kill-switch = `ZW_FLEX_INTRINSIC_MAIN_PROBE=0`
//!（default-on）。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use std::sync::Arc;
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::StyleSystem;

fn compute(html: &str) -> (Document, LayoutBox, HashMap<NodeId, zero_style_system::ComputedStyle>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let root = Arc::try_unwrap(engine.compute(&doc, &styles).root).unwrap_or_else(|arc| (*arc).clone());
    (doc, root, styles)
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

/// 找指定 tag 名（首个）的 (NodeId, &LayoutBox)。
fn find_tag<'a>(doc: &'a Document, b: &'a LayoutBox, tag: &str) -> Option<(NodeId, &'a LayoutBox)> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.local_name().eq_ignore_ascii_case(tag)
    {
        return Some((nid, b));
    }
    for c in &b.children {
        if let Some(found) = find_tag(doc, c, tag) {
            return Some(found);
        }
    }
    None
}

/// 驱动腿（flex-minimum-height-flex-items-029 形）：外层 auto 高 column flex > item
/// `flex:1 0 0px; height:500px`（column wrap）> 两孙项各含 50px 子。item automatic
/// minimum = min(content 100, specified 500) = 100，外层容器高应 100（非 style 注入的
/// 500）；孙项 flex 到 50/50。
#[test]
fn r5331_col_flex_item_specified_height_probe_clamps_container() {
    let html = r#"<html><body><div style="display:flex;flex-direction:column;">
  <div id="inner" style="display:flex;flex-direction:column;flex-wrap:wrap;flex:1 0 0px;height:500px;">
    <div style="flex:1 0 0px;width:100px;background:green;"><div style="height:50px;"></div></div>
    <div style="flex:1 0 0px;width:100px;background:green;"><div style="height:50px;"></div></div>
  </div>
</div></body></html>"#;
    let (doc, root, _) = compute(html);
    let outer = find_tag(&doc, &root, "body")
        .and_then(|(_, b)| b.children.first())
        .expect("outer flex");
    let inner = find_id(&root, &doc, "inner").expect("inner flex");
    assert!(
        (outer.height - 100.0).abs() < 1.0,
        "外层 auto 高 column flex h={:.1} 应 100 = clamp(basis 0, min(content 100, specified 500))，非 style 顶替的 500（css-flexbox-1 §9.9.1）",
        outer.height
    );
    assert!(
        (inner.height - 100.0).abs() < 1.0,
        "inner item h={:.1} 应 flex 到 100（§4.5 automatic minimum）",
        inner.height
    );
}

/// 驱动腿（flex-minimum-height-flex-items-030 形）：specified main size 小于 content 时
/// 贡献取 min 侧——item `flex:1 0 0px; height:70px`（column wrap）+ 两孙项各含 100px 子
/// （content 200）。item automatic minimum = min(content 200, specified 70) = 70，外层
/// 容器高应 70；孙项 100 放不下换列保持 100。
#[test]
fn r5331_col_flex_item_specified_height_below_content_takes_min() {
    let html = r#"<html><body><div style="display:flex;flex-direction:column;width:100px;">
  <div id="inner" style="display:flex;flex-direction:column;flex-wrap:wrap;flex:1 0 0px;height:70px;">
    <div id="leaf1" style="flex:1 0 0px;width:50px;background:green;"><div style="height:100px;"></div></div>
    <div style="flex:1 0 0px;width:50px;background:green;"><div style="height:100px;"></div></div>
  </div>
</div></body></html>"#;
    let (doc, root, _) = compute(html);
    let outer = find_tag(&doc, &root, "body")
        .and_then(|(_, b)| b.children.first())
        .expect("outer flex");
    let leaf1 = find_id(&root, &doc, "leaf1").expect("leaf item");
    assert!(
        (outer.height - 70.0).abs() < 1.0,
        "外层 auto 高 column flex h={:.1} 应 70 = clamp(basis 0, min(content 200, specified 70))，非裸 content 200（css-flexbox-1 §4.5）",
        outer.height
    );
    assert!(
        (leaf1.height - 100.0).abs() < 1.0,
        "孙项 h={:.1} 应保持 automatic minimum 100（shrink 0 放不下换列不收缩）",
        leaf1.height
    );
}

/// 守卫腿（basis auto 不在探针域）：item 无 flex 简写（basis auto → base = main size
/// property），`.max(flex_basis)` 已覆盖 specified size——探针不启用，容器高仍 =
/// item 显式高（§9.2.3 flex base size 求解路径不变）。
#[test]
fn r5331_col_flex_item_auto_basis_keeps_specified_height() {
    let html = r#"<html><body><div style="display:flex;flex-direction:column;">
  <div id="item" style="height:70px;"><div style="height:200px;"></div></div>
</div></body></html>"#;
    let (doc, root, _) = compute(html);
    let outer = find_tag(&doc, &root, "body")
        .and_then(|(_, b)| b.children.first())
        .expect("outer flex");
    assert!(
        (outer.height - 70.0).abs() < 1.0,
        "basis auto item（height:70px）h_outer={:.1} 应 70（flex base = main size property，探针不适用）",
        outer.height
    );
}

/// 守卫腿（row 主轴不在探针域）：float shrink-to-fit 的 flex 容器内在宽面已按
/// Chrome/WebKit/Firefox 校准（csswg#1435、flex_basis_unconstraint_row gentest）——item
/// `flex:1 0 0px; width:200px` 的 specified width 仍注入容器内在宽（探针仅 column）。
#[test]
fn r5331_row_flex_item_specified_width_contributes() {
    let html = r#"<html><body><div id="row" style="display:flex;float:left;">
  <div id="item" style="flex:1 0 0px;width:200px;"></div>
</div></body></html>"#;
    let (doc, root, _) = compute(html);
    let row = find_id(&root, &doc, "row").expect("float flex");
    assert!(
        (row.width - 200.0).abs() < 1.0,
        "float flex 容器 shrink-to-fit 宽 w={:.1} 应 200（row 主轴 specified width 注入面不在探针域）",
        row.width
    );
}
