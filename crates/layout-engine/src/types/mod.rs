//! 布局输出类型定义。
//!
//! 定义 [`LayoutBox`] 和 [`LayoutResult`] 作为布局引擎的输出格式，
//! 描述元素在页面上的几何位置和大小。

use std::collections::HashMap;
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
    /// 内容区域偏移（相对于自身边框盒原点的 border + padding）。
    pub content_x: f32,
    /// 内容区域偏移（相对于自身边框盒原点的 border + padding）。
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
    /// 计算样式声明的 margin-top（已解析为 px）。
    ///
    /// 与 `margin_top` 的区别：`margin_top` 来自 taffy 布局结果，在 margin 折叠后
    /// 可能被放大（例如容器与首个 float 子元素错误折叠）；`declared_margin_top`
    /// 保留计算样式原始值，用于检测并修正 taffy 把 float 当作 block 导致的
    /// 容器 margin 误折叠（CSS §8.3.1：float 的 margin 不折叠）。
    /// 非 Px 长度（Percent/Auto）或垂直书写模式下回退为 `margin_top`（不触发修正）。
    pub declared_margin_top: f32,
    /// 计算样式的 width 是否为 auto（用于 float shrink-to-fit 修正）。
    ///
    /// taffy 把 float 当作普通 block，width:auto 的 float 会被填满可用宽度，
    /// 但 CSS §10.3.5 规定浮动非替换元素 width:auto 应 shrink-to-fit 到内容。
    /// 此标记让 float 后处理识别 width:auto 的 float 并收缩到内容宽度。
    pub declared_width_auto: bool,
    /// `height:auto` 标记（R1277 ④）：是否 computed height 为 Auto。
    ///
    /// float 后处理中「非 BFC 容器内容高度收缩」须仅对 auto-height 容器生效——
    /// 显式高度（definite height）容器不应被 float 重定位后的 content_bottom 重算
    /// 收缩（CSS §10.5：显式高度的 used height 即显式值，内容溢出/不足不改变高度）。
    /// 旧实现对显式高度容器也收缩，致 floats-006 `#div1{height:200px}` 在 float
    /// 上提后 content_bottom 降到 100 而被错误塌缩到 100（R1273 实证）。此标记让
    /// 收缩守卫跳过 definite-height 容器。
    pub declared_height_auto: bool,
    /// 子布局盒。
    pub children: Vec<LayoutBox>,
    /// 是否为绝对定位。
    pub is_absolute: bool,
    /// 是否为替换元素（img/video/iframe/embed/object/svg/canvas 等有固有尺寸）。
    ///
    /// CSS §10.3.8/§10.6.6：替换元素的 auto 尺寸按固有尺寸 + 宽高比解析，**不**按
    /// §10.3.18/§10.6.4（非替换）的全-inset stretch。abspos 全-inset stretch 后处理
    /// 须据此跳过替换元素，避免把 img 固有尺寸覆写为视口 stretch（abspos-025/026）。
    pub is_replaced: bool,
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
    /// 是否创建堆叠上下文（stacking context）。
    /// CSS 2.1：positioned 元素 + z-index 为整数时创建堆叠上下文。
    /// z-index: auto 不创建堆叠上下文——其 positioned 后代参与父级堆叠上下文。
    /// 非 positioned 元素始终为 false。
    pub creates_stacking_context: bool,
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
    /// 是否为布局容器（flex/grid/table）。
    /// 布局容器建立 BFC（CSS Flexbox §3, CSS Grid §3），
    /// 其子元素由各自的布局算法定位，不走 IFC。
    pub is_layout_container: bool,
    /// R1318 §8.3.1 containment 标志：本容器的 in-flow 子中是否有**空块**（collapse-through）
    /// 被应用了正 clearance。由 `adjust_float_positions` 设置（仅空块 cleared）；
    /// `exclude_floats_from_non_bfc_auto_height` 据此跳过收缩——clearance 破坏 collapse-through，
    /// trailing 折叠链留父内（contained），容器高度已由 containment 计算确定，不应被
    /// 「float 不计高度」路径覆盖。
    pub had_clearance: bool,
    /// 是否为「孤立 table-internal 元素」（display:table-row-group/table-row/table-cell 等，
    /// 且父元素非 table/table-internal）——CSS Tables §2.4 应为其生成匿名 table 包装盒。
    /// 此标记让该元素在 establishes_bfc 中被视为匿名 table（建立 BFC，隔离 margin 折叠
    /// + 包含浮动），由 mark_anonymous_table_roots 预处理在 adjust_float_positions 之前设置。
    pub is_anon_table_root: bool,
    /// 多列容器的列间距（column-gap），由 layout 层设置，paint 层用于裁剪。
    /// 非 multicol 容器为 0.0。
    pub column_gap: f32,
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
    /// border-collapse: collapse 时标记哪些边是表格外边缘。
    /// 侧边索引：0=top, 1=right, 2=bottom, 3=left。
    /// 外边缘的边框不进行厚度减半（因为没有邻居共享），
    /// 内边缘的边框减半以避免与邻居重叠。
    pub collapsed_border_outer_edge: [bool; 4],
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
    /// 多列布局额外列片段信息。
    ///
    /// 当一个子元素高度超过列高时（column breaking），它需要视觉上
    /// "跨列"显示 — 同一个子元素在多个列中出现，每列显示其不同高度切片。
    ///
    /// 子元素的主位置（第一个片段）存储在 `x/y` 中。
    /// 此字段存储额外的片段信息，每条格式为：
    /// `(x_in_container, y_in_container, column_x, column_width, col_top, col_h)`
    ///
    /// - `x_in_container`: 片段在容器内容区域中的 x 坐标
    /// - `y_in_container`: 片段在容器内容区域中的 y 坐标
    ///   （含负偏移，使子元素内容不同垂直范围可见）
    /// - `column_x`: 该列在容器内容区域中的起始 x 坐标（用于 paint 层裁剪）
    /// - `column_width`: 列宽度（用于 paint 层水平裁剪）
    /// - `col_top`: 该片段列顶 y（片段在列内的起始 y，用于 R1039 paint 垂直裁剪）
    /// - `col_h`: 该片段视觉高度（slice 高度，用于 R1039 paint 裁到 fragment slice）
    ///
    /// paint 系统对每个额外片段重新绘制子元素，并裁剪到对应列的区域。
    pub column_span_offsets: Vec<(f32, f32, f32, f32, f32, f32)>,
    /// 行内布局结果（来自 layout engine 的 IFC 运行）。
    ///
    /// 当设置时，paint 系统直接复用这些结果渲染文字，不再重新运行 IFC。
    /// 这消除了 paint IFC 字体度量不一致的系统性问题。
    pub inline_layout: Option<Vec<InlineLayoutLine>>,
    /// 存储行内布局结果时的容器宽度。
    ///
    /// paint 使用前会验证当前 content_width 是否匹配此值。
    /// 若不匹配（如 table/multicol 后处理改变了宽度），
    /// paint 回退到重新运行 IFC。
    pub inline_layout_width: f32,
    /// 文本节点的 font_size 映射（来自 layout engine 的 IFC 运行）。
    ///
    /// paint 系统在运行空 styles IFC 后，使用这些正确的 font_size 值
    /// 计算基线偏移，避免 16px 默认值导致的字形定位错误。
    pub text_node_font_sizes: HashMap<NodeId, f32>,
    /// 文本节点是否使用 Ahem 字体的映射（来自 layout engine 的 IFC 运行）。
    ///
    /// paint 系统在运行空 styles IFC 时无法检测 Ahem 字体（无 style 信息），
    /// 使用此映射正确设置 is_ahem 标志，使字符宽度计算使用 1.0×font_size
    /// 而非默认的 0.55×font_size。
    pub text_node_is_ahem: HashMap<NodeId, bool>,
    /// 文本节点的 letter-spacing 映射（来自 layout engine 的 IFC 运行）。
    ///
    /// paint 系统在运行空 styles IFC 时无法获取 letter-spacing（无 style 信息），
    /// 使用此映射正确设置字符间间距。
    pub text_node_letter_spacing: HashMap<NodeId, f32>,
    /// 文本节点的 line-height 映射（来自 layout engine 的 IFC 运行）。
    ///
    /// paint 系统在运行空 styles IFC 时无法获取 line-height（无 style 信息），
    /// 回退为 font_size * 1.2 近似值。对于使用自定义 line-height（如 line-height: 2）
    /// 的元素，近似值会导致行盒高度与 layout IFC 不一致。
    /// 使用此映射确保 paint IFC 的行盒高度与 layout IFC 一致。
    pub text_node_line_heights: HashMap<NodeId, f32>,
    /// 文本节点的 text-transform 映射（来自 layout engine 的 IFC 运行）。
    ///
    /// **R1012 Phase A IFC 统一首切**：text-transform 须在行断前应用，使 layout
    /// 用转换后文本宽度行断。但 paint Path B 重跑 IFC 时 styles 为空，无法读取
    /// 父元素的 text-transform → 行断用原文 → 与 layout 不一致。此映射（key = 文本
    /// 节点 NodeId）由 `store_font_sizes_from_ifc` 在 layout 期填充，paint Path B
    /// 据此构造 `text_transform_overrides`（re-key 到父元素），让 `collect_inline_items`
    /// 在空 styles 下也能应用 transform。空 map（默认）= None = 原文，零回归。
    pub text_node_text_transform: HashMap<NodeId, zero_style_system::TextTransformValue>,
    /// 内联元素的 (font_size, line_height) 映射（来自 layout engine 的 IFC 运行）。
    ///
    /// 与 text_node_font_sizes/line_heights 不同，此映射以元素自身的 NodeId 为键，
    /// 供 paint IFC 在处理内联元素（非文本节点）时使用。
    /// 内联元素在 paint IFC 中无法获取自己的样式（styles 为空），
    /// 导致 font_size 和 line_height 回退到默认值。
    /// 此映射确保 paint IFC 的内联元素使用正确的字体度量和行高。
    pub inline_element_metrics: HashMap<NodeId, (f32, f32)>,
    /// 内联元素的 (margin_left, margin_right) 映射（来自 layout engine 的 IFC 运行）。
    ///
    /// paint IFC 传入空的 styles HashMap，无法获取 inline 元素的水平 margin，
    /// 导致所有 margin 回退为 0。此映射以元素自身的 NodeId 为键，
    /// 供 paint IFC 在处理内联元素时使用正确的 margin 值。
    /// margin 不影响行断（仅影响水平偏移），因此传递到 paint IFC 是安全的。
    pub inline_element_margins: HashMap<NodeId, (f32, f32)>,
    /// 从 taffy 布局缓存中提取的 first_baseline（y 分量）。
    ///
    /// 仅对 flex/inline-flex/grid/inline-grid 容器有值。
    /// 用于 `adjust_inline_block_positions` 中计算 inline-flex/inline-grid
    /// 容器在父 IFC 中的基线位置，替代 font_size 近似。
    pub taffy_baseline: Option<f32>,
    /// R109 §9.2.1.1 匿名块盒的片段文本节点覆盖。
    ///
    /// 当此 LayoutBox 是 inline 元素被 block 子元素拆分后的一个匿名块盒时，
    /// 设为其片段包含的 DOM 子节点（文本 + inline 元素）。paint/layout IFC
    /// 据此只收集该片段的 inline 内容（IFC.fragment_node_ids），而非 inline
    /// 元素的全部子节点。`None` = 正常盒（非匿名块片段）。
    pub fragment_node_ids: Option<Vec<NodeId>>,
    /// R109 §9.2.1.1：此 inline 盒被 in-flow block-level 子元素拆分（生产端接线标记）。
    ///
    /// 当为 true 时，此盒自身的 paint IFC 应跳过——其直接文本已由其匿名块片段
    /// 子盒（带 fragment_node_ids）渲染。仅 env `R109_WIRE=1` 时由 build_subtree
    /// 标记的 inline 父盒为 true。
    pub is_r109_split: bool,
    /// R109 §9.2.1.1：此匿名块片段是其 split inline 片段序列的**首** Inline 片段。
    /// fragment border 边选择：首片段开放右分裂边（shrink 步骤置 border_right=0）。
    pub r109_first_fragment: bool,
    /// R109 §9.2.1.1：此匿名块片段是其 split inline 片段序列的**末** Inline 片段。
    /// fragment border 边选择：末片段开放左分裂边（shrink 步骤置 border_left=0）。
    pub r109_last_fragment: bool,
    /// 表格列背景绘制信息（CSS Tables §17.5.3 列背景）。
    ///
    /// `<col>`/`<colgroup>` 元素不生成常规流盒，其 `background-color` 须由表格
    /// 绘制算法特殊处理：在单元格背景**之下**、按列跨满表格高度绘制。
    /// 每项 = `(node_id, x_offset, width)`：x_offset/width 相对表格 content box，
    /// node_id 指向 `<col>` 或 `<colgroup>` 元素（painter 据此取 background-color）。
    /// 仅含 background-color 非透明且宽度 > 0 的列元素；顺序为 colgroup 在前、
    /// col 在后（colgroup 背景在下层）。由 `collect_table_col_backgrounds` 在
    /// position_cells 后填充（彼时 col_widths + col→column 映射已知）。
    pub table_col_backgrounds: Vec<(NodeId, f32, f32)>,
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
            declared_margin_top: 0.0,
            declared_width_auto: false,
            declared_height_auto: false,
            children: Vec::new(),
            is_absolute: false,
            is_replaced: false,
            is_fixed: false,
            is_sticky: false,
            float: FloatValue::None,
            clear: ClearValue::None,
            overflow_x: OverflowClip::Visible,
            overflow_y: OverflowClip::Visible,
            z_index: 0,
            creates_stacking_context: false,
            scroll_x: 0.0,
            scroll_y: 0.0,
            is_flow_root: false,
            is_multicol: false,
            is_layout_container: false,
            had_clearance: false,
            is_anon_table_root: false,
            column_gap: 0.0,
            is_block_level: false,
            is_relative: false,
            collapsed_border_color_overrides: [None; 4],
            collapsed_border_style_overrides: [const { None }; 4],
            collapsed_border_outer_edge: [false; 4],
            writing_mode: WritingModeValue::HorizontalTb,
            is_anonymous_text_item: false,
            css_order: 0,
            column_span_offsets: Vec::new(),
            inline_layout: None,
            inline_layout_width: 0.0,
            text_node_font_sizes: HashMap::new(),
            text_node_is_ahem: HashMap::new(),
            text_node_letter_spacing: HashMap::new(),
            text_node_line_heights: HashMap::new(),
            text_node_text_transform: HashMap::new(),
            inline_element_metrics: HashMap::new(),
            inline_element_margins: HashMap::new(),
            taffy_baseline: None,
            fragment_node_ids: None,
            is_r109_split: false,
            r109_first_fragment: false,
            r109_last_fragment: false,
            table_col_backgrounds: Vec::new(),
        }
    }
}

