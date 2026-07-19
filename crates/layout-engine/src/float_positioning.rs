//! 浮动定位与收缩后处理。
//!
//! 从 `engine.rs` 抽出（R342，2000 行规则 + Phase A Phase 5 准备）。
//! 包含：float 元素重新定位（CSS 2.1 §9.5）、clear clearance、BFC 浮动排斥、
//! 垂直书写模式块收缩、inline-block shrink-to-fit、匿名 table 根标记。

use std::collections::HashMap;
use zero_css_parser::values::{ColorValue, DisplayValue, FlexDirectionValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_style_system::{ComputedStyle, WritingModeValue};

use crate::types::LayoutBox;

/// 调整 float 元素的位置，并处理 clear 属性。
///
/// taffy 将 float 元素当作普通 block 处理（按正常流排列）。
/// 此后处理步骤将 float 元素重新定位到容器的左侧或右侧，
/// 支持水平并排放置、clear 属性（含 float 元素自身的 clear）。
///
/// ## 实现要点
///
/// - 同侧 float 水平并排放置（CSS 2.1 §9.5.1），空间不足时换行
/// - float 元素的 clear 属性正确生效
/// - 非 float 块元素的 clear 使用 max(normal_flow_Y, float_bottom)
/// - 非 float、非 clear 元素的 Y 偏移扣除 float 元素占据的垂直空间
pub(crate) fn adjust_float_positions(box_node: &mut LayoutBox) {
    let content_abs_y = box_node.y + box_node.content_y;
    adjust_float_positions_with_context(box_node, content_abs_y, 0.0, 0.0, &[]);
}

/// 浮动几何元组：(dir, x, y, width, height+margin_bottom, margin_right)。
/// 坐标相对「持有该浮动的容器的 border-box 原点」。供 BFC 排斥 + 嵌套透传共用。
type FloatGeom = (FloatValue, f32, f32, f32, f32, f32);

/// R1623：BFC 被 float 排斥收缩 width 后，同步收缩 content_width（= width - frame）。
/// 否则内层 adjust_float_positions 递归用旧（大）content_width 做 container_width，
/// 致 BFC 内 float 不按收缩后宽换行/堆叠（floats-bfc-003 inner floats 溢出 BFC）。
fn shrink_bfc_content_width(child: &mut LayoutBox) {
    let frame = child.border_left + child.border_right + child.padding_left + child.padding_right;
    child.content_width = (child.width - frame).max(0.0);
}

/// 纯文本 float shrink-to-fit（CSS §10.3.5）预补 pass。
///
/// `adjust_float_positions` 的收缩分支仅处理含 block 级 / replaced 子元素的 float
/// （`content_child_widths`）；**纯文本 float**（无 block/replaced 子，仅直接文本）保持
/// taffy 全宽——旧注释「shrink-to-fit 需 IFC 测量，留后续」。本 pass 用
/// `text_content_max_width` 测量纯文本 float 的 max-content 宽度并收缩（仅当窄于 taffy
/// 宽度），修 `font-size: 0` float 未收缩（应 0 宽，font-size-zero-3）+ 短文本 float 撑满
/// 全宽。须在 `adjust_float_positions` **之前**运行，使后续 float 定位 / 排斥用收缩后宽度。
pub(crate) fn shrink_pure_text_floats(
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    // 先递归子树（深度优先），使嵌套容器内的 float 也被处理。
    for child in &mut box_node.children {
        shrink_pure_text_floats(child, doc, styles);
    }
    // 仅处理 width:auto 的 float（非替换）。非 auto 宽度 / 非 float / replaced 由别处处理。
    if !box_node.declared_width_auto || matches!(box_node.float, FloatValue::None) || box_node.is_replaced {
        return;
    }
    // 仅水平书写模式：text_content_max_width 度量水平文本宽，垂直模式的 float 其 inline 轴
    // 为垂直（block-size），物理 width 语义不同，由 shrink_vertical_blocks_to_content 独立处理。
    // 不 gate 会致 hyphens-vertical-* 垂直 float 误收缩（±0.04pp 噪声 flip）。
    if !matches!(box_node.writing_mode, WritingModeValue::HorizontalTb) {
        return;
    }
    // 已有 block 级 / replaced 子元素的 float 由 adjust_float_positions 收缩分支处理，
    // 此处不重复（避免双重 shrink / 与 block 子宽度冲突）。
    let has_block_or_replaced = box_node
        .children
        .iter()
        .any(|c| !c.is_absolute && !c.is_fixed && (c.is_block_level || c.is_replaced));
    if has_block_or_replaced {
        return;
    }
    // 纯文本 float：用 text_content_max_width 测 max-content（font-size:0 → 0，无文本 → 0）。
    let Some(dom_id) = box_node.node_id else {
        return;
    };
    let text_max_w = crate::intrinsic_sizing::text_content_max_width(dom_id, doc, styles);
    let shrink_border_box =
        text_max_w + box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    // 仅当内容确实更窄时才收缩（对内容更宽或显式宽度为 no-op）。
    if shrink_border_box < box_node.width {
        box_node.width = shrink_border_box;
        box_node.content_width = text_max_w;
    }
}

/// 垂直书写模式下 width:auto 块级元素收缩到内容（CSS §10.3.3 + CSS Writing Modes §7.1）。
///
/// 规范：垂直书写模式（vertical-rl/lr）中，块级元素的 block-size 为物理 width。
/// block-size:auto 时应基于内容收缩（同水平模式下的 height:auto），而非填满包含块。
///
/// 当前架构的轴交换以**父元素**书写模式为键（converter/tree.rs 与 engine.rs
/// extract_layout）：仅当父元素为垂直模式时才交换子元素几何，使 taffy 以水平模型布局。
/// 但元素**自身**书写模式决定其 width 是 block-size 还是 inline-size。当元素自身为垂直
/// 模式而其父元素为水平模式时（典型场景：body 内一个 `writing-mode: vertical-rl` 的
/// div），轴交换不触发，taffy 把 width:auto 当作行内填充（填满容器宽度），违反规范。
///
/// 此后处理在 float 定位之后遍历布局树，对这类块按内容块轴跨度（最右侧流内子元素
/// margin-box 右缘）收缩 width，并尊重 min-width/max-width。
///
/// **自限性**：仅当内容右缘窄于当前 width 时才收缩，对子元素已正确铺满到内容右缘的
/// 垂直块为 no-op，从而对正确布局的用例零回归。
pub(crate) fn shrink_vertical_blocks_to_content(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    parent_writing_mode: &WritingModeValue,
) {
    let own_vertical = matches!(
        box_node.writing_mode,
        WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
    );
    let parent_horizontal = matches!(parent_writing_mode, WritingModeValue::HorizontalTb);

    if own_vertical && parent_horizontal && box_node.is_block_level && !box_node.is_absolute && !box_node.is_fixed {
        let width_auto = box_node
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(|s| matches!(s.width, LengthValue::Auto));
        if width_auto {
            // 内容块轴跨度 = 最右侧流内子元素 margin-box 右缘（相对父 border-box）。
            let content_extent = box_node
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed)
                .map(|c| c.x + c.width + c.margin_right)
                .fold(0.0f32, f32::max);
            // 解析 min-width/max-width（style 解析阶段 em/rem 等已解析为 Px）。
            let (min_w, max_w) = box_node
                .node_id
                .and_then(|id| styles.get(&id))
                .map(|s| {
                    let lo = match &s.min_width {
                        LengthValue::Px(v) => *v as f32,
                        _ => 0.0,
                    };
                    let hi = match &s.max_width {
                        LengthValue::Px(v) => *v as f32,
                        _ => f32::MAX,
                    };
                    (lo, hi)
                })
                .unwrap_or((0.0, f32::MAX));
            let new_width = content_extent.max(min_w).min(max_w).min(box_node.width);
            if new_width + 0.5 < box_node.width {
                let frame =
                    box_node.border_left + box_node.border_right + box_node.padding_left + box_node.padding_right;
                box_node.width = new_width;
                box_node.content_width = (new_width - frame).max(0.0);
            }
        }
    }

    let pw = box_node.writing_mode.clone();
    for child in &mut box_node.children {
        shrink_vertical_blocks_to_content(child, styles, &pw);
    }
}

