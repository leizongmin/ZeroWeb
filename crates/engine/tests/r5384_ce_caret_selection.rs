//! R5384（css-ui-4 §5.2.3 caret-shape 插入点 / §5.2.2 caret-color 两值）：Selection
//! 插入点 caret 定位 pin——getSelection()/Range 回流偏移驱动 caret 定位、覆盖字符
//! 跟随重涂、行尾溢出可见（pending 延后发射）、元素端点相邻文本解析、1ch 回退。
//! 对端案形 = css/css-ui/caret-eol-001..004、caret-color-block-shape-text-color-003/005、
//! caret-shape-block-fallback-001（reftest）。

use zero_engine::RenderPipeline;
use zero_render_foundation::color::Color;

const FOCUS_OFF: &str = r#"<style>#t:focus{outline:none}</style>"#;

/// text 端点偏移 1：block caret 覆盖 'b' 字元格（x = 片段起点 + 'a' advance），两值
/// 第二值仅重涂覆盖字符（003 语义——'a'/'c' 保持原色）。
#[test]
fn selection_offset_one_recolors_second_glyph() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-color: black white; caret-shape: block; caret-animation: manual; font-size: 5em">abc</div></body>"#
    );
    let mut pipeline = RenderPipeline::new(600.0, 300.0);
    pipeline.set_ce_caret(Some(("#t".into(), vec![0], 1, true)));
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
        "仅覆盖字符 'b' 应着第二色 white，实际 white 字形 {:?}，全部 {:?}",
        white.iter().map(|g| (g.x, g.glyph_id)).collect::<Vec<_>>(),
        p.glyphs.iter().map(|g| (g.x, g.color)).collect::<Vec<_>>()
    );
    assert_eq!(white[0].glyph_id, 'b' as u32, "重涂字符应为偏移位 'b'");
    // caret 本体（黑）x 与覆盖字形 x 对齐（同字元格）。
    let black_caret = p
        .fills
        .iter()
        .find(|f| f.color == Color::rgba(0, 0, 0, 255) && f.rect.size.width > 5.0);
    let caret = black_caret.unwrap_or_else(|| {
        panic!(
            "应存在黑色 block caret fill，fills {:?}",
            p.fills.iter().map(|f| f.color).collect::<Vec<_>>()
        )
    });
    assert!(
        (caret.rect.origin.x - white[0].x).abs() < 2.0,
        "caret x({}) 应与覆盖字形 x({}) 对齐",
        caret.rect.origin.x,
        white[0].x
    );
}

/// 偏移越界（== 文本长度，005 中间态语义）：无覆盖字符 → 零重涂，caret fill 仍在。
#[test]
fn selection_offset_past_end_emits_no_recolor() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-color: black white; caret-shape: block; caret-animation: manual; font-size: 5em">abc</div></body>"#
    );
    let mut pipeline = RenderPipeline::new(600.0, 300.0);
    pipeline.set_ce_caret(Some(("#t".into(), vec![0], 3, true)));
    let result = pipeline.render_html(&html, "");
    let p = result.primitives();
    assert!(
        !p.glyphs.iter().any(|g| g.color == Color::rgba(255, 255, 255, 255)),
        "行尾插入位无覆盖字符，不应有重涂字形，实际 {:?}",
        p.glyphs.iter().map(|g| (g.x, g.color)).collect::<Vec<_>>()
    );
    assert!(
        p.fills
            .iter()
            .any(|f| f.color == Color::rgba(0, 0, 0, 255) && f.rect.size.width > 5.0),
        "行尾 caret fill 仍应发射，fills {:?}",
        p.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 元素端点（child 索引 = childCount，selectAllChildren+collapseToEnd 形态）解析到
/// 前驱文本子末尾；行尾 caret 经 pending 延后发射逃逸 overflow:hidden 裁剪
///（eol-001..004 语义）。Ahem 80px：文本 2 字符宽 160，caret 1ch 回退 = 1em = 80。
#[test]
fn eol_caret_survives_overflow_hidden() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-color: orange; caret-shape: block; caret-animation: manual; font: 80px Ahem; width: 160px; overflow: hidden; white-space: pre">ab</div></body>"#
    );
    let mut pipeline = RenderPipeline::new(600.0, 300.0);
    // 元素端点：宿主自身（空路径）child 索引 1（= 唯一文本子之后）。
    pipeline.set_ce_caret(Some(("#t".into(), vec![], 1, false)));
    let result = pipeline.render_html(&html, "");
    let p = result.primitives();
    let orange: Vec<_> = p
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(255, 165, 0, 255))
        .collect();
    assert_eq!(
        orange.len(),
        1,
        "行尾 caret 应恰好一条 orange fill（逃逸 overflow:hidden），fills {:?}",
        p.fills
            .iter()
            .map(|f| (f.color, f.rect.origin.x, f.rect.size.width))
            .collect::<Vec<_>>()
    );
    let c = orange[0];
    assert!(
        (c.rect.origin.x - 160.0).abs() < 2.0,
        "行尾 caret x 应在文本末缘（160），实际 {}",
        c.rect.origin.x
    );
    assert!(
        (c.rect.size.width - 80.0).abs() < 2.0,
        "无覆盖字符 caret 宽应回退 1ch（Ahem 1em=80），实际 {}",
        c.rect.size.width
    );
}

