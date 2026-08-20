//! CSS 属性指示器绘制 — 交互、提示、表格、排版、容器、吸附等属性的可视化标记。
//!
//! 从 effects.rs 拆分而来，包含所有 CSS 属性指示器的 paint 方法。

use zero_css_parser::values::{ClipPathRadius, LengthValue};
use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{LineCap, LineStyle, StrokePrimitive};
use zero_style_system::{
    ClipPathComputedValue, ComputedStyle, ImageRenderingValue, IsolationValue, OverscrollBehaviorValue,
    PointerEventsValue, TouchActionValue, UserSelectValue,
};

use super::super::helpers::length_to_f32;

fn resolve_indicator_length(lv: &LengthValue, style: &ComputedStyle) -> f32 {
    match lv {
        LengthValue::Px(v) => *v as f32,
        _ => {
            let font_size = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
            zero_style_system::computed::resolve_length(lv, font_size, None, None) as f32
        }
    }
}

impl super::Painter {
    // ── CSS 交互/提示属性指示器 ──────────────────────────

    /// 绘制 CSS cursor 类型指示器。
    ///
    /// 在元素右上角渲染一个 4×4 像素的小方块，颜色根据 cursor 类型不同而变化。
    /// 仅对非 auto/default 的 cursor 值渲染指示器。
    pub(super) fn paint_cursor_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::CursorValue;

        let color = match style.cursor {
            CursorValue::Auto | CursorValue::Default => return,
            CursorValue::Pointer => Color::rgba(0, 120, 215, 200), // 蓝色 — 手指光标
            CursorValue::Text => Color::rgba(0, 0, 0, 200),        // 黑色 — 文本光标
            CursorValue::Crosshair => Color::rgba(255, 0, 0, 200), // 红色 — 十字光标
            CursorValue::Move => Color::rgba(128, 0, 128, 200),    // 紫色 — 移动光标
            CursorValue::Wait => Color::rgba(255, 165, 0, 200),    // 橙色 — 等待光标
            CursorValue::Help => Color::rgba(0, 128, 0, 200),      // 绿色 — 帮助光标
            CursorValue::NotAllowed => Color::rgba(200, 0, 0, 200), // 深红 — 禁止光标
            CursorValue::Grab | CursorValue::Grabbing => Color::rgba(139, 69, 19, 200), // 棕色 — 抓取
            CursorValue::ColResize | CursorValue::EwResize => Color::rgba(0, 128, 128, 200), // 青色 — 水平调整
            CursorValue::RowResize | CursorValue::NsResize => Color::rgba(128, 128, 0, 200), // 橄榄 — 垂直调整
            CursorValue::None => Color::rgba(200, 200, 200, 100),  // 浅灰 — 无光标
            CursorValue::Progress => Color::rgba(0, 0, 200, 200),  // 蓝色 — 进度
            CursorValue::Cell => Color::rgba(0, 200, 0, 200),      // 绿色 — 单元格
            CursorValue::Copy => Color::rgba(100, 100, 255, 200),  // 淡蓝 — 复制
            CursorValue::Alias => Color::rgba(200, 100, 0, 200),   // 深橙 — 别名
            CursorValue::AllScroll => Color::rgba(128, 128, 128, 200), // 灰色 — 全方向滚动
            CursorValue::ZoomIn | CursorValue::ZoomOut => Color::rgba(200, 200, 0, 200), // 黄色 — 缩放
        };

