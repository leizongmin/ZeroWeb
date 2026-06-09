//! 布局输出类型定义。
//!
//! 定义 [`LayoutBox`] 和 [`LayoutResult`] 作为布局引擎的输出格式，
//! 描述元素在页面上的几何位置和大小。

pub use zero_css_parser::values::ClearValue;
use zero_css_parser::values::FloatValue;
use zero_dom::NodeId;
use zero_style_system::WritingModeValue;

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
    /// Float 方向（None 表示非浮动元素）。
    pub float: FloatValue,
    /// Clear 方向（清除哪一侧的浮动元素）。
    pub clear: ClearValue,
    /// 溢出处理。
    pub overflow_x: OverflowClip,
    /// 溢出处理。
    pub overflow_y: OverflowClip,
    /// z-index 值（用于堆叠上下文排序）。
    /// 仅对 positioned 元素（absolute/relative/fixed/sticky）生效。
    /// 默认为 0，对应 z-index: auto。
    pub z_index: i32,
    /// 滚动容器水平滚动偏移（像素，0 表示未滚动）。
    /// 仅当 overflow_x 为 Scroll 时有意义。
    pub scroll_x: f32,
    /// 滚动容器垂直滚动偏移（像素，0 表示未滚动）。
    /// 仅当 overflow_y 为 Scroll 时有意义。
    pub scroll_y: f32,
    /// 是否为 display: flow-root 元素（建立 BFC）。
    pub is_flow_root: bool,
    /// 是否为多列容器（column-count 或 column-width 非 auto）。
    /// 多列容器建立 BFC，阻止与子元素的 margin 折叠（CSS §2）。
    pub is_multicol: bool,
    /// 是否为块级元素（用于 float/clear 后处理判断）。
    ///
    /// CSS 规范中 clear 属性仅适用于块级元素。
    /// 此标志在构建布局树时根据 computed display 值设置。
    pub is_block_level: bool,
    /// 是否为 position: relative（后处理步骤需保留 relative 偏移）。
    pub is_relative: bool,
    /// border-collapse: collapse 时各边的边框颜色覆盖（RGBA u32）。
    /// 侧边索引：0=top, 1=right, 2=bottom, 3=left。
    /// None 表示无覆盖（使用 ComputedStyle 中的颜色）。
    pub collapsed_border_color_overrides: [Option<u32>; 4],
    /// border-collapse: collapse 时各边的边框样式覆盖。
    /// 侧边索引：0=top, 1=right, 2=bottom, 3=left。
    /// 当边框冲突解决后获胜方的样式与单元格原始样式不同时设置。
    pub collapsed_border_style_overrides: [Option<zero_style_system::BorderStyleValue>; 4],
    /// 元素的 writing-mode（用于 paint 阶段旋转文字和后处理轴交换）。
    pub writing_mode: WritingModeValue,
    /// 是否为匿名文本项（flex/grid 容器中的文本节点包装）。
    ///
    /// CSS Flexbox §4 规定，flex 容器中的连续文本内容生成匿名 flex item。
    /// 此标志告诉 paint 系统 node_id 指向的是文本节点本身（而非元素节点），
    /// paint 应直接渲染该文本节点的内容，而非查找子文本节点。
    pub is_anonymous_text_item: bool,
    /// CSS `order` 属性值（默认 0）。
    ///
    /// CSS Flexbox §5.4: flex item 的视觉顺序由 order 属性决定。
    /// taffy 0.7 不支持 order，因此需要在后处理中对 flex 容器的子元素按 order 排序。
    pub css_order: i32,
    /// 多列布局视觉碎片化偏移列表。
    ///
    /// 当一个子元素高度超过列高时，它需要视觉上"跨列"显示。
    /// 每个元素包含 (column_index, y_offset_in_column, visible_height)。
    /// 第一个条目是主位置（已存储在 x/y 中），后续条目是额外的列位置。
    /// paint 系统对每个额外列位置重新绘制子元素（带裁剪）。
    pub column_span_offsets: Vec<(usize, f32, f32)>,
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

impl Default for LayoutBox {
    fn default() -> Self {
        Self {
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
            children: Vec::new(),
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            float: FloatValue::None,
            clear: ClearValue::None,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
            scroll_x: 0.0,
            scroll_y: 0.0,
            is_flow_root: false,
            is_multicol: false,
            is_block_level: false,
            is_relative: false,
            collapsed_border_color_overrides: [None; 4],
            collapsed_border_style_overrides: [const { None }; 4],
            writing_mode: WritingModeValue::HorizontalTb,
            is_anonymous_text_item: false,
            css_order: 0,
            column_span_offsets: Vec::new(),
        }
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
