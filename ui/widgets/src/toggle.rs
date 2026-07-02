//! Toggle — 开关控件（spec FR-009；权限/设置项等用）。
//!
//! 双态（on/off）控件；状态翻转时发出 action（由应用层更新业务状态，spec FR-003 单向数据流）。
//! 控件本身不持有业务状态——`checked` 是从应用状态投影的 props。

use zero_ui_core::action::ActionId;

/// 开关声明（props）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toggle {
    pub checked: bool,
    pub action: ActionId,
    /// 可选标签（设置项文案；生产走 i18n message id）。
    pub label: Option<String>,
}

impl Toggle {
    pub fn new(checked: bool, action: &str) -> Toggle {
        Toggle {
            checked,
            action: ActionId::new(action),
            label: None,
        }
    }

    pub fn with_label(mut self, label: &str) -> Toggle {
        self.label = Some(label.to_string());
        self
    }

    /// 翻转状态并返回要派发的 action（单向数据流：应用接收 action 后回写 checked）。
    pub fn flip(&mut self) -> ActionId {
        self.checked = !self.checked;
        self.action.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_toggles_and_emits_action() {
        let mut t = Toggle::new(false, "perm.geolocation.toggle");
        let a1 = t.flip();
        assert!(t.checked);
        assert_eq!(a1, ActionId::new("perm.geolocation.toggle"));
        let _ = t.flip();
        assert!(!t.checked);
    }

    #[test]
    fn label_optional() {
        let t = Toggle::new(true, "x").with_label("settings.dark_mode");
        assert_eq!(t.label.as_deref(), Some("settings.dark_mode"));
    }
}