/// 后处理：`width:auto` 的 inline-block 收缩到内容宽度（CSS §10.3.9 shrink-to-fit）。
///
/// taffy 0.7 把 width:auto 的 inline-block 拉伸到可用宽度（如同 block），违反
/// inline-block 应 shrink-to-fit 到 max-content 的规范。此处读取流内 block 级子元素
/// 已布局的宽度（margin-box），取最大值作为内容宽度，仅在内容确实更窄时收缩
/// （内容更宽或显式宽度时为 no-op）。与 R129 float-shrink / R138 table-shrink 同谱系，
/// 但作用对象是 inline-block——其子元素已是正确尺寸（如显式 width 的 block），
/// 故仅收缩盒尺寸本身，不重排子元素。
pub(crate) fn shrink_inline_blocks_to_content(
    box_node: &mut LayoutBox,
    doc: &zero_dom::Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    let own_horizontal = matches!(box_node.writing_mode, WritingModeValue::HorizontalTb);
    if own_horizontal && !box_node.is_absolute && !box_node.is_fixed {
        // R372：除 inline-block 外，**带非默认 background 的 inline 元素**（如 morning.work
        // `.item-tag` 徽章 span：display:inline + background-color + padding）也应 shrink-to-fit。
        // ZeroWeb 把 inline 映射为 Block 拉到满宽（满宽色条），此处按 intrinsic 内容宽收缩
        // （仅 width 维度；元素仍是 block 堆叠——完整 inline-box 模型属 Phase A 多会话）。
        // 仅对有 background 的 inline 触发，避免影响纯文本 inline span（其文本经 IFC 收集，
        // 无盒装饰，收缩无意义且可能干扰）。
        let is_shrinkable = box_node.node_id.is_some_and(|id| {
            styles.get(&id).is_some_and(|s| match s.display {
                // R783：inline-flex / inline-grid 同 inline-block 一样是 inline-level 容器，
                // width:auto 应 shrink-to-fit（CSS §10.3.10/§10.3.11），但 taffy 0.7 把它们
                // 当 block 拉伸到可用宽（满宽）。此前仅 inline-block 收缩，inline-flex/inline-grid
                // 漏处理→aspect-ratio-intrinsic-size-001/003/006/008 等内联弹性盒被拉到 784px。
                // 多 item flex-row 的 main-axis 求和语义此处用 max 近似（单 item 等价；多 item
                // 罕见且满宽→max 仍优于 784px 拉伸）。
                DisplayValue::InlineBlock | DisplayValue::InlineFlex | DisplayValue::InlineGrid => true,
                DisplayValue::Inline => {
                    // R372：带非默认 background 的 inline shrink-to-fit。R1480（R109 增量）：
                    // 带 border 的 inline（如 WPT border-width-applies-to-008：display:inline +
                    // border-width:90px）亦应 shrink——否则 inline→taffy::Block 拉满宽，border
                    // 画在满宽 box（应 content-width = 内容 + 左右 border）。
                    let has_bg = s.background_color != ColorValue::Transparent;
                    let has_border = matches!(&s.border_top_width, LengthValue::Px(v) if *v > 0.0)
                        || matches!(&s.border_bottom_width, LengthValue::Px(v) if *v > 0.0)
                        || matches!(&s.border_left_width, LengthValue::Px(v) if *v > 0.0)
                        || matches!(&s.border_right_width, LengthValue::Px(v) if *v > 0.0);
                    has_bg || has_border
                }
                _ => false,
            })
        });
        let width_auto = box_node
            .node_id
            .is_some_and(|id| styles.get(&id).is_some_and(|s| matches!(s.width, LengthValue::Auto)));
        if is_shrinkable && width_auto {
            // 内容最大宽度（max-content）。R1017：InlineFlex/InlineGrid 当 box_content_max_width
            // 测得 0（aspect-ratio 空 item 等 box_content 无法度量）时，fallback 到专用 flex_intrinsic
            //（含 aspect-ratio transferred + container-cross 推导）；否则保留 box_content_max_width
            //（覆盖 gap/abspos/文本等有 content 案，避免回归）。
            let intrinsic_border_box = crate::intrinsic_sizing::box_content_max_width(box_node, doc, styles);
            let intrinsic_border_box = if intrinsic_border_box > 0.5 {
                intrinsic_border_box
            } else {
                let flex_intrinsic = box_node.node_id.and_then(|id| styles.get(&id)).and_then(|s| {
                    if !matches!(s.display, DisplayValue::InlineFlex | DisplayValue::InlineGrid) {
                        return None;
                    }
                    if matches!(s.display, DisplayValue::InlineGrid) {
                        crate::intrinsic_sizing::grid_intrinsic_width(box_node, doc, styles)
                    } else if matches!(
                        s.flex_direction,
                        FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
                    ) {
                        crate::intrinsic_sizing::flex_column_intrinsic_width(box_node, doc, styles)
                    } else {
                        crate::intrinsic_sizing::flex_row_intrinsic_width(box_node, doc, styles)
                    }
                });
                flex_intrinsic.unwrap_or(0.0)
            };
            let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
            let content_max_w = (intrinsic_border_box - frame).max(0.0);
            if content_max_w > 0.0 {
                let shrink_border_box = content_max_w + frame;
                if shrink_border_box + 0.5 < box_node.width {
                    box_node.width = shrink_border_box;
                    box_node.content_width = content_max_w;
                }
            }
        }
    }

    for child in &mut box_node.children {
        shrink_inline_blocks_to_content(child, doc, styles);
    }
}

/// 标记孤立 table-internal 元素（CSS Tables §2.4）为匿名 table 根。
///
/// 当 `display:table-row-group/table-row/table-cell/...` 出现在非 table 上下文中
///（父元素非 table/table-internal）时，CSS 规范应为其生成匿名 table 包装盒。
/// 此预遍历近似该行为：把这类孤立元素的 `is_anon_table_root` 置真，使其在
/// `establishes_bfc` 中被视为匿名 table（建立 BFC，隔离 margin 折叠 + 包含浮动）。
/// 在 `adjust_float_positions` 之前运行，确保 float exclusion 识别这些容器。
pub(crate) fn mark_anonymous_table_roots(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    in_table_context: bool,
) {
    let display = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .map(|s| s.display.clone());
    let is_table = matches!(display, Some(DisplayValue::Table | DisplayValue::InlineTable));
    let is_table_internal = matches!(
        display,
        Some(
            DisplayValue::TableRowGroup
                | DisplayValue::TableHeaderGroup
                | DisplayValue::TableFooterGroup
                | DisplayValue::TableRow
                | DisplayValue::TableCell
                | DisplayValue::TableCaption
        )
    );

    if is_table_internal && !in_table_context {
        box_node.is_anon_table_root = true;
        // 孤立 table-internal 充当匿名 table（块级），使其被 adjust_float_positions
        // 的 clear / BFC float-exclusion 逻辑（要求 is_block_level）正确处理。
        box_node.is_block_level = true;
    }

    let child_context = in_table_context || is_table || is_table_internal;
    for child in &mut box_node.children {
        mark_anonymous_table_roots(child, styles, child_context);
    }
}

/// R1392：递归收集非 BFC 后代中的浮动元素外底边（content-relative 到外层容器）。
///
/// CSS §9.5：clear 须清除同 BFC 上下文内的所有浮动。ZW 的 `active_left/right_float_bottom`
/// 仅收集**直接** float 子，嵌套在非 BFC 后代中的 float 对后续 clear 兄弟不可见 →
/// clear 失效（adjoining-float-before-clearance：float 嵌在 wrapper 内，clear:left 看不到它）。
/// BFC 后代建立独立浮动上下文（其内 float 不外溢），递归在 BFC 边界停止。
///
/// `child_border_y` = child 的 border-box 顶，相对外层 border-box（累加祖先 y）。
/// `outer_content_y_offset` = 外层 border→content 偏移，用于换算 content-relative。
fn nested_float_bottoms(child: &LayoutBox, child_border_y: f32, outer_content_y_offset: f32) -> (f32, f32) {
    use zero_css_parser::values::FloatValue;
    let mut left = 0.0f32;
    let mut right = 0.0f32;
    for c in &child.children {
        if c.is_absolute || c.is_fixed {
            continue;
        }
        // c.y 相对 child 的 border-box 原点；累加到外层 border-relative。
        let c_border_y = child_border_y + c.y;
        if !matches!(c.float, FloatValue::None) {
            let bottom = c_border_y - outer_content_y_offset + c.height + c.margin_bottom;
            match c.float {
                FloatValue::Left => left = left.max(bottom),
                FloatValue::Right => right = right.max(bottom),
                _ => {}
            }
        } else if !crate::margin_collapse::establishes_bfc(c) {
            // 非 BFC 后代：其内浮动与外层同 BFC，继续递归收集。
            let (l, r) = nested_float_bottoms(c, c_border_y, outer_content_y_offset);
            if l > left {
                left = l;
            }
            if r > right {
                right = r;
            }
        }
        // BFC 后代：独立浮动上下文，停止递归。
    }
    (left, right)
}

