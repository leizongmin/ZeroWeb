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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// 创建 MutationRecord — 基础字段验证。
    #[test]
    fn test_mutation_record_child_list() {
        let mut doc = crate::Document::new();
        let target = doc.create_element("div");
        let child = doc.create_element("span");

        let record = MutationRecord {
            mutation_type: MutationType::ChildList,
            target,
            added_nodes: vec![child],
            removed_nodes: vec![],
            previous_sibling: None,
            attribute_name: None,
            old_value: None,
        };

        assert_eq!(record.mutation_type, MutationType::ChildList);
        assert_eq!(record.target, target);
        assert_eq!(record.added_nodes.len(), 1);
        assert!(record.removed_nodes.is_empty());
        assert!(record.attribute_name.is_none());
    }

    /// MutationRecord — 属性变更记录。
    #[test]
    fn test_mutation_record_attributes() {
        let mut doc = crate::Document::new();
        let target = doc.create_element("div");

        let record = MutationRecord {
            mutation_type: MutationType::Attributes,
            target,
            added_nodes: vec![],
            removed_nodes: vec![],
            previous_sibling: None,
            attribute_name: Some("class".to_string()),
            old_value: Some("old-class".to_string()),
        };

        assert_eq!(record.mutation_type, MutationType::Attributes);
        assert_eq!(record.attribute_name.as_deref(), Some("class"));
        assert_eq!(record.old_value.as_deref(), Some("old-class"));
    }

    /// MutationRecord — CharacterData 变更。
    #[test]
    fn test_mutation_record_character_data() {
        let mut doc = crate::Document::new();
        let target = doc.create_text_node("hello");

        let record = MutationRecord {
            mutation_type: MutationType::CharacterData,
            target,
            added_nodes: vec![],
            removed_nodes: vec![],
            previous_sibling: None,
            attribute_name: None,
            old_value: Some("hello".to_string()),
        };

        assert_eq!(record.mutation_type, MutationType::CharacterData);
        assert_eq!(record.old_value.as_deref(), Some("hello"));
    }

    /// MutationObserver 回调被正确调用。
    #[test]
    fn test_observer_notify_calls_callback() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let count_clone = call_count.clone();

        let observer = MutationObserver::new(Box::new(move |records| {
            count_clone.fetch_add(records.len(), Ordering::SeqCst);
        }));

        let mut doc = crate::Document::new();
        let target = doc.create_element("div");
        let child = doc.create_element("span");

        let records = vec![
            MutationRecord {
                mutation_type: MutationType::ChildList,
                target,
                added_nodes: vec![child],
                removed_nodes: vec![],
                previous_sibling: None,
                attribute_name: None,
                old_value: None,
            },
            MutationRecord {
                mutation_type: MutationType::Attributes,
                target,
                added_nodes: vec![],
                removed_nodes: vec![],
                previous_sibling: None,
                attribute_name: Some("id".to_string()),
                old_value: None,
            },
        ];

        observer.notify(&records);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    /// MutationObserver — 空记录列表不触发回调内容。
    #[test]
    fn test_observer_notify_empty_records() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let count_clone = call_count.clone();

        let observer = MutationObserver::new(Box::new(move |records| {
            count_clone.fetch_add(records.len(), Ordering::SeqCst);
        }));

        observer.notify(&[]);
        assert_eq!(call_count.load(Ordering::SeqCst), 0);
    }

    /// MutationType — PartialEq 验证。
    #[test]
    fn test_mutation_type_equality() {
        assert_eq!(MutationType::ChildList, MutationType::ChildList);
        assert_eq!(MutationType::Attributes, MutationType::Attributes);
        assert_eq!(MutationType::CharacterData, MutationType::CharacterData);
        assert_ne!(MutationType::ChildList, MutationType::Attributes);
        assert_ne!(MutationType::Attributes, MutationType::CharacterData);
    }

    /// MutationRecord — clone 一致性验证。
    #[test]
    fn test_mutation_record_clone() {
        let mut doc = crate::Document::new();
        let target = doc.create_element("div");

        let record = MutationRecord {
            mutation_type: MutationType::ChildList,
            target,
            added_nodes: vec![],
            removed_nodes: vec![],
            previous_sibling: None,
            attribute_name: Some("data-x".to_string()),
            old_value: Some("old".to_string()),
        };

        let cloned = record.clone();
        assert_eq!(cloned.mutation_type, record.mutation_type);
        assert_eq!(cloned.target, record.target);
        assert_eq!(cloned.attribute_name, record.attribute_name);
        assert_eq!(cloned.old_value, record.old_value);
    }
}
