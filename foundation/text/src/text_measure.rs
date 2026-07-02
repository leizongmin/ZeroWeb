//! 文本测量（spec IF-008 `TextMeasurer`）。
//!
//! 用于 UI 控件（Button/TextInput/label）布局前的尺寸预估，以及 WebView inline 排版。
//! M1 定义接口与 `TextMetrics`；具体实现（基于 fontdue 度量 + 行高）在 M2。

use crate::diagnostics::TextError;
use crate::font_request::{FontRequest, TextDirection};
use serde::{Deserialize, Serialize};

/// 测量输入（spec IF-008）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextMeasureInput {
    pub text: String,
    pub font_request: FontRequest,
    pub size_px: f32,
    /// 最大宽度（启用自动换行时提供）。
    pub max_width: Option<f32>,
    pub direction: TextDirection,
}

/// 测量结果。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TextMetrics {
    /// 排版后的总逻辑宽度（单行或多行中最宽者）。
    pub width: f32,
    /// 总高度（含所有行）。
    pub height: f32,
    /// 基线相对顶部的 ascent。
    pub ascent: f32,
    /// 基线相对底部的 descent。
    pub descent: f32,
    /// 行数（无换行 = 1）。
    pub line_count: u32,
}

impl TextMetrics {
    pub fn line_height(self) -> f32 {
        self.ascent + self.descent
    }
}

/// 文本测量器（spec IF-008 `TextMeasurer`）。
pub trait TextMeasurer {
    fn measure(&self, input: &TextMeasureInput) -> Result<TextMetrics, TextError>;
}
