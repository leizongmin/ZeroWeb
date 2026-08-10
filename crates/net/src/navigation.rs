//! 导航历史管理模块。
//!
//! 提供浏览器风格的导航历史栈（前进/后退/替换）。

/// 导航历史条目。
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// 页面 URL。
    pub url: String,
    /// 页面标题。
    pub title: Option<String>,
}

/// 导航历史管理器。
pub struct NavigationHistory {
    entries: Vec<HistoryEntry>,
    current_index: usize,
    max_entries: usize,
}

impl NavigationHistory {
    /// 创建新的导航历史管理器。
    ///
    /// `max_entries` 为最大历史条目数，超出时自动丢弃最早的条目。最小为 1（0 归一化为 1，
    /// 否则 navigate 的驱逐循环会清空 entries 后留下悬空 current_index）。
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            current_index: 0,
            max_entries: max_entries.max(1),
        }
    }

    /// 导航到新 URL（清除前进历史）。
    pub fn navigate(&mut self, url: &str, title: Option<String>) {
        // 清除当前之后的所有条目
        self.entries.truncate(self.current_index + 1);

        // 添加新条目
        self.entries.push(HistoryEntry {
            url: url.to_string(),
            title,
        });

        self.current_index = self.entries.len() - 1;

        // 如果超出最大条目数，移除最早的条目
        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
            if self.current_index > 0 {
                self.current_index -= 1;
            }
        }
    }

    /// 后退一步。
    pub fn go_back(&mut self) -> Option<&HistoryEntry> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.entries[self.current_index])
        } else {
            None
        }
    }

    /// 前进一步。
    ///
    /// R3230：HTML session-history 语义——无下一项时为 no-op（返 None），**不 panic**。
    /// 旧实现 `len() - 1` 在空历史下 usize 下溢（debug panic / release 经 wrap 致越界 panic）。
    pub fn go_forward(&mut self) -> Option<&HistoryEntry> {
        let next = self.current_index.checked_add(1)?;
        let entry = self.entries.get(next)?;
        self.current_index = next;
        Some(entry)
    }

    /// 后退 N 步。
    ///
    /// R3230：越界或空历史时 no-op（None），不 panic。旧实现 `entries[self.current_index]`
    /// 在空历史 + n=0 时越界 panic。
    pub fn go_back_n(&mut self, n: usize) -> Option<&HistoryEntry> {
        let target = self.current_index.checked_sub(n)?;
        let entry = self.entries.get(target)?;
        self.current_index = target;
        Some(entry)
    }

    /// 前进 N 步。
    ///
    /// R3230：越界或空历史时 no-op（None），不 panic（统一 checked_add + get 模式）。
    pub fn go_forward_n(&mut self, n: usize) -> Option<&HistoryEntry> {
        let target = self.current_index.checked_add(n)?;
        let entry = self.entries.get(target)?;
        self.current_index = target;
        Some(entry)
    }

    /// 获取当前条目。
    pub fn current(&self) -> Option<&HistoryEntry> {
        self.entries.get(self.current_index)
    }

    /// 替换当前条目（replaceState）。
    pub fn replace_current(&mut self, url: &str, title: Option<String>) {
        if let Some(entry) = self.entries.get_mut(self.current_index) {
            entry.url = url.to_string();
            entry.title = title;
        }
    }

    /// 是否可以后退。
    pub fn can_go_back(&self) -> bool {
        self.current_index > 0
    }

    /// 是否可以前进。
    pub fn can_go_forward(&self) -> bool {
        !self.entries.is_empty() && self.current_index < self.entries.len() - 1
    }

    /// 历史长度。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 历史是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_new() {
        let nav = NavigationHistory::new(50);
        assert!(nav.is_empty());
        assert_eq!(nav.len(), 0);
        assert!(nav.current().is_none());
        assert!(!nav.can_go_back());
        assert!(!nav.can_go_forward());
    }

    #[test]
    fn test_navigation_navigate() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", Some("A".to_string()));
        assert_eq!(nav.len(), 1);
        assert_eq!(nav.current().unwrap().url, "http://a.com");

        nav.navigate("http://b.com", Some("B".to_string()));
        assert_eq!(nav.len(), 2);
        assert_eq!(nav.current().unwrap().url, "http://b.com");
    }

    #[test]
    fn test_navigation_go_back() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);

        let entry = nav.go_back().unwrap();
        assert_eq!(entry.url, "http://b.com");
        assert_eq!(nav.current().unwrap().url, "http://b.com");
    }

    #[test]
    fn test_navigation_go_forward() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.go_back();

        let entry = nav.go_forward().unwrap();
        assert_eq!(entry.url, "http://b.com");
        assert_eq!(nav.current().unwrap().url, "http://b.com");
    }

    #[test]
    fn test_navigation_replace() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", Some("Old".to_string()));
        nav.replace_current("http://replaced.com", Some("New".to_string()));

        assert_eq!(nav.current().unwrap().url, "http://replaced.com");
        assert_eq!(nav.current().unwrap().title.as_deref(), Some("New"));
    }

    #[test]
    fn test_navigation_clears_forward() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.go_back(); // now at b.com

        // Navigate should clear forward history (c.com)
        nav.navigate("http://d.com", None);
        assert_eq!(nav.len(), 3); // a, b, d
        assert!(!nav.can_go_forward());
    }

    #[test]
    fn test_navigation_max_entries() {
        let mut nav = NavigationHistory::new(3);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.navigate("http://d.com", None); // a.com should be evicted
        assert_eq!(nav.len(), 3);
        assert_eq!(nav.current().unwrap().url, "http://d.com");
    }

    #[test]
    fn test_navigation_cannot_go_beyond() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);

        // Can't go back from first entry
        assert!(nav.go_back().is_none());
        // Can't go forward when at the end
        assert!(nav.go_forward().is_none());
        // Can't go back N beyond start
        assert!(nav.go_back_n(5).is_none());
        // Can't go forward N beyond end
        assert!(nav.go_forward_n(5).is_none());
    }

    #[test]
    fn test_navigation_go_back_n_happy_path() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.navigate("http://d.com", None);
        let entry = nav.go_back_n(2).unwrap();
        assert_eq!(entry.url, "http://b.com");
        assert_eq!(nav.current().unwrap().url, "http://b.com");
    }

    #[test]
    fn test_navigation_go_forward_n_happy_path() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.go_back_n(2); // at a.com
        let entry = nav.go_forward_n(2).unwrap();
        assert_eq!(entry.url, "http://c.com");
    }

    #[test]
    fn test_navigation_replace_empty_history() {
        let mut nav = NavigationHistory::new(50);
        nav.replace_current("http://x.com", None);
        // 空历史中 replace 是 no-op
        assert!(nav.is_empty());
    }

    #[test]
    fn test_navigation_max_entries_one() {
        let mut nav = NavigationHistory::new(1);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        assert_eq!(nav.len(), 1);
        assert_eq!(nav.current().unwrap().url, "http://b.com");
    }

    #[test]
    fn test_navigation_multiple_back_then_forward() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.navigate("http://d.com", None);
        nav.go_back(); // c
        nav.go_back(); // b
        assert_eq!(nav.current().unwrap().url, "http://b.com");
        nav.go_forward(); // c
        nav.go_forward(); // d
        assert_eq!(nav.current().unwrap().url, "http://d.com");
    }

    #[test]
    fn test_navigation_eviction_and_go_back() {
        let mut nav = NavigationHistory::new(2);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None); // a 被淘汰
        assert_eq!(nav.len(), 2);
        let entry = nav.go_back().unwrap();
        assert_eq!(entry.url, "http://b.com");
        assert!(nav.go_back().is_none()); // a 已被淘汰
    }

    // ── Additional navigation history tests ──

    /// 测试 replace_current 后历史长度不变。
    #[test]
    fn test_replace_preserves_length() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", Some("A".to_string()));
        nav.navigate("http://b.com", Some("B".to_string()));
        assert_eq!(nav.len(), 2);

        nav.replace_current("http://replaced.com", Some("R".to_string()));
        assert_eq!(nav.len(), 2, "replace 不应改变历史长度");
        assert_eq!(nav.current().unwrap().url, "http://replaced.com");
    }

    /// 测试 replace_current 在历史中间位置正常工作。
    #[test]
    fn test_replace_in_middle_of_history() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.go_back(); // at b.com

        nav.replace_current("http://b-new.com", Some("B New".to_string()));
        assert_eq!(nav.current().unwrap().url, "http://b-new.com");
        assert_eq!(nav.current().unwrap().title.as_deref(), Some("B New"));
        // forward history (c.com) should still exist
        assert!(nav.can_go_forward());
        let fwd = nav.go_forward().unwrap();
        assert_eq!(fwd.url, "http://c.com");
    }

    /// 测试导航到相同 URL 仍然添加新条目。
    #[test]
    fn test_navigate_same_url_adds_entry() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", Some("First".to_string()));
        nav.navigate("http://a.com", Some("Second".to_string()));

        assert_eq!(nav.len(), 2, "相同 URL 也应产生新的历史条目");
        assert_eq!(nav.current().unwrap().title.as_deref(), Some("Second"));
    }

    /// 测试连续后退和前进后的状态一致性。
    #[test]
    fn test_back_forward_consistency() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.navigate("http://d.com", None);

        // 后退两步到 b
        nav.go_back();
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://b.com");

        // 前进一步到 c
        nav.go_forward();
        assert_eq!(nav.current().unwrap().url, "http://c.com");

        // 再后退一步到 b
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://b.com");

        // 导航新 URL 清除 c,d
        nav.navigate("http://e.com", None);
        assert_eq!(nav.len(), 3); // a, b, e
        assert!(!nav.can_go_forward());
    }

    /// 测试 go_back_n(0) 返回当前条目。
    #[test]
    fn test_go_back_n_zero() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);

        let entry = nav.go_back_n(0).unwrap();
        assert_eq!(entry.url, "http://b.com");
        assert_eq!(nav.current().unwrap().url, "http://b.com");
    }

    /// 测试 go_forward_n(0) 返回当前条目。
    #[test]
    fn test_go_forward_n_zero() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.go_back(); // at a

        let entry = nav.go_forward_n(0).unwrap();
        assert_eq!(entry.url, "http://a.com");
    }

    /// 测试在边界处 can_go_back / can_go_forward 正确返回。
    #[test]
    fn test_can_go_back_forward_at_boundaries() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        assert!(!nav.can_go_back(), "第一个条目不应能后退");
        assert!(!nav.can_go_forward(), "最新条目不应能前进");

        nav.navigate("http://b.com", None);
        assert!(nav.can_go_back(), "非第一个条目应能后退");
        assert!(!nav.can_go_forward());

        nav.go_back();
        assert!(!nav.can_go_back());
        assert!(nav.can_go_forward(), "有前进历史时应能前进");
    }

    /// 测试空历史记录的后退/前进行为。
    #[test]
    fn test_empty_history_back_forward() {
        let nav = NavigationHistory::new(50);
        assert!(!nav.can_go_back(), "空历史不应能后退");
        assert!(!nav.can_go_forward(), "空历史不应能前进");
        assert!(nav.current().is_none(), "空历史当前条目应为 None");
    }

    /// 测试 replace_state 在历史中间位置替换当前条目。
    #[test]
    fn test_replace_state_in_middle() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", Some("A".into()));
        nav.navigate("http://b.com", Some("B".into()));
        nav.navigate("http://c.com", Some("C".into()));
        nav.go_back(); // at b

        nav.replace_current("http://b-new.com", Some("B-New".into()));
        let current = nav.current().unwrap();
        assert_eq!(current.url, "http://b-new.com");
        assert_eq!(current.title, Some("B-New".into()));

        // 前进历史不受影响
        let fwd = nav.go_forward().unwrap();
        assert_eq!(fwd.url, "http://c.com");

        // 后退到 a 不受影响
        nav.go_back(); // at b-new
        nav.go_back(); // at a
        assert_eq!(nav.current().unwrap().url, "http://a.com");
    }

    /// 测试 max_entries=1 的极端容量限制。
    #[test]
    fn test_max_entries_one() {
        let mut nav = NavigationHistory::new(1);
        nav.navigate("http://a.com", None);
        assert_eq!(nav.len(), 1);

        nav.navigate("http://b.com", None);
        assert_eq!(nav.len(), 1);
        assert_eq!(nav.current().unwrap().url, "http://b.com");
        assert!(!nav.can_go_back(), "max=1 不应有后退历史");
    }

    /// 测试连续后退后前进保持状态一致。
    #[test]
    fn test_sequential_back_then_forward() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.navigate("http://d.com", None);

        // 连续后退到 a
        assert_eq!(nav.go_back().unwrap().url, "http://c.com");
        assert_eq!(nav.go_back().unwrap().url, "http://b.com");
        assert_eq!(nav.go_back().unwrap().url, "http://a.com");

        // 连续前进到 d
        assert_eq!(nav.go_forward().unwrap().url, "http://b.com");
        assert_eq!(nav.go_forward().unwrap().url, "http://c.com");
        assert_eq!(nav.go_forward().unwrap().url, "http://d.com");
    }

    /// 测试 navigate 在后退位置清除前进历史。
    #[test]
    fn test_navigate_clears_forward_at_middle() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.go_back(); // at b

        // 新导航应清除 c
        nav.navigate("http://d.com", Some("D".into()));
        assert!(!nav.can_go_forward(), "新导航后不应有前进历史");
        assert_eq!(nav.current().unwrap().url, "http://d.com");

        // 后退一步到 b
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://b.com");

        // 再后退到 a
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://a.com");
    }

    /// 测试 go_back_n 和 go_forward_n 的边界值。
    #[test]
    fn test_go_back_forward_n_boundary() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        nav.navigate("http://d.com", None);

        // 后退 3 步到 a
        let entry = nav.go_back_n(3).unwrap();
        assert_eq!(entry.url, "http://a.com");

        // 再后退应失败
        assert!(nav.go_back_n(1).is_none());

        // 前进 3 步到 d
        let entry = nav.go_forward_n(3).unwrap();
        assert_eq!(entry.url, "http://d.com");

        // 再前进应失败
        assert!(nav.go_forward_n(1).is_none());
    }

    // ── 新增边界条件测试 ──

    /// Fresh NavigationHistory → can_go_back=false, can_go_forward=false.
    /// 验证新创建的导航历史管理器初始状态正确。
    #[test]
    fn test_navigation_initial_state() {
        let nav = NavigationHistory::new(50);
        assert!(!nav.can_go_back(), "初始状态不应能后退");
        assert!(!nav.can_go_forward(), "初始状态不应能前进");
        assert!(nav.is_empty(), "初始状态应为空");
        assert_eq!(nav.len(), 0, "初始长度应为 0");
        assert!(nav.current().is_none(), "初始状态当前条目应为 None");
    }

    /// 测试 replace_current 在无历史条目时为 no-op，不产生任何副作用。
    #[test]
    fn test_navigation_replace_current_when_empty() {
        let mut nav = NavigationHistory::new(50);
        nav.replace_current("http://should-not-exist.com", Some("不应存在".into()));
        assert!(nav.is_empty(), "空历史中 replace_current 应为 no-op");
        assert_eq!(nav.len(), 0);
        assert!(nav.current().is_none());
    }

    /// 测试 go_forward() 在没有前进历史时为 no-op，返回 None 且状态不变。
    #[test]
    fn test_navigation_go_forward_beyond_available() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);

        // 当前在最新条目（b），go_forward() 应返回 None
        assert!(nav.go_forward().is_none(), "没有前进历史时 go_forward 应返回 None");
        assert_eq!(nav.current().unwrap().url, "http://b.com", "状态不应改变");

        // 后退一步后再尝试前进到末尾再 go_forward
        nav.go_back(); // at a
        nav.go_forward(); // at b (end)
        assert!(nav.go_forward().is_none(), "到达最新条目后 go_forward 应返回 None");
        assert_eq!(nav.current().unwrap().url, "http://b.com");
    }

    /// 测试后退后新导航会清除前进历史。
    #[test]
    fn test_navigation_clear_forward_on_new_navigate() {
        let mut nav = NavigationHistory::new(50);
        nav.navigate("http://a.com", Some("A".into()));
        nav.navigate("http://b.com", Some("B".into()));
        nav.navigate("http://c.com", Some("C".into()));

        // 后退到 b → 前进历史为 [c]
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://b.com");
        assert!(nav.can_go_forward(), "后退后应有前进历史");

        // 新导航清除前进历史
        nav.navigate("http://d.com", Some("D".into()));
        assert!(!nav.can_go_forward(), "新导航后前进历史应被清除");
        assert_eq!(nav.len(), 3, "历史应为 a, b, d");
        assert_eq!(nav.current().unwrap().url, "http://d.com");

        // 验证 c 确实被移除：后退到 b，再后退到 a
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://b.com");
        nav.go_back();
        assert_eq!(nav.current().unwrap().url, "http://a.com");
        // 无法再后退
        assert!(!nav.can_go_back());
    }

    /// 测试恰好 max_entries 条目时不触发淘汰。
    #[test]
    fn test_navigation_max_entries_boundary() {
        let mut nav = NavigationHistory::new(3);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None);
        // 恰好 3 条，等于 max_entries，不应淘汰
        assert_eq!(nav.len(), 3, "恰好 max_entries 条目不应淘汰");
        assert!(nav.can_go_back());
        // 添加第 4 条时才应淘汰最旧的
        nav.navigate("http://d.com", None);
        assert_eq!(nav.len(), 3, "超出 max_entries 后应淘汰到 3 条");
    }

    // ── 边界测试 ──

    #[test]
    /// 测试 navigate("") 添加空 URL 条目。
    fn test_navigate_empty_url() {
        let mut nav = NavigationHistory::new(10);
        nav.navigate("", None);
        assert_eq!(nav.len(), 1);
        assert_eq!(nav.current().unwrap().url, "");
    }

    #[test]
    /// 测试 evict 后 go_back_n(0) 返回当前条目。
    fn test_go_back_n_zero_after_eviction() {
        let mut nav = NavigationHistory::new(2);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None); // 淘汰 a.com
        assert_eq!(nav.len(), 2);
        let result = nav.go_back_n(0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().url, "http://c.com");
    }

    /// R3230：空历史下 go_forward/go_back/go_back_n/go_forward_n 须 no-op（None），不 panic。
    /// 旧实现 go_forward `len()-1` usize 下溢（debug panic / release 越界 panic）；
    /// go_back_n(0) `entries[0]` 空历史越界 panic。
    #[test]
    fn test_navigation_empty_history_no_panic_r3230() {
        let mut nav = NavigationHistory::new(10);
        // 空历史——所有 go* 调用须返 None，不 panic。
        assert!(
            nav.go_forward().is_none(),
            "go_forward() 空历史须 None（旧 usize 下溢 panic）"
        );
        assert!(nav.go_back().is_none());
        assert!(
            nav.go_back_n(0).is_none(),
            "go_back_n(0) 空历史须 None（旧 entries[0] 越界 panic）"
        );
        assert!(nav.go_forward_n(0).is_none());
        assert!(nav.go_back_n(5).is_none());
        assert!(nav.go_forward_n(3).is_none());
        assert!(nav.current().is_none());
        assert!(!nav.can_go_back());
        assert!(!nav.can_go_forward());
    }

    /// R3230：max_entries=0 归一化为 1（navigate 不再清空 entries 留悬空 current_index）。
    #[test]
    fn test_navigation_max_entries_zero_normalized_r3230() {
        let mut nav = NavigationHistory::new(0);
        nav.navigate("http://a.com", None);
        assert_eq!(nav.len(), 1, "max_entries=0 归一化为 1，navigate 后 1 条目");
        assert_eq!(nav.current().unwrap().url, "http://a.com");
        // 再 navigate（驱逐最旧，但归一化 max=1 → 保留最新）
        nav.navigate("http://b.com", None);
        assert_eq!(nav.len(), 1);
        assert_eq!(nav.current().unwrap().url, "http://b.com");
    }

    /// R3230：非空历史下 go_forward_n / go_back_n 正常路径回归（checked_add/checked_sub 改写后行为不变）。
    #[test]
    fn test_navigation_go_n_nonempty_r3230() {
        let mut nav = NavigationHistory::new(10);
        nav.navigate("http://a.com", None);
        nav.navigate("http://b.com", None);
        nav.navigate("http://c.com", None); // current=c (idx 2)
        // go_back_n(2) → a (idx 0)
        assert_eq!(nav.go_back_n(2).unwrap().url, "http://a.com");
        // go_forward_n(1) → b (idx 1)
        assert_eq!(nav.go_forward_n(1).unwrap().url, "http://b.com");
        // 越界 → None
        assert!(nav.go_forward_n(10).is_none());
        assert!(nav.go_back_n(10).is_none());
    }
}
