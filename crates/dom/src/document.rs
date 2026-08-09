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
            content_is_xml: false,
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

    /// 从 HTML 解析器（`DomBuilder`）直接搬移节点表构造——NodeId 与树结构
    /// （parent/children）保持有效，零克隆（旧实现逐节点 clone 重建双树，是
    /// `parse_html` 30-40% 的开销）。仅 parser 内部使用。
    pub(crate) fn from_builder_parts(nodes: SlotMap<NodeId, NodeData>, root: NodeId) -> Self {
        let mut doc = Self {
            nodes,
            root,
            id_map: HashMap::new(),
            observers: Vec::new(),
            pending_mutations: Vec::new(),
            event_listeners: HashMap::new(),
            shadow_roots: HashMap::new(),
            slot_assignments: HashMap::new(),
        };
        // 重建 id 索引（builder 的 TreeSink 不维护 id_map；成本与旧实现
        // create_element_with_qname 注册相同——O(E) 遍历 + 哈希插入）
        let mut id_map: HashMap<String, NodeId> = HashMap::with_capacity(doc.nodes.len() / 4);
        for (id, node) in doc.nodes.iter() {
            if let NodeKind::Element(elem) = &node.kind
                && let Some(id_attr) = &elem.id
            {
                id_map.insert(id_attr.clone(), id);
            }
        }
        doc.id_map = id_map;
        doc
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
        let node_id = self.nodes.insert(NodeData::new(NodeKind::Element(elem_data)));

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
        self.nodes.insert(NodeData::new(NodeKind::Text(TextData::new(text))))
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
    pub fn create_document_type(&mut self, name: &str, public_id: Option<String>, system_id: Option<String>) -> NodeId {
        self.nodes
            .insert(NodeData::new(NodeKind::DocumentType(DocumentTypeData {
                name: name.to_string(),
                public_id,
                system_id,
            })))
    }

    /// 创建一个处理指令节点。
    pub fn create_processing_instruction(&mut self, target: &str, data: &str) -> NodeId {
        self.nodes.insert(NodeData::new(NodeKind::ProcessingInstruction(
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

        // 注册 id 映射（将 child 及其后代的 id 注册到 id_map）
        self.register_id_map_recursive(child);

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

        // 从 id_map 中移除被删除节点（及其后代）的 id 映射
        self.remove_id_map_recursive(child);

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
    pub fn insert_before(&mut self, parent: NodeId, new_node: NodeId, ref_node: NodeId) -> Result<(), DomError> {
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

        // 注册 id 映射
        self.register_id_map_recursive(new_node);

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
    pub fn replace_child(&mut self, parent: NodeId, new_child: NodeId, old_child: NodeId) -> Result<NodeId, DomError> {
        if !self.contains(parent) || !self.contains(new_child) || !self.contains(old_child) {
            return Err(DomError::NodeNotFound(parent));
        }
        if new_child == self.root {
            return Err(DomError::CannotInsertDocumentRoot);
        }
        if self.is_ancestor(new_child, parent) {
            return Err(DomError::WouldCreateCycle);
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

        // 更新 id_map：移除旧节点的 id 映射，注册新节点的 id 映射
        self.remove_id_map_recursive(old_child);
        self.register_id_map_recursive(new_child);

        Ok(old_child)
    }

    /// 克隆节点。如果 `deep` 为 true，递归克隆所有子孙节点。
    pub fn clone_node(&mut self, node: NodeId, deep: bool) -> NodeId {
        let cloned_kind = match self.nodes.get(node).map(|n| n.kind.clone()) {
            Some(kind) => kind,
            None => return node, // fallback: 如果节点不存在，返回自身
        };

        let new_id = self.nodes.insert(NodeData::new(cloned_kind));

        // 注意：克隆节点的 id 不注册到 id_map。
        // 原因：id 在文档中必须唯一，克隆节点与原始节点共享相同的 id 值，
        // 如果都注册会导致 id_map 条目被覆盖。
        // 调用方应在将克隆节点插入文档后手动设置新的唯一 id。

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

    /// 从外部文档导入节点。如果 `deep` 为 true，递归导入所有子孙节点。
    ///
    /// 与 `clone_node` 类似，但强调语义：导入的节点是独立的副本，
    /// 没有父节点（未附着到任何树），可安全地插入到当前文档中。
    pub fn import_node(&mut self, node_id: NodeId, deep: bool) -> NodeId {
        let cloned_kind = match self.nodes.get(node_id).map(|n| n.kind.clone()) {
            Some(kind) => kind,
            None => return node_id,
        };

        let new_id = self.nodes.insert(NodeData::new(cloned_kind));

        if deep && let Some(children) = self.nodes.get(node_id).map(|n| n.children.clone()) {
            for child in children {
                let cloned_child = self.import_node(child, true);
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
        if idx > 0 { Some(siblings[idx - 1]) } else { None }
    }

    /// 获取所有子节点 ID 列表（按文档顺序）。
    pub fn child_nodes(&self, id: NodeId) -> Vec<NodeId> {
        self.nodes.get(id).map(|n| n.children.clone()).unwrap_or_default()
    }

    /// 检查节点是否有子节点。
    pub fn has_child_nodes(&self, id: NodeId) -> bool {
        self.nodes.get(id).map(|n| n.has_children()).unwrap_or(false)
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
    pub fn compare_document_position(&self, node1: NodeId, node2: NodeId) -> Option<DocumentPosition> {
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
        self.nodes.get(node).map(|n| n.children.len()).unwrap_or(0)
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
                    if let Some(NodeKind::Text(data)) = self.nodes.get_mut(id).map(|n| &mut n.kind) {
                        data.content = text.to_string();
                    }
                }
                NodeKind::Comment(_) => {
                    if let Some(NodeKind::Comment(data)) = self.nodes.get_mut(id).map(|n| &mut n.kind) {
                        data.content = text.to_string();
                    }
                }
                NodeKind::Element(_) | NodeKind::DocumentFragment | NodeKind::ShadowRoot(_) => {
                    // 清除所有子节点
                    let children: Vec<NodeId> = self.nodes.get(id).map(|n| n.children.clone()).unwrap_or_default();

                    for child in &children {
                        // 先清理被移除子节点及其后代的 id_map
                        self.remove_id_map_recursive(*child);
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
                // 空字符串不注册到 id_map
                if !value.is_empty() {
                    self.id_map.insert(value.to_string(), id);
                }
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

    /// 单次 DFS 收集多个 tag 的元素（合并多次 `get_elements_by_tag_name` 的全树
    /// 遍历——pipeline 每帧对 meta/style/img 等多次遍历，合并后遍历次数减半）。
    pub fn get_elements_by_tag_names(&self, tags: &[&str]) -> Vec<NodeId> {
        let mut result = Vec::new();
        self.collect_by_tag_names(self.root, tags, &mut result);
        result
    }

    /// 根据命名空间和标签名查找所有匹配的元素节点。
    ///
    /// - `namespace` 为 `Some(ns)` 时匹配指定命名空间，为 `None` 时匹配任意命名空间（通配）。
    /// - `local_name` 为 `"*"` 时匹配所有元素（通配），否则匹配指定标签名。
    pub fn get_elements_by_tag_name_ns(&self, namespace: Option<&str>, local_name: &str) -> Vec<NodeId> {
        let mut result = Vec::new();
        self.collect_by_tag_name_ns(self.root, namespace, local_name, &mut result);
        result
    }

    /// 根据类名查找所有匹配的元素节点。
    pub fn get_elements_by_class_name(&self, class: &str) -> Vec<NodeId> {
        let mut result = Vec::new();
        self.collect_by_class_name(self.root, class, &mut result);
        result
    }

    /// 在指定节点的子树中查找第一个匹配选择器的元素。
    ///
    /// 支持简单选择器及后代（空格）、子（`>`）组合器。
    pub fn query_selector(&self, root: NodeId, selector: &str) -> Option<NodeId> {
        let chain = crate::query::parse_selector_chain(selector.trim())?;
        if chain.parts.len() == 1 {
            return self.find_first_matching(root, &chain.parts[0]);
        }
        self.find_first_matching_chain(root, &chain)
    }

    /// 在指定节点的子树中查找所有匹配选择器的元素。
    pub fn query_selector_all(&self, root: NodeId, selector: &str) -> Vec<NodeId> {
        let chain = match crate::query::parse_selector_chain(selector.trim()) {
            Some(c) => c,
            None => return vec![],
        };
        if chain.parts.len() == 1 {
            let mut result = Vec::new();
            self.collect_matching(root, &chain.parts[0], &mut result);
            return result;
        }
        let mut candidates = Vec::new();
        self.collect_matching(root, &chain.parts[chain.parts.len() - 1], &mut candidates);
        candidates
            .into_iter()
            .filter(|id| self.node_matches_selector_chain(*id, &chain))
            .collect()
    }

    // ── Shadow DOM ──────────────────────────────────────────────

    /// 为宿主元素附加 ShadowRoot。
    ///
    /// 创建一个新的 ShadowRoot 节点并附加到指定的宿主元素上。
    /// 返回错误如果：宿主不是元素节点，或宿主已有 ShadowRoot。
    pub fn attach_shadow(&mut self, host: NodeId, mode: ShadowRootMode) -> Result<NodeId, DomError> {
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
        let shadow_id = self.nodes.insert(NodeData::new(NodeKind::ShadowRoot(shadow_data)));

        // 设置宿主引用
        if let Some(NodeKind::ShadowRoot(data)) = self.nodes.get_mut(shadow_id).map(|n| &mut n.kind) {
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

    /// 解析 Shadow DOM slot 分配。
    ///
    /// 遍历宿主元素（`host_node_id`）的 shadow 树中所有 `<slot>` 元素，
    /// 将宿主 light DOM 子节点中匹配的元素分配到对应 slot：
    ///
    /// - 有 `slot="name"` 属性的子元素分配到 `<slot name="name">`
    /// - 没有 `slot` 属性的子元素分配到默认 slot（`<slot>` 无 name 属性）
    /// - 如果某个 slot 没有匹配的 light DOM 子节点，使用 slot 自身的子节点作为回退内容
    ///
    /// 该方法会清除并重新计算该宿主的全部 slot 分配。
    pub fn resolve_slots(&mut self, host_node_id: NodeId) {
        // 获取 shadow root
        let shadow_id = match self.shadow_roots.get(&host_node_id).copied() {
            Some(id) => id,
            None => return,
        };

        // 收集 shadow 树中所有 <slot> 元素：(NodeId, Option<name>)
        let slot_elements = self.collect_slot_elements(shadow_id);
        if slot_elements.is_empty() {
            return;
        }

        // 清除这些 slot 元素的旧分配
        for &(slot_id, ref slot_name_opt) in &slot_elements {
            let name = slot_name_opt.as_deref().unwrap_or("");
            self.slot_assignments.remove(&(slot_id, name.to_string()));
        }

        // 收集宿主的 light DOM 子节点（排除 ShadowRoot 自身）
        let host_children = self
            .nodes
            .get(host_node_id)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        let light_children: Vec<NodeId> = host_children
            .iter()
            .copied()
            .filter(|&id| !matches!(self.nodes.get(id).map(|n| &n.kind), Some(NodeKind::ShadowRoot(_))))
            .collect();

        // 对每个 light DOM 子节点，确定它分配到哪个 slot
        for &child_id in &light_children {
            let slot_attr = self.nodes.get(child_id).and_then(|n| match &n.kind {
                NodeKind::Element(elem) => elem.get_attribute("slot"),
                _ => None,
            });

            let matched_slot = if let Some(ref slot_name) = slot_attr {
                // 有 slot 属性：找匹配 name 的 slot
                slot_elements
                    .iter()
                    .find(|(_, name_opt)| name_opt.as_deref() == Some(slot_name.as_str()))
            } else {
                // 没有 slot 属性：分配到默认 slot（无 name 属性的 slot）
                slot_elements.iter().find(|(_, name_opt)| name_opt.is_none())
            };

            if let Some(&(slot_id, ref name_opt)) = matched_slot {
                let name = name_opt.as_deref().unwrap_or("").to_string();
                self.slot_assignments.entry((slot_id, name)).or_default().push(child_id);
            }
        }
    }

    /// 获取 slot 元素已分配的节点列表。
    ///
    /// 在调用 `resolve_slots` 之后使用此方法获取分配结果。
    /// 如果 slot 有已分配的 light DOM 节点，返回这些节点；
    /// 如果没有已分配的节点（即使用回退内容），返回空列表。
    ///
    /// `slot_node_id` 是 `<slot>` 元素的 NodeId。
    pub fn get_assigned_nodes(&self, slot_node_id: NodeId) -> Vec<NodeId> {
        // 获取 slot 的 name 属性
        let slot_name = self
            .nodes
            .get(slot_node_id)
            .and_then(|n| match &n.kind {
                NodeKind::Element(elem) => elem.get_attribute("name"),
                _ => None,
            })
            .unwrap_or_default();

        self.slot_assignments
            .get(&(slot_node_id, slot_name))
            .cloned()
            .unwrap_or_default()
    }

    /// 在 ShadowRoot 子树中查找第一个匹配选择器的元素。
    ///
    /// 类似 `query_selector`，但范围限定在 shadow DOM 树内，
    /// 不会穿透到嵌套的 ShadowRoot 边界。
    pub fn query_selector_shadow(&self, shadow_root: NodeId, selector: &str) -> Option<NodeId> {
        let parsed = crate::query::parse_simple_selector(selector)?;
        self.find_first_matching_shadow(shadow_root, &parsed)
    }

    /// 在 ShadowRoot 子树中查找所有匹配选择器的元素。
    ///
    /// 类似 `query_selector_all`，但范围限定在 shadow DOM 树内，
    /// 不会穿透到嵌套的 ShadowRoot 边界。
    pub fn query_selector_all_shadow(&self, shadow_root: NodeId, selector: &str) -> Vec<NodeId> {
        let parsed = match crate::query::parse_simple_selector(selector) {
            Some(s) => s,
            None => return vec![],
        };
        let mut result = Vec::new();
        self.collect_matching_shadow(shadow_root, &parsed, &mut result);
        result
    }

    // ── 规范化 ──────────────────────────────────────────────────

    /// 规范化指定节点的子树。
    ///
    /// 遵循 WHATWG DOM 规范中 `Node.normalize()` 的语义：
    /// - 合并相邻的 Text 节点（将后续文本内容追加到第一个文本节点，然后移除后续节点）
    /// - 移除空的 Text 节点（内容为 `""` 的文本节点）
    /// - 递归处理 Element 类型的子节点
    pub fn normalize(&mut self, node_id: NodeId) {
        if !self.contains(node_id) {
            return;
        }

        let children = self.child_nodes(node_id);

        // 第一遍：递归处理 Element 子节点（先处理深层，再处理浅层）
        for &child in &children {
            let is_element = self
                .nodes
                .get(child)
                .map(|n| matches!(n.kind, NodeKind::Element(_)))
                .unwrap_or(false);
            if is_element {
                self.normalize(child);
            }
        }

        // 第二遍：收集需要移除和合并的文本节点信息
        // 遍历子节点列表，合并相邻文本节点，移除空文本节点
        let mut to_remove: Vec<NodeId> = Vec::new();
        let mut i = 0;
        let len = children.len();

        while i < len {
            let child = children[i];
            let is_text = self
                .nodes
                .get(child)
                .map(|n| matches!(n.kind, NodeKind::Text(_)))
                .unwrap_or(false);

            if is_text {
                let is_empty = self
                    .nodes
                    .get(child)
                    .and_then(|n| match &n.kind {
                        NodeKind::Text(d) => Some(d.content.is_empty()),
                        _ => None,
                    })
                    .unwrap_or(false);

                if is_empty {
                    // 空文本节点，标记移除
                    to_remove.push(child);
                } else {
                    // 收集后续相邻的文本节点，合并到当前节点
                    let mut j = i + 1;
                    while j < len {
                        let next = children[j];
                        let next_is_text = self
                            .nodes
                            .get(next)
                            .map(|n| matches!(n.kind, NodeKind::Text(_)))
                            .unwrap_or(false);
                        if !next_is_text {
                            break;
                        }
                        // 追加文本内容到当前节点
                        let next_content = self
                            .nodes
                            .get(next)
                            .and_then(|n| match &n.kind {
                                NodeKind::Text(d) => Some(d.content.clone()),
                                _ => None,
                            })
                            .unwrap_or_default();
                        if let Some(NodeKind::Text(data)) = self.nodes.get_mut(child).map(|n| &mut n.kind) {
                            data.content.push_str(&next_content);
                        }
                        to_remove.push(next);
                        j += 1;
                    }
                    i = j;
                    continue;
                }
            }
            i += 1;
        }

        // 第三遍：从父节点子列表中移除标记的节点，清除父引用
        for &id in &to_remove {
            if let Some(parent_data) = self.nodes.get_mut(node_id) {
                parent_data.children.retain(|&c| c != id);
            }
            if let Some(child_data) = self.nodes.get_mut(id) {
                child_data.parent = None;
            }
        }
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

    // ── content is XML / XHTML ──────────────────────────────────

    /// 文档内容是否按 XML/XHTML 语义处理（影响选择器大小写敏感性等）。
    ///
    /// ZW 用 html5ever 统一按 HTML 解析，但对 WPT `.xht`/`.xhtml` 文档须按 XML 语义
    /// 处理选择器大小写（CSS Selectors §6.3：HTML 不敏感、XML 敏感）。
    pub fn content_is_xml(&self) -> bool {
        match &self.nodes.get(self.root).map(|n| &n.kind) {
            Some(NodeKind::Document(data)) => data.content_is_xml,
            _ => false,
        }
    }

    /// 设置文档的 XML/XHTML 内容语义标志。
    pub fn set_content_is_xml(&mut self, is_xml: bool) {
        if let Some(NodeKind::Document(data)) = self.nodes.get_mut(self.root).map(|n| &mut n.kind) {
            data.content_is_xml = is_xml;
        }
    }

    /// 检测文档是否为 XHTML 并设置 `content_is_xml` 标志。
    ///
    /// 判据：根的子节点中存在 `DocumentType`，且其 `public_id` 含 "XHTML"（大小写不敏感，
    /// 匹配 `-//W3C//DTD XHTML 1.0//EN` 等所有 XHTML DOCTYPE）。这是 WPT `.xht`/`.xhtml`
    /// 文件的标准标记（10618/10633 含此信号）。
    ///
    /// 在 `DomBuilder::into_document` 收尾处调用；对纯 HTML 文档（无 DOCTYPE 或 HTML5 DOCTYPE）
    /// 保持 `false`，行为不变。
    pub fn detect_and_set_content_is_xml(&mut self) {
        let is_xml = self
            .child_nodes(self.root)
            .iter()
            .any(|child| match self.get(*child).map(|n| &n.kind) {
                Some(NodeKind::DocumentType(dt)) => dt
                    .public_id
                    .as_deref()
                    .is_some_and(|pid| pid.to_ascii_lowercase().contains("xhtml")),
                _ => false,
            });
        self.set_content_is_xml(is_xml);
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

    /// 移除所有已注册的 MutationObserver（相当于 disconnect）。
    pub fn clear_observers(&mut self) {
        self.observers.clear();
    }

    // ── 事件系统 ─────────────────────────────────────────────────

    /// 为指定节点添加事件监听器。
    ///
    /// `event_type` 是事件类型名（如 "click"、"input"）。
    /// `callback` 是事件触发时的回调函数。
    /// `capture` 为 true 时监听器在捕获阶段触发，否则在冒泡阶段触发。
    pub fn add_event_listener(&mut self, node: NodeId, event_type: &str, callback: EventListenerFn, capture: bool) {
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

    /// 递归移除节点及其后代在 id_map 中的条目。
    ///
    /// 当节点从文档树中移除时调用，确保 `get_element_by_id` 不再返回已移除的节点。
    fn remove_id_map_recursive(&mut self, id: NodeId) {
        // 收集当前节点的 id（如果有）
        let node_id_value = self.nodes.get(id).and_then(|n| match &n.kind {
            NodeKind::Element(elem) => elem.id.clone(),
            _ => None,
        });
        if let Some(ref id_val) = node_id_value {
            self.id_map.remove(id_val);
        }
        // 递归处理子节点
        let children = self.nodes.get(id).map(|n| n.children.clone()).unwrap_or_default();
        for child in children {
            self.remove_id_map_recursive(child);
        }
    }

    /// 递归注册节点及其后代的 id 到 id_map。
    ///
    /// 当节点插入文档树时调用，使 `get_element_by_id` 能找到新插入的节点。
    fn register_id_map_recursive(&mut self, node_id: NodeId) {
        // 收集当前节点的 id（如果有）
        let id_value = self.nodes.get(node_id).and_then(|n| match &n.kind {
            NodeKind::Element(elem) => elem.id.clone(),
            _ => None,
        });
        if let Some(ref id_val) = id_value {
            self.id_map.insert(id_val.clone(), node_id);
        }
        // 递归处理子节点
        let children = self.nodes.get(node_id).map(|n| n.children.clone()).unwrap_or_default();
        for child in children {
            self.register_id_map_recursive(child);
        }
    }

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
    fn collect_by_tag_names(&self, id: NodeId, tags: &[&str], result: &mut Vec<NodeId>) {
        let node_data = match self.nodes.get(id) {
            Some(n) => n,
            None => return,
        };
        if let NodeKind::Element(elem) = &node_data.kind
            && tags.iter().any(|t| elem.local_name().eq_ignore_ascii_case(t))
        {
            result.push(id);
        }
        for &child in &node_data.children {
            self.collect_by_tag_names(child, tags, result);
        }
    }

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

        for &child in &node_data.children {
            self.collect_by_tag_name(child, tag, result);
        }
    }

    /// 递归收集指定命名空间和标签名的元素。
    fn collect_by_tag_name_ns(&self, id: NodeId, namespace: Option<&str>, local_name: &str, result: &mut Vec<NodeId>) {
        let node_data = match self.nodes.get(id) {
            Some(n) => n,
            None => return,
        };

        if let NodeKind::Element(elem) = &node_data.kind {
            let ns_match = match namespace {
                Some(ns) => elem.namespace() == ns,
                None => true,
            };
            let name_match = local_name == "*" || elem.local_name().eq_ignore_ascii_case(local_name);
            if ns_match && name_match {
                result.push(id);
            }
        }

        let children: Vec<NodeId> = node_data.children.to_vec();
        for child in children {
            self.collect_by_tag_name_ns(child, namespace, local_name, result);
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

        let children: Vec<NodeId> = node_data.children.to_vec();
        for child in children {
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
    fn find_first_matching(&self, id: NodeId, selector: &crate::query::SimpleSelector) -> Option<NodeId> {
        let node_data = self.nodes.get(id)?;
        if let NodeKind::Element(_) = &node_data.kind
            && self.element_matches_selector(id, selector)
        {
            return Some(id);
        }
        let children: Vec<NodeId> = node_data.children.to_vec();
        for child in children {
            if let Some(found) = self.find_first_matching(child, selector) {
                return Some(found);
            }
        }
        None
    }

    fn find_first_matching_chain(&self, id: NodeId, chain: &crate::query::SelectorChain) -> Option<NodeId> {
        let node_data = self.nodes.get(id)?;
        if let NodeKind::Element(_) = &node_data.kind
            && chain.parts.last().is_some_and(|s| self.element_matches_selector(id, s))
            && self.node_matches_selector_chain(id, chain)
        {
            return Some(id);
        }
        let children: Vec<NodeId> = node_data.children.to_vec();
        for child in children {
            if let Some(found) = self.find_first_matching_chain(child, chain) {
                return Some(found);
            }
        }
        None
    }

    fn node_matches_selector_chain(&self, node: NodeId, chain: &crate::query::SelectorChain) -> bool {
        let parts = &chain.parts;
        if parts.is_empty() {
            return false;
        }
        let mut current = node;
        let mut idx = parts.len() - 1;
        if !self.element_matches_selector(current, &parts[idx]) {
            return false;
        }
        while idx > 0 {
            let comb = chain.combinators[idx - 1];
            idx -= 1;
            current = match comb {
                crate::query::Combinator::Child => match self.parent_element_node(current) {
                    Some(p) => p,
                    None => return false,
                },
                crate::query::Combinator::Descendant => {
                    match self.find_ancestor_matching_selector(current, &parts[idx]) {
                        Some(p) => p,
                        None => return false,
                    }
                }
            };
            if !self.element_matches_selector(current, &parts[idx]) {
                return false;
            }
        }
        true
    }

    fn element_matches_selector(&self, node: NodeId, selector: &crate::query::SimpleSelector) -> bool {
        let matched = self
            .nodes
            .get(node)
            .and_then(|n| match &n.kind {
                NodeKind::Element(elem) => Some(selector.matches_full(elem, self.compute_element_position(node))),
                _ => None,
            })
            .unwrap_or(false);
        if !matched {
            return false;
        }
        // :has() 需 Document 子树求值（matches_full 延后返 true），此处额外评估。
        // 其他伪类已由 matches_full 评估，故非 Has 一律 true。
        selector.pseudos.iter().all(|p| match p {
            crate::query::PseudoClass::Has { inner, child_scope } => {
                self.element_has_matching(node, inner, *child_scope)
            }
            _ => true,
        })
    }

    /// `:has(inner)` 求值——node 是否拥有匹配 inner 的后代（默认）或直接子（`child_scope`）。
    fn element_has_matching(&self, node: NodeId, inner: &str, child_scope: bool) -> bool {
        if child_scope {
            // :has(> inner)——直接元素子 c **自身**匹配 inner（单简单选择器，常见 `:has(> .foo)`）。
            // 不搜 c 的后代（那会是 `:has(> * inner)` 语义，致假阳性）。inner 含组合器（如
            // `:has(> .a .b)`）的相对求值为 follow-up（parse_simple_selector 对含空格者返非预期 → 不匹配）。
            let children = self.nodes.get(node).map(|n| n.children.to_vec()).unwrap_or_default();
            let sel = crate::query::parse_simple_selector(inner);
            for c in children {
                let is_elem = self
                    .nodes
                    .get(c)
                    .is_some_and(|n| matches!(n.kind, NodeKind::Element(_)));
                if is_elem
                    && let Some(sel) = &sel
                    && self.element_matches_selector(c, sel)
                {
                    return true;
                }
            }
            false
        } else {
            // :has(inner)——后代匹配（query_selector_all 在 node 子树求值，含组合器链）。
            !self.query_selector_all(node, inner).is_empty()
        }
    }

    /// 计算元素的 sibling 位置上下文（伪类评估用）。
    ///
    /// - `child_index`/`child_count`：在元素父节点的**元素**子中 1-based 序号 / 总数。
    /// - `type_index`/`type_count`：在同 tag 元素子中的序号 / 总数。
    /// - `is_root`：无元素父（`<html>`）。
    /// - `is_empty`：无任何子节点（`:empty`）。
    fn compute_element_position(&self, node: NodeId) -> crate::query::ElementPosition {
        let mut pos = crate::query::ElementPosition::default();
        let node_data = match self.nodes.get(node) {
            Some(n) => n,
            None => return pos,
        };
        pos.is_empty = node_data.children.is_empty();
        let tag = match &node_data.kind {
            NodeKind::Element(e) => e.local_name(),
            _ => return pos,
        };
        let Some(parent) = self.parent_element_node(node) else {
            // 根元素（html）：唯一兄弟、唯一同 tag、是根。
            pos.is_root = true;
            pos.child_index = 1;
            pos.child_count = 1;
            pos.type_index = 1;
            pos.type_count = 1;
            return pos;
        };
        let siblings: Vec<NodeId> = self.nodes.get(parent).map(|p| p.children.to_vec()).unwrap_or_default();
        let mut child_idx = 0usize;
        let mut type_idx = 0usize;
        let mut child_total = 0usize;
        let mut type_total = 0usize;
        // 先全量统计同 tag 总数 + 定位自身序号。
        for sib in &siblings {
            if let Some(NodeKind::Element(e)) = self.nodes.get(*sib).map(|n| &n.kind) {
                child_total += 1;
                if e.local_name().eq_ignore_ascii_case(tag) {
                    type_total += 1;
                }
            }
        }
        for sib in &siblings {
            if let Some(NodeKind::Element(e)) = self.nodes.get(*sib).map(|n| &n.kind) {
                child_idx += 1;
                if e.local_name().eq_ignore_ascii_case(tag) {
                    type_idx += 1;
                }
                if *sib == node {
                    break;
                }
            }
        }
        pos.child_index = child_idx;
        pos.child_count = child_total;
        pos.type_index = type_idx;
        pos.type_count = type_total;
        pos
    }

    fn parent_element_node(&self, node: NodeId) -> Option<NodeId> {
        let mut current = self.parent_node(node);
        while let Some(pid) = current {
            if self
                .nodes
                .get(pid)
                .is_some_and(|n| matches!(n.kind, NodeKind::Element(_)))
            {
                return Some(pid);
            }
            current = self.parent_node(pid);
        }
        None
    }

    fn find_ancestor_matching_selector(&self, node: NodeId, selector: &crate::query::SimpleSelector) -> Option<NodeId> {
        let mut current = self.parent_node(node);
        while let Some(pid) = current {
            if self.element_matches_selector(pid, selector) {
                return Some(pid);
            }
            current = self.parent_node(pid);
        }
        None
    }

    /// 收集所有匹配的节点。
    fn collect_matching(&self, id: NodeId, selector: &crate::query::SimpleSelector, result: &mut Vec<NodeId>) {
        let node_data = match self.nodes.get(id) {
            Some(n) => n,
            None => return,
        };

        if let NodeKind::Element(_) = &node_data.kind
            && self.element_matches_selector(id, selector)
        {
            result.push(id);
        }

        let children: Vec<NodeId> = node_data.children.to_vec();
        for child in children {
            self.collect_matching(child, selector, result);
        }
    }

    /// 在 shadow DOM 内查找第一个匹配的元素，不穿透嵌套的 ShadowRoot 边界。
    fn find_first_matching_shadow(&self, id: NodeId, selector: &crate::query::SimpleSelector) -> Option<NodeId> {
        let node_data = self.nodes.get(id)?;
        if let NodeKind::Element(_) = &node_data.kind
            && self.element_matches_selector(id, selector)
        {
            return Some(id);
        }
        let children: Vec<NodeId> = node_data.children.to_vec();
        for child in children {
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
    fn collect_matching_shadow(&self, id: NodeId, selector: &crate::query::SimpleSelector, result: &mut Vec<NodeId>) {
        let node_data = match self.nodes.get(id) {
            Some(n) => n,
            None => return,
        };

        if let NodeKind::Element(_) = &node_data.kind
            && self.element_matches_selector(id, selector)
        {
            result.push(id);
        }

        let children: Vec<NodeId> = node_data.children.to_vec();
        for child in children {
            // 不进入嵌套的 ShadowRoot
            if let Some(child_data) = self.nodes.get(child)
                && matches!(child_data.kind, NodeKind::ShadowRoot(_))
            {
                continue;
            }
            self.collect_matching_shadow(child, selector, result);
        }
    }

    /// 收集 shadow 树中所有 `<slot>` 元素。
    ///
    /// 返回 `(NodeId, Option<name属性值>)` 列表。
    /// 不穿透嵌套的 ShadowRoot 边界。
    fn collect_slot_elements(&self, root: NodeId) -> Vec<(NodeId, Option<String>)> {
        let mut result = Vec::new();
        self.collect_slot_elements_recursive(root, &mut result);
        result
    }

    /// 递归收集 `<slot>` 元素。
    fn collect_slot_elements_recursive(&self, node_id: NodeId, result: &mut Vec<(NodeId, Option<String>)>) {
        let node_data = match self.nodes.get(node_id) {
            Some(n) => n,
            None => return,
        };

        // 检查当前节点是否为 <slot> 元素
        if let NodeKind::Element(elem) = &node_data.kind
            && elem.local_name() == "slot"
        {
            let name = elem.get_attribute("name");
            result.push((node_id, name));
        }

        // 递归遍历子节点，不穿透嵌套的 ShadowRoot
        let children = node_data.children.clone();
        for child in children {
            if let Some(child_data) = self.nodes.get(child)
                && matches!(child_data.kind, NodeKind::ShadowRoot(_))
            {
                continue;
            }
            self.collect_slot_elements_recursive(child, result);
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
