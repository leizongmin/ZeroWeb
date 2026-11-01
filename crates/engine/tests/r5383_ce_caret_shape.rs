//! R5383（css-ui-4 §5.2.3 caret-shape / §5.2.2 caret-color 两值）：contenteditable
//! 宿主插入 caret 绘制 pin——block 几何（下一可见字符字元格）、auto 色 50% 不透明、
//! 两值第二色改写覆盖字形、空宿主盒级 1ch 回退。对端案形 =
//! css/css-ui/caret-shape-block-*、caret-color-block-shape-text-color-*（reftest）。

use zero_engine::RenderPipeline;
use zero_render_foundation::color::Color;

const FOCUS_OFF: &str = r#"<style>#t:focus{outline:none}</style>"#;

#[test]
fn block_caret_fill_over_first_char() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-color: lime; caret-shape: block; caret-animation: manual; font-size: 5em">abc</div></body>"#
    );
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_focused_selector(Some("#t"));
    let result = pipeline.render_html(&html, "");
    let p = result.primitives();
    let lime: Vec<_> = p
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(0, 255, 0, 255))
        .collect();
    assert!(
        !lime.is_empty(),
        "focused CE + caret-shape:block 应绘制 caret fill，实际 fills {:?}",
        p.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
    // block caret 宽 = 覆盖字符 advance（80px 字号下远大于 bar 的 1px）。
    assert!(
        lime.iter().any(|f| f.rect.size.width > 5.0),
        "block caret 宽应为首字符 advance（>5px），实际 {:?}",
        lime.iter()
            .map(|f| (f.rect.origin.x, f.rect.origin.y, f.rect.size.width, f.rect.size.height))
            .collect::<Vec<_>>()
    );
}

#[test]
fn block_caret_auto_color_is_half_alpha() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-shape: block; caret-animation: manual; font-size: 5em">abc</div></body>"#
    );
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_focused_selector(Some("#t"));
    let result = pipeline.render_html(&html, "");
    let p = result.primitives();
    // 无 caret-color：block caret = 50% 片段色（黑 → a=128，round 口径），与
    // .tentative 案 ref rgba(0,0,0,0.5) 解析路径同值。
    assert!(
        p.fills
            .iter()
            .any(|f| f.color == Color::rgba(0, 0, 0, 128) && f.rect.size.width > 5.0),
        "auto block caret 应为 50% alpha 片段色 fill，实际 fills {:?}",
        p.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

#[test]
fn two_value_caret_color_recolors_overlapped_glyph() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-color: black white; caret-shape: block; caret-animation: manual; font-size: 5em">abc</div></body>"#
    );
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_focused_selector(Some("#t"));
    let result = pipeline.render_html(&html, "");
    let p = result.primitives();
    let white: Vec<_> = p
        .glyphs
        .iter()
        .filter(|g| g.color == Color::rgba(255, 255, 255, 255))
        .collect();
    assert_eq!(
        white.len(),
        1,
        "仅覆盖字符 'a' 应着第二色 white，实际 white 字形 {:?}，全部 {:?}",
        white.iter().map(|g| (g.x, g.glyph_id)).collect::<Vec<_>>(),
        p.glyphs.iter().map(|g| (g.x, g.color)).collect::<Vec<_>>()
    );
    // 其余字符保持元素色（默认黑）。
    assert!(
        p.glyphs
            .iter()
            .filter(|g| g.color != Color::rgba(255, 255, 255, 255))
            .all(|g| g.color == Color::rgba(0, 0, 0, 255)),
        "未覆盖字符应保持黑色"
    );
}

#[test]
fn empty_editable_block_caret_fills_one_ch() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-color: green; caret-shape: block; caret-animation: manual; font: 100px/1 Ahem; width: 1ch; background: red"></div></body>"#
    );
    let mut pipeline = RenderPipeline::new(200.0, 200.0);
    pipeline.set_focused_selector(Some("#t"));
    let result = pipeline.render_html(&html, "");
    let p = result.primitives();
    // 空宿主盒级路径：block caret = 1ch × 行高（无下一可见字符回退 1ch）。
    let green: Vec<_> = p
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(0, 128, 0, 255) && f.rect.size.width >= 90.0)
        .collect();
    assert!(
        !green.is_empty(),
        "空 CE 宿主应绘制 1ch×行高 block caret，实际 fills {:?}",
        p.fills
            .iter()
            .map(|f| (f.color, f.rect.size.width, f.rect.size.height))
            .collect::<Vec<_>>()
    );
}

#[test]
fn unfocused_editable_has_no_caret() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-color: lime; caret-shape: block; font-size: 5em">abc</div></body>"#
    );
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    let result = pipeline.render_html(&html, "");
    let p = result.primitives();
    assert!(
        !p.fills.iter().any(|f| f.color == Color::rgba(0, 255, 0, 255)),
        "未聚焦 CE 不应绘制 caret"
    );
}
