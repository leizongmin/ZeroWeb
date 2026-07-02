//! ListView — 可虚拟化的列表（spec FR-009 / §8.4.1B 大量记录走 `ui/collections`）。
//!
//! M1 提供 selection 模型骨架；真实虚拟化（窗口化渲染、回收）在 `ui/collections`。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListView {
    pub item_count: usize,
    pub selected: Option<usize>,
}

impl ListView {
    pub fn new(item_count: usize) -> ListView {
        ListView {
            item_count,
            selected: None,
        }
    }
    pub fn select(&mut self, index: usize) -> bool {
        if index < self.item_count {
            self.selected = Some(index);
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
    fn select_within_bounds() {
        let mut lv = ListView::new(5);
        assert!(lv.select(2));
        assert_eq!(lv.selected, Some(2));
        assert!(!lv.select(10));
    }
}
