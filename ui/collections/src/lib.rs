//! # zero-ui-collections
//!
//! 虚拟化集合（spec §8.4.1 `zero-ui-collections` / FR-016 / IF-010 `VirtualCollection` /
//! §8.4.1B 万条记录不卡顿、§8.8 materialization + stable key 测）。
//!
//! 提供 [`LazyList`]（按 scroll offset 算可视窗口）+ [`VirtualCollection`] trait（IF-010：
//! item_count / item_key / build_item）+ [`materialize`]（物化可视窗口为 WidgetSpec）+
//! [`find_duplicate_key`]（key 去重 diagnostic）+ [`Selection`]。

use compact_str::CompactString;
use hashbrown::{HashMap, HashSet};
use zero_ui_core::widget::WidgetSpec;

/// 稳定条目身份（用于跨重建保持状态、recycler 复用）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemKey(pub CompactString);

impl ItemKey {
    pub fn new(id: &str) -> ItemKey {
        ItemKey(CompactString::new(id))
    }
}

/// 虚拟列表可视窗口（半开区间 `[start, end)`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleWindow {
    pub start: usize,
    pub end: usize,
}

impl VisibleWindow {
    pub const EMPTY: VisibleWindow = VisibleWindow { start: 0, end: 0 };

    /// 窗口内索引迭代。
    pub fn iter(self) -> impl Iterator<Item = usize> {
        self.start..self.end
    }

    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

/// Lazy list：根据 scroll offset + item 高度算可见区间（定高条目）。
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

    /// 给定 scroll offset（px），返回可见 `[start, end)`（含上下各一条缓冲，避免边缘闪烁）。
    pub fn window_at(&self, scroll_y: f32) -> VisibleWindow {
        let h = self.item_height_px.max(1.0);
        let start = ((scroll_y / h).floor().max(0.0) as usize).min(self.total);
        let visible_count = ((self.viewport_height_px / h).ceil() as usize) + 1;
        let end = (start + visible_count).min(self.total);
        VisibleWindow { start, end }
    }
}

/// 物化结果：一个可视条目的索引 + 稳定 key + 声明树 spec。
#[derive(Debug, Clone)]
pub struct MaterializedItem {
    pub index: usize,
    pub key: ItemKey,
    pub spec: WidgetSpec,
}

/// 虚拟集合 trait（spec IF-010）。
///
/// 宿主只对**可视窗口**内的索引调 `item_key` / `build_item`，实现万条记录不卡顿（§8.4.1B）。
/// `item_key` 必须稳定（同索引同 key，跨重建不变）；重复 key 由 [`find_duplicate_key`] 诊断。
pub trait VirtualCollection {
    fn item_count(&self) -> usize;
    fn item_key(&self, index: usize) -> ItemKey;
    fn build_item(&self, index: usize) -> WidgetSpec;
}

/// 物化可视窗口：对 `[start, end)` 调 `item_key` + `build_item`，跳过越界索引（防御）。
///
/// 返回 [`MaterializedItem`] 列表（按索引升序）。这是宿主把虚拟集合渲染为声明树的桥梁
/// （§8.8 materialization 测）：只构建可见条目，recycler 按 [`ItemKey`] 复用 widget 实例。
pub fn materialize<C: VirtualCollection + ?Sized>(c: &C, window: VisibleWindow) -> Vec<MaterializedItem> {
    let count = c.item_count();
    window
        .iter()
        .filter(|&i| i < count)
        .map(|i| MaterializedItem {
            index: i,
            key: c.item_key(i),
            spec: c.build_item(i),
        })
        .collect()
}

/// 便捷物化：按 `LazyList` 在 `scroll_y` 处算窗口并物化。
pub fn materialize_at<C: VirtualCollection + ?Sized>(c: &C, list: &LazyList, scroll_y: f32) -> Vec<MaterializedItem> {
    materialize(c, list.window_at(scroll_y))
}

