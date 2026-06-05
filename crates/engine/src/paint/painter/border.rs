//! 边框绘制 — 常规边框、border-image、outline。
//!
//! 包含 BorderEdgeSpec、paint_borders、paint_border_image、paint_border_edge、
//! border_fill_rect、paint_3d_border、paint_outline。

use zero_css_parser::values::ColorValue;
use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageKey;
use zero_render_foundation::primitive::{ImagePrimitive, LineCap, LineStyle, StrokePrimitive};
use zero_style_system::{BorderCollapseValue, BorderImageSourceComputedValue, BorderStyleValue, ComputedStyle};

use super::super::color::color_value_to_render;
use super::super::helpers::{length_to_f32, simple_hash};

/// 边框边缘规格 — 描述一条边框的几何位置和方向。
pub(super) struct BorderEdgeSpec {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub thickness: f32,
    pub is_horizontal: bool,
    /// 垂直边框时，填充区域是否向左延伸（右侧边框为 true）。
    pub extend_left: bool,
}

impl super::Painter {
    /// 绘制边框（4 条边，支持多种 border-style）。
    ///
    /// 分别绘制上、右、下、左四条边框。根据 border-style 生成不同类型的图元：
    /// - Solid/None/Hidden：填充矩形（原有行为）
    /// - Dotted：圆头点线描边
    /// - Dashed：方头虚线描边
    /// - Double：双线填充矩形（中间留空隙）
    /// - Groove/Ridge/Inset/Outset：3D 效果双色填充
    pub(super) fn paint_borders(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let w = box_node.width;
        let h = box_node.height;

        // border-collapse:collapse 时，内边框（右和下）厚度减半，避免与邻居重叠
        let collapse = matches!(style.border_collapse, BorderCollapseValue::Collapse);
        let half = |v: f32| if collapse { v / 2.0 } else { v };

        // 上边框
        if box_node.border_top > 0.0
            && style.border_top_style != BorderStyleValue::None
            && style.border_top_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x,
                    y1: abs_y,
                    x2: abs_x + w,
                    y2: abs_y,
                    thickness: half(box_node.border_top),
                    is_horizontal: true,
                    extend_left: false,
                },
                &style.border_top_style,
                &style.border_top_color,
            );
        }

        // 右边框
        if box_node.border_right > 0.0
            && style.border_right_style != BorderStyleValue::None
            && style.border_right_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x + w,
                    y1: abs_y + half(box_node.border_top),
                    x2: abs_x + w,
                    y2: abs_y + h - half(box_node.border_bottom),
                    thickness: half(box_node.border_right),
                    is_horizontal: false,
                    extend_left: true,
                },
                &style.border_right_style,
                &style.border_right_color,
            );
        }

        // 下边框
        if box_node.border_bottom > 0.0
            && style.border_bottom_style != BorderStyleValue::None
            && style.border_bottom_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x,
                    y1: abs_y + h,
                    x2: abs_x + w,
                    y2: abs_y + h,
                    thickness: half(box_node.border_bottom),
                    is_horizontal: true,
                    extend_left: false,
                },
                &style.border_bottom_style,
                &style.border_bottom_color,
            );
        }

        // 左边框
        if box_node.border_left > 0.0
            && style.border_left_style != BorderStyleValue::None
            && style.border_left_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x,
                    y1: abs_y + half(box_node.border_top),
                    x2: abs_x,
                    y2: abs_y + h - half(box_node.border_bottom),
                    thickness: half(box_node.border_left),
                    is_horizontal: false,
                    extend_left: false,
                },
                &style.border_left_style,
                &style.border_left_color,
            );
        }
    }

    /// 绘制 border-image。
    ///
    /// 当 border-image-source 不为 none 时，将图片按 slice 分割为
    /// 9 个区域（4 角 + 4 边 + 中心），分别绘制到边框的对应区域。
    /// 当前实现支持 stretch 模式（默认），生成 ImagePrimitive 图元。
    pub(super) fn paint_border_image(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let url = match &style.border_image_source {
            BorderImageSourceComputedValue::None => return,
            BorderImageSourceComputedValue::Url(u) => u.clone(),
        };

        let bt = box_node.border_top;
        let br = box_node.border_right;
        let bb = box_node.border_bottom;
        let bl = box_node.border_left;

        // 至少有一条边框才绘制
        if bt <= 0.0 && br <= 0.0 && bb <= 0.0 && bl <= 0.0 {
            return;
        }

        let w = box_node.width;
        let h = box_node.height;
        let key = simple_hash(&url);

        // 辅助：创建 ImagePrimitive（每次创建新的 ImageKey，因为 ImageKey 不是 Copy）
        let make_img = |rect: Rect| ImagePrimitive {
            rect,
            image_key: ImageKey::new(key),
        };

        // 边框区域的坐标
        let bx = abs_x;
        let by = abs_y;

        // 四条边的尺寸
        let edge_h_w = (w - bl - br).max(0.0);
        let edge_v_h = (h - bt - bb).max(0.0);

        let fill = style.border_image_slice.fill;

        // 中心区域（当 fill 为 true 时绘制）
        if fill && edge_h_w > 0.0 && edge_v_h > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + bl, by + bt, edge_h_w, edge_v_h)));
        }

        // 四个角
        if bl > 0.0 && bt > 0.0 {
            self.primitives.add_image(make_img(Rect::new(bx, by, bl, bt)));
        }
        if br > 0.0 && bt > 0.0 {
            self.primitives.add_image(make_img(Rect::new(bx + w - br, by, br, bt)));
        }
        if br > 0.0 && bb > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + w - br, by + h - bb, br, bb)));
        }
        if bl > 0.0 && bb > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx, by + h - bb, bl, bb)));
        }

        // 四条边（stretch 模式）
        if edge_h_w > 0.0 && bt > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + bl, by, edge_h_w, bt)));
        }
        if br > 0.0 && edge_v_h > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + w - br, by + bt, br, edge_v_h)));
        }
        if edge_h_w > 0.0 && bb > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + bl, by + h - bb, edge_h_w, bb)));
        }
        if bl > 0.0 && edge_v_h > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx, by + bt, bl, edge_v_h)));
        }
    }

    /// 绘制单条边框（根据 border-style 生成合适的图元）。
    pub(super) fn paint_border_edge(
        &mut self,
        spec: &BorderEdgeSpec,
        border_style: &BorderStyleValue,
        color: &ColorValue,
    ) {
        let render_color = color_value_to_render(color);

        match border_style {
            BorderStyleValue::None | BorderStyleValue::Hidden => {}
            BorderStyleValue::Solid => {
                self.primitives.add_fill(self.border_fill_rect(spec), render_color);
            }
            BorderStyleValue::Dotted => {
                self.primitives.add_stroke(StrokePrimitive {
                    x1: spec.x1,
                    y1: spec.y1,
                    x2: spec.x2,
                    y2: spec.y2,
                    width: spec.thickness,
                    color: render_color,
                    style: LineStyle::Dotted,
                    cap: LineCap::Round,
                });
            }
            BorderStyleValue::Dashed => {
                self.primitives.add_stroke(StrokePrimitive {
                    x1: spec.x1,
                    y1: spec.y1,
                    x2: spec.x2,
                    y2: spec.y2,
                    width: spec.thickness,
                    color: render_color,
                    style: LineStyle::Dashed,
                    cap: LineCap::Square,
                });
            }
            BorderStyleValue::Double => {
                let gap = (spec.thickness / 3.0).max(1.0);
                let line_w = ((spec.thickness - gap) / 2.0).max(1.0);
                if spec.is_horizontal {
                    self.primitives
                        .add_fill(Rect::new(spec.x1, spec.y1, spec.x2 - spec.x1, line_w), render_color);
                    self.primitives.add_fill(
                        Rect::new(spec.x1, spec.y1 + line_w + gap, spec.x2 - spec.x1, line_w),
                        render_color,
                    );
                } else {
                    let outer_x = if spec.extend_left {
                        spec.x1 - spec.thickness
                    } else {
                        spec.x1
                    };
                    self.primitives
                        .add_fill(Rect::new(outer_x, spec.y1, line_w, spec.y2 - spec.y1), render_color);
                    self.primitives.add_fill(
                        Rect::new(outer_x + line_w + gap, spec.y1, line_w, spec.y2 - spec.y1),
                        render_color,
                    );
                }
            }
            BorderStyleValue::Groove => {
                let (light, dark) = groove_ridge_colors(&render_color);
                self.paint_3d_border(spec, &light, &dark);
            }
            BorderStyleValue::Ridge => {
                let (light, dark) = groove_ridge_colors(&render_color);
                self.paint_3d_border(spec, &dark, &light);
            }
            BorderStyleValue::Inset => {
                let lighter = lighten(&render_color, 0.3);
                let darker = darken(&render_color, 0.3);
                self.paint_3d_border(spec, &darker, &lighter);
            }
            BorderStyleValue::Outset => {
                let lighter = lighten(&render_color, 0.3);
                let darker = darken(&render_color, 0.3);
                self.paint_3d_border(spec, &lighter, &darker);
            }
        }
    }

    /// 根据 BorderEdgeSpec 计算填充矩形。
    fn border_fill_rect(&self, spec: &BorderEdgeSpec) -> Rect {
        if spec.is_horizontal {
            Rect::new(spec.x1, spec.y1, spec.x2 - spec.x1, spec.thickness)
        } else if spec.extend_left {
            Rect::new(spec.x1 - spec.thickness, spec.y1, spec.thickness, spec.y2 - spec.y1)
        } else {
            Rect::new(spec.x1, spec.y1, spec.thickness, spec.y2 - spec.y1)
        }
    }

    /// 绘制 3D 效果边框（groove/ridge/inset/outset 使用）。
    ///
    /// 将边框分为上下两半（水平边）或左右两半（垂直边），分别使用不同颜色。
    fn paint_3d_border(&mut self, spec: &BorderEdgeSpec, first_color: &Color, second_color: &Color) {
        let half = spec.thickness / 2.0;
        if spec.is_horizontal {
            self.primitives
                .add_fill(Rect::new(spec.x1, spec.y1, spec.x2 - spec.x1, half), *first_color);
            self.primitives.add_fill(
                Rect::new(spec.x1, spec.y1 + half, spec.x2 - spec.x1, half),
                *second_color,
            );
        } else {
            let fill_x = if spec.extend_left {
                spec.x1 - spec.thickness
            } else {
                spec.x1
            };
            self.primitives
                .add_fill(Rect::new(fill_x, spec.y1, half, spec.y2 - spec.y1), *first_color);
            self.primitives.add_fill(
                Rect::new(fill_x + half, spec.y1, half, spec.y2 - spec.y1),
                *second_color,
            );
        }
    }

    /// 绘制 outline（位于 border 外侧，支持多种 outline-style）。
    ///
    /// outline 绘制为 4 条边框段，根据 outline-style 生成不同图元类型。
    pub(super) fn paint_outline(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::OutlineStyleValue;

        let outline_width = length_to_f32(&style.outline_width);

        if outline_width <= 0.0 || style.outline_style == OutlineStyleValue::None {
            return;
        }

        let offset = length_to_f32(&style.outline_offset);

        let w = box_node.width;
        let h = box_node.height;
        let total_offset = outline_width + offset;
        let color = color_value_to_render(&style.outline_color);

        // 计算外侧矩形坐标
        let ox = abs_x - total_offset;
        let oy = abs_y - total_offset;
        let ow = w + 2.0 * total_offset;
        let oh = h + 2.0 * total_offset;

        match style.outline_style {
            OutlineStyleValue::None => {}
            OutlineStyleValue::Solid => {
                // 实线：4 个填充矩形
                self.primitives.add_fill(Rect::new(ox, oy, ow, outline_width), color);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - outline_width, ow, outline_width), color);
                self.primitives.add_fill(
                    Rect::new(ox, oy + outline_width, outline_width, oh - 2.0 * outline_width),
                    color,
                );
                self.primitives.add_fill(
                    Rect::new(
                        ox + ow - outline_width,
                        oy + outline_width,
                        outline_width,
                        oh - 2.0 * outline_width,
                    ),
                    color,
                );
            }
            OutlineStyleValue::Dotted => {
                let mid_y_top = oy + outline_width / 2.0;
                let mid_y_bottom = oy + oh - outline_width / 2.0;
                let mid_x_left = ox + outline_width / 2.0;
                let mid_x_right = ox + ow - outline_width / 2.0;
                for (x1, y1, x2, y2) in [
                    (ox, mid_y_top, ox + ow, mid_y_top),
                    (ox, mid_y_bottom, ox + ow, mid_y_bottom),
                    (mid_x_left, mid_y_top, mid_x_left, mid_y_bottom),
                    (mid_x_right, mid_y_top, mid_x_right, mid_y_bottom),
                ] {
                    self.primitives.add_stroke(StrokePrimitive {
                        x1,
                        y1,
                        x2,
                        y2,
                        width: outline_width,
                        color,
                        style: LineStyle::Dotted,
                        cap: LineCap::Round,
                    });
                }
            }
            OutlineStyleValue::Dashed => {
                let mid_y_top = oy + outline_width / 2.0;
                let mid_y_bottom = oy + oh - outline_width / 2.0;
                let mid_x_left = ox + outline_width / 2.0;
                let mid_x_right = ox + ow - outline_width / 2.0;
                for (x1, y1, x2, y2) in [
                    (ox, mid_y_top, ox + ow, mid_y_top),
                    (ox, mid_y_bottom, ox + ow, mid_y_bottom),
                    (mid_x_left, mid_y_top, mid_x_left, mid_y_bottom),
                    (mid_x_right, mid_y_top, mid_x_right, mid_y_bottom),
                ] {
                    self.primitives.add_stroke(StrokePrimitive {
                        x1,
                        y1,
                        x2,
                        y2,
                        width: outline_width,
                        color,
                        style: LineStyle::Dashed,
                        cap: LineCap::Square,
                    });
                }
            }
            OutlineStyleValue::Double => {
                let gap = (outline_width / 3.0).max(1.0);
                let line_w = ((outline_width - gap) / 2.0).max(1.0);
                // 外线
                self.primitives.add_fill(Rect::new(ox, oy, ow, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - line_w, ow, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ox, oy + line_w, line_w, oh - 2.0 * line_w), color);
                self.primitives.add_fill(
                    Rect::new(ox + ow - line_w, oy + line_w, line_w, oh - 2.0 * line_w),
                    color,
                );
                // 内线
                let ix = ox + line_w + gap;
                let iy = oy + line_w + gap;
                let iw = ow - 2.0 * (line_w + gap);
                let ih = oh - 2.0 * (line_w + gap);
                self.primitives.add_fill(Rect::new(ix, iy, iw, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ix, iy + ih - line_w, iw, line_w), color);
                self.primitives
                    .add_fill(Rect::new(ix, iy + line_w, line_w, ih - 2.0 * line_w), color);
                self.primitives.add_fill(
                    Rect::new(ix + iw - line_w, iy + line_w, line_w, ih - 2.0 * line_w),
                    color,
                );
            }
            OutlineStyleValue::Groove | OutlineStyleValue::Ridge => {
                let (first, second) = if matches!(style.outline_style, OutlineStyleValue::Groove) {
                    groove_ridge_colors(&color)
                } else {
                    let (l, d) = groove_ridge_colors(&color);
                    (d, l)
                };
                let half = outline_width / 2.0;
                self.primitives.add_fill(Rect::new(ox, oy, ow, half), first);
                self.primitives.add_fill(Rect::new(ox, oy + half, ow, half), second);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - outline_width, ow, half), first);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - half, ow, half), second);
                self.primitives
                    .add_fill(Rect::new(ox, oy + outline_width, half, oh - 2.0 * outline_width), first);
                self.primitives.add_fill(
                    Rect::new(ox + half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
                self.primitives.add_fill(
                    Rect::new(
                        ox + ow - outline_width,
                        oy + outline_width,
                        half,
                        oh - 2.0 * outline_width,
                    ),
                    first,
                );
                self.primitives.add_fill(
                    Rect::new(ox + ow - half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
            }
            OutlineStyleValue::Inset | OutlineStyleValue::Outset => {
                let (first, second) = if matches!(style.outline_style, OutlineStyleValue::Inset) {
                    (darken(&color, 0.3), lighten(&color, 0.3))
                } else {
                    (lighten(&color, 0.3), darken(&color, 0.3))
                };
                let half = outline_width / 2.0;
                self.primitives.add_fill(Rect::new(ox, oy, ow, half), first);
                self.primitives.add_fill(Rect::new(ox, oy + half, ow, half), second);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - outline_width, ow, half), first);
                self.primitives
                    .add_fill(Rect::new(ox, oy + oh - half, ow, half), second);
                self.primitives
                    .add_fill(Rect::new(ox, oy + outline_width, half, oh - 2.0 * outline_width), first);
                self.primitives.add_fill(
                    Rect::new(ox + half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
                self.primitives.add_fill(
                    Rect::new(
                        ox + ow - outline_width,
                        oy + outline_width,
                        half,
                        oh - 2.0 * outline_width,
                    ),
                    first,
                );
                self.primitives.add_fill(
                    Rect::new(ox + ow - half, oy + outline_width, half, oh - 2.0 * outline_width),
                    second,
                );
            }
        }
    }
}

/// Groove/Ridge 颜色对生成。
fn groove_ridge_colors(color: &Color) -> (Color, Color) {
    (lighten(color, 0.3), darken(color, 0.3))
}

/// 使颜色变亮。
fn lighten(color: &Color, amount: f32) -> Color {
    Color::rgba(
        (color.r as f32 + (255.0 - color.r as f32) * amount).min(255.0) as u8,
        (color.g as f32 + (255.0 - color.g as f32) * amount).min(255.0) as u8,
        (color.b as f32 + (255.0 - color.b as f32) * amount).min(255.0) as u8,
        color.a,
    )
}

/// 使颜色变暗。
fn darken(color: &Color, amount: f32) -> Color {
    Color::rgba(
        (color.r as f32 * (1.0 - amount)).min(255.0) as u8,
        (color.g as f32 * (1.0 - amount)).min(255.0) as u8,
        (color.b as f32 * (1.0 - amount)).min(255.0) as u8,
        color.a,
    )
}
