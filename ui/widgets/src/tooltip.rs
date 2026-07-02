//! Tooltip — 悬浮提示（spec FR-009）。
//!
//! 悬停达延迟后显示；文案应通过 message id 引用（spec FR-013）。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tooltip {
    pub message_id: String,
    pub delay_ms: u32,
}

impl Tooltip {
    pub fn new(message_id: &str) -> Tooltip {
        Tooltip {
            message_id: message_id.to_string(),
            delay_ms: 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let t = Tooltip::new("browser.nav.back.tooltip");
        assert_eq!(t.delay_ms, 500);
    }
}