pub(crate) fn adjust_float_positions_with_context(
    box_node: &mut LayoutBox,
    box_content_abs_y: f32,
    inherited_left_bottom_abs: f32,
    inherited_right_bottom_abs: f32,
    // R1618/R1619 Slice 2：祖先 BFC 上下文内的浮动几何（已转换到本容器 border-box 帧）。
    // 非 BFC 子递归时透传，使嵌套 BFC（如 overflow:hidden 在 margin-div 内）能避开
    // 祖先 float（CSS §9.5：BFC border-box 不重叠同 BFC 上下文内任意 float）。
    // env ZW_NESTED_BFC_FLOAT_AVOID=0 关闭（kill-switch，default-on）。
    inherited_floats: &[FloatGeom],
) {
    use zero_css_parser::values::ClearValue;
    use zero_css_parser::values::FloatValue;

    // R1277 ②：inline 级子（is_block_level=false，如 `<span>` display:inline）不推进
    // flow_bottom——CSS §9.5.1 仅 block-level 前置内容约束 float 的 outer top 边。
    // 旧实现把 inline→taffy::Block 映射的 span 当 block 累入 flow_bottom，致后续 float
    // 被推到 inline 内容之下（floats-006 float 落 rel_y=100 而非 0）。须与 ④（显式高度
    // 守卫）协调上 default：② 单独 net -3 floats-clear（R1272），②+④ 同码 A/B 全 10 dir
    // NET 0 + floats-006 11.54→4.79（R1277）。env `ZW_FLOAT_LIFT_INLINE=0` 可关闭
    //（kill-switch，default-on；与 ④ 绑定，单独关闭会重现 -3）。
    let lift_inline = std::env::var("ZW_FLOAT_LIFT_INLINE").as_deref() != Ok("0");

    // 容器的内容区域宽度
    let container_width = box_node.content_width;

    // CSS Flexbox §4 / Grid §4 / Tables §2.4：flex/grid/table 容器的流内子元素
    //（即布局项）其 `float` 与 `clear` 不产生浮动或清除效果——`float` 计算为 `none`。
    // taffy 内部已据此布局，但 ZeroWeb 的浮动后处理（本函数）按 `child.float` 重新
    // 定位，会把带 `float:right` 的 flex item 误推到容器右缘。此处对布局容器父级的
    // 直接子元素将 `float` 归零，使后处理（含 paint 的 float 排斥/绘制）一致忽略它。
    if box_node.is_layout_container {
        for child in &mut box_node.children {
            child.float = FloatValue::None;
        }
    }

    // taffy 子元素的 Y 坐标是相对于父元素的 border-box 原点，
    // 而 flow_bottom / line_y 等追踪变量是相对于 content area 原点。
    // 当容器有 border-top 或 padding-top 时，需要加上偏移量。
    let content_y_offset = box_node.border_top + box_node.padding_top;
    let inherited_left_bottom = (inherited_left_bottom_abs - box_content_abs_y).max(0.0);
    let inherited_right_bottom = (inherited_right_bottom_abs - box_content_abs_y).max(0.0);

    // CSS §8.3.1 修正：float 的 margin 不与父容器折叠。但 taffy 把 float 当作普通
    // block 排列，当容器的首个流内子元素是 float 且容器无 border-top/padding-top
    //（margin 可与子元素折叠）时，容器的 margin-top 会被折叠到该 float 的 margin
    //（取 max），使容器（及其全部内容）整体偏低。此处把多折叠的量从容器 y 中扣除
    // 并恢复 margin_top。
    //
    // 精确门控（四重条件，确保只修真正的 float-margin 折叠，排除 taffy 把 float 当
    // block 引起的其它膨胀）：
    //   1. 容器无 border-top/padding-top（margin 可与首个子元素折叠）
    //   2. 容器布局 margin_top > 声明值（发生了膨胀）
    //   3. 容器 margin_top == 首个 float 子元素 margin_top（容器 mt 被折叠到该 float mt）
    //   4. 该 float 子元素的 margin_top == 其声明值（float 的 mt 自身未被 taffy 膨胀）
    if content_y_offset == 0.0 && box_node.margin_top > box_node.declared_margin_top + 0.01 {
        if let Some(fc) = box_node
            .children
            .iter()
            .find(|c| !c.is_absolute && !c.is_fixed)
            .filter(|c| !matches!(c.float, FloatValue::None))
        {
            let container_absorbed_float_mt = (box_node.margin_top - fc.margin_top).abs() < 0.01;
            let float_mt_is_clean = (fc.margin_top - fc.declared_margin_top).abs() < 0.01;
            if container_absorbed_float_mt && float_mt_is_clean {
                let over_collapse = box_node.margin_top - box_node.declared_margin_top;
                box_node.y -= over_collapse;
                box_node.margin_top = box_node.declared_margin_top;
            }
        }
    }

    // 第一阶段：重新定位 float 元素，记录每个 float 在 taffy 布局中占据的垂直空间
    //
    // CSS 2.1 §9.5.1 float 定位规则：
    // 1. Float 必须尽可能高（"as high as possible"）
    // 2. Float 的 outer top 不得高于前面元素生成的块的 outer top
    // 3. Float 不参与 margin 折叠
    //
    // 关键：min_line_y 必须基于正常流内容的实际位置（考虑 margin 折叠），
    // 而不是 taffy 的 Y（taffy 将 float 当作 block，包含了前元素的 margin 折叠）。
    let mut line_y = 0.0f32;
    let mut line_max_height = 0.0f32;
    let mut left_used_width = 0.0f32;
    let mut right_used_width = 0.0f32;
    let mut left_float_bottom = inherited_left_bottom;
    let mut right_float_bottom = inherited_right_bottom;

    // 正常流的垂直位置追踪（用于计算 float 的最小 Y）
    // 关键：flow_bottom 必须独立于 taffy 的 Y 来计算。
    // taffy 将 float 当作 block 排列，导致后续正常流元素的 Y 偏移过大。
    // 我们通过累加正常流元素的高度 + margin 折叠来独立追踪 flow_bottom。
    let mut flow_bottom = 0.0f32; // 上一个正常流元素的 border-bottom（相对于容器 content area）
    let mut last_flow_mb = 0.0f32; // 上一个正常流元素的 margin-bottom
    // 第一个流内子元素的 margin-top 会与无 border-top/padding-top 的父容器折叠
    //（CSS §8.3.1），此时子元素位于容器 content 原点，其 margin-top 不应计入 flow_bottom，
    // 否则后续 float 的 min_float_y 会把该 margin-top 双重计入，使 float 偏低。
    let mut first_in_flow = true;

    // 记录每个 float 子元素在 taffy 布局中的 Y 和高度，用于后续偏移修正
    let mut float_taffy_y: Vec<(usize, f32, f32)> = Vec::new(); // (index, taffy_y, outer_height)

    for (idx, child) in box_node.children.iter_mut().enumerate() {
        // 跳过绝对定位和 fixed 元素
        if child.is_absolute || child.is_fixed {
            continue;
        }

        // 正常流元素：独立更新 flow_bottom
        // 不使用 taffy 的 Y（包含 float 垂直空间），而是自行累加
        if matches!(child.float, FloatValue::None) && !child.is_absolute && !child.is_fixed {
            // 第一个流内子元素的 margin-top 与无 border-top/padding-top 的父容器折叠：
            // margin-top 上浮到父容器外，子元素位于 content 原点，不计入 flow_bottom。
            let parent_collapses_first = first_in_flow && content_y_offset == 0.0;
            first_in_flow = false;
            // R1277 ②：inline 级子（is_block_level=false）不推进 flow_bottom——
            // CSS §9.5.1 仅 block-level 前置内容约束 float 的 outer top。
            if !lift_inline || child.is_block_level {
                // 独立计算正常流位置：使用 margin 折叠
                if crate::margin_collapse::is_empty_block(child) {
                    let collapsed_self_margin =
                        crate::margin_collapse::collapse_two_margins(child.margin_top, child.margin_bottom);
                    last_flow_mb = crate::margin_collapse::collapse_two_margins(last_flow_mb, collapsed_self_margin);
                } else {
                    let collapsed_margin = if parent_collapses_first {
                        0.0
                    } else {
                        crate::margin_collapse::collapse_two_margins(last_flow_mb, child.margin_top)
                    };
                    let child_y = flow_bottom + collapsed_margin;
                    let child_border_bottom = child_y + child.height;
                    flow_bottom = child_border_bottom;
                    last_flow_mb = child.margin_bottom;
                }
            }
            // 处理非 float 元素的 clear 属性（延迟到第二阶段）
            if matches!(child.float, FloatValue::None) {
                continue;
            }
        }

        // 记录 float 元素的 taffy Y 位置和高度
        let child_outer_height = child.margin_top + child.height + child.margin_bottom;
        float_taffy_y.push((idx, child.y, child_outer_height));

        // CSS §10.3.5：width:auto 的浮动非替换元素应 shrink-to-fit 到内容宽度。
        // taffy 把 float 当作普通 block（填满可用宽度），此处对 width:auto 且有
        // 块级子元素的 float 收缩到子元素最大 border-box 宽度（仅当窄于当前宽度）。
        // 纯文本内容（无块级子元素）的 float 保持 taffy 宽度——其 shrink-to-fit 需
        // IFC 测量，留作后续。仅当内容确实更窄时才收缩，对内容更宽或显式宽度的 float 为 no-op。
        if child.declared_width_auto {
            // 内容宽度候选：块级子元素取最大 border-box 宽度，**inline-level replaced
            // 子元素**（img/video 等原子 inline 盒，已有确定 used 宽度）也纳入——R180 教训
            //「content 宽须 inline 级求和 + block 级取最大」。旧实现仅 `is_block_level` 致
            // float div 仅含 `<img>`（inline-level）时 content_child_widths 为空→跳过收缩→
            // float 撑满全宽，img 无法覆盖 div 背景（max-width-110，red 68400px 外露）。
            // replaced 的 used width 已解析（含 max-width 等约束），无需 IFC 测量。
            // 纯文本 float（无 block 级、无 replaced 子元素）仍保持 taffy 宽度（需 IFC，留后续）。
            let content_child_widths: Vec<f32> = child
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed && (c.is_block_level || c.is_replaced))
                .map(|c| c.width)
                .collect();
            let content_max_w = content_child_widths.iter().copied().fold(0.0f32, f32::max);
            // 有块级或 replaced 子元素时收缩到内容宽度（content_max_w + padding + border）。
            // **content_max_w 可能为 0**（如 visibility:collapse 的 flex item 主尺寸归零，
            // 或空内容块）——旧条件 `content_max_w > 0.0` 在此跳过收缩致 float 撑满全宽
            //（flexbox-collapsed-item-horiz-001 根因，R300）。改为「有内容子元素即收缩」：
            // 空内容 float 收缩到 padding+border（最小盒），仍比全宽更接近 shrink-to-fit 语义。
            // 纯文本 float（无 block 级/replaced 子元素）保持 taffy 宽度——其 shrink-to-fit
            // 需 IFC 测量，留后续。
            if !content_child_widths.is_empty() {
                let shrink_border_box =
                    content_max_w + child.padding_left + child.padding_right + child.border_left + child.border_right;
                if shrink_border_box < child.width {
                    child.width = shrink_border_box;
                    child.content_width = content_max_w;
                }
            }
        }

        // 计算浮动元素的总占用尺寸（含 margin）
        let child_outer_width = child.margin_left + child.width + child.margin_right;

        // CSS 2.1 §9.5.1 float 定位约束：
        // 1. Float 的 outer top 不得高于前面正常流元素生成的块盒的 outer top
        // 2. Float 不参与 margin 折叠
        //
        // CSS 2.1 §9.5.1：float 的 margin box 顶边按正常流规则确定。
        // 这里的 `line_y` / `child.y` 都追踪的是 border-box 顶边，因此用于
        // 约束 float 最小垂直位置时，不能再次把当前元素的 margin-top 加进去，
        // 否则后续 `child.y = line_y + margin_top` 会双计 margin-top。
        let min_float_y = flow_bottom + last_flow_mb;
        if min_float_y > line_y {
            line_y = min_float_y;
            left_used_width = 0.0;
            right_used_width = 0.0;
            line_max_height = 0.0;
        }

        // 处理 float 元素自身的 clear 属性
        match child.clear {
            ClearValue::Left => {
                if left_float_bottom > line_y {
                    line_y = left_float_bottom;
                    left_used_width = 0.0;
                    right_used_width = 0.0;
                    line_max_height = 0.0;
                }
            }
            ClearValue::Right => {
                if right_float_bottom > line_y {
                    line_y = right_float_bottom;
                    left_used_width = 0.0;
                    right_used_width = 0.0;
                    line_max_height = 0.0;
                }
            }
            ClearValue::Both => {
                let clear_y = left_float_bottom.max(right_float_bottom);
                if clear_y > line_y {
                    line_y = clear_y;
                    left_used_width = 0.0;
                    right_used_width = 0.0;
                    line_max_height = 0.0;
                }
            }
            ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd => {}
        }

        // 检查当前行是否有足够空间放置此浮动元素
        let available_width = container_width - left_used_width - right_used_width;
        if child_outer_width > available_width && line_max_height > 0.0 {
            line_y += line_max_height;
            left_used_width = 0.0;
            right_used_width = 0.0;
            line_max_height = 0.0;
        }

        match child.float {
            FloatValue::Left => {
                child.x = left_used_width + child.margin_left;
                child.y = content_y_offset + line_y + child.margin_top;

                left_used_width += child_outer_width;
                let new_bottom = line_y + child_outer_height;
                left_float_bottom = left_float_bottom.max(new_bottom);
            }
            FloatValue::Right => {
                right_used_width += child_outer_width;
                child.x = container_width - right_used_width + child.margin_left;
                child.y = content_y_offset + line_y + child.margin_top;

                let new_bottom = line_y + child_outer_height;
                right_float_bottom = right_float_bottom.max(new_bottom);
            }
            FloatValue::InlineStart | FloatValue::InlineEnd | FloatValue::None => {}
        }

        // CSS 2.1 §9.5.1：零高度浮动元素（margin-box 高度为 0）不推进 line_y。
        // 一个没有内容、没有 border、没有 padding 的空浮动元素不应占据垂直空间，
        // 后续浮动元素应从相同的 line_y 开始。
        if child_outer_height > 0.0 {
            line_max_height = line_max_height.max(child_outer_height);
        }
    }

    // 第二阶段：修正非 float 子元素的 Y 位置 + BFC 浮动排斥
    // CSS 规范中 float 元素脱离正常流，不应占据垂直空间。
    // taffy 将 float 当作正常 block 排列，导致后续非 float 元素的 Y 偏移过大。
    // 策略：维护一个 float_y_offset（累积 float 在 taffy 中占据的垂直空间），
    // 对每个非 float 子元素从 Y 中扣除 offset；clear 元素消耗 offset。
    //
    // BFC 浮动排斥（CSS 2.1 §9.5）：建立 BFC 的块级元素不得与浮动元素重叠。
    // 当一个非 float 块级元素建立 BFC 且与浮动元素垂直重叠时，
    // 需将其水平位置偏移到浮动元素旁边。
    let has_active_float_context =
        !float_taffy_y.is_empty() || inherited_left_bottom > 0.0 || inherited_right_bottom > 0.0;
    // R1393：adjoining-float 吸收是否在本容器触发（触发后须跑 containment 收缩容器高度，
    // 否则被吸收的 margin 仍留在容器高度里显红）。函数级——在 if has_active_float_context
    // 块内设置，containment gate（块外）读取。
    let mut ran_adjoining_clearance = false;
    // R1393：adjoining-float——容器无直接 float 子，但非 BFC 后代内嵌套 float + 有 clear 子时，
    // 须走主 clearance 路径（else 分支 R1389 按「无 float context」处理，clear 看不到嵌套
    // 浮动）。窄 gate：仅同时有 clear 子 + 嵌套浮动才扩 has_active_float_context，避免影响
    // 普通容器。env `ZW_ADJOINING_FLOAT_CLEARANCE=0` 关闭（kill-switch，default-on）。
    let has_active_float_context = has_active_float_context
        || (std::env::var("ZW_ADJOINING_FLOAT_CLEARANCE").as_deref() != Ok("0")
            && box_node.children.iter().any(|c| {
                c.is_block_level
                    && !c.is_absolute
                    && !c.is_fixed
                    && !matches!(
                        c.clear,
                        ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
                    )
            })
            && box_node.children.iter().any(|c| {
                !c.is_absolute
                    && !c.is_fixed
                    && matches!(c.float, FloatValue::None)
                    && !crate::margin_collapse::establishes_bfc(c)
                    && {
                        let (nl, nr) = nested_float_bottoms(c, 0.0, 0.0);
                        nl > 0.0 || nr > 0.0
                    }
            }));
    let mut child_float_contexts: Vec<(f32, f32)> =
        vec![(inherited_left_bottom, inherited_right_bottom); box_node.children.len()];

    // R1316/R1317：以下流追踪变量在子循环（has_active_float_context 块）内维护，
    // 并被后面的容器高度 containment 计算（§8.3.1）读取，故声明在函数级。
    let mut flow_bottom = 0.0f32; // 上一个非 float 流内元素的 border-bottom（content-relative）
    let mut last_flow_mb = 0.0f32; // 上一个非 float 流内元素的 margin-bottom（折叠链）
    // R1316：一旦本容器内任一 in-flow 子被应用了正 clearance，taffy 的布局
    // 即与正确流位置发散（taffy 不建模 clearance）。此后所有非 clear 块级
    // 兄弟须以 flow_bottom 为权威基准重定位，而非沿用 taffy 的（错误）位置。
    let mut had_clearance = false;
    // R1318 §8.3.1 containment：是否有**空块**（collapse-through）被应用了正 clearance。
    // containment 公式（flow_bottom + chain − consumed mt）仅对 empty cleared block
    // 验证正确（margin-collapse-clear-012/013、margin-collapse-033/034/035 谱系）；
    // 非空 cleared（如 replaced element）的 containment 几何不同，本轮不处理（避回归）。
    let mut had_empty_clearance = false;
    // R1317 §8.3.1 containment：被 clearance「消耗」的 cleared 元素 margin-top
    // 累积（每个正 clearance 子贡献其 margin_top）。计算 contained parent height
    // 时须从 trailing 折叠链中扣除，避免 clearance-absorbed margin 双计。
    let mut clearance_consumed_mt = 0.0f32;

    // 收集本容器自身的 float 子元素几何（border-box 帧），用于 BFC 排斥 + 嵌套透传。
    // 注意：c.y 已含 margin_top（Phase 1 定位），故 float_h 只需 height + margin_bottom。
    let float_geometries: Vec<FloatGeom> = box_node
        .children
        .iter()
        .filter(|c| !matches!(c.float, FloatValue::None))
        .map(|c| {
            (
                c.float.clone(),
                c.x,
                c.y,
                c.width,
                c.height + c.margin_bottom,
                c.margin_right,
            )
        })
        .collect();
    // R1619 Slice 2：合并祖先 float（已在本容器 border-box 帧）与自身 float，供 BFC 排斥段
    //（块内）与递归透传（块外）使用——使嵌套在非 BFC 后代内的 BFC 能避开祖先 float。
    // env ZW_NESTED_BFC_FLOAT_AVOID=0 关闭（kill-switch，default-on）。
    let nested_avoid_on = std::env::var("ZW_NESTED_BFC_FLOAT_AVOID").as_deref() != Ok("0");
    let all_floats: Vec<FloatGeom> = if nested_avoid_on && !inherited_floats.is_empty() {
        inherited_floats
            .iter()
            .chain(float_geometries.iter())
            .cloned()
            .collect()
    } else {
        float_geometries.clone()
    };

    if has_active_float_context {
        let mut float_y_offset = 0.0f32;
        // 追踪正常流内容的位置，用于 clearance 假设位置计算
        let mut active_left_float_bottom = inherited_left_bottom;
        let mut active_right_float_bottom = inherited_right_bottom;
        // R1393：追踪来自「非 BFC 后代嵌套浮动」的底边（区别于直接 float 子），用于
        // adjoining-float 吸收判定——clear 清除的是嵌套浮动时，其 margin 经 wrapper 与该
        // 浮动 adjoining（§8.3.1+§9.5.2），须吸收而非正常折叠。
        let mut nested_left_bottom: f32 = 0.0;
        let mut nested_right_bottom: f32 = 0.0;

        // float_geometries / all_floats 已在 if 块外（函数作用域）定义，使 BFC 排斥段
        //（块内）与递归透传（块外）都能访问。

        // R1369：预计算每个子是否有「后续 in-flow block 同胞」（definite-width BFC 推到 float
        // 下方时，若有后续 block 同胞会留空隙/错位 → 仅无后续同胞时才安全推下）。
        let n_children = box_node.children.len();
        let has_following_block_sibling: Vec<bool> = (0..n_children)
            .map(|i| {
                box_node.children[i + 1..]
                    .iter()
                    .any(|s| s.is_block_level && !s.is_absolute && !s.is_fixed && matches!(s.float, FloatValue::None))
            })
            .collect();

        for (idx, child) in box_node.children.iter_mut().enumerate() {
            child_float_contexts[idx] = (active_left_float_bottom, active_right_float_bottom);
            if child.is_absolute || child.is_fixed {
                continue;
            }

            if !matches!(child.float, FloatValue::None) {
                // float 元素：将其 taffy 高度加入 offset
                // CSS 2.1：零高度浮动元素不占据垂直空间

                // CSS 2.1 §9.5.1：float 元素不应高于正常流内容的位置。
                // Phase 1 定位 float 时不知道 normal flow 的位置，
                // 这里修正：将 float 的 Y 推到至少与当前流位置齐平。
                // 注意：flow_bottom 是 content-relative，child.y 是 border-relative
                let child_content_y = child.y - content_y_offset;
                if child_content_y < flow_bottom {
                    let shift = flow_bottom - child_content_y;
                    child.y = content_y_offset + flow_bottom;
                    // 仅更新此 float 所在侧的 float_bottom 追踪
                    match child.float {
                        FloatValue::Left => active_left_float_bottom += shift,
                        FloatValue::Right => active_right_float_bottom += shift,
                        _ => {}
                    }
                }

                let float_total_height = child.margin_top + child.height + child.margin_bottom;
                float_y_offset += float_total_height;
                let child_bottom = child.y - content_y_offset + child.height + child.margin_bottom;
                match child.float {
                    FloatValue::Left => active_left_float_bottom = active_left_float_bottom.max(child_bottom),
                    FloatValue::Right => active_right_float_bottom = active_right_float_bottom.max(child_bottom),
                    _ => {}
                }
                continue;
            }

            // 保存 taffy 的原始 Y（调整前）
            let original_taffy_y = child.y;

            // CSS 规范：clear 属性仅适用于块级元素（CSS 2.1 §13.5）
            if !child.is_block_level {
                // 非块级元素（如 inline）：扣除 float offset，不处理 clear
                if float_y_offset > 0.0 {
                    child.y -= float_y_offset;
                }
                // R1277 ②：inline 级子不推进 flow_bottom（CSS §9.5.1 仅 block-level
                // 前置内容约束 float 顶边）。否则 lifted float 后的 inline 内容会把
                // 后续 float 再度推下。
                if !lift_inline {
                    flow_bottom = flow_bottom.max(child.y - content_y_offset + child.height);
                }
                continue;
            }

            // CSS 2.1 §9.5.2 Clearance 计算
            // 假设位置（hypothetical position）：无 clear 时元素应在的位置，
            // 基于正常流的 flow_bottom + margin 折叠计算。
            //
            // CSS 2.1 §9.5.2 clearance 算法：
            // 1. 计算 hypothetical position（假设 clear:none，含 margin 折叠）
            // 2. 计算 clearance = max(0, clear_bottom - hypothetical_position)
            // 3. 如果 clearance > 0：元素推到 clear_bottom
            // 4. 如果 clearance == 0：clear 仍阻止 margin 折叠
            //    元素放在 flow_bottom + margin_top（不折叠）
            // R1316：本子是否被应用了正 clearance（破坏 collapse-through）。
            let mut clearance_applied = false;
            match child.clear {
                ClearValue::Left | ClearValue::Right | ClearValue::Both => {
                    let clear_bottom = match child.clear {
                        ClearValue::Left => active_left_float_bottom,
                        ClearValue::Right => active_right_float_bottom,
                        _ => active_left_float_bottom.max(active_right_float_bottom),
                    };
                    // 假设位置：基于正常流的 flow_bottom + margin 折叠
                    // CSS 2.1 §9.5.2：「as if 'clear' were 'none'」
                    let collapsed_margin = crate::margin_collapse::collapse_two_margins(last_flow_mb, child.margin_top);
                    let hypothetical_y = flow_bottom + collapsed_margin;

                    if clear_bottom > hypothetical_y {
                        // 正 clearance：margin 不折叠
                        // CSS 2.1 §9.5.2 C1/C2 双路径算法：
                        // C1（含 margin 折叠）：clearance = clear_bottom - hypothetical_y
                        // C2（不含 margin 折叠）：clearance = clear_bottom - (flow_bottom + child.margin_top)
                        // 最终位置 = max(clear_bottom, flow_bottom + child.margin_top)
                        // 当元素自身 margin-top 足够大时，即使不折叠也已在浮动之下
                        let uncollapsed_pos = flow_bottom + child.margin_top;
                        child.y = content_y_offset + clear_bottom.max(uncollapsed_pos);
                        // R1316 defect ①/②：正 clearance 被应用 → 该子的 margin 不再
                        // collapse-through（§8.3.1），它建立流位置，且其后所有兄弟
                        // 须以 flow_bottom 重定位（taffy 未知 clearance）。
                        clearance_applied = true;
                        // R1318 §8.3.1 containment：clearance 「消耗」了空 cleared 块的 margin-top
                        //（hypothetical 用它定位，clearance 填充余下到 clear_bottom 的间隙）。
                        // 计算 contained parent height 时须从 trailing 折叠链扣除，避免双计。
                        // 仅对 empty cleared block（collapse-through）—— 非空 cleared（replaced
                        // 等）containment 几何不同，本轮不触（避 clear-on-replaced-element 回归）。
                        if crate::margin_collapse::is_empty_block(child) {
                            clearance_consumed_mt += child.margin_top;
                            had_empty_clearance = true;
                        }
                    } else if (clear_bottom - hypothetical_y).abs() < 0.001 {
                        // 零 clearance（hypothetical_y ≈ clear_bottom）：
                        // CSS 2.1 §9.5.2：clearance 引入后，位置 = hypothetical + clearance。
                        // clearance = max(clear_bottom - P, H - P)，其中 P 为不折叠边距位置。
                        // 当 H == clear_bottom 时，P + clearance = H = clear_bottom。
                        // 因此元素位置 = hypothetical_y（使用折叠边距计算的假设位置）。
                        // 零 clearance 仍阻止 margin 折叠（CSSWG resolution），
                        // 但视觉位置与假设位置相同。
                        child.y = content_y_offset + hypothetical_y;
                    } else {
                        // hypothetical_y > clear_bottom：元素（含 margin）已过浮动。
                        // R1393 adjoining-float 吸收：若 clear 清除的是「嵌套在非 BFC 后代中的
                        // 浮动」（clear_bottom 由 nested_bottom 决定），则 clear 的 margin 经
                        // 该 wrapper 与浮动 adjoining（§8.3.1+§9.5.2）——须 apply clearance 把
                        // clear 定到 clear_bottom 并吸收 margin，否则 margin 把 clear 推到
                        // hypothetical 留下红间隙（adjoining-float-before-clearance：clear mt:400
                        // 应落 float 底 50 而非 450）。须配合 R1392 nested-float 可见性。
                        // env `ZW_ADJOINING_FLOAT_CLEARANCE=0` 关闭（kill-switch，default-on）。
                        let nested_for_side = match child.clear {
                            ClearValue::Left => nested_left_bottom,
                            ClearValue::Right => nested_right_bottom,
                            _ => nested_left_bottom.max(nested_right_bottom),
                        };
                        // adjoining 信号：clear 清除的是「嵌套在非 BFC 后代中的浮动」
                        //（clear_bottom 由 nested_bottom 决定）。direct-float 变体（new-fc 谱系）
                        // 用 clear_bottom>flow_bottom 信号会过冲（4 案回归），故仅 gate 嵌套浮动。
                        let adjoining = std::env::var("ZW_ADJOINING_FLOAT_CLEARANCE").as_deref() != Ok("0")
                            && nested_for_side > 0.0
                            && (clear_bottom - nested_for_side).abs() < 0.5;
                        if adjoining {
                            child.y = content_y_offset + clear_bottom;
                            clearance_applied = true;
                            ran_adjoining_clearance = true;
                        } else {
                            // 普通情形：margin 正常折叠，clear 留在 hypothetical。
                            child.y = content_y_offset + hypothetical_y;
                        }
                    }
                    float_y_offset = (original_taffy_y - child.y).max(0.0);
                    // R1316 defect ①：正 clearance 使流发散，标记后续兄弟须重定位。
                    if clearance_applied {
                        had_clearance = true;
                    }
                }
                ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd => {
                    // 非 clear 的普通元素：使用独立的 flow_bottom 追踪计算正确位置
                    // 简单的 child.y -= float_y_offset 无法正确处理 margin 折叠，
                    // 因为 taffy 将 float 当作 block 排列，其 margin 折叠方式
                    // 与 float 不存在时的折叠方式不同。
                    //
                    // R1316 defect ①：除残留 float 空间（float_y_offset>0）外，
                    // 若本容器此前出现过正 clearance（had_clearance），taffy 对本
                    // 子的定位同样不可信 —— 须以 flow_bottom 重定位。
                    if float_y_offset > 0.0 || had_clearance {
                        // CSS 2.1：非 clear 元素的位置 = 正常流位置（假设 float 不存在）
                        let collapsed_margin =
                            crate::margin_collapse::collapse_two_margins(last_flow_mb, child.margin_top);
                        let correct_y = flow_bottom + collapsed_margin;
                        child.y = content_y_offset + correct_y;
                        // 更新 float_y_offset 以反映 taffy Y 与正确 Y 的差异
                        float_y_offset = (original_taffy_y - child.y).max(0.0);
                    }
                }
            }

            // 空块自身的上下 margin 会自折叠，并继续传递给后继兄弟；
            // 它自身不应把 flow_bottom 往下推进一段“实心高度”。
            //
            // R1316 defect ②：但被应用了正 clearance 的元素（即便高度为 0）破坏
            // collapse-through（§8.3.1）—— 它建立流位置，flow_bottom 须推进到其
            // border-box 底边，其 margin-bottom 不再 collapse-through 传给后继。
            // 否则 cleared 空块之后的首个兄弟会用陈旧的 flow_bottom 定位，违反
            // DOM 顺序（出现在 cleared 元素之前）。
            let establishes_flow_position = !crate::margin_collapse::is_empty_block(child) || clearance_applied;
            if !establishes_flow_position {
                let collapsed_self_margin =
                    crate::margin_collapse::collapse_two_margins(child.margin_top, child.margin_bottom);
                last_flow_mb = crate::margin_collapse::collapse_two_margins(last_flow_mb, collapsed_self_margin);
            } else {
                // 更新流内容追踪（使用 content-relative 坐标）
                flow_bottom = child.y - content_y_offset + child.height;
                last_flow_mb = child.margin_bottom;
            }

            // BFC 浮动排斥（CSS 2.1 §9.5）：
            // 建立 BFC 的块级元素不得与同容器的浮动元素重叠。
            // 当 BFC 元素的垂直范围与浮动元素重叠时，水平偏移以避开浮动。
            if child.is_block_level
                && !child.is_absolute
                && !child.is_fixed
                && matches!(child.float, FloatValue::None)
                && crate::margin_collapse::establishes_bfc(child)
            {
                let child_top = child.y;
                let child_bottom = child.y + child.height;

                // R1730 Slice 5（RFC §10.2）：多-float BFC 协调。当 BFC 子同时垂直重叠 ≥2 个
                // 同容器 float 时，per-float 循环（R1369/R1722/R1728）独立逐 float pushdown/squeeze
                // 会 over-push（floats-wrap-top-below-bfc-003l span2 被推到 float R 底 164 而非
                // float L 底 89）。协调：收集所有垂直重叠 float，按候选 y（自然 y ∪ 各重叠 float
                // bottom，升序）找首个使 BFC 不重叠任何 float 的 y（该 y 处所有现役 float 联合约束
                // 出可行 x 区间 [x_lo, x_hi]，BFC 宽能放下即可行），取 x_lo（尊 BFC 既有 margin_left
                // 左对齐）；找不到可行 y 则下到最晚 float bottom。margin-left:auto 的右对齐特化须
                // margin_auto 字段（LayoutBox 暂无），001r 此版落 x=margin_left（与现状一致非回归）。
                // kill-switch ZW_BFC_MULTIFLOAT_COORD=0（default-on）。scope gate：≥2 垂直重叠 float
                // + 无后续 in-flow block 同胞——单 float 走既有 per-float 循环，零回归基线。
                let coord_on = std::env::var("ZW_BFC_MULTIFLOAT_COORD").as_deref() != Ok("0");
                let mut coord_handled = false;
                if coord_on
                    && !has_following_block_sibling[idx]
                    && !child.declared_width_auto
                    && !child.is_layout_container
                {
                    let overlapping: Vec<&FloatGeom> = float_geometries
                        .iter()
                        .filter(|g| {
                            let (_, _, fy, _, fh, _) = g;
                            let fbottom = *fy + *fh;
                            child_top < fbottom && child_bottom > *fy
                        })
                        .collect();
                    if overlapping.len() >= 2 {
                        let w = child.width;
                        let h = child.height;
                        // 候选 y：自然 y + 各重叠 float bottom（> 自然 y），升序去重。
                        let mut y_candidates: Vec<f32> = vec![child.y];
                        for g in &overlapping {
                            let (_, _, fy, _, fh, _) = g;
                            let fb = *fy + *fh;
                            if fb > child.y + 0.5 {
                                y_candidates.push(fb);
                            }
                        }
                        y_candidates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        y_candidates.dedup_by(|a, b| (*a - *b).abs() < 0.5);
                        let mut placed: Option<(f32, f32)> = None;
                        for &cand_y in &y_candidates {
                            // 该 y 处可行 x 区间：左 float 推 x_lo 右移，右 float 推 x_hi 左移。
                            let mut x_lo = child.margin_left;
                            let mut x_hi = (container_width - w).max(child.margin_left);
                            for g in &overlapping {
                                let (fd, fx, fy, fwidth, fh, fmargin_r) = g;
                                let fbottom = *fy + *fh;
                                if !(cand_y < fbottom && cand_y + h > *fy) {
                                    continue;
                                }
                                match fd {
                                    FloatValue::Left => {
                                        // BFC 须在 float margin-box 右侧：x >= fx + width + margin_r。
                                        x_lo = x_lo.max(fx + fwidth + fmargin_r);
                                    }
                                    FloatValue::Right => {
                                        // BFC 须在 float 左侧：x + w <= fx（float 左 margin 未存，用 border-box 左）。
                                        x_hi = x_hi.min(fx - w);
                                    }
                                    _ => {}
                                }
                            }
                            if x_lo <= x_hi + 0.5 {
                                placed = Some((x_lo, cand_y));
                                break;
                            }
                        }
                        if let Some((px, py)) = placed {
                            child.x = px;
                            child.y = py;
                        } else {
                            // 无可行 y（BFC 宽放不下任何候选处）→ 下到最晚重叠 float bottom。
                            let max_bottom = overlapping
                                .iter()
                                .map(|g| {
                                    let (_, _, fy, _, fh, _) = g;
                                    *fy + *fh
                                })
                                .fold(child.y, f32::max);
                            child.y = max_bottom;
                            child.x = child.margin_left;
                        }
                        coord_handled = true;
                    }
                }

                if !coord_handled {
                    for (float_dir, float_x, float_y, float_border_w, float_h, float_margin_r) in &float_geometries {
                        let float_top = *float_y;
                        let float_bottom = *float_y + *float_h;

                        // 检查垂直重叠
                        if !(child_top < float_bottom && child_bottom > float_top) {
                            continue;
                        }
                        match float_dir {
                            FloatValue::Left => {
                                // 左浮动：将 BFC 元素推到浮动元素的 margin-box 右侧
                                // float_x 是边框盒左边，加上边框宽度和右 margin
                                let avoidance_x = float_x + float_border_w + float_margin_r;
                                // R1369：definite-width BFC（width 未填满容器）若 overflow 容器
                                //（child.x + width > container_width），应推到 float 下方（CSS §9.5：
                                // BFC border-box 不重叠 float；definite 宽度保持不 shrink）。
                                // **关键**：taffy 0.12 native float 可能已把 BFC 推到 float 右
                                //（child.x == avoidance_x），故本检查须在 `avoidance_x > child.x`
                                // 之外做（否则 taffy 已推时整块 skip）。auto-width BFC（填满容器）
                                // 仍走 shrink-to-fit（else 分支）。仅无后续 in-flow block 同胞时推下。
                                let overflows = child.x + child.width > container_width + 0.5;
                                let is_definite_width = child.width < container_width - 0.5;
                                // R1728：补充「放不下 float 旁」判定。原 R1369 gate 仅查「溢出容器」，
                                // 漏「float 占满宽致其右可用宽 < BFC 声明宽」——此时 BFC 同样无法旁置，
                                // 须推到 float 下方（CSS §9.5：BFC border-box 不重叠 float；definite
                                // 宽度不 shrink）。floats-wrap-top-below-bfc-002r span2：float:left 300
                                // 宽，span2（overflow:hidden）声明宽 200 放不下其右 [300,400]=100 → 应
                                // pushdown 到 float 下方（原行为是 squeeze 到 x=300/w=100，错）。
                                // **关键**：仅对「声明宽（非 auto）」BFC 触发——auto 宽 BFC（如
                                // floats-bfc-003 的 #bfc、new-fc-beside-float）须 shrink-to-fit 旁置
                                //（spec：BFC 占 float 旁可用宽），用 declared_width_auto 区分（width
                                // 已 shrink 的 auto BFC 其 child.width < container_width 但非「definite」）。
                                // kill-switch ZW_BFC_LEFT_FIT_PUSHBELOW=0 回退纯溢出 gate。
                                let avail_beside = (container_width - avoidance_x).max(0.0);
                                let fits_beside = child.width <= avail_beside + 0.5;
                                let left_fit_pushbelow =
                                    std::env::var("ZW_BFC_LEFT_FIT_PUSHBELOW").as_deref() != Ok("0");
                                let must_pushdown = overflows
                                    || (left_fit_pushbelow
                                        && is_definite_width
                                        && !child.declared_width_auto
                                        && !fits_beside);
                                if is_definite_width && !has_following_block_sibling[idx] && must_pushdown {
                                    if float_bottom > child.y {
                                        child.y = float_bottom;
                                    }
                                    // 回正常流位置（taffy float push 前）：block border-box 左 =
                                    // 父 content-box 左 + margin_left（child.x 相对父 content-box）。
                                    child.x = child.margin_left;
                                } else if avoidance_x > child.x {
                                    // taffy 未推 → ZW 推到 float 右 + shrink-to-fit（原行为）
                                    child.x = avoidance_x;
                                    // 缩小宽度以不超出容器
                                    let max_width = container_width - child.x;
                                    if child.width > max_width {
                                        child.width = max_width.max(0.0);
                                        shrink_bfc_content_width(child);
                                    }
                                }
                            }
                            FloatValue::Right if child.x + child.width > *float_x => {
                                // R1722：float:right definite-width BFC 放不下 float 左侧可用宽
                                //（child.x + width > float_x）→ 推到 float 下方（mirror of R1369 左
                                // float overflows 推下，CSS §9.5：BFC border-box 不重叠 float；definite
                                // 宽度保持不 shrink）。仅 definite-width + 无后续 in-flow block 同胞时推下，
                                // 否则保持 shrink-to-fit（原行为）。kill-switch ZW_BFC_RIGHT_PUSHBELOW=0。
                                let is_definite_width = child.width < container_width - 0.5;
                                if std::env::var("ZW_BFC_RIGHT_PUSHBELOW").as_deref() != Ok("0")
                                    && is_definite_width
                                    && !has_following_block_sibling[idx]
                                    && !child.is_layout_container
                                {
                                    if float_bottom > child.y {
                                        child.y = float_bottom;
                                    }
                                    child.x = child.margin_left;
                                } else {
                                    // 右浮动：缩小 BFC 元素宽度以不重叠 float 的 margin-box
                                    let new_width = float_x - child.x;
                                    child.width = new_width.max(0.0);
                                    shrink_bfc_content_width(child);
                                }
                            }
                            _ => {}
                        }
                    }
                } // end if !coord_handled（R1730 Slice 5：coord_handled 时跳过 per-float 循环）

                // R1619 Slice 2（嵌套 BFC 祖先 float 下沉）：直接同胞 float 由上方 float_geometries
                // 循环（R1369 左 / 右 shrink）处理；此处处理**祖先 float**（经 inherited_floats 透传，
                // 即嵌套在非 BFC 后代内的 BFC 看到的外层 float）。declared-width BFC（width 非 auto）
                // 放不下 float 旁可用宽时，下沉到 float 底（CSS §9.5），而非被 shrink 到 0/负宽。
                // 与 R1369 分离：R1369 用 `width < container_width` 代理，嵌套+margin 上下文下 BFC
                // 溢出窄父失效（with-margin-008/009）；此处用 `!declared_width_auto` 精确判定。
                // env ZW_NESTED_BFC_FLOAT_AVOID=0 关闭（kill-switch，default-on）。
                if nested_avoid_on
                    && !inherited_floats.is_empty()
                    && !has_following_block_sibling[idx]
                    && !child.declared_width_auto
                {
                    let ctop = child.y;
                    let cbottom = child.y + child.height;
                    for (fdir, fx, fy, fbw, fh, fmr) in inherited_floats.iter() {
                        let fbottom = fy + fh;
                        // 垂直重叠
                        if !(ctop < fbottom && cbottom > *fy) {
                            continue;
                        }
                        // float 旁可用宽（本容器 border-box 帧）
                        let available = match fdir {
                            FloatValue::Left => container_width - (fx + fbw + fmr),
                            FloatValue::Right => fx - child.x,
                            _ => continue,
                        };
                        if child.width > available + 0.5 {
                            if fbottom > child.y {
                                child.y = fbottom;
                            }
                            // 回正常流左对齐（float 在侧，BFC 下沉后不与 float 同行）。
                            child.x = child.margin_left;
                            break;
                        }
                    }
                }
            }

            // R1392：nested-float clearance——非 BFC 子内的嵌套浮动须对后续 clear 兄弟可见
            //（§9.5：clear 清除同 BFC 上下文内所有浮动）。ZW 的 active_*_float_bottom 仅收集
            // 直接 float 子，嵌套在非 BFC wrapper 内的 float（adjoining-float-before-clearance）
            // 对 clear 不可见。此处在子定位完成后扫描其非 BFC 后代的浮动底边，并入追踪。
            // env `ZW_NESTED_FLOAT_CLEARANCE=0` 关闭（kill-switch，default-on）。
            if std::env::var("ZW_NESTED_FLOAT_CLEARANCE").as_deref() != Ok("0")
                && !child.is_absolute
                && !child.is_fixed
                && matches!(child.float, FloatValue::None)
                && !crate::margin_collapse::establishes_bfc(child)
            {
                let cy = child.y;
                let (nl, nr) = nested_float_bottoms(child, cy, content_y_offset);
                if nl > 0.0 {
                    nested_left_bottom = nested_left_bottom.max(nl);
                }
                if nr > 0.0 {
                    nested_right_bottom = nested_right_bottom.max(nr);
                }
                if nl > active_left_float_bottom {
                    active_left_float_bottom = nl;
                }
                if nr > active_right_float_bottom {
                    active_right_float_bottom = nr;
                }
            }
        }
    } else if content_y_offset == 0.0 && std::env::var("ZW_CLEAR_NO_FLOAT_CONTEXT").as_deref() != Ok("0") {
        // R1389：has_active_float_context=false（容器无直接 float 子，且 inherited float
        // bottom 已 clamp 到 0 = 容器已在所有祖先 float 下方）时，clear 子元素**无浮动可清除**。
        // 但 taffy 0.12 仍基于同 BFC 的祖先 float 对 clear 子误 apply clearance，把 clear 子
        // 推到其 flow 位置之下并膨胀容器高度（no-clearance-due-to-large-margin：red h=83 应 20，
        // clear 在 red 底部而非顶部）。此处对窄情形（容器无 border-top/padding-top + 唯一 in-flow
        // block 子为 clear 元素 + auto-height）将 clear 子重定位到容器 content top（其 margin-top
        // 经无 border/padding-top 的容器折叠穿出，位置本应如此），并按 in-flow 子 border-box 收缩
        // 容器高度。env `ZW_CLEAR_NO_FLOAT_CONTEXT=0` 关闭（kill-switch，default-on）。
        let clear_idx = box_node.children.iter().position(|c| {
            !c.is_absolute
                && !c.is_fixed
                && c.is_block_level
                && !matches!(
                    c.clear,
                    ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
                )
        });
        if let Some(idx) = clear_idx {
            // clear 子须为唯一 in-flow block 子（无其它 block 兄弟），避免重定位影响兄弟流位置
            //（多 block 兄弟情形须完整 flow 跟踪，留多 session）。
            let in_flow_block_count = box_node
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed && c.is_block_level)
                .count();
            if in_flow_block_count == 1 {
                let respect_explicit_height = std::env::var("ZW_FLOAT_RESPECT_HEIGHT").as_deref() != Ok("0");
                let auto_height = !respect_explicit_height || box_node.declared_height_auto;
                if auto_height {
                    // 重定位 clear 子到 content top（margin-top 折叠穿出容器，border-top 落在
                    // 容器 content 原点；content_y_offset 此分支恒为 0）。
                    box_node.children[idx].y = content_y_offset;
                    // 按 in-flow 子 border-box 收缩容器（clear 的 margin-top 不计入高度）。
                    let content_bottom =
                        box_node
                            .children
                            .iter()
                            .filter(|c| !c.is_absolute && !c.is_fixed)
                            .fold(0.0f32, |max_y, c| {
                                let bottom = c.y + c.height + c.margin_bottom;
                                max_y.max(bottom)
                            });
                    let new_content_height = (content_bottom - content_y_offset).max(0.0);
                    if new_content_height < box_node.content_height {
                        box_node.content_height = new_content_height;
                        let new_total = new_content_height
                            + box_node.padding_top
                            + box_node.padding_bottom
                            + box_node.border_top
                            + box_node.border_bottom;
                        if new_total < box_node.height {
                            box_node.height = new_total;
                        }
                    }
                }
            }
        }
    }

    // 调整容器高度：
    // CSS 2.1 §10.6.7：建立 BFC 的容器必须包含浮动元素的外底边
    // （margin-box bottom）。非 BFC 容器中，浮动元素不贡献高度，
    // 因此需要收缩容器以去除 taffy 将 float 当作 block 时多计算的空间。
    //
    // 注意：子元素的 x/y 坐标是相对于父元素 content area 的，
    // 所以 content_bottom 也是相对于 content area 顶部的。
    // 不需要减去 content_y（那是相对于自身 border-box 的局部偏移，不含位置量）。
    if !float_taffy_y.is_empty() || ran_adjoining_clearance {
        let establishes_bfc = crate::margin_collapse::establishes_bfc(box_node);
        // 多列容器虽然建立 BFC（阻止 margin 折叠），
        // 但其内容通过列分布，不应使用 BFC 的浮动包含高度逻辑。
        // 使用非 BFC 路径按正常流内容高度收缩。
        let use_bfc_float_containment = establishes_bfc && !box_node.is_multicol;

        if use_bfc_float_containment {
            // BFC 容器：浮动元素已包含在高度中（taffy 正确计算了含 float 的 auto height）
            // 不需要收缩。但需要确保高度至少覆盖到最低浮动元素的外底边。
            let float_bottom = box_node
                .children
                .iter()
                .filter(|c| !matches!(c.float, FloatValue::None) && !c.is_absolute && !c.is_fixed)
                .fold(0.0f32, |max_y, c| {
                    let bottom = c.y + c.height + c.margin_bottom;
                    max_y.max(bottom)
                });
            if float_bottom > box_node.content_height {
                box_node.content_height = float_bottom;
                let new_total = float_bottom
                    + box_node.padding_top
                    + box_node.padding_bottom
                    + box_node.border_top
                    + box_node.border_bottom;
                if new_total > box_node.height {
                    box_node.height = new_total;
                }
            }
        } else {
            // 非 BFC 容器：浮动元素不贡献高度，收缩容器
            //
            // R1277 ④：显式高度（definite height）容器不应被收缩。CSS §10.5：显式高度的
            // used height 即显式值，float 重定位后 content_bottom 下降（如 floats-006 float
            // 上提后 content_bottom 100 < height 200）不应改变容器高度——内容仅溢出/不足。
            // 旧实现无此守卫，致 `#div1{height:200px}` 在 float 上提后被错误塌缩到 100
            //（R1273 实证，R1277 经 DIV1_TRACE 二分定位塌缩源=本函数 L779 非 BFC 收缩路径，
            // 非 exclude_floats_from_non_bfc_auto_height）。env `ZW_FLOAT_RESPECT_HEIGHT=0`
            // 可关闭（kill-switch，default-on；同 R109_BACKFILL 约定）。
            let respect_explicit_height = std::env::var("ZW_FLOAT_RESPECT_HEIGHT").as_deref() != Ok("0");
            if respect_explicit_height && !box_node.declared_height_auto {
                // 显式高度容器：跳过收缩（CSS §10.5 used height = 显式值）。
            } else {
                // R1323 §8.3.1：auto-height 非 BFC + clear 子（进入此块即有 float 上下文）
                // → clearance_active。覆盖非空 cleared（margin-collapse-clear-015：clear-left
                // 有子，empty-gate 排除 containment math 但仍须 sibling-shift）+ negative clearance
                //（hypothetical>clear_bottom，clearance 仍 stop collapse per 014 assert）。
                // sibling-shift leak 公式按 declared_margin_bottom 自门控（027 #div2 declared mb 安全）。
                let has_clear_child = box_node.children.iter().any(|c| {
                    !matches!(
                        c.clear,
                        ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
                    )
                });
                if has_clear_child {
                    box_node.clearance_active = true;
                }
                if had_empty_clearance {
                    // R1317 §8.3.1 containment（empty cleared）：trailing collapse-through 链留父内。
                    // 父 content_height =（最后建立流位置子 border-box 底）+（trailing 链 − consumed mt）。
                    // 例 012：flow_bottom=100 + (last_flow_mb=140 − consumed=40) = 200。
                    let contained_chain = (last_flow_mb - clearance_consumed_mt).max(0.0);
                    let content_height = (flow_bottom + contained_chain).max(0.0);
                    if content_height > box_node.content_height {
                        box_node.content_height = content_height;
                        let new_total = content_height
                            + box_node.padding_top
                            + box_node.padding_bottom
                            + box_node.border_top
                            + box_node.border_bottom;
                        if new_total > box_node.height {
                            box_node.height = new_total;
                        }
                        // 仅当 containment 实际扩张时才标记 had_clearance（exclude_floats 跳过用）。
                        box_node.had_clearance = true;
                    }
                } else {
                    let content_bottom =
                        box_node
                            .children
                            .iter()
                            .filter(|c| !c.is_absolute && !c.is_fixed)
                            .fold(0.0f32, |max_y, c| {
                                let bottom = c.y + c.height + c.margin_bottom;
                                max_y.max(bottom)
                            });
                    // R1324：content_bottom 为 border-box 相对（c.y 含 content_y_offset =
                    // border_top + padding_top），须换算为 content 相对后再赋 content_height
                    //（后者不含 border/padding）。否则带 border-top/padding-top 的容器
                    //（margin-collapse-clear-015 border-top:1px）content_height 会多算一段
                    // content_y_offset，容器偏高 → 后续 in-flow 兄弟整体下移（015 残余 1px）。
                    let content_height = (content_bottom - content_y_offset).max(0.0);
                    // 如果内容区域实际高度小于 taffy 计算的高度，收缩容器
                    if content_height < box_node.content_height {
                        box_node.content_height = content_height;
                        // 更新总高度（包含 padding + border）
                        let new_total = content_height
                            + box_node.padding_top
                            + box_node.padding_bottom
                            + box_node.border_top
                            + box_node.border_bottom;
                        // 仅当新高度更小时才更新（不扩大容器）
                        if new_total < box_node.height {
                            box_node.height = new_total;
                        }
                    }
                }
            }
        }
    }

    // R1318/R1319：had_clearance 仅在上方 containment 分支实际扩张 auto-height 容器时置 true
    //（非显式高度、且确有 empty-block clearance）。供 exclude_floats 跳过收缩 + sibling-shift
    // 位移后续兄弟。box_node.had_clearance 默认 false（literal/Default 初始化）。

    // 递归处理子容器
    for (idx, child) in box_node.children.iter_mut().enumerate() {
        let (left_ctx, right_ctx) = child_float_contexts
            .get(idx)
            .copied()
            .unwrap_or((inherited_left_bottom, inherited_right_bottom));
        let child_content_abs_y = box_content_abs_y + child.y + child.content_y;
        if crate::margin_collapse::establishes_bfc(child) {
            // BFC 子：独立浮动上下文，祖先 float 不透传。
            adjust_float_positions_with_context(child, child_content_abs_y, 0.0, 0.0, &[]);
        } else {
            // 非 BFC 子：其内 float 与外层同 BFC 上下文。R1619 Slice 2 透传 all_floats
            //（祖先 + 自身）到子 border-box 帧（减 child.x/child.y），使嵌套 BFC 后代能避开外层 float。
            let child_inherited: Vec<FloatGeom> = if nested_avoid_on && !all_floats.is_empty() {
                // all_floats 借用 box_node.children；先取出 child 偏移再 map（避免与 child mut 借用冲突）。
                let (cx, cy) = (child.x, child.y);
                all_floats
                    .iter()
                    .map(|(d, fx, fy, w, h, mr)| (d.clone(), fx - cx, fy - cy, *w, *h, *mr))
                    .collect()
            } else {
                Vec::new()
            };
            adjust_float_positions_with_context(
                child,
                child_content_abs_y,
                box_content_abs_y + left_ctx,
                box_content_abs_y + right_ctx,
                &child_inherited,
            );
        }
    }
}
