//! 绘制辅助 — 渐变转换家族（自 helpers.rs 按职责拆分，T59；纯移动零行为变化）。

use zero_css_parser::values::{
    ColorHueMethod, ColorInterpolation, ColorInterpolationSpace, ColorValue, GradientColorStop, GradientDirection,
    GradientValue, LengthValue, RadialSize, eval_calc,
};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{
    GradientColorSpace, GradientInterpolation, GradientKind, GradientPrimitive, GradientStop, HueMethod,
};

use super::color::resolve_color_current;

/// 将 CSS GradientValue 转换为 GradientPrimitive。
///
/// 目前支持 linear-gradient 和 radial-gradient。
/// conic-gradient 暂不渲染（返回 None）。
pub fn gradient_to_primitive(
    gradient: &GradientValue,
    rect: &Rect,
    element_color: &ColorValue,
) -> Option<GradientPrimitive> {
    gradient_to_primitive_with_font_size(gradient, rect, element_color, 16.0)
}

/// 将 CSS GradientValue 转换为 GradientPrimitive，并用当前 font-size 解析残余相对长度。
pub fn gradient_to_primitive_with_font_size(
    gradient: &GradientValue,
    rect: &Rect,
    element_color: &ColorValue,
    font_size_px: f32,
) -> Option<GradientPrimitive> {
    let w = rect.size.width;
    let h = rect.size.height;
    match gradient {
        GradientValue::Linear(lg) => {
            let kind = linear_direction_to_kind(&lg.direction, rect);
            // R5077（css-images-1 §3.2）：色标绝对长度（px）位置定义在渐变线上，
            // 须按渐变线长归一到 [0,1]（渲染侧 t = 线上投影参数）。旧实现 raw px
            // 直接入 offset——t 恒 ≤ 首个色标 offset，整条渐变渲染为首色标纯色
            //（clear-on-replaced-element：90px 高 red/green 硬停带渲全红）。
            let line_length = match kind {
                GradientKind::Linear { x0, y0, x1, y1 } => ((x1 - x0).hypot(y1 - y0)).max(0.0),
                _ => 0.0,
            };
            let stops = convert_color_stops_with_font_size(&lg.stops, element_color, font_size_px, line_length);
            Some(GradientPrimitive {
                interpolation: map_interpolation(lg.interpolation),
                rect: *rect,
                kind,
                stops,
                repeating: lg.repeating,
                clip: None,
            })
        }
        GradientValue::Radial(rg) => {
            let cx = rect.left() + resolve_position_with_font_size(&rg.position_x, w, font_size_px);
            let cy = rect.top() + resolve_position_with_font_size(&rg.position_y, h, font_size_px);
            let outer = match &rg.size {
                RadialSize::ClosestSide => (cx - rect.left())
                    .min(rect.right() - cx)
                    .min(cy - rect.top())
                    .min(rect.bottom() - cy),
                RadialSize::FarthestSide => (cx - rect.left())
                    .max(rect.right() - cx)
                    .max(cy - rect.top())
                    .max(rect.bottom() - cy),
                RadialSize::ClosestCorner => {
                    let tl = (cx - rect.left()).hypot(cy - rect.top());
                    let tr = (rect.right() - cx).hypot(cy - rect.top());
                    let bl = (cx - rect.left()).hypot(rect.bottom() - cy);
                    let br = (rect.right() - cx).hypot(rect.bottom() - cy);
                    tl.min(tr).min(bl).min(br)
                }
                RadialSize::FarthestCorner => {
                    let tl = (cx - rect.left()).hypot(cy - rect.top());
                    let tr = (rect.right() - cx).hypot(cy - rect.top());
                    let bl = (cx - rect.left()).hypot(rect.bottom() - cy);
                    let br = (rect.right() - cx).hypot(rect.bottom() - cy);
                    tl.max(tr).max(bl).max(br)
                }
                RadialSize::Length(lv) => resolve_gradient_length(lv, font_size_px),
            };
            // R5077：径向色标绝对长度相对终止形状半径（outer）归一。
            let line_length = outer.max(0.0);
            let stops = convert_color_stops_with_font_size(&rg.stops, element_color, font_size_px, line_length);
            Some(GradientPrimitive {
                interpolation: map_interpolation(rg.interpolation),
                rect: *rect,
                kind: GradientKind::Radial {
                    cx,
                    cy,
                    inner_radius: 0.0,
                    outer_radius: outer.max(0.01),
                },
                stops,
                repeating: rg.repeating,
                clip: None,
            })
        }
        GradientValue::Conic(cg) => {
            let cx = rect.left() + resolve_position_with_font_size(&cg.position_x, w, font_size_px);
            let cy = rect.top() + resolve_position_with_font_size(&cg.position_y, h, font_size_px);
            let start_angle = cg.from_angle.to_radians() as f32;
            // R5077：conic 色标位置为 angle/%（解析期已归一），无绝对长度——不归一。
            let stops = convert_color_stops_with_font_size(&cg.stops, element_color, font_size_px, 0.0);
            Some(GradientPrimitive {
                interpolation: map_interpolation(cg.interpolation),
                rect: *rect,
                kind: GradientKind::Conic { cx, cy, start_angle },
                stops,
                repeating: cg.repeating,
                clip: None,
            })
        }
    }
}

