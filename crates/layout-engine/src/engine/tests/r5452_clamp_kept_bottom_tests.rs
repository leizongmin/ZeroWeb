//! R5452（css-overflow-4 §line-clamp）：own-IFC-capped 容器（R2431 已在 IFC 内裁至
//! n 行、非 pure-Ahem 不落 `inline_layout`）的跨块 walk 尾部 extent 下限 = 保留行底
//!（`clamp_kept_line_bottom` 镜像，compute_final_inline_layouts store-gate 前记录）。
//! inline 子盒底边是 strut 常数盒（自身行高 < 容器行盒高时底边在行底之上），仅按
//! 子盒底边重算高度会把容器缩穿保留行——block-ellipsis-032 runner 环境实证：div
//! taffy 高 24 被缩到 19.624（span.green 底边 = strut 常数 18.624 + y 1），ref 24，
//! 三盒全部失配。clamp 容器 auto 高度语义 = clamp 边界 = 保留行底。
//! driving: css-overflow block-ellipsis-032.html。

use crate::engine::LayoutEngine;
use zero_style_system::StyleSystem;

const CSS: &str = ".clamp { line-clamp: 1; font: 16px/48px serif; width: 96px; } \
.inner { line-height: 16px; }";

fn layout(body: &str) -> crate::types::LayoutResult {
    let html = format!("<html><body>{body}</body></html>");
    let doc = zero_dom::parse_html(&html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(CSS);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 032 形状（runner 环境 mono 29ch 的估计环境镜像）：容器行高 48 支撑行盒（strut），
/// inline span 自身行高 16——子盒底边（strut 常数盒）< 保留行底。cap 至 1 行后容器
/// 高度不得缩穿保留行底 48；span 底边前置断言保证形状真有 delta（否则 pin 空转）。
#[test]
fn r5452_kept_line_bottom_floors_shrunk_container_height() {
    let result = layout("<div class=\"clamp\"><span class=\"inner\">aa bb</span> cc dd ee ff</div>");
    let body = &result.root.children[1];
    let clamp = body
        .children
        .iter()
        .find(|c| c.line_clamp_clamped)
        .expect("clamp 容器盒");
    let span = &clamp.children[0];
    let span_bottom = span.y + span.height;
    assert!(
        span_bottom > 0.5 && span_bottom < 47.5,
        "前置：span 子盒底边应低于保留行底 48（否则形状退化 pin 空转），实得 {span_bottom}"
    );
    assert!(
        (clamp.content_height - 48.0).abs() < 0.5,
        "clamp 容器内容高应 = 保留行底 48（非 span 子盒底边 {span_bottom}），实得 {}",
        clamp.content_height
    );
    assert!(
        (clamp.height - 48.0).abs() < 0.5,
        "盒高应 = 48（无 padding/border），实得 {}",
        clamp.height
    );
    assert!(
        clamp.clamp_kept_line_bottom.is_some_and(|v| (v - 48.0).abs() < 0.5),
        "保留行底镜像应 = 48，实得 {:?}",
        clamp.clamp_kept_line_bottom
    );
}

/// 未被 cap 的容器（行数 < n）：`clamped` false、不镜像保留行底，walk 尾 floor
/// 不参与（恒等语义）。
#[test]
fn r5452_uncapped_container_has_no_kept_bottom_mirror() {
    let result = layout("<div class=\"clamp\"><span class=\"inner\">aa</span></div>");
    let body = &result.root.children[1];
    let clamp = &body.children[0];
    assert!(
        !clamp.line_clamp_clamped,
        "行数 < n 的容器不应被 cap（clamped 应 false）"
    );
    assert!(
        clamp.clamp_kept_line_bottom.is_none(),
        "未 cap 容器不应镜像保留行底，实得 {:?}",
        clamp.clamp_kept_line_bottom
    );
}
