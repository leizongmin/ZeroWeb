//! SecurityBadge — 站点安全标识（spec §8.4.1A）。
//!
//! 组合通用 [`Badge`] + [`Tooltip`]；只展示安全摘要（HTTPS / 证书 / mixed content / 危险站点）。
//! 点击打开 `SiteInfoPanel` 由 shell 在 overlay 层处理（非本组件 action）。生产文案走 i18n message id。

use zero_ui_widgets::badge::{Badge, BadgeTone};
use zero_ui_widgets::tooltip::Tooltip;

/// 站点安全状态（从 browser-shell navigation/security state 投影）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecurityState {
    /// HTTPS 有效。
    #[default]
    Secure,
    /// HTTP（无加密）。
    Insecure,
    /// HTTPS 页面含混合内容。
    Mixed,
    /// 危险站点（钓鱼/恶意，由 safe-browsing 标记）。
    Dangerous,
}

/// 站点安全标识（props）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityBadge {
    pub state: SecurityState,
}

impl SecurityBadge {
    pub fn new(state: SecurityState) -> SecurityBadge {
        SecurityBadge { state }
    }

    /// 摘要文案（短标签，供 Badge 文本显示）。
    pub fn summary_label(&self) -> &'static str {
        match self.state {
            SecurityState::Secure => "secure",
            SecurityState::Insecure => "not-secure",
            SecurityState::Mixed => "mixed-content",
            SecurityState::Dangerous => "dangerous",
        }
    }

    /// 安全状态 → i18n message id（tooltip 悬停展示完整安全摘要）。
    ///
    /// 返回 `crate::i18n::ids::SECURITY_*` 常量，经 catalog 解析为可见文案
    /// （如 "Connection is secure"）；生产取代原先 M2 字面量占位。
    pub fn tooltip_message_id(&self) -> &'static str {
        match self.state {
            SecurityState::Secure => crate::i18n::ids::SECURITY_SECURE,
            SecurityState::Insecure => crate::i18n::ids::SECURITY_INSECURE,
            SecurityState::Mixed => crate::i18n::ids::SECURITY_MIXED,
            SecurityState::Dangerous => crate::i18n::ids::SECURITY_DANGEROUS,
        }
    }

    /// 安全状态 → semantic tone（组件只消费 semantic token，不硬编码浏览器色值）。
    pub fn tone(&self) -> BadgeTone {
        match self.state {
            SecurityState::Secure => BadgeTone::Success,
            SecurityState::Insecure => BadgeTone::Warning,
            SecurityState::Mixed => BadgeTone::Warning,
            SecurityState::Dangerous => BadgeTone::Error,
        }
    }

    /// 组合通用 Badge（绘制走 ui/widgets）。
    pub fn build_badge(&self) -> Badge {
        Badge::new(self.summary_label(), self.tone())
    }

    /// 组合通用 Tooltip（悬停展示完整安全摘要，文案经 i18n catalog 解析）。
    pub fn build_tooltip(&self) -> Tooltip {
        Tooltip::new(self.tooltip_message_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_state_maps_to_label_and_tone() {
        let cases = [
            (SecurityState::Secure, "secure", BadgeTone::Success),
            (SecurityState::Insecure, "not-secure", BadgeTone::Warning),
            (SecurityState::Mixed, "mixed-content", BadgeTone::Warning),
            (SecurityState::Dangerous, "dangerous", BadgeTone::Error),
        ];
        for (state, label, tone) in cases {
            let b = SecurityBadge::new(state);
            assert_eq!(b.summary_label(), label);
            assert_eq!(b.tone(), tone);
        }
    }

    #[test]
    fn badge_and_tooltip_carry_label() {
        let b = SecurityBadge::new(SecurityState::Secure);
        assert_eq!(b.build_badge().text, "secure");
        // DC-10：tooltip message_id 使用 i18n catalog id，非 M2 字面量占位。
        assert_eq!(b.build_tooltip().message_id, crate::i18n::ids::SECURITY_SECURE);
        assert_eq!(b.tooltip_message_id(), crate::i18n::ids::SECURITY_SECURE);
    }

    #[test]
    fn tooltip_message_ids_cover_all_states() {
        // 每个安全状态都有对应的 tooltip i18n message id。
        for state in [
            SecurityState::Secure,
            SecurityState::Insecure,
            SecurityState::Mixed,
            SecurityState::Dangerous,
        ] {
            let b = SecurityBadge::new(state);
            let id = b.tooltip_message_id();
            assert!(!id.is_empty(), "{state:?} 缺少 tooltip message id");
            assert!(
                id.starts_with("browser.security."),
                "{state:?} id 应为 browser.security.* 前缀: {id}"
            );
        }
    }
}
