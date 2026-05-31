//! DOM Document 实现 — 节点容器和树操作 API。

use crate::event::{Event, EventListenerFn, EventPhase, ListenerEntry};
use crate::mutation::{MutationObserver, MutationRecord, MutationType};
use crate::node::*;
use hashbrown::HashMap;
use slotmap::SlotMap;

// ── DocumentPosition ─────────────────────────────────────────────────

/// `compare_document_position` 返回的节点位置掩码常量。
///
/// 遵循 WHATWG DOM 规范中 `Node.compareDocumentPosition()` 的返回值定义。
/// 多个标志可以组合（如 `CONTAINS | PRECEDING`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentPosition(u8);

impl DocumentPosition {
    /// 两个节点在不同的文档或未连接的树中。
    pub const DISCONNECTED: u8 = 0x01;
    /// node2 在 node1 之前（文档顺序）。
    pub const PRECEDING: u8 = 0x02;
    /// node2 在 node1 之后（文档顺序）。
    pub const FOLLOWING: u8 = 0x04;
    /// node2 是 node1 的祖先。
    pub const CONTAINS: u8 = 0x08;
    /// node2 是 node1 的后代。
    pub const CONTAINED_BY: u8 = 0x10;

    /// 从原始 u8 值创建。
    #[inline]
    pub fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    /// 获取原始 u8 值。
    #[inline]
    pub fn bits(self) -> u8 {
        self.0
    }

    /// 检查是否包含指定标志。
    #[inline]
    pub fn contains(self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }
}

// ── Document ────────────────────────────────────────────────────────

/// DOM 文档，所有节点数据的容器。
///
/// 每个 `Document` 实例拥有独立的节点存储空间。
/// 通过 [`NodeId`] 引用节点，提供 O(1) 的节点查找性能。
pub struct Document {
    /// 节点存储（slotmap 提供稳定 ID 和 O(1) 查找）。
    nodes: SlotMap<NodeId, NodeData>,
    /// 文档根节点。
    root: NodeId,
    /// ID → NodeId 索引（用于 getElementById 快速查找）。
    id_map: HashMap<String, NodeId>,
    /// 已注册的 MutationObserver 列表。
    observers: Vec<MutationObserver>,
    /// 待处理的 mutation 记录。
    pending_mutations: Vec<MutationRecord>,
    /// 事件监听器存储：键为 (NodeId, event_type)，值为监听器列表。
    event_listeners: HashMap<(NodeId, String), Vec<ListenerEntry>>,
    /// 宿主元素 → ShadowRoot 节点映射。
    shadow_roots: HashMap<NodeId, NodeId>,
    /// Slot 分配：键为 (slot 元素 NodeId, slot 名)，值为已分配的 NodeId 列表。
    slot_assignments: HashMap<(NodeId, String), Vec<NodeId>>,
}

impl Document {
    /// 创建一个新的空文档。
    pub fn new() -> Self {
        let mut nodes = SlotMap::with_key();
        let root = nodes.insert(NodeData::new(NodeKind::Document(DocumentData {
            quirks_mode: QuirksMode::NoQuirks,
        })));

        Self {
            nodes,
            root,
            id_map: HashMap::new(),
            observers: Vec::new(),
            pending_mutations: Vec::new(),
            event_listeners: HashMap::new(),
            shadow_roots: HashMap::new(),
            slot_assignments: HashMap::new(),
        }
    }

    /// 获取文档根节点 ID。
    #[inline]
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// 获取文档中的节点总数。
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    // ── 节点查找 ────────────────────────────────────────────────

    /// 根据 NodeId 获取节点数据的不可变引用。
    ///
    /// 如果节点不存在（已被删除或 ID 无效），返回 `None`。
    #[inline]
    pub fn get(&self, id: NodeId) -> Option<&NodeData> {
        self.nodes.get(id)
    }

