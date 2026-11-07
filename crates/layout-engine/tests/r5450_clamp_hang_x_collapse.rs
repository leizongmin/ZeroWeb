//! R5450 引擎级钉（block-ellipsis-032 红带 band 域根因）：clamp trim 清零的悬挂
//! 空格片段 x 折叠到剪除点。R5297 `owner_line_band_extent_map` 按 (x, x+width)
//! union，清零片段保留递进 x 时祖先 band 域仍延伸 → paint R639 fragment bg 臂
//! whole_box 按幻影带涂红底；折叠后 band 退化为零宽点（032 红带消除）。
//!
//! IFC 级臂钉见 src/inline/tests/r5450_clamp_hang_x_collapse_tests.rs；
//! kill-switch 独立进程钉见 r5450_clamp_hang_x_collapse_kill_switch.rs。

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

/// 032 形态 HTML：内容词 + pre-wrap 悬挂空格 span（多空格，pre-wrap 逐空格成
/// 片段）+ 被截断的 "More"。
const SHAPE_HTML: &str = r#"<html><body>
<div class="d"><span>ab cd</span><span class="hangs">        </span>More</div>
</body></html>"#;

const SHAPE_CSS: &str = ".d { width: 200px; font: 16px Ahem; line-clamp: 1; margin: 0 } \
.hangs { white-space: pre-wrap; background-color: red; }";

const SHAPE_CSS_NO_CLAMP: &str = ".d { width: 200px; font: 16px Ahem; margin: 0 } \
.hangs { white-space: pre-wrap; background-color: red; }";

/// 主判例：clamp:1 保留行上被清零的悬挂空格片段 x 全部折叠到剪除点，行 band
/// 上界（max(x+width)）不越过该点（R5297 幻影带域零延伸）。
#[test]
fn clamped_cleared_hang_fragments_x_collapsed() {
    let result = compute(SHAPE_HTML, SHAPE_CSS);
    let lines = find_inline_lines(&result.root).expect("存树 IFC 应存在");
    assert_eq!(lines.len(), 1, "clamp:1 保留 1 行，实得 {lines:?}");
    let frags = &lines[0].fragments;
    let cleared: Vec<_> = frags.iter().filter(|f| f.text.is_empty()).collect();
    assert!(
        cleared.len() >= 2,
        "pre-wrap 悬挂空格逐空格成片段且同臂清零，实得 {} 个清零片段（全部 {:?}）",
        cleared.len(),
        frags.iter().map(|f| (&f.text, f.x, f.width)).collect::<Vec<_>>()
    );
    let trim_x = cleared[0].x;
    assert!(
        cleared.iter().all(|f| (f.x - trim_x).abs() < 0.01),
        "清零片段 x 全部折叠到剪除点 {trim_x}，实得 {:?}",
        cleared.iter().map(|f| f.x).collect::<Vec<_>>()
    );
    let band_end = frags.iter().map(|f| f.x + f.width).fold(f32::MIN, f32::max);
    assert!(
        (band_end - trim_x).abs() < 0.01,
        "行 band 上界不越过剪除点（红底幻影带域消失）：{band_end} vs {trim_x}"
    );
}

/// 负向守卫：同形态去掉 line-clamp → 悬挂空格片段保留且 x 自然递进（不折叠、
/// 不误剪非截断内容）。
#[test]
fn unclamped_keeps_advancing_space_fragments() {
    let result = compute(SHAPE_HTML, SHAPE_CSS_NO_CLAMP);
    let lines = find_inline_lines(&result.root).expect("存树 IFC 应存在");
    assert!(lines.len() >= 2, "无 clamp：内容折 2 行以上，实得 {} 行", lines.len());
    let spaces: Vec<_> = lines[0].fragments.iter().filter(|f| f.text == " ").collect();
    assert!(
        spaces.len() >= 2,
        "保留行含逐空格片段，实得 {:?}",
        lines[0].fragments.iter().map(|f| (&f.text, f.x)).collect::<Vec<_>>()
    );
    assert!(
        spaces.windows(2).all(|w| w[0].x < w[1].x),
        "未截断：空格片段 x 严格递进（不折叠），实得 {:?}",
        spaces.iter().map(|f| f.x).collect::<Vec<_>>()
    );
}
