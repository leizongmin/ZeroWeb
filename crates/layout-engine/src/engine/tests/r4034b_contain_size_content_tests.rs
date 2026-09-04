//! R4034b：`contain: size` 内容测量抑制回归测试。
//!
//! CSS Containment 1 §3：contain:size 使元素按「无内容」sized。R4034 修正 nbsp 空白判定后
//! （nbsp 是 preserved 内容），三处内容测量路径须同步被 containment 抑制，否则 contained
//! 元素被 nbsp 行高/advance 撑大：
//! ① measure_text_content（taffy leaf 测量）——返回 CIS 替代（无 CIS = 0）；
//! ② remeasure_inline_only_containers 的 DOM 文本重测——不回填内容行高；
//! ③ apply_abspos_shrink_to_fit_width 的补测——不把内容 max-content 拉回。
//! driving：css-contain/contain-animation-001（contain:strict abspos div 的 nbsp 内容
//! → 216 高红底露出，应 100×100 纯边框）。

use std::sync::Arc;

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn layout(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    let root = Arc::try_unwrap(r.root).unwrap_or_else(|arc| (*arc).clone());
    (doc, root)
}

fn find_div<'a>(root: &'a LayoutBox, doc: &zero_dom::Document) -> &'a LayoutBox {
    fn walk<'a>(b: &'a LayoutBox, doc: &zero_dom::Document) -> Option<&'a LayoutBox> {
        if let Some(nid) = b.node_id
            && let Some(n) = doc.get(nid)
            && matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "div")
        {
            return Some(b);
        }
        b.children.iter().find_map(|c| walk(c, doc))
    }
    walk(root, doc).expect("div box")
}

/// contain:strict 块级 div 仅含 nbsp：高度 = 纯 padding+border（内容不贡献）。
/// R4034 前 nbsp 被 trim() 误判无内容而巧合正确；R4034 后须由 containment gate 维持。
#[test]
fn r4034b_contain_strict_block_nbsp_no_content_contribution() {
    let (doc, root) = layout(
        r#"<html><body><div style="contain: strict; font-size: 100px; border: 10px solid;">&nbsp;</div></body></html>"#,
    );
    let div = find_div(&root, &doc);
    // 内容高 0（无 CIS）→ border-box 高 = 20（上下边框）。
    assert!(
        div.height < 21.0,
        "contain:strict 块的 nbsp 内容不应贡献高度（应为 20 纯边框），got {}",
        div.height
    );
}

/// contain:size + contain-intrinsic-size：测量面返回 CIS 替代值（css-sizing-4
/// §intrinsic-size-override 的测量面接线，与 R4008 converter 面同语义）。
#[test]
fn r4034b_contain_size_uses_cis_as_content_measurement() {
    let (doc, root) = layout(
        r#"<html><body><div style="contain: size; contain-intrinsic-height: 50px; font-size: 100px;">&nbsp;</div></body></html>"#,
    );
    let div = find_div(&root, &doc);
    assert!(
        (div.height - 50.0).abs() < 1.0,
        "contain:size + CIS 50px → 内容测量 = CIS 50，got {}",
        div.height
    );
}