/// 溢出行钳制（eol-004 语义）：12 字符溢出 5ch 盒，行尾插入位钳制到内容区右缘。
#[test]
fn eol_caret_clamped_to_visible_line_end() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-color: orange; caret-shape: block; caret-animation: manual; font: 80px Ahem; width: 400px; overflow: hidden; white-space: pre">abcdefghijkl</div></body>"#
    );
    let mut pipeline = RenderPipeline::new(600.0, 300.0);
    pipeline.set_ce_caret(Some(("#t".into(), vec![], 1, false)));
    let result = pipeline.render_html(&html, "");
    let p = result.primitives();
    let caret = p
        .fills
        .iter()
        .find(|f| f.color == Color::rgba(255, 165, 0, 255))
        .unwrap_or_else(|| {
            panic!(
                "行尾 caret 应发射，fills {:?}",
                p.fills.iter().map(|f| f.color).collect::<Vec<_>>()
            )
        });
    assert!(
        (caret.rect.origin.x - 400.0).abs() < 2.0,
        "越界插入位应钳制到内容区右缘（400=5ch），实际 {}",
        caret.rect.origin.x
    );
}

/// stale selector / 空路径越界：解析失败不注入 → 无 caret fill（默认位也不触发——
/// 无焦点）。
#[test]
fn stale_target_emits_no_caret() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-color: lime; caret-shape: block; caret-animation: manual; font-size: 5em">abc</div></body>"#
    );
    let mut pipeline = RenderPipeline::new(600.0, 300.0);
    pipeline.set_ce_caret(Some(("#nonexistent".into(), vec![], 0, false)));
    let result = pipeline.render_html(&html, "");
    let p = result.primitives();
    assert!(
        !p.fills.iter().any(|f| f.color == Color::rgba(0, 255, 0, 255)),
        "stale selector 不应绘制 caret fill，实际 fills {:?}",
        p.fills.iter().map(|f| f.color).collect::<Vec<_>>()
    );
}

/// 偏移落在目标容器之前的片段不误发射：caret 只在容器节点对应的片段发射一次。
#[test]
fn caret_fires_only_in_container_fragment() {
    let html = format!(
        r#"<body style="margin:0">{FOCUS_OFF}<div id="t" contenteditable style="caret-color: lime; caret-shape: block; caret-animation: manual; font: 80px Ahem"><span id="s">abc</span>def</div></body>"#
    );
    let mut pipeline = RenderPipeline::new(800.0, 300.0);
    // 宿主路径 [0] → span，元素端点 child 索引 1 = 'abc' 文本之后（span 内末尾）——
    // 练路径下钻 + 前驱文本解析（'def' 文本不在 span 内，不得误命中）。
    pipeline.set_ce_caret(Some(("#t".into(), vec![0], 1, false)));
    let result = pipeline.render_html(&html, "");
    let p = result.primitives();
    let lime: Vec<_> = p
        .fills
        .iter()
        .filter(|f| f.color == Color::rgba(0, 255, 0, 255))
        .collect();
    assert_eq!(
        lime.len(),
        1,
        "插入位 caret 应恰一条，fills {:?}",
        lime.iter()
            .map(|f| (f.rect.origin.x, f.rect.size.width))
            .collect::<Vec<_>>()
    );
    // 'abc' 3 字符 × 80px → caret 在 x=240。
    assert!(
        (lime[0].rect.origin.x - 240.0).abs() < 2.0,
        "caret 应在 'abc' 末缘 240，实际 {}",
        lime[0].rect.origin.x
    );
}
