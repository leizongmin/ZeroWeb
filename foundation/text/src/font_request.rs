//! 字体请求模型（spec IF-008 `FontRequest` 与支撑类型）。
//!
//! 调用方（UI render / WebView engine）把 CSS `font-*` 或 UI token 转换为 `FontRequest`，
//! 交给 [`crate::FontProvider`] 解析为具体字体。

use serde::{Deserialize, Serialize};

/// 稳定字体标识（解析后由 FontProvider 分配）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontId(pub u32);

/// 字体族名（如 `"Segoe UI"`、`"sans-serif"` 通用族）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontFamily(pub String);

impl FontFamily {
    pub fn new(name: &str) -> FontFamily {
        FontFamily(name.to_string())
    }
}

/// 字重（CSS 100..900）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FontWeight(pub u16);

impl FontWeight {
    pub const THIN: FontWeight = FontWeight(100);
    pub const NORMAL: FontWeight = FontWeight(400);
    pub const MEDIUM: FontWeight = FontWeight(500);
    pub const BOLD: FontWeight = FontWeight(700);
    pub const BLACK: FontWeight = FontWeight(900);

    /// 归一化到 CSS 离散字重档位（100 步进）。
    pub fn normalize(self) -> FontWeight {
        let step = ((self.0 + 50) / 100).clamp(1, 9) * 100;
        FontWeight(step)
    }
}

/// 字体样式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

/// 字体拉伸度（CSS `font-stretch`，百分比 50..200，100 = normal）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct FontStretch(pub u16);

impl FontStretch {
    pub const NORMAL: FontStretch = FontStretch(100);
    pub const CONDENSED: FontStretch = FontStretch(75);
    pub const EXPANDED: FontStretch = FontStretch(125);
}

/// 区域设置 id（M1 自带最小定义；M2 可与 `ui/i18n` 统一）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct LocaleId(pub String);

impl LocaleId {
    pub fn new(tag: &str) -> LocaleId {
        LocaleId(tag.to_string())
    }
}

/// Unicode 脚本（HarfBuzz `hb_script_t` 的 4 字节 tag，如 `b"latn"`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Script(pub u32);

impl Script {
    pub const fn from_tag(a: u8, b: u8, c: u8, d: u8) -> Script {
        Script(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | (d as u32))
    }
    pub const LATIN: Script = Script::from_tag(b'L', b'a', b't', b'n');
    pub const ARABIC: Script = Script::from_tag(b'A', b'r', b'a', b'b');
}

/// 文本方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextDirection {
    #[default]
    Ltr,
    Rtl,
    /// 由首段强方向字符决定（需 bidi 分析）。
    Auto,
}

/// 字体请求（spec IF-008）。`size_px` 在 shaping/measure 输入中提供。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontRequest {
    /// 候选族（按优先级；首个匹配者胜出，未匹配字符走 fallback chain）。
    pub families: Vec<FontFamily>,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
    pub locale: Option<LocaleId>,
}

impl FontRequest {
    pub fn new(family: &str) -> FontRequest {
        FontRequest {
            families: vec![FontFamily::new(family)],
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            stretch: FontStretch::NORMAL,
            locale: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_normalize() {
        assert_eq!(FontWeight(380).normalize(), FontWeight(400));
        assert_eq!(FontWeight(620).normalize(), FontWeight(600));
        assert_eq!(FontWeight(720).normalize(), FontWeight(700));
        assert_eq!(FontWeight(0).normalize(), FontWeight(100));
        assert_eq!(FontWeight(9999).normalize(), FontWeight(900));
    }

    #[test]
    fn request_builder() {
        let r = FontRequest::new("sans-serif");
        assert_eq!(r.families.len(), 1);
        assert_eq!(r.weight, FontWeight::NORMAL);
    }
}
