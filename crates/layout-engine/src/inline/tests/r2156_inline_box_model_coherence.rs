//! R2156 Phase A inline-box-model coherence 切片的辅助判定单测。
//!
//! 覆盖 `InlineFormattingContext::inline_elem_has_nested_inline_block`（R1576，检测 inline
//! 元素是否含嵌套 atomic inline 后代）与 `inline_subtree_has_ooflow_descendant`（R2156 ooflow
//! 守卫，检测 inline 子树是否含 abspos/fixed 后代）。二者共同 gate tree.rs 子循环中「跳过
//! inline taffy 节点」的决策，解 37-form-controls `<p><label>text <input></label></p>` 结构
//! 重叠 + 文本串联，同时保 nested-inline-abspos-child 簇的 abspos CB。

use super::super::*;
use zero_css_parser::values::{DisplayValue, PositionValue};
use zero_dom::parse_html;
use zero_style_system::ComputedStyle;

/// 构造一个 ComputedStyle，display + position 可指定，其余 default。
fn style(display: DisplayValue, position: PositionValue) -> ComputedStyle {
    let mut s = ComputedStyle::default();
    s.display = display;
    s.position = position;
    s
}

/// 从 doc 取 body 下首个元素（测试锚点）。
fn first_body_element(doc: &Document) -> NodeId {
    let html = doc.first_child(doc.root()).unwrap();
    let body = doc.last_child(html).unwrap();
    doc.first_child(body).unwrap()
}

/// 取某元素的首个**元素**子节点（跳过文本节点）。
fn first_element_child(doc: &Document, id: NodeId) -> Option<NodeId> {
    doc.child_nodes(id)
        .iter()
        .copied()
        .find(|&c| doc.get(c).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))))
}

/// 取某元素的 class 属性（缺失返回空串）。
fn class_of(doc: &Document, id: NodeId) -> String {
    doc.get(id)
        .and_then(|n| match &n.kind {
            NodeKind::Element(e) => Some(e.get_attribute("class").unwrap_or_default()),
            _ => None,
        })
        .unwrap_or_default()
}

/// R1576：`<p><label><input></label></p>` —— label（inline）含 input（inline-block）后代。
/// `inline_elem_has_nested_inline_block(label)` 须返回 true（驱动 R2156 跳过 label taffy 节点）。
#[test]
fn r2156_inline_label_wrapping_input_has_nested_atomic() {
    let doc = parse_html("<p><label>Name: <input type=\"text\"></label></p>");
    let p = first_body_element(&doc);
    let label = doc.first_child(p).unwrap();

    let mut styles = HashMap::new();
    styles.insert(label, style(DisplayValue::Inline, PositionValue::Static));
    // input 默认 display:inline-block（ua_default_display），position:static。
    if let Some(input) = first_element_child(&doc, label) {
        if doc.get(input).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))) {
            styles.insert(input, style(DisplayValue::InlineBlock, PositionValue::Static));
        }
    }

    assert!(
        InlineFormattingContext::inline_elem_has_nested_inline_block(&doc, &styles, label),
        "label 包裹 input（inline-block）应检测到嵌套 atomic inline"
    );
}

/// R1576 负例：纯文本 inline 元素（无 atomic 后代）须返回 false（保持扁平化文本，向后兼容）。
#[test]
fn r2156_plain_text_inline_has_no_nested_atomic() {
    let doc = parse_html("<p><a href=\"#\">just text</a></p>");
    let p = first_body_element(&doc);
    let a = doc.first_child(p).unwrap();

    let mut styles = HashMap::new();
    styles.insert(a, style(DisplayValue::Inline, PositionValue::Static));

    assert!(
        !InlineFormattingContext::inline_elem_has_nested_inline_block(&doc, &styles, a),
        "纯文本 inline 元素不应误判为含嵌套 atomic inline"
    );
}

/// 给 outer_span 子树中所有元素赋样式：`.inline-content` → inline-block + absolute，
/// 其余 inline 元素 → inline + relative。返回赋完样式的 HashMap（含 outer_span）。
fn styles_for_abspos_subtree(doc: &Document, outer_span: NodeId) -> HashMap<NodeId, ComputedStyle> {
    let mut styles = HashMap::new();
    styles.insert(outer_span, style(DisplayValue::Inline, PositionValue::Static));
    let mut stack = doc.child_nodes(outer_span);
    while let Some(cid) = stack.pop() {
        if doc.get(cid).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))) {
            if class_of(doc, cid).contains("inline-content") {
                styles.insert(cid, style(DisplayValue::InlineBlock, PositionValue::Absolute));
            } else {
                styles.insert(cid, style(DisplayValue::Inline, PositionValue::Relative));
            }
            stack.extend(doc.child_nodes(cid));
        }
    }
    styles
}