/// 行内布局行盒 — paint 系统复用的 IFC 结果。
///
/// 存储在 `LayoutBox.inline_layout` 中，避免 paint 系统重新运行 IFC。
/// 仅包含 paint 系统渲染文字所需的最小数据。
#[derive(Debug, Clone)]
pub struct InlineLayoutLine {
    /// 行盒 y 坐标（相对于容器内容区域）。
    pub y: f32,
    /// 行盒高度。
    pub height: f32,
    /// 行盒中的片段列表。
    pub fragments: Vec<InlineLayoutFragment>,
    /// 行盒基线相对行顶的 y（= ascent，CSS §10.8.1）。R816 linebox 度量统一 Phase 1：
    /// 由 compute_final 从 IFC LineBox 存储供 paint 复用。Phase 1 仅存储，paint 尚未读取。
    pub baseline_y: f32,
    /// 行盒 ascent（baseline 到行顶，含 half-leading 上半）。Phase 1 存储。
    pub ascent: f32,
    /// 行盒 descent（baseline 到行底，含 half-leading 下半）。Phase 1 存储。
    pub descent: f32,
}

/// 行内布局片段 — paint 系统使用的文本定位信息。
#[derive(Debug, Clone)]
pub struct InlineLayoutFragment {
    /// 片段 x 坐标（在行盒内）。
    pub x: f32,
    /// 片段 y 坐标（在行盒内，经过 vertical-align 后）。
    /// 对于 baseline 对齐：frag.y = baseline_y - height。
    /// 基线位置 = frag.y + height（不是 frag.y + font_size）。
    pub y: f32,
    /// 片段宽度。
    pub width: f32,
    /// 片段高度（line-height 盒高度，用于计算基线位置）。
    pub height: f32,
    /// 字体大小（用于字形渲染大小）。
    pub font_size: f32,
    /// 是否使用 Ahem 字体（影响字形宽度计算）。
    pub is_ahem: bool,
    /// R817 linebox 度量统一 Phase 2：片段**实际**字体是否 Ahem（来自 IFC run.is_ahem_font，
    /// 区别于容器级 `is_ahem`）。用于 paint is_ahem glyph 基线定位——仅对真正 Ahem 方块字形
    /// （ascent=font_size）应用 `baseline_y - font_size` 公式；容器为 Ahem 但片段实为其它字体
    /// （如 font-051 的 serif span）时为 false，保留旧 v_offset 行为避免回归。
    pub is_ahem_font: bool,
    /// 文本内容。
    pub text: String,
    /// 对应 DOM 节点 ID（用于去重）。
    pub node_id: Option<NodeId>,
    /// 片段基线相对行顶的 y（baseline 对齐时 = line.baseline_y + vertical_align_offset）。
    /// R816 Phase 1 存储，paint 尚未读取（行为不变）。
    pub baseline_y: f32,
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
