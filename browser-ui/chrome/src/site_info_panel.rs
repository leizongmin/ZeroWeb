//! SiteInfoPanel — 站点信息面板（spec §8.4.1A）。
//!
//! 组合通用 [`Popover`] + [`ListView`]（+ 权限项 Toggle/Button 由 shell 在列表项内组合）；
//! 展示权限、证书、站点设置；权限变更发出 grant/deny action。

use crate::browser_action::BrowserAction;
use zero_ui_core::geometry::Rect;
use zero_ui_widgets::list_view::ListView;
use zero_ui_widgets::popover::{Popover, PopoverPlacement};

/// 单个站点权限状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SitePermission {
    pub feature: String,
    pub granted: bool,
}

/// 站点信息面板（props）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteInfoPanel {
    pub origin: String,
    pub permissions: Vec<SitePermission>,
}

impl SiteInfoPanel {
    pub fn new(origin: &str, permissions: Vec<SitePermission>) -> SiteInfoPanel {
        SiteInfoPanel {
            origin: origin.to_string(),
            permissions,
        }
    }

    /// 组合通用 popover（锚定安全徽章下方）。
    pub fn build_popover(&self, anchor: Rect) -> Popover {
        Popover::new(anchor, PopoverPlacement::Below)
    }

    /// 权限列表（每项一个可切换权限；虚拟化走 ui/collections）。
    pub fn build_permission_list(&self) -> ListView {
        ListView::new(self.permissions.len())
    }

    /// 切换第 `idx` 个权限：当前 granted → Deny；当前 denied → Grant；越界 → None。
    pub fn on_toggle_permission(&self, idx: usize) -> Option<BrowserAction> {
        let p = self.permissions.get(idx)?;
        Some(if p.granted {
            BrowserAction::DenyPermission(p.feature.clone())
        } else {
            BrowserAction::GrantPermission(p.feature.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SiteInfoPanel {
        SiteInfoPanel::new(
            "https://example.com",
            vec![
                SitePermission {
                    feature: "geolocation".into(),
                    granted: true,
                },
                SitePermission {
                    feature: "camera".into(),
                    granted: false,
                },
            ],
        )
    }

    #[test]
    fn popover_and_list_built() {
        let p = sample();
        assert_eq!(p.build_popover(Rect::ZERO).placement, PopoverPlacement::Below);
        let lv = p.build_permission_list();
        assert_eq!(lv.item_count, 2);
    }

    #[test]
    fn toggle_flips_grant_deny() {
        let p = sample();
        // granted → deny
        assert_eq!(
            p.on_toggle_permission(0),
            Some(BrowserAction::DenyPermission("geolocation".into()))
        );
        // denied → grant
        assert_eq!(
            p.on_toggle_permission(1),
            Some(BrowserAction::GrantPermission("camera".into()))
        );
        assert!(p.on_toggle_permission(9).is_none(), "越界");
    }
}
