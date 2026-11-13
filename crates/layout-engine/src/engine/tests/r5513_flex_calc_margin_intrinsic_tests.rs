//! R5513（css-sizing-3 #intrinsic-sizes + csswg #823066，R4136 口径入 flex 臂）：flex
//! item margin 的 Calc(P%±Npx) 形固有测量只取 px 部分。convert 层把 calc margin 剥成
//! Percent(P%)（px 偏移由 R4136 布局后补），post-taffy margin_* 在 intrinsic 语境的
//! % 分量解析为 0——`flex_row_intrinsic_width`/`flex_column_intrinsic_width` 裸读
//! margin_* 丢 px 部分，容器 min-content/max-content 塌 0（calc-margins-flex：绿面
//! 整缺 ✗2.08%）。修复 = 两臂经 [`intrinsic_margin_contribution`] 取 margin（calc 取
//! px 分量、普通 margin 照用 post-taffy 值）。

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

/// row 容器 `width:min-content`：空 item `margin-left: calc(10% + 100px)` 的 px 分量
/// 入账——容器 intrinsic = 100（WPT calc-margins-flex 驱动案，ref 绿方块 100×100）。
#[test]
fn r5513_flex_row_calc_margin_intrinsic() {
    let html = r#"<html><body style="margin:0">
<div id=c style="display: flex; width: min-content; height: 100px; background: green;">
  <div style="margin-left: calc(10% + 100px);"></div>
</div>
</body></html>"#;
    let (doc, root, _styles) = compute(html);
    let c = find_id(&root, &doc, "c").expect("flex 容器存在");
    assert!(
        (c.width - 100.0).abs() <= 0.5,
        "row 容器 min-content 应含 calc margin px 分量 100，实际 {}",
        c.width
    );
    assert!(
        (c.height - 100.0).abs() <= 0.5,
        "容器高度应 100（显式 height），实际 {}",
        c.height
    );
}

/// column 容器同口径：cross 轴 intrinsic 取最宽 item outer 贡献，calc margin px 分量
/// 不丢（空 item + margin-left calc → 容器宽 100）。
#[test]
fn r5513_flex_column_calc_margin_intrinsic() {
    let html = r#"<html><body style="margin:0">
<div id=c style="display: flex; flex-direction: column; width: min-content; height: 100px; background: green;">
  <div style="margin-left: calc(10% + 100px);"></div>
</div>
</body></html>"#;
    let (doc, root, _styles) = compute(html);
    let c = find_id(&root, &doc, "c").expect("flex 容器存在");
    assert!(
        (c.width - 100.0).abs() <= 0.5,
        "column 容器 cross intrinsic 应含 calc margin px 分量 100，实际 {}",
        c.width
    );
}

/// 普通 px margin 照用 post-taffy 值（回归守卫）：R4136 臂只对 Calc 形切换口径，
/// `margin-left: 40px` 的 intrinsic 贡献不变（容器 min-content = 40）。
#[test]
fn r5513_flex_plain_margin_intrinsic_unchanged() {
    let html = r#"<html><body style="margin:0">
<div id=c style="display: flex; width: min-content; height: 100px; background: green;">
  <div style="margin-left: 40px;"></div>
</div>
</body></html>"#;
    let (doc, root, _styles) = compute(html);
    let c = find_id(&root, &doc, "c").expect("flex 容器存在");
    assert!(
        (c.width - 40.0).abs() <= 0.5,
        "普通 px margin intrinsic 贡献应 40 不变，实际 {}",
        c.width
    );
}