/// 检测重复 key（spec IF-010 错误处理：virtual item key 重复 → diagnostic，不得静默）。
/// 扫描全量条目，返回首个重复出现的 key（`None` 表示无重复）。
///
/// 注意：这是 O(n) 全量扫描，仅用于测试 / 调试 / 小集合；大集合应由数据源保证 key 唯一。
pub fn find_duplicate_key<C: VirtualCollection + ?Sized>(c: &C) -> Option<ItemKey> {
    let mut seen: HashSet<ItemKey> = HashSet::new();
    for i in 0..c.item_count() {
        let key = c.item_key(i);
        if !seen.insert(key.clone()) {
            return Some(key);
        }
    }
    None
}

/// 选择集（单选/多选通用，按 [`ItemKey`]）。
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
    pub fn select(&mut self, key: ItemKey) {
        self.keys.insert(key);
    }
    pub fn clear(&mut self) {
        self.keys.clear();
    }
    pub fn contains(&self, key: &ItemKey) -> bool {
        self.keys.contains(key)
    }
    pub fn len(&self) -> usize {
        self.keys.len()
    }
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

/// 按可视窗口物化后、对每个可见 key 缓存一份 widget 实例状态的 recycler。
///
/// 当窗口滚动时，离开窗口的 key 其状态可回收给新进入的 key（避免为每个 item 持有状态）。
/// 本类型提供「当前窗口的 key → 缓存值」映射与滚动后回收的钩子；缓存值类型由调用方决定。
#[derive(Debug, Default)]
pub struct Recycler<V> {
    slots: HashMap<ItemKey, V>,
}

impl<V> Recycler<V> {
    pub fn new() -> Recycler<V> {
        Recycler { slots: HashMap::new() }
    }

    /// 当前缓存的 key 数（= 物化窗口内已分配状态数）。
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// 按 key 取缓存值。
    pub fn get(&self, key: &ItemKey) -> Option<&V> {
        self.slots.get(key)
    }

    /// 为新窗口回收：保留 `keep` 中的 key，移除其余；返回被回收（移除）的 key 数。
    /// 调用方随后为仍缺失的 key 重新分配状态（用回收的 slot）。
    pub fn retain_window(&mut self, keep: &HashSet<ItemKey>) -> usize {
        let before = self.slots.len();
        self.slots.retain(|k, _| keep.contains(k));
        before - self.slots.len()
    }

    /// 插入/更新一个 key 的缓存值。
    pub fn set(&mut self, key: ItemKey, value: V) {
        self.slots.insert(key, value);
    }
}

/// 闭包支撑的虚拟集合（测试 / 简单数据源）：count + key 闭包 + build 闭包。
pub struct DynamicCollection {
    count: usize,
    key_fn: Box<dyn Fn(usize) -> ItemKey>,
    build_fn: Box<dyn Fn(usize) -> WidgetSpec>,
}

impl DynamicCollection {
    pub fn new<Fk, Fb>(count: usize, key_fn: Fk, build_fn: Fb) -> DynamicCollection
    where
        Fk: Fn(usize) -> ItemKey + 'static,
        Fb: Fn(usize) -> WidgetSpec + 'static,
    {
        DynamicCollection {
            count,
            key_fn: Box::new(key_fn),
            build_fn: Box::new(build_fn),
        }
    }
}

