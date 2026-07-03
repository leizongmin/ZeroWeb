//! Scene snapshot — 把 Scene 拍平为确定性字符串（spec FR-016 testing）。

use zero_ui_core::theme::Color;
use zero_ui_render::{RenderPrimitive, Scene};

/// 生成场景的确定性快照（按 entries 顺序；坐标保留 2 位小数）。
///
/// 用于 CI golden test：相同 widget 树 + 相同状态 → 相同字符串。
pub fn snapshot_scene(scene: &Scene) -> String {
    let mut out = String::new();
    for (i, e) in scene.entries.iter().enumerate() {
        out.push_str(&format!("{}|src={}", i, e.source.0.as_str()));
        if let Some(clip) = e.clip {
            out.push_str(&format!(
                "|clip={},{},{},{}",
                clip.left(),
                clip.top(),
                clip.right(),
                clip.bottom()
            ));
        }
        out.push('|');
        match &e.primitive {
            RenderPrimitive::FillRect { rect, color, .. } => {
                out.push_str(&format!(
                    "fill {},{},{},{} {}",
                    rect.left(),
                    rect.top(),
                    rect.right(),
                    rect.bottom(),
                    fmt_color(*color)
                ));
            }
            RenderPrimitive::StrokeRect {
                rect,
                color,
                stroke_width,
                ..
            } => {
                out.push_str(&format!(
                    "stroke {},{},{},{} w{} {}",
                    rect.left(),
                    rect.top(),
                    rect.right(),
                    rect.bottom(),
                    stroke_width,
                    fmt_color(*color)
                ));
            }
            RenderPrimitive::Text {
                text,
                position,
                size_px,
                color,
            } => {
                out.push_str(&format!(
                    "text {} @{},{} sz{} {}",
                    text,
                    position.x,
                    position.y,
                    size_px,
                    fmt_color(*color)
                ));
            }
            RenderPrimitive::TextBlob { blob, position, color } => {
                // 预 shape 文本（DC-11）：按 glyph 数 + 总前进量做确定性摘要。
                out.push_str(&format!(
                    "textblob {}g @{},{} adv{:.1} {}",
                    blob.shaped.glyph_count(),
                    position.x,
                    position.y,
                    blob.shaped.total_advance_x,
                    fmt_color(*color)
                ));
            }
            RenderPrimitive::ExternalSurface { rect, surface_id } => {
                // 外部表面（DC-3 WebView）：确定性摘要（rect + surface_id）。
                out.push_str(&format!(
                    "surface{} {},{},{},{}",
                    surface_id,
                    rect.left(),
                    rect.top(),
                    rect.right(),
                    rect.bottom()
                ));
            }
            RenderPrimitive::Image { rect, key, tint } => {
                // 预注册图像（如 SVG 图标）：确定性摘要（ref + rect + tint）。
                out.push_str(&format!(
                    "image{} {},{},{},{} {}",
                    key.0,
                    rect.left(),
                    rect.top(),
                    rect.right(),
                    rect.bottom(),
                    fmt_color(*tint)
                ));
            }
        }
        out.push('\n');
    }
    out
}

fn fmt_color(c: Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        to_u8(c.r),
        to_u8(c.g),
        to_u8(c.b),
        to_u8(c.a)
    )
}

