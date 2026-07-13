//! R1411 回归测试：column-flex **非 stretch** 替换 item 的 main（height）修正。
//! 驱动案 css-flexbox/flex-aspect-ratio-img-column-007：`.flex{display:flex;
//! flex-direction:column; width:200px; align-items:flex-start}` 内 `<img>`(20×50,
//! min-width 把 cross 解析到 40) 应 40×100（main = cross × ih/iw = 40×(50/20)=100）。
//! taffy 误用 flex-line cross（容器宽 200）推 main → 200/0.4=500（5× 过高）。本 pass
//! 在 LayoutBox 层按 item 自身 cross 修正 main。kill-switch `ZW_FLEX_COL_ASPECT_MAIN=0`
//! 关闭时测试应失败（load-bearing）。

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
fn r1411_column_flex_nonstretch_replaced_main_uses_item_cross() {
    let html = r#"<html><body>
<div style="display:flex; flex-direction:column; width:200px; align-items:flex-start;">
  <img id="img" style="min-width:40px;" src="x.png">
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_element_by_id("img").expect("img");
    let mut img_sizes = HashMap::new();
    // 固有 20×50（w/h=0.4）；min-width:40px 把 cross 解析到 40（> 固有 20）。
    img_sizes.insert(img_id, (20.0f32, 50.0f32));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, HashMap::new());
    let img = find(&result.root, img_id).unwrap();
    println!(
        "R1411 img w={:.0} h={:.0}（期望 40×100：cross=40, main=cross×ih/iw=100）",
        img.width, img.height
    );
    assert!(
        (img.width - 40.0).abs() < 3.0 && (img.height - 100.0).abs() < 3.0,
        "R1411: column-flex 非 stretch 替换 item 应 40×100（按 item 自身 cross 推 main），\
         got w={:.0} h={:.0}（h≈500 = taffy 误用容器 cross 200 推 main 的 bug）",
        img.width,
        img.height
    );
}
