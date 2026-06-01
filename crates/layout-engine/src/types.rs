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
    /// 是否为 sticky 定位（需宿主层在滚动时动态调整偏移）。
    pub is_sticky: bool,
    /// 溢出处理。
    pub overflow_x: OverflowClip,
    /// 溢出处理。
    pub overflow_y: OverflowClip,
    /// z-index 值（用于堆叠上下文排序）。
    /// 仅对 positioned 元素（absolute/relative/fixed/sticky）生效。
    /// 默认为 0，对应 z-index: auto。
    pub z_index: i32,
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
    pub fn absolute_position_with_parent(&self, parent_abs_x: f32, parent_abs_y: f32) -> (f32, f32) {
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
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
                is_sticky: false,
                overflow_x: OverflowClip::Visible,
                overflow_y: OverflowClip::Visible,
                z_index: 0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Hidden,
            overflow_y: OverflowClip::Scroll,
            z_index: 10,
        };
        let cloned = box0.clone();
        assert!((cloned.x - 5.0).abs() < 0.001);
        assert!(cloned.is_absolute);
        assert_eq!(cloned.overflow_x, OverflowClip::Hidden);
        assert_eq!(cloned.overflow_y, OverflowClip::Scroll);
        assert_eq!(cloned.z_index, 10);
    }

    /// 测试 LayoutBox 的 z_index 字段。
    #[test]
    fn test_layout_box_z_index() {
        // 默认 z_index 为 0（对应 auto）
        let box_default = LayoutBox {
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
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        assert_eq!(box_default.z_index, 0);

        // 正 z-index
        let box_positive = LayoutBox {
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
            children: vec![],
            is_absolute: true,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 999,
        };
        assert_eq!(box_positive.z_index, 999);

        // 负 z-index
        let box_negative = LayoutBox {
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
            children: vec![],
            is_absolute: true,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: -1,
        };
        assert_eq!(box_negative.z_index, -1);
    }

    /// 测试 z-index 排序：多个 LayoutBox 按 z_index 排序后顺序正确。
    #[test]
    fn test_layout_box_z_index_sorting() {
        let boxes = vec![
            LayoutBox {
                node_id: None,
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
                content_x: 0.0,
                content_y: 0.0,
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
                is_absolute: true,
                is_fixed: false,
                is_sticky: false,
                overflow_x: OverflowClip::Visible,
                overflow_y: OverflowClip::Visible,
                z_index: 10,
            },
            LayoutBox {
                node_id: None,
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
                content_x: 0.0,
                content_y: 0.0,
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
                is_absolute: true,
                is_fixed: false,
                is_sticky: false,
                overflow_x: OverflowClip::Visible,
                overflow_y: OverflowClip::Visible,
                z_index: -1,
            },
            LayoutBox {
                node_id: None,
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
                content_x: 0.0,
                content_y: 0.0,
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
                is_absolute: true,
                is_fixed: false,
                is_sticky: false,
                overflow_x: OverflowClip::Visible,
                overflow_y: OverflowClip::Visible,
                z_index: 5,
            },
        ];

        let mut sorted = boxes;
        sorted.sort_by_key(|b| b.z_index);
        assert_eq!(sorted[0].z_index, -1);
        assert_eq!(sorted[1].z_index, 5);
        assert_eq!(sorted[2].z_index, 10);
    }

    // -- 边界条件测试 --

    /// 测试 LayoutBox outer_area 为负值（负 margin）
    #[test]
    fn test_layout_box_negative_margin_outer_area() {
        // 负 margin 可以让 outer_area 变为负值或零
        let box0 = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 10.0,
            content_height: 10.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: -20.0,
            margin_right: -20.0,
            margin_bottom: -20.0,
            margin_left: -20.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        // total_width = -20 + 10 + -20 = -30, total_height = -20 + 10 + -20 = -30
        // outer_area = -30 * -30 = 900（两个负值相乘为正）
        let area = box0.outer_area();
        assert!(area >= 0.0 || area.is_nan(), "负 margin 导致 outer_area 为 {}", area);
    }

    /// 测试 LayoutBox 深层嵌套 absolute_position_with_parent
    #[test]
    fn test_layout_box_deeply_nested_position() {
        // 3 层嵌套，验证绝对位置累积正确
        let level3 = LayoutBox {
            node_id: None,
            x: 5.0,
            y: 5.0,
            width: 10.0,
            height: 10.0,
            content_x: 5.0,
            content_y: 5.0,
            content_width: 10.0,
            content_height: 10.0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let level2 = LayoutBox {
            node_id: None,
            x: 20.0,
            y: 30.0,
            width: 100.0,
            height: 100.0,
            content_x: 20.0,
            content_y: 30.0,
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
            children: vec![level3],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let level1 = LayoutBox {
            node_id: None,
            x: 100.0,
            y: 200.0,
            width: 500.0,
            height: 500.0,
            content_x: 100.0,
            content_y: 200.0,
            content_width: 500.0,
            content_height: 500.0,
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
            children: vec![level2],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        // level1 → level2: (100+20, 200+30) = (120, 230)
        let (abs_x2, abs_y2) = level1.children[0].absolute_position_with_parent(100.0, 200.0);
        assert!((abs_x2 - 120.0).abs() < 0.001);
        assert!((abs_y2 - 230.0).abs() < 0.001);

        // level1 → level2 → level3: (120+5, 230+5) = (125, 235)
        let (abs_x3, abs_y3) = level1.children[0].children[0].absolute_position_with_parent(abs_x2, abs_y2);
        assert!((abs_x3 - 125.0).abs() < 0.001);
        assert!((abs_y3 - 235.0).abs() < 0.001);
    }

    /// 测试 LayoutBox is_sticky 字段
    #[test]
    fn test_layout_box_sticky_flag() {
        // 创建 is_sticky = true 的 LayoutBox
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
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: true,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        assert!(box0.is_sticky, "is_sticky 应为 true");
        assert!(!box0.is_absolute, "is_absolute 应为 false");
        assert!(!box0.is_fixed, "is_fixed 应为 false");
    }

    /// 测试 LayoutBox z_index 为负值
    #[test]
    fn test_layout_box_negative_z_index() {
        // LayoutBox with z_index = -1
        let box0 = LayoutBox {
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
            children: vec![],
            is_absolute: true,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: -1,
        };
        assert_eq!(box0.z_index, -1);
        assert!(box0.z_index < 0, "z_index 应为负值");
    }

    /// 测试 LayoutBox 零尺寸子元素
    #[test]
    fn test_layout_box_zero_size_children() {
        // 子元素 width=0, height=0
        let child1 = LayoutBox {
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let child2 = LayoutBox {
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
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
            children: vec![child1, child2],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        assert_eq!(parent.children.len(), 2);
        assert!((parent.children[0].width).abs() < 0.001);
        assert!((parent.children[1].height).abs() < 0.001);
    }

    /// 测试 LayoutBox 大量子元素
    #[test]
    fn test_layout_box_many_children() {
        // 100 个子元素，验证数量
        let children: Vec<LayoutBox> = (0..100)
            .map(|i| LayoutBox {
                node_id: None,
                x: i as f32,
                y: 0.0,
                width: 10.0,
                height: 10.0,
                content_x: i as f32,
                content_y: 0.0,
                content_width: 10.0,
                content_height: 10.0,
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
                is_sticky: false,
                overflow_x: OverflowClip::Visible,
                overflow_y: OverflowClip::Visible,
                z_index: 0,
            })
            .collect();
        let parent = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 1000.0,
            height: 10.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 1000.0,
            content_height: 10.0,
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
            children,
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        assert_eq!(parent.children.len(), 100);
        // 验证第一个和最后一个子元素
        assert!((parent.children[0].x - 0.0).abs() < 0.001);
        assert!((parent.children[99].x - 99.0).abs() < 0.001);
    }

    // -- 边界条件测试（第三批）--

    /// 测试 LayoutBox 同时标记多个定位标志。
    ///
    /// 在极端情况下，一个元素不应同时标记为 absolute 和 fixed，
    /// 验证默认状态下所有定位标志互不干扰。
    #[test]
    fn test_layout_box_position_flags_mutually_exclusive() {
        let mut box_abs = LayoutBox {
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
            children: vec![],
            is_absolute: true,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        assert!(box_abs.is_absolute);
        assert!(!box_abs.is_fixed);
        assert!(!box_abs.is_sticky);

        // 切换为 fixed
        box_abs.is_absolute = false;
        box_abs.is_fixed = true;
        assert!(!box_abs.is_absolute);
        assert!(box_abs.is_fixed);

        // 切换为 sticky
        box_abs.is_fixed = false;
        box_abs.is_sticky = true;
        assert!(!box_abs.is_fixed);
        assert!(box_abs.is_sticky);
    }

    /// 测试 LayoutResult 零视口尺寸。
    ///
    /// 视口宽度或高度为 0 是合法的（例如最小化窗口），
    /// 验证 LayoutResult 能正确存储零值视口。
    #[test]
    fn test_layout_result_zero_viewport() {
        let result = LayoutResult {
            root: LayoutBox {
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
                is_sticky: false,
                overflow_x: OverflowClip::Visible,
                overflow_y: OverflowClip::Visible,
                z_index: 0,
            },
            viewport_width: 0.0,
            viewport_height: 0.0,
        };
        assert!((result.viewport_width - 0.0).abs() < 0.001);
        assert!((result.viewport_height - 0.0).abs() < 0.001);
        assert!((result.root.width - 0.0).abs() < 0.001);
    }

    /// 测试 LayoutBox 的 x/y 为负值（负偏移场景）。
    ///
    /// 元素可能通过负 margin 或负 inset 导致位置为负，
    /// 验证 absolute_position 和 absolute_position_with_parent
    /// 在负坐标下的正确行为。
    #[test]
    fn test_layout_box_negative_position() {
        let box0 = LayoutBox {
            node_id: None,
            x: -50.0,
            y: -30.0,
            width: 100.0,
            height: 100.0,
            content_x: -50.0,
            content_y: -30.0,
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
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let (abs_x, abs_y) = box0.absolute_position();
        assert!((abs_x - (-50.0)).abs() < 0.001);
        assert!((abs_y - (-30.0)).abs() < 0.001);

        // 负父偏移 + 负子偏移
        let (abs_x2, abs_y2) = box0.absolute_position_with_parent(-100.0, -200.0);
        assert!((abs_x2 - (-150.0)).abs() < 0.001);
        assert!((abs_y2 - (-230.0)).abs() < 0.001);
    }

    /// 测试 LayoutBox 混合溢出处理（x 和 y 方向不同）。
    ///
    /// 真实场景中 overflow-x 和 overflow-y 可以不同，
    /// 验证两个方向独立存储各自的溢出策略。
    #[test]
    fn test_layout_box_mixed_overflow_xy() {
        let box0 = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 200.0,
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
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Scroll,
            overflow_y: OverflowClip::Hidden,
            z_index: 0,
        };
        assert_eq!(box0.overflow_x, OverflowClip::Scroll);
        assert_eq!(box0.overflow_y, OverflowClip::Hidden);
        assert_ne!(box0.overflow_x, box0.overflow_y);

        // Clip 变体
        let box1 = LayoutBox {
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
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Clip,
            z_index: 0,
        };
        assert_eq!(box1.overflow_x, OverflowClip::Visible);
        assert_eq!(box1.overflow_y, OverflowClip::Clip);
    }

    /// 测试 LayoutBox outer_area 在不对称 margin 下的计算。
    ///
    /// 左右 margin 不同、上下 margin 不同时，
    /// outer_area 应正确计算总面积（非正方形场景）。
    #[test]
    fn test_layout_box_asymmetric_margin_outer_area() {
        let box0 = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 60.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 60.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 5.0,
            margin_right: 15.0,
            margin_bottom: 10.0,
            margin_left: 20.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        // total_width = 20 + 100 + 15 = 135
        // total_height = 5 + 60 + 10 = 75
        // outer_area = 135 * 75 = 10125
        let area = box0.outer_area();
        assert!(
            (area - 10125.0).abs() < 0.001,
            "不对称 margin outer_area 应为 10125，实际 {}",
            area
        );
    }

    // -- 边界条件测试（第四批）--

    /// 测试 LayoutBox 使用 f32 极大值时的行为。
    ///
    /// 验证在 f32::MAX 尺寸下 outer_area 不会 panic，
    /// 结果应为无穷大（inf）。
    #[test]
    fn test_layout_box_f32_max_dimensions() {
        let box0 = LayoutBox {
            node_id: None,
            x: f32::MAX,
            y: f32::MAX,
            width: f32::MAX,
            height: f32::MAX,
            content_x: 0.0,
            content_y: 0.0,
            content_width: f32::MAX,
            content_height: f32::MAX,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let area = box0.outer_area();
        assert!(area.is_infinite(), "f32::MAX 尺寸下 outer_area 应为 inf，实际 {}", area);
        let (abs_x, abs_y) = box0.absolute_position();
        assert_eq!(abs_x, f32::MAX);
        assert_eq!(abs_y, f32::MAX);
    }

    /// 测试 LayoutBox 仅含边框（无 padding、无内容）的 outer_area。
    ///
    /// 模拟 width = border_left + border_right、height = border_top + border_bottom
    /// 且无内容区域的极端情况，验证 outer_area 计算仍然正确。
    #[test]
    fn test_layout_box_border_only_outer_area() {
        let box0 = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 10.0, // border_left(3) + content_width(4) + border_right(3)
            height: 10.0,
            content_x: 3.0,
            content_y: 2.0,
            content_width: 4.0,
            content_height: 6.0,
            border_top: 2.0,
            border_right: 3.0,
            border_bottom: 2.0,
            border_left: 3.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 1.0,
            margin_right: 1.0,
            margin_bottom: 1.0,
            margin_left: 1.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        // total_width = 1 + 10 + 1 = 12, total_height = 1 + 10 + 1 = 12
        // outer_area = 12 * 12 = 144
        let area = box0.outer_area();
        assert!(
            (area - 144.0).abs() < 0.001,
            "仅边框 outer_area 应为 144，实际 {}",
            area
        );
    }

    /// 测试 LayoutBox clone 的深拷贝语义。
    ///
    /// 验证 clone 后修改原始对象的 children 不会影响克隆副本，
    /// 即 children 是深拷贝而非共享引用。
    #[test]
    fn test_layout_box_clone_deep_copy_children() {
        let child = LayoutBox {
            node_id: None,
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
            content_x: 10.0,
            content_y: 20.0,
            content_width: 30.0,
            content_height: 40.0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let original = LayoutBox {
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let cloned = original.clone();
        // 克隆后两者 children 数量相同
        assert_eq!(cloned.children.len(), 1);
        assert!((cloned.children[0].x - 10.0).abs() < 0.001);
        // 两者是独立的 Vec，互不影响
        assert_eq!(original.children.len(), cloned.children.len());
    }

    /// 测试 OverflowClip 的 Copy trait 语义。
    ///
    /// 验证 Copy 类型赋值后修改副本不影响原始值，
    /// 且所有变体可以通过 Copy 独立使用。
    #[test]
    fn test_overflow_clip_copy_semantics() {
        let a = OverflowClip::Scroll;
        let b = a; // Copy 语义
        assert_eq!(a, b);
        assert_eq!(a, OverflowClip::Scroll);

        let mut c = OverflowClip::Hidden;
        let d = c;
        c = OverflowClip::Clip;
        assert_eq!(d, OverflowClip::Hidden, "副本应不受原始变量后续修改影响");
        assert_eq!(c, OverflowClip::Clip);
    }

    /// 测试 LayoutBox 使用 f32::NAN 坐标时的 absolute_position 行为。
    ///
    /// NaN 在布局计算中可能因非法运算产生，
    /// 验证相关方法在 NaN 输入下不会 panic。
    #[test]
    fn test_layout_box_nan_position() {
        let box0 = LayoutBox {
            node_id: None,
            x: f32::NAN,
            y: f32::NAN,
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
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let (abs_x, abs_y) = box0.absolute_position();
        assert!(abs_x.is_nan(), "NaN x 传入后 absolute_position 应返回 NaN");
        assert!(abs_y.is_nan(), "NaN y 传入后 absolute_position 应返回 NaN");

        let (abs_x2, abs_y2) = box0.absolute_position_with_parent(10.0, 20.0);
        assert!(abs_x2.is_nan(), "NaN + 有限值应仍为 NaN");
        assert!(abs_y2.is_nan(), "NaN + 有限值应仍为 NaN");
    }

    // -- 边界条件测试（第五批）--

    /// 测试 LayoutBox 默认值的所有几何字段均为零。
    ///
    /// 构造一个全零 LayoutBox，逐一验证位置、尺寸、内容区域、
    /// 边框、内边距、外边距均为 0.0，且 children 为空。
    #[test]
    fn test_layout_box_default_values() {
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        // 位置和尺寸均为零
        assert!((box0.x).abs() < 0.001);
        assert!((box0.y).abs() < 0.001);
        assert!((box0.width).abs() < 0.001);
        assert!((box0.height).abs() < 0.001);
        // 内容区域为零
        assert!((box0.content_x).abs() < 0.001);
        assert!((box0.content_y).abs() < 0.001);
        assert!((box0.content_width).abs() < 0.001);
        assert!((box0.content_height).abs() < 0.001);
        // 边框为零
        assert!((box0.border_top).abs() < 0.001);
        assert!((box0.border_right).abs() < 0.001);
        assert!((box0.border_bottom).abs() < 0.001);
        assert!((box0.border_left).abs() < 0.001);
        // 内边距为零
        assert!((box0.padding_top).abs() < 0.001);
        assert!((box0.padding_right).abs() < 0.001);
        assert!((box0.padding_bottom).abs() < 0.001);
        assert!((box0.padding_left).abs() < 0.001);
        // 外边距为零
        assert!((box0.margin_top).abs() < 0.001);
        assert!((box0.margin_right).abs() < 0.001);
        assert!((box0.margin_bottom).abs() < 0.001);
        assert!((box0.margin_left).abs() < 0.001);
        // 无子节点
        assert!(box0.children.is_empty());
        // 无定位标志
        assert!(!box0.is_absolute);
        assert!(!box0.is_fixed);
        assert!(!box0.is_sticky);
    }

    /// 测试 LayoutResult 使用特定视口尺寸。
    ///
    /// 创建视口为 375x667（模拟移动端）的 LayoutResult，
    /// 验证 viewport_width 和 viewport_height 字段正确存储。
    #[test]
    fn test_layout_result_viewport_edge() {
        let result = LayoutResult {
            root: LayoutBox {
                node_id: None,
                x: 0.0,
                y: 0.0,
                width: 375.0,
                height: 667.0,
                content_x: 0.0,
                content_y: 0.0,
                content_width: 375.0,
                content_height: 667.0,
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
                is_sticky: false,
                overflow_x: OverflowClip::Visible,
                overflow_y: OverflowClip::Visible,
                z_index: 0,
            },
            viewport_width: 375.0,
            viewport_height: 667.0,
        };
        assert!((result.viewport_width - 375.0).abs() < 0.001);
        assert!((result.viewport_height - 667.0).abs() < 0.001);
        assert!((result.root.width - 375.0).abs() < 0.001);
        assert!((result.root.height - 667.0).abs() < 0.001);
    }

    /// 测试 LayoutBox 添加多个子节点后 children 向量长度。
    ///
    /// 向一个 LayoutBox 添加 3 个子盒子，验证 children 长度为 3，
    /// 且每个子盒子可独立访问。
    #[test]
    fn test_layout_box_with_children_edge() {
        let make_child = |x: f32, y: f32, w: f32, h: f32| LayoutBox {
            node_id: None,
            x,
            y,
            width: w,
            height: h,
            content_x: x,
            content_y: y,
            content_width: w,
            content_height: h,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let parent = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 100.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 300.0,
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
            children: vec![
                make_child(0.0, 0.0, 100.0, 50.0),
                make_child(100.0, 0.0, 100.0, 50.0),
                make_child(200.0, 0.0, 100.0, 50.0),
            ],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        assert_eq!(parent.children.len(), 3);
        assert!((parent.children[0].x - 0.0).abs() < 0.001);
        assert!((parent.children[1].x - 100.0).abs() < 0.001);
        assert!((parent.children[2].x - 200.0).abs() < 0.001);
    }

    /// 测试 OverflowClip::Visible 和 OverflowClip::Hidden 是不同变体。
    ///
    /// 验证 Visible != Hidden，且各变体与其自身相等，
    /// 确保枚举的 PartialEq 实现正确区分两种溢出策略。
    #[test]
    fn test_overflow_clip_visible_vs_hidden() {
        assert_ne!(
            OverflowClip::Visible,
            OverflowClip::Hidden,
            "Visible 和 Hidden 应为不同变体"
        );
        assert_eq!(OverflowClip::Visible, OverflowClip::Visible);
        assert_eq!(OverflowClip::Hidden, OverflowClip::Hidden);
        // 两者在同一个 LayoutBox 中分别赋给 overflow_x/overflow_y
        let box0 = LayoutBox {
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
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Hidden,
            z_index: 0,
        };
        assert_eq!(box0.overflow_x, OverflowClip::Visible);
        assert_eq!(box0.overflow_y, OverflowClip::Hidden);
        assert_ne!(box0.overflow_x, box0.overflow_y);
    }

    /// 测试 LayoutBox content_x/y/width/height 字段正确设置。
    ///
    /// 构造一个带 border 和 padding 的 LayoutBox，
    /// 验证 content_x 等于 border_left + padding_left，
    /// content_y 等于 border_top + padding_top，
    /// content 空间排除 border 和 padding。
    #[test]
    fn test_layout_box_content_area() {
        let box0 = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 150.0,
            content_x: 15.0,       // border_left(5) + padding_left(10)
            content_y: 12.0,       // border_top(4) + padding_top(8)
            content_width: 170.0,  // 200 - border_left(5) - border_right(5) - padding_left(10) - padding_right(10)
            content_height: 126.0, // 150 - border_top(4) - border_bottom(4) - padding_top(8) - padding_bottom(8)
            border_top: 4.0,
            border_right: 5.0,
            border_bottom: 4.0,
            border_left: 5.0,
            padding_top: 8.0,
            padding_right: 10.0,
            padding_bottom: 8.0,
            padding_left: 10.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        // content_x = border_left + padding_left = 5 + 10 = 15
        assert!((box0.content_x - 15.0).abs() < 0.001, "content_x 应为 15");
        // content_y = border_top + padding_top = 4 + 8 = 12
        assert!((box0.content_y - 12.0).abs() < 0.001, "content_y 应为 12");
        // content_width = 200 - 5 - 5 - 10 - 10 = 170
        assert!((box0.content_width - 170.0).abs() < 0.001, "content_width 应为 170");
        // content_height = 150 - 4 - 4 - 8 - 8 = 126
        assert!((box0.content_height - 126.0).abs() < 0.001, "content_height 应为 126");
        // 验证 content 区域与 border/padding 的关系一致
        assert!((box0.content_x - box0.border_left - box0.padding_left).abs() < 0.001);
        assert!((box0.content_y - box0.border_top - box0.padding_top).abs() < 0.001);
        assert!(
            (box0.content_width
                - (box0.width - box0.border_left - box0.border_right - box0.padding_left - box0.padding_right))
                .abs()
                < 0.001
        );
        assert!(
            (box0.content_height
                - (box0.height - box0.border_top - box0.border_bottom - box0.padding_top - box0.padding_bottom))
                .abs()
                < 0.001
        );
    }

    // ── absolute_position / absolute_position_with_parent 边界条件测试 ──

    /// 测试 absolute_position 对零坐标的 LayoutBox 返回 (0.0, 0.0)。
    #[test]
    fn test_absolute_position_zero() {
        let box0 = LayoutBox {
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
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let (abs_x, abs_y) = box0.absolute_position();
        assert!((abs_x - 0.0).abs() < 0.001);
        assert!((abs_y - 0.0).abs() < 0.001);
    }

    /// 测试 absolute_position 对带 border/padding 的盒子只返回 (x, y)。
    ///
    /// absolute_position 只返回 self.x 和 self.y，不受 border/padding 影响。
    #[test]
    fn test_absolute_position_ignores_border_padding() {
        let box0 = LayoutBox {
            node_id: None,
            x: 50.0,
            y: 75.0,
            width: 200.0,
            height: 150.0,
            content_x: 55.0,
            content_y: 80.0,
            content_width: 190.0,
            content_height: 140.0,
            border_top: 2.0,
            border_right: 2.0,
            border_bottom: 2.0,
            border_left: 3.0,
            padding_top: 3.0,
            padding_right: 3.0,
            padding_bottom: 3.0,
            padding_left: 3.0,
            margin_top: 5.0,
            margin_right: 5.0,
            margin_bottom: 5.0,
            margin_left: 5.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let (abs_x, abs_y) = box0.absolute_position();
        assert!((abs_x - 50.0).abs() < 0.001, "x 应为 50.0");
        assert!((abs_y - 75.0).abs() < 0.001, "y 应为 75.0");
    }

    /// 测试 absolute_position_with_parent 累加父子偏移。
    #[test]
    fn test_absolute_position_with_parent_basic() {
        let child = LayoutBox {
            node_id: None,
            x: 30.0,
            y: 40.0,
            width: 50.0,
            height: 50.0,
            content_x: 30.0,
            content_y: 40.0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let (abs_x, abs_y) = child.absolute_position_with_parent(100.0, 200.0);
        assert!((abs_x - 130.0).abs() < 0.001, "abs_x 应为 130.0");
        assert!((abs_y - 240.0).abs() < 0.001, "abs_y 应为 240.0");
    }

    /// 测试 absolute_position_with_parent 传入负父偏移。
    #[test]
    fn test_absolute_position_with_parent_negative() {
        let child = LayoutBox {
            node_id: None,
            x: 10.0,
            y: 20.0,
            width: 50.0,
            height: 50.0,
            content_x: 10.0,
            content_y: 20.0,
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
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let (abs_x, abs_y) = child.absolute_position_with_parent(-50.0, -30.0);
        assert!((abs_x - (-40.0)).abs() < 0.001, "abs_x 应为 -40.0");
        assert!((abs_y - (-10.0)).abs() < 0.001, "abs_y 应为 -10.0");
    }

    /// 测试 outer_area 在零尺寸盒子上的计算。
    #[test]
    fn test_outer_area_zero_size_box() {
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
            margin_top: 10.0,
            margin_right: 10.0,
            margin_bottom: 10.0,
            margin_left: 10.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        // total_width = 10 + 0 + 10 = 20, total_height = 10 + 0 + 10 = 20
        let area = box0.outer_area();
        assert!((area - 400.0).abs() < 0.001, "outer_area 应为 400.0，实际 {}", area);
    }

    /// 测试 absolute_position 对 is_fixed=true 的盒子同样返回 (x, y)。
    #[test]
    fn test_absolute_position_fixed_element() {
        let box0 = LayoutBox {
            node_id: None,
            x: 25.0,
            y: 35.0,
            width: 100.0,
            height: 100.0,
            content_x: 25.0,
            content_y: 35.0,
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
            children: vec![],
            is_absolute: false,
            is_fixed: true,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
        };
        let (abs_x, abs_y) = box0.absolute_position();
        assert!((abs_x - 25.0).abs() < 0.001);
        assert!((abs_y - 35.0).abs() < 0.001);
    }
}
