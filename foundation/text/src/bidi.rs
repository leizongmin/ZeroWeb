//! BiDi 基础类型（spec §8.4.1 `bidi.rs`）。
//!
//! 承载双向文本运行的层级与切分。完整 Unicode Bidirectional Algorithm（UAX #9）由
//! `unicode-bidi`（workspace 已声明）在 M2 接入；M1 定义数据模型与段落基向解析。

use crate::font_request::TextDirection;

/// BiDi 嵌入层级（偶数 = LTR，奇数 = RTL）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct BidiLevel(pub u8);

impl BidiLevel {
    /// 该 run 是否为 RTL 方向。
    pub fn is_rtl(self) -> bool {
        self.0 & 1 == 1
    }
}

/// 一段同向文本运行（字节范围 + 嵌入层级）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BidiRun {
    /// 起始字节偏移。
    pub start: usize,
    /// 结束字节偏移（exclusive）。
    pub end: usize,
    pub level: BidiLevel,
}

/// 根据段落方向确定基向层级（spec Auto 由首段强方向字符决定，M1 默认 LTR）。
pub fn paragraph_level(direction: TextDirection) -> BidiLevel {
    match direction {
        TextDirection::Rtl => BidiLevel(1),
        TextDirection::Ltr | TextDirection::Auto => BidiLevel(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraph_level_by_direction() {
        assert_eq!(paragraph_level(TextDirection::Ltr), BidiLevel(0));
        assert_eq!(paragraph_level(TextDirection::Rtl), BidiLevel(1));
        assert_eq!(paragraph_level(TextDirection::Auto), BidiLevel(0));
        assert!(paragraph_level(TextDirection::Rtl).is_rtl());
        assert!(!paragraph_level(TextDirection::Ltr).is_rtl());
    }
}
