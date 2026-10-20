//! R5205（css-overflow-4 #line-clamp + CSS2 §8.3.1/§10.6.3）：跨块 clamp 收缩的
//! 后续兄弟位移 + 祖先缩高 + Count(n) 末子 adjoining bottom margin 吸收——
//! `apply_cross_block_line_clamp` 只收缩容器自身，布局期 taffy 定位的 <p> y 与
//! 祖先高仍是收缩前值（023：Following content 下移 2 隐藏行高 64px）；末可见子
//! bottom margin 在容器有 bottom 框边时不外塌（框边阻断折叠组），吸入 extent
//!（023 盒高 148 非 144）。025：吸收用 declared mb，塌穿链上已隐藏子的 margin
//! 不计（assert「margins of hidden boxes are ignored」）。
//! driving: css-overflow line-clamp-023/025.tentative。

use crate::engine::LayoutEngine;
use zero_style_system::StyleSystem;

const CSS: &str = ".clamp { line-clamp: 4; font: 16px / 32px serif; padding: 4px; border: 2px solid black; } \
.inner { margin: 4px; } .inner .inner { white-space: pre; } \
.clamped-margin { margin: 20px; height: 20px; }";

fn layout(body: &str) -> crate::types::LayoutResult {
    let html = format!("<html><body>{body}<p>Following content.</p></body></html>");
    let doc = zero_dom::parse_html(&html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(CSS);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 023：body 下首个 div = clamp 容器，第二个 = <p>。
fn clamp_and_p(result: &crate::types::LayoutResult) -> (&crate::types::LayoutBox, &crate::types::LayoutBox) {
    let body = &result.root.children[1];
    let clamp = body.children.iter().find(|c| c.height > 0.5).expect("clamp 容器盒");
    let p = body.children.last().expect("后续 <p> 盒");
    (clamp, p)
}

/// 023 锚点：盒高含末子 collapsed bottom margin（148 非 144），<p> 上移 2 隐藏
/// 行高（y=164 非 228），body 高随净缩量收缩。
#[test]
fn r5205_clamp_shrink_shifts_following_siblings_and_absorbs_margin() {
    let result = layout(
        "<div class=\"clamp\"><div class=\"inner\"><div class=\"inner\">Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6</div></div></div>",
    );
    let (clamp, p) = clamp_and_p(&result);
    assert!(
        (clamp.height - 148.0).abs() < 0.5,
        "盒高应 = 4 行×32 + 末子 collapsed mb 4 + padding 8 + border 4 = 148，实得 {}",
        clamp.height
    );
    assert!(
        (clamp.content_height - 136.0).abs() < 0.5,
        "内容高应含吸收的 mb（136），实得 {}",
        clamp.content_height
    );
    assert!(
        (p.y - 164.0).abs() < 0.5,
        "<p> 应上移 2 隐藏行高到 y=164（布局期 228），实得 {}",
        p.y
    );
}

/// 025 锚点：塌穿链上已隐藏子的 margin（.clamped-margin margin 20）不得经
/// collapsed-effective mb 计入吸收——盒高同 023（148），<p> y=164。
#[test]
fn r5205_hidden_boxes_margins_ignored_in_absorption() {
    let result = layout(
        "<div class=\"clamp\"><div class=\"inner\"><div class=\"inner\">Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6</div><div class=\"clamped-margin\"></div></div><div class=\"clamped-margin\"></div></div>",
    );
    let (clamp, p) = clamp_and_p(&result);
    assert!(
        (clamp.height - 148.0).abs() < 0.5,
        "隐藏子的 margin 不应计入盒高（148 非 164/308），实得 {}",
        clamp.height
    );
    assert!((p.y - 164.0).abs() < 0.5, "<p> y 应 164，实得 {}", p.y);
    // 两个 .clamped-margin 均在 clamp 点后，整体隐藏。
    let hidden = clamp.children[0]
        .children
        .iter()
        .filter(|c| c.line_clamp_hidden)
        .count();
    assert_eq!(hidden, 1, "内层 clamped-margin 应隐藏");
    assert!(clamp.children[1].line_clamp_hidden, "外层 clamped-margin 应隐藏");
}