        // 在元素右上角绘制 4×4 指示方块
        let x = abs_x + box_node.width - 6.0;
        let y = abs_y + 2.0;
        self.primitives.add_fill(Rect::new(x, y, 4.0, 4.0), color);
    }

    /// 绘制 CSS image-rendering 质量指示器。
    ///
    /// 对非 auto 值的 image-rendering，在图片右下角绘制一个小质量标记：
    /// - pixelated → 方格图案（2×2 网格）
    /// - crisp-edges → 粗线边框
    /// - smooth/high-quality → 圆滑标记
    pub(super) fn paint_image_rendering_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        match style.image_rendering {
            ImageRenderingValue::Auto => {}
            ImageRenderingValue::Pixelated => {
                // 2×2 方格图案表示像素化
                let x = abs_x + box_node.width - 8.0;
                let y = abs_y + box_node.height - 8.0;
                let c = Color::rgba(255, 0, 255, 180);
                self.primitives.add_fill(Rect::new(x, y, 4.0, 4.0), c);
                self.primitives.add_fill(Rect::new(x + 4.0, y + 4.0, 4.0, 4.0), c);
            }
            ImageRenderingValue::CrispEdges => {
                // 粗线边框表示锐利边缘
                let x = abs_x + box_node.width - 10.0;
                let y = abs_y + box_node.height - 10.0;
                let c = Color::rgba(255, 140, 0, 180);
                self.primitives.add_fill(Rect::new(x, y, 10.0, 2.0), c);
                self.primitives.add_fill(Rect::new(x, y, 2.0, 10.0), c);
            }
            ImageRenderingValue::Smooth | ImageRenderingValue::HighQuality => {
                // 圆滑标记（单个圆点）
                let x = abs_x + box_node.width - 6.0;
                let y = abs_y + box_node.height - 6.0;
                self.primitives
                    .add_fill(Rect::new(x, y, 4.0, 4.0), Color::rgba(0, 200, 100, 180));
            }
        }
    }

    /// 绘制 CSS isolation: isolate 指示器。
    ///
    /// 在元素左上角绘制一个紫色 L 形标记，表示创建了新的堆叠上下文。
    pub(super) fn paint_isolation_indicator(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if !matches!(style.isolation, IsolationValue::Isolate) {
            return;
        }

        let c = Color::rgba(128, 0, 128, 160);
        // L 形标记：水平线 + 垂直线
        self.primitives.add_fill(Rect::new(abs_x, abs_y, 8.0, 2.0), c);
        self.primitives.add_fill(Rect::new(abs_x, abs_y, 2.0, 8.0), c);
    }

    /// 绘制 CSS will-change 指示器。
    ///
    /// 在元素右上角绘制一个黄色三角形警告标记，表示即将发生的变化。
    pub(super) fn paint_will_change_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if style.will_change.is_empty() {
            return;
        }

        let x = abs_x + box_node.width - 8.0;
        let y = abs_y + 2.0;
        let c = Color::rgba(255, 200, 0, 200);
        // 用 3 个 fill 模拟三角形标记
        self.primitives.add_fill(Rect::new(x + 3.0, y, 2.0, 2.0), c);
        self.primitives.add_fill(Rect::new(x + 2.0, y + 2.0, 4.0, 2.0), c);
        self.primitives.add_fill(Rect::new(x + 1.0, y + 4.0, 6.0, 2.0), c);
    }

    /// 绘制 CSS pointer-events: none 指示器。
    ///
    /// 在元素右上角绘制一个红色 × 标记，表示元素不接收指针事件。
    pub(super) fn paint_pointer_events_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if !matches!(style.pointer_events, PointerEventsValue::None) {
            return;
        }

        let x = abs_x + box_node.width - 8.0;
        let y = abs_y + 2.0;
        let c = Color::rgba(220, 20, 20, 180);
        // × 标记：两条交叉对角线（用 stroke）
        self.primitives.add_stroke(StrokePrimitive {
            x1: x,
            y1: y,
            x2: x + 6.0,
            y2: y + 6.0,
            width: 1.5,
            color: c,
            style: LineStyle::Solid,
            cap: LineCap::Square,
        });
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 6.0,
            y1: y,
            x2: x,
            y2: y + 6.0,
            width: 1.5,
            color: c,
            style: LineStyle::Solid,
            cap: LineCap::Square,
        });
    }

    /// 绘制 CSS user-select: none 指示器。
    ///
    /// 在元素左上角绘制一个灰色锁形标记，表示文本不可选择。
    pub(super) fn paint_user_select_indicator(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        if !matches!(style.user_select, UserSelectValue::None) {
            return;
        }

        let x = abs_x + 2.0;
        let y = abs_y + 2.0;
        let c = Color::rgba(128, 128, 128, 180);
        // 锁形标记：矩形锁体 + 半弧锁扣
        self.primitives.add_fill(Rect::new(x, y + 4.0, 6.0, 4.0), c);
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 1.5,
            y1: y + 4.0,
            x2: x + 1.5,
            y2: y + 1.0,
            width: 1.0,
            color: c,
            style: LineStyle::Solid,
            cap: LineCap::Round,
        });
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 4.5,
            y1: y + 4.0,
            x2: x + 4.5,
            y2: y + 1.0,
            width: 1.0,
            color: c,
            style: LineStyle::Solid,
            cap: LineCap::Round,
        });
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 1.5,
            y1: y + 1.0,
            x2: x + 4.5,
            y2: y + 1.0,
            width: 1.0,
            color: c,
            style: LineStyle::Solid,
            cap: LineCap::Round,
        });
    }

    /// 绘制 CSS overscroll-behavior 指示器。
    ///
    /// 对 contain/none 值，在元素底部中央绘制一条水平线，表示滚动边界被限制。
    pub(super) fn paint_overscroll_behavior_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let (c, w) = match style.overscroll_behavior_x {
            OverscrollBehaviorValue::Contain => (Color::rgba(255, 100, 0, 180), 12.0),
            OverscrollBehaviorValue::None => (Color::rgba(200, 0, 0, 200), 16.0),
            OverscrollBehaviorValue::Auto => return,
        };

        let x = abs_x + (box_node.width - w) / 2.0;
        let y = abs_y + box_node.height - 3.0;
        self.primitives.add_fill(Rect::new(x, y, w, 2.0), c);
    }

    /// 绘制 CSS touch-action 指示器。
    ///
    /// 对非 auto 值，在元素右下角绘制一个小标记。
    pub(super) fn paint_touch_action_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let c = match style.touch_action {
            TouchActionValue::Auto | TouchActionValue::Manipulation => return,
            TouchActionValue::None => Color::rgba(200, 0, 0, 180),
            TouchActionValue::PanX => Color::rgba(0, 100, 200, 180),
            TouchActionValue::PanY => Color::rgba(0, 200, 100, 180),
            TouchActionValue::PanXPanY => Color::rgba(100, 100, 200, 180),
        };

        let x = abs_x + box_node.width - 5.0;
        let y = abs_y + box_node.height - 5.0;
        self.primitives.add_fill(Rect::new(x, y, 3.0, 3.0), c);
    }

    /// CSS scroll-snap 视觉指示器 — 渲染吸附轴和对齐点标记。
    pub(super) fn paint_scroll_snap_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::property::types::{ScrollSnapAlign, ScrollSnapAxis, ScrollSnapStrictness};

        let snap_type = &style.scroll_snap_type;
        if matches!(snap_type.strictness, ScrollSnapStrictness::None) {
            return;
        }

        // 严格度颜色：mandatory 红色，proximity 橙色
        let color = match snap_type.strictness {
            ScrollSnapStrictness::Mandatory => Color::rgba(230, 51, 51, 179),
            ScrollSnapStrictness::Proximity => Color::rgba(230, 153, 51, 179),
            _ => return,
        };

        // 绘制吸附轴标记线
        match snap_type.axis {
            ScrollSnapAxis::X => {
                let y = abs_y + box_node.height - 2.0;
                self.primitives
                    .add_fill(Rect::new(abs_x, y, box_node.width, 2.0), color);
            }
            ScrollSnapAxis::Y => {
                let x = abs_x + box_node.width - 2.0;
                self.primitives
                    .add_fill(Rect::new(x, abs_y, 2.0, box_node.height), color);
            }
            ScrollSnapAxis::Both => {
                let y = abs_y + box_node.height - 2.0;
                self.primitives
                    .add_fill(Rect::new(abs_x, y, box_node.width, 2.0), color);
                let x = abs_x + box_node.width - 2.0;
                self.primitives
                    .add_fill(Rect::new(x, abs_y, 2.0, box_node.height), color);
            }
        }

        // scroll-snap-align 对齐点指示
        match style.scroll_snap_align {
            ScrollSnapAlign::Start => {
                self.primitives
                    .add_fill(Rect::new(abs_x, abs_y, 4.0, 4.0), Color::rgba(51, 179, 230, 204));
            }
            ScrollSnapAlign::Center => {
                let cx = abs_x + box_node.width / 2.0 - 2.0;
                let cy = abs_y + box_node.height / 2.0 - 2.0;
                self.primitives
                    .add_fill(Rect::new(cx, cy, 4.0, 4.0), Color::rgba(51, 230, 128, 204));
            }
            ScrollSnapAlign::End => {
                self.primitives.add_fill(
                    Rect::new(abs_x + box_node.width - 4.0, abs_y + box_node.height - 4.0, 4.0, 4.0),
                    Color::rgba(230, 128, 51, 204),
                );
            }
            ScrollSnapAlign::None => {}
        }
    }

    /// CSS perspective 渲染 — 为子元素创建 3D 透视上下文。
    pub(super) fn paint_perspective_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let perspective = resolve_indicator_length(&style.perspective, style);
        if perspective <= 0.0 {
            return;
        }

        let origin_x = match &style.perspective_origin_x {
            LengthValue::Percentage(p) => abs_x + box_node.width * (*p as f32 / 100.0),
            lv => abs_x + resolve_indicator_length(lv, style),
        };
        let origin_y = match &style.perspective_origin_y {
            LengthValue::Percentage(p) => abs_y + box_node.height * (*p as f32 / 100.0),
            lv => abs_y + resolve_indicator_length(lv, style),
        };

        let vanish_color = Color::rgba(77, 128, 230, 204);
        let cross_size: f32 = 6.0;

        // 水平线
        self.primitives.add_fill(
            Rect::new(origin_x - cross_size / 2.0, origin_y - 0.5, cross_size, 1.0),
            vanish_color,
        );
        // 垂直线
        self.primitives.add_fill(
            Rect::new(origin_x - 0.5, origin_y - cross_size / 2.0, 1.0, cross_size),
            vanish_color,
        );

        // 在消失点周围渲染深度环
        let depth = perspective.min(50.0);
        let ring_color = Color::rgba(77, 128, 230, 102);
        for i in 0..4u32 {
            let angle = std::f32::consts::FRAC_PI_2 * i as f32;
            let dx = depth * angle.cos();
            let dy = depth * angle.sin();
            self.primitives.add_fill(
                Rect::new(origin_x + dx - 1.0, origin_y + dy - 1.0, 2.0, 2.0),
                ring_color,
            );
        }
        let marker_color = Color::rgba(77, 128, 230, 77);
        self.primitives
            .add_fill(Rect::new(abs_x, abs_y + box_node.height - 3.0, 12.0, 3.0), marker_color);
    }

    /// CSS backface-visibility: hidden 指示器 — 标记元素的背面不可见。
    pub(super) fn paint_backface_visibility_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        _style: &ComputedStyle,
    ) {
        let dash_len = 4.0_f32;
        let gap_len = 3.0_f32;
        let thickness = 1.0_f32;
        let color = Color::rgba(179, 77, 179, 153); // 紫色虚线

        // 顶部边框虚线
        let mut x = abs_x;
        while x < abs_x + box_node.width {
            let w = dash_len.min(abs_x + box_node.width - x);
            self.primitives.add_fill(Rect::new(x, abs_y, w, thickness), color);
            x += dash_len + gap_len;
        }

        // 底部边框虚线
        x = abs_x;
        let bottom_y = abs_y + box_node.height - thickness;
        while x < abs_x + box_node.width {
            let w = dash_len.min(abs_x + box_node.width - x);
            self.primitives.add_fill(Rect::new(x, bottom_y, w, thickness), color);
            x += dash_len + gap_len;
        }

        // 左侧边框虚线
        let mut y = abs_y;
        while y < abs_y + box_node.height {
            let h = dash_len.min(abs_y + box_node.height - y);
            self.primitives.add_fill(Rect::new(abs_x, y, thickness, h), color);
            y += dash_len + gap_len;
        }

        // 右侧边框虚线
        y = abs_y;
        let right_x = abs_x + box_node.width - thickness;
        while y < abs_y + box_node.height {
            let h = dash_len.min(abs_y + box_node.height - y);
            self.primitives.add_fill(Rect::new(right_x, y, thickness, h), color);
            y += dash_len + gap_len;
        }
    }

    /// CSS transform-style: preserve-3d 指示器 — 标记 3D 渲染上下文。
    pub(super) fn paint_transform_style_indicator(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        _style: &ComputedStyle,
    ) {
        let ox = abs_x + 2.0;
        let oy = abs_y + 2.0;
        let s = 5.0_f32;
        let d = 3.0_f32;

        // 正面（蓝色）
        self.primitives
            .add_fill(Rect::new(ox, oy, s, s), Color::rgba(51, 128, 204, 179));
        // 顶面（深蓝）
        self.primitives
            .add_fill(Rect::new(ox + d, oy - d, s, d), Color::rgba(38, 89, 166, 179));
        // 右面（更深蓝）
        self.primitives
            .add_fill(Rect::new(ox + s, oy, d, s), Color::rgba(26, 64, 128, 179));
    }

    /// CSS border-spacing 渲染 — 显示表格单元格间距。
    pub(super) fn paint_border_spacing_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let h_spacing = style.border_spacing.horizontal;
        let v_spacing = style.border_spacing.vertical;

        if h_spacing <= 0.0 && v_spacing <= 0.0 {
            return;
        }

        let color = Color::rgba(153, 153, 153, 102);
        let cx = abs_x + box_node.border_left;
        let cy = abs_y + box_node.border_top;

        if h_spacing > 0.0 {
            self.primitives
                .add_fill(Rect::new(cx, cy, h_spacing.min(box_node.content_width), 1.0), color);
        }
        if v_spacing > 0.0 {
            self.primitives
                .add_fill(Rect::new(cx, cy, 1.0, v_spacing.min(box_node.content_height)), color);
        }
    }

    /// CSS caption-side 渲染指示器 — 标记表格标题位置。
    pub(super) fn paint_caption_side_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::property::types::CaptionSideValue;

        let bar_h = 3.0_f32;
        let bar_w = box_node.width.min(20.0);

        match style.caption_side {
            CaptionSideValue::Top => {
                self.primitives.add_fill(
                    Rect::new(abs_x, abs_y - bar_h - 1.0, bar_w, bar_h),
                    Color::rgba(77, 179, 128, 153),
                );
            }
            CaptionSideValue::Bottom => {
                self.primitives.add_fill(
                    Rect::new(abs_x, abs_y + box_node.height + 1.0, bar_w, bar_h),
                    Color::rgba(128, 77, 179, 153),
                );
            }
        }
    }

    /// 绘制 CSS clip-path 视觉指示器。
    ///
    /// 为非 none 的 clip-path 渲染指示性图元：
    /// - inset()：在裁剪区域内绘制虚线边框
    /// - circle()/ellipse()：绘制圆/椭圆轮廓线
    /// - polygon()：绘制多边形轮廓线
    pub(super) fn paint_clip_path(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        let clip = &style.clip_path;
        if matches!(clip, ClipPathComputedValue::None) {
            return;
        }

        let w = box_node.width;
        let h = box_node.height;

        match clip {
            ClipPathComputedValue::Inset {
                top,
                right,
                bottom,
                left,
                ..
            } => {
                let t = length_to_f32(top);
                let r = length_to_f32(right);
                let b = length_to_f32(bottom);
                let l = length_to_f32(left);
                let clip_x = abs_x + l;
                let clip_y = abs_y + t;
                let clip_w = w - l - r;
                let clip_h = h - t - b;
                if clip_w > 0.0 && clip_h > 0.0 {
                    let color = Color::rgba(128, 0, 128, 100);
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: clip_x,
                        y1: clip_y,
                        x2: clip_x + clip_w,
                        y2: clip_y,
                        width: 1.0,
                        color,
                        style: LineStyle::Dashed,
                        cap: LineCap::Square,
                    });
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: clip_x + clip_w,
                        y1: clip_y,
                        x2: clip_x + clip_w,
                        y2: clip_y + clip_h,
                        width: 1.0,
                        color,
                        style: LineStyle::Dashed,
                        cap: LineCap::Square,
                    });
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: clip_x + clip_w,
                        y1: clip_y + clip_h,
                        x2: clip_x,
                        y2: clip_y + clip_h,
                        width: 1.0,
                        color,
                        style: LineStyle::Dashed,
                        cap: LineCap::Square,
                    });
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: clip_x,
                        y1: clip_y + clip_h,
                        x2: clip_x,
                        y2: clip_y,
                        width: 1.0,
                        color,
                        style: LineStyle::Dashed,
                        cap: LineCap::Square,
                    });
                }
            }
            ClipPathComputedValue::Circle { radius, position } => {
                let r = match radius {
                    ClipPathRadius::Length(l) => length_to_f32(l),
                    ClipPathRadius::ClosestSide | ClipPathRadius::FarthestSide => w.min(h) / 2.0,
                };
                let cx = position.as_ref().map(|(x, _)| length_to_f32(x)).unwrap_or(w / 2.0);
                let cy = position.as_ref().map(|(_, y)| length_to_f32(y)).unwrap_or(h / 2.0);
                let color = Color::rgba(0, 128, 128, 100);
                let segs = 12;
                for i in 0..segs {
                    let a1 = (i as f32 / segs as f32) * 2.0 * std::f32::consts::PI;
                    let a2 = ((i + 1) as f32 / segs as f32) * 2.0 * std::f32::consts::PI;
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: abs_x + cx + r * a1.cos(),
                        y1: abs_y + cy + r * a1.sin(),
                        x2: abs_x + cx + r * a2.cos(),
                        y2: abs_y + cy + r * a2.sin(),
                        width: 1.0,
                        color,
                        style: LineStyle::Dotted,
                        cap: LineCap::Round,
                    });
                }
            }
            ClipPathComputedValue::Ellipse { rx, ry, position } => {
                let rx_v = match rx {
                    ClipPathRadius::Length(l) => length_to_f32(l),
                    ClipPathRadius::ClosestSide | ClipPathRadius::FarthestSide => w / 2.0,
                };
                let ry_v = match ry {
                    ClipPathRadius::Length(l) => length_to_f32(l),
                    ClipPathRadius::ClosestSide | ClipPathRadius::FarthestSide => h / 2.0,
                };
                let cx = position.as_ref().map(|(x, _)| length_to_f32(x)).unwrap_or(w / 2.0);
                let cy = position.as_ref().map(|(_, y)| length_to_f32(y)).unwrap_or(h / 2.0);
                let color = Color::rgba(128, 128, 0, 100);
                let segs = 12;
                for i in 0..segs {
                    let a1 = (i as f32 / segs as f32) * 2.0 * std::f32::consts::PI;
                    let a2 = ((i + 1) as f32 / segs as f32) * 2.0 * std::f32::consts::PI;
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: abs_x + cx + rx_v * a1.cos(),
                        y1: abs_y + cy + ry_v * a1.sin(),
                        x2: abs_x + cx + rx_v * a2.cos(),
                        y2: abs_y + cy + ry_v * a2.sin(),
                        width: 1.0,
                        color,
                        style: LineStyle::Dotted,
                        cap: LineCap::Round,
                    });
                }
            }
            ClipPathComputedValue::Polygon { points, .. } => {
                if points.len() < 2 {
                    return;
                }
                let color = Color::rgba(0, 128, 0, 100);
                for i in 0..points.len() {
                    let (x1, y1) = &points[i];
                    let (x2, y2) = &points[(i + 1) % points.len()];
                    self.primitives.add_stroke(StrokePrimitive {
                        x1: abs_x + length_to_f32(x1),
                        y1: abs_y + length_to_f32(y1),
                        x2: abs_x + length_to_f32(x2),
                        y2: abs_y + length_to_f32(y2),
                        width: 1.0,
                        color,
                        style: LineStyle::Dashed,
                        cap: LineCap::Square,
                    });
                }
            }
            _ => {}
        }
    }

    /// 绘制 CSS direction 属性指示器。
    ///
    /// direction:rtl 时在左上角绘制红色 → 箭头（表示从右到左的文本方向）。
    pub(super) fn paint_direction_indicator(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::DirectionValue;
        if matches!(style.direction, DirectionValue::Ltr) {
            return;
        }
        // direction:rtl — 红色左箭头 ←
        let x = abs_x;
        let y = abs_y;
        let color = Color::rgba(220, 50, 50, 180);
        // 箭头主线 ←
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 12.0,
            y1: y + 6.0,
            x2: x + 2.0,
            y2: y + 6.0,
            width: 2.0,
            color,
            style: LineStyle::Solid,
            cap: LineCap::Square,
        });
        // 箭头头部 ∧
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 5.0,
            y1: y + 3.0,
            x2: x + 2.0,
            y2: y + 6.0,
            width: 2.0,
            color,
            style: LineStyle::Solid,
            cap: LineCap::Square,
        });
        // 箭头头部 ∨
        self.primitives.add_stroke(StrokePrimitive {
            x1: x + 5.0,
            y1: y + 9.0,
            x2: x + 2.0,
            y2: y + 6.0,
            width: 2.0,
            color,
            style: LineStyle::Solid,
            cap: LineCap::Square,
        });
        // RTL 标记小方块
        self.primitives.add_fill(Rect::new(x + 14.0, y + 2.0, 6.0, 8.0), color);
    }

    /// 绘制 CSS tab-size 属性指示器。
    ///
    /// 非 8（默认值）时在右上角绘制青色等宽方块表示制表符宽度。
    pub(super) fn paint_tab_size_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::TabSizeValue;
        let w = box_node.width;
        let tab_count = match &style.tab_size {
            TabSizeValue::Number(n) => *n,
            TabSizeValue::Length(l) => {
                let px = length_to_f32(l);
                if px > 0.0 { (px / 8.0).round() as u32 } else { return }
            }
        };
        // 默认值 8 不渲染
        if tab_count == 8 || tab_count == 0 {
            return;
        }
        let color = Color::rgba(0, 180, 180, 160);
        let start_x = abs_x + w - 6.0 * tab_count.min(6) as f32 - 4.0;
        let y = abs_y + 2.0;
        let count = tab_count.min(6);
        for i in 0..count {
            self.primitives
                .add_fill(Rect::new(start_x + i as f32 * 6.0, y, 4.0, 4.0), color);
        }
    }

    /// 绘制 CSS border-collapse 属性指示器。
    ///
    /// collapse 时在右下角绘制橙色双线边框标记（表示合并边框模型）。
    pub(super) fn paint_border_collapse_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::BorderCollapseValue;
        if matches!(style.border_collapse, BorderCollapseValue::Separate) {
            return;
        }
        let w = box_node.width;
        let h = box_node.height;
        let color = Color::rgba(255, 165, 0, 180);
        // 双线 — 外线
        self.primitives.add_stroke(StrokePrimitive {
            x1: abs_x + w - 14.0,
            y1: abs_y + h - 2.0,
            x2: abs_x + w - 2.0,
            y2: abs_y + h - 2.0,
            width: 1.0,
            color,
            style: LineStyle::Solid,
            cap: LineCap::Square,
        });
        // 双线 — 内线
        self.primitives.add_stroke(StrokePrimitive {
            x1: abs_x + w - 14.0,
            y1: abs_y + h - 5.0,
            x2: abs_x + w - 2.0,
            y2: abs_y + h - 5.0,
            width: 1.0,
            color,
            style: LineStyle::Solid,
            cap: LineCap::Square,
        });
    }

    /// 绘制 CSS table-layout 属性指示器。
    ///
    /// fixed 时在右上角绘制蓝色网格图标。
    pub(super) fn paint_table_layout_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::TableLayoutValue;
        if matches!(style.table_layout, TableLayoutValue::Auto) {
            return;
        }
        let w = box_node.width;
        let x = abs_x + w - 16.0;
        let y = abs_y + 2.0;
        let color = Color::rgba(50, 120, 220, 180);
        // 网格外框
        self.primitives.add_fill(Rect::new(x, y, 12.0, 10.0), color);
        // 网格分割线（用背景色填充两个竖条模拟）
        let bg = Color::rgba(255, 255, 255, 200);
        self.primitives.add_fill(Rect::new(x + 3.0, y, 1.0, 10.0), bg);
        self.primitives.add_fill(Rect::new(x + 7.0, y, 1.0, 10.0), bg);
        // 水平分割线
        self.primitives.add_fill(Rect::new(x, y + 4.0, 12.0, 1.0), bg);
    }

    /// 绘制 CSS font-variant-numeric 属性指示器。
    ///
    /// 非 normal 值时在左下角绘制对应样式的数字标记。
    pub(super) fn paint_font_variant_numeric_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::FontVariantNumericValue;
        let h = box_node.height;
        let (color, _variant) = match &style.font_variant_numeric {
            FontVariantNumericValue::Normal => return,
            FontVariantNumericValue::Ordinal => (Color::rgba(100, 80, 200, 180), "ord"),
            FontVariantNumericValue::SlashedZero => (Color::rgba(200, 80, 100, 180), "0/"),
            FontVariantNumericValue::LiningNums => (Color::rgba(80, 160, 80, 180), "ln"),
            FontVariantNumericValue::OldstyleNums => (Color::rgba(160, 120, 40, 180), "on"),
            FontVariantNumericValue::ProportionalNums => (Color::rgba(80, 120, 200, 180), "pm"),
            FontVariantNumericValue::TabularNums => (Color::rgba(200, 120, 80, 180), "tm"),
            FontVariantNumericValue::DiagonalFractions => (Color::rgba(180, 60, 160, 180), "df"),
            FontVariantNumericValue::StackedFractions => (Color::rgba(60, 160, 180, 180), "sf"),
        };
        // 在左下角绘制小标记
        let x = abs_x + 2.0;
        let y = abs_y + h - 12.0;
        // 标记背景
        self.primitives
            .add_fill(Rect::new(x, y, 20.0, 10.0), Color::rgba(240, 240, 240, 200));
        // 标记方块（颜色区分变体类型）
        self.primitives.add_fill(Rect::new(x + 2.0, y + 2.0, 6.0, 6.0), color);
    }

    /// 绘制 CSS contain 属性指示器。
    ///
    /// 非 none 值时在右上角绘制包含标记（不同包含类型用不同颜色表示）。
    pub(super) fn paint_contain_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::ContainComputedValue;
        let (color, label) = match &style.contain {
            ContainComputedValue::None => return,
            ContainComputedValue::Strict => (Color::rgba(220, 40, 40, 200), "S"),
            ContainComputedValue::Content => (Color::rgba(40, 160, 40, 200), "C"),
            ContainComputedValue::Size => (Color::rgba(40, 80, 200, 200), "Sz"),
            ContainComputedValue::Layout => (Color::rgba(200, 120, 40, 200), "L"),
            ContainComputedValue::Style => (Color::rgba(160, 40, 160, 200), "St"),
            ContainComputedValue::Paint => (Color::rgba(40, 180, 180, 200), "P"),
            ContainComputedValue::Custom(_) => (Color::rgba(120, 120, 120, 200), "M"),
        };
        let w = box_node.width;
        let x = abs_x + w - 16.0;
        let y = abs_y + 2.0;
        // 背景框
        self.primitives
            .add_fill(Rect::new(x, y, 14.0, 10.0), Color::rgba(240, 240, 240, 200));
        // 颜色标记方块
        self.primitives.add_fill(Rect::new(x + 1.0, y + 1.0, 12.0, 8.0), color);
        // 带有虚线边框表示"包含"
        let border = Color::rgba(60, 60, 60, 180);
        self.primitives.add_fill(Rect::new(x, y, 14.0, 1.0), border);
        self.primitives.add_fill(Rect::new(x, y + 9.0, 14.0, 1.0), border);
        let _ = label;
    }

    /// 绘制 CSS unicode-bidi 属性指示器。
    ///
    /// 非 normal 值时在左侧绘制双向文本覆盖标记。
    pub(super) fn paint_unicode_bidi_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::UnicodeBidiValue;
        let color = match &style.unicode_bidi {
            UnicodeBidiValue::Normal => return,
            UnicodeBidiValue::Embed => Color::rgba(80, 140, 220, 200),
            UnicodeBidiValue::Isolate => Color::rgba(140, 80, 220, 200),
            UnicodeBidiValue::BidiOverride => Color::rgba(220, 60, 60, 200),
            UnicodeBidiValue::IsolateOverride => Color::rgba(220, 100, 60, 200),
            UnicodeBidiValue::Plaintext => Color::rgba(60, 180, 120, 200),
        };
        let h = box_node.height;
        let x = abs_x - 4.0;
        let y = abs_y;
        // 垂直条标记（表示双向文本覆盖）
        self.primitives.add_fill(Rect::new(x, y, 3.0, h), color);
        // 顶部三角标记
        self.primitives.add_fill(Rect::new(x - 2.0, y, 2.0, 4.0), color);
    }

    /// 绘制 CSS box-decoration-break 属性指示器。
    ///
    /// clone 值时在元素右侧绘制克隆标记（slice 为默认不渲染）。
    pub(super) fn paint_box_decoration_break_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::BoxDecorationBreakValue;
        if matches!(style.box_decoration_break, BoxDecorationBreakValue::Slice) {
            return;
        }
        let w = box_node.width;
        let h = box_node.height;
        let x = abs_x + w - 8.0;
        let y = abs_y + h - 8.0;
        let color = Color::rgba(100, 160, 60, 200);
        // 克隆标记：两个重叠的小方块
        self.primitives
            .add_fill(Rect::new(x, y, 6.0, 6.0), Color::rgba(240, 240, 240, 200));
        self.primitives.add_fill(Rect::new(x + 1.0, y + 1.0, 5.0, 5.0), color);
        self.primitives.add_fill(Rect::new(x + 2.0, y + 2.0, 5.0, 5.0), color);
    }

    /// 绘制 CSS overflow-wrap 属性指示器。
    ///
    /// break-word 或 anywhere 时在右下角绘制断词标记。
    pub(super) fn paint_overflow_wrap_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::OverflowWrapValue;
        let color = match &style.overflow_wrap {
            OverflowWrapValue::Normal => return,
            OverflowWrapValue::BreakWord => Color::rgba(220, 120, 40, 200),
            OverflowWrapValue::Anywhere => Color::rgba(180, 60, 180, 200),
        };
        let w = box_node.width;
        let h = box_node.height;
        let x = abs_x + w - 10.0;
        let y = abs_y + h - 8.0;
        // 断词标记：折线（模拟文字断开效果）
        self.primitives.add_fill(Rect::new(x, y, 8.0, 1.0), color);
        self.primitives.add_fill(Rect::new(x, y + 3.0, 4.0, 1.0), color);
        self.primitives.add_fill(Rect::new(x + 4.0, y + 6.0, 4.0, 1.0), color);
    }

    /// 绘制 CSS text-align-last 属性指示器。
    ///
    /// 非 auto 值时在右下角绘制末行对齐标记。
    pub(super) fn paint_text_align_last_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::TextAlignLastValue;
        let (color, lines) = match &style.text_align_last {
            TextAlignLastValue::Auto => return,
            TextAlignLastValue::Start | TextAlignLastValue::Left => (Color::rgba(80, 140, 220, 200), 1),
            TextAlignLastValue::End | TextAlignLastValue::Right => (Color::rgba(220, 80, 80, 200), 2),
            TextAlignLastValue::Center => (Color::rgba(80, 180, 80, 200), 3),
            TextAlignLastValue::Justify => (Color::rgba(180, 140, 40, 200), 4),
        };
        let w = box_node.width;
        let h = box_node.height;
        let x = abs_x + w - 14.0;
        let y = abs_y + h - 8.0;
        // 根据对齐类型绘制不同数量的横线
        for i in 0..lines {
            let lw = match i {
                0 => 10.0,
                1 => 7.0,
                2 => 5.0,
                _ => 4.0,
            };
            self.primitives
                .add_fill(Rect::new(x, y + (i as f32) * 2.0, lw, 1.0), color);
        }
    }

    /// 绘制 CSS break-before/after/inside 属性指示器。
    ///
    /// 非 auto 值时在元素边缘绘制断点标记。
    pub(super) fn paint_break_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::{BreakInsideValue, BreakValue, PageBreakValue};
        let w = box_node.width;
        let h = box_node.height;

        // break-before / page-break-before
        if !matches!(style.break_before, BreakValue::Auto) || !matches!(style.page_break_before, PageBreakValue::Auto) {
            let color = Color::rgba(200, 60, 60, 200);
            // 顶部断点标记：双横线
            self.primitives.add_fill(Rect::new(abs_x, abs_y, w, 1.0), color);
            self.primitives.add_fill(Rect::new(abs_x, abs_y + 2.0, w, 1.0), color);
        }

        // break-after / page-break-after
        if !matches!(style.break_after, BreakValue::Auto) || !matches!(style.page_break_after, PageBreakValue::Auto) {
            let color = Color::rgba(60, 60, 200, 200);
            // 底部断点标记：双横线
            self.primitives
                .add_fill(Rect::new(abs_x, abs_y + h - 3.0, w, 1.0), color);
            self.primitives
                .add_fill(Rect::new(abs_x, abs_y + h - 1.0, w, 1.0), color);
        }

        // break-inside / page-break-inside
        if !matches!(style.break_inside, BreakInsideValue::Auto)
            || !matches!(style.page_break_inside, PageBreakValue::Auto)
        {
            let color = Color::rgba(200, 160, 40, 200);
            // 内部断点标记：四周虚线框
            self.primitives.add_fill(Rect::new(abs_x, abs_y, w, 1.0), color);
            self.primitives
                .add_fill(Rect::new(abs_x, abs_y + h - 1.0, w, 1.0), color);
            self.primitives.add_fill(Rect::new(abs_x, abs_y, 1.0, h), color);
            self.primitives
                .add_fill(Rect::new(abs_x + w - 1.0, abs_y, 1.0, h), color);
        }
    }

    /// 绘制 CSS scroll-margin / scroll-padding 属性指示器。
    ///
    /// 非零值时在元素周围绘制滚动吸附区域标记。
    pub(super) fn paint_scroll_area_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::ScrollPadding;

        // scroll-margin — 红色虚线边框
        let sm_t = style.scroll_margin_top;
        let sm_r = style.scroll_margin_right;
        let sm_b = style.scroll_margin_bottom;
        let sm_l = style.scroll_margin_left;
        if sm_t > 0.0 || sm_r > 0.0 || sm_b > 0.0 || sm_l > 0.0 {
            let color = Color::rgba(220, 80, 80, 120);
            let x = abs_x - sm_l;
            let y = abs_y - sm_t;
            let w = box_node.width + sm_l + sm_r;
            let h = box_node.height + sm_t + sm_b;
            self.primitives.add_fill(Rect::new(x, y, w, 1.0), color);
            self.primitives.add_fill(Rect::new(x, y + h - 1.0, w, 1.0), color);
            self.primitives.add_fill(Rect::new(x, y, 1.0, h), color);
            self.primitives.add_fill(Rect::new(x + w - 1.0, y, 1.0, h), color);
        }

        // scroll-padding — 蓝色虚线边框
        let sp_t = match &style.scroll_padding_top {
            ScrollPadding::Length(v) => *v,
            ScrollPadding::Auto => 0.0,
        };
        let sp_r = match &style.scroll_padding_right {
            ScrollPadding::Length(v) => *v,
            ScrollPadding::Auto => 0.0,
        };
        let sp_b = match &style.scroll_padding_bottom {
            ScrollPadding::Length(v) => *v,
            ScrollPadding::Auto => 0.0,
        };
        let sp_l = match &style.scroll_padding_left {
            ScrollPadding::Length(v) => *v,
            ScrollPadding::Auto => 0.0,
        };
        if sp_t > 0.0 || sp_r > 0.0 || sp_b > 0.0 || sp_l > 0.0 {
            let color = Color::rgba(80, 120, 220, 120);
            let x = abs_x + sp_l;
            let y = abs_y + sp_t;
            let w = box_node.width - sp_l - sp_r;
            let h = box_node.height - sp_t - sp_b;
            if w > 0.0 && h > 0.0 {
                self.primitives.add_fill(Rect::new(x, y, w, 1.0), color);
                self.primitives.add_fill(Rect::new(x, y + h - 1.0, w, 1.0), color);
                self.primitives.add_fill(Rect::new(x, y, 1.0, h), color);
                self.primitives.add_fill(Rect::new(x + w - 1.0, y, 1.0, h), color);
            }
        }
    }

    /// 绘制 CSS scroll-snap-stop 属性指示器。
    ///
    /// always 值时在吸附轴位置绘制强制停止标记。
    pub(super) fn paint_scroll_snap_stop_indicator(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::ScrollSnapStop;
        if matches!(style.scroll_snap_stop, ScrollSnapStop::Normal) {
            return;
        }
        let color = Color::rgba(220, 40, 40, 200);
        let w = box_node.width;
        let h = box_node.height;
        // 强制停止标记：中心红色方块 + 十字线
        let cx = abs_x + w / 2.0;
        let cy = abs_y + h / 2.0;
        self.primitives.add_fill(Rect::new(cx - 3.0, cy - 3.0, 6.0, 6.0), color);
        self.primitives.add_fill(Rect::new(cx - 8.0, cy, 16.0, 1.0), color);
        self.primitives.add_fill(Rect::new(cx, cy - 8.0, 1.0, 16.0), color);
    }

    /// 绘制 CSS container-type 属性指示器。
    ///
    /// 非 normal 值时在左上角绘制容器查询上下文标记。
    pub(super) fn paint_container_type_indicator(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        use zero_style_system::ContainerType;
        let (color, tag) = match &style.container_type {
            ContainerType::Normal => return,
            ContainerType::Size => (Color::rgba(60, 140, 220, 200), "S"),
            ContainerType::InlineSize => (Color::rgba(140, 60, 200, 200), "I"),
        };
        let x = abs_x + 2.0;
        let y = abs_y + 2.0;
        // 容器标记：方块 + 标签背景
        self.primitives
            .add_fill(Rect::new(x, y, 12.0, 8.0), Color::rgba(240, 240, 240, 200));
        self.primitives.add_fill(Rect::new(x + 1.0, y + 1.0, 10.0, 6.0), color);
        // container-name 存在时额外标记
        if style.container_name.is_some() {
            let name_color = Color::rgba(200, 160, 40, 200);
            self.primitives.add_fill(Rect::new(x + 12.0, y, 4.0, 4.0), name_color);
        }
        let _ = tag;
    }
}

/// 裁剪单个 tile 到 origin 区域，返回裁剪后的 (x, y, w, h)。
#[allow(clippy::too_many_arguments)]
///
/// 如果 tile 完全在 origin 外返回 None。
pub(super) fn clip_tile_to_origin(
    tile_x: f32,
    tile_y: f32,
    tile_w: f32,
    tile_h: f32,
    origin_x: f32,
    origin_y: f32,
    origin_w: f32,
    origin_h: f32,
) -> Option<(f32, f32, f32, f32)> {
    let tile_right = tile_x + tile_w;
    let tile_bottom = tile_y + tile_h;
    let origin_right = origin_x + origin_w;
    let origin_bottom = origin_y + origin_h;

    // 完全在区域外
    if tile_right <= origin_x || tile_x >= origin_right || tile_bottom <= origin_y || tile_y >= origin_bottom {
        return None;
    }

    let cx = tile_x.max(origin_x);
    let cy = tile_y.max(origin_y);
    let cw = tile_right.min(origin_right) - cx;
    let ch = tile_bottom.min(origin_bottom) - cy;

    if cw > 0.0 && ch > 0.0 {
        Some((cx, cy, cw, ch))
    } else {
        None
    }
}
