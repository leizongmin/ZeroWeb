//! StatusBubble — 状态气泡（spec FR-009；如 find 计数 "3/10"）。

use zero_ui_widgets::badge::BadgeTone;

#[derive(Debug, Clone, PartialEq)]
pub struct StatusBubble {
    pub text: String,
    pub tone: BadgeTone,
    pub visible: bool,
}

impl StatusBubble {
    pub fn new(text: &str, tone: BadgeTone) -> StatusBubble {
        StatusBubble {
            text: text.to_string(),
            tone,
            visible: true,
        }
    }
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_visibility() {
        let mut b = StatusBubble::new("3/10", BadgeTone::Neutral);
        assert!(b.visible);
        b.hide();
        assert!(!b.visible);
    }
}
