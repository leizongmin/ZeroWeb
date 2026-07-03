//! Scene → 光栅后端抽象（spec TBD-2）。
//!
//! [`RenderBackend`] 是 `ui/render` 与具体光栅后端（render-foundation 的 GPU/CPU 后端在
//! M2 后续实现）之间的边界 trait：`ui/render` 通过它把扁平 [`Scene`] 派发给后端，
//! **不直接依赖** render-foundation（spec TBD-2），保持通用 UI 层与 wgpu/tiny-skia 解耦。
//!
//! 文本走两条路径（DC-11）：
//! - 预 shape：调用方用 `zero-text-foundation` 的 `TextShaper`/`TextMeasurer` 产出
//!   [`TextBlob`](zero_text_foundation::TextBlob)，后端 [`RenderBackend::draw_text_blob`]
//!   直接光栅 glyph，不再 reshape（推荐路径，UI 与 WebView 共享同一 shaping）。
//! - 原始字符串：[`RenderBackend::draw_text`]，由后端自行 shape（简单场景/测试）。

use crate::render_node::RenderPrimitive;
use crate::scene::Scene;
use zero_text_foundation::TextBlob;
use zero_ui_core::geometry::{Point, Rect, Rounding};
use zero_ui_core::theme::Color;

/// Scene 光栅后端（TBD-2）。后端实现这些方法；`ui/render` 只负责把 Scene 派发。
pub trait RenderBackend {
    /// 填充矩形（含圆角）。
    fn fill_rect(&mut self, rect: Rect, color: Color, rounding: Rounding);
    /// 描边矩形。
    fn stroke_rect(&mut self, rect: Rect, color: Color, stroke_width: f32, rounding: Rounding);
    /// 预 shape 文本（DC-11）：直接光栅 `TextBlob` 的 glyph。
    fn draw_text_blob(&mut self, blob: &TextBlob, position: Point, color: Color);
    /// 原始字符串文本（后端自行 shape 或占位）。
    fn draw_text(&mut self, text: &str, position: Point, size_px: f32, color: Color);
    /// 外部合成表面（DC-3）：后端按 `surface_id` 取回 WebView/平台视图纹理并合成到 `rect`。
    fn draw_external_surface(&mut self, rect: Rect, surface_id: u64);
    /// 应用裁剪：后续绘制命令受 `clip` 约束，直到下一次 `apply_clip`。
    ///
    /// **`None` 语义**（深度审查 lei-deep-review 澄清）：表示「不主动裁剪」——后续绘制
    /// 只受后端自然边界（surface/framebuffer/视口）约束。`Some(rect)` 表示严格裁剪到 rect
    /// 内。注意：`Rect::intersect` 对无交集返回 `None`，故视口外节点经 host clip 链可能
    /// 产出 `None`；后端实现须确保 `None` 下不超出 surface 边界（如回落到视口 rect），
    /// 而非把 `None` 当作「无限大画布」导致越界像素泄漏。
    fn apply_clip(&mut self, clip: Option<Rect>);
}

