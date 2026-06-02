//! 合成层逻辑 — 决定哪些元素需要提升为独立合成层。

use std::collections::HashMap;

use zero_css_parser::values::PositionValue;
use zero_dom::NodeId;
use zero_layout_engine::LayoutBox;
use zero_style_system::ComputedStyle;
use zero_style_system::property::ZIndexValue;

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
    /// z-index 值（用于合成时的堆叠排序）。
    pub z_index: i32,
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
            z_index: 0,
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
/// - 显式设置了 `z-index`（非 auto）
///
/// 提升后的图层按 z-index 排序，保证正确的堆叠顺序。
/// 根图层始终在最底层（z-index = 0）。
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

    // 按 z-index 排序：根图层始终在最前（最后绘制），
    // 负 z-index 图层在根图层之后、正 z-index 之前
    // 排序规则：z_index 升序 → 小的先绘制 → 大的覆盖小的
    layers[1..].sort_by_key(|l| l.z_index);

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
        // 读取 z-index
        let z_index = match style.z_index {
            ZIndexValue::Integer(z) => z,
            ZIndexValue::Auto => 0,
        };

        // 判断是否需要提升为独立合成层
        let should_promote = style.opacity < 1.0
            || style.position == PositionValue::Fixed
            || box_node.is_fixed
            || style.z_index != ZIndexValue::Auto;

        if should_promote {
            let mut layer = CompositingLayer::new(*next_id);
            *next_id += 1;
            layer.boxes.push(box_node.clone());
            layer.offset_x = box_node.x;
            layer.offset_y = box_node.y;
            layer.width = box_node.width;
            layer.height = box_node.height;
            layer.z_index = z_index;
            if style.opacity < 1.0 {
                layer.opacity = style.opacity as f32;
            }
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
mod tests;
