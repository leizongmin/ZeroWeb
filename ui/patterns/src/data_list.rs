//! DataList — 数据列表组合模式（spec FR-009；底层虚拟化在 `ui/collections`）。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataList {
    pub row_count: usize,
    pub selected: Option<usize>,
}

impl DataList {
    pub fn new(row_count: usize) -> DataList {
        DataList {
            row_count,
            selected: None,
        }
    }
    pub fn select(&mut self, index: usize) -> bool {
        if index < self.row_count {
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
    fn select_bounds() {
        let mut d = DataList::new(3);
        assert!(d.select(2));
        assert!(!d.select(5));
    }
}