fn resolve_position_with_font_size(v: &LengthValue, container_size: f32, font_size_px: f32) -> f32 {
    match v {
        LengthValue::Percentage(p) => *p as f32 / 100.0 * container_size,
        LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_) => {
            container_size / 2.0
        }
        other => resolve_gradient_length(other, font_size_px),
    }
}

fn resolve_gradient_length(v: &LengthValue, font_size_px: f32) -> f32 {
    zero_style_system::computed::resolve_length(v, font_size_px as f64, None, None) as f32
}

/// 将 css-parser `ColorInterpolation` 映射为 render-foundation `GradientInterpolation`
/// （CSS Color 4 `gradient in <colorspace>`，R2289）。
fn map_interpolation(i: ColorInterpolation) -> GradientInterpolation {
    let space = match i.space {
        ColorInterpolationSpace::Srgb => GradientColorSpace::Srgb,
        ColorInterpolationSpace::SrgbLinear => GradientColorSpace::SrgbLinear,
        ColorInterpolationSpace::Lab => GradientColorSpace::Lab,
        ColorInterpolationSpace::Oklab => GradientColorSpace::Oklab,
        ColorInterpolationSpace::Lch => GradientColorSpace::Lch,
        ColorInterpolationSpace::Oklch => GradientColorSpace::Oklch,
    };
    let hue = match i.hue {
        ColorHueMethod::Shorter => HueMethod::Shorter,
        ColorHueMethod::Longer => HueMethod::Longer,
        ColorHueMethod::Increasing => HueMethod::Increasing,
        ColorHueMethod::Decreasing => HueMethod::Decreasing,
    };
    GradientInterpolation { space, hue }
}

