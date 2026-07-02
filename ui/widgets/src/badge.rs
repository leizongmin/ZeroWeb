//! Badge — 小型状态/计数标签（spec FR-009）。

use zero_ui_core::theme::{Color, SemanticTokens};

/// Badge 语义色调。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeTone {
    Neutral,
    Info,
    Success,
    Warning,
    Error,
}

/// Badge 控件数据。
#[derive(Debug, Clone)]
pub struct Badge {
    pub text: String,
    pub tone: BadgeTone,
}

impl Badge {
    pub fn new(text: &str, tone: BadgeTone) -> Badge {
        Badge {
            text: text.to_string(),
            tone,
        }
    }

    /// tone → 背景色（DC-5：消费 semantic token，不硬编码浏览器色值）。
    ///
    /// 每个 tone 映射到一个 token 对的**背景**（见 [`Self::text_color`] 取匹配前景），
    /// 故 badge 继承主题可访问性（token 对已由 contrast lint 保证 WCAG AA）。
    pub fn color(&self, tokens: &SemanticTokens) -> Color {
        match self.tone {
            BadgeTone::Neutral => tokens.surface,
            BadgeTone::Info => tokens.primary,
            BadgeTone::Success => tokens.success,
            BadgeTone::Warning => tokens.warning,
            BadgeTone::Error => tokens.error,
        }
    }

    /// tone → 与 [`Self::color`] 匹配的前景色（badge 文字色，DC-5 token 对）。
    pub fn text_color(&self, tokens: &SemanticTokens) -> Color {
        match self.tone {
            BadgeTone::Neutral => tokens.on_surface,
            BadgeTone::Info => tokens.on_primary,
            BadgeTone::Success => tokens.on_success,
            BadgeTone::Warning => tokens.on_warning,
            BadgeTone::Error => tokens.on_error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_to_color_consumes_tokens() {
        // DC-5：tone → 对应 semantic token 背景（非硬编码色值）。
        let t = SemanticTokens::light();
        assert_eq!(Badge::new("3", BadgeTone::Error).color(&t), t.error);
        assert_eq!(Badge::new("i", BadgeTone::Info).color(&t), t.primary);
        assert_ne!(
            Badge::new("new", BadgeTone::Info).color(&t),
            Badge::new("", BadgeTone::Neutral).color(&t),
        );
        // 前景与背景配对。
        assert_eq!(Badge::new("3", BadgeTone::Error).text_color(&t), t.on_error);
    }

    #[test]
    fn tone_pairs_pass_wcag_aa() {
        // DC-5 闭环：每个 tone 的 (text_color, color) 对 ≥ WCAG AA 4.5（light + dark）。
        // badge 消费 token 对后继承主题可访问性——contrast lint 保证 token 对 AA。
        use zero_ui_core::theme::{contrast_ratio, passes_wcag_aa};
        for tokens in [SemanticTokens::light(), SemanticTokens::dark()] {
            for tone in [
                BadgeTone::Neutral,
                BadgeTone::Info,
                BadgeTone::Success,
                BadgeTone::Warning,
                BadgeTone::Error,
            ] {
                let b = Badge::new("x", tone);
                let (fg, bg) = (b.text_color(&tokens), b.color(&tokens));
                assert!(
                    passes_wcag_aa(fg, bg, false),
                    "{tone:?} {:?} pair fg={fg:?} bg={bg:?} ratio {:.2} < 4.5",
                    tokens,
                    contrast_ratio(fg, bg)
                );
            }
        }
    }
}
