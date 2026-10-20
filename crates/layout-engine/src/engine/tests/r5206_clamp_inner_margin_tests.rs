//! R5206（CSS2 §8.3.1/§10.6.3）：clamp 容器**内层**中间盒的 extent fold 补
//! 末可见 in-flow 子 declared bottom margin——中间盒有 bottom 框边时框边阻断
//! 折叠组，子的 adjoining mb 属本盒 extent（022 紫盒高 140 = 框 2 + 橙
//! margin 8 + 橙盒 130，修前漏橙 mb 得 136）。与 R5205 Fix C（容器尾）互为
//! 一层递进：容器尾吸收在 walk() 尾，内层吸收在 walk_children fold。
//! driving: css-overflow line-clamp-022/024.tentative。

use crate::engine::LayoutEngine;
use zero_style_system::StyleSystem;

const CSS: &str = ".clamp { line-clamp: 4; font: 16px / 32px serif; padding: 4px; border: 2px solid black; } \
.inner { margin: 4px; border: 1px solid black; } .inner .inner { white-space: pre; }";

fn layout(body: &str) -> crate::types::LayoutResult {
    layout_with(CSS, body)
}

fn layout_with(css: &str, body: &str) -> crate::types::LayoutResult {
    let html = format!("<html><body>{body}<p>Following content.</p></body></html>");
    let doc = zero_dom::parse_html(&html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 022：body 下首个 div = clamp 容器，第二个 = <p>。
fn clamp_and_p(result: &crate::types::LayoutResult) -> (&crate::types::LayoutBox, &crate::types::LayoutBox) {
    let body = &result.root.children[1];
    let clamp = body.children.iter().find(|c| c.height > 0.5).expect("clamp 容器盒");
    let p = body.children.last().expect("后续 <p> 盒");
    (clamp, p)
}

/// 022 锚点：有 bottom 框边的内层紫盒含橙子 declared mb（140 非 136），容器
/// 高 160 = 紫 140 + 紫 margin 8 + padding 8 + border 4，<p> y = 160 + p
/// margin-top 16 = 176。
#[test]
fn r5206_inner_box_absorbs_last_child_margin_with_bottom_border() {
    let result = layout(
        "<div class=\"clamp\"><div class=\"inner\"><div class=\"inner\">Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6</div></div></div>",
    );
    let (clamp, p) = clamp_and_p(&result);
    let purple = &clamp.children[0];
    assert!(
        (purple.height - 140.0).abs() < 0.5,
        "紫盒高应 = 橙 margin 8 + 橙盒 130 + 框 2 = 140（漏橙 mb 则 136），实得 {}",
        purple.height
    );
    assert!(
        (clamp.height - 160.0).abs() < 0.5,
        "容器高应 = 紫 140 + 紫 margin 8 + padding 8 + border 4 = 160，实得 {}",
        clamp.height
    );
    assert!((p.y - 176.0).abs() < 0.5, "<p> y 应 = 160 + 16 = 176，实得 {}", p.y);
}

/// 023 判例回归：内层无边框时 margin 塌穿排除，fold 不吸收——无边框臂走
/// 原 extent，防本修复误伤 023 族。
#[test]
fn r5206_borderless_inner_keeps_plain_extent() {
    let result = layout_with(
        ".clamp { line-clamp: 4; font: 16px / 32px serif; padding: 4px; border: 2px solid black; } \
         .inner { margin: 4px; } .inner .inner { white-space: pre; }",
        "<div class=\"clamp\"><div class=\"inner\"><div class=\"inner\">Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6</div></div></div>",
    );
    let (clamp, _) = clamp_and_p(&result);
    // R5205 023 判例值：容器 148 = 4 行×32 + 紫 mb 4 + padding 8 + border 4。
    assert!(
        (clamp.height - 148.0).abs() < 0.5,
        "无边框内层不应改变 023 容器高 148，实得 {}",
        clamp.height
    );
}
