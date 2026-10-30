//! R5350 inline 内容区锚定绘制**扩张轴**锚（独立测试进程——轴默认开，进程内
//! set_var 与并行单测互踩，R3858 教训；kill-switch `ZW_INLINE_CA_EXPAND=0`）：
//! 1. 站 fill 带域 once：透明 bg owner 的选区高亮整盒化为单矩形（跨词缝连续
//!    带，band-once 去重——SFBB 关时段表恒空，逐片段重绘在半透明色下 α 双重
//!    加深）；轴关态逐 run 双矩形由 r5270 lib 测试钉定。
//! 2. 选区 fill 与元素 bg 同站同 rect（R5270 创始不变量整盒化后保持）。
//!
//! 有边框盒不入轴（扩张轴边框守卫）的常驻锚 = corpus reftest insert-009
//!（nosplit-ref 嵌套边框栈 3px 平移实证，与修复同提交）。
//!
//! 页面注记：orphan 形（前置块级 `<p>` 兄弟）是唯一触发 R5270 站的形态；该
//! 形态下文本整形丢自身 font-size/family 回落 16px 默认字体（既有缺陷，颜色
//! 样式不受影响）——Ahem 尺寸敏感断言在此形态不成立，本文件只锚形态无关的
//! 发射拓扑不变量。

use zero_engine::RenderPipeline;
use zero_render_foundation::color::Color;

/// 扩张轴（默认开）：透明 bg owner 的选区站 fill = 跨词缝连续带**单矩形**
///（once 去重），覆盖全部字形横向域（含词缝）。
#[test]
fn expand_axis_station_fill_band_once_crosses_word_gap() {
    let html = r#"<body style="margin:0"><p style="margin:0">intro</p><div id="test" style="display:inline; color: fuchsia; font: 40px Ahem">aa bb</div></body>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#test".to_string(), 0, 5)]);
    let result = pipeline.render_html(html, "div::selection { background-color: rgb(0,128,0); color: aqua }");
    let prims = result.primitives();
    let greens: Vec<_> = prims
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(0, 128, 0, 255))
        .map(|f| f.rect)
        .collect();
    assert_eq!(greens.len(), 1, "带域 once 应恰一选区 fill，实际 {greens:?}");
    let band = greens[0];
    let glyph_xs: Vec<_> = prims
        .glyphs
        .iter()
        .filter(|g| g.color == Color::rgba(0, 255, 255, 255))
        .map(|g| g.x)
        .collect();
    assert!(glyph_xs.len() >= 2, "被选文本应至少 2 个字形，实际 {glyph_xs:?}");
    let first = glyph_xs.iter().cloned().fold(f32::MAX, f32::min);
    let last = glyph_xs.iter().cloned().fold(f32::MIN, f32::max);
    assert!(
        band.left() <= first && band.right() > last,
        "带应覆盖全部字形横向域（含词缝，非逐词断带），band={band:?} glyphs=[{first},{last}]"
    );
}

/// 扩张轴：元素 bg 与选区 fill 同站同 rect（整盒化后 R5270 不变量保持），发
/// 射序 bg → fill（fill 画于其上）。
#[test]
fn expand_axis_selection_fill_matches_element_bg_rect() {
    let html = r#"<body style="margin:0"><p style="margin:0">intro</p><div id="test" style="display:inline; background-color: red; font: 40px Ahem">aa bb</div></body>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#test".to_string(), 0, 5)]);
    let result = pipeline.render_html(html, "div::selection { background-color: rgb(0,128,0); color: aqua }");
    let prims = result.primitives();
    let reds: Vec<_> = prims
        .fills
        .iter()
        .enumerate()
        .filter(|(_, f)| f.color == Color::rgba(255, 0, 0, 255))
        .collect();
    let greens: Vec<_> = prims
        .fills
        .iter()
        .enumerate()
        .filter(|(_, f)| f.color == Color::rgba(0, 128, 0, 255))
        .collect();
    assert_eq!(reds.len(), 1, "元素 bg 应带域 once 单矩形，实际 {reds:?}");
    assert_eq!(greens.len(), 1, "选区 fill 应带域 once 单矩形，实际 {greens:?}");
    assert!(greens[0].0 > reds[0].0, "fill 应画于 bg 之上（发射序 bg → fill）");
    assert_eq!(greens[0].1.rect, reds[0].1.rect, "选区 fill 与元素 bg 应同站同 rect");
}
