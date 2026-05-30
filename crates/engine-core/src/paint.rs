//! 绘制命令生成 — 将布局盒树转换为渲染图元。

use std::collections::HashMap;

use zero_css_parser::values::ColorValue;
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::RenderPrimitives;
use zero_style_system::ComputedStyle;

/// 绘制命令生成器 — 将布局盒树转换为渲染图元。
pub struct Painter {
    /// 生成的渲染图元列表。
    primitives: RenderPrimitives,
}

impl Painter {
    /// 创建新的绘制命令生成器。
    pub fn new() -> Self {
        Self {
            primitives: RenderPrimitives::new(),
        }
    }

    /// 绘制整个布局树。
    ///
    /// 遍历 LayoutBox 树，为每个有样式的节点生成背景和边框填充图元。
    pub fn paint(&mut self, layout: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
        self.paint_node(layout, styles, 0.0, 0.0);
    }

    /// 绘制单个节点（递归）。
    ///
    /// 根据节点的计算样式生成背景色填充和边框填充图元，
    /// 然后递归绘制子节点。
    fn paint_node(
        &mut self,
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
        offset_x: f32,
        offset_y: f32,
    ) {
        let abs_x = offset_x + box_node.x;
        let abs_y = offset_y + box_node.y;

        // 获取该节点对应的计算样式
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
        {
            // 1. 背景色填充
            if style.background_color != ColorValue::Transparent {
                self.primitives.add_fill(
                    Rect::new(abs_x, abs_y, box_node.width, box_node.height),
                    color_value_to_render(&style.background_color),
                );
            }

            // 2. 边框填充（4 个矩形：上/右/下/左）
            if box_node.border_top > 0.0
                || box_node.border_right > 0.0
                || box_node.border_bottom > 0.0
                || box_node.border_left > 0.0
            {
                self.paint_borders(box_node, abs_x, abs_y, style);
            }
        }

        // 3. 递归绘制子节点（子节点偏移 = 父 padding + border）
        let child_offset_x = abs_x + box_node.padding_left + box_node.border_left;
        let child_offset_y = abs_y + box_node.padding_top + box_node.border_top;
        for child in &box_node.children {
            self.paint_node(child, styles, child_offset_x, child_offset_y);
        }
    }

    /// 绘制边框（4 个矩形）。
    ///
    /// 分别绘制上、右、下、左四条边框。每条边框是一个填充矩形。
    fn paint_borders(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
    ) {
        let w = box_node.width;
        let h = box_node.height;

        // 上边框
        if box_node.border_top > 0.0 {
            self.primitives.add_fill(
                Rect::new(abs_x, abs_y, w, box_node.border_top),
                color_value_to_render(&style.border_top_color),
            );
        }

        // 右边框
        if box_node.border_right > 0.0 {
            self.primitives.add_fill(
                Rect::new(
                    abs_x + w - box_node.border_right,
                    abs_y + box_node.border_top,
                    box_node.border_right,
                    h - box_node.border_top - box_node.border_bottom,
                ),
                color_value_to_render(&style.border_right_color),
            );
        }

        // 下边框
        if box_node.border_bottom > 0.0 {
            self.primitives.add_fill(
                Rect::new(
                    abs_x,
                    abs_y + h - box_node.border_bottom,
                    w,
                    box_node.border_bottom,
                ),
                color_value_to_render(&style.border_bottom_color),
            );
        }

        // 左边框
        if box_node.border_left > 0.0 {
            self.primitives.add_fill(
                Rect::new(
                    abs_x,
                    abs_y + box_node.border_top,
                    box_node.border_left,
                    h - box_node.border_top - box_node.border_bottom,
                ),
                color_value_to_render(&style.border_left_color),
            );
        }
    }

    /// 获取生成的渲染图元（消费 painter）。
    pub fn into_primitives(self) -> RenderPrimitives {
        self.primitives
    }

