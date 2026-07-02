//! Badge — 小型状态/计数标签（spec FR-009）。

use zero_ui_core::theme::Color;

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

    /// tone → 背景色（M1 基线；M2 改为消费 theme semantic token）。
    pub fn color(self) -> Color {
        match self.tone {
            BadgeTone::Neutral => Color::rgb(0.5, 0.5, 0.5),
            BadgeTone::Info => Color::rgb(0.13, 0.58, 0.95),
            BadgeTone::Success => Color::rgb(0.18, 0.7, 0.33),
            BadgeTone::Warning => Color::rgb(0.95, 0.76, 0.2),
            BadgeTone::Error => Color::rgb(0.86, 0.21, 0.27),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_to_color() {
        assert_eq!(Badge::new("3", BadgeTone::Error).color(), Color::rgb(0.86, 0.21, 0.27));
        assert_ne!(
            Badge::new("new", BadgeTone::Info).color(),
            Badge::new("", BadgeTone::Neutral).color()
        );
    }
}
