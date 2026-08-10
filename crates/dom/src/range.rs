//! DOM Range API 实现 — 文档范围的创建、操作和提取。
//!
//! 实现了 WHATWG DOM Standard 中的 Range 接口核心功能。

use crate::document::Document;
use crate::node::{NodeData, NodeId, NodeKind};

// ── Range ────────────────────────────────────────────────────────────────

/// DOM Range — 表示文档中两个边界点之间的连续范围。
///
/// 每个边界点由一个节点和偏移量组成。对于文本节点，偏移量是 UTF-8 字节偏移；
/// 对于元素节点，偏移量是子节点索引。
#[derive(Debug, Clone)]
pub struct Range {
    /// 起始容器节点。
    start_container: NodeId,
    /// 起始偏移量。
    start_offset: usize,
    /// 结束容器节点。
    end_container: NodeId,
    /// 结束偏移量。
    end_offset: usize,
}

/// Range 操作可能产生的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeError {
    /// 起始点在结束点之后。
    StartAfterEnd,
    /// 节点不属于当前文档。
    WrongDocument,
    /// 无效的偏移量。
    IndexSizeError,
    /// 范围已失效。
    Detached,
}

impl Range {
    /// 创建一个新 Range，从 start 节点的 offset 0 开始到同一节点的末尾。
    pub fn new(start: NodeId, end: NodeId) -> Self {
        Self {
            start_container: start,
            start_offset: 0,
            end_container: end,
            end_offset: 0,
        }
    }

    /// 创建一个空范围（折叠到指定位置）。
    pub fn at(node: NodeId, offset: usize) -> Self {
        Self {
            start_container: node,
            start_offset: offset,
            end_container: node,
            end_offset: offset,
        }
    }

    /// 获取起始容器。
    #[inline]
    pub fn start_container(&self) -> NodeId {
        self.start_container
    }

    /// 获取起始偏移。
    #[inline]
    pub fn start_offset(&self) -> usize {
        self.start_offset
    }

    /// 获取结束容器。
    #[inline]
    pub fn end_container(&self) -> NodeId {
        self.end_container
    }

    /// 获取结束偏移。
    #[inline]
    pub fn end_offset(&self) -> usize {
        self.end_offset
    }

    /// 设置起始位置。
    pub fn set_start(&mut self, node: NodeId, offset: usize) -> Result<(), RangeError> {
        self.start_container = node;
        self.start_offset = offset;
        Ok(())
    }

    /// 设置结束位置。
    pub fn set_end(&mut self, node: NodeId, offset: usize) -> Result<(), RangeError> {
        self.end_container = node;
        self.end_offset = offset;
        Ok(())
    }

    /// 判断范围是否折叠（起始 == 结束）。
    pub fn collapsed(&self) -> bool {
        self.start_container == self.end_container && self.start_offset == self.end_offset
    }

    /// 折叠范围到起始或结束位置。
    pub fn collapse(&mut self, to_start: bool) {
        if to_start {
            self.end_container = self.start_container;
            self.end_offset = self.start_offset;
        } else {
            self.start_container = self.end_container;
            self.start_offset = self.end_offset;
        }
    }

    /// 选中整个节点的内容。
    pub fn select_node_contents(&mut self, doc: &Document, node: NodeId) -> Result<(), RangeError> {
        self.start_container = node;
        self.start_offset = 0;
        self.end_container = node;
        self.end_offset = doc.child_nodes(node).len();
        Ok(())
    }

    /// 选中节点本身（含前后的边界偏移）。
    pub fn select_node(&mut self, doc: &Document, node: NodeId) -> Result<(), RangeError> {
        let parent = doc.parent_node(node).ok_or(RangeError::Detached)?;
        let siblings = doc.child_nodes(parent);
        let index = siblings
            .iter()
            .position(|&n| n == node)
            .ok_or(RangeError::WrongDocument)?;
        self.start_container = parent;
        self.start_offset = index;
        self.end_container = parent;
        self.end_offset = index + 1;
        Ok(())
    }

    /// 获取范围内包含的所有文本内容。
    pub fn text_content(&self, doc: &Document) -> String {
        let nodes = self.collect_top_level_nodes(doc);
        let mut result = String::new();
        for node_id in nodes {
            if let Some(text) = doc.text_content(node_id) {
                result.push_str(&text);
            }
        }
        result
    }

