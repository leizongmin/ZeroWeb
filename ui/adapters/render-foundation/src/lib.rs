//! # zero-ui-adapter-render-foundation
//!
//! render-foundation 光栅后端适配器（spec TBD-2 闭环）。
//!
//! [`RenderFoundationBackend`] 实现 [`zero_ui_render::RenderBackend`]，把通用 UI SDK 产出的
//! 扁平 [`Scene`](zero_ui_render::Scene) 累积为 render-foundation 的
//! [`RenderPrimitives`](zero_render_foundation::primitive::RenderPrimitives)，供现有 GPU/CPU
//! 渲染管线消费。`ui/render` 因此**不直接依赖** render-foundation（spec TBD-2），二者经本适配器耦合。
//!
//! 调用：`paint_scene(&scene, &mut backend)` → `backend.into_primitives()`。
//!
//! ## 当前覆盖（DC-14 视觉迁移的几何基础）
//! - `fill_rect`：圆角为零 → [`FillPrimitive`]；否则 → [`RoundedRectPrimitive`]（四角半径分别映射）。
//! - `stroke_rect`：矩形四角顶点 → [`PathStrokePrimitive`]（closed）。**圆角暂忽略**（render-foundation
//!   无 stroke 圆角矩形图元；TODO 跟踪）。
//! - `apply_clip`：`Some(rect)` → [`add_clip`]；`None` → 回落视口矩形（"无裁剪" = 整个视口）。
//!   render-foundation 经 `draw_order` 流式应用裁剪，与本适配器的流式 `apply_clip` 语义一致。
//!
//! ## 暂未覆盖（明确 follow-up，非阻塞当前几何闭环）
//! - `draw_text` / `draw_text_blob`：**no-op**。文本光栅需把 `zero-text-foundation::TextBlob`
//!   的 glyph 映射到 render-foundation `GlyphPrimitive`（不同 FontId/glyph cache），属 **DC-11
//!   字体栈统一**。在本适配器文本方法落地前，含文本的 Scene 不会出现文字——调用方须知。
//! - `draw_external_surface`：**no-op**。WebView/平台视图纹理合成属 **DC-3 phase-2**。
//!
//! [`FillPrimitive`]: zero_render_foundation::primitive::FillPrimitive
//! [`RoundedRectPrimitive`]: zero_render_foundation::primitive::RoundedRectPrimitive
//! [`PathStrokePrimitive`]: zero_render_foundation::primitive::PathStrokePrimitive
//! [`add_clip`]: zero_render_foundation::primitive::RenderPrimitives::add_clip

use zero_render_foundation::color::Color as RfColor;
use zero_render_foundation::geometry::{Point as RfPoint, Rect as RfRect, Size as RfSize};
use zero_render_foundation::primitive::{RenderPrimitives, RoundedRectPrimitive};
use zero_ui_core::geometry::{Rect, Rounding};
use zero_ui_core::theme::Color;
use zero_ui_render::RenderBackend;

/// render-foundation 后端：把 `RenderBackend` 调用累积为 [`RenderPrimitives`]。
///
/// 构造时传入 `viewport`（用于 `apply_clip(None)` 的"无裁剪"回落）。绘制完成后用
/// [`into_primitives`](Self::into_primitives) 取出累积结果交给 render-foundation 渲染。
pub struct RenderFoundationBackend {
    primitives: RenderPrimitives,
    viewport: RfRect,
}

impl RenderFoundationBackend {
    /// 创建后端，`viewport` 为目标帧的全区域（用于 clip=None 回落）。
    pub fn new(viewport: RfRect) -> Self {
        RenderFoundationBackend {
            primitives: RenderPrimitives::default(),
            viewport,
        }
    }

    /// 取出累积的 [`RenderPrimitives`]（消费后端）。
    pub fn into_primitives(self) -> RenderPrimitives {
        self.primitives
    }

    /// 只读访问累积结果（测试/调试用）。
    pub fn primitives(&self) -> &RenderPrimitives {
        &self.primitives
    }
}

