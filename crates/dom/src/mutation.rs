//! MutationObserver 基础框架。
//!
//! 提供基础的 DOM 变更观察能力，支持 childList 和 attributes 变更类型。

use crate::node::NodeId;

/// Mutation 变更类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationType {
    /// 子节点变更。
    ChildList,
    /// 属性变更。
    Attributes,
    /// 文本内容变更。
    CharacterData,
}

/// 一条 mutation 记录，对应 WHATWG MutationRecord 接口。
#[derive(Debug, Clone)]
pub struct MutationRecord {
    /// 变更类型。
    pub mutation_type: MutationType,
    /// 变更的目标节点。
    pub target: NodeId,
    /// 新增的子节点列表。
    pub added_nodes: Vec<NodeId>,
    /// 移除的子节点列表。
    pub removed_nodes: Vec<NodeId>,
    /// 前一个兄弟节点。
    pub previous_sibling: Option<NodeId>,
    /// 变更的属性名。
    pub attribute_name: Option<String>,
    /// 变更前的值。
    pub old_value: Option<String>,
}

/// Mutation 回调函数类型。
pub type MutationCallbackFn = Box<dyn Fn(&[MutationRecord])>;

/// MutationObserver — 观察 DOM 变更。
///
/// 基础实现：注册回调函数，当 Document 处理 mutation 记录时调用回调。
/// 完整的 WHATWG MutationObserver API（包含 observe 选项、微任务队列等）将在 JS 绑定层实现。
pub struct MutationObserver {
    /// 回调函数。
    callback: MutationCallbackFn,
}

impl MutationObserver {
    /// 创建新的 MutationObserver，使用指定的回调函数。
    pub fn new(callback: MutationCallbackFn) -> Self {
        Self { callback }
    }

    /// 通知 observer 有新的 mutation 记录。
    pub fn notify(&self, records: &[MutationRecord]) {
        (self.callback)(records);
    }
}