/// 将线性渐变方向转换为 GradientKind::Linear。
pub fn linear_direction_to_kind(dir: &GradientDirection, rect: &Rect) -> GradientKind {
    let w = rect.size.width;
    let h = rect.size.height;
    let cx = rect.left() + w / 2.0;
    let cy = rect.top() + h / 2.0;
    match dir {
        GradientDirection::ToBottom => GradientKind::Linear {
            x0: cx,
            y0: rect.top(),
            x1: cx,
            y1: rect.bottom(),
        },
        GradientDirection::ToTop => GradientKind::Linear {
            x0: cx,
            y0: rect.bottom(),
            x1: cx,
            y1: rect.top(),
        },
        GradientDirection::ToRight => GradientKind::Linear {
            x0: rect.left(),
            y0: cy,
            x1: rect.right(),
            y1: cy,
        },
        GradientDirection::ToLeft => GradientKind::Linear {
            x0: rect.right(),
            y0: cy,
            x1: rect.left(),
            y1: cy,
        },
        GradientDirection::ToTopRight => GradientKind::Linear {
            x0: rect.left(),
            y0: rect.bottom(),
            x1: rect.right(),
            y1: rect.top(),
        },
        GradientDirection::ToTopLeft => GradientKind::Linear {
            x0: rect.right(),
            y0: rect.bottom(),
            x1: rect.left(),
            y1: rect.top(),
        },
        GradientDirection::ToBottomRight => GradientKind::Linear {
            x0: rect.left(),
            y0: rect.top(),
            x1: rect.right(),
            y1: rect.bottom(),
        },
        GradientDirection::ToBottomLeft => GradientKind::Linear {
            x0: rect.right(),
            y0: rect.top(),
            x1: rect.left(),
            y1: rect.bottom(),
        },
        GradientDirection::Angle(deg) => {
            // 角度转坐标：0deg = to top, 90deg = to right, 180deg = to bottom
            let rad = (deg - 90.0).to_radians();
            let dx = rad.cos();
            let dy = rad.sin();
            let half_diag = w.hypot(h) / 2.0;
            GradientKind::Linear {
                x0: cx - dx as f32 * half_diag,
                y0: cy - dy as f32 * half_diag,
                x1: cx + dx as f32 * half_diag,
                y1: cy + dy as f32 * half_diag,
            }
        }
    }
}

/// 将 CSS 渐变色标转换为渲染层 GradientStop。
pub fn convert_color_stops(stops: &[GradientColorStop], element_color: &ColorValue) -> Vec<GradientStop> {
    convert_color_stops_with_resolver(stops, element_color, |lv| match lv {
        LengthValue::Percentage(p) => *p as f32 / 100.0,
        LengthValue::Px(px) => *px as f32,
        // calc/min/max/clamp 求值为 px（无 parent_length：百分比不可解→None→回退 0.0）。
        // driving: css-images gradient-infinity（calc(1px/0) / calc(Infinity*1px)）。
        LengthValue::Calc(expr) => eval_calc(expr, None).unwrap_or(0.0) as f32,
        _ => 0.0,
    })
}

fn convert_color_stops_with_font_size(
    stops: &[GradientColorStop],
    element_color: &ColorValue,
    font_size_px: f32,
    // R5077：渐变线长（linear）/终止形状半径（radial）；0 = 不归一（conic 等无绝对
    // 长度色标）。绝对长度色标除以它归一到 [0,1]。
    line_length: f32,
) -> Vec<GradientStop> {
    convert_color_stops_with_resolver(stops, element_color, |lv| match lv {
        LengthValue::Percentage(p) => *p as f32 / 100.0,
        LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_) => 0.0,
        other => {
            let px = resolve_gradient_length(other, font_size_px);
            if line_length > 1e-3 { px / line_length } else { px }
        }
    })
}

