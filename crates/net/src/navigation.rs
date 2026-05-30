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
    /// `max_entries` 为最大历史条目数，超出时自动丢弃最早的条目。
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            current_index: 0,
            max_entries,
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
    pub fn go_forward(&mut self) -> Option<&HistoryEntry> {
        if self.current_index < self.entries.len() - 1 {
            self.current_index += 1;
            Some(&self.entries[self.current_index])
        } else {
            None
        }
    }

    /// 后退 N 步。
    pub fn go_back_n(&mut self, n: usize) -> Option<&HistoryEntry> {
        if n > self.current_index {
            return None;
        }
        self.current_index -= n;
        Some(&self.entries[self.current_index])
    }

    /// 前进 N 步。
    pub fn go_forward_n(&mut self, n: usize) -> Option<&HistoryEntry> {
        let target = self.current_index + n;
        if target >= self.entries.len() {
            return None;
        }
        self.current_index = target;
        Some(&self.entries[self.current_index])
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
}
