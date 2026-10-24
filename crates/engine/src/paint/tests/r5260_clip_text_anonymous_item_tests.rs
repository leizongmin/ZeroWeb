//! R5260（clip:text 恒色染色接入 flex/grid 匿名文本项——css-backgrounds-4
//! §background-clip:text）：paint_anonymous_text_item 旧直接以容器 color 发射字形，
//! 不经 paint_node_inner 恒色 push 站——flex 容器 `color:transparent + bg green +
//! clip:text` 下字形保持透明（clip-text-flex 全空白）。按 owner 容器自身 / DOM
//! 祖先链静态判定补链（与 paint_text 主路径同规）。

use crate::pipeline::RenderPipeline;

fn glyph_colors(html: &str) -> Vec<(u8, u8, u8, u8)> {
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(html, "");
    result
        .primitives()
        .glyphs
        .iter()
        .map(|g| (g.color.r, g.color.g, g.color.b, g.color.a))
        .collect()
}

/// flex 匿名文本项字形承载 bg 恒色（text 透明 = 恒色本体）。
#[test]
fn anonymous_text_item_carries_clip_text_solid_color() {
    let colors = glyph_colors(
        r#"<body><div style="display:flex; color:transparent; background-color:green; background-clip:text; font-size:40px">flex</div></body>"#,
    );
    assert!(!colors.is_empty(), "flex 匿名文本项应发射字形");
    assert!(
        colors.iter().all(|c| *c == (0, 128, 0, 255)),
        "clip:text + 透明 text 下匿名项字形应为 bg 恒色 green，实际 {colors:?}"
    );
}

/// 嵌套形态：clip:text 恒色在 DOM 祖先（flex 容器外层）时匿名项经祖先链接线。
#[test]
fn anonymous_text_item_uses_ancestral_clip_text_color() {
    let colors = glyph_colors(
        r#"<body><div style="color:transparent; background-color:green; background-clip:text"><div style="display:flex"><span>ab</span></div></div></body>"#,
    );
    assert!(!colors.is_empty());
    assert!(
        colors.iter().all(|c| *c == (0, 128, 0, 255)),
        "祖先 clip:text 恒色应经 DOM 链到达匿名项字形，实际 {colors:?}"
    );
}

/// 无 clip:text 上下文时匿名项字形保持容器 color（零扰动对照）。
#[test]
fn anonymous_text_item_without_clip_text_unchanged() {
    let colors = glyph_colors(r#"<body><div style="display:flex; color:rgb(255,0,0); font-size:40px">xy</div></body>"#);
    assert!(!colors.is_empty());
    assert!(
        colors.iter().all(|c| *c == (255, 0, 0, 255)),
        "无 clip:text 时字形应保持容器 color red，实际 {colors:?}"
    );
}
