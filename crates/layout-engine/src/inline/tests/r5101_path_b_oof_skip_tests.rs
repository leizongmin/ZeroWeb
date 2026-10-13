//! R5101（CSS2 §9.7）：paint Path B（空 styles）的 out-of-flow 直接子 skip。
//!
//! Path B 重建 IFC 时 styles 为空，collect 的 OOF skip 臂（读 ComputedStyle.position）
//! 不可达——abspos/fixed 直接子被当 inline 盒收入行内流推进行宽
//!（run-in-contains-abspos-001：abspos header 推进 ~47px，后继文本右移一行宽）。
//! 修复 = 布局期分类的 `oof_child_nodes` 注入集（镜像 float_child_nodes slice10 模式），
//! collect 空 styles 臂查集跳过。本文件在 collect 层标定两臂语义。

use super::super::*;

/// 拼接 Text run 文本（断言用）。
fn items_text(items: &[InlineItem]) -> String {
    let mut s = String::new();
    for it in items {
        if let InlineItem::Text(r) = it {
            s.push_str(&r.text);
        }
    }
    s
}

/// 注入集消费：空 styles + oof_child_nodes 含 abspos 子 → 其文本不入行内流，
/// 普通 inline 子文本保留（真值源注入生效）。
#[test]
fn r5101_path_b_oof_set_skips_abspos_child() {
    let html = r#"<html><body><div id="c">Before <span id="abs" style="position: absolute">AbsText</span><span id="norm">NormText</span></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let container = doc.get_element_by_id("c").expect("container div");
    let abs_id = doc.get_element_by_id("abs").expect("abspos span");

    let mut ctx = InlineFormattingContext::new(800.0);
    let mut oof = std::collections::HashSet::new();
    oof.insert(abs_id);
    ctx.set_oof_child_nodes(oof);
    // 空 styles = paint Path B 重建形态（painter ctx.layout(doc, node, &HashMap::new())）。
    let text = items_text(&ctx.collect_inline_items(&doc, container, &HashMap::new()));
    assert!(text.contains("NormText"), "普通 inline 子文本须保留，got {text:?}");
    assert!(
        !text.contains("AbsText"),
        "R5101: 注入集内 abspos 子文本不得入行内流（CSS2 §9.7 out of flow），got {text:?}"
    );
}

/// 有 styles 臂回归守卫：无注入集时 styled OOF skip 照常生效（abspos 子按
/// ComputedStyle.position 跳过）——两臂共享 skip 出口，注入集只补 Path B 空表。
#[test]
fn r5101_styled_arm_skips_abspos_without_injection() {
    let html = r#"<html><body><div id="c">Before <span id="abs" style="position: absolute">AbsText</span><span id="norm">NormText</span></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let container = doc.get_element_by_id("c").expect("container div");

    let ctx = InlineFormattingContext::new(800.0);
    let text = items_text(&ctx.collect_inline_items(&doc, container, &styles));
    assert!(text.contains("NormText"), "普通 inline 子文本须保留，got {text:?}");
    assert!(
        !text.contains("AbsText"),
        "styled 臂须按 position:absolute 跳过（既有 R1682 语义），got {text:?}"
    );
}
