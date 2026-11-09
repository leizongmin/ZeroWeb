//! R5451 引擎级钉（block-ellipsis-032 对齐参与域）：clamp trim 生效后末保留行
//! 再对齐——ellipsis 计入行内容参与对齐（css-overflow-4 §block-ellipsis；ref
//! 把 ellipsis 计入行内容再对齐，ZW 对齐 pass 先于 clamp cap 且 content_width
//! 含 run 级 hang 宽）。right 盒 trim 后整行附加位移使 kept + ellipsis 合宽
//! 右对齐到内容右缘；left 盒不受再对齐影响（kept 原位）。
//!
//! IFC 级臂钉见 src/inline/tests/r5451_clamp_hang_realign_tests.rs。

use zero_layout_engine::types::{InlineLayoutLine, LayoutBox};
use zero_layout_engine::{LayoutEngine, LayoutResult};

/// 引擎 compute（800×600 视口；样式经外部 sheet 传入，r5342 同款）。
fn compute(html: &str, css: &str) -> LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 收集（首个）存树 IFC 行列表。
fn find_inline_lines(node: &LayoutBox) -> Option<&Vec<InlineLayoutLine>> {
    if let Some(lines) = &node.inline_layout {
        return Some(lines);
    }
    node.children.iter().find_map(find_inline_lines)
}

/// 032 形态 HTML：内容词 + pre-wrap 悬挂空格 span（Ahem 逐空格 16px）+ 被截断
/// 的 "More"。容器 200px = 12.5em；行 1 内容 "ab cd"(80px) + 8 空格(128px)
/// = 208px，"More" 截断。
const SHAPE_HTML: &str = r#"<html><body>
<div class="d"><span>ab cd</span><span class="hangs">        </span>More</div>
</body></html>"#;

/// right 形态：老对齐把 hang 宽计入（content_width 208 → right offset −8），
/// trim 后再对齐：kept 起点 = 200 − 80 − 16（Ahem '…' = font_size）= 104。
const SHAPE_CSS_RIGHT: &str = ".d { width: 200px; font: 16px Ahem; line-clamp: 1; \
text-align: right; margin: 0 } \
.hangs { white-space: pre-wrap; background-color: red; }";

/// left 形态（负向）：再对齐不触发（Left 无位移），kept 原位 x=0。
const SHAPE_CSS_LEFT: &str = ".d { width: 200px; font: 16px Ahem; line-clamp: 1; margin: 0 } \
.hangs { white-space: pre-wrap; background-color: red; }";

/// 主判例：right 盒 trim 后整行再对齐，kept + ellipsis 合宽右对齐到内容右缘
///（kept 起点 104、末端 184，ellipsis 预留 [184, 200]）；清零片段 glued 到
/// kept 末端。
#[test]
fn right_clamped_line_realigned_to_content_right() {
    let result = compute(SHAPE_HTML, SHAPE_CSS_RIGHT);
    let lines = find_inline_lines(&result.root).expect("存树 IFC 应存在");
    assert_eq!(lines.len(), 1, "clamp:1 保留 1 行，实得 {lines:?}");
    let frags = &lines[0].fragments;
    let kept: Vec<_> = frags.iter().filter(|f| !f.text.is_empty()).collect();
    assert!(!kept.is_empty(), "保留内容非空");
    let kept_end = kept.iter().map(|f| f.x + f.width).fold(f32::MIN, f32::max);
    // Ahem '…' advance = font_size = 16；容器内容宽 200。
    assert!(
        (kept_end - (200.0 - 16.0)).abs() < 0.01,
        "right 再对齐：kept 末端 = 内容右缘 − ellipsis 宽（184），实得 {kept_end}（全部 {:?}）",
        kept.iter().map(|f| (f.text.as_str(), f.x, f.width)).collect::<Vec<_>>()
    );
    let cleared: Vec<_> = frags.iter().filter(|f| f.text.is_empty()).collect();
    assert!(
        cleared.iter().all(|f| (f.x - kept_end).abs() < 0.01),
        "清零片段 glued 到 kept 末端，实得 {:?}",
        cleared.iter().map(|f| f.x).collect::<Vec<_>>()
    );
    let band_end = frags.iter().map(|f| f.x + f.width).fold(f32::MIN, f32::max);
    assert!(
        (band_end - kept_end).abs() < 0.01,
        "行 band 上界 = kept 末端（ellipsis 预留位，幻影域零延伸）：{band_end} vs {kept_end}"
    );
}

/// 负向守卫：left 盒再对齐不触发——kept 起点 x=0（老对齐原位），清零片段
/// glued 到 kept 末端（R5450 口径）。
#[test]
fn left_clamped_line_keeps_origin() {
    let result = compute(SHAPE_HTML, SHAPE_CSS_LEFT);
    let lines = find_inline_lines(&result.root).expect("存树 IFC 应存在");
    assert_eq!(lines.len(), 1, "clamp:1 保留 1 行，实得 {lines:?}");
    let frags = &lines[0].fragments;
    let kept: Vec<_> = frags.iter().filter(|f| !f.text.is_empty()).collect();
    let first_kept_x = kept.iter().map(|f| f.x).fold(f32::MAX, f32::min);
    assert!(
        first_kept_x.abs() < 0.01,
        "left 再对齐不触发：kept 起点 = 0，实得 {first_kept_x}"
    );
    let kept_end = kept.iter().map(|f| f.x + f.width).fold(f32::MIN, f32::max);
    let cleared: Vec<_> = frags.iter().filter(|f| f.text.is_empty()).collect();
    assert!(
        cleared.iter().all(|f| (f.x - kept_end).abs() < 0.01),
        "清零片段 glued 到 kept 末端，实得 {:?}",
        cleared.iter().map(|f| f.x).collect::<Vec<_>>()
    );
}