impl RenderBackend for RenderFoundationBackend {
    fn fill_rect(&mut self, rect: Rect, color: Color, rounding: Rounding) {
        let rf_rect = to_rf_rect(rect);
        let rf_color = to_rf_color(color);
        if rounding.top_left == 0.0
            && rounding.top_right == 0.0
            && rounding.bottom_right == 0.0
            && rounding.bottom_left == 0.0
        {
            self.primitives.add_fill(rf_rect, rf_color);
        } else {
            self.primitives.add_rounded_rect(RoundedRectPrimitive {
                rect: rf_rect,
                color: rf_color,
                top_left_radius: rounding.top_left,
                top_right_radius: rounding.top_right,
                bottom_right_radius: rounding.bottom_right,
                bottom_left_radius: rounding.bottom_left,
            });
        }
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, stroke_width: f32, _rounding: Rounding) {
        // render-foundation 无 stroke 圆角矩形图元；用 4 角顶点的闭合路径描边。
        // 圆角 `_rounding` 暂忽略（TODO：圆角描边需路径曲线，跟踪）。
        let (x1, y1) = (rect.origin.x, rect.origin.y);
        let (x2, y2) = (rect.origin.x + rect.size.width, rect.origin.y + rect.size.height);
        // 闭合矩形：左上 → 右上 → 右下 → 左下（renderer closed=true 连回左上）。
        let vertices = vec![x1, y1, x2, y1, x2, y2, x1, y2];
        self.primitives
            .add_path_stroke(vertices, to_rf_color(color), stroke_width, true);
    }

    fn draw_text_blob(
        &mut self,
        _blob: &zero_text_foundation::TextBlob,
        _position: zero_ui_core::geometry::Point,
        _color: Color,
    ) {
        // TODO(DC-11)：TextBlob → GlyphPrimitive 映射需字体栈统一（不同 FontId/glyph cache）。当前 no-op。
    }

    fn draw_text(&mut self, _text: &str, _position: zero_ui_core::geometry::Point, _size_px: f32, _color: Color) {
        // TODO(DC-11)：原始字符串文本需后端 shape；当前 no-op（等字体栈统一）。
    }

    fn draw_external_surface(&mut self, _rect: Rect, _surface_id: u64) {
        // TODO(DC-3 phase-2)：WebView/平台视图纹理合成（按 surface_id 取回）。
    }

    fn apply_clip(&mut self, clip: Option<Rect>) {
        // Some → 裁剪到该矩形；None → 回落视口（"无裁剪" = 整个视口）。
        // render-foundation 经 draw_order 流式应用裁剪，与本适配器语义一致。
        let rf_rect = match clip {
            Some(r) => to_rf_rect(r),
            None => self.viewport,
        };
        self.primitives.add_clip(rf_rect);
    }
}

// ── 类型转换（ui/core → render-foundation）─────────────────────────────

/// ui/core Color（f32 0.0..=1.0）→ render-foundation Color（u8 0..=255）。
fn to_rf_color(c: Color) -> RfColor {
    RfColor {
        r: ch(c.r),
        g: ch(c.g),
        b: ch(c.b),
        a: ch(c.a),
    }
}