    /// 删除范围内的所有内容。
    pub fn delete_contents(&mut self, doc: &mut Document) -> Result<(), RangeError> {
        let nodes_to_remove = self.collect_top_level_nodes(doc);
        for node_id in nodes_to_remove.iter().rev() {
            if let Some(parent) = doc.parent_node(*node_id) {
                let _ = doc.remove_child(parent, *node_id);
            }
        }
        self.collapse(true);
        Ok(())
    }

    /// 提取范围内的内容到 DocumentFragment。
    ///
    /// 被提取的节点从文档中移除。
    pub fn extract_contents(&mut self, doc: &mut Document) -> Result<NodeId, RangeError> {
        let nodes_to_extract = self.collect_top_level_nodes(doc);
        let fragment = doc.create_document_fragment();

        for node_id in nodes_to_extract {
            if let Some(parent) = doc.parent_node(node_id) {
                let _ = doc.remove_child(parent, node_id);
            }
            let _ = doc.append_child(fragment, node_id);
        }
        self.collapse(true);
        Ok(fragment)
    }

    /// 克隆范围内的内容到新的 DocumentFragment。
    pub fn clone_contents(&self, doc: &mut Document) -> Result<NodeId, RangeError> {
        let nodes_to_clone = self.collect_top_level_nodes(doc);
        let fragment = doc.create_document_fragment();

        for node_id in nodes_to_clone {
            let cloned = doc.clone_node(node_id, true);
            let _ = doc.append_child(fragment, cloned);
        }
        Ok(fragment)
    }

    /// 在范围的起始位置插入节点。
    ///
    /// spec `dom-range-insertnode`：若 start 节点为 **Text** 节点，先在 `start_offset` 处拆分文本
    ///（原节点保留 `[0, offset)`，新建后半文本节点作为原节点的下一兄弟），再把 `node` 插到后半之前
    ///（即两半之间）。否则（元素/注释等容器）按 `start_offset` 子索引 `insertBefore`，越界则 `appendChild`。
    /// R3215：旧实现不分节点类型，对 Text 容器取 `child_nodes`（Text 无元素子 → 空）→ `appendChild` 进
    /// Text 节点（非法，无操作），致文本内部插入静默失效（rich-text 编辑 `range.insertNode(span)` 场景）。
    /// 字符偏移拆分（非字节），多字节安全。spec：https://dom.spec.whatwg.org/#dom-range-insertnode
    pub fn insert_node(&mut self, doc: &mut Document, node: NodeId) -> Result<(), RangeError> {
        let start = self.start_container;
        let is_text = doc.get(start).is_some_and(|n| matches!(n.kind, NodeKind::Text(_)));
        if is_text {
            let parent = doc.parent_node(start);
            let full = doc.text_content(start).unwrap_or_default();
            let off = self.start_offset.min(full.chars().count());
            let head: String = full.chars().take(off).collect();
            let tail: String = full.chars().skip(off).collect();
            doc.set_text_content(start, &head);
            let second = doc.create_text_node(&tail);
            if let Some(p) = parent {
                // 后半插到原节点下一兄弟之前（即原节点之后）；`node` 插到后半之前 → 终序 [head][node][tail]。
                match doc.next_sibling(start) {
                    Some(ref_node) => {
                        let _ = doc.insert_before(p, second, ref_node);
                    }
                    None => {
                        let _ = doc.append_child(p, second);
                    }
                }
                let _ = doc.insert_before(p, node, second);
            }
            return Ok(());
        }
        // 非 Text 容器：start_offset 子索引 insertBefore，越界 appendChild。
        let children = doc.child_nodes(start);
        if self.start_offset < children.len() {
            let ref_node = children[self.start_offset];
            let _ = doc.insert_before(start, node, ref_node);
        } else {
            let _ = doc.append_child(start, node);
        }
        Ok(())
    }

