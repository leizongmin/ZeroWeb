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
    pub fn insert_node(&mut self, doc: &mut Document, node: NodeId) -> Result<(), RangeError> {
        let parent = self.start_container;
        let children = doc.child_nodes(parent);
        if self.start_offset < children.len() {
            let ref_node = children[self.start_offset];
            let _ = doc.insert_before(parent, node, ref_node);
        } else {
            let _ = doc.append_child(parent, node);
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
}
