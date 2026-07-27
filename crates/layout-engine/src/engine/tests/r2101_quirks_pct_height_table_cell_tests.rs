//! R2101：quirks mode 百分比高度 quirk **不适用**于 table-cell 的后代。
//! driving case：`percentages-grandchildren-quirks-mode-001.html`（css-tables，11.44%→0.00%）。
//! 结构（quirks mode，无 DOCTYPE）：
//!   `display:table-cell; height:100px` > `div width:100px`（auto 高）> `div height:100%`（红）
//! CSS Quirks §percentage-height：百分比高度 quirk（不明确 CB 按 ICB/viewport 解析）**不适用**
//! 于 table-cell 后代——后代 height:% 须 compute-to-auto（standards 行为），而非解析到 viewport。
//! 旧实现 R2016 quirks 分支无条件按 viewport 解析 → 红格被撑到 viewport 高（覆盖绿格）→ 非「绿方」。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeKind;
use zero_style_system::StyleSystem;

fn layout(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result.root)
}

/// 找最深的 `div` 元素盒，返回其 used height。
/// driving case 中 height:100% 红格是深度最大的 div（table-cell > 宽100格 > 红格）。
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

/// R2101 主驱：quirks mode 下 table-cell 后代的 `height:100%`（auto 父，不明确 CB）
/// 应 compute-to-auto（~0），而非按 viewport（600px）解析。
#[test]
fn r2101_quirks_pct_height_in_table_cell_descendant_is_auto() {
    // 无 DOCTYPE → quirks mode（html5ever 按 DOCTYPE 缺失判定 Quirks）。
    let html = r#"<div style="display:table-cell; height:100px; background:green;">
  <div style="width:100px;">
    <div style="height:100%; background:red;"></div>
  </div>
</div>"#;
    let (doc, root) = layout(html);
    let h = deepest_div_height(&doc, &root).expect("target div not found");
    // 修复前：红格 height:100% 按 viewport 解析 ≈ 600px（覆盖绿格，非「绿方」）。
    // 修复后：table-cell 后代 compute-to-auto ≈ 0（红格塌缩，露出绿格）。
    assert!(
        h < 50.0,
        "quirks-mode height:100% inside table-cell must compute to auto (~0), not viewport (600); got h={h}"
    );
}

/// R2101 非回归守：table-cell **之外**的 quirks-mode `height:%`（不明确 CB）仍按 viewport
/// 解析（R2016 legacy「百分比高度生效」行为保持）。此案不在 table-cell 子树内，guard 不触发。
/// （非 table-cell 案的保持由 css-tables A/B 全失败集零新增 + grid/float quirks 测试 code-path
/// 不触及本 guard 共同经验证；此处再加一条单元级守。）
#[test]
fn r2101_quirks_pct_height_outside_table_cell_still_resolves_to_viewport() {
    // 无 DOCTYPE → quirks mode；内格 height:50% 父为 auto（不明确 CB），非 table-cell 后代。
    let html = r#"<div>
  <div style="height:50%; background:red;"></div>
</div>"#;
    let (doc, root) = layout(html);
    let h = deepest_div_height(&doc, &root).expect("target div not found");
    // R2016 quirks：50% × viewport(600) = 300px（容许 taffy/box-sizing 小幅偏差）。
    assert!(
        h > 150.0,
        "quirks-mode height:50% outside table-cell must still resolve against viewport (~300), not auto; got h={h}"
    );
}
