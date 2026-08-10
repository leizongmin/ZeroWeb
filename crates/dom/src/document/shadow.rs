//! Document Shadow DOM 操作 —— 拆自 `mod.rs`（rule 5 单文件 <2000 行，R3164）。
//!
//! 本模块为 [`super::Document`] 的 Shadow DOM 面（attach_shadow / shadow_root / slot 分配 /
//! shadow 子树查询）。作为 `document` 模块的**子模块**，可访问 [`super::Document`] 的私有字段
//!（`nodes` / `shadow_roots` / `slot_assignments`）与 `mod.rs` 的私有查询助手
//!（`collect_slot_elements` / `find_first_matching_shadow` / `collect_matching_shadow`）——Rust 隐私规则：
//! 私有项对定义模块及其后代可见，故无需任何可见性改动（行为不变重组）。

use crate::node::{NodeData, NodeId, NodeKind, ShadowRootData, ShadowRootMode};

use super::Document;
use super::DomError;

impl Document {
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
}
