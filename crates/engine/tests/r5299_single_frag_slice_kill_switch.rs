//! R5299 单行整盒 slice 收口锚（独立测试进程——`ZW_INLINE_SINGLE_FRAG_BOX_BORDER=1`
//! 进程内 set_var 与并行单测互踩，R3858 教训）：
//! 1. band 域 bg + once 绘制：词缝连续带、单 fill（逐片段重绘会以后者 fill 盖
//!    前者字形——selection-012 flag-on 实证）；
//! 2. 选区 fill 与元素 bg 同 band 域（逐词域在词缝露元素底色，selection-012
//!    ref 全带绿 vs test 词缝红）；
//! 3. backdrop-filter owner 保留基线口径（滤区取盒级 rect，整盒合成未实现，
//!    backdrop-filter-inline-positioning）。

use zero_engine::RenderPipeline;
use zero_render_foundation::color::Color;

/// 旗标开：单行 bg span 的背景 = owner 行连续域（跨词缝）单矩形（once 绘制），
/// 非逐词两矩形。
#[test]
fn single_frag_bg_band_continuous_once() {
    unsafe { std::env::set_var("ZW_INLINE_SINGLE_FRAG_BOX_BORDER", "1") };
    let html =
        r#"<body style="margin:0"><div><span style="background:#00F; font: 40px Ahem">aa bb</span></div></body>"#;
    let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
    let p = result.primitives();
    let blues: Vec<_> = p
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(0, 0, 255, 255) && f.rect.size.width > 1.0 && f.rect.size.height > 1.0)
        .collect();
    assert_eq!(blues.len(), 1, "band 域 once 绘制应单 fill，实际 {:?}", blues);
    // 带宽覆盖两词 + 缝：4 全角字符 160px + 词缝 > 170。
    let band = blues[0].rect;
    assert!(band.size.width > 170.0, "带应跨词缝连续，实际宽 {}", band.size.width);
    unsafe { std::env::remove_var("ZW_INLINE_SINGLE_FRAG_BOX_BORDER") };
}

/// 旗标开：选区 fill 与元素 bg 同 x 域（同 rect）——逐词域词缝露底色。
#[test]
fn single_frag_selection_fill_matches_bg_band() {
    unsafe { std::env::set_var("ZW_INLINE_SINGLE_FRAG_BOX_BORDER", "1") };
    let html = r#"<body style="margin:0"><div><span id="s" style="background:#F00; font: 40px Ahem">aa bb</span></div></body>"#;
    let mut pipeline = RenderPipeline::new(400.0, 300.0);
    pipeline.set_text_selections(vec![("#s".to_string(), 0, 5)]);
    let result = pipeline.render_html(html, "");
    let p = result.primitives();
    // bg（红）与选区高亮（UA 默认色）各一 rect；两者 x/宽应全等（band 域）。
    let red: Vec<_> = p
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(255, 0, 0, 255) && f.rect.size.width > 1.0)
        .collect();
    assert_eq!(red.len(), 1, "元素 bg 应单 fill，实际 {:?}", red);
    let sel: Vec<_> = p
        .fills
        .iter()
        .filter(|f| f.color != Color::rgba(255, 0, 0, 255) && f.color.a > 0 && f.rect.size.width > 1.0)
        .collect();
    let bg = red[0].rect;
    assert!(
        sel.iter()
            .any(|f| (f.rect.origin.x - bg.origin.x).abs() < 0.5 && (f.rect.size.width - bg.size.width).abs() < 0.5),
        "选区 fill 应与元素 bg 同 band 域，bg={bg:?} sel={:?}",
        sel.iter().map(|f| f.rect).collect::<Vec<_>>()
    );
    unsafe { std::env::remove_var("ZW_INLINE_SINGLE_FRAG_BOX_BORDER") };
}

/// 旗标开：backdrop-filter owner 保留基线口径——bg 不走整盒（带 ± pad 外延），
/// 宽 = 词域（滤区盒级 rect 合成语义未实现）。
#[test]
fn backdrop_filter_owner_keeps_legacy_geometry() {
    unsafe { std::env::set_var("ZW_INLINE_SINGLE_FRAG_BOX_BORDER", "1") };
    let html = r#"<body style="margin:0"><div><span style="backdrop-filter: invert(1); background:#F00; font: 40px Ahem">ab</span></div></body>"#;
    let result = RenderPipeline::new(400.0, 300.0).render_html(html, "");
    let p = result.primitives();
    let reds: Vec<_> = p
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(255, 0, 0, 255) && f.rect.size.width > 1.0)
        .collect();
    assert_eq!(reds.len(), 1, "bg 应单 fill，实际 {:?}", reds);
    // 词域宽 = 2 全角字符 80px；整盒几何会 ± pad（20px）外延 → >100。
    assert!(
        reds[0].rect.size.width < 100.0,
        "backdrop owner 应保留基线词域 bg，实际宽 {}",
        reds[0].rect.size.width
    );
    unsafe { std::env::remove_var("ZW_INLINE_SINGLE_FRAG_BOX_BORDER") };
}
