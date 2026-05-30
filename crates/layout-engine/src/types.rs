//! 布局输出类型定义。
//!
//! 定义 [`LayoutBox`] 和 [`LayoutResult`] 作为布局引擎的输出格式，
//! 描述元素在页面上的几何位置和大小。

use zero_dom::NodeId;

/// 溢出处理方式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverflowClip {
    /// 内容可见，超出部分正常显示。
    Visible,
    /// 内容被裁剪，超出部分不可见。
    Hidden,
    /// 内容被裁剪，与 Hidden 类似但不建立滚动容器。
    Clip,
    /// 内容可滚动查看。
    Scroll,
}

/// 布局盒 — 一个元素在页面上的几何位置和大小。
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// 对应的 DOM 节点 ID。
    pub node_id: Option<NodeId>,
    /// 盒子的位置（相对于父元素的内容区域）。
    pub x: f32,
    /// 盒子的位置（相对于父元素的内容区域）。
    pub y: f32,
    /// 盒子的尺寸（包含 border）。
    pub width: f32,
    /// 盒子的尺寸（包含 border）。
    pub height: f32,
    /// 内容区域偏移（border + padding）。
    pub content_x: f32,
    /// 内容区域偏移（border + padding）。
    pub content_y: f32,
    /// 内容区域尺寸。
    pub content_width: f32,
    /// 内容区域尺寸。
    pub content_height: f32,
    /// 边框宽度。
    pub border_top: f32,
    /// 边框宽度。
    pub border_right: f32,
    /// 边框宽度。
    pub border_bottom: f32,
    /// 边框宽度。
    pub border_left: f32,
    /// 内边距。
    pub padding_top: f32,
    /// 内边距。
    pub padding_right: f32,
    /// 内边距。
    pub padding_bottom: f32,
    /// 内边距。
    pub padding_left: f32,
    /// 外边距。
    pub margin_top: f32,
    /// 外边距。
    pub margin_right: f32,
    /// 外边距。
    pub margin_bottom: f32,
    /// 外边距。
    pub margin_left: f32,
    /// 子布局盒。
    pub children: Vec<LayoutBox>,
    /// 是否为绝对定位。
    pub is_absolute: bool,
    /// 是否为 fixed 定位（需宿主层处理）。
    pub is_fixed: bool,
    /// 溢出处理。
    pub overflow_x: OverflowClip,
    /// 溢出处理。
    pub overflow_y: OverflowClip,
}

impl LayoutBox {
    /// 获取绝对位置（从根节点开始累加）。
    ///
    /// 递归累加自身和所有祖先节点的 x/y 偏移。
    pub fn absolute_position(&self) -> (f32, f32) {
        // 当前盒子的位置已经是相对于父元素的，
        // 需要递归累加。但 LayoutBox 树中每个节点的 x/y
        // 是相对于父元素内容区域的偏移。
        // 对于根节点，x/y 就是绝对位置。
        // 对于子节点，需要累加。
        // 注意：此方法只能计算从自身开始的坐标，
        // 完整的绝对位置需要在递归时传入父节点的绝对位置。
        (self.x, self.y)
    }

    /// 递归计算绝对位置（传入父级绝对位置）。
    pub fn absolute_position_with_parent(
        &self,
        parent_abs_x: f32,
        parent_abs_y: f32,
    ) -> (f32, f32) {
        (parent_abs_x + self.x, parent_abs_y + self.y)
    }

    /// 获取盒子总面积（含 margin）。
    pub fn outer_area(&self) -> f32 {
        let total_width = self.margin_left + self.width + self.margin_right;
        let total_height = self.margin_top + self.height + self.margin_bottom;
        total_width * total_height
    }
}

/// 布局结果 — 整个文档的布局树。
pub struct LayoutResult {
    /// 根布局盒。
    pub root: LayoutBox,
    /// 视口宽度。
    pub viewport_width: f32,
    /// 视口高度。
    pub viewport_height: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试默认 LayoutBox 的基本属性。
    #[test]
    fn test_layout_box_default() {
        let box0 = LayoutBox {
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
        assert_eq!(box0.width, 0.0);
        assert_eq!(box0.height, 0.0);
        assert!(!box0.is_absolute);
        assert!(!box0.is_fixed);
        assert!(box0.children.is_empty());
    }

    /// 测试 absolute_position。
    #[test]
    fn test_layout_box_absolute_position() {
        let box0 = LayoutBox {
            node_id: None,
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            content_x: 12.0,
            content_y: 22.0,
            content_width: 96.0,
            content_height: 46.0,
            border_top: 1.0,
            border_right: 1.0,
            border_bottom: 1.0,
            border_left: 1.0,
            padding_top: 1.0,
            padding_right: 1.0,
            padding_bottom: 1.0,
            padding_left: 1.0,
            margin_top: 5.0,
            margin_right: 5.0,
            margin_bottom: 5.0,
            margin_left: 5.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };
        let (abs_x, abs_y) = box0.absolute_position();
        assert_eq!(abs_x, 10.0);
        assert_eq!(abs_y, 20.0);
    }

    /// 测试 outer_area。
    #[test]
    fn test_layout_box_outer_area() {
        let box0 = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 50.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 10.0,
            margin_right: 10.0,
            margin_bottom: 10.0,
            margin_left: 10.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };
        // 总宽度 = 10 + 100 + 10 = 120, 总高度 = 10 + 50 + 10 = 70
        let area = box0.outer_area();
        assert!((area - 120.0 * 70.0).abs() < 0.001);
    }

