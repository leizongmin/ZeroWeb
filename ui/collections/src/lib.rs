//! # zero-ui-collections
//!
//! 虚拟化集合（spec §8.4.1 `zero-ui-collections` / FR-016 / §8.4.1B 万条记录不卡顿）。
//!
//! M1 提供 ItemKey（稳定身份）、LazyList 可视窗口计算、Selection。

use compact_str::CompactString;
use hashbrown::HashSet;

/// 稳定条目身份（用于跨重建保持状态、recycler 复用）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemKey(pub CompactString);

impl ItemKey {
    pub fn new(id: &str) -> ItemKey {
        ItemKey(CompactString::new(id))
    }
}

/// 虚拟列表可视窗口。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleWindow {
    pub start: usize,
    pub end: usize,
}

/// Lazy list：根据 scroll offset + item 高度算可见区间。
#[derive(Debug, Clone, Copy)]
pub struct LazyList {
    pub total: usize,
    pub item_height_px: f32,
    pub viewport_height_px: f32,
}

impl LazyList {
    pub fn new(total: usize, item_height_px: f32, viewport_height_px: f32) -> LazyList {
        LazyList {
            total,
            item_height_px,
            viewport_height_px,
        }
    }

    /// 给定 scroll offset（px），返回可见 [start, end)。
    pub fn window_at(&self, scroll_y: f32) -> VisibleWindow {
        let h = self.item_height_px.max(1.0);
        let start = (scroll_y / h).floor() as usize;
        let visible_count = ((self.viewport_height_px / h).ceil() as usize) + 1;
        let end = (start + visible_count).min(self.total);
        VisibleWindow {
            start: start.min(self.total),
            end,
        }
    }
}

/// 选择集。
#[derive(Debug, Clone, Default)]
pub struct Selection {
    pub keys: HashSet<ItemKey>,
}

impl Selection {
    pub fn new() -> Selection {
        Selection::default()
    }
    pub fn toggle(&mut self, key: ItemKey) {
        if !self.keys.insert(key.clone()) {
            self.keys.remove(&key);
        }
    }
    pub fn contains(&self, key: &ItemKey) -> bool {
        self.keys.contains(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_list_window_clamps() {
        let list = LazyList::new(1000, 40.0, 400.0);
        let w = list.window_at(800.0);
        // start = 800/40 = 20；可见 ~10+1 条。
        assert_eq!(w.start, 20);
        assert!(w.end <= 32 && w.end > w.start);
        // 超出末尾 → end clamps to total。
        let w2 = list.window_at(100_000.0);
        assert_eq!(w2.start, 1000);
        assert_eq!(w2.end, 1000);
    }

    #[test]
    fn selection_toggle() {
        let mut s = Selection::new();
        let k = ItemKey::new("a");
        s.toggle(k.clone());
        assert!(s.contains(&k));
        s.toggle(k.clone());
        assert!(!s.contains(&k));
    }
}