fn ch(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// ui/core Rect → render-foundation Rect。
fn to_rf_rect(r: Rect) -> RfRect {
    RfRect {
        origin: RfPoint {
            x: r.origin.x,
            y: r.origin.y,
        },
        size: RfSize {
            width: r.size.width,
            height: r.size.height,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::{Point, Rect, Rounding};
    use zero_ui_core::theme::Color;
    use zero_ui_core::widget::WidgetId;
    use zero_ui_render::scene::{Scene, SceneEntry};
    use zero_ui_render::{RenderPrimitive, paint_scene};

    fn viewport() -> RfRect {
        RfRect {
            origin: RfPoint { x: 0.0, y: 0.0 },
            size: RfSize {
                width: 800.0,
                height: 600.0,
            },
        }
    }

    fn entry(clip: Option<Rect>, prim: RenderPrimitive) -> SceneEntry {
        SceneEntry {
            source: WidgetId::new("t"),
            clip,
            primitive: prim,
        }
    }

    #[test]
    fn fill_rect_plain_uses_fill_no_rounding() {
        let mut b = RenderFoundationBackend::new(viewport());
        b.fill_rect(Rect::from_ltrb(0.0, 0.0, 10.0, 20.0), Color::WHITE, Rounding::ZERO);
        let p = b.into_primitives();
        assert_eq!(p.fills.len(), 1);
        assert!(p.rounded_rects.is_empty());
        // f32 0-1 → u8：WHITE(1,1,1,1) → (255,255,255,255)。
        assert_eq!(
            p.fills[0].color,
            RfColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
        assert_eq!(p.fills[0].rect.origin.x, 0.0);
        assert_eq!(p.fills[0].rect.size.width, 10.0);
        assert_eq!(p.fills[0].rect.size.height, 20.0);
    }

    #[test]
    fn fill_rect_rounded_maps_radii() {
        let mut b = RenderFoundationBackend::new(viewport());
        b.fill_rect(
            Rect::from_ltrb(0.0, 0.0, 10.0, 10.0),
            Color::BLACK,
            Rounding {
                top_left: 2.0,
                top_right: 4.0,
                bottom_right: 6.0,
                bottom_left: 8.0,
            },
        );
        let p = b.into_primitives();
        assert!(p.fills.is_empty());
        assert_eq!(p.rounded_rects.len(), 1);
        let r = &p.rounded_rects[0];
        assert_eq!(r.top_left_radius, 2.0);
        assert_eq!(r.top_right_radius, 4.0);
        assert_eq!(r.bottom_right_radius, 6.0);
        assert_eq!(r.bottom_left_radius, 8.0);
    }

    #[test]
    fn stroke_rect_emits_closed_path_of_four_corners() {
        let mut b = RenderFoundationBackend::new(viewport());
        b.stroke_rect(Rect::from_ltrb(1.0, 2.0, 11.0, 22.0), Color::WHITE, 1.5, Rounding::ZERO);
        let p = b.into_primitives();
        assert_eq!(p.path_strokes.len(), 1);
        let s = &p.path_strokes[0];
        assert!(s.closed);
        assert_eq!(s.line_width, 1.5);
        // 4 角顶点（8 个 f32）。
        assert_eq!(s.vertices, vec![1.0, 2.0, 11.0, 2.0, 11.0, 22.0, 1.0, 22.0]);
    }

    #[test]
    fn apply_clip_some_and_none() {
        let mut b = RenderFoundationBackend::new(viewport());
        b.apply_clip(Some(Rect::from_ltrb(5.0, 5.0, 50.0, 50.0)));
        b.apply_clip(None); // 回落视口
        let p = b.primitives();
        assert_eq!(p.clips.len(), 2);
        // 第一条 = 显式裁剪；第二条 = 视口（无裁剪回落）。
        assert_eq!(p.clips[0].rect.origin.x, 5.0);
        assert_eq!(p.clips[0].rect.size.width, 45.0);
        assert_eq!(p.clips[1].rect.size.width, 800.0);
        assert_eq!(p.clips[1].rect.size.height, 600.0);
    }

    #[test]
    fn paint_scene_dispatches_geometry_into_buckets() {
        // Scene：plain fill（无裁剪）→ rounded fill（裁剪 c）→ stroke（无裁剪）。
        // paint_scene 对每条 entry 先 apply_clip 再绘制。
        let clip = Rect::from_ltrb(0.0, 0.0, 100.0, 100.0);
        let mut scene = Scene::new();
        scene.push(entry(
            None,
            RenderPrimitive::FillRect {
                rect: Rect::from_ltrb(0.0, 0.0, 10.0, 10.0),
                color: Color::WHITE,
                rounding: Rounding::ZERO,
            },
        ));
        scene.push(entry(
            Some(clip),
            RenderPrimitive::FillRect {
                rect: Rect::from_ltrb(0.0, 0.0, 5.0, 5.0),
                color: Color::BLACK,
                rounding: Rounding::all(3.0),
            },
        ));
        scene.push(entry(
            None,
            RenderPrimitive::StrokeRect {
                rect: Rect::from_ltrb(0.0, 0.0, 8.0, 8.0),
                color: Color::WHITE,
                stroke_width: 1.0,
                rounding: Rounding::ZERO,
            },
        ));

        let mut b = RenderFoundationBackend::new(viewport());
        paint_scene(&scene, &mut b);
        let p = b.into_primitives();

        assert_eq!(p.fills.len(), 1); // plain fill
        assert_eq!(p.rounded_rects.len(), 1); // rounded fill
        assert_eq!(p.rounded_rects[0].top_left_radius, 3.0);
        assert_eq!(p.path_strokes.len(), 1); // stroke
        // 裁剪：每条 entry 一个 apply_clip → clips.len() == 3（None→viewport, Some→clip, None→viewport）。
        assert_eq!(p.clips.len(), 3);
        assert_eq!(p.clips[1].rect.size.width, 100.0); // 第二条 = 显式 clip
        // draw_order 按插入顺序记录 clip/fill/rounded_rect/path_stroke 交错。
        assert!(p.draw_order.len() >= 6);
    }

    #[test]
    fn text_and_external_surface_are_documented_noops() {
        // 当前 follow-up（DC-11/DC-3 phase-2）：文本与外部表面 no-op，不产生图元。
        let mut b = RenderFoundationBackend::new(viewport());
        b.draw_text("hi", Point::ZERO, 12.0, Color::WHITE);
        b.draw_external_surface(Rect::from_ltrb(0.0, 0.0, 10.0, 10.0), 7);
        let p = b.into_primitives();
        assert!(p.glyphs.is_empty());
        assert!(p.images.is_empty());
        assert!(p.fills.is_empty());
    }
}
