//! 沙箱模块。
//!
//! 提供进程隔离原语和 iframe sandbox 属性强制执行功能。

use crate::origin::Origin;

/// iframe sandbox 标志，对应 HTML `sandbox` 属性支持的值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IframeSandboxFlag {
    /// 允许表单提交。
    AllowForms,
    /// 允许弹窗（window.open）。
    AllowPopups,
    /// 允许同源访问（不设置此项时 iframe 内容被视为不同源）。
    AllowSameOrigin,
    /// 允许运行脚本。
    AllowScripts,
    /// 允许导航顶层窗口。
    AllowTopNavigation,
    /// 允许通过用户激活进行顶层导航。
    AllowTopNavigationByUserActivation,
    /// 允许弹窗逃离沙箱限制。
    AllowPopupsToEscapeSandbox,
    /// 允许下载文件。
    AllowDownloads,
    /// 允许呈现模式。
    AllowPresentation,
    /// 允许锁定屏幕方向。
    AllowOrientationLock,
    /// 允许指针锁定。
    AllowPointerLock,
    /// 允许自动播放。
    AllowAutoplay,
    /// 允许模态窗口（alert/confirm/prompt）。
    AllowModals,
}

impl IframeSandboxFlag {
    /// 从 HTML sandbox 属性值字符串解析。
    pub fn parse_flag(s: &str) -> Option<Self> {
        match s {
            "allow-forms" => Some(Self::AllowForms),
            "allow-popups" => Some(Self::AllowPopups),
            "allow-same-origin" => Some(Self::AllowSameOrigin),
            "allow-scripts" => Some(Self::AllowScripts),
            "allow-top-navigation" => Some(Self::AllowTopNavigation),
            "allow-top-navigation-by-user-activation" => Some(Self::AllowTopNavigationByUserActivation),
            "allow-popups-to-escape-sandbox" => Some(Self::AllowPopupsToEscapeSandbox),
            "allow-downloads" => Some(Self::AllowDownloads),
            "allow-presentation" => Some(Self::AllowPresentation),
            "allow-orientation-lock" => Some(Self::AllowOrientationLock),
            "allow-pointer-lock" => Some(Self::AllowPointerLock),
            "allow-autoplay" => Some(Self::AllowAutoplay),
            "allow-modals" => Some(Self::AllowModals),
            _ => None,
        }
    }
}

/// iframe 沙箱策略。
///
/// 由 `<iframe sandbox="...">` 属性解析而来，决定 iframe 内内容的限制。
#[derive(Debug, Clone)]
pub struct IframeSandbox {
    /// 已启用的沙箱标志集合。空集合表示最严格限制。
    flags: Vec<IframeSandboxFlag>,
}

impl IframeSandbox {
    /// 从 sandbox 属性值字符串解析。
    ///
    /// 格式为空格分隔的标志列表，例如 `"allow-scripts allow-same-origin"`。
    pub fn parse(attribute_value: &str) -> Self {
        let flags = attribute_value
            .split_whitespace()
            .filter_map(IframeSandboxFlag::parse_flag)
            .collect();
        Self { flags }
    }

    /// 创建最严格的沙箱（无任何标志）。
    pub fn strict() -> Self {
        Self { flags: vec![] }
    }

    /// 检查是否包含指定标志。
    pub fn has_flag(&self, flag: IframeSandboxFlag) -> bool {
        self.flags.contains(&flag)
    }

    /// 是否允许执行脚本。
    pub fn allows_scripts(&self) -> bool {
        self.has_flag(IframeSandboxFlag::AllowScripts)
    }

    /// 是否允许表单提交。
    pub fn allows_forms(&self) -> bool {
        self.has_flag(IframeSandboxFlag::AllowForms)
    }

    /// 是否允许同源访问。
    pub fn allows_same_origin(&self) -> bool {
        self.has_flag(IframeSandboxFlag::AllowSameOrigin)
    }

    /// 是否允许弹窗。
    pub fn allows_popups(&self) -> bool {
        self.has_flag(IframeSandboxFlag::AllowPopups)
    }

    /// 是否允许顶层导航。
    pub fn allows_top_navigation(&self) -> bool {
        self.has_flag(IframeSandboxFlag::AllowTopNavigation)
    }

    /// 获取有效源。
    ///
    /// 当不设置 `allow-same-origin` 时，iframe 内容被赋予唯一的不透明源。
    /// 当设置 `allow-same-origin` 时，iframe 使用其自身的源。
    pub fn effective_origin(&self, iframe_origin: &Origin) -> SandboxOrigin {
        if self.allows_same_origin() {
            SandboxOrigin::Normal(iframe_origin.clone())
        } else {
            SandboxOrigin::Opaque
        }
    }
}

/// 沙箱中的有效源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxOrigin {
    /// 不透明源（每次创建沙箱时生成唯一源）。
    Opaque,
    /// 正常源（使用 iframe 自身源）。
    Normal(Origin),
}

/// 检查沙箱 iframe 是否可以导航其父页面。
///
/// `sandbox` 为 iframe 的沙箱策略。
/// `user_activated` 表示是否由用户激活触发（如点击）。
pub fn check_sandbox_navigation(sandbox: &IframeSandbox, user_activated: bool) -> bool {
    if sandbox.allows_top_navigation() {
        return true;
    }
    if user_activated && sandbox.has_flag(IframeSandboxFlag::AllowTopNavigationByUserActivation) {
        return true;
    }
    false
}

