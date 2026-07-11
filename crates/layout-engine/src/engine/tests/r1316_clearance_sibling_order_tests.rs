//! R1316：clearance-push-following-sibling DOM 顺序修复回归测试。
//!
//! 谱系：WPT `css/CSS2/floats-clear/margin-collapse-clear-012.xht`（§8.3.1）。
//! 结构：非 BFC 父容器（border-top 封顶）含 float + clear（空块）+ 空块兄弟。
//! clearance 把 #clear-left 推到 float 底边之下；#following-sibling 须定位在
//! #clear-left **之后**（DOM 顺序），而非 taffy 给出的陈旧位置（其前）。
//!
//! R1316 两处缺陷：
//!  - defect ①（line 683 clamp）：clearance 下推量被 `.max(0.0)` 丢失，后续非 clear
//!    兄弟不重定位 → 出现在 cleared 元素之前（DOM 顺序违反）。
//!  - defect ②（line 704 empty-block 分支）：cleared 空块走 collapse-through 分支不
//!    推进 flow_bottom，后续兄弟用陈旧 flow_bottom。
//!
//! 本测试为 defect ①+② 协同修复的 load-bearing 断言：following-sibling.y 必须不小于
//! clear-left.y（两者均空块，rel_y 比较以排除父容器绝对偏移）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::{ClearValue, FloatValue};
use zero_style_system::StyleSystem;

/// 定位含 float 子 + clear 子的块级父容器（模拟 012 的 #parent-lime）。
fn find_clear_parent(root: &LayoutBox) -> Option<&LayoutBox> {
    let has_float = root.children.iter().any(|c| !matches!(c.float, FloatValue::None));
    let has_clear = root.children.iter().any(|c| {
        !matches!(
            c.clear,
            ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
        )
    });
    if root.is_block_level && has_float && has_clear {
        return Some(root);
    }
    for child in &root.children {
        if let Some(f) = find_clear_parent(child) {
            return Some(f);
        }
    }
    None
}

/// 012 结构：clearance 推 #clear-left 过 float；#following-sibling 须在 #clear-left 之后。
///
/// 旧实现（defect ①+②）：#following-sibling 沿用 taffy 陈旧位置，出现在 #clear-left
/// **之前**（dump：following @132.6 < clear @152.6 = DOM 顺序违反）。
#[test]
fn test_clearance_preserves_following_sibling_dom_order() {
    let html = r#"<html><body style="margin:0">
      <div id="parent" style="border-top:1px solid black;width:50%">
        <div id="float-left" style="float:left;height:100px;width:100px"></div>
        <div id="clear-left" style="clear:left;margin-top:40px;margin-bottom:80px"></div>
        <div id="following-sibling" style="margin-bottom:140px"></div>
      </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let parent = find_clear_parent(&result.root).expect("should find #parent (float + clear children)");

    let float_left = parent
        .children
        .iter()
        .find(|c| !matches!(c.float, FloatValue::None))
        .expect("should find float-left child");
    let clear_left = parent
        .children
        .iter()
        .find(|c| {
            !matches!(
                c.clear,
                ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
            )
        })
        .expect("should find clear-left child");
    // following-sibling = 第 4 个子（float / clear / following）；非 float 非 clear。
    let following = parent
        .children
        .iter()
        .rfind(|c| {
            matches!(c.float, FloatValue::None)
                && matches!(
                    c.clear,
                    ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
                )
                && c.is_block_level
        })
        .expect("should find following-sibling child");

    // 前置：clearance 确实把 #clear-left 推到 float 底边之下（clear 生效）。
    let float_bottom = float_left.y + float_left.height;
    assert!(
        clear_left.y + 0.5 >= float_bottom,
        "clear-left must be pushed past float bottom (clearance applied); clear_left.y={} float_bottom={}",
        clear_left.y,
        float_bottom
    );

    // ★ R1316 load-bearing：DOM 顺序 —— following-sibling 必须不在 clear-left 之前。
    // 两者均为空块（无视觉高度），比较 border-box 顶边（content-relative 排除父偏移）。
    let clear_rel_y = clear_left.y - parent.content_y;
    let following_rel_y = following.y - parent.content_y;
    assert!(
        following_rel_y + 0.5 >= clear_rel_y,
        "following-sibling must not precede clear-left (DOM order); \
         following_rel_y={} clear_rel_y={} (defect ①+② regression: following before cleared)",
        following_rel_y,
        clear_rel_y
    );
}

/// defect ② 单独 load-bearing：cleared 空块（h=0）推进 flow_bottom。
/// 旧实现：cleared 空块走 is_empty_block 分支不推进 → flow_bottom 停在 float 前。
/// 此处用一个非空 following 块验证它落在 cleared 空块之后（而非与 float 重叠）。
#[test]
fn test_cleared_empty_block_advances_flow_for_next_solid_sibling() {
    let html = r#"<html><body style="margin:0">
      <div id="parent" style="border-top:1px solid black;width:50%">
        <div id="float-left" style="float:left;height:100px;width:100px"></div>
        <div id="clear-left" style="clear:left;margin-top:40px;margin-bottom:80px"></div>
        <div id="solid-sibling" style="height:50px"></div>
      </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let parent = find_clear_parent(&result.root).expect("should find #parent");
    let clear_left = parent
        .children
        .iter()
        .find(|c| {
            !matches!(
                c.clear,
                ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
            )
        })
        .expect("should find clear-left");
    // solid-sibling = 非 float 非 clear 且有高度（h=50）。
    let solid = parent
        .children
        .iter()
        .find(|c| {
            matches!(c.float, FloatValue::None)
                && matches!(
                    c.clear,
                    ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
                )
                && (c.height - 50.0).abs() < 1.0
        })
        .expect("should find solid-sibling (h=50)");

    // solid-sibling 顶边须不低于 clear-left 的 cleared 位置（其 margin-bottom 折叠后）。
    let clear_rel_y = clear_left.y - parent.content_y;
    let solid_rel_y = solid.y - parent.content_y;
    assert!(
        solid_rel_y + 0.5 >= clear_rel_y,
        "solid sibling after cleared empty block must not overlap back to cleared position; \
         solid_rel_y={} clear_rel_y={} (defect ②: cleared empty block did not advance flow_bottom)",
        solid_rel_y,
        clear_rel_y
    );
}
