//! R1423 回归测试：balance 模式 multicol 容器须填充 `text_node_is_ahem`（等字体度量）。
//!
//! 驱动案 css-multicol/multicol-columns-001（`columns:100px 6`，column-fill 默认 balance，
//! height:auto，纯 inline 文本）：ZW 此前把此类容器渲染为扁平全宽块（仅 42% 文本）。
//! 真因：`compute_final_inline_layouts` 对 multicol 容器早返回，跳过
//! `store_font_sizes_from_ifc`，致 `text_node_is_ahem` 空 → paint 侧重跑 IFC 用空 styles，
//! is_ahem=false（Ahem 'x' 估 11px 应 20px）→ 列宽下少换行（22 行应 44）→ 列欠填。
//!
//! R1423 修复：`store_inline_multicol_columns` 对 balance 模式也计算列宽 IFC 并调用
//! `store_font_sizes_from_ifc` 填充度量（不存列分布，paint 对 balance 用自己的
//! multicol_info 重跑）。修复后 multicol-columns-001 9.99%→5.50%，paint IFC 行数 22→44。
//! 本测试 load-bearing：无修复时 balance multicol 的 `text_node_is_ahem` 为空。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

#[test]
fn r1423_balance_multicol_populates_text_node_is_ahem() {
    // column-count:3 + column-gap:0 + Ahem 文本 = balance 模式（默认 column-fill）inline-only multicol。
    let html = r#"<html><body>
<div id="mc" style="column-count:3; column-gap:0; width:300px; font:20px/1 Ahem;">
x xx xxx xxxx xxxxx
x xx xxx xxxx xxxxx
x xx xxx xxxx xxxxx
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mc_id = doc.get_element_by_id("mc").expect("multicol div #mc");

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, HashMap::new(), HashMap::new());
    let mc = find(&result.root, mc_id).unwrap();

    // 修复后：balance multicol 容器的 text_node_is_ahem 必须非空（供 paint 重跑 IFC 用）。
    // 无修复时：multicol 早返回跳过 store_font_sizes_from_ifc → text_node_is_ahem 空。
    assert!(
        !mc.text_node_is_ahem.is_empty(),
        "R1423: balance multicol 容器应填充 text_node_is_ahem（供 paint 重跑 IFC 获得 is_ahem），\
         got empty（修复前 bug：multicol 早返回跳过 store_font_sizes_from_ifc → paint IFC \
         is_ahem=false → 列宽下少换行 → 列欠填）"
    );
    // 填充的 is_ahem 值应为 true（容器 font-family=Ahem）。
    let any_ahem = mc.text_node_is_ahem.values().any(|&v| v);
    assert!(
        any_ahem,
        "R1423: balance multicol 容器的 text_node_is_ahem 应含 true（Ahem 文本），\
         got all false"
    );
}