fn convert_color_stops_with_resolver<F>(
    stops: &[GradientColorStop],
    element_color: &ColorValue,
    mut resolve_position: F,
) -> Vec<GradientStop>
where
    F: FnMut(&LengthValue) -> f32,
{
    let n = stops.len();
    stops
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let offset = s.position.as_ref().map(&mut resolve_position).unwrap_or(if n <= 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            });
            GradientStop {
                offset,
                // R2370：currentColor 按**使用该渐变的元素**自身 color 解析（CSS Color §resolving）。
                // 旧 color_value_to_render 无元素上下文 → currentColor 回落黑色。driving: color-stop-currentcolor。
                color: resolve_color_current(&s.color, element_color),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::helpers::resolve_document_url;
    use zero_css_parser::values::{
        ColorValue, ConicGradient, GradientColorStop, GradientDirection, GradientValue, LengthValue, LinearGradient,
        RadialGradient, RadialShape, RadialSize,
    };
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::GradientKind;

    // ── gradient_to_primitive ───────────────────────────────────────────

    #[test]
    fn test_linear_gradient_to_primitive() {
        let grad = GradientValue::Linear(LinearGradient {
            interpolation: Default::default(),
            direction: GradientDirection::ToBottom,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(255, 0, 0, 255),
                position: None,
            }],
            repeating: false,
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255))
            .expect("linear gradient should convert");
        assert_eq!(prim.rect, rect);
        assert!(matches!(prim.kind, GradientKind::Linear { .. }));
        assert_eq!(prim.stops.len(), 1);
    }

    /// R2370：gradient color-stop 的 currentColor 解析为**使用该渐变的元素**自身 color。
    /// 旧 convert_color_stops 用 color_value_to_render 无元素上下文 → currentColor 回落黑色。
    /// driving: css-images color-stop-currentcolor。
    #[test]
    fn test_gradient_to_primitive_currentcolor_resolves_to_element_color() {
        let grad = GradientValue::Linear(LinearGradient {
            interpolation: Default::default(),
            direction: GradientDirection::ToBottom,
            stops: vec![GradientColorStop {
                color: ColorValue::CurrentColor,
                position: None,
            }],
            repeating: false,
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let element_color = ColorValue::Rgba(255, 0, 0, 255); // red
        let prim = gradient_to_primitive(&grad, &rect, &element_color).expect("gradient should convert");
        assert_eq!(
            prim.stops[0].color,
            zero_render_foundation::color::Color::rgb(255, 0, 0),
            "gradient currentcolor stop 应解析为元素 color（red），非黑色"
        );
    }

    #[test]
    fn test_radial_gradient_to_primitive() {
        let grad = GradientValue::Radial(RadialGradient {
            interpolation: Default::default(),
            shape: RadialShape::Ellipse,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            size: RadialSize::FarthestCorner,
            repeating: false,
            stops: vec![
                GradientColorStop {
                    color: ColorValue::Rgba(255, 255, 255, 255),
                    position: None,
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 0, 0, 255),
                    position: None,
                },
            ],
        });
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255))
            .expect("radial gradient should convert");
        assert!(matches!(prim.kind, GradientKind::Radial { .. }));
    }

    #[test]
    fn test_conic_gradient_to_primitive() {
        let grad = GradientValue::Conic(ConicGradient {
            interpolation: Default::default(),
            repeating: false,
            from_angle: 90.0,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            stops: vec![
                GradientColorStop {
                    color: ColorValue::Rgba(255, 0, 0, 255),
                    position: None,
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 0, 255, 255),
                    position: None,
                },
            ],
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255))
            .expect("conic gradient should convert");
        assert!(matches!(prim.kind, GradientKind::Conic { .. }));
        assert_eq!(prim.stops.len(), 2);
        if let GradientKind::Conic {
            cx, cy, start_angle, ..
        } = prim.kind
        {
            assert!((cx - 100.0).abs() < 0.1);
            assert!((cy - 50.0).abs() < 0.1);
            assert!((start_angle - 90.0_f64.to_radians() as f32).abs() < 0.01);
        }
    }

    #[test]
    fn test_radial_closest_side() {
        let grad = GradientValue::Radial(RadialGradient {
            interpolation: Default::default(),
            shape: RadialShape::Ellipse,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            size: RadialSize::ClosestSide,
            repeating: false,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            }],
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255)).unwrap();
        if let GradientKind::Radial {
            cx, cy, outer_radius, ..
        } = prim.kind
        {
            // 50% of 200 = 100, 50% of 100 = 50
            assert!((cx - 100.0).abs() < 0.1);
            assert!((cy - 50.0).abs() < 0.1);
            assert!((outer_radius - 50.0).abs() < 0.1);
        } else {
            panic!("expected radial gradient");
        }
    }

    #[test]
    fn test_radial_farthest_side() {
        let grad = GradientValue::Radial(RadialGradient {
            interpolation: Default::default(),
            shape: RadialShape::Ellipse,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            size: RadialSize::FarthestSide,
            repeating: false,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            }],
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255)).unwrap();
        if let GradientKind::Radial { outer_radius, .. } = prim.kind {
            assert!((outer_radius - 100.0).abs() < 0.1);
        } else {
            panic!("expected radial gradient");
        }
    }

    #[test]
    fn test_radial_length_size() {
        let grad = GradientValue::Radial(RadialGradient {
            interpolation: Default::default(),
            shape: RadialShape::Ellipse,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            size: RadialSize::Length(LengthValue::Px(75.0)),
            repeating: false,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            }],
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255)).unwrap();
        if let GradientKind::Radial { outer_radius, .. } = prim.kind {
            assert!((outer_radius - 75.0).abs() < 0.1);
        } else {
            panic!("expected radial gradient");
        }
    }

    // ── linear_direction_to_kind ────────────────────────────────────────

    #[test]
    fn test_linear_direction_to_bottom() {
        let rect = Rect::new(0.0, 0.0, 100.0, 200.0);
        let kind = linear_direction_to_kind(&GradientDirection::ToBottom, &rect);
        if let GradientKind::Linear { x0, y0, x1: _, y1: _ } = kind {
            assert!((x0 - 50.0).abs() < 0.01);
            assert!((y0 - 0.0).abs() < 0.01);
        } else {
            panic!("expected linear gradient");
        }
    }

    #[test]
    fn test_linear_direction_to_top() {
        let rect = Rect::new(0.0, 0.0, 100.0, 200.0);
        let kind = linear_direction_to_kind(&GradientDirection::ToTop, &rect);
        if let GradientKind::Linear { x0, y0, x1: _, y1 } = kind {
            assert!((x0 - 50.0).abs() < 0.01);
            assert!((y0 - 200.0).abs() < 0.01);
            assert!((y1 - 0.0).abs() < 0.01);
        } else {
            panic!("expected linear gradient");
        }
    }

    #[test]
    fn test_linear_direction_to_right() {
        let rect = Rect::new(0.0, 0.0, 100.0, 200.0);
        let kind = linear_direction_to_kind(&GradientDirection::ToRight, &rect);
        if let GradientKind::Linear { x0, y0: _, x1, y1: _ } = kind {
            assert!((x0 - 0.0).abs() < 0.01);
            assert!((x1 - 100.0).abs() < 0.01);
        } else {
            panic!("expected linear gradient");
        }
    }

    #[test]
    fn test_linear_direction_to_left() {
        let rect = Rect::new(0.0, 0.0, 100.0, 200.0);
        let kind = linear_direction_to_kind(&GradientDirection::ToLeft, &rect);
        if let GradientKind::Linear { x0, y0: _, x1, y1: _ } = kind {
            assert!((x0 - 100.0).abs() < 0.01);
            assert!((x1 - 0.0).abs() < 0.01);
        } else {
            panic!("expected linear gradient");
        }
    }

    /// R5077（css-images-1 §3.2）：px 色标按渐变线长归一到 [0,1]——90px 高
    /// to bottom 渐变的 `red 30px` → offset 1/3。旧实现 raw px 直入 offset，
    /// 渲染 t∈[0,1] 恒 ≤ 首色标 → 整条渐变渲首色标纯色。
    /// driving: css/CSS2/floats-clear clear-on-replaced-element（90px red/green 硬停带）。
    #[test]
    fn test_r5077_px_stops_normalize_to_gradient_line() {
        let grad = GradientValue::Linear(LinearGradient {
            interpolation: Default::default(),
            direction: GradientDirection::ToBottom,
            stops: vec![
                GradientColorStop {
                    color: ColorValue::Rgba(255, 0, 0, 255),
                    position: Some(LengthValue::Px(30.0)),
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 128, 0, 255),
                    position: Some(LengthValue::Px(60.0)),
                },
            ],
            repeating: false,
        });
        let rect = Rect::new(0.0, 0.0, 90.0, 90.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255))
            .expect("linear gradient should convert");
        assert_eq!(prim.stops.len(), 2);
        assert!(
            (prim.stops[0].offset - 30.0 / 90.0).abs() < 1e-4,
            "px 30 在 90px 渐变线上应归一为 1/3，实际 {}",
            prim.stops[0].offset
        );
        assert!(
            (prim.stops[1].offset - 60.0 / 90.0).abs() < 1e-4,
            "px 60 在 90px 渐变线上应归一为 2/3，实际 {}",
            prim.stops[1].offset
        );
        // 百分比色标不参与归一（语义已相对线长）。
        let grad_pct = GradientValue::Linear(LinearGradient {
            interpolation: Default::default(),
            direction: GradientDirection::ToBottom,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(255, 0, 0, 255),
                position: Some(LengthValue::Percentage(50.0)),
            }],
            repeating: false,
        });
        let prim_pct = gradient_to_primitive(&grad_pct, &rect, &ColorValue::Rgba(0, 0, 0, 255))
            .expect("pct gradient should convert");
        assert!(
            (prim_pct.stops[0].offset - 0.5).abs() < 1e-4,
            "百分比色标保持 p/100，实际 {}",
            prim_pct.stops[0].offset
        );
    }

    #[test]
    fn test_linear_direction_angle() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        // 90deg = to right
        let kind = linear_direction_to_kind(&GradientDirection::Angle(90.0), &rect);
        if let GradientKind::Linear { x0, x1, .. } = kind {
            assert!(x0 < x1); // moves rightward
        } else {
            panic!("expected linear gradient");
        }
    }

    // ── convert_color_stops ─────────────────────────────────────────────

    #[test]
    fn test_convert_stops_with_positions() {
        let stops = vec![
            GradientColorStop {
                color: ColorValue::Rgba(255, 0, 0, 255),
                position: Some(LengthValue::Percentage(0.0)),
            },
            GradientColorStop {
                color: ColorValue::Rgba(0, 0, 255, 255),
                position: Some(LengthValue::Percentage(100.0)),
            },
        ];
        let result = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));
        assert_eq!(result.len(), 2);
        assert!((result[0].offset - 0.0).abs() < 0.01);
        assert!((result[1].offset - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_stops_auto_distribute() {
        let stops = vec![
            GradientColorStop {
                color: ColorValue::Rgba(255, 0, 0, 255),
                position: None,
            },
            GradientColorStop {
                color: ColorValue::Rgba(0, 255, 0, 255),
                position: None,
            },
            GradientColorStop {
                color: ColorValue::Rgba(0, 0, 255, 255),
                position: None,
            },
        ];
        let result = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));
        assert!((result[0].offset - 0.0).abs() < 0.01);
        assert!((result[1].offset - 0.5).abs() < 0.01);
        assert!((result[2].offset - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_stops_single_stop_offset_zero() {
        let stops = vec![GradientColorStop {
            color: ColorValue::Rgba(128, 128, 128, 255),
            position: None,
        }];
        let result = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));
        assert_eq!(result.len(), 1);
        assert!((result[0].offset - 0.0).abs() < 0.01);
    }

    #[test]
    fn resolve_document_url_root_relative_path() {
        assert_eq!(
            resolve_document_url("https://example.com/page", "/aaa"),
            "https://example.com/aaa"
        );
    }

    #[test]
    fn resolve_document_url_relative_path() {
        assert_eq!(
            resolve_document_url("https://example.com/dir/page", "other.html"),
            "https://example.com/dir/other.html"
        );
    }

    #[test]
    fn resolve_document_url_preserves_special_schemes() {
        assert_eq!(
            resolve_document_url("https://example.com", "mailto:test@example.com"),
            "mailto:test@example.com"
        );
        assert_eq!(
            resolve_document_url("https://example.com", "javascript:void(0)"),
            "javascript:void(0)"
        );
    }
}
