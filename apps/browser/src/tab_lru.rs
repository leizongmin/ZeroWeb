//! 后台 Tab LRU — 超出上限时冻结 worker，仅保留 UI 快照。

use std::collections::{HashMap, HashSet, VecDeque};

use zero_browser_shell::TabId;

/// 默认最多同时保持多少个 Tab worker 存活。
pub const DEFAULT_MAX_LIVE_WORKERS: usize = 3;

/// 后台 Tab LRU 策略状态。
pub struct TabLruPolicy {
    max_live: usize,
    /// 最近切换离开的顺序（队首 = 最先冻结候选）。
    deactivate_order: VecDeque<TabId>,
    /// 已冻结（worker 已关闭，快照保留）。
    frozen: HashSet<TabId>,
    /// 解冻后用于恢复导航的 URL。
    restore_urls: HashMap<TabId, String>,
}

impl TabLruPolicy {
    /// 创建 LRU 策略。
    pub fn new(max_live: usize) -> Self {
        Self {
            max_live: max_live.max(1),
            deactivate_order: VecDeque::new(),
            frozen: HashSet::new(),
            restore_urls: HashMap::new(),
        }
    }

    /// Tab 失去前台时调用。
    pub fn note_deactivated(&mut self, tab_id: TabId) {
        self.deactivate_order.retain(|&id| id != tab_id);
        self.deactivate_order.push_back(tab_id);
    }

    /// 是否已冻结。
    pub fn is_frozen(&self, tab_id: TabId) -> bool {
        self.frozen.contains(&tab_id)
    }

    /// 标记为已冻结。
    pub fn mark_frozen(&mut self, tab_id: TabId, restore_url: Option<String>) {
        self.frozen.insert(tab_id);
        self.deactivate_order.retain(|&id| id != tab_id);
        if let Some(url) = restore_url {
            self.restore_urls.insert(tab_id, url);
        }
    }

    /// 解冻：清除冻结标记并返回待恢复 URL。
    pub fn thaw(&mut self, tab_id: TabId) -> Option<String> {
        self.frozen.remove(&tab_id);
        self.restore_urls.remove(&tab_id)
    }

    /// 移除 Tab 时清理 LRU 状态。
    pub fn remove_tab(&mut self, tab_id: TabId) {
        self.deactivate_order.retain(|&id| id != tab_id);
        self.frozen.remove(&tab_id);
        self.restore_urls.remove(&tab_id);
    }

    /// 选择可冻结的后台 Tab（非 `active` 且仍有 worker）。
    pub fn pick_freeze_victim(&self, active: Option<TabId>, live_workers: &HashMap<TabId, ()>) -> Option<TabId> {
        for &id in &self.deactivate_order {
            if Some(id) == active {
                continue;
            }
            if live_workers.contains_key(&id) && !self.frozen.contains(&id) {
                return Some(id);
            }
        }
        live_workers
            .keys()
            .copied()
            .find(|&id| Some(id) != active && !self.frozen.contains(&id))
    }

    /// 当前 live worker 是否超过上限。
    pub fn should_freeze(&self, live_count: usize, active: Option<TabId>) -> bool {
        if live_count <= self.max_live {
            return false;
        }
        active.is_some() || live_count > self.max_live
    }

    /// 最大 live worker 数。
    pub fn max_live(&self) -> usize {
        self.max_live
    }
}

impl Default for TabLruPolicy {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_LIVE_WORKERS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_victim_skips_active_tab() {
        let mut lru = TabLruPolicy::new(2);
        let a = TabId(1);
        let b = TabId(2);
        lru.note_deactivated(a);
        let mut live = HashMap::new();
        live.insert(a, ());
        live.insert(b, ());
        assert_eq!(lru.pick_freeze_victim(Some(a), &live), Some(b));
    }
}
