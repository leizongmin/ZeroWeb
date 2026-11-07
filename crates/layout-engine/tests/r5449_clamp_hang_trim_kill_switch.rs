//! R5449 kill-switch 回退锚（独立测试进程——`ZW_CLAMP_HANG_TRIM=0` 时 clamp 截断
//! 不剪末保留行 hangable 尾随空格，032 红带残留复现；进程内 set_var 与并行单测
//! 互踩，R3858 教训——本文件仅此一测，独占进程环境）。
//!
//! 谱系与几何同 r5449_clamp_hang_trim.rs 主判例（032 形态：内容词 + pre-wrap
//! 悬挂空格 span + 被截断的 "More"）。

use zero_layout_engine::types::{InlineLayoutLine, LayoutBox};
use zero_layout_engine::{LayoutEngine, LayoutResult};

/// 收集（首个）存树 IFC 行列表。
fn find_inline_lines(node: &LayoutBox) -> Option<&Vec<InlineLayoutLine>> {
    if let Some(lines) = &node.inline_layout {
        return Some(lines);
    }
    node.children.iter().find_map(find_inline_lines)
}

#[test]
fn kill_switch_restores_hang_space_fragments() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；OnceLock 首读前生效。
    unsafe { std::env::set_var("ZW_CLAMP_HANG_TRIM", "0") };

    let html = r#"<html><body>
<div class="d"><span>ab cd</span><span class="hangs">    </span>More</div>
</body></html>"#;
    let css = ".d { width: 96px; font: 16px Ahem; line-clamp: 1; margin: 0 } \
.hangs { white-space: pre-wrap; background-color: red; }";
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result: LayoutResult = engine.compute(&doc, &styles);
    let lines = find_inline_lines(&result.root).expect("存树 IFC 应存在");
    assert_eq!(lines.len(), 1, "clamp:1 保留 1 行，实得 {lines:?}");
    let texts: Vec<&str> = lines[0].fragments.iter().map(|f| f.text.as_str()).collect();
    assert!(
        texts.iter().any(|t| t.ends_with(' ')),
        "kill-switch=0 恢复旧行为：悬挂空格片段保留（032 红带残留），实得 {texts:?}"
    );
}
