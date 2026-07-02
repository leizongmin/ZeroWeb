//! 文本 blob — 一段已 shape + 测量的可绘制文本（spec §8.4.1 `text_blob.rs`）。
//!
//! `ui/render` 与 WebView 把 `TextBlob` 作为文本绘制单元；它持有 glyph runs（含 atlas 引用）
//! 与度量（用于光标/caret/hit-test）。

use crate::shaping::ShapedText;
use crate::text_measure::TextMetrics;
use serde::{Deserialize, Serialize};

/// 可绘制文本单元。
///
/// `ShapedText` 与 `TextMetrics` 均派生 `Serialize`，故 `TextBlob` 也可序列化——
/// 使 `ui/render::RenderPrimitive::TextBlob` 能纳入可序列化的 Scene（DC-11 phase 2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextBlob {
    pub shaped: ShapedText,
    pub metrics: TextMetrics,
}

impl TextBlob {
    pub fn new(shaped: ShapedText, metrics: TextMetrics) -> TextBlob {
        TextBlob { shaped, metrics }
    }

    /// 字节偏移 → x 坐标的简化 hit-test（M1：按 glyph 簇线性映射）。
    ///
    /// 返回该字节偏移对应的 caret x（相对 blob 左边缘）。完整双向/cluster 映射在 M2。
    pub fn caret_x_for_byte(&self, byte_idx: usize) -> f32 {
        let mut x = 0.0f32;
        let mut consumed = 0usize;
        for run in &self.shaped.runs {
            for g in &run.glyphs {
                if consumed >= byte_idx {
                    return x;
                }
                x += g.x_advance;
                consumed = g.cluster as usize;
            }
        }
        x
    }
}
