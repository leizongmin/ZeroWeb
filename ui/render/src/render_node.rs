//! Render·Scene tree 节点与图元（spec §8.4.2 `RenderNode` / FR-004）。
//!
//! Render tree 负责 layout 后的几何、绘制、命中、裁剪、合成、a11y；按失效标记增量更新。

use serde::{Deserialize, Serialize};
use zero_ui_core::geometry::{Rect, Rounding, Vec2};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::WidgetId;

/// 单个绘制图元（spec FR 13 种 RenderPrimitives 的 M1 子集；M2 接 render-foundation 全集）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderPrimitive {
    /// 填充矩形（含圆角）。
    FillRect {
        rect: Rect,
        color: Color,
        rounding: Rounding,
    },
    /// 描边矩形。
    StrokeRect {
        rect: Rect,
        color: Color,
        stroke_width: f32,
        rounding: Rounding,
    },
    /// 文本（M1 承载字符串 + 位置；M2 由 foundation/text 产出 TextBlob/glyph 引用）。
    Text {
        text: String,
        position: zero_ui_core::geometry::Point,
        size_px: f32,
        color: Color,
    },
    /// 预 shape 文本（DC-11 phase 2：由 foundation/text 的 `TextShaper` + `TextMeasurer`
    /// 产出的 `TextBlob`，后端直接光栅 glyph，不再 reshape）。
    TextBlob {
        blob: zero_text_foundation::TextBlob,
        position: zero_ui_core::geometry::Point,
        color: Color,
    },
}

impl RenderPrimitive {
    /// 沿向量平移图元的几何（color/text 内容不变）。
    ///
    /// retained host 的 paint 遍历用：widget 以局部坐标（原点 = 节点左上角）paint，
    /// host 收集每节点 Scene 时按节点绝对 origin 平移后并入全局 Scene。
    pub fn translate(self, offset: Vec2) -> RenderPrimitive {
        match self {
            RenderPrimitive::FillRect { rect, color, rounding } => RenderPrimitive::FillRect {
                rect: rect.translate(offset.x, offset.y),
                color,
                rounding,
            },
            RenderPrimitive::StrokeRect {
                rect,
                color,
                stroke_width,
                rounding,
            } => RenderPrimitive::StrokeRect {
                rect: rect.translate(offset.x, offset.y),
                color,
                stroke_width,
                rounding,
            },
            RenderPrimitive::Text {
                text,
                position,
                size_px,
                color,
            } => RenderPrimitive::Text {
                text,
                position: position.translate(offset.x, offset.y),
                size_px,
                color,
            },
            RenderPrimitive::TextBlob { blob, position, color } => RenderPrimitive::TextBlob {
                blob,
                position: position.translate(offset.x, offset.y),
                color,
            },
        }
    }
}

/// Render tree 节点（spec §8.4.2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderNode {
    pub id: WidgetId,
    pub rect: Rect,
    pub clip: Option<Rect>,
    pub primitives: Vec<RenderPrimitive>,
    pub children: Vec<RenderNode>,
}

impl RenderNode {
    pub fn new(id: WidgetId, rect: Rect) -> RenderNode {
        RenderNode {
            id,
            rect,
            clip: None,
            primitives: Vec::new(),
            children: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_text_foundation::{ShapedText, TextBlob, TextMetrics};
    use zero_ui_core::geometry::Point;

    fn blob() -> TextBlob {
        TextBlob::new(
            ShapedText {
                runs: Vec::new(),
                total_advance_x: 0.0,
                total_advance_y: 0.0,
            },
            TextMetrics {
                width: 0.0,
                height: 0.0,
                ascent: 0.0,
                descent: 0.0,
                line_count: 0,
            },
        )
    }

    #[test]
    fn translate_fill_and_stroke_rect_shift_rect_origin() {
        let fill = RenderPrimitive::FillRect {
            rect: Rect::from_ltrb(0.0, 0.0, 10.0, 10.0),
            color: Color::BLACK,
            rounding: Rounding::ZERO,
        };
        match fill.translate(Vec2::new(5.0, 7.0)) {
            RenderPrimitive::FillRect { rect, .. } => {
                assert_eq!(rect, Rect::from_ltrb(5.0, 7.0, 15.0, 17.0));
            }
            _ => panic!("expected FillRect"),
        }

        let stroke = RenderPrimitive::StrokeRect {
            rect: Rect::from_ltrb(1.0, 2.0, 3.0, 4.0),
            color: Color::WHITE,
            stroke_width: 1.0,
            rounding: Rounding::ZERO,
        };
        match stroke.translate(Vec2::new(10.0, 20.0)) {
            RenderPrimitive::StrokeRect { rect, .. } => {
                assert_eq!(rect, Rect::from_ltrb(11.0, 22.0, 13.0, 24.0));
            }
            _ => panic!("expected StrokeRect"),
        }
    }

    #[test]
    fn translate_text_and_text_blob_shift_position() {
        let text = RenderPrimitive::Text {
            text: "hi".to_string(),
            position: Point::new(1.0, 2.0),
            size_px: 12.0,
            color: Color::BLACK,
        };
        match text.translate(Vec2::new(100.0, 50.0)) {
            RenderPrimitive::Text { position, text, size_px, .. } => {
                assert_eq!(position, Point::new(101.0, 52.0));
                assert_eq!(text, "hi");
                assert_eq!(size_px, 12.0);
            }
            _ => panic!("expected Text"),
        }

        let tb = RenderPrimitive::TextBlob {
            blob: blob(),
            position: Point::new(0.0, 0.0),
            color: Color::WHITE,
        };
        match tb.translate(Vec2::new(8.0, 9.0)) {
            RenderPrimitive::TextBlob { position, .. } => {
                assert_eq!(position, Point::new(8.0, 9.0));
            }
            _ => panic!("expected TextBlob"),
        }
    }
}
