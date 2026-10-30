//! R5270（css-pseudo-4 §highlight-painting/§highlight-bounds）：inline owner
//! 自持选区**行高亮 fill**——orphan inline（无 LayoutBox → inline_heights 无
//! 条目 → owner_h==0 → R639 片段 bg 站触发，driving: active-selection-012 红
//! 底 2.27%）被整 contents 选中时，高亮 fill 与元素背景**同 rect 公式、同发
//! 射站**（紧随元素 bg = 画于其上，宏体次序先于片段字形 = 字形之下），逐 run
//! 矩形（词间隙不连体——R5269 行级连体 fill 实证铺出带外细条）。
//! 页面形取 012 结构（前置块级 `<p>` 兄弟 = orphan 触发形：无前缀时 inline div
//! 持 LayoutBox，bg 走盒级站，本臂门不触发）。部分行高亮维持挂账；clip:text
//! 恒色 owner 与元素 bg 同款抑制。

use zero_render_foundation::color::Color;

use crate::pipeline::RenderPipeline;

/// 整contents 选中 orphan inline：green 选区 fill 与 red 元素 bg **逐 run 同矩
/// 形**（同站发射），每个 green 在对应 red 之后（画于其上）。
#[test]
fn sel_fill_rect_matches_element_bg_rect_and_paints_above() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_text_selections(vec![("#test".to_string(), 0, 13)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><p style="margin:0">intro</p><div id="test" style="display:inline; background-color: red; color: fuchsia; font: 50px Ahem">Selected Text</div></body>"#,
        "div::selection { background-color: green; color: aqua }",
    );
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
    assert!(!reds.is_empty(), "元素 bg red fill 应存在，实际 {:?}", prims.fills);
    assert_eq!(
        greens.len(),
        reds.len(),
        "选区 fill 应与元素 bg 逐 run 一一对应，实际 reds {:?} greens {:?}",
        reds,
        greens
    );
    for ((gi, g), (ri, r)) in greens.iter().zip(reds.iter()) {
        assert!(
            gi > ri,
            "选区 fill 应画于元素 bg 之上（发射序 red→green），实际 {} → {}",
            ri,
            gi
        );
        assert_eq!(g.rect, r.rect, "选区 fill 矩形应与元素 bg 完全同几何（同站同公式）");
    }
    assert!(
        prims
            .glyphs
            .iter()
            .filter(|g| g.color != Color::rgba(0, 0, 0, 255))
            .all(|g| g.color == Color::rgba(0, 255, 255, 255)),
        "字形应为自持伪样式色 aqua 且不被 fill 盖"
    );
}

/// 透明元素 bg 的 owner 仍铺选区 fill（门扩展臂，非 has_bg 搭载）；逐 run 矩形
/// 同行不连体（词间隙留空，R5269 行级连体回归锚）。
/// R5350：本测钉 R5270 站**逐 run**几何——扩张轴（默认开）把站 fill 与元素
/// bg 同公式整盒化（带域），逐 run 锚钉在轴关态（`ZW_INLINE_CA_EXPAND=0`，
/// Rust 2024 set_var 为 unsafe；扩张轴几何锚在 crates/engine/tests/
/// r5350_inline_ca_expand_axis.rs 独立进程——R3858 教训：进程内 set_var 与
/// 并行单测互踩）。
#[test]
fn transparent_bg_owner_still_gets_selection_fill() {
    unsafe { std::env::set_var("ZW_INLINE_CA_EXPAND", "0") };
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_text_selections(vec![("#test".to_string(), 0, 13)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><p style="margin:0">intro</p><div id="test" style="display:inline; color: fuchsia; font: 50px Ahem">Selected Text</div></body>"#,
        "div::selection { background-color: rgb(0,128,0); color: aqua }",
    );
    let prims = result.primitives();
    let mut greens: Vec<_> = prims
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(0, 128, 0, 255))
        .map(|f| f.rect)
        .collect();
    greens.sort_by(|a, b| a.origin.x.total_cmp(&b.origin.x));
    assert_eq!(
        greens.len(),
        2,
        "应逐 run 铺 2 个选区 fill，实际 fills {:?}",
        prims.fills
    );
    for r in &greens {
        assert!(r.size.width > 0.0 && r.size.height > 0.0);
    }
    assert_eq!(greens[0].origin.y, greens[1].origin.y, "同 run 行 fill 应同行顶");
    assert_eq!(greens[0].size.height, greens[1].size.height);
    assert!(
        greens[1].origin.x >= greens[0].origin.x + greens[0].size.width,
        "逐 run 矩形不连体（词间隙不铺），实际 {:?}",
        greens
    );
    unsafe { std::env::remove_var("ZW_INLINE_CA_EXPAND") };
}

/// clip:text 恒色 owner 与元素 bg 同款抑制——站触发（orphan 形）下选区 fill
/// 仍不走片段 bg 站。
#[test]
fn clip_text_solid_owner_selection_fill_suppressed() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_text_selections(vec![("#test".to_string(), 0, 1)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><p style="margin:0">intro</p><div id="test" style="display:inline; font-size:100px; background-image: linear-gradient(rgb(0,255,0), rgb(0,255,0)); background-clip: text; color: transparent">X</div></body>"#,
        "div::selection { background-color: blue; color: white }",
    );
    let prims = result.primitives();
    assert!(
        !prims.fills.iter().any(|f| f.color == Color::rgba(0, 0, 255, 255)),
        "clip:text 恒色 owner 不应铺选区 fill（同元素 bg 抑制），实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 选区终点超界（end ≥ textContent 长即整 contents，≡ R5256 口径）→ 仍铺。
#[test]
fn selection_end_overflow_still_counts_as_whole_contents() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    pipeline.set_text_selections(vec![("#test".to_string(), 0, 999)]);
    let result = pipeline.render_html(
        r#"<body style="margin:0"><p style="margin:0">intro</p><div id="test" style="display:inline; color: fuchsia; font: 50px Ahem">Selected Text</div></body>"#,
        "div::selection { background-color: rgb(0,128,0); color: aqua }",
    );
    let prims = result.primitives();
    assert!(
        prims.fills.iter().any(|f| f.color == Color::rgba(0, 128, 0, 255)),
        "整 contents（end 超界）应铺选区 fill，实际 fills {:?}",
        prims.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}
