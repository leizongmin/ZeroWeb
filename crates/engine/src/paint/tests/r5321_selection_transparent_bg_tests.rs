//! R5321（css-pseudo-4 §highlight-cascading）：作者 `::selection` 显式
//! `background-color: transparent` 压过 UA origin——伪样式须存储（此前存储门
//! 后值判据把 transparent/同值声明与无规则同判，误不存储 → paint 侧 R5272
//! UA 默认高亮回落再铺 OS 高亮底）。driving:
//! selection-background-color-transparent（mismatch 型，test 与 notref 须异）。
//! 控制腿同页无 ::selection 规则仍回落 UA 默认（R5272 机制不回退）。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

fn highlight() -> Color {
    Color::rgba(51, 153, 255, 255)
}

/// 作者 transparent ::selection bg → 不铺 UA 默认 Highlight fill。
#[test]
fn author_transparent_selection_bg_suppresses_ua_default() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("div".to_string(), 0, 13)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><div style="margin:0">Selected text</div></body></html>"#,
        "div::selection { background-color: transparent; }",
    );
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color == highlight()),
        "作者 transparent 声明应压过 UA origin（零 Highlight fill），实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 控制腿：同页去掉 ::selection 规则 → UA 默认 Highlight fill 照铺（R5272 不回退）。
#[test]
fn no_author_rule_still_falls_back_to_ua_highlight() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("div".to_string(), 0, 13)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><div style="margin:0">Selected text</div></body></html>"#,
        "",
    );
    let prims = result.primitives();
    assert!(
        prims.fills.iter().any(|f| f.color == highlight()),
        "无作者规则应回落 UA 默认 Highlight fill，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 作者声明非 transparent 背景 → 照铺作者色（既有存储腿不回退）。
#[test]
fn author_blue_selection_bg_still_paints_author_color() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("div".to_string(), 0, 13)]);
    let result = pipeline.render_html(
        r#"<html><body style="margin:0"><div style="margin:0">Selected text</div></body></html>"#,
        "div::selection { background-color: rgb(0, 128, 0); }",
    );
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color == highlight()),
        "作者规则在场不应回落 UA 默认 Highlight"
    );
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(0, 128, 0, 255)),
        "作者绿色背景应照铺，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}