fn to_u8(f: f32) -> u8 {
    (f.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::{Point, Rect};
    use zero_ui_core::widget::WidgetId;
    use zero_ui_render::{Scene, SceneEntry};

    #[test]
    fn snapshot_is_deterministic() {
        let mut s1 = Scene::new();
        s1.push(SceneEntry {
            source: WidgetId::new("btn"),
            clip: None,
            primitive: RenderPrimitive::FillRect {
                rect: Rect::from_ltrb(0.0, 0.0, 96.0, 32.0),
                color: Color::BLACK,
                rounding: zero_ui_core::geometry::Rounding::ZERO,
            },
        });
        let snap = snapshot_scene(&s1);
        assert!(snap.contains("fill 0,0,96,32 #000000ff"), "got: {snap}");
        // 同输入 → 同输出。
        assert_eq!(snapshot_scene(&s1), snap);
    }

    #[test]
    fn snapshot_includes_text() {
        let mut s = Scene::new();
        s.push(SceneEntry {
            source: WidgetId::new("lbl"),
            clip: None,
            primitive: RenderPrimitive::Text {
                text: "Hi".into(),
                position: Point::new(1.0, 2.0),
                size_px: 14.0,
                color: Color::WHITE,
            },
        });
        let snap = snapshot_scene(&s);
        assert!(snap.contains("text Hi @1,2"), "got: {snap}");
    }

    #[test]
    fn snapshot_includes_stroke_rect() {
        let mut s = Scene::new();
        s.push(SceneEntry {
            source: WidgetId::new("frame"),
            clip: None,
            primitive: RenderPrimitive::StrokeRect {
                rect: Rect::from_ltrb(0.0, 0.0, 10.0, 20.0),
                color: Color::WHITE,
                stroke_width: 2.0,
                rounding: zero_ui_core::geometry::Rounding::ZERO,
            },
        });
        let snap = snapshot_scene(&s);
        assert!(snap.contains("stroke 0,0,10,20 w2 #ffffffff"), "got: {snap}");
    }

    #[test]
    fn snapshot_includes_clip() {
        let mut s = Scene::new();
        s.push(SceneEntry {
            source: WidgetId::new("clipped"),
            clip: Some(Rect::from_ltrb(1.0, 2.0, 3.0, 4.0)),
            primitive: RenderPrimitive::FillRect {
                rect: Rect::from_ltrb(0.0, 0.0, 50.0, 50.0),
                color: Color::BLACK,
                rounding: zero_ui_core::geometry::Rounding::ZERO,
            },
        });
        let snap = snapshot_scene(&s);
        assert!(snap.contains("|clip=1,2,3,4|"), "got: {snap}");
        // 第二条目索引递增。
        assert!(snap.starts_with("0|src=clipped"), "got: {snap}");
    }

    #[test]
    fn snapshot_includes_text_blob() {
        use zero_text_foundation::shaping::ShapedText;
        use zero_text_foundation::text_blob::TextBlob;
        use zero_text_foundation::text_measure::TextMetrics;

        let mut s = Scene::new();
        let shaped = ShapedText {
            runs: vec![],
            total_advance_x: 42.0,
            total_advance_y: 0.0,
        };
        let metrics = TextMetrics {
            width: 42.0,
            height: 16.0,
            ascent: 14.0,
            descent: 2.0,
            line_count: 1,
        };
        let blob = TextBlob { shaped, metrics };
        s.push(SceneEntry {
            source: WidgetId::new("blob"),
            clip: None,
            primitive: RenderPrimitive::TextBlob {
                blob,
                position: Point::new(10.0, 20.0),
                color: Color::BLACK,
            },
        });
        let snap = snapshot_scene(&s);
        assert!(snap.contains("textblob 0g @10,20 adv42.0 #000000ff"), "got: {snap}");
    }

    #[test]
    fn snapshot_includes_external_surface() {
        let mut s = Scene::new();
        s.push(SceneEntry {
            source: WidgetId::new("webview"),
            clip: None,
            primitive: RenderPrimitive::ExternalSurface {
                rect: Rect::from_ltrb(0.0, 96.0, 1280.0, 704.0),
                surface_id: 1,
            },
        });
        let snap = snapshot_scene(&s);
        assert!(snap.contains("surface1 0,96,1280,704"), "got: {snap}");
    }

    #[test]
    fn to_u8_clamps_out_of_range() {
        // Values outside [0,1] are safely clamped.
        assert_eq!(to_u8(-0.5), 0);
        assert_eq!(to_u8(1.5), 255);
        assert_eq!(to_u8(0.5), 128); // 0.5 * 255 = 127.5 → rounds to 128
        assert_eq!(to_u8(0.0), 0);
        assert_eq!(to_u8(1.0), 255);
    }
}
