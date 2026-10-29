//! R5330：column flex item 主轴高 R1743 回填豁免（css-flexbox-1 §4.1 flexed main size）。
//!
//! 根因：R1743 parent height backfill（shift_siblings_after_ifc_grow 末尾 max-bottom 回填）
//! 的 gate 按显示谱系放行 flex item（display:block），把 flex 算法已收缩定尺的 column
//! item 主轴高重新撑回子内容底（flex-minimum-height-flex-items-027：item 100px 被回填到
//! 子底 200px）。修复 = 父为 column flex ∧ 子 is_flex_grid_item ∧ 子指定 min-height 非
//! auto 时豁免 parent_backfill 分支（css-flexbox-1 §4.1：flexed main size 即 definite
//! used size，内容溢出可见但不回撑 item）。min-height:auto 的 item 保留回填——taffy
//! automatic min 测量回 transferred size 而非 content size suggestion
//!（flex-aspect-ratio-026），回填是该短板的现行补偿位。
//! kill-switch = `ZW_FLEX_COL_ITEM_NO_HEIGHT_BACKFILL=0`（default-on）。
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

/// 驱动腿（flex-minimum-height-flex-items-027 形）：column flex 容器 100px + item
/// `min-height:0` + 子 200px → item 主轴高保持 flex 收缩定尺 100px，不被 max-bottom
/// 回填到 200px。
#[test]
fn r5330_col_flex_item_min0_stays_flexed_height() {
    let html = r#"<html><body><div style="display:flex;flex-direction:column;height:100px;width:100px;">
  <div id="item" style="min-height:0;background:green;">
    <div style="height:200px;"></div>
  </div>
</div></body></html>"#;
    let (doc, root, _) = compute(html);
    let item = find_id(&root, &doc, "item").expect("flex item");
    assert!(
        (item.height - 100.0).abs() < 1.0,
        "column flex item（min-height:0）h={:.1} 应保持 flex 定尺 100px，不被 R1743 回填撑到子底 200px（css-flexbox-1 §4.1）",
        item.height
    );
}

/// 守卫腿（R1743 语义存活）：block 流域回填不受本豁免影响——`<div>a<br>b</div>` 直接
/// 作 body 子时 body h ≈ div h（r1743_body_height_repro 同锚，钉死豁免未误伤普通块流）。
#[test]
fn r5330_r1743_block_backfill_survives() {
    let (doc, root, _) = compute(r#"<html><body><div>a<br>b</div></body></html>"#);
    let (_, body) = find_tag(&doc, &root, "body").expect("body");
    let (_, div) = find_tag(&doc, &root, "div").expect("div");
    assert!(
        body.height >= div.height - 1.0,
        "body h={:.1} 应 ≈ div h={:.1}（R1743 块流域回填不受 column flex 豁免影响）",
        body.height,
        div.height
    );
}

/// 守卫腿（row 容器不在豁免域）：row flex item 的 height 是 cross 轴（内容决定型），
/// max-bottom 回填仍是合法 R1743 修复位——容器 auto 高时 item 子内容决定 item 高。
#[test]
fn r5330_row_flex_item_cross_height_backfill_survives() {
    let html = r#"<html><body><div style="display:flex;width:100px;">
  <div id="item"><div style="height:200px;"></div></div>
</div></body></html>"#;
    let (doc, root, _) = compute(html);
    let item = find_id(&root, &doc, "item").expect("row flex item");
    assert!(
        (item.height - 200.0).abs() < 1.0,
        "row flex item h={:.1} 应 ≈ 200（height 为 cross 轴内容决定型，回填位不受豁免）",
        item.height
    );
}

/// 守卫腿（min-height:auto 保留回填）：同 027 结构但 item 无指定 min-height——automatic
/// minimum 生效，内容 size suggestion 决定 item 高（taffy automatic min 短板由回填补偿，
/// flex-aspect-ratio-026 谱系），item 高应 ≈ 200。
#[test]
fn r5330_col_flex_item_auto_min_backfill_survives() {
    let html = r#"<html><body><div style="display:flex;flex-direction:column;height:100px;width:100px;">
  <div id="item">
    <div style="height:200px;"></div>
  </div>
</div></body></html>"#;
    let (doc, root, _) = compute(html);
    let item = find_id(&root, &doc, "item").expect("flex item");
    assert!(
        (item.height - 200.0).abs() < 1.0,
        "column flex item（min-height:auto）h={:.1} 应 ≈ 200（automatic minimum = content size suggestion，回填补偿位保留）",
        item.height
    );
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
