//! 合成层逻辑 — 决定哪些元素需要提升为独立合成层。

use std::collections::HashMap;

use zero_css_parser::values::PositionValue;
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_style_system::ComputedStyle;

/// 合成层 — 可以独立渲染和合成的图层。
#[derive(Debug, Clone)]
pub struct CompositingLayer {
    /// 图层 ID。
    pub id: usize,
    /// 图层内的布局盒。
    pub boxes: Vec<LayoutBox>,
    /// 图层偏移（相对于父图层）。
    pub offset_x: f32,
    /// 图层偏移（相对于父图层）。
    pub offset_y: f32,
    /// 图层尺寸。
    pub width: f32,
    /// 图层尺寸。
    pub height: f32,
    /// 透明度。
    pub opacity: f32,
    /// 是否为根图层。
    pub is_root: bool,
}

impl CompositingLayer {
    /// 创建新的合成层。
    pub fn new(id: usize) -> Self {
        Self {
            id,
            boxes: Vec::new(),
            offset_x: 0.0,
            offset_y: 0.0,
            width: 0.0,
            height: 0.0,
            opacity: 1.0,
            is_root: false,
        }
    }

    /// 计算图层包围盒。
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for b in &self.boxes {
            min_x = min_x.min(b.x);
            min_y = min_y.min(b.y);
            max_x = max_x.max(b.x + b.width);
            max_y = max_y.max(b.y + b.height);
        }

