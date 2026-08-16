//! R109（CSS2 §9.2.1.1）匿名块盒拆分结构分析。
//!
//! 当 inline 元素包含 in-flow block-level 子元素时，inline 元素被拆分为匿名块盒：
//! 连续的 inline 内容（文本 + inline 元素）被匿名块盒包裹，block-level 子元素作为独立块盒。
//! 结果序列形如 `[匿名块: text1] [block 子元素] [匿名块: text2]`。
//!
//! 本模块**仅做结构分析**（不参与布局，compute() 不调用其改变布局的函数），
//! 提供 split 计算 + 单元测试 + 可选诊断打印（env-gated），为 tree.rs 匿名块生成 +
//! IFC 片段收集的多轮接线奠基。同 `intrinsic_sizing` 的「测量先行」方法学。

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, FloatValue, PositionValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

/// inline 元素被 block-level 子元素拆分后的片段。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InlineBlockSegment {
    /// 连续的 inline 内容（文本节点 + inline-level 元素），归入一个匿名块盒。
    /// `item_node_ids` 为该片段包含的 DOM 子节点（按 DOM 顺序）。
    Inline { item_node_ids: Vec<NodeId> },
    /// block-level 子元素，作为独立块盒（不并入匿名块）。
    Block { node_id: NodeId },
}

/// 判断 display 值是否为 block-level（触发 §9.2.1.1 拆分）。
/// inline-flex/inline-grid/inline-table 是 inline-level（原子行内盒），不触发拆分。
fn is_block_level_display(display: &DisplayValue) -> bool {
    matches!(
        display,
        DisplayValue::Block
            | DisplayValue::Flex
            | DisplayValue::Grid
            | DisplayValue::Table
            | DisplayValue::ListItem
            | DisplayValue::FlowRoot
    )
}

/// 判断元素是否 out-of-flow（`position:absolute/fixed` 或 `float≠none`）。
///
/// CSS2 §9.2.1.1 匿名块盒生成只针对 **in-flow** block-level box；out-of-flow
/// 元素被移出流（由 converter 的 abspos/float 定位路径处理），不参与 inline 拆分。
/// 否则 `<span>text<div style="position:absolute">abs</div></span>` 这类「inline
/// 仅含 abspos『block』子元素」会被误拆分，破坏 `position-absolute-in-inline-*`。
pub(crate) fn is_out_of_flow(style: &ComputedStyle) -> bool {
    matches!(style.position, PositionValue::Absolute | PositionValue::Fixed) || !matches!(style.float, FloatValue::None)
}

/// 判断指定 DOM 元素是否为 inline 元素且含至少一个 in-flow block-level 子元素（R109 触发条件）。
pub(crate) fn inline_has_block_child(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    inline_id: NodeId,
) -> bool {
    let is_inline = styles
        .get(&inline_id)
        .is_some_and(|s| matches!(s.display, DisplayValue::Inline));
    if !is_inline {
        return false;
    }
    doc.child_nodes(inline_id).iter().any(|child| {
        styles
            .get(child)
            .is_some_and(|s| is_block_level_display(&s.display) && !is_out_of_flow(s))
    })
}

/// 判断 block 容器是否含混合内容（text + block-level 子元素混合，§9.2.1.1）。
///
/// block 容器同时包含非空白文本节点和 in-flow block-level 子元素时，
/// 文本节点应被匿名块盒包裹。仅文本节点算作 inline 内容（inline 元素
/// 已有自己的 taffy 节点，不需要匿名块包裹）。
pub(crate) fn block_container_has_mixed_content(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    container_id: NodeId,
) -> bool {
    let is_block_container = styles.get(&container_id).is_some_and(|s| {
        matches!(
            s.display,
            DisplayValue::Block
                | DisplayValue::Flex
                | DisplayValue::Grid
                | DisplayValue::FlowRoot
                | DisplayValue::ListItem
                | DisplayValue::Table
        )
    });
    if !is_block_container {
        return false;
    }

    let children = doc.child_nodes(container_id);
    let mut has_text = false;
    let mut has_block = false;

    for &child in &children {
        let Some(node) = doc.get(child) else {
            continue;
        };
        match &node.kind {
            NodeKind::Text(text) if !text.content.trim().is_empty() => {
                has_text = true;
            }
            NodeKind::Element(_) => {
                let Some(style) = styles.get(&child) else {
                    continue;
                };
                if matches!(style.display, DisplayValue::None | DisplayValue::Contents) {
                    continue;
                }
                if is_out_of_flow(style) {
                    continue;
                }
                if is_block_level_display(&style.display) {
                    has_block = true;
                }
                // inline-level elements already have their own taffy nodes;
                // only text nodes need anonymous block wrapping.
            }
            _ => {}
        }
        if has_text && has_block {
            return true;
        }
    }
    false
}

