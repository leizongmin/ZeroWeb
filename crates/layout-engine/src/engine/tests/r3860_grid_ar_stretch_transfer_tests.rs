//! R3860 回归测试：grid item「显式 stretch 轴 → track 尺寸钳制 + ratio 传递」
//! （css-grid §6.6 + css-sizing-4 §aspect-ratio stretch-preferred）。
//!
//! taffy 0.12 grid 对 aspect_ratio item 按 ratio（cross=列轨）解 main 后被 stretch
//! 拉伸时不回传 ratio——grid-028（rows 100/cols 200 + align-self:stretch）item 200×200
//! 溢出 100px row；chromium = 100×100（block stretch 钳 height，inline 经 ratio 传递）。
//! 对称案 grid-030（justify-self:stretch → width=col、height 反向传递）。
//! kill-switch `ZW_AR_GRID_STRETCH=0` 关闭本 fix 时本测试应失败（load-bearing）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
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
fn r3860_grid_align_stretch_transfers_inline_via_ratio() {
    let html = r#"<html><body>
<div id="g" style="display:grid; grid-template: 100px / 200px">
  <div id="item" style="aspect-ratio: 1/1; align-self: stretch; background: green"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let item_id = doc.get_element_by_id("item").expect("item");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let item = find(&result.root, item_id).unwrap();
    assert!(
        (item.height - 100.0).abs() < 0.5 && (item.width - 100.0).abs() < 0.5,
        "R3860: block-stretch 应钳 height=row(100) 并经 ratio 传递 width=100（200x200 溢出为错），\
         got {}x{}",
        item.width,
        item.height
    );
}

#[test]
fn r3860_grid_justify_stretch_transfers_block_via_ratio() {
    let html = r#"<html><body>
<div id="g" style="display:grid; grid-template: 200px / 100px">
  <div id="item" style="aspect-ratio: 1/1; justify-self: stretch; background: green"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let item_id = doc.get_element_by_id("item").expect("item");
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let item = find(&result.root, item_id).unwrap();
    assert!(
        (item.width - 100.0).abs() < 0.5 && (item.height - 100.0).abs() < 0.5,
        "R3860: inline-stretch 应钳 width=col(100) 并经 ratio 反向传递 height=100，\
         got {}x{}",
        item.width,
        item.height
    );
}
