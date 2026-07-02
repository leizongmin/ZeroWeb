//! AddressBar / Omnibox — 地址栏（spec §8.4.1A）。
//!
//! 组合通用 [`TextInputState`]（URL/搜索输入）+ [`SuggestionList`]（搜索/历史建议）+
//! [`SecurityBadge`]（安全摘要）。输入由 TextInput/IME 处理；提交发出 navigate/search action；
//! 建议列表由浏览器模型提供（§8.4.1B omnibox 建议）。

use crate::browser_action::BrowserAction;
use crate::security_badge::{SecurityBadge, SecurityState};
use zero_ui_patterns::suggestion_list::{Suggestion, SuggestionList};
use zero_ui_widgets::text_input::TextInputState;

/// 地址栏（props）。
#[derive(Debug, Clone, PartialEq)]
pub struct AddressBar {
    pub text: String,
    pub security: SecurityState,
    pub suggestions: Vec<Suggestion>,
}

impl AddressBar {
    pub fn new(text: &str, security: SecurityState) -> AddressBar {
        AddressBar {
            text: text.to_string(),
            security,
            suggestions: Vec::new(),
        }
    }

    pub fn with_suggestions(mut self, suggestions: Vec<Suggestion>) -> AddressBar {
        self.suggestions = suggestions;
        self
    }

    /// 同步地址到 TextInput state。
    pub fn build_text_input(&self) -> TextInputState {
        let mut s = TextInputState::empty();
        s.insert(&self.text);
        s
    }

    /// 组合通用 SuggestionList（建议来自浏览器模型）。
    pub fn build_suggestions(&self) -> SuggestionList {
        SuggestionList::new(self.suggestions.clone())
    }

    /// 组合 SecurityBadge（地址栏左侧安全摘要）。
    pub fn build_security_badge(&self) -> SecurityBadge {
        SecurityBadge::new(self.security)
    }

    /// 提交地址栏：判定 URL vs 搜索。
    /// 极简启发式（M2）：含 `.` 或 `://` 视为 URL，否则搜索。真实判定由 browser-shell omnibox 完成。
    pub fn classify(&self) -> AddressSubmission {
        let t = self.text.trim();
        if t.contains("://") || (t.contains('.') && !t.contains(' ')) {
            AddressSubmission::Navigate(t.to_string())
        } else {
            AddressSubmission::Search(t.to_string())
        }
    }

    /// 提交 → BrowserAction。
    pub fn on_submit(&self) -> BrowserAction {
        match self.classify() {
            AddressSubmission::Navigate(url) => BrowserAction::Navigate(url),
            AddressSubmission::Search(q) => BrowserAction::Search(q),
        }
    }

    /// 选中建议 → Navigate（建议 label 作为目标 URL/查询）。
    pub fn on_suggestion_activated(&self, index: usize) -> Option<BrowserAction> {
        self.suggestions
            .get(index)
            .map(|s| BrowserAction::Navigate(s.label.clone()))
    }
}

/// 地址栏提交分类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressSubmission {
    Navigate(String),
    Search(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_input_and_security_composed() {
        let bar = AddressBar::new("https://example.com", SecurityState::Secure);
        assert_eq!(bar.build_text_input().text, "https://example.com");
        assert_eq!(bar.build_security_badge().state, SecurityState::Secure);
    }

    #[test]
    fn classify_url_vs_search() {
        let url = AddressBar::new("example.com", SecurityState::Insecure);
        assert_eq!(url.classify(), AddressSubmission::Navigate("example.com".into()));
        let full = AddressBar::new("https://example.com/path", SecurityState::Secure);
        assert!(matches!(full.classify(), AddressSubmission::Navigate(_)));
        // 含空格 → 搜索。
        let search = AddressBar::new("rust web framework", SecurityState::Insecure);
        assert_eq!(
            search.classify(),
            AddressSubmission::Search("rust web framework".into())
        );
        // 无点短词 → 搜索。
        let word = AddressBar::new("localhost-search", SecurityState::Insecure);
        assert!(matches!(word.classify(), AddressSubmission::Search(_)));
    }

    #[test]
    fn submit_emits_correct_action() {
        assert!(matches!(
            AddressBar::new("example.com", SecurityState::Secure).on_submit(),
            BrowserAction::Navigate(_)
        ));
        assert!(matches!(
            AddressBar::new("hello world", SecurityState::Insecure).on_submit(),
            BrowserAction::Search(_)
        ));
    }

    #[test]
    fn suggestion_activation_navigates() {
        let bar = AddressBar::new("ex", SecurityState::Secure).with_suggestions(vec![Suggestion {
            label: "https://example.com".into(),
            detail: None,
        }]);
        assert_eq!(
            bar.on_suggestion_activated(0),
            Some(BrowserAction::Navigate("https://example.com".into()))
        );
        assert!(bar.on_suggestion_activated(5).is_none());
        // SuggestionList carries the suggestions.
        assert_eq!(bar.build_suggestions().items.len(), 1);
    }
}
