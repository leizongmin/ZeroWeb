//! Locale 标识与父级派生（spec IF-007）。

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// BCP-47 风格 locale 标签（如 `en-US`、`zh-Hans-CN`、`ar`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct LocaleId(pub CompactString);

impl LocaleId {
    pub fn new(tag: &str) -> LocaleId {
        LocaleId(CompactString::new(tag))
    }

    /// 是否为默认/根 locale（空或 `und`）。
    pub fn is_default(&self) -> bool {
        self.0.is_empty() || self.0.eq_ignore_ascii_case("und")
    }

    /// 派生父 locale：`en-US` → `en`；`zh-Hans-CN` → `zh-Hans` → `zh`。
    pub fn parent(&self) -> Option<LocaleId> {
        let s = self.0.as_str();
        // 先按 `-` 找最后一段裁剪。
        if let Some(idx) = s.rfind('-') {
            let parent = &s[..idx];
            if parent.is_empty() {
                None
            } else {
                Some(LocaleId::new(parent))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_chain() {
        let zh = LocaleId::new("zh-Hans-CN");
        assert_eq!(zh.parent(), Some(LocaleId::new("zh-Hans")));
        assert_eq!(LocaleId::new("zh-Hans").parent(), Some(LocaleId::new("zh")));
        assert_eq!(LocaleId::new("zh").parent(), None);
    }
}
