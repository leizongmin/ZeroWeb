//! R5449 引擎级 032 形态钉（block-ellipsis-032 红带驱动）：line-clamp 截断后末保留行
//! 的 pre-wrap 悬挂空格片段被剪除（红底 span 无残留空格）；未 clamp 内容空格保留。
//!
//! IFC 级臂钉（纯空白清零 / 混合剪尾）见 src/inline/tests/r5449_clamp_hang_trim_tests.rs；
//! kill-switch 独立进程钉见 r5449_clamp_hang_trim_kill_switch.rs。

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

/// 032 形态 HTML：内容词 + pre-wrap 悬挂空格 span + 被截断的 "More"。
const SHAPE_HTML: &str = r#"<html><body>
<div class="d"><span>ab cd</span><span class="hangs">    </span>More</div>
</body></html>"#;

const SHAPE_CSS: &str = ".d { width: 96px; font: 16px Ahem; line-clamp: 1; margin: 0 } \
.hangs { white-space: pre-wrap; background-color: red; }";

const SHAPE_CSS_NO_CLAMP: &str = ".d { width: 96px; font: 16px Ahem; margin: 0 } \
.hangs { white-space: pre-wrap; background-color: red; }";

/// 主判例：clamp:1 保留行不再以空格片段收尾（悬挂空格剪除，红带 advance 消失）。
#[test]
fn clamped_kept_line_trailing_hang_spaces_trimmed() {
    let result = compute(SHAPE_HTML, SHAPE_CSS);
    let lines = find_inline_lines(&result.root).expect("存树 IFC 应存在");
    assert_eq!(lines.len(), 1, "clamp:1 保留 1 行，实得 {lines:?}");
    let texts: Vec<&str> = lines[0].fragments.iter().map(|f| f.text.as_str()).collect();
    assert!(texts.iter().any(|t| t.contains("cd")), "内容词保留：{texts:?}");
    assert!(
        !texts.iter().any(|t| t.ends_with(' ')),
        "悬挂空格片段剪除（032 红带消失）：{texts:?}"
    );
}

/// 负向守卫：同形态去掉 line-clamp → 悬挂空格片段原样保留（不误剪非截断内容）。
#[test]
fn unclamped_keeps_trailing_spaces() {
    let result = compute(SHAPE_HTML, SHAPE_CSS_NO_CLAMP);
    let lines = find_inline_lines(&result.root).expect("存树 IFC 应存在");
    assert!(lines.len() >= 2, "无 clamp：内容折 2 行以上，实得 {} 行", lines.len());
    assert!(
        lines.iter().any(|l| l.fragments.iter().any(|f| f.text.ends_with(' '))),
        "未截断：悬挂空格片段不剪"
    );
}
