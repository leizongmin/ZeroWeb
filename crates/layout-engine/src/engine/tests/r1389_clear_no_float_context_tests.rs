//! R1389：无活跃 float context 时 clear 子的 spurious clearance 覆盖测试。
//!
//! 谱系：WPT `css/CSS2/floats-clear/no-clearance-due-to-large-margin.html`（CSS §9.5.2）。
//! 结构：float(left,100×100) + wrapper(padding-top:1px) > red(无 border/padding-top) >
//! clear(clear:left, margin-top:150px, `<br>`)。
//! red 已在 float 下方（float bottom 100 < red content_abs_y），故 clear 子**无浮动可清除**。
//! 但 taffy 0.12 仍基于同 BFC 的祖先 float 对 clear 误 apply clearance，把 clear 推到 red 底部
//! 并把 red 膨胀到 ~83px（应 ≈ clear border-box 20px 或更小，使 red 不显红）。
//!
//! R1389 fix（`adjust_float_positions` 的 `else if` 分支）：对 has_active_float_context=false、
//! 无 border-top/padding-top、唯一 in-flow block 子为 clear 元素、auto-height 的容器，将 clear 子
//! 重定位到容器 content top（其 mt 折叠穿出），并按 in-flow 子 border-box 收缩容器。
//!
//! 本测试为 load-bearing 断言：red 高度须不被 spurious clearance 膨胀（远小于 margin-top:150
//! 带来的 taffy 堆叠值），且 clear 子须位于 red 顶部（非被推到底部）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::ClearValue;

/// 查找「唯一 in-flow block 子为 clear 元素」的块级容器（模拟 WPT 的 #red）。
fn find_clear_only_parent(root: &LayoutBox) -> Option<&LayoutBox> {
    let in_flow_blocks: Vec<&LayoutBox> = root
        .children
        .iter()
        .filter(|c| !c.is_absolute && !c.is_fixed && c.is_block_level)
        .collect();
    if in_flow_blocks.len() == 1
        && !matches!(
            in_flow_blocks[0].clear,
            ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
        )
    {
        return Some(root);
    }
    for child in &root.children {
        if let Some(f) = find_clear_only_parent(child) {
            return Some(f);
        }
    }
    None
}

/// no-clearance-due-to-large-margin：red 不被 spurious clearance 膨胀。
///
/// 旧实现（taffy 误 apply clearance）：red 高度被膨胀到 ~83px（clear 被推到底部），
/// 显红；clear 未落在 red 顶部。
#[test]
fn test_clear_no_float_context_does_not_inflate_parent() {
    let html = r#"<html><body style="margin:0">
      <div id="float" style="float:left; width:100px; height:100px; background:green"></div>
      <div id="wrapper" style="padding-top:1px;">
        <div id="red" style="background:red;">
          <div id="clear" style="clear:left; background:white; margin-top:150px;"><br></div>
        </div>
      </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let red = find_clear_only_parent(&result.root).expect("应找到 #red（唯一 in-flow block 子为 clear 元素）");
    let clear = red
        .children
        .iter()
        .find(|c| {
            !matches!(
                c.clear,
                ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
            )
        })
        .expect("应找到 #clear 子");

    // R1389：red 高度须不被 spurious clearance 膨胀。clear border-box ≈ br 行高（~20px）。
    // taffy 误 apply clearance 会把 red 膨胀到 ~83px；修复后 red ≤ clear border-box + 容差。
    let clear_border_box = clear.height;
    assert!(
        red.height <= clear_border_box + 1.0,
        "red.height={} 须 ≤ clear border-box {}+1（不应被 spurious clearance 膨胀）",
        red.height,
        clear_border_box
    );

    // clear 子须位于 red 顶部（相对 red border-box 的 y ≈ content_y_offset = 0），
    // 而非被 taffy clearance 推到 red 底部。
    assert!(
        clear.y <= 1.0,
        "clear.y={} 须 ≈ 0（位于 red 顶部，非被 spurious clearance 推到底部）",
        clear.y
    );
}