/// 检查沙箱 iframe 是否可以打开弹窗。
///
/// `sandbox` 为 iframe 的沙箱策略。
/// `popup_sandbox` 为弹窗的沙箱策略（如果弹窗也受沙箱约束）。
pub fn check_sandbox_popup(sandbox: &IframeSandbox) -> bool {
    sandbox.allows_popups()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iframe_sandbox_parse() {
        let sandbox = IframeSandbox::parse("allow-scripts allow-forms");
        assert!(sandbox.allows_scripts());
        assert!(sandbox.allows_forms());
        assert!(!sandbox.allows_same_origin());
    }

    #[test]
    fn test_iframe_sandbox_strict() {
        let sandbox = IframeSandbox::strict();
        assert!(!sandbox.allows_scripts());
        assert!(!sandbox.allows_forms());
        assert!(!sandbox.allows_same_origin());
        assert!(!sandbox.allows_popups());
        assert!(!sandbox.allows_top_navigation());
    }

    #[test]
    fn test_iframe_sandbox_parse_empty() {
        let sandbox = IframeSandbox::parse("");
        assert!(!sandbox.allows_scripts());
    }

    #[test]
    fn test_iframe_sandbox_parse_all_flags() {
        let sandbox = IframeSandbox::parse(
            "allow-forms allow-popups allow-same-origin allow-scripts allow-top-navigation allow-modals",
        );
        assert!(sandbox.allows_forms());
        assert!(sandbox.allows_popups());
        assert!(sandbox.allows_same_origin());
        assert!(sandbox.allows_scripts());
        assert!(sandbox.allows_top_navigation());
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowModals));
    }

    #[test]
    fn test_iframe_sandbox_unknown_flags_ignored() {
        let sandbox = IframeSandbox::parse("allow-scripts allow-unknown-flag");
        assert!(sandbox.allows_scripts());
        assert_eq!(sandbox.flags.len(), 1);
    }

    #[test]
    fn test_sandbox_effective_origin_opaque() {
        let sandbox = IframeSandbox::parse("allow-scripts");
        let iframe_origin = Origin::parse("https://example.com").unwrap();
        let effective = sandbox.effective_origin(&iframe_origin);
        assert_eq!(effective, SandboxOrigin::Opaque);
    }

    #[test]
    fn test_sandbox_effective_origin_normal() {
        let sandbox = IframeSandbox::parse("allow-scripts allow-same-origin");
        let iframe_origin = Origin::parse("https://example.com").unwrap();
        let effective = sandbox.effective_origin(&iframe_origin);
        assert_eq!(effective, SandboxOrigin::Normal(iframe_origin.clone()));
    }

    #[test]
    fn test_check_sandbox_navigation_blocked() {
        let sandbox = IframeSandbox::strict();
        assert!(!check_sandbox_navigation(&sandbox, false));
        assert!(!check_sandbox_navigation(&sandbox, true));
    }

    #[test]
    fn test_check_sandbox_navigation_top_nav() {
        let sandbox = IframeSandbox::parse("allow-top-navigation");
        assert!(check_sandbox_navigation(&sandbox, false));
    }

    #[test]
    fn test_check_sandbox_navigation_user_activation() {
        let sandbox = IframeSandbox::parse("allow-top-navigation-by-user-activation");
        assert!(!check_sandbox_navigation(&sandbox, false));
        assert!(check_sandbox_navigation(&sandbox, true));
    }

    #[test]
    fn test_check_sandbox_popup() {
        let sandbox = IframeSandbox::parse("allow-popups");
        assert!(check_sandbox_popup(&sandbox));
    }

    #[test]
    fn test_check_sandbox_popup_blocked() {
        let sandbox = IframeSandbox::strict();
        assert!(!check_sandbox_popup(&sandbox));
    }

    #[test]
    fn test_sandbox_flag_from_str_roundtrip() {
        let flags = [
            IframeSandboxFlag::AllowForms,
            IframeSandboxFlag::AllowPopups,
            IframeSandboxFlag::AllowSameOrigin,
            IframeSandboxFlag::AllowScripts,
            IframeSandboxFlag::AllowTopNavigation,
            IframeSandboxFlag::AllowTopNavigationByUserActivation,
            IframeSandboxFlag::AllowPopupsToEscapeSandbox,
            IframeSandboxFlag::AllowDownloads,
            IframeSandboxFlag::AllowPresentation,
            IframeSandboxFlag::AllowOrientationLock,
            IframeSandboxFlag::AllowPointerLock,
            IframeSandboxFlag::AllowAutoplay,
            IframeSandboxFlag::AllowModals,
        ];
        for flag in &flags {
            let name = match flag {
                IframeSandboxFlag::AllowForms => "allow-forms",
                IframeSandboxFlag::AllowPopups => "allow-popups",
                IframeSandboxFlag::AllowSameOrigin => "allow-same-origin",
                IframeSandboxFlag::AllowScripts => "allow-scripts",
                IframeSandboxFlag::AllowTopNavigation => "allow-top-navigation",
                IframeSandboxFlag::AllowTopNavigationByUserActivation => "allow-top-navigation-by-user-activation",
                IframeSandboxFlag::AllowPopupsToEscapeSandbox => "allow-popups-to-escape-sandbox",
                IframeSandboxFlag::AllowDownloads => "allow-downloads",
                IframeSandboxFlag::AllowPresentation => "allow-presentation",
                IframeSandboxFlag::AllowOrientationLock => "allow-orientation-lock",
                IframeSandboxFlag::AllowPointerLock => "allow-pointer-lock",
                IframeSandboxFlag::AllowAutoplay => "allow-autoplay",
                IframeSandboxFlag::AllowModals => "allow-modals",
            };
            assert_eq!(IframeSandboxFlag::parse_flag(name), Some(*flag));
        }
    }
}
