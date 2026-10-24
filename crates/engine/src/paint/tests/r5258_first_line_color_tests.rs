//! R5258（::first-line 首格式行字形 color，切片 1——CSS2 §5.12.1）：块容器
//! `::first-line` 伪样式的 color 应用到首格式行（行组 line_top == 组内最小）的
//! 字形；inline 后代解析 color == 盒 color（≈继承）随之，显式声明胜出继承。
//! 切片 2（css-pseudo-4 §first-line-inheritance）：首行 inline 后代从伪样式
//! 继承 color（style 期直亲传播），bg 渐变 currentcolor 随之
//!（driving: linear-gradient-currentcolor-first-line）。bg/font 类属性不应用；
//! 竖排不应用；跨行 inline 次行回落、深结构域穿透（均 FIXME）。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

/// 首行 color 应用到块自身文本与继承的 inline 后代。
#[test]
fn first_line_color_applies_to_block_and_inherited_inline() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(
        r#"<body><div style="color: red; font: 50px Ahem;"><span>Hello</span></div></body>"#,
        "div::first-line { color: green }",
    );
    let prims = result.primitives();
    assert!(
        !prims.glyphs.is_empty() && prims.glyphs.iter().all(|g| g.color == Color::rgba(0, 128, 0, 255)),
        "首行（单行页全部行）字形应为伪样式 green，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
}

/// 次行不应用——仅行组最小的首行随伪样式。
#[test]
fn first_line_color_second_line_keeps_element_color() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(
        r#"<body><div style="color: red; font: 50px Ahem; width: 300px">AAAAA BBBBB</div></body>"#,
        "div::first-line { color: green }",
    );
    let prims = result.primitives();
    assert!(
        prims.glyphs.iter().any(|g| g.color == Color::rgba(0, 128, 0, 255)),
        "首行字形应为 green"
    );
    assert!(
        prims.glyphs.iter().any(|g| g.color == Color::rgba(255, 0, 0, 255)),
        "次行字形应保持元素 red"
    );
}

/// inline 后代显式 color 声明胜出 ::first-line 继承。
#[test]
fn first_line_explicit_descendant_color_wins() {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(
        r#"<body><div style="color: red; font: 50px Ahem;"><span style="color: blue">Hello</span></div></body>"#,
        "div::first-line { color: green }",
    );
    let prims = result.primitives();
    assert!(
        !prims.glyphs.is_empty() && prims.glyphs.iter().all(|g| g.color == Color::rgba(0, 0, 255, 255)),
        "显式 color 的后代字形应保持 blue，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
}

/// 切片 2：首行 inline 后代 bg 渐变 currentcolor 随伪样式继承变绿
///（WPT linear-gradient-currentcolor-first-line 同形——span 无显式 color，
/// 渐变停色按继承后的元素 color 解析）。
#[test]
fn first_line_inherited_color_feeds_gradient_currentcolor() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let result = pipeline.render_html(
        r#"<body><div><span>Green</span></div></body>"#,
        "div { color: red; } div::first-line { color: green; } span { background-image: linear-gradient(currentcolor, currentcolor); }",
    );
    let prims = result.primitives();
    let green = Color::rgba(0, 128, 0, 255);
    assert!(
        prims.glyphs.iter().filter(|g| g.color == green).count() == 5,
        "span 字形应随伪样式 green，实际 {:?}",
        prims.glyphs.iter().map(|g| g.color).collect::<Vec<_>>()
    );
    assert!(
        !prims.gradients.is_empty() && prims.gradients.iter().all(|g| g.stops.iter().all(|s| s.color == green)),
        "span bg 渐变 currentcolor 停色应为 green，实际 {:?}",
        prims
            .gradients
            .iter()
            .flat_map(|g| g.stops.iter().map(|s| s.color))
            .collect::<Vec<_>>()
    );
}
