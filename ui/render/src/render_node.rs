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
    /// 外部合成表面（DC-3：WebView/平台视图/视频纹理等由后端独立光栅的表面）。
    ///
    /// UI SDK 只记录其外部矩形 + 宿主分配的 `surface_id`；真实纹理/primitives 由后端
    /// （持有 `zero-webview` 的合成层）按 id 取回合成。本图元不承载浏览器类型 → ui/render
    /// 不依赖 `zero-webview`（DC-1）。
    ExternalSurface { rect: Rect, surface_id: u64 },
    /// 预注册图像（如 SVG 图标）。`key` 引用宿主注册到桥接的图像（通常单通道 alpha 掩码），
    /// `tint` 为着色（典型 = 主题前景 token）。后端按 key 取回位图、按 tint 着色、缩放到
    /// `rect` 光栅（与 glyph 文本路径对称）。`ui/render` 只持 SDK 层 `ImageRef`，不依赖
    /// render-foundation（DC-1）。
    Image {
        rect: Rect,
        key: zero_ui_core::image::ImageRef,
        tint: Color,
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
            RenderPrimitive::ExternalSurface { rect, surface_id } => RenderPrimitive::ExternalSurface {
                rect: rect.translate(offset.x, offset.y),
                surface_id,
            },
            RenderPrimitive::Image { rect, key, tint } => RenderPrimitive::Image {
                rect: rect.translate(offset.x, offset.y),
                key,
                tint,
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

        let surface = RenderPrimitive::ExternalSurface {
            rect: Rect::from_ltrb(0.0, 0.0, 50.0, 50.0),
            surface_id: 7,
        };
        match surface.translate(Vec2::new(100.0, 200.0)) {
            RenderPrimitive::ExternalSurface { rect, surface_id } => {
                assert_eq!(rect, Rect::from_ltrb(100.0, 200.0, 150.0, 250.0));
                assert_eq!(surface_id, 7);
            }
            _ => panic!("expected ExternalSurface"),
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
            RenderPrimitive::Text {
                position,
                text,
                size_px,
                ..
            } => {
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

    #[test]
    fn translate_image_shifts_rect_preserves_key_and_tint() {
        let img = RenderPrimitive::Image {
            rect: Rect::from_ltrb(0.0, 0.0, 16.0, 16.0),
            key: zero_ui_core::image::ImageRef::new(3),
            tint: Color::BLACK,
        };
        match img.translate(Vec2::new(40.0, 12.0)) {
            RenderPrimitive::Image { rect, key, tint } => {
                assert_eq!(rect, Rect::from_ltrb(40.0, 12.0, 56.0, 28.0));
                assert_eq!(key, zero_ui_core::image::ImageRef::new(3));
                assert_eq!(tint, Color::BLACK);
            }
            other => panic!("expected Image, got {other:?}"),
        }
    }
}
