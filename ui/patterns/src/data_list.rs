//! DataList — 数据列表组合模式（spec FR-009；底层虚拟化在 `ui/collections`）。
//!
//! 组合 [`ListView`] + [`VirtualCollection`]：ListView 持有滚动/选择状态，
//! data_source 提供条目数据。`materialize()` 物化当前可见窗口。

use zero_ui_collections::{MaterializedItem, VirtualCollection};
use zero_ui_widgets::list_view::ListView;

/// 数据列表组合：持有 ListView 状态 + VirtualCollection 数据源引用。
pub struct DataList<'a, C: VirtualCollection> {
    pub view: &'a mut ListView,
    pub data: &'a C,
}

impl<'a, C: VirtualCollection> DataList<'a, C> {
    pub fn new(view: &'a mut ListView, data: &'a C) -> Self {
        DataList { view, data }
    }

    /// 物化当前可见窗口条目。
    pub fn materialize(&self) -> Vec<MaterializedItem> {
        self.view.materialize_items(self.data)
    }

    /// 选中索引处条目。
    pub fn select(&mut self, index: usize) -> bool {
        self.view.select(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_collections::{DynamicCollection, ItemKey};
    use zero_ui_core::widget::WidgetSpec;

    fn test_data(n: usize) -> DynamicCollection {
        DynamicCollection::new(
            n,
            |i| ItemKey::new(&format!("d.{i}")),
            |i| {
                let mut s = WidgetSpec::new("DataRow");
                s.props
                    .insert("label", zero_ui_core::binding::Value::Text(format!("Row {i}")));
                s
            },
        )
    }

    #[test]
    fn data_list_materializes_window() {
        let data = test_data(100);
        let mut lv = ListView::new(100);
        let dl = DataList::new(&mut lv, &data);
        let items = dl.materialize();
        // 默认视口 600px / 40px item ≈ 15 items
        assert!(!items.is_empty());
        assert!(items.len() <= 20);
        assert_eq!(items[0].index, 0);
    }

    #[test]
    fn data_list_select_updates_view() {
        let data = test_data(50);
        let mut lv = ListView::new(50);
        let mut dl = DataList::new(&mut lv, &data);
        assert!(dl.select(3));
        assert_eq!(dl.view.selected, Some(3));
    }
}
