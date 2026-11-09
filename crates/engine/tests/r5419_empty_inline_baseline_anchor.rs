//! R5419：空 inline（零宽 TextRun）单 fragment 盒 y 锚 = 自身基线 − 字体 ascent
//! （CSS2 §10.8.1/§10.6.2——inline 非替换盒 content area = 主字体 A+D，与
//! line-height 无关；chromium oracle 空 span gBCR 顶 = baseline−A 实证）。
//!
//! 案形（empty-inline-002，corpus 2.08%）：div（font 100px/1em Ahem）内空 span
//! 携 bg/border/padding。旧径 `child.y = agg.first_y = baseline − line_height`
//! （break_lines 空 inline 臂 fragment height=line_height 口径），Ahem 1em 形态
//! content 顶比 chromium 低 descent 20px；修复在 finalization 空 inline 臂局部
//! 合成 `first_y + first_frag_h − A`（va=baseline；R4383 own_baseline「片段底 =
//! 行基线」判例面不动 va 定位）。
//!
//! kill-switch `ZW_EMPTY_INLINE_BASELINE_ANCHOR=0` 回退片段顶锚（旧行为）；OFF
//! 轮由 corpus 单案精确回退覆盖（R3858 口径），本文件不含 OFF 针。

use zero_engine::RenderPipeline;

/// 空 span 绿 bg fill 的 (x, y)（唯一高 > 1 的绿 fill；无 padding 空 span 宽 0）。
fn empty_span_top(html: &str) -> (f32, f32) {
    let result = RenderPipeline::new(800.0, 600.0).render_html(html, "");
    let green = result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == 0 && f.color.g == 128 && f.color.b == 0 && f.rect.size.height > 1.0)
        .expect("空 span background fill");
    (green.rect.origin.x, green.rect.origin.y)
}

/// ★load-bearing（empty-inline-002 同值形）：100px/1em Ahem 容器内空 span——
/// content area 顶 = baseline 80 − ascent 80 = 行盒顶 0。旧径 = baseline −
/// line_height = −20（下坠 descent 20px），本针红。
#[test]
fn r5419_empty_inline_content_top_at_line_box_top() {
    let html = r#"<body><div style="font:100px/1em Ahem"><span style="background:green"></span>x</div></body>"#;
    let (x, y) = empty_span_top(html);
    assert_eq!(x, 8.0, "空 span 无水平 frame，x = body margin 8，实测 {x}");
    assert!(
        (y - 8.0).abs() < 0.5,
        "空 span content 顶 = baseline−ascent = 行盒顶+body margin 8（CSS2 §10.6.2），旧径 −20 红，实测 {y}"
    );
}

/// 泛化形（line-height ≠ A+D）：100px/2em Ahem——strut leaded ascent = hl 50 +
/// 80 = 130，行盒顶 0、baseline 130；content 顶 = baseline − 80 = 50。旧径
/// baseline − line_height = −70。合成式（first_y + height − A）应跟随
/// line_height 增量非巧合同值。
#[test]
fn r5419_empty_inline_leaded_line_height_tracks_ascent() {
    let html = r#"<body><div style="font:100px/2em Ahem"><span style="background:green"></span>x</div></body>"#;
    let (_, y) = empty_span_top(html);
    assert!(
        (y - 58.0).abs() < 0.5,
        "line-height 2em 空 span content 顶 = baseline 138 − ascent 80 = 58（含 body margin 8），旧径 −62 红，实测 {y}"
    );
}

/// frame 传播（corpus 案核心机制）：空 span padding 100 + border 25——盒顶 =
/// content 顶 − padding_top − border_top = 0 − 125 = −125；旧径 = first_y −
/// frame = −145。height/agg 区间聚合不动（350 = A+D 100 + frame 250）。
#[test]
fn r5419_empty_inline_frame_deducted_from_baseline_anchor() {
    let html = r#"<body><div style="font:100px/1em Ahem"><span style="background:green;border:25px solid green;padding:100px"></span>x</div></body>"#;
    let (_, y) = empty_span_top(html);
    assert!(
        (y - (-117.0)).abs() < 0.5,
        "盒顶 = content 顶 8 − padding 100 − border 25 = −117，旧径 −137 红，实测 {y}"
    );
}

/// 守卫（any_text 臂不受扰）：有内容 span 走既有 any_text 重写路径（行盒锚），
/// 不进空 inline 臂——fill 顶 = 行盒顶+body margin 8 与修复无关（重复跑同值 = 零扰动）。
#[test]
fn r5419_text_span_path_untouched() {
    let html = r#"<body><div style="font:100px/1em Ahem"><span style="background:green">x</span></div></body>"#;
    let (_, y) = empty_span_top(html);
    assert!(
        (y - 8.0).abs() < 0.5,
        "有内容 span 走 any_text 臂行盒锚（本修复域外），顶 8，实测 {y}"
    );
}

/// 守卫（原子行内级盒排除，background-color-animation-fallback-replace ref 页
/// 回归实证）：display:inline 的 canvas（replaced、无内容、非零宽 fragment）与
/// 空 span 同进 finalization count==1 臂——first_frag_h = 盒高非 line_height，
/// 合成式不得作用（误作用 = canvas 下移 height−A = 85px，本针红）。
/// canvas fill 顶 = 行盒顶 + body margin 8（与块级 div 同位基线行为）。
#[test]
fn r5419_atomic_inline_canvas_untouched() {
    let html = r#"<body><canvas id="c" width="100" height="100" style="background:green"></canvas></body>"#;
    let result = RenderPipeline::new(800.0, 600.0).render_html(html, "");
    let green = result
        .primitives()
        .fills
        .iter()
        .find(|f| f.color.r == 0 && f.color.g == 128 && f.color.b == 0 && f.rect.size.width > 1.0)
        .expect("canvas background fill");
    assert!(
        (green.rect.origin.y - 8.0).abs() < 0.5,
        "inline canvas 原子盒不受合成式作用（first_w==0 门排除），顶 8，实测 {}",
        green.rect.origin.y
    );
}
