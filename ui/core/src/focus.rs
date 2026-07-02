//! 焦点模型（spec FR-011 / DC-8）。
//!
//! 可聚焦组件参与焦点遍历：Tab 按声明顺序或显式 `TraversalPolicy` 切换；支持焦点作用域（modal/popup）。

use crate::widget::WidgetId;
use serde::{Deserialize, Serialize};

/// 焦点遍历方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusDirection {
    Forward,
    Backward,
}

/// 遍历策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TraversalPolicy {
    /// 按声明顺序（默认）。
    #[default]
    DeclarationOrder,
    /// 显式 `tab_index` 顺序（数字升序，0 = 默认顺序，负值跳过）。
    Explicit,
}

/// 焦点作用域（用于 modal barrier / popover focus trap）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusScope {
    pub id: WidgetId,
    /// 作用域内可聚焦组件的声明顺序。
    pub focusables: Vec<WidgetId>,
    pub trap: bool,
}

impl FocusScope {
    /// 按方向在作用域内求下一个焦点 id。
    ///
    /// 返回 `None` 表示作用域内无可聚焦项或请求逃逸（非 trap 作用域）。
    pub fn next(&self, current: Option<&WidgetId>, dir: FocusDirection) -> Option<&WidgetId> {
        if self.focusables.is_empty() {
            return None;
        }
        let idx = current.and_then(|c| self.focusables.iter().position(|f| f == c));
        match dir {
            FocusDirection::Forward => {
                let next = idx.map(|i| i + 1).unwrap_or(0);
                self.focusables.get(next % self.focusables.len())
            }
            FocusDirection::Backward => {
                let len = self.focusables.len();
                let prev = idx.map(|i| (i + len - 1) % len).unwrap_or(len - 1);
                self.focusables.get(prev)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> FocusScope {
        FocusScope {
            id: WidgetId::new("root"),
            focusables: vec![WidgetId::new("a"), WidgetId::new("b"), WidgetId::new("c")],
            trap: true,
        }
    }

    #[test]
    fn forward_backward_wrap() {
        let s = scope();
        assert_eq!(s.next(None, FocusDirection::Forward), Some(&WidgetId::new("a")));
        assert_eq!(
            s.next(Some(&WidgetId::new("a")), FocusDirection::Forward),
            Some(&WidgetId::new("b"))
        );
        assert_eq!(
            s.next(Some(&WidgetId::new("c")), FocusDirection::Forward),
            Some(&WidgetId::new("a"))
        );
        assert_eq!(
            s.next(Some(&WidgetId::new("a")), FocusDirection::Backward),
            Some(&WidgetId::new("c"))
        );
    }
}
