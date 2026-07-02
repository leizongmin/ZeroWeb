//! IconButton — 图标按钮（spec FR-009 首批 widget）。
//!
//! 与 [`crate::Button`] 同语义（点击只发 action），但以图标标识而非文本标签；
//! 可选 tooltip（悬停展示）。导航按钮、菜单触发、地址栏动作等用。

use zero_ui_core::action::ActionId;

/// 图标按钮声明（props）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconButton {
    /// 图标标识（semantic icon name，如 `nav-back`、`menu`；由绘制层映射到资源）。
    pub icon: String,
    pub action: ActionId,
    pub enabled: bool,
    /// 可选 tooltip message id。
    pub tooltip: Option<String>,
}

impl IconButton {
    pub fn new(icon: &str, action: &str) -> IconButton {
        IconButton {
            icon: icon.to_string(),
            action: ActionId::new(action),
            enabled: true,
            tooltip: None,
        }
    }

    pub fn with_tooltip(mut self, tooltip_msg: &str) -> IconButton {
        self.tooltip = Some(tooltip_msg.to_string());
        self
    }

    pub fn disabled(mut self) -> IconButton {
        self.enabled = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_and_decorate() {
        let b = IconButton::new("nav-back", "browser.go_back")
            .with_tooltip("nav.back.tooltip")
            .disabled();
        assert_eq!(b.icon, "nav-back");
        assert!(!b.enabled);
        assert_eq!(b.tooltip.as_deref(), Some("nav.back.tooltip"));
    }

    #[test]
    fn default_enabled() {
        let b = IconButton::new("menu", "app.menu");
        assert!(b.enabled);
        assert!(b.tooltip.is_none());
    }
}