    /// 获取渲染图元引用。
    pub fn primitives(&self) -> &RenderPrimitives {
        &self.primitives
    }
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 ComputedStyle 的 ColorValue 转换为 render-foundation 的 Color。
pub fn color_value_to_render(color: &ColorValue) -> Color {
    match color {
        ColorValue::Rgba(r, g, b, a) => Color::rgba(*r, *g, *b, *a),
        ColorValue::Transparent => Color::rgba(0, 0, 0, 0),
        ColorValue::Named(name) => named_color_to_render(name),
        ColorValue::CurrentColor => Color::rgba(0, 0, 0, 255),
        ColorValue::Hsla(_, _, _, _) => Color::rgba(0, 0, 0, 255), // HSL 转换暂用黑色回退
    }
}

/// 将命名颜色转换为渲染颜色。
pub fn named_color_to_render(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "red" => Color::rgb(255, 0, 0),
        "green" => Color::rgb(0, 128, 0),
        "blue" => Color::rgb(0, 0, 255),
        "black" => Color::rgb(0, 0, 0),
        "white" => Color::rgb(255, 255, 255),
        "yellow" => Color::rgb(255, 255, 0),
        "cyan" | "aqua" => Color::rgb(0, 255, 255),
        "magenta" | "fuchsia" => Color::rgb(255, 0, 255),
        "gray" | "grey" => Color::rgb(128, 128, 128),
        "silver" => Color::rgb(192, 192, 192),
        "maroon" => Color::rgb(128, 0, 0),
        "olive" => Color::rgb(128, 128, 0),
        "lime" => Color::rgb(0, 255, 0),
        "purple" => Color::rgb(128, 0, 128),
        "teal" => Color::rgb(0, 128, 128),
        "navy" => Color::rgb(0, 0, 128),
        "orange" => Color::rgb(255, 165, 0),
        "pink" => Color::rgb(255, 192, 203),
        "brown" => Color::rgb(165, 42, 42),
        _ => Color::rgb(0, 0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::values::ColorValue;
    use zero_layout_engine::types::OverflowClip;

    /// 测试空布局树不产生任何图元。
    #[test]
    fn test_painter_empty_layout() {
        let layout = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 0.0,
            content_height: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };
        let mut painter = Painter::new();
        let styles = HashMap::new();
        painter.paint(&layout, &styles);
        assert!(painter.primitives().is_empty());
    }

    /// 辅助函数：创建简单 LayoutBox。
    fn make_box(
        node_id: Option<NodeId>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> LayoutBox {
        LayoutBox {
            node_id,
            x,
            y,
            width,
            height,
            content_x: 0.0,
            content_y: 0.0,
            content_width: width,
            content_height: height,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        }
    }

    /// 辅助函数：创建带边框的 LayoutBox。
    fn make_box_with_border(
        node_id: Option<NodeId>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        border_top: f32,
        border_right: f32,
        border_bottom: f32,
        border_left: f32,
    ) -> LayoutBox {
        LayoutBox {
            node_id,
            x,
            y,
            width,
            height,
            content_x: border_left,
            content_y: border_top,
            content_width: width - border_left - border_right,
            content_height: height - border_top - border_bottom,
            border_top,
            border_right,
            border_bottom,
            border_left,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        }
    }

    /// 测试背景色生成填充图元。
    #[test]
    fn test_painter_background_color() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles);

        let primitives = painter.primitives();
        assert_eq!(primitives.fills.len(), 1);
        assert_eq!(primitives.fills[0].color, Color::rgb(255, 0, 0));
        assert_eq!(primitives.fills[0].rect.origin.x, 0.0);
        assert_eq!(primitives.fills[0].rect.origin.y, 0.0);
        assert_eq!(primitives.fills[0].rect.size.width, 100.0);
        assert_eq!(primitives.fills[0].rect.size.height, 50.0);
    }

