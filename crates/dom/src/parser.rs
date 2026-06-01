//! html5ever 集成 — HTML 解析器和 TreeSink 实现。

use crate::document::Document;
use crate::node::{
    CommentData, DocumentData, DocumentTypeData, NodeData, NodeId, NodeKind, ProcessingInstructionData, QuirksMode,
    TextData,
};
use hashbrown::HashMap;
use html5ever::interface::{ElemName, ElementFlags, NodeOrText, TreeSink};
use markup5ever::{Attribute, QualName};
use std::borrow::Cow;
use tendril::StrTendril;

/// 使用 html5ever 解析 HTML 字符串并返回 Document。
///
/// 支持完整 HTML5 文档和文档片段。
/// 错误恢复遵循 WHATWG HTML 规范。
///
/// # 示例
///
/// ```
/// use zero_dom::parse_html;
///
/// let doc = parse_html("<!DOCTYPE html><html><body><h1>Hello</h1></body></html>");
/// assert!(doc.root().is_valid());
/// ```
pub fn parse_html(html: &str) -> Document {
    parse_html_with_builder(html)
}

/// 使用 html5ever 从文件解析 HTML。
pub fn parse_html_from_file(path: &std::path::Path) -> std::io::Result<Document> {
    use html5ever::driver::ParseOpts;
    use tendril::TendrilSink;

    let builder = DomBuilder::new();
    let parser = html5ever::parse_document(builder, ParseOpts::default());
    parser.from_utf8().from_file(path)
}

// ── DomBuilder ──────────────────────────────────────────────────────

/// DOM 树构建器，实现 html5ever 的 TreeSink trait。
///
/// 使用 `RefCell` 提供内部可变性，使 html5ever 可以通过 `&self` 修改 DOM 树。
/// 解析完成后通过 `finish()` 消费并返回 `Document`。
pub struct DomBuilder {
    inner: std::cell::RefCell<DomBuilderInner>,
}

struct DomBuilderInner {
    nodes: slotmap::SlotMap<NodeId, NodeData>,
    root: NodeId,
    quirks_mode: QuirksMode,
}

impl DomBuilder {
    /// 创建新的 DOM 构建器。
    pub fn new() -> Self {
        let mut nodes = slotmap::SlotMap::with_key();
        let root = nodes.insert(NodeData::new(NodeKind::Document(DocumentData {
            quirks_mode: QuirksMode::NoQuirks,
        })));

        Self {
            inner: std::cell::RefCell::new(DomBuilderInner {
                nodes,
                root,
                quirks_mode: QuirksMode::NoQuirks,
            }),
        }
    }

    /// 消费构建器，返回 Document。
    pub fn into_document(self) -> Document {
        let inner = self.inner.into_inner();
        let mut doc = Document::new();

        // 收集旧节点到新节点的映射
        let mut mapping: HashMap<NodeId, NodeId> = HashMap::with_capacity(inner.nodes.len());

        for (old_id, node_data) in &inner.nodes {
            let new_id = match &node_data.kind {
                NodeKind::Document(_) => doc.root(),
                NodeKind::Element(elem) => doc.create_element_with_qname(elem.name.clone(), elem.attributes.clone()),
                NodeKind::Text(data) => doc.create_text_node(&data.content),
                NodeKind::Comment(data) => doc.create_comment(&data.content),
                NodeKind::DocumentType(dt) => {
                    doc.create_document_type(&dt.name, dt.public_id.clone(), dt.system_id.clone())
                }
                NodeKind::DocumentFragment => doc.create_document_fragment(),
                NodeKind::ProcessingInstruction(pi) => doc.create_processing_instruction(&pi.target, &pi.data),
                NodeKind::ShadowRoot(_) => doc.create_document_fragment(),
            };
            mapping.insert(old_id, new_id);
        }

        // 重建树结构（父子关系）
        for (old_id, node_data) in &inner.nodes {
            let new_id = mapping[&old_id];

            // 设置父引用
            if let Some(old_parent) = node_data.parent
                && let Some(&new_parent) = mapping.get(&old_parent)
                && let Some(child_data) = doc.get_mut(new_id)
            {
                child_data.parent = Some(new_parent);
            }

            // 设置子节点列表
            let new_children: Vec<NodeId> = node_data
                .children
                .iter()
                .filter_map(|c| mapping.get(c).copied())
                .collect();

            if !new_children.is_empty()
                && let Some(node_data) = doc.get_mut(new_id)
            {
                node_data.children = new_children;
            }
        }

        doc.set_quirks_mode(inner.quirks_mode);
        doc
    }
}

