//! R3769：跨块盒 line-clamp 预算恰好用尽时（clamp 点落在中间子首行前），该子
//! `line_clamp_cap = Some(0)` 且 `line_clamp_clamped = true`。
//!
//! paint 侧（zero-engine text.rs）按 `max >= 1` 守卫曾把 cap=0 当「不 clamp」，non-stored
//! 路径该子 glyph 全部照绘；hidden 兄弟盒在 paint_node 主路径漏检同样照绘。R3769 修复
//! paint 侧触发条件与 paint_node 的 hidden 检查；本文件锁定 layout 侧确实产生
//! cap=Some(0)（下游语义的前提）。driving：css-overflow-4 line-clamp。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

/// 深度优先查找第一个满足谓词的盒。
fn find_box<'a>(root: &'a LayoutBox, pred: &dyn Fn(&LayoutBox) -> bool) -> Option<&'a LayoutBox> {
    if pred(root) {
        return Some(root);
    }
    root.children.iter().find_map(|c| find_box(c, pred))
}

/// line-clamp:2 容器含 3 个各 1 行的 block 子：子1/子2 消耗完 2 行预算，
/// 子3 需 1 行 > remaining 0 → cap=Some(0) + clamped + 盒高收缩到 0。
#[test]
fn r3769_cross_block_third_child_gets_cap_zero() {
    let html = r#"<html><body style="margin:0">
<div style="width:80px; line-clamp:2; overflow:hidden; font:20px/1 serif">
<div>XXXX</div><div>YYYY</div><div>ZZZZ</div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(400.0, 400.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(400.0, 400.0);
    let result = engine.compute(&doc, &styles);

    // 子3 = 含 ZZZZ 文本的 div（布局树里文本不生成独立盒；3 个 div 子均含各自文本）。
    // 按序取容器的第 3 个 block 子。
    use zero_style_system::property::types::LineClampComputedValue as LCC;
    let is_clamp_container = |b: &LayoutBox| {
        b.node_id
            .is_some_and(|id| styles.get(&id).is_some_and(|s| matches!(s.line_clamp, LCC::Count(2))))
    };
    let container = find_box(&result.root, &is_clamp_container).expect("clamp container box");
    let blocks: Vec<_> = container.children.iter().filter(|c| c.node_id.is_some()).collect();
    assert!(blocks.len() >= 3, "前置：容器应含 3 个子 div，实得 {}", blocks.len());
    let third = blocks[2];
    assert_eq!(
        third.line_clamp_cap,
        Some(0),
        "R3769: 预算用尽后第三子应得到 cap=Some(0)"
    );
    assert!(third.line_clamp_clamped, "R3769: 第三子应标记 clamped");
    assert!(
        third.height < 0.5,
        "R3769: 第三子盒高应收缩到 0（0 行可见），实得 {}",
        third.height
    );
}