/// R2156 ooflow 守卫正例：inline 子树含 position:absolute 后代 → 须返回 true（保留 taffy 子树
/// 供 abspos CB，nested-inline-abspos-child 簇）。结构镜像该 reftest：inline-content 同时
/// inline-block + absolute。
#[test]
fn r2156_inline_subtree_with_abspos_descendant_detected() {
    let doc = parse_html("<div><span><span class=\"parent\"><div class=\"inline-content\"></div></span></span></div>");
    let div = first_body_element(&doc);
    let outer_span = doc.first_child(div).unwrap();
    let styles = styles_for_abspos_subtree(&doc, outer_span);

    assert!(
        InlineFormattingContext::inline_subtree_has_ooflow_descendant(&doc, &styles, outer_span),
        "含 abspos 后代的 inline 子树须被守卫检出（保留 taffy 子树供 CB）"
    );
}

/// R2156 ooflow 守卫负例：inline 子树仅含 static/relative 后代（无 abspos/fixed）→ 返回 false
/// （可安全跳过 taffy 节点）。
#[test]
fn r2156_inline_subtree_without_ooflow_not_detected() {
    let doc = parse_html("<p><label>text <input type=\"text\"></label></p>");
    let p = first_body_element(&doc);
    let label = doc.first_child(p).unwrap();

    let mut styles = HashMap::new();
    styles.insert(label, style(DisplayValue::Inline, PositionValue::Static));
    if let Some(input) = first_element_child(&doc, label) {
        if doc.get(input).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))) {
            styles.insert(input, style(DisplayValue::InlineBlock, PositionValue::Static));
        }
    }

    assert!(
        !InlineFormattingContext::inline_subtree_has_ooflow_descendant(&doc, &styles, label),
        "无 abspos/fixed 后代的 inline 子树不应触发 ooflow 守卫"
    );
}

/// 组合判定（gate 真实决策）：label 含 input（atomic）但无 ooflow → 可跳过 taffy 节点
/// （has_nested=true && ooflow=false）。这是 37-form-controls 的修复路径。
#[test]
fn r2156_gate_decision_form_control_pattern() {
    let doc = parse_html("<p><label>Name: <input type=\"text\"></label></p>");
    let p = first_body_element(&doc);
    let label = doc.first_child(p).unwrap();

    let mut styles = HashMap::new();
    styles.insert(label, style(DisplayValue::Inline, PositionValue::Static));
    if let Some(input) = first_element_child(&doc, label) {
        if doc.get(input).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))) {
            styles.insert(input, style(DisplayValue::InlineBlock, PositionValue::Static));
        }
    }

    let has_nested = InlineFormattingContext::inline_elem_has_nested_inline_block(&doc, &styles, label);
    let has_ooflow = InlineFormattingContext::inline_subtree_has_ooflow_descendant(&doc, &styles, label);
    // gate 决策 = has_nested && !has_ooflow（且 label 自身非 ooflow、horizontal-tb）。
    assert!(
        has_nested && !has_ooflow,
        "form-control 模式（atomic 后代 + 无 ooflow）应触发 gate 跳过"
    );
}

/// 组合判定（abspos 簇）：含 abspos 后代 → 守卫阻止跳过（gate_skip=false），保 CB。
#[test]
fn r2156_gate_decision_blocks_when_ooflow_present() {
    let doc = parse_html("<div><span><span class=\"parent\"><div class=\"inline-content\"></div></span></span></div>");
    let div = first_body_element(&doc);
    let outer_span = doc.first_child(div).unwrap();
    let styles = styles_for_abspos_subtree(&doc, outer_span);

    let has_nested = InlineFormattingContext::inline_elem_has_nested_inline_block(&doc, &styles, outer_span);
    let has_ooflow = InlineFormattingContext::inline_subtree_has_ooflow_descendant(&doc, &styles, outer_span);
    // gate_skip = has_nested && !has_ooflow；含 abspos 后代时须为 false（守卫阻止跳过保 CB）。
    assert!(
        !has_nested || has_ooflow,
        "含 abspos 后代时守卫须阻止 gate 跳过（保 CB）"
    );
}
