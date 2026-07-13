//! R1371 回归测试：abspos flex 容器内替换 item 的 cross-stretch + transferred-size。
//! 驱动案 css-flexbox/flex-abspos-inset-nested-002：`.inner-flex` = abspos + flex +
//! top/bottom（definite height 300）内 `<img>`(1×1) 应 stretch 到 cross 300 并按
//! aspect-ratio(1:1) transferred 到 main 300 → 300×300。旧实现 img=1×1（+2 兄弟案同）。

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
fn r1371_verify_abspos_flex_replaced_stretch() {
    let html = r#"<html><body>
<div style="display:flex">
  <div style="width:100%">
    <div id="inter" style="position:relative; height:300px">
      <div id="inner" style="display:flex; position:absolute; top:0; bottom:0">
        <img id="img" style="display:block" src="x.png">
      </div>
    </div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let img_id = doc.get_element_by_id("img").expect("img");
    let inner_id = doc.get_element_by_id("inner").expect("inner");
    let mut img_sizes = HashMap::new();
    img_sizes.insert(img_id, (1.0f32, 1.0f32));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute_with_img_sizes(&doc, &styles, img_sizes, HashMap::new());
    let inner = find(&result.root, inner_id).unwrap();
    let img = find(&result.root, img_id).unwrap();
    println!(
        "inner w={:.0} h={:.0} | img w={:.0} h={:.0}",
        inner.width, inner.height, img.width, img.height
    );
    assert!(
        (img.width - 300.0).abs() < 3.0 && (img.height - 300.0).abs() < 3.0,
        "R1371: img 应 300×300（stretch+transfer），got w={:.0} h={:.0}",
        img.width,
        img.height
    );
}
