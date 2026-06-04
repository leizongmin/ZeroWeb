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

impl LayoutResult {
    /// 生成稳定的文本快照，用于测试对比。
    ///
    /// 输出格式为每行一个节点的缩进树形结构，包含位置和尺寸信息。
    /// 坐标精度固定为 2 位小数，确保快照的稳定性。
    pub fn snapshot(&self) -> String {
        let mut buf = String::new();
        buf.push_str(&format!(
            "viewport: {:.2}x{:.2}\n",
            self.viewport_width, self.viewport_height
        ));
        self.root.snapshot_into(0, &mut buf);
        buf
    }
}

impl LayoutBox {
    /// 递归生成快照文本到 `buf`。
    fn snapshot_into(&self, depth: usize, buf: &mut String) {
        let indent = "  ".repeat(depth);
        let nid = self.node_id.map_or("-".to_string(), |id| format!("{:?}", id));
        buf.push_str(&format!(
            "{}[{}] pos=({:.2},{:.2}) size=({:.2},{:.2}) content=({:.2},{:.2} {:.2}x{:.2})",
            indent,
            nid,
            self.x,
            self.y,
            self.width,
            self.height,
            self.content_x,
            self.content_y,
            self.content_width,
            self.content_height,
        ));
        // 仅在非零值时输出 border/padding/margin
        if self.border_top > 0.0 || self.border_right > 0.0 || self.border_bottom > 0.0 || self.border_left > 0.0 {
            buf.push_str(&format!(
                " border=({:.2},{:.2},{:.2},{:.2})",
                self.border_top, self.border_right, self.border_bottom, self.border_left,
            ));
        }
        if self.padding_top > 0.0 || self.padding_right > 0.0 || self.padding_bottom > 0.0 || self.padding_left > 0.0 {
            buf.push_str(&format!(
                " padding=({:.2},{:.2},{:.2},{:.2})",
                self.padding_top, self.padding_right, self.padding_bottom, self.padding_left,
            ));
        }
        if self.margin_top > 0.0 || self.margin_right > 0.0 || self.margin_bottom > 0.0 || self.margin_left > 0.0 {
            buf.push_str(&format!(
                " margin=({:.2},{:.2},{:.2},{:.2})",
                self.margin_top, self.margin_right, self.margin_bottom, self.margin_left,
            ));
        }
        if self.is_absolute {
            buf.push_str(" abs");
        }
        if self.is_fixed {
            buf.push_str(" fixed");
        }
        if self.is_sticky {
            buf.push_str(" sticky");
        }
        if self.z_index != 0 {
            buf.push_str(&format!(" z={}", self.z_index));
        }
        buf.push('\n');
        for child in &self.children {
            child.snapshot_into(depth + 1, buf);
        }
    }

    /// 在布局树中按深度优先顺序查找第 N 个（0-indexed）节点。
    ///
    /// 返回 `(绝对 X, 绝对 Y, width, height)` 或 `None`。
    pub fn nth_box(&self, index: usize) -> Option<(f32, f32, f32, f32)> {
        let mut counter = 0usize;
        self.nth_box_inner(0.0, 0.0, index, &mut counter)
    }

    fn nth_box_inner(
        &self,
        parent_x: f32,
        parent_y: f32,
        target: usize,
        counter: &mut usize,
    ) -> Option<(f32, f32, f32, f32)> {
        let abs_x = parent_x + self.x;
        let abs_y = parent_y + self.y;
        if *counter == target {
            return Some((abs_x, abs_y, self.width, self.height));
        }
        *counter += 1;
        for child in &self.children {
            let cx = abs_x + self.content_x;
            let cy = abs_y + self.content_y;
            if let Some(result) = child.nth_box_inner(cx, cy, target, counter) {
                return Some(result);
            }
        }
        None
    }

    /// 统计布局树中的节点总数（含自身）。
    pub fn count_boxes(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_boxes()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests;
