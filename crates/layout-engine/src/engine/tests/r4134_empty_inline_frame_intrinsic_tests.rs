//! R4134（CSS2 §10.3.9 + inline 盒模型）：空 display:Inline 子带 frame（padding/border）
//! 时对父盒 max-content 的贡献 = frame，而非 R1298 的一概 0。
//!
//! 空元素子带水平 padding（如 WPT word-spacing-characters-001 ref 页
//! `.spacer { padding-left: 4em }` 的空嵌套 span）其盒几何含 padding——chromium
//! 实测该 spacer 撑开 4em。旧实现贡献 0 → 带 background 的 inline 父
//! （R372 shrink-to-fit 臂）测得 intrinsic 0 → R4033 guard（content==0 且
//! frame==0）跳过收缩 → 外层 span 保持 taffy 拉伸满宽（蓝条画满 767px）。
//! 纯空 span（frame=0）维持 R1298 的 0 贡献，R1298 锚（inline-block-baseline-015）
//! 行为不变。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str, css: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// 带 bg 的 inline 父含「空 padded inline 子」：父应收缩到内容宽（文本 + 子 frame），
/// 不保持 taffy 拉伸满宽。driving: word-spacing-characters-001 ref 页形态。
#[test]
fn r4134_bg_inline_with_empty_padded_child_shrinks() {
    let html = r#"<html><body style="margin:0">
    <p style="margin:1em; font-family: monospace; font-size:16px">
      <span id="outer" style="background: blue; color: blue">A <span style="padding-left: 64px"></span>B</span>
    </p></body></html>"#;
    let (doc, result) = layout(html, "");
    let outer = doc
        .get_elements_by_tag_name("span")
        .first()
        .copied()
        .expect("outer span");
    let (w, _h) = find_box(&result.root, outer).expect("outer span box");
    // 内容 = "A B"（~21.6px @ 16px mono）+ spacer padding 64 ≈ 85px。
    // 旧实现：intrinsic 0 → 不收缩 → 768（拉伸满宽）。
    assert!(
        w < 200.0,
        "R4134: 带 bg 的 inline 父应收缩到内容宽（~85），实际 {w}（拉伸满宽 = 修复未生效）"
    );
    assert!(
        w > 60.0,
        "R4134: 收缩目标须含 spacer 的 padding（>60），实际 {w}（frame 贡献丢失）"
    );
}

/// 纯空 inline（无 frame）贡献仍为 0（R1298 行为面锚定）——不因本修复引入
/// 拉伸宽回归（inline-block-baseline-015 语境）。
#[test]
fn r4134_pure_empty_inline_still_contributes_zero() {
    let html = r#"<html><body style="margin:0">
    <div style="width:600px">
      <div style="display: inline-block">
        <span></span>
        <div style="width:100px;height:50px"></div>
      </div>
    </div></body></html>"#;
    let (doc, result) = layout(html, "");
    let divs = doc.get_elements_by_tag_name("div");
    let ib = *divs.last().expect("inner inline-block div");
    let (w, _h) = find_box(&result.root, ib).expect("inline-block box");
    assert!(
        (w - 100.0).abs() < 1.0,
        "R1298 锚：纯空 span（frame=0）贡献 0，max-content 应 = block 子 100，实际 {w}"
    );
}