    /// 收集范围内最顶层的直接子节点列表。
    ///
    /// 当 start == end 时，返回容器在 [start_offset, end_offset) 范围内的子节点。
    fn collect_top_level_nodes(&self, doc: &Document) -> Vec<NodeId> {
        // 最常见的情况：start == end，范围覆盖同一容器的子节点子集
        if self.start_container == self.end_container {
            let children = doc.child_nodes(self.start_container);
            let start = self.start_offset.min(children.len());
            let end = self.end_offset.min(children.len());
            return children[start..end].to_vec();
        }

        // 不同容器的情况：收集从 start 到 end 之间的顶层节点
        let mut result = Vec::new();

        // start 容器侧的子节点（从 start_offset 开始）
        let start_children = doc.child_nodes(self.start_container);
        let start_idx = self.start_offset.min(start_children.len());
        result.extend_from_slice(&start_children[start_idx..]);

        // end 容器侧的子节点（到 end_offset 为止）
        let end_children = doc.child_nodes(self.end_container);
        let end_idx = self.end_offset.min(end_children.len());
        result.extend_from_slice(&end_children[..end_idx]);

        result
    }

    /// 克隆当前 Range，返回一个具有相同边界的新 Range。
    pub fn clone_range(&self) -> Self {
        self.clone()
    }

    /// 比较两个 Range 的边界点位置关系。
    ///
    /// 返回值：
    /// - `-1`：self 的结束在 other 的开始之前（self 完全在 other 前面）。
    /// - `0`：两个 Range 共享边界点或边界重合。
    /// - `1`：self 的开始在 other 的结束之后（self 完全在 other 后面）。
    pub fn compare_boundary_points(&self, other: &Range) -> i32 {
        // 先比较结束容器/偏移 vs 起始容器/偏移
        if self.end_container == other.start_container && self.end_offset < other.start_offset {
            return -1;
        }
        if self.start_container == other.end_container && self.start_offset > other.end_offset {
            return 1;
        }
        // 检查完全相同的边界
        if self.start_container == other.start_container
            && self.start_offset == other.start_offset
            && self.end_container == other.end_container
            && self.end_offset == other.end_offset
        {
            return 0;
        }
        // 同一容器内的一般比较
        if self.end_container == other.start_container && self.end_offset <= other.start_offset {
            return -1;
        }
        if self.start_container == other.end_container && self.start_offset >= other.end_offset {
            return 1;
        }
        0
    }

    /// 获取范围的字符串表示（用于调试）。
    pub fn to_debug_string(&self, doc: &Document) -> String {
        let start_name = node_debug_name(doc, self.start_container);
        let end_name = node_debug_name(doc, self.end_container);
        format!(
            "Range [{}, offset={}] → [{}, offset={}]",
            start_name, self.start_offset, end_name, self.end_offset
        )
    }
}