impl VirtualCollection for DynamicCollection {
    fn item_count(&self) -> usize {
        self.count
    }
    fn item_key(&self, index: usize) -> ItemKey {
        (self.key_fn)(index)
    }
    fn build_item(&self, index: usize) -> WidgetSpec {
        (self.build_fn)(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dl_history(n: usize) -> DynamicCollection {
        // 模拟下载历史：item i -> key "dl.{i}", spec component "DownloadItem"。
        DynamicCollection::new(
            n,
            |i| ItemKey::new(&format!("dl.{i}")),
            |i| {
                let mut s = WidgetSpec::new("DownloadItem");
                s.props
                    .insert("title", zero_ui_core::binding::Value::Text(format!("File {i}")));
                s
            },
        )
    }

    #[test]
    fn lazy_list_window_clamps() {
        let list = LazyList::new(1000, 40.0, 400.0);
        let w = list.window_at(800.0);
        assert_eq!(w.start, 20);
        assert!(w.end <= 32 && w.end > w.start);
        // 超出末尾 → start clamps to total，end == start。
        let w2 = list.window_at(100_000.0);
        assert_eq!(w2.start, 1000);
        assert_eq!(w2.end, 1000);
        assert!(w2.is_empty());
    }

    #[test]
    fn materialize_only_visible_window_items() {
        // §8.4.1B 万条记录：只物化可见窗口，不构建全部。
        let c = dl_history(10_000);
        let list = LazyList::new(10_000, 40.0, 400.0);
        let items = materialize_at(&c, &list, 800.0); // start=20
        assert!(!items.is_empty());
        // 第一项 index=20，key "dl.20"，spec component DownloadItem。
        assert_eq!(items[0].index, 20);
        assert_eq!(items[0].key, ItemKey::new("dl.20"));
        assert_eq!(items[0].spec.component.0.as_str(), "DownloadItem");
        // 物化条目数 ≈ 可见窗口大小（远小于 10000）。
        assert!(
            items.len() <= 12,
            "materialized count bounded by window, got {}",
            items.len()
        );
        // 索引升序。
        let idx: Vec<usize> = items.iter().map(|m| m.index).collect();
        let mut sorted = idx.clone();
        sorted.sort();
        assert_eq!(idx, sorted);
    }

    #[test]
    fn materialize_skips_out_of_range() {
        // 窗口超出 count → 越界索引被跳过（防御）。
        let c = dl_history(5);
        let w = VisibleWindow { start: 0, end: 10 };
        let items = materialize(&c, w);
        assert_eq!(items.len(), 5, "only existing items materialized");
        assert_eq!(items.last().unwrap().index, 4);
    }

    #[test]
    fn materialize_empty_window() {
        let c = dl_history(100);
        assert!(materialize(&c, VisibleWindow::EMPTY).is_empty());
        assert!(materialize(&c, VisibleWindow { start: 5, end: 5 }).is_empty());
    }

    #[test]
    fn stable_keys_across_calls() {
        // §8.8 stable key：同索引同 key，跨调用不变（recycler 据此复用状态）。
        let c = dl_history(100);
        let k1 = c.item_key(42);
        let k2 = c.item_key(42);
        assert_eq!(k1, k2);
        assert_eq!(k1, ItemKey::new("dl.42"));
        // 不同索引不同 key。
        assert_ne!(c.item_key(0), c.item_key(1));
    }

    #[test]
    fn find_duplicate_key_returns_first_dup() {
        let c = DynamicCollection::new(
            5,
            |i| {
                if i == 2 || i == 4 {
                    ItemKey::new("dup")
                } else {
                    ItemKey::new(&format!("u{i}"))
                }
            },
            |_| WidgetSpec::new("Item"),
        );
        assert_eq!(find_duplicate_key(&c), Some(ItemKey::new("dup")));
        // 无重复 → None。
        let clean = DynamicCollection::new(5, |i| ItemKey::new(&format!("u{i}")), |_| WidgetSpec::new("Item"));
        assert!(find_duplicate_key(&clean).is_none());
    }

    #[test]
    fn selection_toggle_select_clear() {
        let mut s = Selection::new();
        let k = ItemKey::new("a");
        s.toggle(k.clone());
        assert!(s.contains(&k));
        assert_eq!(s.len(), 1);
        s.toggle(k.clone());
        assert!(!s.contains(&k));
        // select + clear。
        s.select(ItemKey::new("b"));
        s.select(ItemKey::new("c"));
        assert_eq!(s.len(), 2);
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn recycler_retains_only_window_keys() {
        // 滚动后：recycler 只保留新窗口的 key，回收其余。
        let mut r: Recycler<u32> = Recycler::new();
        r.set(ItemKey::new("dl.0"), 100);
        r.set(ItemKey::new("dl.1"), 101);
        r.set(ItemKey::new("dl.2"), 102);
        assert_eq!(r.len(), 3);
        // 新窗口只含 dl.1 / dl.2。
        let mut keep = HashSet::new();
        keep.insert(ItemKey::new("dl.1"));
        keep.insert(ItemKey::new("dl.2"));
        let recycled = r.retain_window(&keep);
        assert_eq!(recycled, 1, "dl.0 recycled");
        assert_eq!(r.len(), 2);
        assert!(r.get(&ItemKey::new("dl.1")).is_some());
        assert!(r.get(&ItemKey::new("dl.0")).is_none());
    }
}
