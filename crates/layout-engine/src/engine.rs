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
    pub fn compute(
        &self,
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> LayoutResult {
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

        let is_absolute = computed.is_some_and(|s| {
            matches!(s.position, PositionValue::Absolute)
        });
        let is_fixed = computed.is_some_and(|s| {
            matches!(s.position, PositionValue::Fixed)
        });
        let overflow_x = computed.map_or(OverflowClip::Visible, |s| {
            convert_overflow_to_clip(&s.overflow_x)
        });
        let overflow_y = computed.map_or(OverflowClip::Visible, |s| {
            convert_overflow_to_clip(&s.overflow_y)
        });

        // 计算内容区域
        let content_x = layout.location.x + layout.border.left + layout.padding.left;
        let content_y = layout.location.y + layout.border.top + layout.padding.top;
        let content_width = (layout.size.width
            - layout.border.left
            - layout.border.right
            - layout.padding.left
            - layout.padding.right)
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
mod tests {
    use super::*;
    use zero_css_parser::values::{
        AlignmentValue, DisplayValue, FlexDirectionValue, FlexWrapValue, LengthValue,
        OverflowValue, PositionValue,
    };
    use zero_dom::Document;

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
            box2.y, box1.y, box1.height
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
            box2.x, box1.x
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
}