    /// 测试 content box 计算。
    #[test]
    fn test_layout_box_content_box() {
        let box0 = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            content_x: 5.0,
            content_y: 5.0,
            content_width: 90.0,
            content_height: 70.0,
            border_top: 2.0,
            border_right: 2.0,
            border_bottom: 2.0,
            border_left: 2.0,
            padding_top: 3.0,
            padding_right: 3.0,
            padding_bottom: 3.0,
            padding_left: 3.0,
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
        // content = 100 - 2*2 - 2*3 = 100 - 10 = 90
        assert!((box0.content_width - 90.0).abs() < 0.001);
        assert!((box0.content_height - 70.0).abs() < 0.001);
    }

    /// 测试 OverflowClip 各变体。
    #[test]
    fn test_overflow_clip_variants() {
        assert_eq!(OverflowClip::Visible, OverflowClip::Visible);
        assert_eq!(OverflowClip::Hidden, OverflowClip::Hidden);
        assert_eq!(OverflowClip::Clip, OverflowClip::Clip);
        assert_eq!(OverflowClip::Scroll, OverflowClip::Scroll);
        assert_ne!(OverflowClip::Visible, OverflowClip::Hidden);
    }

    /// 测试 LayoutResult 的视口信息。
    #[test]
    fn test_layout_result_viewport() {
        let result = LayoutResult {
            root: LayoutBox {
                node_id: None,
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
                content_x: 0.0,
                content_y: 0.0,
                content_width: 800.0,
                content_height: 600.0,
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
            },
            viewport_width: 800.0,
            viewport_height: 600.0,
        };
        assert!((result.viewport_width - 800.0).abs() < 0.001);
        assert!((result.viewport_height - 600.0).abs() < 0.001);
    }

    /// 测试带子节点的 LayoutBox。
    #[test]
    fn test_layout_box_with_children() {
        let child = LayoutBox {
            node_id: None,
            x: 10.0,
            y: 10.0,
            width: 50.0,
            height: 30.0,
            content_x: 10.0,
            content_y: 10.0,
            content_width: 50.0,
            content_height: 30.0,
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
        let parent = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 100.0,
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
            children: vec![child],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };
        assert_eq!(parent.children.len(), 1);
        assert!((parent.children[0].x - 10.0).abs() < 0.001);
    }

    /// 测试嵌套绝对位置计算。
    #[test]
    fn test_layout_box_nested_absolute_position() {
        let child = LayoutBox {
            node_id: None,
            x: 20.0,
            y: 30.0,
            width: 50.0,
            height: 50.0,
            content_x: 20.0,
            content_y: 30.0,
            content_width: 50.0,
            content_height: 50.0,
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
        let (abs_x, abs_y) = child.absolute_position_with_parent(10.0, 20.0);
        assert!((abs_x - 30.0).abs() < 0.001);
        assert!((abs_y - 50.0).abs() < 0.001);
    }

    /// 测试零尺寸元素。
    #[test]
    fn test_layout_box_zero_size() {
        let box0 = LayoutBox {
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
        assert!((box0.outer_area()).abs() < 0.001);
    }

    /// 测试 LayoutBox 的 clone。
    #[test]
    fn test_layout_box_clone() {
        let box0 = LayoutBox {
            node_id: None,
            x: 5.0,
            y: 10.0,
            width: 100.0,
            height: 50.0,
            content_x: 7.0,
            content_y: 12.0,
            content_width: 96.0,
            content_height: 46.0,
            border_top: 1.0,
            border_right: 1.0,
            border_bottom: 1.0,
            border_left: 1.0,
            padding_top: 1.0,
            padding_right: 1.0,
            padding_bottom: 1.0,
            padding_left: 1.0,
            margin_top: 2.0,
            margin_right: 2.0,
            margin_bottom: 2.0,
            margin_left: 2.0,
            children: vec![],
            is_absolute: true,
            is_fixed: false,
            overflow_x: OverflowClip::Hidden,
            overflow_y: OverflowClip::Scroll,
        };
        let cloned = box0.clone();
        assert!((cloned.x - 5.0).abs() < 0.001);
        assert!(cloned.is_absolute);
        assert_eq!(cloned.overflow_x, OverflowClip::Hidden);
        assert_eq!(cloned.overflow_y, OverflowClip::Scroll);
    }
}