/// 获取节点的调试名称。
fn node_debug_name(doc: &Document, id: NodeId) -> String {
    match doc.get(id) {
        Some(NodeData {
            kind: NodeKind::Document(_),
            ..
        }) => "#document".to_string(),
        Some(NodeData {
            kind: NodeKind::Element(e),
            ..
        }) => format!("<{}>", e.local_name()),
        Some(NodeData {
            kind: NodeKind::Text(t),
            ..
        }) => format!("#text({:?})", &t.content[..t.content.len().min(20)]),
        Some(NodeData {
            kind: NodeKind::Comment(c),
            ..
        }) => format!("#comment({:?})", &c.content[..c.content.len().min(20)]),
        Some(NodeData {
            kind: NodeKind::DocumentFragment,
            ..
        }) => "#document-fragment".to_string(),
        _ => format!("{:?}", id),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_html;

    /// 辅助函数：获取 body 节点。parse_html 创建 document > html > body 结构。
    fn body_of(doc: &Document) -> NodeId {
        let html = doc.first_child(doc.root()).unwrap();
        // html 有 head 和 body 两个子节点，body 是最后一个
        doc.last_child(html).unwrap()
    }

    /// 测试 Range 基本创建和访问。
    #[test]
    fn test_range_creation() {
        let doc = parse_html("<p>Hello</p>");
        let body = body_of(&doc);
        let p = doc.first_child(body).unwrap();
        let range = Range::new(p, p);
        assert_eq!(range.start_container(), p);
        assert_eq!(range.end_container(), p);
        assert_eq!(range.start_offset(), 0);
    }

    /// 测试折叠范围。
    #[test]
    fn test_range_collapsed() {
        let doc = parse_html("<p>Hello</p>");
        let body = body_of(&doc);
        let p = doc.first_child(body).unwrap();
        let range = Range::at(p, 0);
        assert!(range.collapsed());

        let range2 = Range::new(p, p);
        // start == end, offset == 0, 所以折叠
        assert!(range2.collapsed());
    }

    /// 测试 collapse 到起始/结束。
    #[test]
    fn test_range_collapse() {
        let doc = parse_html("<p>Hello</p>");
        let body = body_of(&doc);
        let p = doc.first_child(body).unwrap();
        let mut range = Range::new(p, p);
        range.set_end(p, 5).unwrap();
        assert!(!range.collapsed());

        range.collapse(true);
        assert!(range.collapsed());
        assert_eq!(range.start_offset(), 0);

        let mut range2 = Range::new(p, p);
        range2.set_end(p, 5).unwrap();
        range2.collapse(false);
        assert!(range2.collapsed());
        assert_eq!(range2.start_offset(), 5);
    }

    /// 测试 select_node_contents。
    #[test]
    fn test_select_node_contents() {
        let doc = parse_html("<div><p>A</p><p>B</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();

        let mut range = Range::new(div, div);
        range.select_node_contents(&doc, div).unwrap();

        assert_eq!(range.start_container(), div);
        assert_eq!(range.start_offset(), 0);
        assert_eq!(range.end_container(), div);
        assert_eq!(range.end_offset(), 2); // 两个 <p> 子节点
    }

    /// 测试 select_node。
    #[test]
    fn test_select_node() {
        let doc = parse_html("<div><p>A</p><p>B</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();
        let children = doc.child_nodes(div);
        let _p1 = children[0];
        let p2 = children[1];

        let mut range = Range::new(div, div);
        range.select_node(&doc, p2).unwrap();

        assert_eq!(range.start_container(), div);
        assert_eq!(range.start_offset(), 1);
        assert_eq!(range.end_container(), div);
        assert_eq!(range.end_offset(), 2);
    }

    /// 测试 text_content 收集。
    #[test]
    fn test_range_text_content() {
        let doc = parse_html("<p>Hello <b>World</b></p>");
        let body = body_of(&doc);
        let p = doc.first_child(body).unwrap();

        let mut range = Range::new(p, p);
        range.set_end(p, doc.child_nodes(p).len()).unwrap();

        let text = range.text_content(&doc);
        assert!(text.contains("Hello"), "text should contain 'Hello', got: {text}");
        assert!(text.contains("World"), "text should contain 'World', got: {text}");
    }

    /// 测试 delete_contents。
    #[test]
    fn test_range_delete_contents() {
        let mut doc = parse_html("<div><p>A</p><p>B</p><p>C</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();

        let mut range = Range::new(div, div);
        range.select_node_contents(&doc, div).unwrap();
        range.delete_contents(&mut doc).unwrap();

        assert_eq!(doc.child_nodes(div).len(), 0);
        assert!(range.collapsed());
    }

    /// 测试 extract_contents。
    #[test]
    fn test_range_extract_contents() {
        let mut doc = parse_html("<div><p>A</p><p>B</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();

        let mut range = Range::new(div, div);
        range.select_node_contents(&doc, div).unwrap();
        let fragment = range.extract_contents(&mut doc).unwrap();

        // 原始 div 应该为空
        assert_eq!(doc.child_nodes(div).len(), 0, "div should be empty after extract");
        // fragment 应该包含提取的节点
        assert_eq!(doc.child_nodes(fragment).len(), 2, "fragment should have 2 children");
    }

    /// 测试 clone_contents。
    #[test]
    fn test_range_clone_contents() {
        let mut doc = parse_html("<div><p>A</p><p>B</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();

        let mut range = Range::new(div, div);
        range.select_node_contents(&doc, div).unwrap();
        let fragment = range.clone_contents(&mut doc).unwrap();

        // 原始 div 不变
        assert_eq!(
            doc.child_nodes(div).len(),
            2,
            "original div should still have 2 children"
        );
        // fragment 是克隆
        assert_eq!(
            doc.child_nodes(fragment).len(),
            2,
            "cloned fragment should have 2 children"
        );
    }

    /// 测试 insert_node。
    #[test]
    fn test_range_insert_node() {
        let mut doc = parse_html("<div><p>A</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();

        let new_p = doc.create_element("p");
        doc.set_text_content(new_p, "inserted");

        let mut range = Range::at(div, 0);
        range.insert_node(&mut doc, new_p).unwrap();

        let children = doc.child_nodes(div);
        assert_eq!(children.len(), 2, "div should have 2 children after insert");
    }

    /// R3215：insert_node 在 **Text 节点** start 容器上拆分文本并在两半之间插入（spec
    /// `dom-range-insertnode`）。旧实现不分节点类型致文本内部插入静默失效。
    #[test]
    fn test_range_insert_node_text_split_r3215() {
        let mut doc = parse_html("<div>Hello World</div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();
        // div 的首子是文本节点 "Hello World"。
        let text = doc.first_child(div).unwrap();
        assert!(matches!(doc.get(text).unwrap().kind, NodeKind::Text(_)));

        let span = doc.create_element("span");
        doc.set_text_content(span, "X");

        // start 容器 = text，offset = 5（"Hello" 与 " World" 之间）。
        let mut range = Range::at(text, 5);
        range.insert_node(&mut doc, span).unwrap();

        // 终序应为 [text "Hello"][span "X"][text " World"]。
        let kids = doc.child_nodes(div);
        assert_eq!(kids.len(), 3, "div 应有 3 子（拆分两半 + 插入节点）");
        assert_eq!(doc.text_content(kids[0]), Some("Hello".to_string()));
        assert!(matches!(doc.get(kids[1]).unwrap().kind, NodeKind::Element(_)));
        assert_eq!(doc.text_content(kids[1]), Some("X".to_string()));
        assert_eq!(doc.text_content(kids[2]), Some(" World".to_string()));
        // 原文本节点保留前半（未被整体覆盖）。
        assert_eq!(doc.text_content(text), Some("Hello".to_string()));
    }

    /// R3215：start_offset 越界（> 文本长度）clamp 到末尾，文本不拆分，节点插到末尾。
    #[test]
    fn test_range_insert_node_text_offset_clamp_r3215() {
        let mut doc = parse_html("<div>Hi</div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();
        let text = doc.first_child(div).unwrap();

        let span = doc.create_element("em");
        doc.set_text_content(span, "!");

        // offset 99 远超 "Hi"（2 字符）→ clamp 到 2，head="Hi" tail=""。
        let mut range = Range::at(text, 99);
        range.insert_node(&mut doc, span).unwrap();

        let kids = doc.child_nodes(div);
        assert_eq!(kids.len(), 3, "仍拆出空 tail 文本节点");
        assert_eq!(doc.text_content(kids[0]), Some("Hi".to_string()));
        assert_eq!(doc.text_content(kids[1]), Some("!".to_string()));
        assert_eq!(doc.text_content(kids[2]), Some(String::new()));
    }

    /// 测试 Range::at 创建折叠范围。
    #[test]
    fn test_range_at() {
        let doc = parse_html("<p>Test</p>");
        let body = body_of(&doc);
        let p = doc.first_child(body).unwrap();

        let range = Range::at(p, 2);
        assert!(range.collapsed());
        assert_eq!(range.start_offset(), 2);
        assert_eq!(range.end_offset(), 2);
    }

    /// 测试空文档上的 Range。
    #[test]
    fn test_range_empty_document() {
        let doc = parse_html("");
        let range = Range::at(doc.root(), 0);
        assert!(range.collapsed());
    }

    /// 测试 set_start 和 set_end。
    #[test]
    fn test_range_set_boundaries() {
        let doc = parse_html("<div><p>A</p><p>B</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();
        let children = doc.child_nodes(div);
        assert!(children.len() >= 2, "div should have at least 2 children");

        let mut range = Range::new(children[0], children[1]);
        range.set_start(children[0], 0).unwrap();
        range.set_end(children[1], 1).unwrap();

        assert_eq!(range.start_container(), children[0]);
        assert_eq!(range.end_container(), children[1]);
    }

    // ── 边界用例测试 ────────────────────────────────────────────────────

    /// 测试 Range::select_node 选中整个元素节点（非其内容）。
    ///
    /// select_node 应将范围设置为父容器中包含目标节点的 [index, index+1) 区间，
    /// 而非选中节点的子节点。
    #[test]
    fn test_range_select_node() {
        let doc = parse_html("<div><span>first</span><em>second</em><b>third</b></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();
        let children = doc.child_nodes(div);
        assert_eq!(children.len(), 3, "div should have 3 children");

        // 选中第二个子节点 <em>
        let em = children[1];
        let mut range = Range::new(div, div);
        range.select_node(&doc, em).unwrap();

        // start_container 和 end_container 应该是父节点 div，而非 em 自身
        assert_eq!(range.start_container(), div, "start_container should be parent div");
        assert_eq!(range.end_container(), div, "end_container should be parent div");
        // 偏移量应该包围目标节点：[1, 2)
        assert_eq!(range.start_offset(), 1, "start_offset should be index of em in div");
        assert_eq!(range.end_offset(), 2, "end_offset should be index+1");

        // 选中第一个子节点 <span> → [0, 1)
        let span = children[0];
        let mut range2 = Range::new(div, div);
        range2.select_node(&doc, span).unwrap();
        assert_eq!(range2.start_offset(), 0);
        assert_eq!(range2.end_offset(), 1);

        // 选中最后一个子节点 <b> → [2, 3)
        let b = children[2];
        let mut range3 = Range::new(div, div);
        range3.select_node(&doc, b).unwrap();
        assert_eq!(range3.start_offset(), 2);
        assert_eq!(range3.end_offset(), 3);
    }

    /// 测试 Range::select_node 对 Document 节点应返回错误。
    ///
    /// Document 节点没有父节点，select_node 调用 parent_node 时返回 None，
    /// 应当产生 RangeError::Detached。
    #[test]
    fn test_range_select_node_invalid() {
        let doc = parse_html("<p>text</p>");
        let root = doc.root();

        let mut range = Range::at(root, 0);
        let result = range.select_node(&doc, root);
        assert!(
            matches!(result, Err(RangeError::Detached)),
            "select_node on Document root should return Detached error, got {:?}",
            result
        );
    }

    /// 测试 Range::text_content 提取跨节点的部分文本内容。
    ///
    /// 当范围跨越多个子节点时，text_content 应收集所有被覆盖节点的文本；
    /// 使用偏移量可以只提取部分子节点的文本。
    #[test]
    fn test_range_text_content_partial() {
        // 测试跨多个子节点提取文本
        let doc = parse_html("<p>Hello <b>World</b>!</p>");
        let body = body_of(&doc);
        let p = doc.first_child(body).unwrap();
        let children = doc.child_nodes(p);

        // 范围覆盖 p 的所有子节点
        let mut range = Range::new(p, p);
        range.set_end(p, children.len()).unwrap();
        let text = range.text_content(&doc);
        assert!(text.contains("Hello"), "text should contain 'Hello', got: {text}");
        assert!(text.contains("World"), "text should contain 'World', got: {text}");
        assert!(text.contains("!"), "text should contain '!', got: {text}");

        // 范围只覆盖前半部分（offset 0..1），仅第一个文本节点
        let mut range2 = Range::new(p, p);
        range2.set_end(p, 1).unwrap();
        let text2 = range2.text_content(&doc);
        assert!(
            text2.contains("Hello") && !text2.contains("World"),
            "partial range should only have 'Hello', got: {text2}"
        );

        // 范围只覆盖后半部分（offset 1..end），不含第一个文本节点
        let mut range3 = Range::new(p, p);
        range3.set_start(p, 1).unwrap();
        range3.set_end(p, children.len()).unwrap();
        let text3 = range3.text_content(&doc);
        assert!(
            !text3.contains("Hello") && text3.contains("World"),
            "latter range should have 'World' not 'Hello', got: {text3}"
        );
    }

    /// 测试 Range::to_debug_string 生成预期的调试格式字符串。
    #[test]
    fn test_range_to_debug_string() {
        let doc = parse_html("<div><p>Hello</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();
        let p = doc.first_child(div).unwrap();

        let range = Range::new(div, p);
        let debug = range.to_debug_string(&doc);

        // 应包含 <div> 和 <p> 标签名
        assert!(
            debug.contains("<div>"),
            "debug string should contain '<div>', got: {debug}"
        );
        assert!(debug.contains("<p>"), "debug string should contain '<p>', got: {debug}");
        assert!(
            debug.contains("offset=0"),
            "debug string should contain 'offset=0', got: {debug}"
        );
        assert!(
            debug.starts_with("Range ["),
            "debug string should start with 'Range [', got: {debug}"
        );

        // 折叠范围的调试字符串
        let collapsed = Range::at(p, 3);
        let debug2 = collapsed.to_debug_string(&doc);
        assert!(
            debug2.contains("offset=3"),
            "collapsed debug string should contain 'offset=3', got: {debug2}"
        );
    }

    /// 测试 set_start / set_end 对越界偏移量的行为。
    ///
    /// 当前实现不验证偏移量是否在合法范围内，始终返回 Ok(())。
    /// 此测试记录该行为，确保 API 调用不会 panic。
    #[test]
    fn test_range_set_start_end_invalid_offset() {
        let doc = parse_html("<p>Hi</p>");
        let body = body_of(&doc);
        let p = doc.first_child(body).unwrap();

        // 超大偏移量不应导致 panic
        let mut range = Range::new(p, p);
        let result = range.set_start(p, usize::MAX);
        assert!(result.is_ok(), "set_start with large offset should not error");

        let result2 = range.set_end(p, 999_999);
        assert!(result2.is_ok(), "set_end with large offset should not error");

        assert_eq!(range.start_offset(), usize::MAX);
        assert_eq!(range.end_offset(), 999_999);
    }

    /// 测试 clone_contents 在范围分割子节点列表时的行为。
    ///
    /// 当范围从 offset != 0 开始，或 end_offset != 子节点总数时，
    /// clone_contents 应只克隆范围内的子节点子集。
    #[test]
    fn test_range_clone_contents_partial_text() {
        let mut doc = parse_html("<div><p>A</p><p>B</p><p>C</p><p>D</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();
        let children = doc.child_nodes(div);
        assert_eq!(children.len(), 4, "div should have 4 children");

        // 只克隆中间两个 <p>（offset 1..3）
        let mut range = Range::new(div, div);
        range.set_start(div, 1).unwrap();
        range.set_end(div, 3).unwrap();

        let fragment = range.clone_contents(&mut doc).unwrap();

        // fragment 应包含 2 个克隆节点
        let frag_children = doc.child_nodes(fragment);
        assert_eq!(frag_children.len(), 2, "fragment should have 2 cloned children");

        // 原始 div 应保持不变（4 个子节点）
        assert_eq!(
            doc.child_nodes(div).len(),
            4,
            "original div should still have 4 children"
        );

        // 验证克隆节点的文本内容 — 范围 [1,3) 对应原始 B 和 C
        let first_clone = frag_children[0];
        let second_clone = frag_children[1];
        assert_eq!(
            doc.text_content(first_clone).as_deref(),
            Some("B"),
            "first cloned child should have text 'B' (original index 1)"
        );
        assert_eq!(
            doc.text_content(second_clone).as_deref(),
            Some("C"),
            "second cloned child should have text 'C' (original index 2)"
        );
    }

    /// 测试 insert_node 在偏移量指向容器子节点中间位置时的行为。
    ///
    /// 当 start_offset 对应容器中已有子节点的索引时，
    /// insert_node 应在该子节点之前插入新节点。
    #[test]
    fn test_range_insert_node_mid_text() {
        let mut doc = parse_html("<div><p>first</p><p>last</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();
        let original_children = doc.child_nodes(div);
        assert_eq!(original_children.len(), 2);

        // 在 div 的 offset 1 处（即两个 <p> 之间）插入新节点
        let new_node = doc.create_element("span");
        doc.set_text_content(new_node, "middle");

        let mut range = Range::at(div, 1);
        range.insert_node(&mut doc, new_node).unwrap();

        let children = doc.child_nodes(div);
        assert_eq!(children.len(), 3, "div should have 3 children after insert");

        // 新节点应在中间位置
        assert_eq!(children[0], original_children[0], "first child unchanged");
        assert_eq!(children[1], new_node, "inserted node at index 1");
        assert_eq!(children[2], original_children[1], "last child shifted to index 2");

        // 验证文本内容
        assert_eq!(
            doc.text_content(children[1]).as_deref(),
            Some("middle"),
            "inserted span should contain 'middle'"
        );
    }

    // ── cloneRange / compareBoundaryPoints 测试 ──────────────────────────

    /// 测试 clone_range：克隆后新 Range 应具有相同的边界点。
    #[test]
    fn test_range_clone_range() {
        let doc = parse_html("<div><p>A</p><p>B</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();

        let mut range = Range::new(div, div);
        range.set_start(div, 1).unwrap();
        range.set_end(div, 2).unwrap();

        let cloned = range.clone_range();
        assert_eq!(cloned.start_container(), range.start_container());
        assert_eq!(cloned.start_offset(), range.start_offset());
        assert_eq!(cloned.end_container(), range.end_container());
        assert_eq!(cloned.end_offset(), range.end_offset());

        // 修改原 Range 不应影响克隆
        range.set_start(div, 0).unwrap();
        assert_eq!(cloned.start_offset(), 1, "clone should be independent");
    }

    /// 测试 compare_boundary_points：第一个 Range 结束在第二个开始之前，返回 -1。
    #[test]
    fn test_range_compare_before() {
        let doc = parse_html("<div><p>A</p><p>B</p><p>C</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();

        // 第一个范围覆盖 offset 0..1，第二个覆盖 2..3
        let mut r1 = Range::new(div, div);
        r1.set_start(div, 0).unwrap();
        r1.set_end(div, 1).unwrap();

        let mut r2 = Range::new(div, div);
        r2.set_start(div, 2).unwrap();
        r2.set_end(div, 3).unwrap();

        assert_eq!(r1.compare_boundary_points(&r2), -1);
    }

    /// 测试 compare_boundary_points：第一个 Range 开始在第二个结束之后，返回 1。
    #[test]
    fn test_range_compare_after() {
        let doc = parse_html("<div><p>A</p><p>B</p><p>C</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();

        // 第一个范围覆盖 offset 2..3，第二个覆盖 0..1
        let mut r1 = Range::new(div, div);
        r1.set_start(div, 2).unwrap();
        r1.set_end(div, 3).unwrap();

        let mut r2 = Range::new(div, div);
        r2.set_start(div, 0).unwrap();
        r2.set_end(div, 1).unwrap();

        assert_eq!(r1.compare_boundary_points(&r2), 1);
    }

    /// 测试 compare_boundary_points：两个完全相同的 Range，返回 0。
    #[test]
    fn test_range_compare_same() {
        let doc = parse_html("<div><p>A</p><p>B</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();

        let mut r1 = Range::new(div, div);
        r1.set_start(div, 0).unwrap();
        r1.set_end(div, 2).unwrap();

        let mut r2 = Range::new(div, div);
        r2.set_start(div, 0).unwrap();
        r2.set_end(div, 2).unwrap();

        assert_eq!(r1.compare_boundary_points(&r2), 0);
    }

    /// 测试 select_node_contents 对深层嵌套元素：范围应覆盖所有子节点。
    #[test]
    fn test_range_select_node_contents_deep() {
        let doc = parse_html("<div><p><span>A</span><b>B</b></p><p>C</p></div>");
        let body = body_of(&doc);
        let div = doc.first_child(body).unwrap();
        let children = doc.child_nodes(div);
        let p = children[0]; // 第一个 <p>，内含 <span> 和 <b>

        // 选中 <p> 的所有内容
        let mut range = Range::new(p, p);
        range.select_node_contents(&doc, p).unwrap();

        assert_eq!(range.start_container(), p);
        assert_eq!(range.start_offset(), 0);
        assert_eq!(range.end_container(), p);
        // <p> 有 <span>A</span>、<b>B</b> 两个子元素
        assert_eq!(range.end_offset(), 2, "<p> should have 2 children (span, b)");

        // 验证范围覆盖的文本内容包含所有嵌套文本
        let text = range.text_content(&doc);
        assert!(text.contains("A"), "text should contain 'A', got: {text}");
        assert!(text.contains("B"), "text should contain 'B', got: {text}");
    }
}
