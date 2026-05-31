//! 布局引擎协调器。
//!
//! [`LayoutEngine`] 接收 DOM 和计算样式，通过 taffy 计算布局，
//! 输出 [`LayoutResult`]（布局盒树）。

use std::collections::HashMap;
use taffy::prelude::*;
use zero_css_parser::values::{OverflowValue, PositionValue};
use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;

use crate::tree::build_layout_tree;
use crate::types::{LayoutBox, LayoutResult, OverflowClip};

/// 布局引擎 — 接收 DOM + 计算样式，输出布局盒树。
///
/// 使用 taffy 作为底层布局算法实现，支持 Block、Flexbox 和 Grid 布局。
pub struct LayoutEngine {
    /// 视口宽度。
    pub viewport_width: f32,
    /// 视口高度。
    pub viewport_height: f32,
}

impl LayoutEngine {
    /// 创建新的布局引擎实例。
    ///
    /// # 参数
    ///
    /// - `viewport_width` — 视口宽度（像素）
    /// - `viewport_height` — 视口高度（像素）
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
        }
    }

    /// 计算整个文档的布局。
    ///
    /// # 流程
    ///
    /// 1. 从 DOM + 计算样式构建 taffy 树
    /// 2. 使用 taffy 计算布局
    /// 3. 从 taffy 结果中提取 LayoutBox 树
    ///
    /// # 参数
    ///
    /// - `doc` — DOM 文档
    /// - `styles` — 元素 NodeId → ComputedStyle 映射
    pub fn compute(&self, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) -> LayoutResult {
        // 1. 构建 taffy 树
        let (mut taffy_tree, root_id, taffy_to_dom) =
            build_layout_tree(doc, styles, self.viewport_width, self.viewport_height);

        // 2. 计算布局
        let available_space = taffy::geometry::Size {
            width: AvailableSpace::Definite(self.viewport_width),
            height: AvailableSpace::Definite(self.viewport_height),
        };
        let _ = taffy_tree.compute_layout(root_id, available_space);

        // 3. 提取 LayoutBox 树
        let root_box = Self::extract_layout(&taffy_tree, root_id, &taffy_to_dom, styles);

        LayoutResult {
            root: root_box,
            viewport_width: self.viewport_width,
            viewport_height: self.viewport_height,
        }
    }

    /// 从 taffy 布局结果中提取 LayoutBox 树。
    fn extract_layout(
        taffy: &TaffyTree<()>,
        taffy_id: taffy::NodeId,
        taffy_to_dom: &HashMap<taffy::NodeId, NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> LayoutBox {
        let layout = taffy.layout(taffy_id).cloned().unwrap_or_default();
        let dom_id = taffy_to_dom.get(&taffy_id).copied();

        // 获取 ComputedStyle 用于提取定位和溢出信息
        let computed = dom_id.and_then(|id| styles.get(&id));

        let is_absolute = computed.is_some_and(|s| matches!(s.position, PositionValue::Absolute));
        let is_fixed = computed.is_some_and(|s| matches!(s.position, PositionValue::Fixed));
        let overflow_x = computed.map_or(OverflowClip::Visible, |s| convert_overflow_to_clip(&s.overflow_x));
        let overflow_y = computed.map_or(OverflowClip::Visible, |s| convert_overflow_to_clip(&s.overflow_y));

        // 计算内容区域
        let content_x = layout.location.x + layout.border.left + layout.padding.left;
        let content_y = layout.location.y + layout.border.top + layout.padding.top;
        let content_width =
            (layout.size.width - layout.border.left - layout.border.right - layout.padding.left - layout.padding.right)
                .max(0.0);
        let content_height = (layout.size.height
            - layout.border.top
            - layout.border.bottom
            - layout.padding.top
            - layout.padding.bottom)
            .max(0.0);

        // 递归提取子节点
        let children_taffy = taffy.children(taffy_id).unwrap_or_default();
        let mut children_boxes = Vec::with_capacity(children_taffy.len());
        for child_taffy in &children_taffy {
            children_boxes.push(Self::extract_layout(taffy, *child_taffy, taffy_to_dom, styles));
        }

        LayoutBox {
            node_id: dom_id,
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
            content_x,
            content_y,
            content_width,
            content_height,
            border_top: layout.border.top,
            border_right: layout.border.right,
            border_bottom: layout.border.bottom,
            border_left: layout.border.left,
            padding_top: layout.padding.top,
            padding_right: layout.padding.right,
            padding_bottom: layout.padding.bottom,
            padding_left: layout.padding.left,
            margin_top: layout.margin.top,
            margin_right: layout.margin.right,
            margin_bottom: layout.margin.bottom,
            margin_left: layout.margin.left,
            children: children_boxes,
            is_absolute,
            is_fixed,
            overflow_x,
            overflow_y,
        }
    }
}

