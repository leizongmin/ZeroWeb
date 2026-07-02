//! PermissionPrompt — 权限提示（spec §8.4.1A）。
//!
//! 组合通用 [`DialogScaffold`]（desktop 也可用 anchored popover；M2 用 dialog）+
//! [`Toggle`]（remember）。geolocation/camera/mic/notification 等 Web 权限由
//! WebView/permission controller 产生请求；用户选择发出 grant/deny action。

use crate::browser_action::BrowserAction;
use zero_ui_patterns::dialog_scaffold::DialogScaffold;
use zero_ui_widgets::toggle::Toggle;

/// 权限请求（props）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionPrompt {
    /// 权限特性名（`geolocation` / `camera` / `microphone` / `notifications` / ...）。
    pub feature: String,
    pub origin: String,
}

impl PermissionPrompt {
    pub fn new(feature: &str, origin: &str) -> PermissionPrompt {
        PermissionPrompt {
            feature: feature.to_string(),
            origin: origin.to_string(),
        }
    }

    /// 组合通用 dialog（标题 message id = `perm.<feature>.title`）。
    pub fn build_dialog(&self) -> DialogScaffold {
        DialogScaffold::new(&format!("perm.{}.title", self.feature))
    }

    /// "记住选择" toggle（默认未勾选）。
    pub fn build_remember_toggle(&self) -> Toggle {
        Toggle::new(false, &format!("perm.{}.remember", self.feature))
    }

    pub fn on_allow(&self) -> BrowserAction {
        BrowserAction::GrantPermission(self.feature.clone())
    }

    pub fn on_deny(&self) -> BrowserAction {
        BrowserAction::DenyPermission(self.feature.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_title_and_remember_toggle() {
        let p = PermissionPrompt::new("geolocation", "https://example.com");
        let d = p.build_dialog();
        assert_eq!(d.title_msg, "perm.geolocation.title");
        let t = p.build_remember_toggle();
        assert!(!t.checked);
        assert_eq!(t.label, None);
    }

    #[test]
    fn allow_deny_carry_feature() {
        let p = PermissionPrompt::new("camera", "https://meet.example.com");
        assert_eq!(p.on_allow(), BrowserAction::GrantPermission("camera".into()));
        assert_eq!(p.on_deny(), BrowserAction::DenyPermission("camera".into()));
    }
}
