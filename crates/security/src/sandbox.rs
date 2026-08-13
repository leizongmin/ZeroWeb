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
    ///
    /// R3388：HTML 规范要求 sandbox 属性 token 按 **ASCII 大小写不敏感**匹配
    /// （HTML §4.7.4 sandbox 属性 token，reflecting 的 keyword matching）。旧实现用
    /// 精确 `match`，致 mixed-case token（如 `Allow-Scripts`）被静默丢弃 → 整个 sandbox
    /// 退化为最严格，丢失作者意图启用的能力。HTML 属性值保留大小写（仅标签/属性名
    /// 小写化，属性值为 CDATA 原样保留），故页面写 `sandbox="Allow-Same-Origin"` 会
    /// 真实到达。改 ASCII case-insensitive 比较。
    pub fn parse_flag(s: &str) -> Option<Self> {
        // 规范引用：https://html.spec.whatwg.org/#attr-iframe-sandbox
        if s.eq_ignore_ascii_case("allow-forms") {
            Some(Self::AllowForms)
        } else if s.eq_ignore_ascii_case("allow-popups") {
            Some(Self::AllowPopups)
        } else if s.eq_ignore_ascii_case("allow-same-origin") {
            Some(Self::AllowSameOrigin)
        } else if s.eq_ignore_ascii_case("allow-scripts") {
            Some(Self::AllowScripts)
        } else if s.eq_ignore_ascii_case("allow-top-navigation") {
            Some(Self::AllowTopNavigation)
        } else if s.eq_ignore_ascii_case("allow-top-navigation-by-user-activation") {
            Some(Self::AllowTopNavigationByUserActivation)
        } else if s.eq_ignore_ascii_case("allow-popups-to-escape-sandbox") {
            Some(Self::AllowPopupsToEscapeSandbox)
        } else if s.eq_ignore_ascii_case("allow-downloads") {
            Some(Self::AllowDownloads)
        } else if s.eq_ignore_ascii_case("allow-presentation") {
            Some(Self::AllowPresentation)
        } else if s.eq_ignore_ascii_case("allow-orientation-lock") {
            Some(Self::AllowOrientationLock)
        } else if s.eq_ignore_ascii_case("allow-pointer-lock") {
            Some(Self::AllowPointerLock)
        } else if s.eq_ignore_ascii_case("allow-autoplay") {
            Some(Self::AllowAutoplay)
        } else if s.eq_ignore_ascii_case("allow-modals") {
            Some(Self::AllowModals)
        } else {
            None
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

    // R3388 回归锁定：HTML 规范要求 sandbox 属性 token 按 ASCII 大小写不敏感匹配
    // （HTML §attribute sandbox tokens）。旧实现 parse_flag 用精确 match，致 mixed-case
    // token（如 "Allow-Scripts"）被静默丢弃 → 整个 sandbox 退化为最严格，丢失作者意图
    // 启用的能力（脚本/同源/表单等）。HTML 属性值保留大小写（仅标签/属性名小写化），
    // 故页面写 sandbox="Allow-Same-Origin" 会真实到达 parse()。
    #[test]
    fn test_iframe_sandbox_flag_parsing_case_insensitive_r3388() {
        // 全大写
        let sandbox = IframeSandbox::parse("ALLOW-SCRIPTS ALLOW-FORMS");
        assert!(sandbox.allows_scripts(), "全大写 token 须识别");
        assert!(sandbox.allows_forms());
        // 首字母大写（HTML 属性常见书写）
        let sandbox = IframeSandbox::parse("Allow-Same-Origin Allow-Top-Navigation");
        assert!(sandbox.allows_same_origin(), "首字母大写 token 须识别");
        assert!(sandbox.allows_top_navigation());
        // 混合大小写 + 长名
        let sandbox = IframeSandbox::parse("allow-Top-Navigation-By-User-Activation");
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowTopNavigationByUserActivation));
        // 已有小写形式不受影响（回归保护）
        assert!(IframeSandbox::parse("allow-scripts").allows_scripts());
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

    /// 测试当 allows_same_origin 为 false 时，effective_origin 返回不透明源。
    #[test]
    fn test_effective_origin_opaque_when_no_same_origin() {
        let sandbox = IframeSandbox::parse("allow-scripts allow-forms");
        let iframe_origin = Origin::parse("https://example.com").unwrap();
        assert!(!sandbox.allows_same_origin());
        let effective = sandbox.effective_origin(&iframe_origin);
        assert_eq!(effective, SandboxOrigin::Opaque);
    }

    /// 测试当 allows_same_origin 为 true 时，effective_origin 保留原始源。
    #[test]
    fn test_effective_origin_preserves_when_same_origin_allowed() {
        let sandbox = IframeSandbox::parse("allow-scripts allow-same-origin");
        let iframe_origin = Origin::parse("https://example.com").unwrap();
        assert!(sandbox.allows_same_origin());
        let effective = sandbox.effective_origin(&iframe_origin);
        assert_eq!(effective, SandboxOrigin::Normal(iframe_origin.clone()));
    }

    /// 测试带用户激活的导航 → 允许。
    #[test]
    fn test_check_sandbox_navigation_with_activation() {
        let sandbox = IframeSandbox::parse("allow-top-navigation-by-user-activation");
        assert!(check_sandbox_navigation(&sandbox, true));
    }

    /// 测试不带用户激活的导航 → 在仅有 user-activation 标志时应被阻止。
    #[test]
    fn test_check_sandbox_navigation_without_activation() {
        let sandbox = IframeSandbox::parse("allow-top-navigation-by-user-activation");
        assert!(!check_sandbox_navigation(&sandbox, false));
    }

    /// 测试带用户激活的弹窗 → 允许（只要有 allow-popups 标志）。
    #[test]
    fn test_check_sandbox_popup_with_activation() {
        let sandbox = IframeSandbox::parse("allow-popups");
        assert!(check_sandbox_popup(&sandbox));
    }

    /// 测试不带用户激活的弹窗 → 不影响弹窗权限（popup 由标志控制，与用户激活无关）。
    #[test]
    fn test_check_sandbox_popup_without_activation() {
        let sandbox = IframeSandbox::strict();
        assert!(!check_sandbox_popup(&sandbox));
    }

    /// 测试带有 allow-forms 标志的沙箱允许表单提交，
    /// 但仍阻止其他功能（脚本、弹窗、同源）。
    #[test]
    fn test_sandbox_allows_forms() {
        let sandbox = IframeSandbox::parse("allow-forms");
        assert!(sandbox.allows_forms(), "allow-forms 应允许表单提交");
        assert!(!sandbox.allows_scripts(), "仅有 allow-forms 不应允许脚本");
        assert!(!sandbox.allows_popups(), "仅有 allow-forms 不应允许弹窗");
        assert!(!sandbox.allows_same_origin(), "仅有 allow-forms 不应允许同源");
        assert!(!sandbox.allows_top_navigation(), "仅有 allow-forms 不应允许顶层导航");

        // 最严格沙箱（无标志）应阻止表单提交
        let strict = IframeSandbox::strict();
        assert!(!strict.allows_forms(), "严格沙箱应阻止表单提交");
    }

    // ── 边界测试 ──

    #[test]
    /// 测试重复标志解析不 panic。
    fn test_sandbox_duplicate_flags() {
        let sandbox = IframeSandbox::parse("allow-scripts allow-scripts");
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowScripts));
    }

    #[test]
    /// 测试少用标志的 has_flag。
    fn test_sandbox_rare_flags() {
        let sandbox = IframeSandbox::parse("allow-downloads allow-presentation allow-pointer-lock");
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowDownloads));
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowPresentation));
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowPointerLock));
    }

    #[test]
    /// 测试 SandboxOrigin 相等性。
    fn test_sandbox_origin_equality() {
        assert_eq!(SandboxOrigin::Opaque, SandboxOrigin::Opaque);
        let o = Origin::parse("https://example.com").unwrap();
        assert_eq!(SandboxOrigin::Normal(o.clone()), SandboxOrigin::Normal(o));
    }

    #[test]
    /// 测试同时包含两种导航标志的沙箱。
    fn test_sandbox_both_navigation_flags() {
        let sandbox = IframeSandbox::parse("allow-top-navigation allow-top-navigation-by-user-activation");
        assert!(sandbox.allows_top_navigation());
    }

    // ── 边界测试（round 23）──

    /// 测试沙箱 allow-same-origin + allow-scripts 的危险组合。
    ///
    /// 根据 HTML 规范，当 iframe sandbox 同时设置 allow-same-origin 和
    /// allow-scripts 时，嵌入页面可以通过 JavaScript 移除自身的 sandbox 属性，
    /// 这等于完全绕过了沙箱保护。这是已知的不安全组合，浏览器会发出警告。
    /// 此测试记录该行为：两个标志同时设置时均正常生效。
    #[test]
    fn test_sandbox_dangerous_same_origin_plus_scripts() {
        let sandbox = IframeSandbox::parse("allow-same-origin allow-scripts");

        // 两个标志均生效
        assert!(sandbox.allows_scripts(), "allow-scripts 应允许脚本");
        assert!(sandbox.allows_same_origin(), "allow-same-origin 应允许同源");
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowScripts));
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowSameOrigin));

        // effective_origin 保留原始源（非不透明源）
        let iframe_origin = Origin::parse("https://example.com").unwrap();
        assert_eq!(
            sandbox.effective_origin(&iframe_origin),
            SandboxOrigin::Normal(iframe_origin.clone()),
            "allow-same-origin 时应保留原始源"
        );

        // 其他功能仍受限
        assert!(!sandbox.allows_forms(), "不应允许表单提交");
        assert!(!sandbox.allows_popups(), "不应允许弹窗");
        assert!(!sandbox.allows_top_navigation(), "不应允许顶层导航");

        // 对比：仅 allow-scripts 无 allow-same-origin → 不透明源
        let sandbox_scripts_only = IframeSandbox::parse("allow-scripts");
        assert_eq!(
            sandbox_scripts_only.effective_origin(&iframe_origin),
            SandboxOrigin::Opaque,
            "仅 allow-scripts 时应为不透明源"
        );
    }

    /// 测试沙箱 allow-top-navigation 与 allow-top-navigation-by-user-activation 组合。
    ///
    /// 当两个导航标志同时存在时，allow-top-navigation 优先级更高，
    /// 无需用户激活即可导航顶层窗口。仅有 user-activation 标志时，
    /// 非用户激活的导航应被阻止。
    #[test]
    fn test_sandbox_navigation_flags_combination() {
        // 同时具有两个导航标志 → allow-top-navigation 优先
        let sandbox_both = IframeSandbox::parse("allow-top-navigation allow-top-navigation-by-user-activation");
        assert!(
            check_sandbox_navigation(&sandbox_both, false),
            "有 allow-top-navigation 时无需用户激活即可导航"
        );
        assert!(
            check_sandbox_navigation(&sandbox_both, true),
            "有 allow-top-navigation 时用户激活也可导航"
        );

        // 仅有 user-activation 标志 → 非激活时阻止
        let sandbox_user_only = IframeSandbox::parse("allow-top-navigation-by-user-activation");
        assert!(
            !check_sandbox_navigation(&sandbox_user_only, false),
            "仅有 user-activation 标志时非激活应阻止"
        );
        assert!(
            check_sandbox_navigation(&sandbox_user_only, true),
            "仅有 user-activation 标志时激活应允许"
        );

        // 无任何导航标志 → 始终阻止
        let sandbox_none = IframeSandbox::parse("allow-scripts allow-forms");
        assert!(!check_sandbox_navigation(&sandbox_none, false), "无导航标志时应阻止");
        assert!(
            !check_sandbox_navigation(&sandbox_none, true),
            "无导航标志时即使激活也应阻止"
        );
    }
}
