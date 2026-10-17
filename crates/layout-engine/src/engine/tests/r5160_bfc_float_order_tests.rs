//! R5160（CSS 2.1 §9.5）：BFC 浮动排斥只消费**文档顺序在前**的 float——float 顶边
//! 不得高于其假设流位（§9.5.1 规则 2），后序 float 的正确位恒在前序块之下，对更早
//! 的块级 BFC 不构成避让约束。旧实现按 Phase 1 临时 y 做垂直重叠判定：后置 float
//! 群被临时放在前序 BFC 块的 y 带内（Phase 2 才钳到真实流位），块被反向缩进
//!（contain-size-block-002 block2 x=68/w=716，chromium 满宽 x=0 实证）。
//! kill-switch `ZW_BFC_FLOAT_ORDER_GATE=0`。

use crate::engine::LayoutEngine;
use zero_css_parser::Parser;
use zero_style_system::StyleSystem;

/// 布局并返回 body 直接子层各块 (x, y, w, h)（按出现顺序，float 与块混排）。
fn layout_blocks(html: &str, css: &str) -> Vec<(f32, f32, f32, f32)> {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    result.root.children[1]
        .children
        .iter()
        .map(|c| (c.x, c.y, c.width, c.height))
        .collect()
}

/// contain-size-block-002 前七块实证形（渐进截断 n=7 复现）：第三块
///（overflow:scroll + contain:size + height:60，BFC）被后置 float 群
///（floatLBasic 4px + floatLWidth 64px，DOM 顺序在后、最终流位在块之下）
/// 依 Phase 1 临时 y 误缩进 x=68/w=716（chromium 满宽 x=0）。修复后块 x=0。
#[test]
fn test_bfc_block_not_shifted_by_later_floats() {
    let html = r#"<html><body style="margin:0">
<div style="contain:size;overflow:scroll;border:2px solid green"><div style="height:100px;width:100px">inner</div></div>
<br>
<div style="contain:size;overflow:scroll;min-height:60px;border:2px solid green"><div style="height:100px;width:100px">inner</div></div>
<br>
<div style="contain:size;overflow:scroll;height:60px;border:2px solid green"><div style="height:100px;width:100px">inner</div></div>
<br>
<div style="contain:size;overflow:scroll;max-width:60px;border:2px solid green"><div style="height:100px;width:100px">inner</div></div>
<br>
<div style="contain:size;overflow:scroll;width:60px;border:2px solid green"><div style="height:100px;width:100px">inner</div></div>
<br>
<div style="contain:size;overflow:scroll;float:left;border:2px solid green"><div style="height:100px;width:100px">inner</div></div>
<br>
<div style="contain:size;overflow:scroll;float:left;width:60px;border:2px solid green"><div style="height:100px;width:100px">inner</div></div>
</body></html>"#;
    let blocks = layout_blocks(html, "");
    // 第三个 border 块（height:60）须保持流位 x=0 满宽。
    assert!(
        (blocks[4].0 - 0.0).abs() < 0.5,
        "third bordered block must stay at x=0, got {} (all: {blocks:?})",
        blocks[4].0
    );
}

/// kill-switch OFF 反证基座：`ZW_BFC_FLOAT_ORDER_GATE=0` 时旧行为复活（块被后置
/// float 的 Phase 1 临时 y 误缩进）。断言在 OFF 运行下应失败——由 OFF scoped 跑
/// 外部反证（OnceLock 无进程内翻转），此处保持 ON 语义正向断言。
#[test]
fn test_bfc_block_avoids_earlier_float_still_works() {
    // 前置 float（文档顺序在前）+ 后随 BFC 块：块须避开 float（既有语义保持）。
    let html = r#"<html><body style="margin:0">
<div style="float:left;width:100px;height:200px;background:gray"></div>
<div style="overflow:hidden;height:60px;border:2px solid blue"></div>
</body></html>"#;
    let blocks = layout_blocks(html, "");
    assert!(
        blocks[1].0 > 50.0,
        "BFC block after a left float must avoid it (x > 50), got {}",
        blocks[1].0
    );
}