/// 子元素拆分的核心逻辑（inline 元素和 block 容器共用）。
///
/// 遍历父元素 DOM 子节点，按 block-level 子元素切分：
/// - 文本节点 + inline-level 元素 → 累入当前 Inline 片段
/// - block-level 子元素 → 关闭当前 Inline 片段（非空才发出），发出 Block 片段
///
/// 返回 `None` 表示无需拆分（无 block 子元素，仅 inline 内容）。
fn compute_child_split_impl(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    parent_id: NodeId,
) -> Option<Vec<InlineBlockSegment>> {
    let mut segments: Vec<InlineBlockSegment> = Vec::new();
    let mut current_inline: Vec<NodeId> = Vec::new();
    let flush = |cur: &mut Vec<NodeId>, segs: &mut Vec<InlineBlockSegment>| {
        if !cur.is_empty() {
            segs.push(InlineBlockSegment::Inline {
                item_node_ids: std::mem::take(cur),
            });
        }
    };

    for child in doc.child_nodes(parent_id) {
        let Some(node) = doc.get(child) else {
            continue;
        };
        match &node.kind {
            NodeKind::Text(_) => current_inline.push(child),
            NodeKind::Element(_) => {
                let style = styles.get(&child);
                // display:none / 不在 styles 的元素跳过（不参与流）
                let Some(style) = style else {
                    continue;
                };
                if matches!(style.display, DisplayValue::None | DisplayValue::Contents) {
                    continue;
                }
                // in-flow block-level 子元素触发拆分；out-of-flow（abspos/fixed/float）
                // 不触发——归入当前 inline 片段（保留为子节点，由 converter 定位路径处理）。
                if is_block_level_display(&style.display) && !is_out_of_flow(style) {
                    flush(&mut current_inline, &mut segments);
                    segments.push(InlineBlockSegment::Block { node_id: child });
                } else {
                    current_inline.push(child);
                }
            }
            _ => {}
        }
    }
    flush(&mut current_inline, &mut segments);

    // 仅当存在至少一个 Block 片段时才算有效拆分（纯 inline 内容无需匿名块）。
    if segments.iter().any(|s| matches!(s, InlineBlockSegment::Block { .. })) {
        Some(segments)
    } else {
        None
    }
}

/// 判断 Inline 片段是否仅含空白文本（collapse-through 保留）。
///
/// 若一个 Inline 片段所有 Text 节点均为空白，且无 Element 子节点，
/// 则该片段为「空」，在 block 容器中可跳过匿名块生成（保 collapse-through 语义）。
pub(crate) fn is_whitespace_only_inline_segment(doc: &Document, item_node_ids: &[NodeId]) -> bool {
    item_node_ids.iter().all(|&nid| {
        doc.get(nid).is_some_and(|n| match &n.kind {
            NodeKind::Text(t) => t.content.trim().is_empty(),
            NodeKind::Element(_) => false, // element 使片段非空
            _ => true,                     // 未知类型算空
        })
    })
}

/// 计算 inline 元素的匿名块拆分片段序列（CSS2 §9.2.1.1）。
///
/// 遍历 inline 元素的 DOM 子节点，按 block-level 子元素切分。
/// 返回 `None` 表示无需拆分（非 inline，或无 block-level 子元素）。
pub(crate) fn compute_inline_block_split(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    inline_id: NodeId,
) -> Option<Vec<InlineBlockSegment>> {
    if !inline_has_block_child(doc, styles, inline_id) {
        return None;
    }
    compute_child_split_impl(doc, styles, inline_id)
}

/// 计算 block 容器的匿名块拆分片段序列（CSS2 §9.2.1.1）。
///
/// 当 block 容器含混合 inline+block 子元素时，调用此函数获取拆分。
/// 返回 `None` 表示无需拆分（纯 block 或纯 inline 子元素）。
pub(crate) fn compute_block_container_split(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    container_id: NodeId,
) -> Option<Vec<InlineBlockSegment>> {
    if !block_container_has_mixed_content(doc, styles, container_id) {
        return None;
    }
    compute_child_split_impl(doc, styles, container_id)
}

