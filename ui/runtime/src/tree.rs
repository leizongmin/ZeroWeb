//! UI 树运行态 — Element tree + scene 缓存 + 失效聚合（spec FR-003/FR-004 运行时）。

use zero_ui_core::element::{Element, reconcile};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::widget::WidgetSpec;
use zero_ui_render::Scene;

/// 运行时 UI 树：把 `WidgetSpec`（声明）reconcile 为 retained `Element`，并缓存 scene。
#[derive(Debug, Default)]
pub struct UiTree {
    pub root: Option<Element>,
    pub scene: Scene,
    /// 待消费的失效（layout/paint/...）。
    pub pending: InvalidationFlags,
    epoch: u32,
}

impl UiTree {
    pub fn new() -> UiTree {
        UiTree::default()
    }

    /// 用新声明树 reconcile；结构变化由 element reconcile 内部按 WidgetId 处理。
    /// 首次建树标记 layout+paint；后续 rebuild 至少标记 paint。
    pub fn set_root(&mut self, spec: &WidgetSpec) {
        self.epoch += 1;
        match &mut self.root {
            Some(root) => {
                reconcile(root, spec, self.epoch);
                self.pending |= InvalidationFlags::NEEDS_PAINT;
            }
            None => {
                self.root = Some(Element::from_spec(spec, self.epoch));
                self.pending |= InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
            }
        }
    }

    /// 标记外部失效（如主题/locale 变化）。
    pub fn mark(&mut self, flags: InvalidationFlags) {
        self.pending |= flags;
    }

    /// 消费并清空 pending 失效。
    pub fn take_pending(&mut self) -> InvalidationFlags {
        let f = self.pending;
        self.pending = InvalidationFlags::CLEAN;
        f
    }
}
