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
    /// 按方向在作用域内求下一个焦点 id（DC-8 phase-3）。
    ///
    /// - `trap = true`（modal/popup）：到达作用域边界时**折返**（wrap），焦点永不逃逸。
    /// - `trap = false`：到达边界时返回 `None`，表示请求逃逸到外层焦点遍历。
    /// - 作用域内无可聚焦项，或 `current` 不在作用域内（首次进入）：按方向落到首/末项。
    pub fn next(&self, current: Option<&WidgetId>, dir: FocusDirection) -> Option<&WidgetId> {
        if self.focusables.is_empty() {
            return None;
        }
        let idx = current.and_then(|c| self.focusables.iter().position(|f| f == c));
        let len = self.focusables.len();
        match dir {
            FocusDirection::Forward => {
                let next = idx.map(|i| i + 1);
                match next {
                    Some(n) if n < len => self.focusables.get(n),
                    // 越过末尾：trap 折返到首项，否则逃逸。
                    Some(_) if self.trap => self.focusables.first(),
                    Some(_) => None,
                    // 无当前焦点（首次进入）：首项。
                    None => self.focusables.first(),
                }
            }
            FocusDirection::Backward => {
                match idx {
                    Some(0) => {
                        // 越过开头：trap 折返到末项，否则逃逸。
                        if self.trap { self.focusables.last() } else { None }
                    }
                    Some(i) => self.focusables.get(i - 1),
                    // 无当前焦点（首次进入）：末项。
                    None => self.focusables.last(),
                }
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

    fn scope_non_trap() -> FocusScope {
        FocusScope {
            id: WidgetId::new("root"),
            focusables: vec![WidgetId::new("a"), WidgetId::new("b"), WidgetId::new("c")],
            trap: false,
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

    #[test]
    fn non_trap_forward_escapes_at_end() {
        // trap=false：越过末项 → None（逃逸）；中间项仍推进。
        let s = scope_non_trap();
        assert_eq!(s.next(Some(&WidgetId::new("c")), FocusDirection::Forward), None);
        assert_eq!(
            s.next(Some(&WidgetId::new("b")), FocusDirection::Forward),
            Some(&WidgetId::new("c"))
        );
    }

    #[test]
    fn non_trap_backward_escapes_at_start() {
        // trap=false：越过首项 → None（逃逸）。
        let s = scope_non_trap();
        assert_eq!(s.next(Some(&WidgetId::new("a")), FocusDirection::Backward), None);
    }

    #[test]
    fn first_entry_lands_on_edge() {
        // 首次进入（current 不在作用域）：Forward→首项，Backward→末项。
        let s = scope();
        assert_eq!(s.next(None, FocusDirection::Forward), Some(&WidgetId::new("a")));
        assert_eq!(s.next(None, FocusDirection::Backward), Some(&WidgetId::new("c")));
    }
}