/// 将 OverflowValue 转换为 OverflowClip。
fn convert_overflow_to_clip(value: &OverflowValue) -> OverflowClip {
    match value {
        OverflowValue::Visible => OverflowClip::Visible,
        OverflowValue::Hidden => OverflowClip::Hidden,
        OverflowValue::Clip => OverflowClip::Clip,
        OverflowValue::Scroll | OverflowValue::Auto => OverflowClip::Scroll,
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use zero_css_parser::values::{
        AlignmentValue, DisplayValue, FlexDirectionValue, FlexWrapValue, LengthValue, OverflowValue, PositionValue,
    };
    use zero_dom::Document;
    use zero_style_system::FlexBasisValue;

    /// 创建带指定 display 和 size 的 ComputedStyle。
    fn make_style_with_display(display: DisplayValue, width: f64, height: f64) -> ComputedStyle {
        let mut style = ComputedStyle::default();
        style.display = display;
        if width > 0.0 {
            style.width = LengthValue::Px(width);
        }
        if height > 0.0 {
            style.height = LengthValue::Px(height);
        }
        style
    }

    /// 创建 html > body 容器，返回 (doc, body_id)。
    fn make_doc_with_body() -> (Document, NodeId) {
        let mut doc = Document::new();
        let root = doc.root();
        let html = doc.create_element("html");
        doc.append_child(root, html).unwrap();
        let body = doc.create_element("body");
        doc.append_child(html, body).unwrap();
        (doc, body)
    }

    /// 测试简单 block 布局。
    #[test]
    fn test_compute_simple_block_layout() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let mut styles = HashMap::new();
        styles.insert(div, make_style_with_display(DisplayValue::Block, 100.0, 50.0));

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        assert!((result.viewport_width - 800.0).abs() < 0.001);
        assert!((result.viewport_height - 600.0).abs() < 0.001);
    }

    /// 测试 block 垂直堆叠。
    #[test]
    fn test_compute_block_vertical_stack() {
        let (mut doc, body) = make_doc_with_body();
        let mut div_ids = Vec::new();
        for _ in 0..3 {
            let div = doc.create_element("div");
            doc.append_child(body, div).unwrap();
            div_ids.push(div);
        }

        let mut styles = HashMap::new();
        for id in div_ids {
            styles.insert(id, make_style_with_display(DisplayValue::Block, 100.0, 30.0));
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 flex row 布局。
    #[test]
    fn test_compute_flex_row() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();

        let mut item_ids = Vec::new();
        for _ in 0..3 {
            let item = doc.create_element("span");
            doc.append_child(container, item).unwrap();
            item_ids.push(item);
        }

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.flex_direction = FlexDirectionValue::Row;
        container_style.width = LengthValue::Px(300.0);
        container_style.height = LengthValue::Px(100.0);
        styles.insert(container, container_style);

        for id in item_ids {
            let mut item_style = ComputedStyle::default();
            item_style.width = LengthValue::Px(80.0);
            item_style.height = LengthValue::Px(40.0);
            styles.insert(id, item_style);
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 flex column 布局。
    #[test]
    fn test_compute_flex_column() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(container, item2).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.flex_direction = FlexDirectionValue::Column;
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(200.0);
        styles.insert(container, container_style);

        for id in [item1, item2] {
            let mut item_style = ComputedStyle::default();
            item_style.width = LengthValue::Px(100.0);
            item_style.height = LengthValue::Px(50.0);
            styles.insert(id, item_style);
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 flex-grow。
    #[test]
    fn test_compute_flex_grow() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(container, item2).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(100.0);
        styles.insert(container, container_style);

        let mut item1_style = ComputedStyle::default();
        item1_style.flex_grow = 1.0;
        item1_style.height = LengthValue::Px(50.0);
        styles.insert(item1, item1_style);

        let mut item2_style = ComputedStyle::default();
        item2_style.flex_grow = 2.0;
        item2_style.height = LengthValue::Px(50.0);
        styles.insert(item2, item2_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 flex-wrap。
    #[test]
    fn test_compute_flex_wrap() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.flex_wrap = FlexWrapValue::Wrap;
        container_style.width = LengthValue::Px(100.0);
        container_style.height = LengthValue::Px(200.0);
        styles.insert(container, container_style);

        for _ in 0..5 {
            let item = doc.create_element("span");
            doc.append_child(container, item).unwrap();
            let mut item_style = ComputedStyle::default();
            item_style.width = LengthValue::Px(50.0);
            item_style.height = LengthValue::Px(30.0);
            styles.insert(item, item_style);
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 flex gap。
    #[test]
    fn test_compute_flex_gap() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(container, item2).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.gap = LengthValue::Px(10.0);
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(100.0);
        styles.insert(container, container_style);

        for id in [item1, item2] {
            let mut item_style = ComputedStyle::default();
            item_style.width = LengthValue::Px(50.0);
            item_style.height = LengthValue::Px(50.0);
            styles.insert(id, item_style);
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 flex 居中对齐。
    #[test]
    fn test_compute_flex_alignment_center() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item = doc.create_element("span");
        doc.append_child(container, item).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.justify_content = AlignmentValue::Center;
        container_style.align_items = AlignmentValue::Center;
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(200.0);
        styles.insert(container, container_style);

        let mut item_style = ComputedStyle::default();
        item_style.width = LengthValue::Px(50.0);
        item_style.height = LengthValue::Px(50.0);
        styles.insert(item, item_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 space-between 对齐。
    #[test]
    fn test_compute_flex_space_between() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(container, item2).unwrap();
        let item3 = doc.create_element("span");
        doc.append_child(container, item3).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.justify_content = AlignmentValue::SpaceBetween;
        container_style.width = LengthValue::Px(300.0);
        container_style.height = LengthValue::Px(100.0);
        styles.insert(container, container_style);

        for id in [item1, item2, item3] {
            let mut item_style = ComputedStyle::default();
            item_style.width = LengthValue::Px(50.0);
            item_style.height = LengthValue::Px(50.0);
            styles.insert(id, item_style);
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 grid 基本布局。
    #[test]
    fn test_compute_grid_basic() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(grid, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(grid, item2).unwrap();

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.width = LengthValue::Px(200.0);
        grid_style.height = LengthValue::Px(200.0);
        styles.insert(grid, grid_style);

        for id in [item1, item2] {
            styles.insert(id, ComputedStyle::default());
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 grid 带 template。
    #[test]
    fn test_compute_grid_with_template() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(grid, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(grid, item2).unwrap();
        let item3 = doc.create_element("span");
        doc.append_child(grid, item3).unwrap();
        let item4 = doc.create_element("span");
        doc.append_child(grid, item4).unwrap();

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.width = LengthValue::Px(200.0);
        grid_style.height = LengthValue::Px(200.0);
        styles.insert(grid, grid_style);

        for id in [item1, item2, item3, item4] {
            styles.insert(id, ComputedStyle::default());
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试绝对定位。
    #[test]
    fn test_compute_absolute_position() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let abs_child = doc.create_element("span");
        doc.append_child(container, abs_child).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(200.0);
        styles.insert(container, container_style);

        let mut abs_style = ComputedStyle::default();
        abs_style.position = PositionValue::Absolute;
        abs_style.top = LengthValue::Px(10.0);
        abs_style.left = LengthValue::Px(20.0);
        abs_style.width = LengthValue::Px(50.0);
        abs_style.height = LengthValue::Px(50.0);
        styles.insert(abs_child, abs_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试相对定位。
    #[test]
    fn test_compute_relative_position() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let rel_child = doc.create_element("span");
        doc.append_child(container, rel_child).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(100.0);
        styles.insert(container, container_style);

        let mut rel_style = ComputedStyle::default();
        rel_style.position = PositionValue::Relative;
        rel_style.top = LengthValue::Px(5.0);
        rel_style.left = LengthValue::Px(5.0);
        rel_style.width = LengthValue::Px(50.0);
        rel_style.height = LengthValue::Px(50.0);
        styles.insert(rel_child, rel_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 overflow hidden。
    #[test]
    fn test_compute_overflow_hidden() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.overflow_x = OverflowValue::Hidden;
        container_style.overflow_y = OverflowValue::Scroll;
        container_style.width = LengthValue::Px(100.0);
        container_style.height = LengthValue::Px(100.0);
        styles.insert(container, container_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试嵌套 flex。
    #[test]
    fn test_compute_nested_flex() {
        let (mut doc, body) = make_doc_with_body();
        let outer = doc.create_element("div");
        doc.append_child(body, outer).unwrap();
        let inner = doc.create_element("div");
        doc.append_child(outer, inner).unwrap();
        let item = doc.create_element("span");
        doc.append_child(inner, item).unwrap();

        let mut styles = HashMap::new();
        let mut outer_style = ComputedStyle::default();
        outer_style.display = DisplayValue::Flex;
        outer_style.flex_direction = FlexDirectionValue::Column;
        outer_style.width = LengthValue::Px(200.0);
        outer_style.height = LengthValue::Px(200.0);
        styles.insert(outer, outer_style);

        let mut inner_style = ComputedStyle::default();
        inner_style.display = DisplayValue::Flex;
        inner_style.flex_direction = FlexDirectionValue::Row;
        styles.insert(inner, inner_style);

        let mut item_style = ComputedStyle::default();
        item_style.width = LengthValue::Px(50.0);
        item_style.height = LengthValue::Px(50.0);
        styles.insert(item, item_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 padding 效果。
    #[test]
    fn test_compute_padding_effect() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let child = doc.create_element("span");
        doc.append_child(container, child).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(200.0);
        container_style.padding_top = LengthValue::Px(10.0);
        container_style.padding_left = LengthValue::Px(10.0);
        styles.insert(container, container_style);

        let mut child_style = ComputedStyle::default();
        child_style.width = LengthValue::Px(100.0);
        child_style.height = LengthValue::Px(100.0);
        styles.insert(child, child_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 border 效果。
    #[test]
    fn test_compute_border_effect() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(200.0);
        container_style.border_top_width = LengthValue::Px(5.0);
        container_style.border_bottom_width = LengthValue::Px(5.0);
        container_style.border_left_width = LengthValue::Px(5.0);
        container_style.border_right_width = LengthValue::Px(5.0);
        styles.insert(container, container_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 margin 效果。
    #[test]
    fn test_compute_margin_effect() {
        let (mut doc, body) = make_doc_with_body();
        let child = doc.create_element("div");
        doc.append_child(body, child).unwrap();

        let mut styles = HashMap::new();
        let mut child_style = ComputedStyle::default();
        child_style.width = LengthValue::Px(100.0);
        child_style.height = LengthValue::Px(100.0);
        child_style.margin_top = LengthValue::Px(20.0);
        child_style.margin_left = LengthValue::Px(20.0);
        styles.insert(child, child_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试 min/max size。
    #[test]
    fn test_compute_min_max_size() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let mut styles = HashMap::new();
        let mut div_style = ComputedStyle::default();
        div_style.min_width = LengthValue::Px(100.0);
        div_style.max_width = LengthValue::Px(300.0);
        div_style.min_height = LengthValue::Px(50.0);
        div_style.max_height = LengthValue::Px(200.0);
        styles.insert(div, div_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width > 0.0);
    }

    /// 测试零尺寸元素。
    #[test]
    fn test_compute_zero_size_element() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let mut styles = HashMap::new();
        styles.insert(div, ComputedStyle::default());

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        // 即使没有显式尺寸，布局也应成功
        assert!(result.root.width >= 0.0);
    }

    // ── 几何验证补充测试 ──

    /// 查找 body 的第一个子元素在布局树中的位置。
    fn find_child_by_node_id(root: &LayoutBox, target_id: NodeId) -> Option<&LayoutBox> {
        for child in &root.children {
            if child.node_id == Some(target_id) {
                return Some(child);
            }
            if let Some(found) = find_child_by_node_id(child, target_id) {
                return Some(found);
            }
        }
        None
    }

    /// 验证 block 布局中子元素的正确尺寸和位置。
    #[test]
    fn test_block_child_exact_geometry() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let mut styles = HashMap::new();
        styles.insert(div, make_style_with_display(DisplayValue::Block, 200.0, 100.0));

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let div_box = find_child_by_node_id(&result.root, div).expect("div found");
        // div 的宽度应该是 200，高度 100
        assert_eq!(div_box.width, 200.0, "div width should be 200px");
        assert_eq!(div_box.height, 100.0, "div height should be 100px");
    }

    /// 验证 padding 出现在布局盒中（taffy 默认 content-box：padding 增加总尺寸）。
    #[test]
    fn test_padding_values_in_layout() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let mut styles = HashMap::new();
        let mut div_style = ComputedStyle::default();
        div_style.display = DisplayValue::Block;
        div_style.width = LengthValue::Px(200.0);
        div_style.height = LengthValue::Px(100.0);
        div_style.padding_top = LengthValue::Px(10.0);
        div_style.padding_bottom = LengthValue::Px(10.0);
        div_style.padding_left = LengthValue::Px(20.0);
        div_style.padding_right = LengthValue::Px(20.0);
        styles.insert(div, div_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let div_box = find_child_by_node_id(&result.root, div).expect("div found");
        assert_eq!(div_box.padding_left, 20.0);
        assert_eq!(div_box.padding_right, 20.0);
        assert_eq!(div_box.padding_top, 10.0);
        assert_eq!(div_box.padding_bottom, 10.0);
        // 总宽度 = width + padding_left + padding_right
        assert_eq!(div_box.width, 240.0, "total width = 200 + 20 + 20");
        // 内容区域 = width（content-box 模式）
        assert_eq!(div_box.content_width, 200.0, "content width = 200 (content-box)");
    }

    /// 验证 border 正确出现在布局盒中。
    #[test]
    fn test_border_values_in_layout() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let mut styles = HashMap::new();
        let mut div_style = ComputedStyle::default();
        div_style.display = DisplayValue::Block;
        div_style.width = LengthValue::Px(200.0);
        div_style.height = LengthValue::Px(100.0);
        div_style.border_top_width = LengthValue::Px(5.0);
        div_style.border_bottom_width = LengthValue::Px(5.0);
        div_style.border_left_width = LengthValue::Px(10.0);
        div_style.border_right_width = LengthValue::Px(10.0);
        styles.insert(div, div_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let div_box = find_child_by_node_id(&result.root, div).expect("div found");
        assert_eq!(div_box.border_top, 5.0);
        assert_eq!(div_box.border_bottom, 5.0);
        assert_eq!(div_box.border_left, 10.0);
        assert_eq!(div_box.border_right, 10.0);
        // 总宽度 = width + border_left + border_right
        assert_eq!(div_box.width, 220.0, "total width = 200 + 10 + 10");
    }

    /// 验证两个 block 子元素垂直堆叠。
    #[test]
    fn test_block_stack_y_positions() {
        let (mut doc, body) = make_doc_with_body();
        let div1 = doc.create_element("div");
        doc.append_child(body, div1).unwrap();
        let div2 = doc.create_element("div");
        doc.append_child(body, div2).unwrap();

        let mut styles = HashMap::new();
        styles.insert(div1, make_style_with_display(DisplayValue::Block, 100.0, 50.0));
        styles.insert(div2, make_style_with_display(DisplayValue::Block, 100.0, 50.0));

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let box1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
        let box2 = find_child_by_node_id(&result.root, div2).expect("div2 found");

        // div2 应在 div1 下方
        assert!(
            box2.y >= box1.y + box1.height,
            "div2 (y={}) should be below div1 (y={}, h={})",
            box2.y,
            box1.y,
            box1.height
        );
    }

    /// 验证 flex 行中子元素水平排列。
    #[test]
    fn test_flex_row_children_horizontal() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(container, item2).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.width = LengthValue::Px(400.0);
        container_style.height = LengthValue::Px(100.0);
        styles.insert(container, container_style);
        styles.insert(item1, make_style_with_display(DisplayValue::Block, 100.0, 50.0));
        styles.insert(item2, make_style_with_display(DisplayValue::Block, 100.0, 50.0));

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let box1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let box2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

        // item2 应在 item1 右侧
        assert!(
            box2.x > box1.x,
            "item2 (x={}) should be right of item1 (x={})",
            box2.x,
            box1.x
        );
    }

    /// 验证 overflow 属性正确传递。
    #[test]
    fn test_overflow_values_propagated() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let mut styles = HashMap::new();
        let mut div_style = ComputedStyle::default();
        div_style.display = DisplayValue::Block;
        div_style.overflow_x = OverflowValue::Hidden;
        div_style.overflow_y = OverflowValue::Scroll;
        div_style.width = LengthValue::Px(100.0);
        styles.insert(div, div_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let div_box = find_child_by_node_id(&result.root, div).expect("div found");
        assert_eq!(div_box.overflow_x, OverflowClip::Hidden);
        assert_eq!(div_box.overflow_y, OverflowClip::Scroll);
    }

    /// 验证空 DOM 文档布局不 panic。
    #[test]
    fn test_layout_empty_document() {
        let doc = Document::new();
        let styles = HashMap::new();
        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        assert!(result.root.width >= 0.0);
    }

    /// 验证绝对定位元素标记正确。
    #[test]
    fn test_absolute_position_flag() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let abs_child = doc.create_element("span");
        doc.append_child(container, abs_child).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(200.0);
        styles.insert(container, container_style);

        let mut abs_style = ComputedStyle::default();
        abs_style.position = PositionValue::Absolute;
        abs_style.width = LengthValue::Px(50.0);
        abs_style.height = LengthValue::Px(50.0);
        styles.insert(abs_child, abs_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs found");
        assert!(abs_box.is_absolute, "should be flagged as absolute");
        assert_eq!(abs_box.width, 50.0);
        assert_eq!(abs_box.height, 50.0);
    }

    // ── 新增集成测试 ──

    /// 测试 block 布局中嵌套元素的几何位置正确。
    ///
    /// 结构：body > div(200x300) > div(100x150)
    /// 内部 div 应在外部 div 的内容区域中定位。
    #[test]
    fn test_block_nested_element_geometry() {
        let (mut doc, body) = make_doc_with_body();
        let outer = doc.create_element("div");
        doc.append_child(body, outer).unwrap();
        let inner = doc.create_element("div");
        doc.append_child(outer, inner).unwrap();

        let mut styles = HashMap::new();
        styles.insert(outer, make_style_with_display(DisplayValue::Block, 200.0, 300.0));
        styles.insert(inner, make_style_with_display(DisplayValue::Block, 100.0, 150.0));

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let outer_box = find_child_by_node_id(&result.root, outer).expect("outer found");
        let inner_box = find_child_by_node_id(&result.root, inner).expect("inner found");

        assert_eq!(outer_box.width, 200.0, "外层 div 宽度应为 200");
        assert_eq!(outer_box.height, 300.0, "外层 div 高度应为 300");
        assert_eq!(inner_box.width, 100.0, "内层 div 宽度应为 100");
        assert_eq!(inner_box.height, 150.0, "内层 div 高度应为 150");

        // 内层 div 应在外层 div 内部
        assert!(inner_box.x >= outer_box.content_x, "内层 x 应 >= 外层内容区域 x");
    }

    /// 测试三层嵌套 block 布局。
    ///
    /// body > div > div > div，每层尺寸递减。
    #[test]
    fn test_block_deep_nesting() {
        let (mut doc, body) = make_doc_with_body();
        let d1 = doc.create_element("div");
        doc.append_child(body, d1).unwrap();
        let d2 = doc.create_element("div");
        doc.append_child(d1, d2).unwrap();
        let d3 = doc.create_element("div");
        doc.append_child(d2, d3).unwrap();

        let mut styles = HashMap::new();
        styles.insert(d1, make_style_with_display(DisplayValue::Block, 600.0, 400.0));
        styles.insert(d2, make_style_with_display(DisplayValue::Block, 400.0, 200.0));
        styles.insert(d3, make_style_with_display(DisplayValue::Block, 200.0, 100.0));

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, d1).expect("d1 found");
        let b2 = find_child_by_node_id(&result.root, d2).expect("d2 found");
        let b3 = find_child_by_node_id(&result.root, d3).expect("d3 found");

        assert_eq!(b1.width, 600.0);
        assert_eq!(b2.width, 400.0);
        assert_eq!(b3.width, 200.0);
        assert_eq!(b3.height, 100.0);
    }

    /// 测试 block 布局中多个子元素垂直堆叠，间距精确。
    #[test]
    fn test_block_stack_with_margin() {
        let (mut doc, body) = make_doc_with_body();
        let div1 = doc.create_element("div");
        doc.append_child(body, div1).unwrap();
        let div2 = doc.create_element("div");
        doc.append_child(body, div2).unwrap();

        let mut styles = HashMap::new();
        let mut style1 = make_style_with_display(DisplayValue::Block, 100.0, 50.0);
        style1.margin_bottom = LengthValue::Px(20.0);
        styles.insert(div1, style1);
        styles.insert(div2, make_style_with_display(DisplayValue::Block, 100.0, 50.0));

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let box1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
        let box2 = find_child_by_node_id(&result.root, div2).expect("div2 found");

        // div2 应在 div1 底部 + margin_bottom 之后
        let expected_y = box1.y + box1.height + box1.margin_bottom;
        assert!(
            (box2.y - expected_y).abs() < 0.01,
            "div2.y ({}) 应等于 div1.y({}) + div1.height({}) + margin_bottom({}) = {}",
            box2.y,
            box1.y,
            box1.height,
            box1.margin_bottom,
            expected_y
        );
    }

    /// 测试 flex-direction: row — 子元素水平排列。
    #[test]
    fn test_flex_row_direction_layout() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(container, item2).unwrap();
        let item3 = doc.create_element("span");
        doc.append_child(container, item3).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.flex_direction = FlexDirectionValue::Row;
        container_style.width = LengthValue::Px(300.0);
        container_style.height = LengthValue::Px(100.0);
        styles.insert(container, container_style);

        for id in [item1, item2, item3] {
            styles.insert(id, make_style_with_display(DisplayValue::Block, 80.0, 40.0));
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");
        let b3 = find_child_by_node_id(&result.root, item3).expect("item3 found");

        // Row 方向：三个元素应水平排列，x 递增
        assert!(b2.x > b1.x, "item2 应在 item1 右侧");
        assert!(b3.x > b2.x, "item3 应在 item2 右侧");

        // y 应相同（同一行）
        assert!(
            (b1.y - b2.y).abs() < 0.01 && (b2.y - b3.y).abs() < 0.01,
            "三个元素应在同一行"
        );
    }

    /// 测试 flex-direction: column — 子元素垂直排列。
    #[test]
    fn test_flex_column_direction_layout() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(container, item2).unwrap();
        let item3 = doc.create_element("span");
        doc.append_child(container, item3).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.flex_direction = FlexDirectionValue::Column;
        container_style.width = LengthValue::Px(300.0);
        container_style.height = LengthValue::Px(200.0);
        styles.insert(container, container_style);

        for id in [item1, item2, item3] {
            styles.insert(id, make_style_with_display(DisplayValue::Block, 80.0, 40.0));
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");
        let b3 = find_child_by_node_id(&result.root, item3).expect("item3 found");

        // Column 方向：三个元素应垂直排列，y 递增
        assert!(b2.y > b1.y, "item2 应在 item1 下方");
        assert!(b3.y > b2.y, "item3 应在 item2 下方");

        // x 应相同（同一列）
        assert!(
            (b1.x - b2.x).abs() < 0.01 && (b2.x - b3.x).abs() < 0.01,
            "三个元素应在同一列"
        );
    }

    /// 测试 flex-direction: row-reverse — 子元素反向水平排列。
    #[test]
    fn test_flex_row_reverse_direction() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(container, item2).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.flex_direction = FlexDirectionValue::RowReverse;
        container_style.width = LengthValue::Px(300.0);
        container_style.height = LengthValue::Px(100.0);
        styles.insert(container, container_style);

        for id in [item1, item2] {
            styles.insert(id, make_style_with_display(DisplayValue::Block, 80.0, 40.0));
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

        // Row-reverse：item1 在右，item2 在左
        assert!(b2.x < b1.x, "row-reverse 中 item2 应在 item1 左侧（x 更小）");
    }

    /// 测试 flex-direction: column-reverse — 子元素反向垂直排列。
    #[test]
    fn test_flex_column_reverse_direction() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(container, item2).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.flex_direction = FlexDirectionValue::ColumnReverse;
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(200.0);
        styles.insert(container, container_style);

        for id in [item1, item2] {
            styles.insert(id, make_style_with_display(DisplayValue::Block, 80.0, 40.0));
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

        // Column-reverse：item1 在下方，item2 在上方
        assert!(b2.y < b1.y, "column-reverse 中 item2 应在 item1 上方（y 更小）");
    }

    /// 测试 Grid 布局中显式的行/列放置。
    ///
    /// 2x2 grid，显式指定每个子元素的 grid-row/grid-column。
    #[test]
    fn test_grid_explicit_placement() {
        use zero_style_system::GridLineValue;

        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        let item1 = doc.create_element("span");
        doc.append_child(grid, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(grid, item2).unwrap();
        let item3 = doc.create_element("span");
        doc.append_child(grid, item3).unwrap();
        let item4 = doc.create_element("span");
        doc.append_child(grid, item4).unwrap();

        let mut styles = HashMap::new();

        // 2 列 2 行的 grid
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("100px 100px".to_string());
        grid_style.grid_template_rows = Some("50px 50px".to_string());
        grid_style.width = LengthValue::Px(200.0);
        grid_style.height = LengthValue::Px(100.0);
        styles.insert(grid, grid_style);

        // item1: row 1, col 1
        let mut item1_style = ComputedStyle::default();
        item1_style.grid_row_start = GridLineValue::Line(1);
        item1_style.grid_row_end = GridLineValue::Line(2);
        item1_style.grid_column_start = GridLineValue::Line(1);
        item1_style.grid_column_end = GridLineValue::Line(2);
        styles.insert(item1, item1_style);

        // item2: row 1, col 2
        let mut item2_style = ComputedStyle::default();
        item2_style.grid_row_start = GridLineValue::Line(1);
        item2_style.grid_row_end = GridLineValue::Line(2);
        item2_style.grid_column_start = GridLineValue::Line(2);
        item2_style.grid_column_end = GridLineValue::Line(3);
        styles.insert(item2, item2_style);

        // item3: row 2, col 1
        let mut item3_style = ComputedStyle::default();
        item3_style.grid_row_start = GridLineValue::Line(2);
        item3_style.grid_row_end = GridLineValue::Line(3);
        item3_style.grid_column_start = GridLineValue::Line(1);
        item3_style.grid_column_end = GridLineValue::Line(2);
        styles.insert(item3, item3_style);

        // item4: row 2, col 2
        let mut item4_style = ComputedStyle::default();
        item4_style.grid_row_start = GridLineValue::Line(2);
        item4_style.grid_row_end = GridLineValue::Line(3);
        item4_style.grid_column_start = GridLineValue::Line(2);
        item4_style.grid_column_end = GridLineValue::Line(3);
        styles.insert(item4, item4_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");
        let b3 = find_child_by_node_id(&result.root, item3).expect("item3 found");
        let b4 = find_child_by_node_id(&result.root, item4).expect("item4 found");

        // item1 (0,0) vs item2 (0,1): item2 应在 item1 右侧
        assert!(
            b2.x > b1.x,
            "item2 (col 2) 应在 item1 (col 1) 右侧: {} vs {}",
            b2.x,
            b1.x
        );

        // item1 (0,0) vs item3 (1,0): item3 应在 item1 下方
        assert!(
            b3.y > b1.y,
            "item3 (row 2) 应在 item1 (row 1) 下方: {} vs {}",
            b3.y,
            b1.y
        );

        // item4 (1,1) 应在 item3 (1,0) 右侧
        assert!(
            b4.x > b3.x,
            "item4 (col 2) 应在 item3 (col 1) 右侧: {} vs {}",
            b4.x,
            b3.x
        );

        // 所有格子宽度应约 100px
        assert!(
            (b1.width - 100.0).abs() < 1.0,
            "item1 宽度应约 100px，实际 {}",
            b1.width
        );
        assert!(
            (b4.width - 100.0).abs() < 1.0,
            "item4 宽度应约 100px，实际 {}",
            b4.width
        );
    }

    /// 测试 Grid 布局中 span 放置。
    #[test]
    fn test_grid_span_placement() {
        use zero_style_system::GridLineValue;

        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        let wide_item = doc.create_element("span");
        doc.append_child(grid, wide_item).unwrap();
        let normal_item = doc.create_element("span");
        doc.append_child(grid, normal_item).unwrap();

        let mut styles = HashMap::new();

        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("100px 100px 100px".to_string());
        grid_style.grid_template_rows = Some("50px".to_string());
        grid_style.width = LengthValue::Px(300.0);
        grid_style.height = LengthValue::Px(50.0);
        styles.insert(grid, grid_style);

        // wide_item: 跨两列
        let mut wide_style = ComputedStyle::default();
        wide_style.grid_column_start = GridLineValue::Line(1);
        wide_style.grid_column_end = GridLineValue::Span(2);
        wide_style.grid_row_start = GridLineValue::Line(1);
        wide_style.grid_row_end = GridLineValue::Line(2);
        styles.insert(wide_item, wide_style);

        // normal_item: 一列
        let mut normal_style = ComputedStyle::default();
        normal_style.grid_column_start = GridLineValue::Line(3);
        normal_style.grid_column_end = GridLineValue::Line(4);
        normal_style.grid_row_start = GridLineValue::Line(1);
        normal_style.grid_row_end = GridLineValue::Line(2);
        styles.insert(normal_item, normal_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let wide_box = find_child_by_node_id(&result.root, wide_item).expect("wide found");
        let normal_box = find_child_by_node_id(&result.root, normal_item).expect("normal found");

        // 宽元素应跨两列（约 200px）
        assert!(
            wide_box.width > normal_box.width,
            "跨两列元素应比单列元素宽: {} vs {}",
            wide_box.width,
            normal_box.width
        );
        assert!(
            (wide_box.width - 200.0).abs() < 1.0,
            "跨两列宽度应约 200px，实际 {}",
            wide_box.width
        );
        assert!(
            (normal_box.width - 100.0).abs() < 1.0,
            "单列宽度应约 100px，实际 {}",
            normal_box.width
        );

        // 两个元素应在同一行
        assert!((wide_box.y - normal_box.y).abs() < 0.01, "同行元素 y 应相同");
    }

    /// 测试 Grid 布局中 fr 单位轨道。
    #[test]
    fn test_grid_fr_tracks() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        let item1 = doc.create_element("span");
        doc.append_child(grid, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(grid, item2).unwrap();

        let mut styles = HashMap::new();

        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("1fr 2fr".to_string());
        grid_style.grid_template_rows = Some("100px".to_string());
        grid_style.width = LengthValue::Px(300.0);
        grid_style.height = LengthValue::Px(100.0);
        styles.insert(grid, grid_style);

        styles.insert(item1, ComputedStyle::default());
        styles.insert(item2, ComputedStyle::default());

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

        // 1fr : 2fr = 100px : 200px
        assert!((b1.width - 100.0).abs() < 1.0, "1fr 应约 100px，实际 {}", b1.width);
        assert!((b2.width - 200.0).abs() < 1.0, "2fr 应约 200px，实际 {}", b2.width);
    }

    // ── 边缘场景和真实世界补充测试 ──

    // -- Block layout edge cases --

    /// 深度嵌套 block 布局（12 层），验证每层尺寸递减且布局不 panic。
    #[test]
    fn test_block_deeply_nested_12_levels() {
        let (mut doc, body) = make_doc_with_body();

        let mut ids: Vec<NodeId> = Vec::new();
        let mut parent = body;
        for _ in 0..12 {
            let div = doc.create_element("div");
            doc.append_child(parent, div).unwrap();
            ids.push(div);
            parent = div;
        }

        let mut styles = HashMap::new();
        for (i, &id) in ids.iter().enumerate() {
            let size = 600.0 - (i as f64) * 45.0;
            styles.insert(id, make_style_with_display(DisplayValue::Block, size, size * 0.6));
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 最外层应有正确宽度
        let outer = find_child_by_node_id(&result.root, ids[0]).expect("outer found");
        assert!(
            (outer.width - 600.0).abs() < 1.0,
            "outer width should be ~600, got {}",
            outer.width
        );

        // 最内层应有正确宽度
        let innermost = find_child_by_node_id(&result.root, ids[11]).expect("innermost found");
        let expected_inner = 600.0 - 11.0 * 45.0; // 105
        assert!(
            (innermost.width - expected_inner).abs() < 1.0,
            "innermost width should be ~{}, got {}",
            expected_inner,
            innermost.width
        );
    }

    /// Block 布局中包含显式零宽度子元素。
    /// 验证 layout engine 不 panic 且几何值合理。
    #[test]
    fn test_block_zero_width_children() {
        let (mut doc, body) = make_doc_with_body();
        let div1 = doc.create_element("div");
        doc.append_child(body, div1).unwrap();
        let div2 = doc.create_element("div");
        doc.append_child(body, div2).unwrap();
        let div3 = doc.create_element("div");
        doc.append_child(body, div3).unwrap();

        let mut styles = HashMap::new();
        // div1: 显式零宽度，有高度
        let mut s1 = ComputedStyle::default();
        s1.display = DisplayValue::Block;
        s1.width = LengthValue::Px(0.0);
        s1.height = LengthValue::Px(50.0);
        styles.insert(div1, s1);
        // div2: 正常尺寸
        styles.insert(div2, make_style_with_display(DisplayValue::Block, 200.0, 50.0));
        // div3: 零尺寸
        styles.insert(div3, make_style_with_display(DisplayValue::Block, 0.0, 0.0));

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
        let b2 = find_child_by_node_id(&result.root, div2).expect("div2 found");
        let b3 = find_child_by_node_id(&result.root, div3).expect("div3 found");

        // div1: block 元素即使设置 width:0，taffy 可能将其拉伸到容器宽度。
        // 无论如何高度应有效
        assert!(b1.height >= 0.0, "div1 height should be non-negative");

        // div2 正常尺寸
        assert_eq!(b2.width, 200.0);
        assert_eq!(b2.height, 50.0);

        // 垂直堆叠顺序：div2 在 div1 之后
        assert!(b2.y >= b1.y, "div2 should be at or below div1");
        assert!(b3.y >= b2.y, "div3 should be at or below div2");
    }

    /// Block 布局中负 margin 造成元素重叠。
    #[test]
    fn test_block_negative_margin_overlap() {
        let (mut doc, body) = make_doc_with_body();
        let div1 = doc.create_element("div");
        doc.append_child(body, div1).unwrap();
        let div2 = doc.create_element("div");
        doc.append_child(body, div2).unwrap();

        let mut styles = HashMap::new();
        let mut style1 = make_style_with_display(DisplayValue::Block, 100.0, 60.0);
        style1.margin_bottom = LengthValue::Px(-20.0);
        styles.insert(div1, style1);

        let mut style2 = make_style_with_display(DisplayValue::Block, 100.0, 60.0);
        style2.margin_top = LengthValue::Px(-10.0);
        styles.insert(div2, style2);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, div1).expect("div1 found");
        let b2 = find_child_by_node_id(&result.root, div2).expect("div2 found");

        // 负 margin 应让 div2 向上移动，与 div1 重叠
        assert!(
            b2.y < b1.y + b1.height,
            "negative margin should cause overlap: b2.y({}) < b1.y({}) + b1.height({})",
            b2.y,
            b1.y,
            b1.height
        );
    }

    /// Block 布局中多元素不同高度，验证总高度累加正确。
    #[test]
    fn test_block_varying_heights_stack() {
        let (mut doc, body) = make_doc_with_body();
        let d1 = doc.create_element("div");
        doc.append_child(body, d1).unwrap();
        let d2 = doc.create_element("div");
        doc.append_child(body, d2).unwrap();
        let d3 = doc.create_element("div");
        doc.append_child(body, d3).unwrap();

        let mut styles = HashMap::new();
        styles.insert(d1, make_style_with_display(DisplayValue::Block, 100.0, 30.0));
        styles.insert(d2, make_style_with_display(DisplayValue::Block, 100.0, 50.0));
        styles.insert(d3, make_style_with_display(DisplayValue::Block, 100.0, 20.0));

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, d1).expect("d1 found");
        let b2 = find_child_by_node_id(&result.root, d2).expect("d2 found");
        let b3 = find_child_by_node_id(&result.root, d3).expect("d3 found");

        // d2 应紧跟 d1
        assert!(
            (b2.y - (b1.y + b1.height)).abs() < 0.01,
            "d2.y({}) should equal d1.y({}) + d1.height({})",
            b2.y,
            b1.y,
            b1.height
        );

        // d3 应紧跟 d2
        assert!(
            (b3.y - (b2.y + b2.height)).abs() < 0.01,
            "d3.y({}) should equal d2.y({}) + d2.height({})",
            b3.y,
            b2.y,
            b2.height
        );
    }

    // -- Flex layout edge cases --

    /// flex-wrap: wrap 时，超出容器宽度的子元素换行到下一行。
    #[test]
    fn test_flex_wrap_multi_line() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();

        let mut item_ids = Vec::new();
        for _ in 0..4 {
            let item = doc.create_element("span");
            doc.append_child(container, item).unwrap();
            item_ids.push(item);
        }

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.flex_wrap = FlexWrapValue::Wrap;
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(200.0);
        styles.insert(container, container_style);

        // 每个item 120px宽，容器 200px → 第一个就快满了，第二个换行
        for id in &item_ids {
            let mut s = ComputedStyle::default();
            s.width = LengthValue::Px(120.0);
            s.height = LengthValue::Px(50.0);
            styles.insert(*id, s);
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
        let b1 = find_child_by_node_id(&result.root, item_ids[1]).expect("item1 found");

        // item1 应在 item0 下方（换行）
        assert!(
            b1.y > b0.y,
            "wrapped item1 (y={}) should be below item0 (y={})",
            b1.y,
            b0.y
        );
    }

    /// flex-grow 在有不同 flex-basis 的子元素上分配剩余空间。
    #[test]
    fn test_flex_grow_with_varying_basis() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(container, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(container, item2).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.width = LengthValue::Px(400.0);
        container_style.height = LengthValue::Px(100.0);
        styles.insert(container, container_style);

        // item1: basis 100px, grow 1
        let mut s1 = ComputedStyle::default();
        s1.flex_basis = FlexBasisValue::Length(LengthValue::Px(100.0));
        s1.flex_grow = 1.0;
        s1.height = LengthValue::Px(50.0);
        styles.insert(item1, s1);

        // item2: basis 100px, grow 2
        let mut s2 = ComputedStyle::default();
        s2.flex_basis = FlexBasisValue::Length(LengthValue::Px(100.0));
        s2.flex_grow = 2.0;
        s2.height = LengthValue::Px(50.0);
        styles.insert(item2, s2);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

        // 剩余空间 = 400 - 100 - 100 = 200px
        // item1 额外 = 200 * 1/3 ≈ 66.67, total ≈ 166.67
        // item2 额外 = 200 * 2/3 ≈ 133.33, total ≈ 233.33
        let total = b1.width + b2.width;
        assert!(
            (total - 400.0).abs() < 1.0,
            "items should fill container: total={}",
            total
        );
        assert!(
            b2.width > b1.width,
            "item2 (grow=2) should be wider than item1 (grow=1): {} vs {}",
            b2.width,
            b1.width
        );
    }

    /// align-items: stretch 使子元素拉伸到容器高度。
    #[test]
    fn test_flex_align_items_stretch() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let item = doc.create_element("span");
        doc.append_child(container, item).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.align_items = AlignmentValue::Stretch;
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(150.0);
        styles.insert(container, container_style);

        // item 只有宽度，没有高度 → stretch 应使其拉伸到 150px
        let mut item_style = ComputedStyle::default();
        item_style.width = LengthValue::Px(80.0);
        styles.insert(item, item_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let item_box = find_child_by_node_id(&result.root, item).expect("item found");
        assert!(
            (item_box.height - 150.0).abs() < 1.0,
            "stretch item height should be ~150, got {}",
            item_box.height
        );
    }

    /// Flex 容器中很多子项导致溢出。
    #[test]
    fn test_flex_many_items_overflow() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();

        let mut item_ids = Vec::new();
        for _ in 0..10 {
            let item = doc.create_element("span");
            doc.append_child(container, item).unwrap();
            item_ids.push(item);
        }

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.width = LengthValue::Px(300.0);
        container_style.height = LengthValue::Px(50.0);
        styles.insert(container, container_style);

        // 每项 50px 宽 × 10 = 500px，超出 300px 容器
        for id in &item_ids {
            let mut s = ComputedStyle::default();
            s.width = LengthValue::Px(50.0);
            s.height = LengthValue::Px(30.0);
            s.flex_shrink = 0.0; // 不收缩
            styles.insert(*id, s);
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 布局不应 panic
        let container_box = find_child_by_node_id(&result.root, container).expect("container found");
        assert_eq!(container_box.width, 300.0, "container width should stay 300");

        // 所有 item 都应存在
        let first = find_child_by_node_id(&result.root, item_ids[0]).expect("first found");
        assert_eq!(first.width, 50.0);
        let last = find_child_by_node_id(&result.root, item_ids[9]).expect("last found");
        assert_eq!(last.width, 50.0);

        // 最后一项应在第一项右侧很远
        assert!(last.x > first.x + 200.0, "last item should overflow past container");
    }

    // -- Grid layout edge cases --

    /// Grid 中行和列同时 span。
    #[test]
    fn test_grid_row_and_column_span() {
        use zero_style_system::GridLineValue;

        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();
        let big = doc.create_element("span");
        doc.append_child(grid, big).unwrap();
        let small = doc.create_element("span");
        doc.append_child(grid, small).unwrap();

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("100px 100px 100px".to_string());
        grid_style.grid_template_rows = Some("50px 50px".to_string());
        grid_style.width = LengthValue::Px(300.0);
        grid_style.height = LengthValue::Px(100.0);
        styles.insert(grid, grid_style);

        // big: spans 2 cols, 2 rows
        let mut big_style = ComputedStyle::default();
        big_style.grid_column_start = GridLineValue::Line(1);
        big_style.grid_column_end = GridLineValue::Span(2);
        big_style.grid_row_start = GridLineValue::Line(1);
        big_style.grid_row_end = GridLineValue::Span(2);
        styles.insert(big, big_style);

        // small: col 3, row 1
        let mut small_style = ComputedStyle::default();
        small_style.grid_column_start = GridLineValue::Line(3);
        small_style.grid_column_end = GridLineValue::Line(4);
        small_style.grid_row_start = GridLineValue::Line(1);
        small_style.grid_row_end = GridLineValue::Line(2);
        styles.insert(small, small_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let big_box = find_child_by_node_id(&result.root, big).expect("big found");
        let small_box = find_child_by_node_id(&result.root, small).expect("small found");

        // big 应跨两列（~200px）和两行（~100px）
        assert!(
            (big_box.width - 200.0).abs() < 1.0,
            "big should span 2 cols (~200px), got {}",
            big_box.width
        );
        assert!(
            (big_box.height - 100.0).abs() < 1.0,
            "big should span 2 rows (~100px), got {}",
            big_box.height
        );

        // small 应是一列宽一行高
        assert!(
            (small_box.width - 100.0).abs() < 1.0,
            "small should be 1 col (~100px), got {}",
            small_box.width
        );
        assert!(
            (small_box.height - 50.0).abs() < 1.0,
            "small should be 1 row (~50px), got {}",
            small_box.height
        );
    }

    /// Grid auto-placement with gap — 子元素自动放置且间距正确。
    #[test]
    fn test_grid_auto_placement_with_gap() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        let mut item_ids = Vec::new();
        for _ in 0..6 {
            let item = doc.create_element("span");
            doc.append_child(grid, item).unwrap();
            item_ids.push(item);
        }

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("100px 100px".to_string());
        grid_style.grid_template_rows = Some("50px 50px 50px".to_string());
        grid_style.gap = LengthValue::Px(10.0);
        grid_style.width = LengthValue::Px(210.0);
        grid_style.height = LengthValue::Px(400.0);
        styles.insert(grid, grid_style);

        // 不给 item 设置明确尺寸，让它们填满 grid cell
        for id in &item_ids {
            styles.insert(*id, ComputedStyle::default());
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
        let b1 = find_child_by_node_id(&result.root, item_ids[1]).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item_ids[2]).expect("item2 found");

        // 同一行两个元素之间应有 10px gap
        // b1.x 应等于 b0.x + b0.width + 10px（gap）
        assert!(
            (b1.x - b0.x - b0.width - 10.0).abs() < 1.0,
            "gap between col0 and col1 should be ~10px: b1.x({}) - b0.x({}) - b0.width({}) = {}",
            b1.x,
            b0.x,
            b0.width,
            b1.x - b0.x - b0.width
        );

        // b2 在下一行（行模板有高度 50px，所以 y 应更大）
        assert!(
            b2.y > b0.y,
            "item2 should be on the next row: b2.y({}) > b0.y({})",
            b2.y,
            b0.y
        );
    }

    /// Grid with minmax() track sizing。
    #[test]
    fn test_grid_minmax_tracks() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(grid, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(grid, item2).unwrap();

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("minmax(50px, 1fr) minmax(100px, 2fr)".to_string());
        grid_style.grid_template_rows = Some("100px".to_string());
        grid_style.width = LengthValue::Px(300.0);
        grid_style.height = LengthValue::Px(100.0);
        styles.insert(grid, grid_style);

        styles.insert(item1, ComputedStyle::default());
        styles.insert(item2, ComputedStyle::default());

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

        // 1fr : 2fr = 100px : 200px
        assert!(
            (b1.width - 100.0).abs() < 1.0,
            "minmax(50px,1fr) should be ~100px, got {}",
            b1.width
        );
        assert!(
            (b2.width - 200.0).abs() < 1.0,
            "minmax(100px,2fr) should be ~200px, got {}",
            b2.width
        );
    }

    /// Grid implicit tracks — 子元素超过显式模板行数时自动创建隐式行。
    #[test]
    fn test_grid_implicit_tracks() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        // 只定义 1 行，但放 3 个子元素 → 需要隐式行
        let mut item_ids = Vec::new();
        for _ in 0..3 {
            let item = doc.create_element("span");
            doc.append_child(grid, item).unwrap();
            item_ids.push(item);
        }

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("100px".to_string());
        grid_style.grid_template_rows = Some("50px".to_string());
        // 设置 grid-auto-rows 使隐式行有明确高度
        grid_style.grid_auto_rows = Some("40px".to_string());
        grid_style.width = LengthValue::Px(100.0);
        grid_style.height = LengthValue::Px(300.0);
        styles.insert(grid, grid_style);

        // 不给 item 设置明确尺寸，让它们填满 grid cell
        for id in &item_ids {
            styles.insert(*id, ComputedStyle::default());
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
        let b1 = find_child_by_node_id(&result.root, item_ids[1]).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item_ids[2]).expect("item2 found");

        // 三个元素应垂直排列
        assert!(b1.y > b0.y, "item1 should be below item0");
        assert!(b2.y > b1.y, "item2 should be below item1");

        // 所有元素宽度应约 100px
        assert!((b0.width - 100.0).abs() < 1.0);
        assert!((b1.width - 100.0).abs() < 1.0);
        assert!((b2.width - 100.0).abs() < 1.0);
    }

    // -- Positioned layout --

    /// 绝对定位元素在 relative 父容器内偏移。
    #[test]
    fn test_absolute_in_relative_parent() {
        let (mut doc, body) = make_doc_with_body();
        let parent = doc.create_element("div");
        doc.append_child(body, parent).unwrap();
        let abs_child = doc.create_element("span");
        doc.append_child(parent, abs_child).unwrap();

        let mut styles = HashMap::new();

        // parent: relative 定位容器
        let mut parent_style = ComputedStyle::default();
        parent_style.display = DisplayValue::Block;
        parent_style.position = PositionValue::Relative;
        parent_style.width = LengthValue::Px(400.0);
        parent_style.height = LengthValue::Px(300.0);
        styles.insert(parent, parent_style);

        // absolute child 相对于 parent 定位
        let mut abs_style = ComputedStyle::default();
        abs_style.position = PositionValue::Absolute;
        abs_style.top = LengthValue::Px(50.0);
        abs_style.left = LengthValue::Px(100.0);
        abs_style.width = LengthValue::Px(80.0);
        abs_style.height = LengthValue::Px(60.0);
        styles.insert(abs_child, abs_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let abs_box = find_child_by_node_id(&result.root, abs_child).expect("abs found");
        assert!(abs_box.is_absolute, "should be flagged absolute");
        assert!(
            (abs_box.x - 100.0).abs() < 1.0,
            "abs x should be ~100, got {}",
            abs_box.x
        );
        assert!((abs_box.y - 50.0).abs() < 1.0, "abs y should be ~50, got {}", abs_box.y);
        assert_eq!(abs_box.width, 80.0);
        assert_eq!(abs_box.height, 60.0);
    }

    /// fixed 定位元素标记为 is_fixed。
    #[test]
    fn test_fixed_position_flag() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let fixed_child = doc.create_element("span");
        doc.append_child(container, fixed_child).unwrap();

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.width = LengthValue::Px(200.0);
        container_style.height = LengthValue::Px(200.0);
        styles.insert(container, container_style);

        let mut fixed_style = ComputedStyle::default();
        fixed_style.position = PositionValue::Fixed;
        fixed_style.top = LengthValue::Px(10.0);
        fixed_style.left = LengthValue::Px(10.0);
        fixed_style.width = LengthValue::Px(50.0);
        fixed_style.height = LengthValue::Px(50.0);
        styles.insert(fixed_child, fixed_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let fixed_box = find_child_by_node_id(&result.root, fixed_child).expect("fixed found");
        assert!(fixed_box.is_fixed, "should be flagged as fixed");
        assert_eq!(fixed_box.width, 50.0);
        assert_eq!(fixed_box.height, 50.0);
    }

    /// 多个绝对定位元素在同一容器中堆叠。
    #[test]
    fn test_multiple_positioned_elements_stacking() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();

        let mut abs_ids = Vec::new();
        for _ in 0..3 {
            let el = doc.create_element("span");
            doc.append_child(container, el).unwrap();
            abs_ids.push(el);
        }

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.width = LengthValue::Px(300.0);
        container_style.height = LengthValue::Px(300.0);
        styles.insert(container, container_style);

        let offsets = [(10.0, 10.0), (50.0, 50.0), (100.0, 100.0)];
        for (i, &id) in abs_ids.iter().enumerate() {
            let mut s = ComputedStyle::default();
            s.position = PositionValue::Absolute;
            s.top = LengthValue::Px(offsets[i].0);
            s.left = LengthValue::Px(offsets[i].1);
            s.width = LengthValue::Px(60.0);
            s.height = LengthValue::Px(60.0);
            styles.insert(id, s);
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let boxes: Vec<&LayoutBox> = abs_ids
            .iter()
            .map(|id| find_child_by_node_id(&result.root, *id).expect("abs found"))
            .collect();

        // 所有都是绝对定位
        for b in &boxes {
            assert!(b.is_absolute, "all should be absolute");
        }

        // 各自的偏移正确
        assert!((boxes[0].x - 10.0).abs() < 1.0);
        assert!((boxes[0].y - 10.0).abs() < 1.0);
        assert!((boxes[1].x - 50.0).abs() < 1.0);
        assert!((boxes[1].y - 50.0).abs() < 1.0);
        assert!((boxes[2].x - 100.0).abs() < 1.0);
        assert!((boxes[2].y - 100.0).abs() < 1.0);
    }

    // -- Layout integration --

    /// 混合 block + flex 布局。
    #[test]
    fn test_mixed_block_and_flex_layout() {
        let (mut doc, body) = make_doc_with_body();
        // block header
        let header = doc.create_element("header");
        doc.append_child(body, header).unwrap();
        // flex nav
        let nav = doc.create_element("nav");
        doc.append_child(body, nav).unwrap();
        let nav_item1 = doc.create_element("span");
        doc.append_child(nav, nav_item1).unwrap();
        let nav_item2 = doc.create_element("span");
        doc.append_child(nav, nav_item2).unwrap();
        // block footer
        let footer = doc.create_element("footer");
        doc.append_child(body, footer).unwrap();

        let mut styles = HashMap::new();

        let mut header_style = ComputedStyle::default();
        header_style.display = DisplayValue::Block;
        header_style.width = LengthValue::Px(800.0);
        header_style.height = LengthValue::Px(60.0);
        styles.insert(header, header_style);

        let mut nav_style = ComputedStyle::default();
        nav_style.display = DisplayValue::Flex;
        nav_style.width = LengthValue::Px(800.0);
        nav_style.height = LengthValue::Px(40.0);
        styles.insert(nav, nav_style);

        for id in [nav_item1, nav_item2] {
            let mut s = ComputedStyle::default();
            s.width = LengthValue::Px(100.0);
            s.height = LengthValue::Px(30.0);
            styles.insert(id, s);
        }

        let mut footer_style = ComputedStyle::default();
        footer_style.display = DisplayValue::Block;
        footer_style.width = LengthValue::Px(800.0);
        footer_style.height = LengthValue::Px(40.0);
        styles.insert(footer, footer_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let header_box = find_child_by_node_id(&result.root, header).expect("header found");
        let nav_box = find_child_by_node_id(&result.root, nav).expect("nav found");
        let footer_box = find_child_by_node_id(&result.root, footer).expect("footer found");

        // 垂直堆叠顺序：header → nav → footer
        assert!(
            nav_box.y >= header_box.y + header_box.height,
            "nav should be below header"
        );
        assert!(footer_box.y >= nav_box.y + nav_box.height, "footer should be below nav");

        // flex 子元素水平排列
        let ni1 = find_child_by_node_id(&result.root, nav_item1).expect("ni1 found");
        let ni2 = find_child_by_node_id(&result.root, nav_item2).expect("ni2 found");
        assert!(ni2.x > ni1.x, "nav items should be horizontal");
    }

    /// 嵌套 flex 容器（外层 column，内层 row）。
    #[test]
    fn test_nested_flex_containers() {
        let (mut doc, body) = make_doc_with_body();
        let outer = doc.create_element("div");
        doc.append_child(body, outer).unwrap();
        let inner = doc.create_element("div");
        doc.append_child(outer, inner).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(inner, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(inner, item2).unwrap();
        let bottom = doc.create_element("span");
        doc.append_child(outer, bottom).unwrap();

        let mut styles = HashMap::new();

        let mut outer_style = ComputedStyle::default();
        outer_style.display = DisplayValue::Flex;
        outer_style.flex_direction = FlexDirectionValue::Column;
        outer_style.width = LengthValue::Px(300.0);
        outer_style.height = LengthValue::Px(200.0);
        styles.insert(outer, outer_style);

        let mut inner_style = ComputedStyle::default();
        inner_style.display = DisplayValue::Flex;
        inner_style.flex_direction = FlexDirectionValue::Row;
        inner_style.width = LengthValue::Px(300.0);
        inner_style.height = LengthValue::Px(100.0);
        styles.insert(inner, inner_style);

        for id in [item1, item2] {
            let mut s = ComputedStyle::default();
            s.width = LengthValue::Px(100.0);
            s.height = LengthValue::Px(50.0);
            styles.insert(id, s);
        }

        let mut bottom_style = ComputedStyle::default();
        bottom_style.width = LengthValue::Px(200.0);
        bottom_style.height = LengthValue::Px(40.0);
        styles.insert(bottom, bottom_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let inner_box = find_child_by_node_id(&result.root, inner).expect("inner found");
        let bottom_box = find_child_by_node_id(&result.root, bottom).expect("bottom found");
        let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

        // inner 和 bottom 垂直排列（外层 column）
        assert!(bottom_box.y > inner_box.y, "bottom should be below inner flex row");

        // item1 和 item2 水平排列（内层 row）
        assert!(b2.x > b1.x, "inner items should be horizontal");
    }

    /// border 和 padding 对最终内容区域大小的影响。
    #[test]
    fn test_border_and_padding_effect_on_content_size() {
        let (mut doc, body) = make_doc_with_body();
        let div = doc.create_element("div");
        doc.append_child(body, div).unwrap();

        let mut styles = HashMap::new();
        let mut div_style = ComputedStyle::default();
        div_style.display = DisplayValue::Block;
        div_style.width = LengthValue::Px(200.0);
        div_style.height = LengthValue::Px(100.0);
        div_style.border_top_width = LengthValue::Px(5.0);
        div_style.border_bottom_width = LengthValue::Px(5.0);
        div_style.border_left_width = LengthValue::Px(10.0);
        div_style.border_right_width = LengthValue::Px(10.0);
        div_style.padding_top = LengthValue::Px(8.0);
        div_style.padding_bottom = LengthValue::Px(8.0);
        div_style.padding_left = LengthValue::Px(12.0);
        div_style.padding_right = LengthValue::Px(12.0);
        styles.insert(div, div_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let div_box = find_child_by_node_id(&result.root, div).expect("div found");

        // 总宽度 = width + border_left + border_right + padding_left + padding_right (content-box)
        let expected_total_w = 200.0 + 10.0 + 10.0 + 12.0 + 12.0;
        assert!(
            (div_box.width - expected_total_w).abs() < 1.0,
            "total width should be ~{}, got {}",
            expected_total_w,
            div_box.width
        );

        // 总高度 = height + border_top + border_bottom + padding_top + padding_bottom
        let expected_total_h = 100.0 + 5.0 + 5.0 + 8.0 + 8.0;
        assert!(
            (div_box.height - expected_total_h).abs() < 1.0,
            "total height should be ~{}, got {}",
            expected_total_h,
            div_box.height
        );

        // 内容区域 = width（content-box 模式）
        assert!(
            (div_box.content_width - 200.0).abs() < 1.0,
            "content_width should be ~200, got {}",
            div_box.content_width
        );
        assert!(
            (div_box.content_height - 100.0).abs() < 1.0,
            "content_height should be ~100, got {}",
            div_box.content_height
        );

        // content_x = x + border_left + padding_left
        assert!(
            (div_box.content_x - div_box.x - 10.0 - 12.0).abs() < 1.0,
            "content_x offset should be border_left + padding_left"
        );
        assert!(
            (div_box.content_y - div_box.y - 5.0 - 8.0).abs() < 1.0,
            "content_y offset should be border_top + padding_top"
        );
    }

    // ── 高优先级边缘场景测试 ──

    /// 零尺寸容器包含子元素 — 验证布局引擎对 0x0 容器不会 panic，
    /// 且子元素几何值合理（不出现 NaN 或负值）。
    #[test]
    fn test_zero_size_container_with_children() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let child1 = doc.create_element("span");
        doc.append_child(container, child1).unwrap();
        let child2 = doc.create_element("span");
        doc.append_child(container, child2).unwrap();

        let mut styles = HashMap::new();
        // 容器显式 0x0
        let mut container_style = ComputedStyle::default();
        container_style.width = LengthValue::Px(0.0);
        container_style.height = LengthValue::Px(0.0);
        styles.insert(container, container_style);

        // 子元素有明确尺寸
        styles.insert(child1, make_style_with_display(DisplayValue::Block, 100.0, 50.0));
        styles.insert(child2, make_style_with_display(DisplayValue::Block, 80.0, 40.0));

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let container_box = find_child_by_node_id(&result.root, container).expect("容器应找到");
        let child1_box = find_child_by_node_id(&result.root, child1).expect("子元素 1 应找到");
        let child2_box = find_child_by_node_id(&result.root, child2).expect("子元素 2 应找到");

        // 容器尺寸不为 NaN 或负值
        assert!(
            container_box.width.is_finite() && container_box.width >= 0.0,
            "容器宽度应为有限非负值，实际 {}",
            container_box.width
        );
        assert!(
            container_box.height.is_finite() && container_box.height >= 0.0,
            "容器高度应为有限非负值，实际 {}",
            container_box.height
        );

        // 子元素尺寸不受零尺寸容器影响，仍保持正确
        assert_eq!(child1_box.width, 100.0, "子元素 1 宽度应为 100");
        assert_eq!(child1_box.height, 50.0, "子元素 1 高度应为 50");
        assert_eq!(child2_box.width, 80.0, "子元素 2 宽度应为 80");
        assert_eq!(child2_box.height, 40.0, "子元素 2 高度应为 40");
    }

    /// 深层嵌套 flexbox（15 层）— 验证布局引擎不会栈溢出，
    /// 且最内层元素尺寸正确。
    #[test]
    fn test_deeply_nested_flexbox() {
        let (mut doc, body) = make_doc_with_body();
        let depth = 15;
        let mut ids: Vec<NodeId> = Vec::new();
        let mut parent = body;

        for i in 0..depth {
            let div = doc.create_element("div");
            doc.append_child(parent, div).unwrap();
            ids.push(div);
            parent = div;

            // 最后一级加一个叶子
            if i == depth - 1 {
                let leaf = doc.create_element("span");
                doc.append_child(div, leaf).unwrap();
                ids.push(leaf);
            }
        }

        let mut styles = HashMap::new();
        for (i, &id) in ids.iter().enumerate() {
            let mut s = ComputedStyle::default();
            if i < depth {
                // 中间层都是 flex 容器
                s.display = DisplayValue::Flex;
                s.flex_direction = FlexDirectionValue::Column;
                let size = 600.0 - (i as f64) * 35.0;
                if size > 0.0 {
                    s.width = LengthValue::Px(size);
                    s.height = LengthValue::Px(size * 0.8);
                }
            } else {
                // 叶子节点
                s.width = LengthValue::Px(50.0);
                s.height = LengthValue::Px(30.0);
            }
            styles.insert(id, s);
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 最外层容器应有正确宽度
        let outer = find_child_by_node_id(&result.root, ids[0]).expect("最外层应找到");
        assert!(
            (outer.width - 600.0).abs() < 1.0,
            "最外层宽度应约 600，实际 {}",
            outer.width
        );

        // 最内层叶子节点应有正确尺寸
        let leaf = find_child_by_node_id(&result.root, ids[depth]).expect("叶子应找到");
        assert_eq!(leaf.width, 50.0, "叶子宽度应为 50");
        assert_eq!(leaf.height, 30.0, "叶子高度应为 30");
    }

    /// 绝对定位元素同时设置 top/left/right/bottom — 验证元素尺寸正确。
    /// 当四个方向都指定时，元素尺寸由 inset 约束决定，而非 content 自动尺寸。
    #[test]
    fn test_absolute_position_all_insets() {
        let (mut doc, body) = make_doc_with_body();
        let container = doc.create_element("div");
        doc.append_child(body, container).unwrap();
        let abs_el = doc.create_element("span");
        doc.append_child(container, abs_el).unwrap();

        let mut styles = HashMap::new();

        // 定位容器：relative + 明确尺寸
        let mut container_style = ComputedStyle::default();
        container_style.position = PositionValue::Relative;
        container_style.width = LengthValue::Px(400.0);
        container_style.height = LengthValue::Px(300.0);
        styles.insert(container, container_style);

        // 绝对定位元素：四个方向全部设置
        // top=20, bottom=40 → 可用高度 = 300 - 20 - 40 = 240
        // left=30, right=50 → 可用宽度 = 400 - 30 - 50 = 320
        let mut abs_style = ComputedStyle::default();
        abs_style.position = PositionValue::Absolute;
        abs_style.top = LengthValue::Px(20.0);
        abs_style.bottom = LengthValue::Px(40.0);
        abs_style.left = LengthValue::Px(30.0);
        abs_style.right = LengthValue::Px(50.0);
        styles.insert(abs_el, abs_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let abs_box = find_child_by_node_id(&result.root, abs_el).expect("绝对元素应找到");
        assert!(abs_box.is_absolute, "应标记为绝对定位");

        // 验证位置偏移
        assert!((abs_box.x - 30.0).abs() < 1.0, "x 偏移应约 30，实际 {}", abs_box.x);
        assert!((abs_box.y - 20.0).abs() < 1.0, "y 偏移应约 20，实际 {}", abs_box.y);

        // 验证由 inset 约束推导的尺寸
        assert!(
            (abs_box.width - 320.0).abs() < 1.0,
            "宽度应约 320（400-30-50），实际 {}",
            abs_box.width
        );
        assert!(
            (abs_box.height - 240.0).abs() < 1.0,
            "高度应约 240（300-20-40），实际 {}",
            abs_box.height
        );
    }

    /// Grid 使用 repeat(auto-fill, ...) 模板 — 验证 grid template 解析不 panic，
    /// 且 auto-fill 降级为单列时子元素布局正确。
    #[test]
    fn test_grid_auto_fill_columns() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        // 放 6 个子元素
        let mut item_ids = Vec::new();
        for _ in 0..6 {
            let item = doc.create_element("span");
            doc.append_child(grid, item).unwrap();
            item_ids.push(item);
        }

        let mut styles = HashMap::new();

        // grid: 使用 repeat(auto-fill, 100px) — taffy 降级为单次展开
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("repeat(auto-fill, 100px)".to_string());
        grid_style.grid_auto_rows = Some("50px".to_string());
        grid_style.width = LengthValue::Px(600.0);
        grid_style.height = LengthValue::Px(400.0);
        styles.insert(grid, grid_style);

        for id in &item_ids {
            styles.insert(*id, ComputedStyle::default());
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 所有子元素都应有有效的布局盒
        let boxes: Vec<&LayoutBox> = item_ids
            .iter()
            .map(|id| find_child_by_node_id(&result.root, *id).expect("grid item 应找到"))
            .collect();

        // 所有元素宽度和高度应为有限非负值
        for (i, b) in boxes.iter().enumerate() {
            assert!(
                b.width.is_finite() && b.width > 0.0,
                "grid item {} 宽度应为正有限值，实际 {}",
                i,
                b.width
            );
            assert!(
                b.height.is_finite() && b.height > 0.0,
                "grid item {} 高度应为正有限值，实际 {}",
                i,
                b.height
            );
        }

        // 元素应在网格中有规律排列（x 或 y 方向分布）
        let x_vals: Vec<f32> = boxes.iter().map(|b| b.x).collect();
        let y_vals: Vec<f32> = boxes.iter().map(|b| b.y).collect();
        let has_x_spread = x_vals.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
            > x_vals.iter().cloned().fold(f32::INFINITY, f32::min);
        let has_y_spread = y_vals.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
            > y_vals.iter().cloned().fold(f32::INFINITY, f32::min);
        assert!(has_x_spread || has_y_spread, "grid 子元素应在 x 或 y 方向有不同位置");
    }

    // ── auto-fill 和 minmax() 集成测试 ──

    /// 测试 repeat(auto-fill, 100px) 在 500px 容器中创建 5 个轨道。
    ///
    /// 每个 item 宽度应约 100px（500 / 5 = 100）。
    #[test]
    fn test_grid_auto_fill_fixed_size() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        // 5 个子元素
        let mut item_ids = Vec::new();
        for _ in 0..5 {
            let item = doc.create_element("span");
            doc.append_child(grid, item).unwrap();
            item_ids.push(item);
        }

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("repeat(auto-fill, 100px)".to_string());
        grid_style.width = LengthValue::Px(500.0);
        grid_style.height = LengthValue::Px(100.0);
        styles.insert(grid, grid_style);

        for id in &item_ids {
            styles.insert(*id, ComputedStyle::default());
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        // 每个 item 宽度应约 100px（500 / 5 = 100）
        for (i, &id) in item_ids.iter().enumerate() {
            let item_box = find_child_by_node_id(&result.root, id).unwrap_or_else(|| panic!("item{} not found", i));
            assert!(
                (item_box.width - 100.0).abs() < 1.0,
                "item{} 宽度应约 100px，实际 {}",
                i,
                item_box.width
            );
        }
    }

    /// 测试 repeat(auto-fill, 100px) 在 340px 容器中带 10px gap 时创建 3 个轨道。
    ///
    /// 3 个 item + 2 个 gap = 3*100 + 2*10 = 320 <= 340，
    /// 但 4 个 item 不行：4*100 + 3*10 = 430 > 340。
    #[test]
    fn test_grid_auto_fill_with_gap() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        let mut item_ids = Vec::new();
        for _ in 0..3 {
            let item = doc.create_element("span");
            doc.append_child(grid, item).unwrap();
            item_ids.push(item);
        }

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("repeat(auto-fill, 100px)".to_string());
        grid_style.gap = LengthValue::Px(10.0);
        grid_style.width = LengthValue::Px(340.0);
        grid_style.height = LengthValue::Px(200.0);
        styles.insert(grid, grid_style);

        for id in &item_ids {
            styles.insert(*id, ComputedStyle::default());
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
        let b1 = find_child_by_node_id(&result.root, item_ids[1]).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item_ids[2]).expect("item2 found");

        // 每个 item 宽度应约 100px
        assert!(
            (b0.width - 100.0).abs() < 1.0,
            "item0 宽度应约 100px，实际 {}",
            b0.width
        );

        // item1 应在 item0 右侧，间距约 10px
        let gap = b1.x - b0.x - b0.width;
        assert!((gap - 10.0).abs() < 1.0, "gap 应约 10px，实际 {}", gap);

        // item2 也应在 item1 右侧（同一行），说明有 3 个轨道
        assert!(b2.x > b1.x, "item2 应在 item1 右侧，说明至少 3 个轨道");
    }

    /// 测试 minmax(100px, 1fr) 在 300px 容器中正确约束轨道大小。
    ///
    /// 两个轨道各 minmax(100px, 1fr)，总 300px -> 各 150px，满足 min=100 和 max=1fr。
    #[test]
    fn test_grid_minmax_basic() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();
        let item1 = doc.create_element("span");
        doc.append_child(grid, item1).unwrap();
        let item2 = doc.create_element("span");
        doc.append_child(grid, item2).unwrap();

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("minmax(100px, 1fr) minmax(100px, 1fr)".to_string());
        grid_style.width = LengthValue::Px(300.0);
        grid_style.height = LengthValue::Px(100.0);
        styles.insert(grid, grid_style);

        styles.insert(item1, ComputedStyle::default());
        styles.insert(item2, ComputedStyle::default());

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b1 = find_child_by_node_id(&result.root, item1).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item2).expect("item2 found");

        // 1fr : 1fr = 150px : 150px，都满足 min 100px
        assert!(
            (b1.width - 150.0).abs() < 1.0,
            "item1 宽度应约 150px（1fr of 300/2），实际 {}",
            b1.width
        );
        assert!(
            (b2.width - 150.0).abs() < 1.0,
            "item2 宽度应约 150px（1fr of 300/2），实际 {}",
            b2.width
        );

        // 总宽度应约 300px
        let total = b1.width + b2.width;
        assert!((total - 300.0).abs() < 1.0, "总宽度应约 300px，实际 {}", total);
    }

    /// 测试 repeat(auto-fill, minmax(100px, 1fr)) 基本支持。
    ///
    /// 在 350px 容器中，auto-fill 应创建 3 个轨道（每个 min 100px），
    /// 剩余空间按 1fr 分配。
    #[test]
    fn test_grid_auto_fill_minmax() {
        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        let mut item_ids = Vec::new();
        for _ in 0..3 {
            let item = doc.create_element("span");
            doc.append_child(grid, item).unwrap();
            item_ids.push(item);
        }

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("repeat(auto-fill, minmax(100px, 1fr))".to_string());
        grid_style.width = LengthValue::Px(350.0);
        grid_style.height = LengthValue::Px(100.0);
        styles.insert(grid, grid_style);

        for id in &item_ids {
            styles.insert(*id, ComputedStyle::default());
        }

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let b0 = find_child_by_node_id(&result.root, item_ids[0]).expect("item0 found");
        let b1 = find_child_by_node_id(&result.root, item_ids[1]).expect("item1 found");
        let b2 = find_child_by_node_id(&result.root, item_ids[2]).expect("item2 found");

        // 每个轨道至少 100px（minmax 的 min 约束）
        assert!(
            b0.width >= 99.0,
            "item0 宽度应 >= 100px（minmax min），实际 {}",
            b0.width
        );
        assert!(
            b1.width >= 99.0,
            "item1 宽度应 >= 100px（minmax min），实际 {}",
            b1.width
        );

        // 三个 item 应在同一行（水平排列）
        assert!(b1.x > b0.x, "item1 应在 item0 右侧");
        assert!(b2.x > b1.x, "item2 应在 item1 右侧");

        // 总宽度应约 350px
        let total = b0.width + b1.width + b2.width;
        assert!((total - 350.0).abs() < 2.0, "总宽度应约 350px，实际 {}", total);
    }

    /// 测试 grid-template-areas 基本 2x2 布局。
    ///
    /// 定义 2x2 区域：
    ///   "header header"
    ///   "sidebar main"
    /// 验证 header 跨两列，sidebar 和 main 各占一列。
    #[test]
    fn test_grid_template_areas_basic() {
        use zero_style_system::GridLineValue;

        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        let header_el = doc.create_element("div");
        doc.append_child(grid, header_el).unwrap();
        let sidebar_el = doc.create_element("div");
        doc.append_child(grid, sidebar_el).unwrap();
        let main_el = doc.create_element("div");
        doc.append_child(grid, main_el).unwrap();

        let mut styles = HashMap::new();

        // grid 容器：2x2 模板 + 区域定义
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("100px 100px".to_string());
        grid_style.grid_template_rows = Some("50px 50px".to_string());
        grid_style.grid_template_areas = Some("\"header header\" \"sidebar main\"".to_string());
        grid_style.width = LengthValue::Px(200.0);
        grid_style.height = LengthValue::Px(100.0);
        styles.insert(grid, grid_style);

        // header: grid-area: header（跨第一行两列）
        let mut header_style = ComputedStyle::default();
        header_style.grid_row_start = GridLineValue::Name("header".to_string());
        header_style.grid_row_end = GridLineValue::Name("header".to_string());
        header_style.grid_column_start = GridLineValue::Name("header".to_string());
        header_style.grid_column_end = GridLineValue::Name("header".to_string());
        styles.insert(header_el, header_style);

        // sidebar: grid-area: sidebar（第二行第一列）
        let mut sidebar_style = ComputedStyle::default();
        sidebar_style.grid_row_start = GridLineValue::Name("sidebar".to_string());
        sidebar_style.grid_row_end = GridLineValue::Name("sidebar".to_string());
        sidebar_style.grid_column_start = GridLineValue::Name("sidebar".to_string());
        sidebar_style.grid_column_end = GridLineValue::Name("sidebar".to_string());
        styles.insert(sidebar_el, sidebar_style);

        // main: grid-area: main（第二行第二列）
        let mut main_style = ComputedStyle::default();
        main_style.grid_row_start = GridLineValue::Name("main".to_string());
        main_style.grid_row_end = GridLineValue::Name("main".to_string());
        main_style.grid_column_start = GridLineValue::Name("main".to_string());
        main_style.grid_column_end = GridLineValue::Name("main".to_string());
        styles.insert(main_el, main_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let header_box = find_child_by_node_id(&result.root, header_el).expect("header found");
        let sidebar_box = find_child_by_node_id(&result.root, sidebar_el).expect("sidebar found");
        let main_box = find_child_by_node_id(&result.root, main_el).expect("main found");

        // header 应跨两列（约 200px），在第一行
        assert!(
            (header_box.width - 200.0).abs() < 1.0,
            "header 应跨两列（~200px），实际 {}",
            header_box.width
        );
        assert!(
            (header_box.height - 50.0).abs() < 1.0,
            "header 应高约 50px，实际 {}",
            header_box.height
        );

        // sidebar 在第二行第一列
        assert!(sidebar_box.y > header_box.y, "sidebar 应在 header 下方");
        assert!(
            (sidebar_box.width - 100.0).abs() < 1.0,
            "sidebar 应宽约 100px，实际 {}",
            sidebar_box.width
        );

        // main 在第二行第二列，在 sidebar 右侧
        assert!(
            main_box.x > sidebar_box.x,
            "main 应在 sidebar 右侧: main.x={} vs sidebar.x={}",
            main_box.x,
            sidebar_box.x
        );
        assert!(
            (main_box.width - 100.0).abs() < 1.0,
            "main 应宽约 100px，实际 {}",
            main_box.width
        );

        // sidebar 和 main 在同一行
        assert!((sidebar_box.y - main_box.y).abs() < 0.01, "sidebar 和 main 应在同一行");
    }

    /// 测试 grid-area 命名引用放置。
    ///
    /// 元素设置 grid-area: "header" 后，应被放置在 header 区域对应的单元格。
    #[test]
    fn test_grid_area_name_placement() {
        use zero_style_system::GridLineValue;

        let (mut doc, body) = make_doc_with_body();
        let grid = doc.create_element("div");
        doc.append_child(body, grid).unwrap();

        let header_el = doc.create_element("div");
        doc.append_child(grid, header_el).unwrap();
        let content_el = doc.create_element("div");
        doc.append_child(grid, content_el).unwrap();

        let mut styles = HashMap::new();

        // grid 容器
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_template_columns = Some("200px 200px".to_string());
        grid_style.grid_template_rows = Some("50px 50px".to_string());
        grid_style.grid_template_areas = Some("\"header header\" \"content content\"".to_string());
        grid_style.width = LengthValue::Px(400.0);
        grid_style.height = LengthValue::Px(100.0);
        styles.insert(grid, grid_style);

        // header: 仅设置 grid-area 为命名 "header"
        let mut header_style = ComputedStyle::default();
        header_style.grid_row_start = GridLineValue::Name("header".to_string());
        header_style.grid_row_end = GridLineValue::Name("header".to_string());
        header_style.grid_column_start = GridLineValue::Name("header".to_string());
        header_style.grid_column_end = GridLineValue::Name("header".to_string());
        styles.insert(header_el, header_style);

        // content: 命名 "content"
        let mut content_style = ComputedStyle::default();
        content_style.grid_row_start = GridLineValue::Name("content".to_string());
        content_style.grid_row_end = GridLineValue::Name("content".to_string());
        content_style.grid_column_start = GridLineValue::Name("content".to_string());
        content_style.grid_column_end = GridLineValue::Name("content".to_string());
        styles.insert(content_el, content_style);

        let engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);

        let header_box = find_child_by_node_id(&result.root, header_el).expect("header found");
        let content_box = find_child_by_node_id(&result.root, content_el).expect("content found");

        // header 应在第一行，跨两列
        assert!(
            (header_box.y).abs() < 1.0,
            "header 应从 y=0 开始，实际 y={}",
            header_box.y
        );
        assert!(
            (header_box.width - 400.0).abs() < 1.0,
            "header 应跨两列（~400px），实际 {}",
            header_box.width
        );

        // content 应在第二行，跨两列
        assert!(content_box.y > header_box.y, "content 应在 header 下方");
        assert!(
            (content_box.width - 400.0).abs() < 1.0,
            "content 应跨两列（~400px），实际 {}",
            content_box.width
        );
    }
}
