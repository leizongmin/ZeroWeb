//! Element tree — retained 实例状态（spec FR-004 / §8.4.2 / DC-2）。
//!
//! Element tree 是「retained」层：保存组件实例状态（焦点、光标、选区、生命周期、绑定缓存）。
//! 关键不变量：WidgetSpec 重建时，稳定 `WidgetId` 的组件在 Element tree 中**保留状态**
//! （spec FR-004 验收场景）。

use crate::geometry::{Rect, Size};
use crate::invalidation::InvalidationFlags;
use crate::widget::{WidgetId, WidgetSpec};
use serde::{Deserialize, Serialize};

/// 单个组件实例的 retained 状态（spec §8.4.2 `ElementState`）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementState {
    pub id: WidgetId,
    pub focusable: bool,
    pub invalidation: InvalidationFlags,
}

/// Element tree 节点。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub state: ElementState,
    pub children: Vec<Element>,
    /// 上次 layout 缓存（失效标记 NEEDS_LAYOUT 时清空重算）。
    pub cached_size: Option<Size>,
    pub cached_rect: Option<Rect>,
    /// 创建/复用 epoch：用于测试断言「同 WidgetId 被复用而非重建」。
    pub epoch: u32,
}

impl Element {
    /// 由 WidgetSpec 新建 element（无既有状态可复用时）。
    pub fn from_spec(spec: &WidgetSpec, epoch: u32) -> Element {
        let id = spec.id.clone().unwrap_or_else(|| WidgetId::new("__anonymous__"));
        Element {
            state: ElementState {
                id,
                focusable: false,
                invalidation: InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT,
            },
            children: spec.children.iter().map(|c| Element::from_spec(c, epoch)).collect(),
            cached_size: None,
            cached_rect: None,
            epoch,
        }
    }

    /// 找到具有指定 WidgetId 的子（含自身）element。
    pub fn find(&self, id: &WidgetId) -> Option<&Element> {
        if self.state.id == *id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(id))
    }

    pub fn find_mut(&mut self, id: &WidgetId) -> Option<&mut Element> {
        if self.state.id == *id {
            return Some(self);
        }
        self.children.iter_mut().find_map(|c| c.find_mut(id))
    }
}

/// 把既有 element tree 与新 WidgetSpec tree 对齐，按 `WidgetId` 复用既有 element（保留状态）。
///
/// M1 采用「同位置 + 同 WidgetId」匹配（Flutter-style canUpdate）；完整 keyed reconciliation 在 runtime。
pub fn reconcile(root: &mut Element, new_spec: &WidgetSpec, epoch: u32) {
    // 根节点身份不变（调用方保证同根）；只刷新 children。
    reconcile_children(&mut root.children, &new_spec.children, epoch);
    root.state.invalidation |= InvalidationFlags::NEEDS_PAINT;
}

fn reconcile_children(existing: &mut Vec<Element>, new_specs: &[WidgetSpec], epoch: u32) {
    let mut next: Vec<Element> = Vec::with_capacity(new_specs.len());
    for (i, spec) in new_specs.iter().enumerate() {
        let reused = existing.get(i).filter(|e| same_identity(e, spec));
        match reused {
            Some(old) => {
                // 复用：保留 retained 状态（epoch 不变），递归对齐其 children。
                let mut el = old.clone();
                reconcile_children(&mut el.children, &spec.children, epoch);
                next.push(el);
            }
            None => next.push(Element::from_spec(spec, epoch)),
        }
    }
    *existing = next;
}

fn same_identity(element: &Element, spec: &WidgetSpec) -> bool {
    match &spec.id {
        Some(id) => &element.state.id == id,
        None => false, // 无 id 的 spec 不参与跨重建复用
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn column_with_text_input() -> WidgetSpec {
        let mut root = WidgetSpec::new("Column");
        root.id = Some(WidgetId::new("root"));
        let mut ti = WidgetSpec::new("TextInput");
        ti.id = Some(WidgetId::new("address"));
        root.children.push(ti);
        root
    }

    #[test]
    fn stable_widget_id_preserves_element_state_across_rebuild() {
        // 首次建树：TextInput "address" 在 epoch=1 创建，并模拟获得焦点（retained 状态）。
        let spec1 = column_with_text_input();
        let mut tree = Element::from_spec(&spec1, 1);
        tree.find_mut(&WidgetId::new("address")).unwrap().state.focusable = true;
        let first_epoch = tree.find(&WidgetId::new("address")).unwrap().epoch;
        assert_eq!(first_epoch, 1);

        // 父组件因状态变化重建 WidgetSpec：TextInput 的 id "address" 不变。
        // （例如新增一个兄弟 Button，但 address 仍在首位且同 id。）
        let mut spec2 = WidgetSpec::new("Column");
        spec2.id = Some(WidgetId::new("root"));
        let mut ti = WidgetSpec::new("TextInput");
        ti.id = Some(WidgetId::new("address"));
        spec2.children.push(ti);

        reconcile(&mut tree, &spec2, 2);

        let addr = tree.find(&WidgetId::new("address")).unwrap();
        // 复用 → epoch 保留为 1（未被重建为 2），焦点状态保留。
        assert_eq!(addr.epoch, 1, "stable WidgetId element must be reused, not recreated");
        assert!(addr.state.focusable, "retained focus state must survive rebuild");
    }

    #[test]
    fn changed_identity_creates_new_element() {
        let spec1 = column_with_text_input();
        let mut tree = Element::from_spec(&spec1, 1);

        // 重建时把首子换成不同 WidgetId 的 Button。
        let mut spec2 = WidgetSpec::new("Column");
        spec2.id = Some(WidgetId::new("root"));
        let mut btn = WidgetSpec::new("Button");
        btn.id = Some(WidgetId::new("go"));
        spec2.children.push(btn);

        reconcile(&mut tree, &spec2, 2);
        let child = &tree.children[0];
        assert_eq!(child.state.id, WidgetId::new("go"));
        assert_eq!(
            child.epoch, 2,
            "new identity must create a fresh element at current epoch"
        );
    }
}