    /// 测试透明背景不生成填充图元。
    #[test]
    fn test_painter_transparent_background() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Transparent;
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles);

        assert!(painter.primitives().is_empty());
    }

    /// 测试上边框生成填充图元。
    #[test]
    fn test_painter_border_top() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 5.0, 0.0, 0.0, 0.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles);

        assert_eq!(painter.primitives().fills.len(), 1);
        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.origin.x, 0.0);
        assert_eq!(fill.rect.origin.y, 0.0);
        assert_eq!(fill.rect.size.width, 100.0);
        assert_eq!(fill.rect.size.height, 5.0);
    }

    /// 测试四条边框都生成填充图元。
    #[test]
    fn test_painter_border_all_sides() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 3.0, 4.0, 5.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
        style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles);

        // 应该有 4 个边框填充
        assert_eq!(painter.primitives().fills.len(), 4);
    }

    /// 测试嵌套盒子的绘制。
    #[test]
    fn test_painter_nested_boxes() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 10.0, 10.0, 30.0, 20.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 80.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut parent_style = ComputedStyle::default();
        parent_style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        styles.insert(parent, parent_style);

        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(100, 100, 255, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles);

        assert_eq!(painter.primitives().fills.len(), 2);

        // 第一个填充是父元素背景
        assert_eq!(painter.primitives().fills[0].color, Color::rgb(200, 200, 200));
        // 第二个填充是子元素背景（位置偏移 10,10）
        assert_eq!(painter.primitives().fills[1].rect.origin.x, 10.0);
        assert_eq!(painter.primitives().fills[1].rect.origin.y, 10.0);
    }

    /// 测试 ColorValue::Rgba 转换。
    #[test]
    fn test_painter_color_value_rgba() {
        let color = color_value_to_render(&ColorValue::Rgba(128, 64, 32, 255));
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 64);
        assert_eq!(color.b, 32);
        assert_eq!(color.a, 255);
    }

    /// 测试 ColorValue::Transparent 转换。
    #[test]
    fn test_painter_color_value_transparent() {
        let color = color_value_to_render(&ColorValue::Transparent);
        assert_eq!(color.a, 0);
    }

    /// 测试命名颜色转换（red, blue, black, white）。
    #[test]
    fn test_painter_color_value_named() {
        assert_eq!(named_color_to_render("red"), Color::rgb(255, 0, 0));
        assert_eq!(named_color_to_render("blue"), Color::rgb(0, 0, 255));
        assert_eq!(named_color_to_render("black"), Color::rgb(0, 0, 0));
        assert_eq!(named_color_to_render("white"), Color::rgb(255, 255, 255));
        // 大小写不敏感
        assert_eq!(named_color_to_render("Red"), Color::rgb(255, 0, 0));
        assert_eq!(named_color_to_render("BLUE"), Color::rgb(0, 0, 255));
        // 未知颜色回退为黑色
        assert_eq!(named_color_to_render("unknown"), Color::rgb(0, 0, 0));
    }

    /// 测试零尺寸盒子不产生有效图元（宽度为 0 时 Rect 退化为零面积）。
    #[test]
    fn test_painter_zero_size_box() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 10.0, 20.0, 0.0, 0.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles);

        // 会生成一个填充，但尺寸为 0
        assert_eq!(painter.primitives().fills.len(), 1);
        assert_eq!(painter.primitives().fills[0].rect.size.width, 0.0);
        assert_eq!(painter.primitives().fills[0].rect.size.height, 0.0);
    }

    /// 测试绝对偏移计算正确。
    #[test]
    fn test_painter_absolute_offset() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 50.0, 30.0, 100.0, 50.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(0, 128, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles);

        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.origin.x, 50.0);
        assert_eq!(fill.rect.origin.y, 30.0);
    }

    /// 测试多个子节点都能生成填充图元。
    #[test]
    fn test_painter_multiple_children() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child1 = doc.create_element("span");
        let child2 = doc.create_element("span");

        let child_box1 = make_box(Some(child1), 0.0, 0.0, 50.0, 20.0);
        let child_box2 = make_box(Some(child2), 0.0, 20.0, 50.0, 20.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 80.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box1, child_box2],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        for id in [child1, child2] {
            let mut s = ComputedStyle::default();
            s.background_color = ColorValue::Rgba(255, 0, 0, 255);
            styles.insert(id, s);
        }

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles);

        // 只有子节点有背景色，父节点没有
        assert_eq!(painter.primitives().fills.len(), 2);
    }

    /// 测试 into_primitives 消费 painter。
    #[test]
    fn test_painter_into_primitives() {
        let mut painter = Painter::new();
        let layout = make_box(None, 0.0, 0.0, 0.0, 0.0);
        let styles = HashMap::new();
        painter.paint(&layout, &styles);
        let primitives = painter.into_primitives();
        assert!(primitives.is_empty());
    }

    /// 测试 Default 实现。
    #[test]
    fn test_painter_default() {
        let painter = Painter::default();
        assert!(painter.primitives().is_empty());
    }

    /// 测试 background + border 同时存在时填充数量（1 background + 4 border = 5）。
    #[test]
    fn test_painter_background_plus_border_fill_count() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 100.0, 50.0, 2.0, 2.0, 2.0, 2.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(200, 200, 200, 255);
        style.border_top_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 0, 255);
        style.border_left_color = ColorValue::Rgba(0, 0, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles);

        // 1 background fill + 4 border fills = 5
        assert_eq!(painter.primitives().fills.len(), 5);
        // First fill is background
        assert_eq!(painter.primitives().fills[0].color, Color::rgb(200, 200, 200));
    }

    /// 测试无样式节点（no node_id）不产生任何填充。
    #[test]
    fn test_painter_no_style_no_fills() {
        let layout = make_box(None, 0.0, 0.0, 100.0, 50.0);
        let mut painter = Painter::new();
        let styles = HashMap::new();
        painter.paint(&layout, &styles);
        assert!(painter.primitives().is_empty());
    }

    /// 测试 only background（no border）产生恰好 1 个填充。
    #[test]
    fn test_painter_only_background_fill_count() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box(Some(elem), 0.0, 0.0, 80.0, 40.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.background_color = ColorValue::Rgba(0, 128, 255, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles);

        assert_eq!(painter.primitives().fills.len(), 1);
    }

    /// 测试 only border（transparent background）产生恰好 4 个填充。
    #[test]
    fn test_painter_only_border_fill_count() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");
        let layout = make_box_with_border(Some(elem), 0.0, 0.0, 80.0, 40.0, 1.0, 1.0, 1.0, 1.0);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        // background is transparent by default
        style.border_top_color = ColorValue::Rgba(255, 0, 0, 255);
        style.border_right_color = ColorValue::Rgba(0, 255, 0, 255);
        style.border_bottom_color = ColorValue::Rgba(0, 0, 255, 255);
        style.border_left_color = ColorValue::Rgba(255, 255, 0, 255);
        styles.insert(elem, style);

        let mut painter = Painter::new();
        painter.paint(&layout, &styles);

        // 4 border fills, no background fill
        assert_eq!(painter.primitives().fills.len(), 4);
    }

    /// 测试带 padding 的子节点偏移。
    #[test]
    fn test_painter_child_offset_with_padding() {
        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let child = doc.create_element("span");

        let child_box = make_box(Some(child), 0.0, 0.0, 50.0, 20.0);
        let parent_box = LayoutBox {
            node_id: Some(parent),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            content_x: 10.0,
            content_y: 10.0,
            content_width: 80.0,
            content_height: 60.0,
            border_top: 5.0,
            border_right: 5.0,
            border_bottom: 5.0,
            border_left: 5.0,
            padding_top: 5.0,
            padding_right: 5.0,
            padding_bottom: 5.0,
            padding_left: 5.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut child_style = ComputedStyle::default();
        child_style.background_color = ColorValue::Rgba(255, 0, 0, 255);
        styles.insert(child, child_style);

        let mut painter = Painter::new();
        painter.paint(&parent_box, &styles);

        // 子节点偏移 = padding_left(5) + border_left(5) = 10
        let fill = &painter.primitives().fills[0];
        assert_eq!(fill.rect.origin.x, 10.0);
        assert_eq!(fill.rect.origin.y, 10.0);
    }
}
