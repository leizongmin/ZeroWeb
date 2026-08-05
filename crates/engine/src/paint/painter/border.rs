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
use zero_style_system::{
    BorderCollapseValue, BorderImageOutsetComputedComponent, BorderImageRepeatComputedMode,
    BorderImageSourceComputedValue, BorderImageWidthComputedComponent, BorderStyleValue, ComputedStyle,
};

use super::super::color::{color_value_to_render, resolve_color_current};
use super::super::helpers::{image_resource_key, length_to_f32};

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
    pub(super) fn paint_borders(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let w = box_node.width;
        let h = box_node.height;

        // border-collapse:collapse 时，内共享边（有邻居）厚度减半，两邻居各画一半合成
        // 居中的完整边框（CSS 2.1 §17.6.2）；外边缘（表格周边，无邻居共享）须绘制
        // 完整厚度。`collapsed_border_outer_edge` 由 resolve_collapsed_borders 阶段 4 标记。
        // kill-switch `ZW_COLLAPSE_OUTER_FULL=0` 关闭（default-on，恢复旧行为：全部减半）。
        let collapse = matches!(style.border_collapse, BorderCollapseValue::Collapse);
        let outer_full = collapse && std::env::var("ZW_COLLAPSE_OUTER_FULL").as_deref() != Ok("0");
        let outer = &box_node.collapsed_border_outer_edge;
        // 每条边的有效厚度：collapse 时外边缘=full，内共享边=half；非 collapse=原值。
        let eff = |side: usize, v: f32| -> f32 {
            if collapse {
                if outer_full && outer[side] { v } else { v / 2.0 }
            } else {
                v
            }
        };
        let top_t = eff(0, box_node.border_top);
        let right_t = eff(1, box_node.border_right);
        let bottom_t = eff(2, box_node.border_bottom);
        let left_t = eff(3, box_node.border_left);

        // border-collapse 颜色覆盖：当表格边框胜出时使用表格的颜色
        let top_color = collapsed_border_color(&box_node.collapsed_border_color_overrides[0], &style.border_top_color);
        let right_color =
            collapsed_border_color(&box_node.collapsed_border_color_overrides[1], &style.border_right_color);
        let bottom_color = collapsed_border_color(
            &box_node.collapsed_border_color_overrides[2],
            &style.border_bottom_color,
        );
        let left_color =
            collapsed_border_color(&box_node.collapsed_border_color_overrides[3], &style.border_left_color);

        // currentColor 解析为元素自身计算 `color`（CSS-Color §resolving）。border-color 初始值
        // 为 currentColor 关键字，经层叠/继承保留，paint 时替换为元素 color 的使用值。
        let current = &style.color;
        let top_color = resolve_border_current_color(top_color, current);
        let right_color = resolve_border_current_color(right_color, current);
        let bottom_color = resolve_border_current_color(bottom_color, current);
        let left_color = resolve_border_current_color(left_color, current);

        // border-collapse 样式覆盖：冲突解决后使用获胜方的样式
        let top_style = box_node.collapsed_border_style_overrides[0]
            .as_ref()
            .unwrap_or(&style.border_top_style);
        let right_style = box_node.collapsed_border_style_overrides[1]
            .as_ref()
            .unwrap_or(&style.border_right_style);
        let bottom_style = box_node.collapsed_border_style_overrides[2]
            .as_ref()
            .unwrap_or(&style.border_bottom_style);
        let left_style = box_node.collapsed_border_style_overrides[3]
            .as_ref()
            .unwrap_or(&style.border_left_style);

        // 上边框
        if box_node.border_top > 0.0 && *top_style != BorderStyleValue::None && *top_style != BorderStyleValue::Hidden {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x,
                    y1: abs_y,
                    x2: abs_x + w,
                    y2: abs_y,
                    thickness: top_t,
                    is_horizontal: true,
                    extend_left: false,
                },
                top_style,
                &top_color,
            );
        }

        // 右边框
        if box_node.border_right > 0.0
            && *right_style != BorderStyleValue::None
            && *right_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x + w,
                    y1: abs_y + top_t,
                    x2: abs_x + w,
                    y2: abs_y + h - bottom_t,
                    thickness: right_t,
                    is_horizontal: false,
                    extend_left: true,
                },
                right_style,
                &right_color,
            );
        }

        // 下边框 — y1 在边框盒底边向上 border_bottom 处（边框区域内侧）
        if box_node.border_bottom > 0.0
            && *bottom_style != BorderStyleValue::None
            && *bottom_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x,
                    y1: abs_y + h - bottom_t,
                    x2: abs_x + w,
                    y2: abs_y + h - bottom_t,
                    thickness: bottom_t,
                    is_horizontal: true,
                    extend_left: false,
                },
                bottom_style,
                &bottom_color,
            );
        }

        // 左边框
        if box_node.border_left > 0.0
            && *left_style != BorderStyleValue::None
            && *left_style != BorderStyleValue::Hidden
        {
            self.paint_border_edge(
                &BorderEdgeSpec {
                    x1: abs_x,
                    y1: abs_y + top_t,
                    x2: abs_x,
                    y2: abs_y + h - bottom_t,
                    thickness: left_t,
                    is_horizontal: false,
                    extend_left: false,
                },
                left_style,
                &left_color,
            );
        }
    }

    /// 绘制 border-image。
    ///
    /// 当 border-image-source 不为 none 时，将图片按 slice 分割为
    /// 9 个区域（4 角 + 4 边 + 中心），分别绘制到边框的对应区域。
    /// 支持所有 border-image-repeat 模式：stretch/repeat/round/space。
    pub(super) fn paint_border_image(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let url = match &style.border_image_source {
            BorderImageSourceComputedValue::None => return,
            BorderImageSourceComputedValue::Url(u) => u.clone(),
            // gradient border-image-source：getComputedStyle 序列化已支持（R2753），但 paint 层将
            // 渐变采样为 9-slice 边框图属复杂渲染（暂未实现），暂不绘制（等同 none，不 panic）。
            BorderImageSourceComputedValue::Gradient(_) => return,
        };

        let bt = box_node.border_top;
        let br = box_node.border_right;
        let bb = box_node.border_bottom;
        let bl = box_node.border_left;

        // 至少有一条边框才绘制
        if bt <= 0.0 && br <= 0.0 && bb <= 0.0 && bl <= 0.0 {
            return;
        }

        let key = image_resource_key(&url, self.document_url.as_deref());

        // 辅助：创建 ImagePrimitive（每次创建新的 ImageKey，因为 ImageKey 不是 Copy）
        let make_img = |rect: Rect| ImagePrimitive {
            rect,
            image_key: ImageKey::new(key),
            clip: None,
        };

        // border-image-outset（CSS Backgrounds 3 §7.2）：border-image 绘制区向外扩展。
        // Number = × 对应边框宽度；Length = 已解析 px。各边 outset 把外矩形向外移，
        // 9 宫格（角/边/中心）相对外矩形定位 → 仅调整 bx/by/w/h 即可（其余公式不变）。
        let outset_px = |c: &BorderImageOutsetComputedComponent, bw: f32| -> f32 {
            match c {
                BorderImageOutsetComputedComponent::Number(n) => n * bw,
                BorderImageOutsetComputedComponent::Length(l) => *l,
            }
        };
        let o_top = outset_px(&style.border_image_outset.top, bt);
        let o_right = outset_px(&style.border_image_outset.right, br);
        let o_bottom = outset_px(&style.border_image_outset.bottom, bb);
        let o_left = outset_px(&style.border_image_outset.left, bl);

        // 边框区域坐标（含 outset 外扩；默认 outset=0 → bx=abs_x/by=abs_y，旧行为不变）。
        let bx = abs_x - o_left;
        let by = abs_y - o_top;
        let w = box_node.width + o_left + o_right;
        let h = box_node.height + o_top + o_bottom;

        // border-image-width（CSS Backgrounds 3 §7.3）：覆盖 9 宫格区域厚度（绘制用）。
        // 默认 Number(1.0) = 1×边框宽度 = 边框宽度 → 区域厚度同边框（旧行为）；显式值则
        // 用指定厚度绘制（可大于边框延伸进 padding/content，或小于）。Auto 简化为边框宽度
        //（真 auto 用图像固有尺寸，ZW paint 期无解码尺寸）；Percent 相对边框盒对应轴。
        // shadow bt/br/bb/bl → 下方 9 宫格公式（角/边/中心）自动用新厚度，guard/outset 既用原值不变。
        let width_px = |c: &BorderImageWidthComputedComponent, bw: f32, box_dim: f32| -> f32 {
            match c {
                BorderImageWidthComputedComponent::Auto => bw,
                BorderImageWidthComputedComponent::Number(n) => n * bw,
                BorderImageWidthComputedComponent::Length(l) => *l,
                BorderImageWidthComputedComponent::Percent(p) => (p / 100.0) * box_dim,
            }
        };
        let bt = width_px(&style.border_image_width.top, bt, h);
        let br = width_px(&style.border_image_width.right, br, w);
        let bb = width_px(&style.border_image_width.bottom, bb, h);
        let bl = width_px(&style.border_image_width.left, bl, w);

        // 四条边的尺寸
        let edge_h_w = (w - bl - br).max(0.0);
        let edge_v_h = (h - bt - bb).max(0.0);

        let fill = style.border_image_slice.fill;

        // 中心区域（当 fill 为 true 时绘制，始终 stretch）
        if fill && edge_h_w > 0.0 && edge_v_h > 0.0 {
            self.primitives
                .add_image(make_img(Rect::new(bx + bl, by + bt, edge_h_w, edge_v_h)));
        }

        // 四个角（始终 stretch，不受 repeat 模式影响）
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
            self.primitives.add_image(make_img(Rect::new(bx, by + h - bb, bl, bb)));
        }

        // 四条边 — 根据 border-image-repeat 模式生成图元
        let h_mode = &style.border_image_repeat.horizontal;
        let v_mode = &style.border_image_repeat.vertical;

        // 上边（水平 repeat 模式）
        if edge_h_w > 0.0 && bt > 0.0 {
            self.paint_border_image_edge_h(
                make_img,
                bx + bl,
                by,
                edge_h_w,
                bt,
                bl, // 自然 tile 宽度 = 左边框宽度
                h_mode,
            );
        }
        // 右边（垂直 repeat 模式）
        if br > 0.0 && edge_v_h > 0.0 {
            self.paint_border_image_edge_v(
                make_img,
                bx + w - br,
                by + bt,
                br,
                edge_v_h,
                bt, // 自然 tile 高度 = 上边框高度
                v_mode,
            );
        }
        // 下边（水平 repeat 模式）
        if edge_h_w > 0.0 && bb > 0.0 {
            self.paint_border_image_edge_h(
                make_img,
                bx + bl,
                by + h - bb,
                edge_h_w,
                bb,
                bl, // 自然 tile 宽度 = 左边框宽度
                h_mode,
            );
        }
        // 左边（垂直 repeat 模式）
        if bl > 0.0 && edge_v_h > 0.0 {
            self.paint_border_image_edge_v(
                make_img,
                bx,
                by + bt,
                bl,
                edge_v_h,
                bt, // 自然 tile 高度 = 上边框高度
                v_mode,
            );
        }
    }

    /// 绘制水平方向的 border-image 边（上边/下边）。
    ///
    /// `tile_w` 是单个 tile 的自然宽度（对应边框宽度），
    /// `total_w` 是需要覆盖的总宽度。
    #[allow(clippy::too_many_arguments)]
    fn paint_border_image_edge_h(
        &mut self,
        make_img: impl Fn(Rect) -> ImagePrimitive,
        start_x: f32,
        y: f32,
        total_w: f32,
        edge_h: f32,
        tile_w: f32,
        mode: &BorderImageRepeatComputedMode,
    ) {
        match mode {
            BorderImageRepeatComputedMode::Stretch => {
                // 拉伸单个 tile 覆盖整条边
                self.primitives
                    .add_image(make_img(Rect::new(start_x, y, total_w, edge_h)));
            }
            BorderImageRepeatComputedMode::Repeat => {
                // 以自然 tile 大小重复，从中心向两边展开
                let n = (total_w / tile_w).ceil().max(1.0) as usize;
                let total_tiles_w = n as f32 * tile_w;
                let mut x = start_x + (total_w - total_tiles_w) / 2.0;
                for _ in 0..n {
                    let clipped = Self::clip_tile(x, y, tile_w, edge_h, start_x, y, total_w, edge_h);
                    if let Some((cx, cy, cw, ch)) = clipped {
                        self.primitives.add_image(make_img(Rect::new(cx, cy, cw, ch)));
                    }
                    x += tile_w;
                }
            }
            BorderImageRepeatComputedMode::Round => {
                // 拉伸 tile 使整数个刚好覆盖
                let n = (total_w / tile_w).round().max(1.0) as usize;
                let stretched = total_w / n as f32;
                let mut x = start_x;
                for _ in 0..n {
                    self.primitives.add_image(make_img(Rect::new(x, y, stretched, edge_h)));
                    x += stretched;
                }
            }
            BorderImageRepeatComputedMode::Space => {
                // 均匀分布 tile，不足 2 个时退化为 stretch
                let n = (total_w / tile_w).floor().max(0.0) as usize;
                if n <= 1 {
                    self.primitives
                        .add_image(make_img(Rect::new(start_x, y, total_w, edge_h)));
                } else {
                    let gap = (total_w - n as f32 * tile_w) / (n + 1) as f32;
                    let mut x = start_x + gap;
                    for _ in 0..n {
                        self.primitives.add_image(make_img(Rect::new(x, y, tile_w, edge_h)));
                        x += tile_w + gap;
                    }
                }
            }
        }
    }

    /// 绘制垂直方向的 border-image 边（左边/右边）。
    #[allow(clippy::too_many_arguments)]
    fn paint_border_image_edge_v(
        &mut self,
        make_img: impl Fn(Rect) -> ImagePrimitive,
        x: f32,
        start_y: f32,
        edge_w: f32,
        total_h: f32,
        tile_h: f32,
        mode: &BorderImageRepeatComputedMode,
    ) {
        match mode {
            BorderImageRepeatComputedMode::Stretch => {
                self.primitives
                    .add_image(make_img(Rect::new(x, start_y, edge_w, total_h)));
            }
            BorderImageRepeatComputedMode::Repeat => {
                let n = (total_h / tile_h).ceil().max(1.0) as usize;
                let total_tiles_h = n as f32 * tile_h;
                let mut y = start_y + (total_h - total_tiles_h) / 2.0;
                for _ in 0..n {
                    let clipped = Self::clip_tile(x, y, edge_w, tile_h, x, start_y, edge_w, total_h);
                    if let Some((cx, cy, cw, ch)) = clipped {
                        self.primitives.add_image(make_img(Rect::new(cx, cy, cw, ch)));
                    }
                    y += tile_h;
                }
            }
            BorderImageRepeatComputedMode::Round => {
                let n = (total_h / tile_h).round().max(1.0) as usize;
                let stretched = total_h / n as f32;
                let mut y = start_y;
                for _ in 0..n {
                    self.primitives.add_image(make_img(Rect::new(x, y, edge_w, stretched)));
                    y += stretched;
                }
            }
            BorderImageRepeatComputedMode::Space => {
                let n = (total_h / tile_h).floor().max(0.0) as usize;
                if n <= 1 {
                    self.primitives
                        .add_image(make_img(Rect::new(x, start_y, edge_w, total_h)));
                } else {
                    let gap = (total_h - n as f32 * tile_h) / (n + 1) as f32;
                    let mut y = start_y + gap;
                    for _ in 0..n {
                        self.primitives.add_image(make_img(Rect::new(x, y, edge_w, tile_h)));
                        y += tile_h + gap;
                    }
                }
            }
        }
    }

    /// 裁剪单个 tile 到边界区域，返回裁剪后的 (x, y, w, h)。
    /// 如果 tile 完全在边界外返回 None。
    #[allow(clippy::too_many_arguments)]
    fn clip_tile(
        tile_x: f32,
        tile_y: f32,
        tile_w: f32,
        tile_h: f32,
        clip_x: f32,
        clip_y: f32,
        clip_w: f32,
        clip_h: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        let tile_right = tile_x + tile_w;
        let tile_bottom = tile_y + tile_h;
        let clip_right = clip_x + clip_w;
        let clip_bottom = clip_y + clip_h;

        if tile_right <= clip_x || tile_x >= clip_right || tile_bottom <= clip_y || tile_y >= clip_bottom {
            return None;
        }

        let cx = tile_x.max(clip_x);
        let cy = tile_y.max(clip_y);
        let cw = tile_right.min(clip_right) - cx;
        let ch = tile_bottom.min(clip_bottom) - cy;

        if cw > 0.0 && ch > 0.0 {
            Some((cx, cy, cw, ch))
        } else {
            None
        }
    }

    /// R1141：dashed/dotted border 用 StrokePrimitive（线居中）绘制，但 border 应在 border-box
    /// 内侧（同 Solid 的 fill rect 语义）。返回使 stroke 中心从「边界线」移到「内侧填充区中心」
    /// 的偏移：horizontal（top/bottom）→ y += thickness/2；vertical 左边框 → x += thickness/2；
    /// vertical 右边框（extend_left）→ x -= thickness/2。
    fn stroke_inward_offset(spec: &BorderEdgeSpec) -> (f32, f32) {
        let half = spec.thickness / 2.0;
        if spec.is_horizontal {
            (0.0, half)
        } else if spec.extend_left {
            (-half, 0.0)
        } else {
            (half, 0.0)
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
                // R1141：stroke 默认以线为中心（width 两侧各半），但 border 应在 border-box
                // 内侧（同 Solid 的 border_fill_rect：从 y1/x1 向厚度方向延伸）。offset stroke
                // 中心 inward 使 dashed/dotted 与 solid 定位一致：horizontal → y += thickness/2；
                // vertical extend_left（右边框）→ x -= thickness/2；vertical else（左边框）→ x += thickness/2。
                // 旧未 offset 致 dashed/dotted border 半宽溢出 border-box（top border y=30 w5
                // 居中 y[27.5,32.5] 而非内侧 y[30,35]，position-*-root-element dashed border -3px 偏移）。
                let (dx, dy) = Self::stroke_inward_offset(spec);
                self.primitives.add_stroke(StrokePrimitive {
                    x1: spec.x1 + dx,
                    y1: spec.y1 + dy,
                    x2: spec.x2 + dx,
                    y2: spec.y2 + dy,
                    width: spec.thickness,
                    color: render_color,
                    style: LineStyle::Dotted,
                    cap: LineCap::Round,
                });
            }
            BorderStyleValue::Dashed => {
                let (dx, dy) = Self::stroke_inward_offset(spec);
                self.primitives.add_stroke(StrokePrimitive {
                    x1: spec.x1 + dx,
                    y1: spec.y1 + dy,
                    x2: spec.x2 + dx,
                    y2: spec.y2 + dy,
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
    pub(super) fn paint_outline(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        use zero_style_system::OutlineStyleValue;
        use zero_style_system::property::types::DisplayValue;

        // CSS2 §17.4：table-column / table-column-group 不生成盒（仅参与列宽计算），
        // 故 outline 不应用——同 R2108「margin 不应用 table-column/cell/column-group」
        // 谱系（display-type exclusion）。driving cluster：outline-applies-to-005/006，
        // 4 个 outline 属性（outline/width/color/style）× 2 display = 8 案全应无红边。
        // 其它 table 类型（row-group/header-group/footer-group/row/cell）生成盒，outline
        // 仍应用（outline-applies-to-001~004 PASS）。
        if matches!(
            style.display,
            DisplayValue::TableColumn | DisplayValue::TableColumnGroup
        ) {
            return;
        }

        let outline_width = length_to_f32(&style.outline_width);

        if outline_width <= 0.0 || style.outline_style == OutlineStyleValue::None {
            return;
        }

        let offset = if style.outline_offset_inset {
            // CSS-UI-4 §4.4: `outline-offset: inset` ≡ 负 outline-width 偏移，
            // 使 outline 绘制在 border-box 内侧（total_offset = outline_width + (-outline_width) = 0，
            // 既有外扩矩形几何退化为贴 border-box 边内侧绘制）。driving: outline-offset-inset-001/003/004。
            -outline_width
        } else {
            length_to_f32(&style.outline_offset)
        };

        let w = box_node.width;
        let h = box_node.height;
        let total_offset = outline_width + offset;
        // outline-color 初始 = currentColor（CSS UI：invert 无浏览器支持回落 currentColor），
        // 须按元素自身 color 解析（color_value_to_render 无元素色上下文会回落黑色）。
        let color = resolve_color_current(&style.outline_color, &style.color);

        // 计算外侧矩形坐标
        let ox = abs_x - total_offset;
        let oy = abs_y - total_offset;
        let ow = w + 2.0 * total_offset;
        let oh = h + 2.0 * total_offset;

        match style.outline_style {
            OutlineStyleValue::None => {}
            // R2379：CSS UI 4 auto 为 UA-defined 描边，ZW 按 solid 渲染（典型焦点环）。
            OutlineStyleValue::Solid | OutlineStyleValue::Auto => {
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

/// 解析 border-collapse 边框颜色覆盖。
///
/// 当 LayoutBox 有 collapsed_border_color_overrides 时，使用覆盖颜色（RGBA u32），
/// 否则回退到 ComputedStyle 中的原始颜色。
fn collapsed_border_color(override_color: &Option<u32>, original: &ColorValue) -> ColorValue {
    match override_color {
        Some(rgba) => {
            let r = ((rgba >> 24) & 0xFF) as u8;
            let g = ((rgba >> 16) & 0xFF) as u8;
            let b = ((rgba >> 8) & 0xFF) as u8;
            let a = (rgba & 0xFF) as u8;
            ColorValue::Rgba(r, g, b, a)
        }
        None => original.clone(),
    }
}

/// 把边框颜色的 `currentColor` 关键字解析为元素自身计算 `color`（CSS-Color §resolving）。
/// 非 currentColor 颜色原样返回。border-color 初始值为 currentColor，故无显式颜色的
/// `border: solid` 会使用元素文本色（与 Chromium 一致）。
fn resolve_border_current_color(color: ColorValue, current: &ColorValue) -> ColorValue {
    if color == ColorValue::CurrentColor {
        current.clone()
    } else {
        color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_border_current_color_substitutes_keyword() {
        // currentColor 解析为元素计算 color
        let green = ColorValue::Named("green".to_string());
        assert_eq!(resolve_border_current_color(ColorValue::CurrentColor, &green), green);
        // 非 currentColor 颜色原样返回，不受影响
        let red = ColorValue::Rgba(255, 0, 0, 255);
        assert_eq!(resolve_border_current_color(red.clone(), &green), red);
    }
}
