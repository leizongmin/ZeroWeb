//! SearchField — 地址栏/搜索框组合模式（spec FR-009 / §8.4.1A）。
//!
//! 组合 TextInput + 触发建议；query 变化驱动 SuggestionList。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchField {
    /// 当前查询（业务文本由应用状态持有；此为组件工作快照）。
    pub query: String,
    /// 占位文案 message id。
    pub placeholder_msg: String,
}

impl SearchField {
    pub fn new(placeholder_msg: &str) -> SearchField {
        SearchField {
            query: String::new(),
            placeholder_msg: placeholder_msg.to_string(),
        }
    }
    pub fn set_query(&mut self, q: &str) -> bool {
        if self.query == q {
            false
        } else {
            self.query = q.to_string();
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_change_detected() {
        let mut f = SearchField::new("search.placeholder");
        assert!(f.set_query("zero"));
        assert!(!f.set_query("zero"));
    }
}
