//! R2170：quirks mode 百分比高度 quirk **不适用**于 flex/grid 容器子树内的后代。
//! driving case：`flex-aspect-ratio-cross-size-002.html`（css-flexbox，20.79%→2.08%）。
//! 结构（quirks mode，无 DOCTYPE）：
//!   `container{width:200px}` > `outer{display:flex; aspect-ratio:4}` >
//!   `inner{display:flex; aspect-ratio:1}` > `wrapper{block}` > `box{height:100%}`
//! CSS Quirks §percentage-height：百分比高度 quirk（不明确 CB 按 ICB/viewport 解析）**不适用**
//! 于 flex/grid 子树——flex/grid 容器高度由 flex/grid 算法决定，非 quirks-definite。
//! 旧实现 R2016 quirks 分支的 `quirks_nearest_definite` 透传穿 flex 容器（flex item 跳过 gate
//! 不更新 my_definite）→ box `height:100%` 按 viewport(600) 解析 → box=600，ar=2→w=1200，
//! 整链 inflate 到 1200×600（应 200×50）。chromium quirks 实测：flex 子树内 compute-to-auto。
//! R2170 加 `inside_flex_grid` gate（同型 R2101 table-cell），kill-switch ZW_QUIRKS_PCT_FLEX_GATE。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::sync::Arc;
use zero_dom::NodeKind;
use zero_style_system::StyleSystem;

fn layout(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, Arc::try_unwrap(result.root).unwrap_or_else(|arc| (*arc).clone()))
}

/// 找最深的 `div` 元素盒，返回其 used height。
fn deepest_div_height(doc: &zero_dom::Document, root: &LayoutBox) -> Option<f32> {
    let mut best: Option<(i32, f32)> = None;
    let mut stack: Vec<(&LayoutBox, i32)> = vec![(root, 0)];
    while let Some((b, depth)) = stack.pop() {
        let is_div = b
            .node_id
            .and_then(|id| doc.get(id))
            .map(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name() == "div"))
            .unwrap_or(false);
        if is_div && best.is_none_or(|(bd, _)| depth > bd) {
            best = Some((depth, b.height));
        }
        for c in &b.children {
            stack.push((c, depth + 1));
        }
    }
    best.map(|(_, h)| h)
}

/// R2170 主驱（flex）：quirks mode 下 flex 容器子树内后代的 `height:100%`（auto 父链，不明确 CB）
/// 应 compute-to-auto（~0），而非按 viewport（600px）解析 inflate。
#[test]
fn r2170_quirks_pct_height_in_flex_subtree_is_auto() {
    // 无 DOCTYPE → quirks mode。flex 容器 > block > height:100% 后代。
    let html = r#"<div style="display:flex;">
  <div>
    <div style="height:100%; background:red;"></div>
  </div>
</div>"#;
    let (doc, root) = layout(html);
    let h = deepest_div_height(&doc, &root).expect("target div not found");
    // 修复前：height:100% 按 viewport 解析 ≈ 600px（inflate）。
    // 修复后：flex 子树内 compute-to-auto ≈ 0。
    assert!(
        h < 50.0,
        "quirks-mode height:100% inside flex subtree must compute to auto (~0), not viewport (600); got h={h}"
    );
}

/// R2170（grid 变体）：quirks mode 下 grid 容器子树内后代的 `height:100%` 同理 compute-to-auto。
#[test]
fn r2170_quirks_pct_height_in_grid_subtree_is_auto() {
    let html = r#"<div style="display:grid;">
  <div>
    <div style="height:100%; background:red;"></div>
  </div>
</div>"#;
    let (doc, root) = layout(html);
    let h = deepest_div_height(&doc, &root).expect("target div not found");
    assert!(
        h < 50.0,
        "quirks-mode height:100% inside grid subtree must compute to auto (~0), not viewport (600); got h={h}"
    );
}

/// R2170 非回归守：flex/grid **之外**的 quirks-mode `height:%`（不明确 CB，纯 block 上下文）
/// 仍按 viewport 解析（R2016 legacy「百分比高度生效」行为保持）。此案不在 flex/grid 子树内，
/// gate 不触发——确保 R2170 gate 作用域精确，未误伤 legacy block 上下文。
#[test]
fn r2170_quirks_pct_height_outside_flex_grid_still_resolves_to_viewport() {
    // 无 DOCTYPE → quirks mode；纯 block 上下文，height:50% 父为 auto（不明确 CB）。
    let html = r#"<div>
  <div style="height:50%; background:red;"></div>
</div>"#;
    let (doc, root) = layout(html);
    let h = deepest_div_height(&doc, &root).expect("target div not found");
    // R2016 quirks：50% × viewport(600) = 300px（容许小幅偏差）。
    assert!(
        h > 150.0,
        "quirks-mode height:50% outside flex/grid must still resolve against viewport (~300), not auto; got h={h}"
    );
}