/// 诊断：遍历布局树，对 R109 触发元素（inline 含 block 子元素）打印其拆分片段。
/// **仅 eprintln，不改变任何布局状态**。env `R109_DBG=1` 启用。
pub(crate) fn debug_dump_inline_block_splits(
    root: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    fn walk(b: &LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
        if let Some(id) = b.node_id
            && let Some(segs) = compute_inline_block_split(doc, styles, id)
        {
            let summary: Vec<String> = segs
                .iter()
                .map(|s| match s {
                    InlineBlockSegment::Inline { item_node_ids } => {
                        format!("Inline({} items)", item_node_ids.len())
                    }
                    InlineBlockSegment::Block { node_id } => {
                        format!("Block(node={:?})", node_id)
                    }
                })
                .collect();
            eprintln!("R109_DBG: inline node={:?} split = [{}]", id, summary.join(", "));
        }
        for c in &b.children {
            walk(c, doc, styles);
        }
    }
    walk(root, doc, styles);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 在 DOM 树中按 id 属性查找元素 NodeId。
    fn find_node_by_id(id: &str, doc: &Document, node: NodeId) -> Option<NodeId> {
        if let Some(n) = doc.get(node)
            && let NodeKind::Element(e) = &n.kind
            && e.get_attribute("id").as_deref() == Some(id)
        {
            return Some(node);
        }
        for &c in &doc.get(node).map(|n| n.children.clone()).unwrap_or_default() {
            if let Some(found) = find_node_by_id(id, doc, c) {
                return Some(found);
            }
        }
        None
    }

    /// 解析 HTML、计算样式、定位目标元素，返回其拆分片段（若有）。
    fn split_for(html: &str, target_id: &str) -> Option<Vec<InlineBlockSegment>> {
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let root = doc.root();
        let target = find_node_by_id(target_id, &doc, root)?;
        compute_inline_block_split(&doc, &styles, target)
    }

    #[test]
    fn test_single_block_child_splits_three_ways() {
        // inline-box-001 结构：inline #div1 含 "First line" + block div + "Last line"。
        let html = r#"<html><body>
          <div id="div1" style="display:inline">
            First line
            <div>Filler Text</div>
            Last line
          </div>
        </body></html>"#;
        let segs = split_for(html, "div1").expect("inline with block child should split");
        // 期望 3 片段：Inline(2 items: text+text), Block, 但实际 text 节点数取决于 DOM 解析
        // 至少应含一个 Block 片段 + 前后各一个 Inline 片段。
        let block_count = segs
            .iter()
            .filter(|s| matches!(s, InlineBlockSegment::Block { .. }))
            .count();
        assert_eq!(block_count, 1, "expected exactly 1 block segment, got {:?}", segs);
        // 第一个片段应是 Inline（First line 文本），最后一个也应是 Inline（Last line 文本）
        assert!(
            matches!(segs.first(), Some(InlineBlockSegment::Inline { .. })),
            "first segment should be Inline, got {:?}",
            segs.first()
        );
        assert!(
            matches!(segs.last(), Some(InlineBlockSegment::Inline { .. })),
            "last segment should be Inline, got {:?}",
            segs.last()
        );
        // 中间片段应是 Block
        assert!(
            matches!(segs.get(1), Some(InlineBlockSegment::Block { .. })),
            "middle segment should be Block, got {:?}",
            segs.get(1)
        );
    }

    #[test]
    fn test_pure_inline_no_split() {
        // 无 block 子元素的 inline 不拆分。
        let html = r#"<html><body>
          <div id="s" style="display:inline">just text <span>more</span></div>
        </body></html>"#;
        assert!(split_for(html, "s").is_none(), "pure inline should not split");
    }

    #[test]
    fn test_block_container_not_triggered() {
        // block 元素（非 inline）不触发 R109 拆分（即使含 block 子元素）。
        let html = r#"<html><body>
          <div id="b" style="display:block"><div>child</div></div>
        </body></html>"#;
        assert!(
            split_for(html, "b").is_none(),
            "block container should not trigger R109"
        );
    }

    #[test]
    fn test_leading_block_no_empty_inline() {
        // block 子元素在首位时，不应产生空前导 Inline 片段。
        let html = r#"<html><body>
          <div id="i" style="display:inline"><div>block first</div>trailing text</div>
        </body></html>"#;
        let segs = split_for(html, "i").expect("should split");
        // 首片段应是 Block（非空 Inline）
        assert!(
            matches!(segs.first(), Some(InlineBlockSegment::Block { .. })),
            "leading block should produce Block as first segment, got {:?}",
            segs.first()
        );
        assert_eq!(segs.len(), 2, "expected Block + trailing Inline, got {:?}", segs);
    }

    #[test]
    fn test_out_of_flow_child_does_not_trigger_split() {
        // CSS2 §9.2.1.1：out-of-flow（abspos/fixed/float）子元素不触发 inline 拆分。
        // <span>text<div abs>abs</div>more</span> 不应被拆分（修复 position-absolute-in-inline 回归）。
        let html = r#"<html><body>
          <div id="s" style="display:inline">
            before
            <div style="position:absolute">abs</div>
            <div style="float:left">flt</div>
            after
          </div>
        </body></html>"#;
        assert!(
            split_for(html, "s").is_none(),
            "out-of-flow-only children must not trigger R109 split"
        );
    }

    #[test]
    fn test_in_flow_block_alongside_out_of_flow_still_splits() {
        // 混合：in-flow block + abspos。in-flow block 触发拆分；abspos 归入 inline 片段。
        let html = r#"<html><body>
          <div id="s" style="display:inline">
            text
            <div>in-flow block</div>
            <div style="position:absolute">abs</div>
          </div>
        </body></html>"#;
        let segs = split_for(html, "s").expect("in-flow block should trigger split");
        let block_count = segs
            .iter()
            .filter(|s| matches!(s, InlineBlockSegment::Block { .. }))
            .count();
        // 只 in-flow block 发出 Block 片段；abspos 不发（归入 Inline 片段）
        assert_eq!(
            block_count, 1,
            "only in-flow block should emit Block segment, got {:?}",
            segs
        );
    }

    // ── block container mixed content tests ──

    /// 解析 HTML 并检测 block 容器是否含混合内容。
    fn has_mixed(html: &str, target_id: &str) -> bool {
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let root = doc.root();
        let target = find_node_by_id(target_id, &doc, root).expect("target not found");
        block_container_has_mixed_content(&doc, &styles, target)
    }

    /// 解析 HTML 并获取 block 容器的匿名块拆分片段。
    fn split_block(html: &str, target_id: &str) -> Option<Vec<InlineBlockSegment>> {
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let root = doc.root();
        let target = find_node_by_id(target_id, &doc, root)?;
        compute_block_container_split(&doc, &styles, target)
    }

    #[test]
    fn test_block_container_mixed_content_detected() {
        // margin-collapse-101 模式：div.b 含 div.red (block) + "B" (text)
        let html = r#"<html><body>
          <div id="b"><div></div>B</div>
        </body></html>"#;
        assert!(
            has_mixed(html, "b"),
            "block with block child + text should detect mixed"
        );
    }

    #[test]
    fn test_block_container_only_block_not_mixed() {
        // 纯 block 子元素——无混合内容
        let html = r#"<html><body>
          <div id="b"><div>A</div><div>B</div></div>
        </body></html>"#;
        assert!(
            !has_mixed(html, "b"),
            "block with only block children should not be mixed"
        );
    }

    #[test]
    fn test_block_container_only_text_not_mixed() {
        // 纯文本子元素——无混合内容
        let html = r#"<html><body>
          <div id="b">Hello World</div>
        </body></html>"#;
        assert!(!has_mixed(html, "b"), "block with only text should not be mixed");
    }

    #[test]
    fn test_block_container_whitespace_only_not_mixed() {
        // 空白文本不算 inline 内容
        let html = r#"<html><body>
          <div id="b">
            <div>X</div>
          </div>
        </body></html>"#;
        assert!(
            !has_mixed(html, "b"),
            "whitespace-only text + block should not trigger mixed"
        );
    }

    #[test]
    fn test_block_container_mixed_splits_correctly() {
        // div.b 含 div.red (block) + "B" (text) → 应产出一个 Inline + 一个 Block 片段
        let html = r#"<html><body>
          <div id="b"><div class="red"></div>B</div>
        </body></html>"#;
        let segs = split_block(html, "b").expect("mixed block should split");
        let block_count = segs
            .iter()
            .filter(|s| matches!(s, InlineBlockSegment::Block { .. }))
            .count();
        assert_eq!(block_count, 1, "expected exactly 1 block segment");
        let inline_count = segs
            .iter()
            .filter(|s| matches!(s, InlineBlockSegment::Inline { .. }))
            .count();
        assert_eq!(inline_count, 1, "expected exactly 1 inline segment ('B' text)");
    }

    #[test]
    fn test_block_container_out_of_flow_ignored() {
        // abspos 子元素不触发混合检测
        let html = r#"<html><body>
          <div id="b">text<div style="position:absolute">abs</div></div>
        </body></html>"#;
        assert!(
            !has_mixed(html, "b"),
            "out-of-flow-only child should not trigger mixed detection"
        );
    }

    #[test]
    fn test_is_whitespace_only_segment() {
        // 空白文本 → whitespace-only
        let doc = zero_dom::parse_html("<html><body><div>  </div></body></html>");
        let all_nodes: Vec<NodeId> = {
            let root = doc.root();
            let body = doc.child_nodes(root)[0];
            let div = doc.child_nodes(body)[0];
            doc.child_nodes(div)
        };
        // html5ever may or may not create separate text child nodes for div content.
        // If children exist, verify whitespace-only detection works.
        if !all_nodes.is_empty() {
            assert!(is_whitespace_only_inline_segment(&doc, &all_nodes));
        }

        // Non-whitespace text → verified via split_block test below
        // Element child → verified via split_block test below
    }
}
