//! ListView — 可虚拟化的列表模型（spec FR-009 / §8.4.1B）。
//!
//! 持有 scroll 偏移 + 条目高度信息，与 [`VirtualCollection`] 配合物化可见窗口。
//! 典型用法：`ui/patterns::DataList` 组合 ListView + 具体 VirtualCollection 实现。

use zero_ui_collections::{
    ItemKey, LazyList, MaterializedItem, Selection, VirtualCollection, VisibleWindow, materialize,
};

/// 可虚拟化的列表视图模型。
#[derive(Debug, Clone)]
pub struct ListView {
    /// 总条目数（与关联的 VirtualCollection 一致）。
    pub item_count: usize,
    /// 当前选中索引（none = 未选中）。
    pub selected: Option<usize>,
    /// 当前滚动偏移（px）。
    pub scroll_offset: f32,
    /// 视口高度（px）。
    pub viewport_height: f32,
    /// 估计条目高度（px），用于窗口计算。
    pub item_height_px: f32,
    /// 选择集（按 key）。
    pub selection: Selection,
}

impl ListView {
    pub fn new(item_count: usize) -> ListView {
        ListView {
            item_count,
            selected: None,
            scroll_offset: 0.0,
            viewport_height: 600.0,
            item_height_px: 40.0,
            selection: Selection::new(),
        }
    }

    /// 更新总条目数。
    pub fn set_count(&mut self, n: usize) {
        self.item_count = n;
        self.selected = self.selected.filter(|&i| i < n);
    }

    pub fn select(&mut self, index: usize) -> bool {
        if index < self.item_count {
            self.selected = Some(index);
            self.selection.select(ItemKey::new(&index.to_string()));
            true
        } else {
            false
        }
    }

    /// 根据当前 scroll_offset 计算可见窗口。
    pub fn visible_window(&self) -> VisibleWindow {
        LazyList::new(self.item_count, self.item_height_px, self.viewport_height)
            .window_at(self.scroll_offset)
    }

    /// 物化可见窗口内的条目（调 `VirtualCollection::build_item`）。
    pub fn materialize_items<C: VirtualCollection + ?Sized>(&self, c: &C) -> Vec<MaterializedItem> {
        materialize(c, self.visible_window())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_collections::DynamicCollection;
    use zero_ui_core::widget::WidgetSpec;

    fn test_collection(n: usize) -> DynamicCollection {
        DynamicCollection::new(
            n,
            |i| ItemKey::new(&format!("item.{i}")),
            |i| {
                let mut s = WidgetSpec::new("ListItem");
                s.props
                    .insert("label", zero_ui_core::binding::Value::Text(format!("Item {i}")));
                s
            },
        )
    }

    #[test]
    fn select_within_bounds() {
        let mut lv = ListView::new(5);
        assert!(lv.select(2));
        assert_eq!(lv.selected, Some(2));
    }

    #[test]
    fn select_out_of_bounds_rejected() {
        let mut lv = ListView::new(5);
        assert!(!lv.select(10));
    }

    #[test]
    fn set_count_maintains_selection() {
        let mut lv = ListView::new(10);
        lv.select(7);
        lv.set_count(5);
        assert!(lv.selected.is_none(), "selection beyond new count cleared");
    }

    #[test]
    fn visible_window_scrolls() {
        let lv = ListView {
            item_count: 1000,
            scroll_offset: 800.0,
            viewport_height: 400.0,
            item_height_px: 40.0,
            ..ListView::new(1000)
        };
        let w = lv.visible_window();
        assert_eq!(w.start, 20);
        assert!(w.end > w.start && w.end <= 1000);
    }

    #[test]
    fn materialize_only_visible_items() {
        let c = test_collection(10_000);
        let lv = ListView {
            item_count: 10_000,
            scroll_offset: 0.0,
            viewport_height: 400.0,
            item_height_px: 40.0,
            ..ListView::new(10_000)
        };
        let items = lv.materialize_items(&c);
        assert!(!items.is_empty());
        assert!(items.len() <= 12, "window bounded, got {}", items.len());
        assert_eq!(items[0].key, ItemKey::new("item.0"));
    }

    #[test]
    fn window_at_end_returns_empty() {
        let c = test_collection(10);
        let lv = ListView {
            item_count: 10,
            scroll_offset: 9999.0,
            viewport_height: 400.0,
            item_height_px: 40.0,
            ..ListView::new(10)
        };
        assert!(lv.materialize_items(&c).is_empty());
    }
}