    /// 根据 NodeId 获取节点数据的可变引用。
    #[inline]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        self.nodes.get_mut(id)
    }

    /// 检查指定 NodeId 是否存在（节点未被删除）。
    #[inline]
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    // ── 节点创建 ────────────────────────────────────────────────

    /// 创建一个新的元素节点。
    pub fn create_element(&mut self, name: &str) -> NodeId {
        use markup5ever::{LocalName, Namespace, QualName};

        let qual_name = QualName::new(
            None,
            Namespace::from("http://www.w3.org/1999/xhtml"),
            LocalName::from(name),
        );
        self.create_element_with_qname(qual_name, Vec::new())
    }

    /// 使用限定名和属性创建元素节点（内部使用）。
    pub(crate) fn create_element_with_qname(
        &mut self,
        name: markup5ever::QualName,
        attrs: Vec<markup5ever::Attribute>,
    ) -> NodeId {
        let elem_data = ElementData::new(name, attrs);
        let node_id = self
            .nodes
            .insert(NodeData::new(NodeKind::Element(elem_data)));

        // 注册 id 映射
        if let Some(NodeKind::Element(elem)) = self.nodes.get(node_id).map(|n| &n.kind)
            && let Some(id) = &elem.id
        {
            self.id_map.insert(id.clone(), node_id);
        }

        node_id
    }

    /// 创建一个新的文本节点。
    pub fn create_text_node(&mut self, text: &str) -> NodeId {
        self.nodes
            .insert(NodeData::new(NodeKind::Text(TextData::new(text))))
    }

    /// 创建一个新的注释节点。
    pub fn create_comment(&mut self, text: &str) -> NodeId {
        self.nodes
            .insert(NodeData::new(NodeKind::Comment(CommentData::new(text))))
    }

    /// 创建一个新的文档片段。
    pub fn create_document_fragment(&mut self) -> NodeId {
        self.nodes.insert(NodeData::new(NodeKind::DocumentFragment))
    }

    /// 创建一个文档类型声明节点。
    pub fn create_document_type(
        &mut self,
        name: &str,
        public_id: Option<String>,
        system_id: Option<String>,
    ) -> NodeId {
        self.nodes
            .insert(NodeData::new(NodeKind::DocumentType(DocumentTypeData {
                name: name.to_string(),
                public_id,
                system_id,
            })))
    }

    /// 创建一个处理指令节点。
    pub fn create_processing_instruction(&mut self, target: &str, data: &str) -> NodeId {
        self.nodes
            .insert(NodeData::new(NodeKind::ProcessingInstruction(
                ProcessingInstructionData {
                    target: target.to_string(),
                    data: data.to_string(),
                },
            )))
    }

    // ── 树操作 ──────────────────────────────────────────────────

    /// 将子节点追加到父节点的子列表末尾。
    ///
    /// 如果子节点已有父节点，会先从原父节点中移除。
    ///
    /// # 错误
    ///
    /// - 父节点或子节点不存在
    /// - 试图将文档根节点作为子节点追加
    /// - 试图将一个节点的祖先作为其子节点追加（循环检测）
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), DomError> {
        if !self.contains(parent) {
            return Err(DomError::NodeNotFound(parent));
        }
        if !self.contains(child) {
            return Err(DomError::NodeNotFound(child));
        }
        if child == self.root {
            return Err(DomError::CannotInsertDocumentRoot);
        }
        if self.is_ancestor(child, parent) {
            return Err(DomError::WouldCreateCycle);
        }

        // 如果 child 已有父节点，先从原父节点移除
        self.detach(child);

        // 设置新父节点
        if let Some(child_data) = self.nodes.get_mut(child) {
            child_data.parent = Some(parent);
        }

        // 追加到父节点的子列表
        if let Some(parent_data) = self.nodes.get_mut(parent) {
            parent_data.children.push(child);
        }

        // 记录 mutation
        self.record_mutation(MutationRecord {
            mutation_type: MutationType::ChildList,
            target: parent,
            added_nodes: vec![child],
            removed_nodes: vec![],
            previous_sibling: self.prev_sibling_of_last_child(parent),
            attribute_name: None,
            old_value: None,
        });

        Ok(())
    }

    /// 从父节点中移除指定的子节点。
    ///
    /// 返回被移除的节点 ID。
    ///
    /// # 错误
    ///
    /// - 父节点或子节点不存在
    /// - 子节点不是父节点的子节点
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) -> Result<NodeId, DomError> {
        if !self.contains(parent) {
            return Err(DomError::NodeNotFound(parent));
        }
        if !self.contains(child) {
            return Err(DomError::NodeNotFound(child));
        }

        // 验证 child 确实是 parent 的子节点
        let is_child = self
            .nodes
            .get(parent)
            .map(|p| p.children.contains(&child))
            .unwrap_or(false);

        if !is_child {
            return Err(DomError::NotAChild { parent, child });
        }

        // 记录 mutation（在移除前）
        self.record_mutation(MutationRecord {
            mutation_type: MutationType::ChildList,
            target: parent,
            added_nodes: vec![],
            removed_nodes: vec![child],
            previous_sibling: self.prev_sibling_of(child),
            attribute_name: None,
            old_value: None,
        });

        // 从父节点子列表中移除
        if let Some(parent_data) = self.nodes.get_mut(parent) {
            parent_data.children.retain(|&id| id != child);
        }

        // 清除子节点的父引用
        if let Some(child_data) = self.nodes.get_mut(child) {
            child_data.parent = None;
        }

        Ok(child)
    }

    /// 在参考节点之前插入新节点。
    ///
    /// # 错误
    ///
    /// - 父节点、新节点或参考节点不存在
    /// - 参考节点不是父节点的子节点
    /// - 循环检测
    pub fn insert_before(
        &mut self,
        parent: NodeId,
        new_node: NodeId,
        ref_node: NodeId,
    ) -> Result<(), DomError> {
        if !self.contains(parent) || !self.contains(new_node) || !self.contains(ref_node) {
            return Err(DomError::NodeNotFound(parent));
        }
        if new_node == self.root {
            return Err(DomError::CannotInsertDocumentRoot);
        }
        if self.is_ancestor(new_node, parent) {
            return Err(DomError::WouldCreateCycle);
        }

        // 找到 ref_node 在 parent.children 中的位置
        let ref_idx = self
            .nodes
            .get(parent)
            .and_then(|p| p.children.iter().position(|&id| id == ref_node))
            .ok_or(DomError::NotAChild {
                parent,
                child: ref_node,
            })?;

        // 如果 new_node 已有父节点，先从原父节点移除
        self.detach(new_node);

        // 设置新父节点
        if let Some(node_data) = self.nodes.get_mut(new_node) {
            node_data.parent = Some(parent);
        }

        // 插入到参考节点之前
        if let Some(parent_data) = self.nodes.get_mut(parent) {
            parent_data.children.insert(ref_idx, new_node);
        }

        // 记录 mutation
        self.record_mutation(MutationRecord {
            mutation_type: MutationType::ChildList,
            target: parent,
            added_nodes: vec![new_node],
            removed_nodes: vec![],
            previous_sibling: self.prev_sibling_of(new_node),
            attribute_name: None,
            old_value: None,
        });

        Ok(())
    }

    /// 用新节点替换旧节点。
    ///
    /// 返回被替换的旧节点 ID。
    pub fn replace_child(
        &mut self,
        parent: NodeId,
        new_child: NodeId,
        old_child: NodeId,
    ) -> Result<NodeId, DomError> {
        if !self.contains(parent) || !self.contains(new_child) || !self.contains(old_child) {
            return Err(DomError::NodeNotFound(parent));
        }

        let old_idx = self
            .nodes
            .get(parent)
            .and_then(|p| p.children.iter().position(|&id| id == old_child))
            .ok_or(DomError::NotAChild {
                parent,
                child: old_child,
            })?;

        // 如果 new_child 已有父节点，先从原父节点移除
        self.detach(new_child);

        // 记录 mutation（在替换前）
        self.record_mutation(MutationRecord {
            mutation_type: MutationType::ChildList,
            target: parent,
            added_nodes: vec![new_child],
            removed_nodes: vec![old_child],
            previous_sibling: None,
            attribute_name: None,
            old_value: None,
        });

        // 从父节点子列表中移除 old_child
        if let Some(parent_data) = self.nodes.get_mut(parent) {
            parent_data.children[old_idx] = new_child;
        }

        // 设置新节点的父引用
        if let Some(node_data) = self.nodes.get_mut(new_child) {
            node_data.parent = Some(parent);
        }

        // 清除旧节点的父引用
        if let Some(node_data) = self.nodes.get_mut(old_child) {
            node_data.parent = None;
        }

        Ok(old_child)
    }

    /// 克隆节点。如果 `deep` 为 true，递归克隆所有子孙节点。
    pub fn clone_node(&mut self, node: NodeId, deep: bool) -> NodeId {
        let cloned_kind = match self.nodes.get(node).map(|n| n.kind.clone()) {
            Some(kind) => kind,
            None => return node, // fallback: 如果节点不存在，返回自身
        };

        let new_id = self.nodes.insert(NodeData::new(cloned_kind));

        // 注册 id 映射（如果克隆的元素有 id）
        if let Some(NodeKind::Element(elem)) = self.nodes.get(new_id).map(|n| &n.kind)
            && let Some(id) = &elem.id
        {
            self.id_map.insert(id.clone(), new_id);
        }

        if deep && let Some(children) = self.nodes.get(node).map(|n| n.children.clone()) {
            for child in children {
                let cloned_child = self.clone_node(child, true);
                // 直接追加（不触发 mutation）
                if let Some(node_data) = self.nodes.get_mut(new_id) {
                    node_data.children.push(cloned_child);
                }
                if let Some(child_data) = self.nodes.get_mut(cloned_child) {
                    child_data.parent = Some(new_id);
                }
            }
        }

        new_id
    }

    // ── 遍历 ────────────────────────────────────────────────────

    /// 获取父节点 ID。
    pub fn parent_node(&self, id: NodeId) -> Option<NodeId> {
        self.nodes.get(id).and_then(|n| n.parent)
    }

    /// 获取第一个子节点 ID。
    pub fn first_child(&self, id: NodeId) -> Option<NodeId> {
        self.nodes.get(id).and_then(|n| n.first_child())
    }

    /// 获取最后一个子节点 ID。
    pub fn last_child(&self, id: NodeId) -> Option<NodeId> {
        self.nodes.get(id).and_then(|n| n.last_child())
    }

    /// 获取下一个兄弟节点 ID。
    pub fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        let parent = self.nodes.get(id).and_then(|n| n.parent)?;
        let siblings = &self.nodes.get(parent)?.children;
        let idx = siblings.iter().position(|&s| s == id)?;
        siblings.get(idx + 1).copied()
    }

    /// 获取上一个兄弟节点 ID。
    pub fn previous_sibling(&self, id: NodeId) -> Option<NodeId> {
        let parent = self.nodes.get(id).and_then(|n| n.parent)?;
        let siblings = &self.nodes.get(parent)?.children;
        let idx = siblings.iter().position(|&s| s == id)?;
        if idx > 0 {
            Some(siblings[idx - 1])
        } else {
            None
        }
    }

    /// 获取所有子节点 ID 列表（按文档顺序）。
    pub fn child_nodes(&self, id: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(id)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    /// 检查节点是否有子节点。
    pub fn has_child_nodes(&self, id: NodeId) -> bool {
        self.nodes
            .get(id)
            .map(|n| n.has_children())
            .unwrap_or(false)
    }

    // ── 遍历扩展 ──────────────────────────────────────────────

    /// 检查 `container` 是否包含 `node`（即 node 是 container 自身或其后代）。
    pub fn node_contains(&self, container: NodeId, node: NodeId) -> bool {
        if container == node {
            return true;
        }
        self.is_ancestor(container, node)
    }

    /// 比较两个节点在文档中的相对位置，返回位置掩码。
    ///
    /// 遵循 WHATWG DOM 规范 `Node.compareDocumentPosition()` 语义。
    /// 如果任一节点不存在，返回 `None`。
    pub fn compare_document_position(
        &self,
        node1: NodeId,
        node2: NodeId,
    ) -> Option<DocumentPosition> {
        if !self.contains(node1) || !self.contains(node2) {
            return None;
        }
        if node1 == node2 {
            return Some(DocumentPosition::from_bits(0));
        }

        // 检查包含关系
        let node1_contains_node2 = self.is_ancestor(node1, node2);
        let node2_contains_node1 = self.is_ancestor(node2, node1);

        if node1_contains_node2 {
            // node2 是 node1 的后代 → node2 在 node1 之后，且被 node1 包含
            return Some(DocumentPosition::from_bits(
                DocumentPosition::CONTAINED_BY | DocumentPosition::FOLLOWING,
            ));
        }
        if node2_contains_node1 {
            // node2 是 node1 的祖先 → node2 在 node1 之前，且包含 node1
            return Some(DocumentPosition::from_bits(
                DocumentPosition::CONTAINS | DocumentPosition::PRECEDING,
            ));
        }

        // 既不包含也不被包含：找最近公共祖先，比较在兄弟列表中的顺序
        let pos = self.compare_tree_position(node1, node2);
        Some(DocumentPosition::from_bits(pos))
    }

    /// 收集 `root` 的所有后代节点，按文档顺序返回。
    pub fn collect_descendants(&self, root: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        self.collect_descendants_recursive(root, &mut result);
        result
    }

    /// 获取节点到文档根的距离（根节点深度为 0）。
    pub fn depth(&self, node: NodeId) -> Option<usize> {
        if !self.contains(node) {
            return None;
        }
        let mut depth = 0;
        let mut current = node;
        while let Some(parent) = self.nodes.get(current).and_then(|n| n.parent) {
            depth += 1;
            current = parent;
        }
        Some(depth)
    }

    /// 获取节点的直接子节点数量。
    pub fn child_count(&self, node: NodeId) -> usize {
        self.nodes
            .get(node)
            .map(|n| n.children.len())
            .unwrap_or(0)
    }

    /// 获取 WHATWG 节点类型编号。
    ///
    /// 返回值：1=Element, 3=Text, 7=ProcessingInstruction,
    /// 8=Comment, 9=Document, 10=DocumentType, 11=DocumentFragment。
    /// 节点不存在时返回 `None`。
    pub fn node_type(&self, node: NodeId) -> Option<u8> {
        self.nodes.get(node).map(|n| match &n.kind {
            NodeKind::Element(_) => 1,
            NodeKind::Text(_) => 3,
            NodeKind::ProcessingInstruction(_) => 7,
            NodeKind::Comment(_) => 8,
            NodeKind::Document(_) => 9,
            NodeKind::DocumentType(_) => 10,
            NodeKind::DocumentFragment => 11,
            NodeKind::ShadowRoot(_) => 11,
        })
    }

    /// 获取节点所属的文档根节点 ID。
    ///
    /// 沿 parent 链向上走到顶端，返回该根节点。
    /// 节点不存在时返回 `None`。
    pub fn owner_document(&self, node: NodeId) -> Option<NodeId> {
        if !self.contains(node) {
            return None;
        }
        let mut current = node;
        while let Some(parent) = self.nodes.get(current).and_then(|n| n.parent) {
            current = parent;
        }
        Some(current)
    }

    // ── 文本内容 ────────────────────────────────────────────────

    /// 获取节点及其子孙的文本内容（递归拼接所有文本节点）。
    pub fn text_content(&self, id: NodeId) -> Option<String> {
        let node_data = self.nodes.get(id)?;
        match &node_data.kind {
            NodeKind::Text(data) => Some(data.content.clone()),
            NodeKind::Comment(data) => Some(data.content.clone()),
            NodeKind::ProcessingInstruction(data) => Some(data.data.clone()),
            NodeKind::Element(_) | NodeKind::Document(_) | NodeKind::DocumentFragment | NodeKind::ShadowRoot(_) => {
                let mut result = String::new();
                self.collect_text(id, &mut result);
                Some(result)
            }
            NodeKind::DocumentType(_) => None,
        }
    }

    /// 设置节点的文本内容。
    ///
    /// 对于元素节点，会清除所有子节点并创建一个文本节点。
    /// 对于文本节点，直接更新内容。
    pub fn set_text_content(&mut self, id: NodeId, text: &str) {
        if let Some(node_data) = self.nodes.get(id) {
            match &node_data.kind {
                NodeKind::Text(_) => {
                    if let Some(NodeKind::Text(data)) = self.nodes.get_mut(id).map(|n| &mut n.kind)
                    {
                        data.content = text.to_string();
                    }
                }
                NodeKind::Comment(_) => {
                    if let Some(NodeKind::Comment(data)) =
                        self.nodes.get_mut(id).map(|n| &mut n.kind)
                    {
                        data.content = text.to_string();
                    }
                }
                NodeKind::Element(_) | NodeKind::DocumentFragment | NodeKind::ShadowRoot(_) => {
                    // 清除所有子节点
                    let children: Vec<NodeId> = self
                        .nodes
                        .get(id)
                        .map(|n| n.children.clone())
                        .unwrap_or_default();

                    for child in &children {
                        if let Some(child_data) = self.nodes.get_mut(*child) {
                            child_data.parent = None;
                        }
                    }

                    if let Some(node_data) = self.nodes.get_mut(id) {
                        node_data.children.clear();
                    }

                    // 如果文本非空，创建新的文本节点
                    if !text.is_empty() {
                        let text_id = self.create_text_node(text);
                        if let Some(node_data) = self.nodes.get_mut(id) {
                            node_data.children.push(text_id);
                        }
                        if let Some(text_data) = self.nodes.get_mut(text_id) {
                            text_data.parent = Some(id);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // ── 属性操作（便捷方法） ─────────────────────────────────────

    /// 获取元素节点的指定属性值。
    pub fn get_attribute(&self, id: NodeId, name: &str) -> Option<String> {
        self.nodes.get(id).and_then(|n| match &n.kind {
            NodeKind::Element(elem) => elem.get_attribute(name),
            _ => None,
        })
    }

    /// 设置元素节点的属性值。
    pub fn set_attribute(&mut self, id: NodeId, name: &str, value: &str) {
        let old_value = self.get_attribute(id, name);

        if let Some(NodeKind::Element(elem)) = self.nodes.get_mut(id).map(|n| &mut n.kind) {
            elem.set_attribute(name, value);

            // 更新 id 映射
            if name == "id" {
                if let Some(old_id) = &old_value {
                    self.id_map.remove(old_id);
                }
                self.id_map.insert(value.to_string(), id);
            }
        }

        // 记录 mutation
        self.record_mutation(MutationRecord {
            mutation_type: MutationType::Attributes,
            target: id,
            added_nodes: vec![],
            removed_nodes: vec![],
            previous_sibling: None,
            attribute_name: Some(name.to_string()),
            old_value,
        });
    }

    /// 移除元素节点的指定属性。
    pub fn remove_attribute(&mut self, id: NodeId, name: &str) {
        let old_value = self.get_attribute(id, name);

        if let Some(NodeKind::Element(elem)) = self.nodes.get_mut(id).map(|n| &mut n.kind) {
            elem.remove_attribute(name);

            // 更新 id 映射
            if name == "id"
                && let Some(old_id) = &old_value
            {
                self.id_map.remove(old_id);
            }
        }

        if old_value.is_some() {
            self.record_mutation(MutationRecord {
                mutation_type: MutationType::Attributes,
                target: id,
                added_nodes: vec![],
                removed_nodes: vec![],
                previous_sibling: None,
                attribute_name: Some(name.to_string()),
                old_value,
            });
        }
    }

    /// 检查元素节点是否有指定属性。
    pub fn has_attribute(&self, id: NodeId, name: &str) -> bool {
        self.nodes
            .get(id)
            .map(|n| match &n.kind {
                NodeKind::Element(elem) => elem.has_attribute(name),
                _ => false,
            })
            .unwrap_or(false)
    }

    /// 获取元素节点的所有属性名。
    pub fn attribute_names(&self, id: NodeId) -> Vec<String> {
        self.nodes
            .get(id)
            .map(|n| match &n.kind {
                NodeKind::Element(elem) => elem.attribute_names(),
                _ => vec![],
            })
            .unwrap_or_default()
    }

    // ── 查询 ────────────────────────────────────────────────────

    /// 根据 ID 查找元素节点（O(1) 通过索引）。
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        self.id_map.get(id).copied()
    }

    /// 根据标签名查找所有匹配的元素节点。
    pub fn get_elements_by_tag_name(&self, tag: &str) -> Vec<NodeId> {
        let mut result = Vec::new();
        self.collect_by_tag_name(self.root, tag, &mut result);
        result
    }

    /// 根据类名查找所有匹配的元素节点。
    pub fn get_elements_by_class_name(&self, class: &str) -> Vec<NodeId> {
        let mut result = Vec::new();
        self.collect_by_class_name(self.root, class, &mut result);
        result
    }

    /// 在指定节点的子树中查找第一个匹配基础选择器的元素。
    ///
    /// 支持的选择器格式：
    /// - 标签名：`"div"`
    /// - ID：`"#myid"`
    /// - 类名：`".myclass"`
    /// - 属性：`"[attr]"` 或 `"[attr=value]"`
    pub fn query_selector(&self, root: NodeId, selector: &str) -> Option<NodeId> {
        let parsed = crate::query::parse_simple_selector(selector)?;
        self.find_first_matching(root, &parsed)
    }

    /// 在指定节点的子树中查找所有匹配基础选择器的元素。
    pub fn query_selector_all(&self, root: NodeId, selector: &str) -> Vec<NodeId> {
        let parsed = match crate::query::parse_simple_selector(selector) {
            Some(s) => s,
            None => return vec![],
        };
        let mut result = Vec::new();
        self.collect_matching(root, &parsed, &mut result);
        result
    }

    // ── Shadow DOM ──────────────────────────────────────────────

    /// 为宿主元素附加 ShadowRoot。
    ///
    /// 创建一个新的 ShadowRoot 节点并附加到指定的宿主元素上。
    /// 返回错误如果：宿主不是元素节点，或宿主已有 ShadowRoot。
    pub fn attach_shadow(
        &mut self,
        host: NodeId,
        mode: ShadowRootMode,
    ) -> Result<NodeId, DomError> {
        // 验证宿主是元素节点
        let is_element = self
            .nodes
            .get(host)
            .map(|n| matches!(n.kind, NodeKind::Element(_)))
            .unwrap_or(false);
        if !is_element {
            return Err(DomError::NotAnElement);
        }

        // 验证宿主没有已有 ShadowRoot
        if self.shadow_roots.contains_key(&host) {
            return Err(DomError::AlreadyHasShadowRoot);
        }

        // 创建 ShadowRoot 节点
        let shadow_data = ShadowRootData::new(mode);
        let shadow_id = self
            .nodes
            .insert(NodeData::new(NodeKind::ShadowRoot(shadow_data)));

        // 设置宿主引用
        if let Some(NodeKind::ShadowRoot(data)) =
            self.nodes.get_mut(shadow_id).map(|n| &mut n.kind)
        {
            data.host = Some(host);
        }

        // 将 ShadowRoot 作为宿主的子节点追加
        // （ShadowRoot 存在于宿主的子列表中，但封装边界阻止外部查询穿透）
        if let Some(host_data) = self.nodes.get_mut(host) {
            host_data.children.push(shadow_id);
        }
        if let Some(shadow_data) = self.nodes.get_mut(shadow_id) {
            shadow_data.parent = Some(host);
        }

        // 注册映射
        self.shadow_roots.insert(host, shadow_id);

        Ok(shadow_id)
    }

    /// 获取宿主元素附加的 ShadowRoot，如果没有则返回 `None`。
    pub fn shadow_root(&self, host: NodeId) -> Option<NodeId> {
        self.shadow_roots.get(&host).copied()
    }

    /// 获取 ShadowRoot 的封装模式。
    ///
    /// 如果节点不是 ShadowRoot，返回 `None`。
    pub fn get_shadow_root_mode(&self, shadow_root: NodeId) -> Option<ShadowRootMode> {
        self.nodes.get(shadow_root).and_then(|n| match &n.kind {
            NodeKind::ShadowRoot(data) => Some(data.mode),
            _ => None,
        })
    }

    /// 获取 slot 元素已分配的节点列表。
    ///
    /// 查找分配给指定 slot 的 light DOM 节点。
    /// `slot_name` 对应 slot 元素的 `name` 属性值。
    pub fn assigned_nodes(&self, slot_element: NodeId, slot_name: &str) -> Vec<NodeId> {
        self.slot_assignments
            .get(&(slot_element, slot_name.to_string()))
            .cloned()
            .unwrap_or_default()
    }

    /// 将 light DOM 节点分配给指定的 slot。
    ///
    /// `slot_element` 是 `<slot>` 元素的 NodeId，
    /// `slot_name` 是 slot 的 `name` 属性值。
    pub fn assign_slot(&mut self, slot_element: NodeId, slot_name: &str, node: NodeId) {
        self.slot_assignments
            .entry((slot_element, slot_name.to_string()))
            .or_default()
            .push(node);
    }

    /// 在 ShadowRoot 子树中查找第一个匹配选择器的元素。
    ///
    /// 类似 `query_selector`，但范围限定在 shadow DOM 树内，
    /// 不会穿透到嵌套的 ShadowRoot 边界。
    pub fn query_selector_shadow(
        &self,
        shadow_root: NodeId,
        selector: &str,
    ) -> Option<NodeId> {
        let parsed = crate::query::parse_simple_selector(selector)?;
        self.find_first_matching_shadow(shadow_root, &parsed)
    }

    /// 在 ShadowRoot 子树中查找所有匹配选择器的元素。
    ///
    /// 类似 `query_selector_all`，但范围限定在 shadow DOM 树内，
    /// 不会穿透到嵌套的 ShadowRoot 边界。
    pub fn query_selector_all_shadow(
        &self,
        shadow_root: NodeId,
        selector: &str,
    ) -> Vec<NodeId> {
        let parsed = match crate::query::parse_simple_selector(selector) {
            Some(s) => s,
            None => return vec![],
        };
        let mut result = Vec::new();
        self.collect_matching_shadow(shadow_root, &parsed, &mut result);
        result
    }

    // ── quirks mode ─────────────────────────────────────────────

    /// 获取文档的 quirks mode。
    pub fn quirks_mode(&self) -> QuirksMode {
        match &self.nodes.get(self.root).map(|n| &n.kind) {
            Some(NodeKind::Document(data)) => data.quirks_mode,
            _ => QuirksMode::NoQuirks,
        }
    }

    /// 设置文档的 quirks mode。
    pub fn set_quirks_mode(&mut self, mode: QuirksMode) {
        if let Some(NodeKind::Document(data)) = self.nodes.get_mut(self.root).map(|n| &mut n.kind) {
            data.quirks_mode = mode;
        }
    }

    // ── MutationObserver ────────────────────────────────────────

    /// 注册 MutationObserver。
    pub fn add_observer(&mut self, observer: MutationObserver) {
        self.observers.push(observer);
    }

    /// 获取待处理的 mutation 记录。
    pub fn take_mutation_records(&mut self) -> Vec<MutationRecord> {
        std::mem::take(&mut self.pending_mutations)
    }

    /// 处理待处理的 mutation 记录（通知所有 observer）。
    pub fn process_mutations(&mut self) {
        let records: Vec<MutationRecord> = std::mem::take(&mut self.pending_mutations);
        for observer in &self.observers {
            observer.notify(&records);
        }
    }

    // ── 事件系统 ─────────────────────────────────────────────────

    /// 为指定节点添加事件监听器。
    ///
    /// `event_type` 是事件类型名（如 "click"、"input"）。
    /// `callback` 是事件触发时的回调函数。
    /// `capture` 为 true 时监听器在捕获阶段触发，否则在冒泡阶段触发。
    pub fn add_event_listener(
        &mut self,
        node: NodeId,
        event_type: &str,
        callback: EventListenerFn,
        capture: bool,
    ) {
        let key = (node, event_type.to_string());
        self.event_listeners
            .entry(key)
            .or_default()
            .push(ListenerEntry { callback, capture });
    }

    /// 移除指定节点上的所有指定类型的事件监听器。
    ///
    /// 返回被移除的监听器数量。
    pub fn remove_event_listener(&mut self, node: NodeId, event_type: &str) -> usize {
        let key = (node, event_type.to_string());
        self.event_listeners.remove(&key).map(|v| v.len()).unwrap_or(0)
    }

    /// 移除指定节点上的所有事件监听器。
    pub fn remove_all_event_listeners(&mut self, node: NodeId) {
        self.event_listeners.retain(|(n, _), _| *n != node);
    }

    /// 向指定节点派发事件。
    ///
    /// 根据 `event.bubbles()` 决定是否冒泡传播：
    /// 1. 捕获阶段：从文档根到目标节点的路径上，触发 `capture = true` 的监听器
    /// 2. 目标阶段：在目标节点上触发所有监听器
    /// 3. 冒泡阶段：从目标节点到文档根的路径上，触发 `capture = false` 的监听器
    ///
    /// 返回 `true` 表示事件未被取消（默认行为可执行），`false` 表示 `preventDefault()` 被调用。
    pub fn dispatch_event(&self, target: NodeId, event: &mut Event) -> bool {
        event.init_for_dispatch(target);

        // 构建从根到目标的祖先路径（不含目标自身）
        let path = self.ancestor_path(target);

        // 1. 捕获阶段：从根向目标方向（path 已是从根到目标的顺序）
        event.phase = EventPhase::Capturing;
        for &node in &path {
            event.current_target = Some(node);
            self.invoke_listeners(node, event, true);
            if event.propagation_stopped() {
                break;
            }
        }

        // 2. 目标阶段
        if !event.propagation_stopped() {
            event.phase = EventPhase::AtTarget;
            event.current_target = Some(target);
            self.invoke_listeners(target, event, false);
        }

        // 3. 冒泡阶段：从目标向根方向（反序遍历 path）
        if event.bubbles() && !event.propagation_stopped() {
            event.phase = EventPhase::Bubbling;
            for &node in path.iter().rev() {
                event.current_target = Some(node);
                self.invoke_listeners(node, event, false);
                if event.propagation_stopped() {
                    break;
                }
            }
        }

        event.current_target = None;
        !event.default_prevented()
    }

    /// 获取指定节点上已注册的指定类型监听器数量。
    pub fn listener_count(&self, node: NodeId, event_type: &str) -> usize {
        let key = (node, event_type.to_string());
        self.event_listeners.get(&key).map(|v| v.len()).unwrap_or(0)
    }

    /// 调用节点上的监听器。
    ///
    /// `capture_only` 为 true 时只调用捕获阶段的监听器，
    /// 为 false 时在目标阶段调用所有监听器、在冒泡阶段只调用非捕获监听器。
    ///
    /// `stopPropagation` 允许当前节点上剩余的监听器继续执行，但阻止传播到其他节点。
    /// `stopImmediatePropagation` 立即停止，不再调用当前节点上的后续监听器。
    fn invoke_listeners(&self, node: NodeId, event: &mut Event, capture_only: bool) {
        let key = (node, event.event_type().to_string());
        let listeners = match self.event_listeners.get(&key) {
            Some(l) => l,
            None => return,
        };

        for entry in listeners {
            // 捕获阶段只调用 capture=true 的监听器
            // 目标和冒泡阶段只调用 capture=false 的监听器
            // 但在目标阶段（AtTarget），两种都调用
            let at_target = event.phase() == EventPhase::AtTarget;
            if capture_only && !entry.capture {
                continue;
            }
            if !capture_only && !at_target && entry.capture {
                continue;
            }

            // stopImmediatePropagation 已设置则不再调用后续监听器
            if event.immediate_propagation_stopped() {
                break;
            }

            (entry.callback)(event);
        }
    }

    /// 获取从节点到文档根的祖先路径（不含文档根，不含节点自身）。
    fn ancestor_path(&self, node: NodeId) -> Vec<NodeId> {
        let mut path = Vec::new();
        let mut current = node;
        while let Some(parent) = self.nodes.get(current).and_then(|n| n.parent) {
            path.push(parent);
            current = parent;
        }
        path.reverse();
        path
    }

    // ── 内部辅助方法 ────────────────────────────────────────────

    /// 从父节点分离节点（不删除节点本身）。
    fn detach(&mut self, id: NodeId) {
        let old_parent = self.nodes.get(id).and_then(|n| n.parent);
        if let Some(parent) = old_parent
            && let Some(parent_data) = self.nodes.get_mut(parent)
        {
            parent_data.children.retain(|&c| c != id);
        }
        if let Some(node_data) = self.nodes.get_mut(id) {
            node_data.parent = None;
        }
    }

    /// 检查 `ancestor` 是否是 `descendant` 的祖先。
    fn is_ancestor(&self, ancestor: NodeId, descendant: NodeId) -> bool {
        let mut current = descendant;
        let mut visited = 0;
        let max_depth = self.nodes.len();

        while let Some(node) = self.nodes.get(current) {
            if current == ancestor {
                return true;
            }
            match node.parent {
                Some(p) => {
                    current = p;
                    visited += 1;
                    if visited > max_depth {
                        break; // 防止循环
                    }
                }
                None => break,
            }
        }
        false
    }

    /// 递归收集文本内容。
    fn collect_text(&self, id: NodeId, result: &mut String) {
        let node_data = match self.nodes.get(id) {
            Some(n) => n,
            None => return,
        };

        match &node_data.kind {
            NodeKind::Text(data) => {
                result.push_str(&data.content);
            }
            NodeKind::Element(_) | NodeKind::Document(_) | NodeKind::DocumentFragment | NodeKind::ShadowRoot(_) => {
                for &child in &node_data.children {
                    self.collect_text(child, result);
                }
            }
            _ => {}
        }
    }

    /// 递归收集指定标签名的元素。
    fn collect_by_tag_name(&self, id: NodeId, tag: &str, result: &mut Vec<NodeId>) {
        let node_data = match self.nodes.get(id) {
            Some(n) => n,
            None => return,
        };

        if let NodeKind::Element(elem) = &node_data.kind
            && elem.local_name().eq_ignore_ascii_case(tag)
        {
            result.push(id);
        }

        for &child in &node_data.children.clone() {
            self.collect_by_tag_name(child, tag, result);
        }
    }

    /// 递归收集指定类名的元素。
    fn collect_by_class_name(&self, id: NodeId, class: &str, result: &mut Vec<NodeId>) {
        let node_data = match self.nodes.get(id) {
            Some(n) => n,
            None => return,
        };

        if let NodeKind::Element(elem) = &node_data.kind
            && elem.class_list.iter().any(|c| c == class)
        {
            result.push(id);
        }

        for &child in &node_data.children.clone() {
            self.collect_by_class_name(child, class, result);
        }
    }

    /// 获取节点的上一个兄弟节点（内部）。
    fn prev_sibling_of(&self, id: NodeId) -> Option<NodeId> {
        self.previous_sibling(id)
    }

    /// 获取父节点最后一个子节点的上一个兄弟（用于 mutation 记录）。
    fn prev_sibling_of_last_child(&self, parent: NodeId) -> Option<NodeId> {
        let children = self.nodes.get(parent).map(|n| &n.children)?;
        if children.len() >= 2 {
            Some(children[children.len() - 2])
        } else {
            None
        }
    }

    /// 记录 mutation（内部）。
    fn record_mutation(&mut self, record: MutationRecord) {
        self.pending_mutations.push(record);
    }

    /// 查找第一个匹配的节点。
    fn find_first_matching(
        &self,
        id: NodeId,
        selector: &crate::query::SimpleSelector,
    ) -> Option<NodeId> {
        let node_data = self.nodes.get(id)?;
        if let NodeKind::Element(elem) = &node_data.kind
            && selector.matches(elem)
        {
            return Some(id);
        }
        for &child in &node_data.children.clone() {
            if let Some(found) = self.find_first_matching(child, selector) {
                return Some(found);
            }
        }
        None
    }

    /// 收集所有匹配的节点。
    fn collect_matching(
        &self,
        id: NodeId,
        selector: &crate::query::SimpleSelector,
        result: &mut Vec<NodeId>,
    ) {
        let node_data = match self.nodes.get(id) {
            Some(n) => n,
            None => return,
        };

        if let NodeKind::Element(elem) = &node_data.kind
            && selector.matches(elem)
        {
            result.push(id);
        }

        for &child in &node_data.children.clone() {
            self.collect_matching(child, selector, result);
        }
    }

    /// 在 shadow DOM 内查找第一个匹配的元素，不穿透嵌套的 ShadowRoot 边界。
    fn find_first_matching_shadow(
        &self,
        id: NodeId,
        selector: &crate::query::SimpleSelector,
    ) -> Option<NodeId> {
        let node_data = self.nodes.get(id)?;
        if let NodeKind::Element(elem) = &node_data.kind
            && selector.matches(elem)
        {
            return Some(id);
        }
        for &child in &node_data.children.clone() {
            // 不进入嵌套的 ShadowRoot
            if let Some(child_data) = self.nodes.get(child)
                && matches!(child_data.kind, NodeKind::ShadowRoot(_))
            {
                continue;
            }
            if let Some(found) = self.find_first_matching_shadow(child, selector) {
                return Some(found);
            }
        }
        None
    }

    /// 在 shadow DOM 内收集所有匹配的元素，不穿透嵌套的 ShadowRoot 边界。
    fn collect_matching_shadow(
        &self,
        id: NodeId,
        selector: &crate::query::SimpleSelector,
        result: &mut Vec<NodeId>,
    ) {
        let node_data = match self.nodes.get(id) {
            Some(n) => n,
            None => return,
        };

        if let NodeKind::Element(elem) = &node_data.kind
            && selector.matches(elem)
        {
            result.push(id);
        }

        for &child in &node_data.children.clone() {
            // 不进入嵌套的 ShadowRoot
            if let Some(child_data) = self.nodes.get(child)
                && matches!(child_data.kind, NodeKind::ShadowRoot(_))
            {
                continue;
            }
            self.collect_matching_shadow(child, selector, result);
        }
    }

    /// 递归收集所有后代节点（按文档顺序，不含 root 自身）。
    fn collect_descendants_recursive(&self, node: NodeId, result: &mut Vec<NodeId>) {
        let children = match self.nodes.get(node).map(|n| n.children.clone()) {
            Some(c) => c,
            None => return,
        };
        for child in children {
            result.push(child);
            self.collect_descendants_recursive(child, result);
        }
    }

    /// 比较两个不互为祖先的节点在文档树中的相对位置。
    ///
    /// 返回 PRECEDING 或 FOLLOWING 位掩码。
    fn compare_tree_position(&self, node1: NodeId, node2: NodeId) -> u8 {
        // 获取从节点到根的祖先路径
        let path1 = self.ancestor_path_with_self(node1);
        let path2 = self.ancestor_path_with_self(node2);

        // 找最近公共祖先
        let mut i = 0;
        while i < path1.len() && i < path2.len() && path1[i] == path2[i] {
            i += 1;
        }

        // i 现在指向分歧点；i-1 是最近公共祖先
        // path1[i] 和 path2[i] 是 LCA 的不同子节点
        if i < path1.len() && i < path2.len() {
            // 在 LCA 的子列表中比较顺序
            let lca = path1[i - 1];
            if let Some(lca_data) = self.nodes.get(lca) {
                let idx1 = lca_data.children.iter().position(|&c| c == path1[i]);
                let idx2 = lca_data.children.iter().position(|&c| c == path2[i]);
                if let (Some(p1), Some(p2)) = (idx1, idx2) {
                    if p1 < p2 {
                        // node1 在 node2 之前 → node2 在 node1 之后
                        return DocumentPosition::FOLLOWING;
                    } else {
                        return DocumentPosition::PRECEDING;
                    }
                }
            }
        }

        // 回退：无法确定位置，标记为 disconnected
        DocumentPosition::DISCONNECTED
    }

    /// 获取从文档根到节点自身的路径（含节点自身）。
    fn ancestor_path_with_self(&self, node: NodeId) -> Vec<NodeId> {
        let mut path = vec![node];
        let mut current = node;
        while let Some(parent) = self.nodes.get(current).and_then(|n| n.parent) {
            path.push(parent);
            current = parent;
        }
        path.reverse();
        path
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

// ── DomError ────────────────────────────────────────────────────────

/// DOM 操作错误类型。
#[derive(Debug, Clone, thiserror::Error)]
pub enum DomError {
    /// 指定的节点不存在。
    #[error("节点不存在: {0:?}")]
    NodeNotFound(NodeId),
    /// 指定的节点不是目标父节点的子节点。
    #[error("节点 {child:?} 不是 {parent:?} 的子节点")]
    NotAChild {
        /// 父节点 ID。
        parent: NodeId,
        /// 子节点 ID。
        child: NodeId,
    },
    /// 操作会导致循环（将祖先作为子孙插入）。
    #[error("操作会导致 DOM 树中出现循环")]
    WouldCreateCycle,
    /// 试图将文档根节点作为子节点插入。
    #[error("不能将文档根节点作为子节点插入")]
    CannotInsertDocumentRoot,
    /// 试图对非元素节点附加 ShadowRoot。
    #[error("只能对元素节点附加 ShadowRoot")]
    NotAnElement,
    /// 试图对已有 ShadowRoot 的元素重复附加。
    #[error("该元素已有 ShadowRoot")]
    AlreadyHasShadowRoot,
}