/// 把扁平 [`Scene`] 派发给 [`RenderBackend`]（按 entries 顺序；每条 entry 先设 clip 再绘制）。
///
/// 这是 Scene → 后端的唯一入口；render-foundation 后端实现 `RenderBackend` 即可消费
/// UI SDK 产出的场景（TBD-2 闭环）。
pub fn paint_scene(scene: &Scene, backend: &mut dyn RenderBackend) {
    for entry in &scene.entries {
        backend.apply_clip(entry.clip);
        match &entry.primitive {
            RenderPrimitive::FillRect { rect, color, rounding } => {
                backend.fill_rect(*rect, *color, *rounding);
            }
            RenderPrimitive::StrokeRect {
                rect,
                color,
                stroke_width,
                rounding,
            } => {
                backend.stroke_rect(*rect, *color, *stroke_width, *rounding);
            }
            RenderPrimitive::Text {
                text,
                position,
                size_px,
                color,
            } => {
                backend.draw_text(text, *position, *size_px, *color);
            }
            RenderPrimitive::TextBlob { blob, position, color } => {
                backend.draw_text_blob(blob, *position, *color);
            }
            RenderPrimitive::ExternalSurface { rect, surface_id } => {
                backend.draw_external_surface(*rect, *surface_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Scene, SceneEntry};
    use zero_text_foundation::{FontRequest, FontdueBackend, ShapeInput, TextMeasurer, TextShaper};
    use zero_ui_core::geometry::{Point, Rect};
    use zero_ui_core::theme::Color;
    use zero_ui_core::widget::WidgetId;

    /// WPT 标准测试字体（跨平台确定性）。路径相对 ui/render crate 根。
    const AHEM: &[u8] = include_bytes!("../../../tests/wpt-runner/fonts/Ahem.ttf");

    /// 记录所有后端调用为字符串，便于断言 paint_scene 派发顺序与参数。
    #[derive(Default)]
    struct MockBackend {
        calls: Vec<String>,
        current_clip: Option<Rect>,
    }

    impl RenderBackend for MockBackend {
        fn fill_rect(&mut self, rect: Rect, color: Color, _rounding: Rounding) {
            self.calls.push(format!("fill {:?} {:?}", rect, color));
        }
        fn stroke_rect(&mut self, rect: Rect, color: Color, sw: f32, _rounding: Rounding) {
            self.calls.push(format!("stroke {:?} {:?} w{}", rect, color, sw));
        }
        fn draw_text_blob(&mut self, blob: &TextBlob, position: Point, color: Color) {
            self.calls.push(format!(
                "blob {}g @{},{} {:?}",
                blob.shaped.glyph_count(),
                position.x,
                position.y,
                color
            ));
        }
        fn draw_text(&mut self, text: &str, position: Point, _size_px: f32, color: Color) {
            self.calls
                .push(format!("text {text} @{},{} {:?}", position.x, position.y, color));
        }
        fn apply_clip(&mut self, clip: Option<Rect>) {
            self.current_clip = clip;
            self.calls.push(format!("clip {:?}", clip));
        }
        fn draw_external_surface(&mut self, rect: Rect, surface_id: u64) {
            self.calls.push(format!(
                "surface {surface_id} @{},{} {}x{}",
                rect.origin.x, rect.origin.y, rect.size.width, rect.size.height
            ));
        }
    }

    fn build_text_blob() -> zero_text_foundation::TextBlob {
        let mut b = FontdueBackend::new();
        b.load_family("Ahem", AHEM).unwrap();
        let req = FontRequest::new("Ahem");
        let shaped = b
            .shape(&ShapeInput {
                text: "Hi".into(),
                font_request: req.clone(),
                size_px: 16.0,
                direction: zero_text_foundation::TextDirection::Ltr,
                script: None,
                scale_factor: 1.0,
            })
            .unwrap();
        let metrics = b
            .measure(&zero_text_foundation::TextMeasureInput {
                text: "Hi".into(),
                font_request: req,
                size_px: 16.0,
                max_width: None,
                direction: zero_text_foundation::TextDirection::Ltr,
            })
            .unwrap();
        zero_text_foundation::TextBlob::new(shaped, metrics)
    }

    #[test]
    fn paint_scene_dispatches_all_primitive_kinds_in_order() {
        let blob = build_text_blob();
        let mut scene = Scene::new();
        scene.push(SceneEntry {
            source: WidgetId::new("card"),
            clip: None,
            primitive: RenderPrimitive::FillRect {
                rect: Rect::from_ltrb(0.0, 0.0, 10.0, 10.0),
                color: Color::BLACK,
                rounding: Rounding::ZERO,
            },
        });
        scene.push(SceneEntry {
            source: WidgetId::new("card"),
            clip: Some(Rect::from_ltrb(0.0, 0.0, 100.0, 100.0)),
            primitive: RenderPrimitive::TextBlob {
                blob,
                position: Point::new(2.0, 3.0),
                color: Color::WHITE,
            },
        });
        scene.push(SceneEntry {
            source: WidgetId::new("lbl"),
            clip: None,
            primitive: RenderPrimitive::Text {
                text: "raw".into(),
                position: Point::ZERO,
                size_px: 12.0,
                color: Color::WHITE,
            },
        });

        let mut backend = MockBackend::default();
        paint_scene(&scene, &mut backend);

        // 每条 entry 先 clip 再绘制。
        assert!(backend.calls[0].starts_with("clip None"));
        assert!(backend.calls[1].starts_with("fill"));
        assert!(backend.calls[2].starts_with("clip Some"));
        // TextBlob 来自真实 shaping：Ahem "Hi" → 2 glyph。
        assert!(backend.calls[3].starts_with("blob 2g @2,3"));
        assert!(backend.calls[4].starts_with("clip None"));
        assert!(backend.calls[5].starts_with("text raw @0,0"));
    }

    #[test]
    fn paint_scene_empty_scene_is_noop() {
        let mut backend = MockBackend::default();
        paint_scene(&Scene::new(), &mut backend);
        assert!(backend.calls.is_empty());
    }

    #[test]
    fn paint_scene_applies_clip_per_entry() {
        let mut scene = Scene::new();
        let clip = Rect::from_ltrb(1.0, 1.0, 2.0, 2.0);
        scene.push(SceneEntry {
            source: WidgetId::new("x"),
            clip: Some(clip),
            primitive: RenderPrimitive::FillRect {
                rect: Rect::ZERO,
                color: Color::WHITE,
                rounding: Rounding::ZERO,
            },
        });
        let mut backend = MockBackend::default();
        paint_scene(&scene, &mut backend);
        assert_eq!(backend.current_clip, Some(clip));
    }
}
