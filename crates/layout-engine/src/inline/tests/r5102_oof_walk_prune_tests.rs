//! R5102（CSS2 §9.7）：扁平化 walk / text 收集的 out-of-flow **后代**剪枝。
//!
//! run-in 前缀里 in-flow inline 元素内的 abspos/fixed 后代，其文本此前被
//! `collect_flat_inline_children` / `collect_text_excluding` 吸收进 run 推进行宽
//!（run-in-contains-inline-006：隐藏 span 内孙代 abspos 的 "header" 幽灵推进
//! 47px）。修复 = 双通道剪枝（display:none PR #39 先例）：layout 趟 styles 直判；
//! paint Path B（空 styles）由 `oof_walk_nodes` 注入集判定。本文件在 collect 层
//! 标定两通道语义。

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

/// 场景 = contains-inline-006 形态：in-flow span 内 abspos 孙代。
fn doc_with_grandchild_abspos() -> zero_dom::Document {
    zero_dom::parse_html(
        r#"<html><body><div id="c">Run-in <span id="outer"><span id="abs" style="position: absolute">AbsText</span></span><span id="norm">NormText</span></div></body></html>"#,
    )
}

/// styled 通道（layout 趟）：有 styles 时 abspos 孙代文本不吸收。
#[test]
fn r5102_styled_channel_prunes_grandchild_abspos_in_walk() {
    let doc = doc_with_grandchild_abspos();
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let container = doc.get_element_by_id("c").expect("container div");

    let ctx = InlineFormattingContext::new(800.0);
    // 走 collect_flat_inline_children 同参路径：主 collect 对含元素子的 inline 发 walk。
    let text = items_text(&ctx.collect_inline_items(&doc, container, &styles));
    assert!(text.contains("NormText"), "in-flow 文本须保留，got {text:?}");
    assert!(
        !text.contains("AbsText"),
        "R5102: abspos 孙代文本不得经 walk 吸收（CSS2 §9.7 out of flow），got {text:?}"
    );
}

/// 注入集通道（paint Path B）：空 styles + oof_walk_nodes 含 abspos 孙代 → 同剪。
#[test]
fn r5102_injected_set_prunes_grandchild_abspos_in_walk() {
    let doc = doc_with_grandchild_abspos();
    let container = doc.get_element_by_id("c").expect("container div");
    let abs_id = doc.get_element_by_id("abs").expect("abspos span");

    let mut ctx = InlineFormattingContext::new(800.0);
    let mut oof = NodeIdSet::default();
    oof.insert(abs_id);
    ctx.oof_walk_nodes = oof;
    // 空 styles = paint Path B 重建形态（painter ctx.layout(doc, node, &HashMap::new())）。
    let text = items_text(&ctx.collect_inline_items(&doc, container, &HashMap::new()));
    assert!(text.contains("NormText"), "in-flow 文本须保留，got {text:?}");
    assert!(
        !text.contains("AbsText"),
        "R5102: 注入集内 abspos 孙代文本不得吸收，got {text:?}"
    );
}

/// collect_text_excluding 通道（flatten run）：abspos 后代子树文本排除。
#[test]
fn r5102_collect_text_excluding_prunes_oof_subtree() {
    let doc = doc_with_grandchild_abspos();
    let outer = doc.get_element_by_id("outer").expect("outer span");
    // styled 通道：styles 直判剪枝。
    let empty = NodeIdSet::default();
    let text = InlineFormattingContext::collect_text_excluding(
        &doc,
        outer,
        &["rt", "rp", "rtc"],
        &HashMap::new(),
        &empty,
        &empty,
    );
    // 空 styles + 空注入集 = 双通道皆不剪（Path B 无注入旧行为）——本断言只定界
    //「无信号不误剪」。
    assert!(
        text.contains("AbsText"),
        "无 styles 且无注入集时不剪（行为保持），got {text:?}"
    );

    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let text =
        InlineFormattingContext::collect_text_excluding(&doc, outer, &["rt", "rp", "rtc"], &styles, &empty, &empty);
    assert!(
        !text.contains("AbsText"),
        "R5102: styled 通道剪 abspos 子树文本，got {text:?}"
    );
}
