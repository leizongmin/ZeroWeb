//! FindBar — 页面查找栏（spec §8.4.1A）。
//!
//! 组合通用 [`TextInputState`]（查询输入）+ [`StatusBubble`]（结果计数）；
//! action 进入 WebView find API，结果计数作为 props 回流。

use crate::browser_action::BrowserAction;
use zero_ui_patterns::status_bubble::StatusBubble;
use zero_ui_widgets::badge::BadgeTone;
use zero_ui_widgets::text_input::TextInputState;

/// 查找栏状态（props）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FindBar {
    pub query: String,
    /// 当前匹配序号（0-based）。
    pub match_index: Option<u32>,
    /// 匹配总数。
    pub match_count: Option<u32>,
    pub open: bool,
}

impl FindBar {
    pub fn new() -> FindBar {
        FindBar::default()
    }

    pub fn open() -> FindBar {
        FindBar {
            open: true,
            ..FindBar::default()
        }
    }

    /// 同步查询到 TextInput state（光标在末尾）。
    pub fn build_text_input(&self) -> TextInputState {
        let mut s = TextInputState::empty();
        s.insert(&self.query);
        s
    }

    /// 结果计数 StatusBubble；关闭或无计数时不显示。
    pub fn build_status(&self) -> Option<StatusBubble> {
        if !self.open {
            return None;
        }
        match (self.match_index, self.match_count) {
            (Some(i), Some(n)) => Some(StatusBubble::new(&format!("{}/{}", i + 1, n), BadgeTone::Neutral)),
            _ => None,
        }
    }

    /// 下一个 / 上一个 / 关闭：仅在打开且有查询时发 action。
    pub fn on_find_next(&self) -> Option<BrowserAction> {
        if self.open && !self.query.is_empty() {
            Some(BrowserAction::FindNext)
        } else {
            None
        }
    }

    pub fn on_find_prev(&self) -> Option<BrowserAction> {
        if self.open && !self.query.is_empty() {
            Some(BrowserAction::FindPrev)
        } else {
            None
        }
    }

    pub fn on_close(&self) -> Option<BrowserAction> {
        if self.open {
            Some(BrowserAction::FindClose)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_input_carries_query() {
        let mut f = FindBar::open();
        f.query = "hello".into();
        let ti = f.build_text_input();
        assert_eq!(ti.text, "hello");
    }

    #[test]
    fn status_shows_count_when_present() {
        let mut f = FindBar::open();
        f.match_index = Some(0);
        f.match_count = Some(5);
        assert_eq!(f.build_status().unwrap().text, "1/5");
    }

    #[test]
    fn status_hidden_when_closed_or_no_count() {
        let f = FindBar::open();
        assert!(f.build_status().is_none(), "无计数不显示");
        let closed = FindBar::new();
        assert!(closed.build_status().is_none(), "关闭不显示");
    }

    #[test]
    fn actions_gated_by_open_and_query() {
        let mut f = FindBar::open();
        // 打开但空查询 → 不发 find next/prev，但可关闭。
        assert!(f.on_find_next().is_none());
        assert_eq!(f.on_close(), Some(BrowserAction::FindClose));
        f.query = "x".into();
        assert_eq!(f.on_find_next(), Some(BrowserAction::FindNext));
        assert_eq!(f.on_find_prev(), Some(BrowserAction::FindPrev));
        // 关闭后不发任何 action。
        f.open = false;
        assert!(f.on_find_next().is_none());
        assert!(f.on_close().is_none());
    }
}
