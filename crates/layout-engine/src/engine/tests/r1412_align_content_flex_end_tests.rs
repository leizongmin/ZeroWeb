//! R1412 回归测试：align-content:flex-end 解析 + flex 多行 pack 到底部。
//! 根因：style-system align-content 解析器（apply_advanced.rs）未处理 flex-start/flex-end
//! 关键字（fall through → return false → 默认 Normal → taffy 默认 flex-start pack），
//! 致 align-content:flex-end 被当作 flex-start（lines 在顶不在底）。css-align-3：flex 容器
//! block 轴上 flex-start/flex-end 等价 start/end（horizontal-tb）。驱动 css-flexbox/
//! flex-align-content-end 簇。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn find_first(root: &LayoutBox, pred: impl Fn(&LayoutBox) -> bool) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if pred(b) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

#[test]
fn r1412_align_content_flex_end_packs_items_to_bottom() {
    // 容器高 300，2 个 120×60 item（2×120=240>200 → wrap 成 2 行）。
    // align-content:flex-end → 2 行应 pack 到底部（item.y 接近 300-行高），非顶部。
    let html = r#"<html><body>
<div style="display:flex; flex-wrap:wrap; align-content:flex-end; height:300px; width:200px;">
  <div id="a" style="height:60px; width:120px; flex:none;"></div>
  <div id="b" style="height:60px; width:120px; flex:none;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    // 第一个 item 的 y：flex-end 应在底部（>100），flex-start（旧 bug）在顶部（<60）。
    let item = find_first(&result.root, |b| b.node_id == doc.get_element_by_id("a")).unwrap();
    println!(
        "R1412 first flex item y={:.0}（flex-end 应 >100 近底部；旧 bug <60 近顶部）",
        item.y
    );
    assert!(
        item.y > 100.0,
        "R1412: align-content:flex-end 应把行 pack 到容器底部，first item y={:.0} 偏小（旧 bug：flex-end 未解析→默认 flex-start pack 顶部）",
        item.y
    );
}
