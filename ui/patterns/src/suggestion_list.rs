//! SuggestionList — 地址栏/搜索建议列表（spec FR-009 / §8.4.1B omnibox 建议）。
//!
//! 大量历史/搜索建议用 lazy collection（`ui/collections`）；M1 提供 selection 模型。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub label: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionList {
    pub items: Vec<Suggestion>,
    pub highlighted: Option<usize>,
}

impl SuggestionList {
    pub fn new(items: Vec<Suggestion>) -> SuggestionList {
        SuggestionList {
            items,
            highlighted: None,
        }
    }
    pub fn move_highlight(&mut self, dir: i32) {
        if self.items.is_empty() {
            return;
        }
        let cur = self.highlighted.unwrap_or(0) as i32;
        let next = (cur + dir).clamp(0, self.items.len() as i32 - 1) as usize;
        self.highlighted = Some(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_moves_and_clamps() {
        let mut s = SuggestionList::new(vec![
            Suggestion {
                label: "a".into(),
                detail: None,
            },
            Suggestion {
                label: "b".into(),
                detail: None,
            },
        ]);
        s.move_highlight(1);
        assert_eq!(s.highlighted, Some(1));
        s.move_highlight(1); // clamp
        assert_eq!(s.highlighted, Some(1));
    }
}
