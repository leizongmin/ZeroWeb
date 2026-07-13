//! R1404 回归测试：aspect-ratio 派生的 flex 容器 definite cross-size。
//! 驱动案 css-flexbox/flex-aspect-ratio-cross-size-001：`.flex{display:flex; width:400px;
//! aspect-ratio:2/1}` 派生 definite height 200，内 `<img>`(1×1) 应 cross-stretch 到 200 +
//! transferred main 200 → 200×200。旧实现 img=1×1（taffy item 布局时容器 cross 仍 indefinite
//! 不 stretch，aspect-ratio 派生 height 在 content layout 后才有；R1371 仅覆盖 abspos 容器）。
//! kill-switch `ZW_ASPECT_RATIO_FLEX_STRETCH=0` 关闭本扩展时测试应失败（load-bearing）。

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
fn r1404_aspect_ratio_flex_stretches_replaced_item() {
    let html = r#"<html><body>
<div style="display:flex; width:400px; aspect-ratio:2/1;">
  <img id="img" style="display:block" src="x.png">
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_element_by_id("img").expect("img");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (1.0f32, 1.0f32));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, HashMap::new());
    let img = find(&result.root, img_id).unwrap();
    println!(
        "R1404 img w={:.0} h={:.0}（期望 200×200：cross-stretch + transferred）",
        img.width, img.height
    );
    assert!(
        (img.width - 200.0).abs() < 3.0 && (img.height - 200.0).abs() < 3.0,
        "R1404: aspect-ratio 派生 definite cross 的 flex 容器内 img 应 200×200（stretch+transfer），\
         got w={:.0} h={:.0}（若 1×1 = aspect-ratio definite cross 未触发 item stretch）",
        img.width,
        img.height
    );
}
