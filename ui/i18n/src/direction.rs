//! 文本方向（spec IF-007 `TextDirection`）与 locale → 方向推断（RTL 检测）。

use crate::locale::LocaleId;
use serde::{Deserialize, Serialize};

/// 文本方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextDirection {
    #[default]
    Ltr,
    Rtl,
    /// 由内容首段强方向字符决定（M1 = Ltr 兜底）。
    Auto,
}

/// 主要 RTL 语言前缀（ISO 639-1）。完整集合留 M2。
const RTL_LANG_PREFIXES: &[&str] = &["ar", "he", "iw", "fa", "ur", "yi", "ps", "sd"];

/// 推断 locale 的默认文本方向。
pub fn direction_for(locale: &LocaleId) -> TextDirection {
    let tag = locale.0.as_str();
    let lang = tag.split('-').next().unwrap_or("").to_ascii_lowercase();
    if RTL_LANG_PREFIXES.contains(&lang.as_str()) {
        TextDirection::Rtl
    } else {
        TextDirection::Ltr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtl_detection() {
        assert_eq!(direction_for(&LocaleId::new("ar")), TextDirection::Rtl);
        assert_eq!(direction_for(&LocaleId::new("ar-SA")), TextDirection::Rtl);
        assert_eq!(direction_for(&LocaleId::new("he-IL")), TextDirection::Rtl);
        assert_eq!(direction_for(&LocaleId::new("en-US")), TextDirection::Ltr);
        assert_eq!(direction_for(&LocaleId::new("zh-Hans-CN")), TextDirection::Ltr);
    }
}