        (min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

/// 合成层提升策略 — 决定哪些元素需要提升为独立合成层。
///
/// 当前提升条件：
/// - `opacity < 1.0`
/// - `position: fixed`
///
/// 其他元素留在根图层中。
pub fn promote_compositing_layers(
    layout: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Vec<CompositingLayer> {
    let mut layers = Vec::new();
    let mut layer_id = 0;

    // 根图层 — 包含所有未被提升的元素
    let mut root_layer = CompositingLayer::new(layer_id);
    root_layer.is_root = true;
    layer_id += 1;

    // 递归遍历布局树
    collect_layers(layout, styles, &mut layers, &mut root_layer, &mut layer_id);

    // 将根图层加入列表
    layers.insert(0, root_layer);

    layers
}

/// 递归遍历布局树，将元素分配到合成层。
fn collect_layers(
    box_node: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    layers: &mut Vec<CompositingLayer>,
    root_layer: &mut CompositingLayer,
    next_id: &mut usize,
) {
    let mut promoted = false;

    if let Some(node_id) = box_node.node_id
        && let Some(style) = styles.get(&node_id)
    {
        // 提升条件 1: opacity < 1.0
        if style.opacity < 1.0 {
            let mut layer = CompositingLayer::new(*next_id);
            *next_id += 1;
            layer.boxes.push(box_node.clone());
            layer.opacity = style.opacity as f32;
            layer.offset_x = box_node.x;
            layer.offset_y = box_node.y;
            layer.width = box_node.width;
            layer.height = box_node.height;
            layers.push(layer);
            promoted = true;
        }
        // 提升条件 2: position: fixed
        else if style.position == PositionValue::Fixed || box_node.is_fixed {
            let mut layer = CompositingLayer::new(*next_id);
            *next_id += 1;
            layer.boxes.push(box_node.clone());
            layer.offset_x = box_node.x;
            layer.offset_y = box_node.y;
            layer.width = box_node.width;
            layer.height = box_node.height;
            layers.push(layer);
            promoted = true;
        }
    }

    // 未被提升的元素加入根图层
    if !promoted {
        root_layer.boxes.push(box_node.clone());

        // 更新根图层尺寸
        let right = box_node.x + box_node.width;
        let bottom = box_node.y + box_node.height;
        if right > root_layer.width {
            root_layer.width = right;
        }
        if bottom > root_layer.height {
            root_layer.height = bottom;
        }
    }

    // 递归处理子节点
    for child in &box_node.children {
        collect_layers(child, styles, layers, root_layer, next_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::values::PositionValue;
    use zero_layout_engine::types::OverflowClip;
    use zero_style_system::ComputedStyle;

    /// 辅助函数：创建简单 LayoutBox。
    fn make_box(node_id: Option<NodeId>, x: f32, y: f32, w: f32, h: f32, is_fixed: bool) -> LayoutBox {
        LayoutBox {
            node_id,
            x,
            y,
            width: w,
            height: h,
            content_x: 0.0,
            content_y: 0.0,
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
            is_fixed,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        }
    }

    /// 测试只有根元素时只有一个根图层。
    #[test]
    fn test_compositing_layer_root_only() {
        let layout = make_box(None, 0.0, 0.0, 800.0, 600.0, false);
        let styles = HashMap::new();
        let layers = promote_compositing_layers(&layout, &styles);

        assert_eq!(layers.len(), 1);
        assert!(layers[0].is_root);
    }

    /// 测试 opacity < 1.0 的元素被提升为独立图层。
    #[test]
    fn test_compositing_layer_opacity_promotion() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");

        let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
        let root_box = LayoutBox {
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
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.opacity = 0.5;
        styles.insert(elem, style);

        let layers = promote_compositing_layers(&root_box, &styles);
        // 应该有根图层 + 1 个提升图层
        assert_eq!(layers.len(), 2);
        assert!(layers[0].is_root);
        assert!(!layers[1].is_root);
        assert!((layers[1].opacity - 0.5).abs() < 0.001);
    }

    /// 测试 position: fixed 的元素被提升为独立图层。
    #[test]
    fn test_compositing_layer_fixed_position_promotion() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");

        let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
        let root_box = LayoutBox {
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
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Fixed;
        styles.insert(elem, style);

        let layers = promote_compositing_layers(&root_box, &styles);
        assert_eq!(layers.len(), 2);
        assert!(layers[0].is_root);
        assert!(!layers[1].is_root);
    }

    /// 测试普通元素不会被提升。
    #[test]
    fn test_compositing_layer_no_promotion_normal() {
        let mut doc = zero_dom::Document::new();
        let elem = doc.create_element("div");

        let child_box = make_box(Some(elem), 0.0, 0.0, 100.0, 50.0, false);
        let root_box = LayoutBox {
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
            children: vec![child_box],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();
        styles.insert(elem, ComputedStyle::default());

        let layers = promote_compositing_layers(&root_box, &styles);
        // 只有根图层
        assert_eq!(layers.len(), 1);
        assert!(layers[0].is_root);
    }

    /// 测试多个元素同时被提升。
    #[test]
    fn test_compositing_layer_multiple_promotions() {
        let mut doc = zero_dom::Document::new();
        let elem1 = doc.create_element("div");
        let elem2 = doc.create_element("div");
        let elem3 = doc.create_element("div");

        let child1 = make_box(Some(elem1), 0.0, 0.0, 100.0, 50.0, false);
        let child2 = make_box(Some(elem2), 0.0, 50.0, 100.0, 50.0, false);
        let child3 = make_box(Some(elem3), 0.0, 100.0, 100.0, 50.0, false);
        let root_box = LayoutBox {
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
            children: vec![child1, child2, child3],
            is_absolute: false,
            is_fixed: false,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
        };

        let mut styles = HashMap::new();

        // elem1: opacity = 0.5（被提升）
        let mut style1 = ComputedStyle::default();
        style1.opacity = 0.5;
        styles.insert(elem1, style1);

        // elem2: position: fixed（被提升）
        let mut style2 = ComputedStyle::default();
        style2.position = PositionValue::Fixed;
        styles.insert(elem2, style2);

        // elem3: 普通（不提升）
        styles.insert(elem3, ComputedStyle::default());

        let layers = promote_compositing_layers(&root_box, &styles);
        // 根图层 + 2 个提升图层
        assert_eq!(layers.len(), 3);
        assert!(layers[0].is_root);
    }

    /// 测试 CompositingLayer 的 bounding_box 方法。
    #[test]
    fn test_compositing_layer_bounding_box() {
        let mut layer = CompositingLayer::new(0);
        layer.boxes.push(make_box(None, 10.0, 20.0, 100.0, 50.0, false));
        layer.boxes.push(make_box(None, 50.0, 30.0, 80.0, 60.0, false));

        let (x, y, w, h) = layer.bounding_box();
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert_eq!(w, 120.0); // max right (130) - min left (10)
        assert_eq!(h, 70.0);  // max bottom (90) - min top (20)
    }
}
