//! Tabs — 标签页容器（通用；浏览器领域 TabBar 见 `ui/patterns`，spec FR-009）。

use compact_str::CompactString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem {
    pub id: CompactString,
    pub label: CompactString,
}

impl TabItem {
    pub fn new(id: &str, label: &str) -> TabItem {
        TabItem {
            id: CompactString::new(id),
            label: CompactString::new(label),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tabs {
    pub items: Vec<TabItem>,
    pub active_index: Option<usize>,
}

impl Tabs {
    pub fn new(items: Vec<TabItem>) -> Tabs {
        Tabs {
            items,
            active_index: None,
        }
    }
    pub fn activate(&mut self, index: usize) -> bool {
        if index < self.items.len() {
            self.active_index = Some(index);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activate_tab() {
        let mut t = Tabs::new(vec![TabItem::new("a", "A"), TabItem::new("b", "B")]);
        assert!(t.activate(1));
        assert_eq!(t.active_index, Some(1));
        assert!(!t.activate(9));
    }
}
