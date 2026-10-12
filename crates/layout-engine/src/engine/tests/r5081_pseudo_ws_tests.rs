//! R5081（css-content-3 × css-text-3）：伪元素合成文本节点的 run 级 white-space
//! 读**文本节点自身** styles 条目优先。
//!
//! `inject_pseudo_text_nodes`（zero-engine/pipeline，生产管线在布局前调用）把伪元素
//! 自身 ComputedStyle 挂在注入文本节点的 styles 条目上
//! （`styles.insert(new_id, pseudo_style)`）——`::before { content: "a\Ab";
//! white-space: nowrap }` 的 `\A` 须按 nowrap 折叠且窄容器不软折行
//! （content-white-space-003）；`white-space: pre` 时 `\A` 保留为强制断行
//! （content-white-space-002）。旧实现 run white-space 只读父级样式，伪元素自身的
//! white-space 声明被丢弃（003 在 4em 容器内折成多行 / 002 强制断行丢失）。
//! 普通 DOM 文本节点无自身条目 → 落回父级继承（R3778 原行为不变，回归臂锚定）。
//! 注：本层直接构造 IFC——伪元素注入发生在 zero-engine 管线，测试用与
//! `inject_pseudo_text_nodes` 相同的「文本节点 + 自身 styles 条目」形态驱动。

use crate::inline::InlineFormattingContext;
use std::collections::HashMap;
use zero_dom::{Document, NodeId, NodeKind, parse_html};
use zero_style_system::{ComputedStyle, WhiteSpaceValue};

fn style_with_ws(ws: WhiteSpaceValue) -> ComputedStyle {
    let mut s = ComputedStyle::default();
    s.white_space = ws;
    s
}

/// 定位 body 下第一个元素（`<div>` 宿主）。
fn host_element(doc: &Document) -> NodeId {
    let html = doc.first_child(doc.root()).expect("html");
    let body = doc.last_child(html).expect("body");
    doc.child_nodes(body)
        .into_iter()
        .find(|&c| doc.get(c).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))))
        .expect("host element")
}

/// 模拟 inject_pseudo_text_nodes 文本路径：注入 content 文本节点并把伪元素
/// 样式挂到该文本节点的 styles 条目上。
fn inject_pseudo_text(doc: &mut Document, host: NodeId, text: &str) -> NodeId {
    let text_id = doc.create_text_node(text);
    let _ = doc.append_child(host, text_id);
    text_id
}

fn fragments_text(ctx: &InlineFormattingContext) -> String {
    ctx.all_fragments().iter().map(|f| f.text.clone()).collect()
}

/// content-white-space-003：`::before` 自身 `white-space: nowrap`——`\A` 折叠为
/// 空格，且窄容器（40px，normal 语义必折行）不软折行 → 恒 1 行。
#[test]
fn r5081_pseudo_text_node_nowrap_owns_white_space() {
    let mut doc = parse_html("<div></div>");
    let host = host_element(&doc);
    let text_id = inject_pseudo_text(&mut doc, host, "This text\n\t\tshould be on\n    \tone\n        line.");

    let mut styles = HashMap::new();
    styles.insert(text_id, style_with_ws(WhiteSpaceValue::Nowrap));

    let mut ctx = InlineFormattingContext::new(40.0);
    ctx.layout(&doc, host, &styles);

    let text = fragments_text(&ctx);
    assert!(!text.contains('\n'), "nowrap 语义 \\A 折叠为空格，实际: {text:?}");
    assert_eq!(ctx.lines.len(), 1, "nowrap 窄容器不软折行，实际 {} 行", ctx.lines.len());
}

/// content-white-space-002：`::before` 自身 `white-space: pre`——`\A` 保留为
/// 强制断行 → 4 行。
#[test]
fn r5081_pseudo_text_node_pre_owns_white_space() {
    let mut doc = parse_html("<div></div>");
    let host = host_element(&doc);
    let text_id = inject_pseudo_text(&mut doc, host, "This text\n\tshould be on\nfour\nlines.");

    let mut styles = HashMap::new();
    styles.insert(text_id, style_with_ws(WhiteSpaceValue::Pre));

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, host, &styles);

    assert_eq!(
        ctx.lines.len(),
        4,
        "pre 语义 \\A 强制断行 → 4 行，实际 {} 行",
        ctx.lines.len()
    );
    // \n 是断行标记不渲染为字形（chromium 同语义）——按行内容断言。
    let line_text = |i: usize| -> String { ctx.lines[i].runs.iter().map(|f| f.text.clone()).collect() };
    assert!(line_text(0).contains("This text"), "行 1 内容，实际 {:?}", line_text(0));
    assert!(line_text(3).contains("lines."), "行 4 内容，实际 {:?}", line_text(3));
}

/// 回归臂（R3778 原行为）：普通 DOM 文本节点**无自身条目** → 落回父级继承。
/// 父 pre + 文本 `\n` 仍强制断行。
#[test]
fn r5081_plain_text_node_falls_back_to_parent_ws() {
    let doc = parse_html("<div>ab\ncd</div>");
    let host = host_element(&doc);

    let mut styles = HashMap::new();
    styles.insert(host, style_with_ws(WhiteSpaceValue::Pre));

    let mut ctx = InlineFormattingContext::new(800.0);
    ctx.layout(&doc, host, &styles);

    assert_eq!(
        ctx.lines.len(),
        2,
        "父级 pre 继承：\\n 强制断行 → 2 行，实际 {} 行",
        ctx.lines.len()
    );
}

/// 回归臂：普通 DOM 文本节点 + 父 nowrap——折叠且窄容器不软折行（与伪元素
/// 自身条目路径同语义，确认 fallback 路径未被收窄破坏）。
#[test]
fn r5081_plain_text_node_parent_nowrap_no_wrap() {
    let doc = parse_html("<div>This text should be on one line.</div>");
    let host = host_element(&doc);

    let mut styles = HashMap::new();
    styles.insert(host, style_with_ws(WhiteSpaceValue::Nowrap));

    let mut ctx = InlineFormattingContext::new(40.0);
    ctx.layout(&doc, host, &styles);

    assert_eq!(
        ctx.lines.len(),
        1,
        "父级 nowrap 继承：窄容器 1 行，实际 {} 行",
        ctx.lines.len()
    );
}