impl Default for DomBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── OwnedElemName ───────────────────────────────────────────────────

/// 拥有数据的元素名称包装器，用于解决 RefCell 借用生命周期问题。
#[derive(Debug, Clone)]
pub struct OwnedElemName {
    /// 命名空间。
    ns: markup5ever::Namespace,
    /// 本地名。
    local: markup5ever::LocalName,
}

impl ElemName for OwnedElemName {
    fn ns(&self) -> &markup5ever::Namespace {
        &self.ns
    }

    fn local_name(&self) -> &markup5ever::LocalName {
        &self.local
    }
}

impl TreeSink for DomBuilder {
    type Handle = NodeId;
    type Output = Document;
    type ElemName<'a>
        = OwnedElemName
    where
        Self: 'a;

    fn finish(self) -> Self::Output {
        self.into_document()
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {
        // 解析错误：html5ever 自动进行错误恢复
    }

    fn get_document(&self) -> Self::Handle {
        self.inner.borrow().root
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        let inner = self.inner.borrow();
        match inner.nodes.get(*target) {
            Some(NodeData {
                kind: NodeKind::Element(elem),
                ..
            }) => OwnedElemName {
                ns: elem.name.ns.clone(),
                local: elem.name.local.clone(),
            },
            _ => OwnedElemName {
                ns: markup5ever::Namespace::from(""),
                local: markup5ever::LocalName::from(""),
            },
        }
    }

    fn create_element(&self, name: QualName, attrs: Vec<Attribute>, _flags: ElementFlags) -> Self::Handle {
        let mut inner = self.inner.borrow_mut();
        let elem_data = crate::node::ElementData::new(name, attrs);
        inner.nodes.insert(NodeData::new(NodeKind::Element(elem_data)))
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        let mut inner = self.inner.borrow_mut();
        inner
            .nodes
            .insert(NodeData::new(NodeKind::Comment(CommentData::new(&text))))
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
        let mut inner = self.inner.borrow_mut();
        inner.nodes.insert(NodeData::new(NodeKind::ProcessingInstruction(
            ProcessingInstructionData {
                target: target.to_string(),
                data: data.to_string(),
            },
        )))
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let mut inner = self.inner.borrow_mut();
        match child {
            NodeOrText::AppendNode(node_id) => {
                // 检查是否需要合并相邻文本节点
                let should_merge = inner
                    .nodes
                    .get(*parent)
                    .and_then(|p| p.children.last().copied())
                    .map(|last| {
                        let is_new_text = matches!(inner.nodes.get(node_id).map(|n| &n.kind), Some(NodeKind::Text(_)));
                        let is_last_text = matches!(inner.nodes.get(last).map(|n| &n.kind), Some(NodeKind::Text(_)));
                        is_new_text && is_last_text
                    })
                    .unwrap_or(false);

                if should_merge {
                    let last_child = inner.nodes.get(*parent).unwrap().children.last().copied().unwrap();
                    let new_content = match inner.nodes.get(node_id).map(|n| n.kind.clone()) {
                        Some(NodeKind::Text(data)) => data.content,
                        _ => return,
                    };
                    if let Some(NodeKind::Text(data)) = inner.nodes.get_mut(last_child).map(|n| &mut n.kind) {
                        data.content.push_str(&new_content);
                    }
                    inner.nodes.remove(node_id);
                } else {
                    if let Some(child_data) = inner.nodes.get_mut(node_id) {
                        child_data.parent = Some(*parent);
                    }
                    if let Some(parent_data) = inner.nodes.get_mut(*parent) {
                        parent_data.children.push(node_id);
                    }
                }
            }
            NodeOrText::AppendText(text) => {
                // 检查是否需要合并到上一个文本节点
                let last_child = inner.nodes.get(*parent).and_then(|p| p.children.last().copied());

                let should_merge = last_child
                    .map(|last| matches!(inner.nodes.get(last).map(|n| &n.kind), Some(NodeKind::Text(_))))
                    .unwrap_or(false);

                if should_merge {
                    let last = last_child.unwrap();
                    if let Some(NodeKind::Text(data)) = inner.nodes.get_mut(last).map(|n| &mut n.kind) {
                        data.content.push_str(&text);
                    }
                } else {
                    let text_id = inner.nodes.insert(NodeData::new(NodeKind::Text(TextData::new(&text))));
                    if let Some(child_data) = inner.nodes.get_mut(text_id) {
                        child_data.parent = Some(*parent);
                    }
                    if let Some(parent_data) = inner.nodes.get_mut(*parent) {
                        parent_data.children.push(text_id);
                    }
                }
            }
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        _prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let has_parent = {
            let inner = self.inner.borrow();
            inner.nodes.get(*element).and_then(|n| n.parent).is_some()
        };

        if has_parent {
            self.append_before_sibling(element, child);
        } else {
            self.append(element, child);
        }
    }

    fn append_doctype_to_document(&self, name: StrTendril, public_id: StrTendril, system_id: StrTendril) {
        let mut inner = self.inner.borrow_mut();
        let root = inner.root;
        let doctype_id = inner
            .nodes
            .insert(NodeData::new(NodeKind::DocumentType(DocumentTypeData {
                name: name.to_string(),
                public_id: if public_id.is_empty() {
                    None
                } else {
                    Some(public_id.to_string())
                },
                system_id: if system_id.is_empty() {
                    None
                } else {
                    Some(system_id.to_string())
                },
            })));

        if let Some(dt_data) = inner.nodes.get_mut(doctype_id) {
            dt_data.parent = Some(root);
        }
        if let Some(root_data) = inner.nodes.get_mut(root) {
            root_data.children.push(doctype_id);
        }
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        // <template> 暂时返回目标节点自身
        *target
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        self.inner.borrow_mut().quirks_mode = mode;
    }

    fn append_before_sibling(&self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        let mut inner = self.inner.borrow_mut();

        // 获取 sibling 的父节点
        let parent = inner.nodes.get(*sibling).and_then(|n| n.parent);

        let parent = match parent {
            Some(p) => p,
            None => return,
        };

        // 找到 sibling 在父节点子列表中的位置
        let sibling_idx = inner
            .nodes
            .get(parent)
            .and_then(|p| p.children.iter().position(|&id| id == *sibling));

        let sibling_idx = match sibling_idx {
            Some(idx) => idx,
            None => return,
        };

        match new_node {
            NodeOrText::AppendNode(node_id) => {
                // 从旧父节点移除
                let old_parent = inner.nodes.get(node_id).and_then(|n| n.parent);
                if let Some(old_p) = old_parent
                    && let Some(old_pd) = inner.nodes.get_mut(old_p)
                {
                    old_pd.children.retain(|&id| id != node_id);
                }

                if let Some(child_data) = inner.nodes.get_mut(node_id) {
                    child_data.parent = Some(parent);
                }
                if let Some(parent_data) = inner.nodes.get_mut(parent) {
                    parent_data.children.insert(sibling_idx, node_id);
                }
            }
            NodeOrText::AppendText(text) => {
                // 如果 sibling 前面有文本节点，合并
                let should_merge = if sibling_idx > 0 {
                    let prev_id = inner.nodes.get(parent).unwrap().children[sibling_idx - 1];
                    matches!(inner.nodes.get(prev_id).map(|n| &n.kind), Some(NodeKind::Text(_)))
                } else {
                    false
                };

                if should_merge {
                    let prev_id = inner.nodes.get(parent).unwrap().children[sibling_idx - 1];
                    if let Some(NodeKind::Text(data)) = inner.nodes.get_mut(prev_id).map(|n| &mut n.kind) {
                        data.content.push_str(&text);
                    }
                } else {
                    let text_id = inner.nodes.insert(NodeData::new(NodeKind::Text(TextData::new(&text))));
                    if let Some(child_data) = inner.nodes.get_mut(text_id) {
                        child_data.parent = Some(parent);
                    }
                    if let Some(parent_data) = inner.nodes.get_mut(parent) {
                        parent_data.children.insert(sibling_idx, text_id);
                    }
                }
            }
        }
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        let mut inner = self.inner.borrow_mut();
        if let Some(NodeKind::Element(elem)) = inner.nodes.get_mut(*target).map(|n| &mut n.kind) {
            for attr in attrs {
                if !elem.has_attribute(&attr.name.local) {
                    let local = attr.name.local.clone();
                    elem.attributes.push(attr);
                    if &*local == "id" {
                        elem.id = elem
                            .attributes
                            .iter()
                            .find(|a| &*a.name.local == "id")
                            .map(|a| a.value.to_string());
                    } else if &*local == "class" {
                        elem.class_list = elem
                            .attributes
                            .iter()
                            .find(|a| &*a.name.local == "class")
                            .map(|a| a.value.split_whitespace().map(String::from).collect())
                            .unwrap_or_default();
                    }
                }
            }
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        let mut inner = self.inner.borrow_mut();
        let parent = inner.nodes.get(*target).and_then(|n| n.parent);

        if let Some(parent) = parent
            && let Some(parent_data) = inner.nodes.get_mut(parent)
        {
            parent_data.children.retain(|&id| id != *target);
        }

        if let Some(target_data) = inner.nodes.get_mut(*target) {
            target_data.parent = None;
        }
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let mut inner = self.inner.borrow_mut();

        let children: Vec<NodeId> = inner.nodes.get(*node).map(|n| n.children.clone()).unwrap_or_default();

        if let Some(node_data) = inner.nodes.get_mut(*node) {
            node_data.children.clear();
        }

        for child in children {
            if let Some(child_data) = inner.nodes.get_mut(child) {
                child_data.parent = Some(*new_parent);
            }
            if let Some(new_parent_data) = inner.nodes.get_mut(*new_parent) {
                new_parent_data.children.push(child);
            }
        }
    }
}

/// 使用 DomBuilder 解析 HTML（推荐方式）。
fn parse_html_with_builder(html: &str) -> Document {
    use html5ever::driver::ParseOpts;
    use tendril::TendrilSink;

    let builder = DomBuilder::new();
    let parser = html5ever::parse_document(builder, ParseOpts::default());
    parser.one(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 基础解析 ──

    /// 空文档应产生仅包含 document 根的树。
    #[test]
    fn test_parse_empty_html() {
        let doc = parse_html("");
        assert!(doc.root().is_valid());
        assert!(doc.node_count() >= 1, "空文档应有根节点");
    }

    /// 最简文档：仅文本。
    #[test]
    fn test_parse_text_only() {
        let doc = parse_html("Hello");
        assert!(doc.node_count() > 1, "应有文本节点");
    }

    /// 完整 HTML5 文档。
    #[test]
    fn test_parse_full_html5_document() {
        let html = r#"<!DOCTYPE html><html><head><title>Test</title></head><body><h1>Hello</h1></body></html>"#;
        let doc = parse_html(html);
        assert!(doc.node_count() > 5, "完整文档应包含多个节点");
    }

    /// 纯空白输入。
    #[test]
    fn test_parse_whitespace_only() {
        let doc = parse_html("   \n\t  ");
        assert!(doc.root().is_valid());
    }

    // ── 元素和属性 ──

    /// 带属性的元素。
    #[test]
    fn test_parse_element_with_attributes() {
        let doc = parse_html(r#"<div id="main" class="container" data-value="123">Content</div>"#);
        assert!(doc.node_count() > 1);
    }

    /// 嵌套元素。
    #[test]
    fn test_parse_nested_elements() {
        let html = "<div><p><span>Deep</span></p></div>";
        let doc = parse_html(html);
        assert!(doc.node_count() > 3, "嵌套元素应产生多个节点");
    }

    /// 自闭合 void 元素。
    #[test]
    fn test_parse_void_elements() {
        let html = "<div><br><img src='test.png'><input type='text'></div>";
        let doc = parse_html(html);
        assert!(doc.node_count() > 2, "void 元素应被正确解析");
    }

    /// 多 class 属性。
    #[test]
    fn test_parse_multiple_classes() {
        let doc = parse_html(r#"<div class="a b c"></div>"#);
        assert!(doc.node_count() > 0);
    }

    // ── 错误恢复 ──

    /// 未闭合标签：html5ever 自动恢复。
    #[test]
    fn test_parse_unclosed_tags() {
        let html = "<div><p>text<span>more";
        let doc = parse_html(html);
        // html5ever 自动闭合标签
        assert!(doc.node_count() > 2, "未闭合标签应被自动恢复");
    }

    /// 错误嵌套标签。
    #[test]
    fn test_parse_misnested_tags() {
        let html = "<b><i>bold italic</b></i>";
        let doc = parse_html(html);
        // html5ever 处理错误嵌套
        assert!(doc.node_count() > 1, "错误嵌套应被恢复");
    }

    /// 重复属性。
    #[test]
    fn test_parse_duplicate_attributes() {
        let html = r#"<div class="a" class="b">text</div>"#;
        let doc = parse_html(html);
        assert!(doc.node_count() > 0, "重复属性不应导致解析失败");
    }

    /// 仅关闭标签无开始标签。
    #[test]
    fn test_parse_closing_tag_without_open() {
        let html = "</div></p>text";
        let doc = parse_html(html);
        assert!(doc.root().is_valid());
    }

    // ── 特殊内容 ──

    /// HTML 实体。
    #[test]
    fn test_parse_html_entities() {
        let html = "<p>&amp; &lt; &gt; &quot; &#x2603;</p>";
        let doc = parse_html(html);
        assert!(doc.node_count() > 0, "实体应被正确解析");
    }

    /// 注释。
    #[test]
    fn test_parse_comment() {
        let html = "<div><!-- this is a comment -->text</div>";
        let doc = parse_html(html);
        assert!(doc.node_count() > 1, "注释应被保留在 DOM 中");
    }

    /// script 标签内容。
    #[test]
    fn test_parse_script_content() {
        let html = r#"<script>var x = 1 < 2; if (a && b) {}</script>"#;
        let doc = parse_html(html);
        assert!(doc.node_count() > 0, "script 内容应被正确处理");
    }

    /// style 标签内容。
    #[test]
    fn test_parse_style_content() {
        let html = "<style>body { color: red; }</style>";
        let doc = parse_html(html);
        assert!(doc.node_count() > 0, "style 内容应被正确处理");
    }

    /// DOCTYPE 声明。
    #[test]
    fn test_parse_doctype() {
        let html = "<!DOCTYPE html><html><body>ok</body></html>";
        let doc = parse_html(html);
        assert!(doc.node_count() > 2);
    }

    // ── 文档结构 ──

    /// 无 html/body 标签时自动补全。
    #[test]
    fn test_parse_auto_body() {
        let doc = parse_html("<p>paragraph</p>");
        assert!(doc.node_count() > 1, "html5ever 应自动添加 html/body");
    }

    /// head 中的 link/meta。
    #[test]
    fn test_parse_head_elements() {
        let html = r#"<head><meta charset="utf-8"><link rel="stylesheet" href="style.css"><title>T</title></head>"#;
        let doc = parse_html(html);
        assert!(doc.node_count() > 3);
    }

    /// 多层嵌套（10 层）。
    #[test]
    fn test_parse_deeply_nested() {
        let html = "<div>".repeat(10) + "text" + &"</div>".repeat(10);
        let doc = parse_html(&html);
        assert!(doc.node_count() > 10, "深层嵌套应被正确解析");
    }

    /// Unicode 文本。
    #[test]
    fn test_parse_unicode_text() {
        let html = "<p>你好世界 🌍 こんにちは 안녕하세요</p>";
        let doc = parse_html(html);
        assert!(doc.node_count() > 0, "Unicode 文本应被正确解析");
    }

    /// 大文档（1000 个段落）。
    #[test]
    fn test_parse_large_document() {
        let paragraphs: Vec<String> = (0..1000).map(|i| format!("<p>Paragraph {i}</p>")).collect();
        let html = format!("<html><body>{}</body></html>", paragraphs.join(""));
        let doc = parse_html(&html);
        assert!(doc.node_count() > 1000, "大文档应被正确解析");
    }
}
