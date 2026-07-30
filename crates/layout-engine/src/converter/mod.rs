//! ComputedStyle → taffy::Style 转换层。
//!
//! 将 [`ComputedStyle`] 的字段映射到 taffy 的 [`taffy::Style`] 结构体，
//! 这是布局引擎的关键适配层。

use zero_css_parser::values::{
    AlignmentValue, BoxSizingValue, ClearValue, DisplayValue, FlexDirectionValue, FlexWrapValue, FloatValue,
    LengthValue, OverflowValue, PositionValue,
};
use zero_style_system::{
    AlignContentValue, BorderCollapseValue, BorderStyleValue, ComputedStyle, FlexBasisValue, GridAutoFlowValue,
    GridLineValue, JustifyItemsValue, JustifySelfValue,
};

use taffy::prelude::*;

/// grid-template-areas 区域映射类型。
///
/// 键为区域名（如 "header"），值为 (row_start, row_end, col_start, col_end)，
/// 行号和列号均为 1-based，区间为 [start, end)。
pub type GridAreaMap = std::collections::HashMap<String, (i16, i16, i16, i16)>;

/// 将 ComputedStyle 转换为 taffy::Style。
///
/// 处理所有 CSS 属性到 taffy 布局属性的映射。
/// `parent_areas` 为父级 grid 容器的 grid-template-areas 区域映射，
/// 用于将子元素的 GridLineValue::Name 解析为行号。
/// `viewport_w`/`viewport_h` 用于解析 vw/vh/vmin/vmax 视口相对单位。
pub fn computed_style_to_taffy(
    style: &ComputedStyle,
    parent_areas: Option<&GridAreaMap>,
    viewport_w: f32,
    viewport_h: f32,
) -> taffy::Style {
    let vw = viewport_w;
    let vh = viewport_h;

    // CSS 2.1 §10.3.5: 浮动非替换元素的 margin-left/right: auto 解析为 0
    let is_float = matches!(style.float, FloatValue::Left | FloatValue::Right);

    // CSS Flexbox §10.1：visibility:collapse 的 flex item 主尺寸归零（成为
    // strut），不再占用主轴空间，但其交叉尺寸仍贡献给 flex line 的高度。
    // 这是 collapse 与 hidden 的关键区别（hidden 仍占满尺寸）。paint 层已对
    // collapse 跳过绘制，此处仅修正布局尺寸。
    // flex-basis 在非 flex 容器中被 taffy 忽略，因此对非 flex 折叠元素无副作用。
    let collapsed = matches!(style.visibility, zero_css_parser::values::VisibilityValue::Collapse);

    // CSS 2.1 §17.5.3/17.5.4：行组和行的 border/padding/margin 无视觉效果。
    // 在 taffy 层面归零，防止 taffy 将这些属性计入布局计算。
    let is_table_internal = matches!(
        style.display,
        DisplayValue::TableRowGroup
            | DisplayValue::TableHeaderGroup
            | DisplayValue::TableFooterGroup
            | DisplayValue::TableRow
    );

    // CSS §8.4 margin：「margin applies to: all elements except elements with table display
    // types other than table-caption, table and inline-table」——即 margin 不应用于 table-cell /
    // table-column / table-column-group（以及 is_table_internal 的行组/行）。cell 的 **padding**
    // 仍应用（§17.5），故此 set 独立于 padding suppression（padding 字段仍用 is_table_internal，
    // 不含 TableCell）。driving test：margin-applies-to-005/006/007（margin:50px 应被忽略）。
    let is_margin_suppressed = is_table_internal
        || matches!(
            style.display,
            DisplayValue::TableCell | DisplayValue::TableColumn | DisplayValue::TableColumnGroup
        );

    // CSS 2.1 §17.6.2（collapsing border model）：border-collapse:collapse 时 table 元素的
    // padding 不应用（「In this model, the [table's] padding is not applied」）。ZW 此前对
    // display:table 的 padding 照常解析，致 collapsing-border-model-011/013 渲染 300×300
    // （应 100×100，100px padding 被错误计入）。仅 table 盒本身（display:table/inline-table），
    // 单元格 padding 不受影响（§17.5：cell padding 始终应用）。
    let is_collapsed_table = matches!(style.display, DisplayValue::Table | DisplayValue::InlineTable)
        && matches!(style.border_collapse, BorderCollapseValue::Collapse);

    // CSS Position §6（css-position-1）：inset 属性（top/right/bottom/left）仅对
    // 非 static 定位元素生效。static 元素的 inset 必须忽略（R689）。
    let is_static = matches!(style.position, PositionValue::Static);

    taffy::Style {
        display: convert_display(&style.display),
        // M3 step(b) 试：native float 激活，但保留 ZW adjust_float_positions（覆盖 native 定位）
        float: match style.float {
            FloatValue::Left | FloatValue::InlineStart => taffy::style::Float::Left,
            FloatValue::Right | FloatValue::InlineEnd => taffy::style::Float::Right,
            _ => taffy::style::Float::None,
        },
        clear: match style.clear {
            ClearValue::Left | ClearValue::InlineStart => taffy::style::Clear::Left,
            ClearValue::Right | ClearValue::InlineEnd => taffy::style::Clear::Right,
            ClearValue::Both => taffy::style::Clear::Both,
            _ => taffy::style::Clear::None,
        },
        box_sizing: convert_box_sizing(&style.box_sizing),
        overflow: taffy::geometry::Point {
            x: convert_overflow(&style.overflow_x),
            y: convert_overflow(&style.overflow_y),
        },
        scrollbar_width: match style.scrollbar_width {
            zero_style_system::ScrollbarWidthComputedValue::Auto => 15.0,
            zero_style_system::ScrollbarWidthComputedValue::Thin => 8.0,
            zero_style_system::ScrollbarWidthComputedValue::None => 0.0,
        },
        position: convert_position(&style.position),
        inset: if is_static {
            // static 定位：inset 全 Auto，taffy（Relative + Auto inset）不偏移，
            // 与 static 正常流语义一致（R689）。
            taffy::geometry::Rect {
                left: taffy::style::LengthPercentageAuto::auto(),
                right: taffy::style::LengthPercentageAuto::auto(),
                top: taffy::style::LengthPercentageAuto::auto(),
                bottom: taffy::style::LengthPercentageAuto::auto(),
            }
        } else {
            taffy::geometry::Rect {
                left: convert_length_to_lpa(&style.left, false, vw, vh),
                right: convert_length_to_lpa(&style.right, false, vw, vh),
                top: convert_length_to_lpa(&style.top, false, vw, vh),
                bottom: convert_length_to_lpa(&style.bottom, false, vw, vh),
            }
        },
        size: if style.contain.has_size() {
            // R2239：contain:size — auto 尺寸解析为 0（content 不贡献 size），显式尺寸保留。
            // R2256：contain-intrinsic-size 覆盖 auto 维的 0（CSS Sizing 4）——size containment
            // 元素的 auto 尺寸取 contain-intrinsic-size（若有），否则 0。driving: css-sizing
            // contain-intrinsic-size-001..（contain:size + contain-intrinsic-size: 111px 222px → 111×222）。
            let cis_dim = |cis: &Option<LengthValue>| match cis {
                Some(l) => convert_length_to_dimension(l, vw, vh),
                None => taffy::style::Dimension::length(0.0),
            };
            taffy::geometry::Size {
                width: match &style.width {
                    LengthValue::Auto => cis_dim(&style.contain_intrinsic_width),
                    _ => convert_length_to_dimension(&style.width, vw, vh),
                },
                height: match &style.height {
                    LengthValue::Auto => cis_dim(&style.contain_intrinsic_height),
                    _ => convert_length_to_dimension(&style.height, vw, vh),
                },
            }
        } else {
            taffy::geometry::Size {
                width: convert_length_to_dimension(&style.width, vw, vh),
                // R1018：flex/inline-flex 容器的 height:MaxContent/MinContent（含 bare fit-content
                // 经 parser 映射）映射 Auto（content-based），非 length(0）。flex 容器 height:auto
                // = 内容最高 item（fit-content-item-002/003/004 驱动）。width 的 MaxContent→0 是
                // R181c 实测要求（gate grow 依赖），height 无 gate 还原。仅限 flex 容器——grid/block
                // 的 height:max-content 在空 item 时应塌缩（max-content of empty=0），Auto 会触发
                // align-self stretch 误拉伸（grid-item-non-auto-height-stretch-001 回归）。
                height: if matches!(style.display, DisplayValue::Flex | DisplayValue::InlineFlex)
                    && matches!(style.height, LengthValue::MaxContent | LengthValue::MinContent)
                {
                    taffy::style::Dimension::auto()
                } else {
                    convert_length_to_dimension(&style.height, vw, vh)
                },
            }
        },
        // R2239：contain:size 须同时覆盖 auto min-size → 0（否则 inline-block 的 auto
        // min-size = min-content 会阻止收缩到 0）。
        min_size: if style.contain.has_size() {
            taffy::geometry::Size {
                width: match &style.min_width {
                    LengthValue::Auto => taffy::style::Dimension::length(0.0),
                    _ => convert_length_to_dimension(&style.min_width, vw, vh),
                },
                height: match &style.min_height {
                    LengthValue::Auto => taffy::style::Dimension::length(0.0),
                    _ => convert_length_to_dimension(&style.min_height, vw, vh),
                },
            }
        } else {
            taffy::geometry::Size {
                width: convert_length_to_dimension(&style.min_width, vw, vh),
                height: convert_length_to_dimension(&style.min_height, vw, vh),
            }
        },
        max_size: taffy::geometry::Size {
            width: convert_max_length_to_dimension(&style.max_width, vw, vh),
            height: convert_max_length_to_dimension(&style.max_height, vw, vh),
        },
        aspect_ratio: style.aspect_ratio,
        margin: if is_margin_suppressed {
            taffy::geometry::Rect::zero()
        } else {
            // R1058 CSS §8.3：非替换 inline 元素（display:inline）的垂直 margin 无效果
            //（不影响行盒高度/布局）。替换 inline（img 等）UA 默认 InlineBlock 不在此列。
            // 旧实现把 inline 的 margin-top/bottom 原样喂给 taffy，致 inline 的垂直 margin
            // 错误生效（block-in-inline-vertical-margins-on-span-ignored：span mt/bt:50
            // 错误推开块子间距；split inline 的匿名块盒经 computed_style_to_taffy 继承同 bug）。
            // 水平 margin 保留（inline 水平 margin 有效，作用于 IFC 内 inline 片段）。
            let inline_vmargin_zero = matches!(style.display, DisplayValue::Inline);
            let zero = taffy::style::LengthPercentageAuto::length(0.0_f32);
            taffy::geometry::Rect {
                left: convert_length_to_lpa(&style.margin_left, is_float, vw, vh),
                right: convert_length_to_lpa(&style.margin_right, is_float, vw, vh),
                top: if inline_vmargin_zero {
                    zero
                } else {
                    convert_length_to_lpa(&style.margin_top, false, vw, vh)
                },
                bottom: if inline_vmargin_zero {
                    zero
                } else {
                    convert_length_to_lpa(&style.margin_bottom, false, vw, vh)
                },
            }
        },
        padding: if is_table_internal || is_collapsed_table {
            taffy::geometry::Rect::zero()
        } else {
            taffy::geometry::Rect {
                left: convert_length_to_lp(&style.padding_left, vw, vh),
                right: convert_length_to_lp(&style.padding_right, vw, vh),
                top: convert_length_to_lp(&style.padding_top, vw, vh),
                bottom: convert_length_to_lp(&style.padding_bottom, vw, vh),
            }
        },
        border: if is_table_internal {
            taffy::geometry::Rect::zero()
        } else {
            // CSS §8.5.3：border-style 为 none/hidden 时 border-width 计算为 0（不进布局盒）。
            // 否则 `border: none` 的隐含 medium 宽度会错误撑大盒模型。
            let border_lp = |w: &LengthValue, s: &BorderStyleValue| -> taffy::style::LengthPercentage {
                if matches!(s, BorderStyleValue::None | BorderStyleValue::Hidden) {
                    convert_length_to_lp(&LengthValue::Px(0.0), vw, vh)
                } else {
                    convert_length_to_lp(w, vw, vh)
                }
            };
            taffy::geometry::Rect {
                left: border_lp(&style.border_left_width, &style.border_left_style),
                right: border_lp(&style.border_right_width, &style.border_right_style),
                top: border_lp(&style.border_top_width, &style.border_top_style),
                bottom: border_lp(&style.border_bottom_width, &style.border_bottom_style),
            }
        },
        align_items: convert_alignment_to_align_items(&style.align_items),
        align_self: convert_alignment_to_align_self(&style.align_self),
        align_content: convert_align_content(&style.align_content),
        justify_content: grid_justify_content(&style.justify_content, &style.display),
        justify_items: convert_justify_items(&style.justify_items),
        justify_self: convert_justify_self(&style.justify_self),
        gap: taffy::geometry::Size {
            // column-gap 长写属性优先；若未设置（0px），回退到 gap 简写
            width: {
                let col = convert_length_to_lp(&style.column_gap, vw, vh);
                if col == taffy::style::LengthPercentage::length(0.0_f32) {
                    convert_length_to_lp(&style.gap, vw, vh)
                } else {
                    col
                }
            },
            // row-gap 长写属性优先；若未设置（0px），回退到 gap 简写
            height: {
                let row = convert_length_to_lp(&style.row_gap, vw, vh);
                if row == taffy::style::LengthPercentage::length(0.0_f32) {
                    convert_length_to_lp(&style.gap, vw, vh)
                } else {
                    row
                }
            },
        },
        grid_template_rows: parse_grid_tracks(&style.grid_template_rows),
        grid_template_columns: parse_grid_tracks(&style.grid_template_columns),
        grid_auto_flow: convert_grid_auto_flow(&style.grid_auto_flow),
        grid_auto_rows: parse_grid_auto_tracks(&style.grid_auto_rows),
        grid_auto_columns: parse_grid_auto_tracks(&style.grid_auto_columns),
        grid_row: {
            let rs = resolve_named_area(&style.grid_row_start, parent_areas, "row-start");
            let re = resolve_named_area(&style.grid_row_end, parent_areas, "row-end");
            taffy::geometry::Line {
                start: convert_grid_line(&rs),
                end: convert_grid_line(&re),
            }
        },
        grid_column: {
            let cs = resolve_named_area(&style.grid_column_start, parent_areas, "col-start");
            let ce = resolve_named_area(&style.grid_column_end, parent_areas, "col-end");
            taffy::geometry::Line {
                start: convert_grid_line(&cs),
                end: convert_grid_line(&ce),
            }
        },
        flex_direction: convert_flex_direction(&style.flex_direction),
        flex_wrap: convert_flex_wrap(&style.flex_wrap),
        flex_basis: if collapsed
            && (std::env::var("ZW_VC_NONFLEX_STRUT").as_deref() == Ok("0") || (style.flex_grow as f32) > 0.0)
        {
            // visibility:collapse flex item 主尺寸归零（§10.1 strut）：
            // - **flexible** collapsed（flex-grow>0）→ 0（flexbox-collapsed-item-horiz-001 Row4）
            // - ③ OFF（ZW_VC_NONFLEX_STRUT=0）→ 所有 collapsed → 0（旧行为）
            // **非-flexible** collapsed（flex-grow==0，③ ON）保留原 flex-basis 作 strut
            // 保宽——CSS Flexbox §10.1「item continues to participate in intrinsic
            // main-size as if visible」，chromium oracle Row1 非-flexible collapsed 贡献
            // 原 base（20px）非 0。旧代码对所有 collapsed 归零致 Row1 container 2px（应 22）。
            taffy::style::Dimension::length(0.0_f32)
        } else {
            convert_flex_basis(&style.flex_basis, vw, vh)
        },
        flex_grow: if collapsed { 0.0 } else { style.flex_grow as f32 },
        flex_shrink: if collapsed { 0.0 } else { style.flex_shrink as f32 },
        ..taffy::Style::default()
    }
}

/// 对垂直书写模式下的元素交换 taffy 属性的水平/垂直轴。
///
/// CSS Writing Modes §7.1：在垂直书写模式中，水平维度的布局规则应用于垂直维度，反之亦然。
/// 这意味着对于 taffy 布局：
/// - width ↔ height（尺寸沿轴互换）
/// - left ↔ top, right ↔ bottom（inset 互换）
/// - margin/padding/border 的 left ↔ top, right ↔ bottom
///
/// 此函数在 `computed_style_to_taffy` 之后调用，对位于垂直书写模式容器中的元素
/// 进行轴交换，使 taffy 仍然以「水平=行内，垂直=块」的模型计算布局，
/// 然后在提取布局结果时通过坐标交换还原正确的视觉位置。
///
/// 交换盒模型属性（inset、size、margin、padding、border）
/// 以及 flex-direction（在垂直书写模式中，CSS row → taffy column）。
/// grid 属性不交换（它们有自己的方向处理）。
pub fn apply_vertical_writing_mode(style: &mut taffy::Style) {
    // 交换 inset: left ↔ top, right ↔ bottom
    std::mem::swap(&mut style.inset.left, &mut style.inset.top);
    std::mem::swap(&mut style.inset.right, &mut style.inset.bottom);

    // 交换 flex-direction：垂直书写模式中 CSS row 的主轴是块轴（垂直）
    // Row ↔ Column, RowReverse ↔ ColumnReverse
    match style.flex_direction {
        taffy::style::FlexDirection::Row => {
            style.flex_direction = taffy::style::FlexDirection::Column;
        }
        taffy::style::FlexDirection::RowReverse => {
            style.flex_direction = taffy::style::FlexDirection::ColumnReverse;
        }
        taffy::style::FlexDirection::Column => {
            style.flex_direction = taffy::style::FlexDirection::Row;
        }
        taffy::style::FlexDirection::ColumnReverse => {
            style.flex_direction = taffy::style::FlexDirection::RowReverse;
        }
    }

    // 交换 size: width ↔ height
    std::mem::swap(&mut style.size.width, &mut style.size.height);

    // 交换 min_size: width ↔ height
    std::mem::swap(&mut style.min_size.width, &mut style.min_size.height);

    // 交换 max_size: width ↔ height
    std::mem::swap(&mut style.max_size.width, &mut style.max_size.height);

    // 交换 margin: left ↔ top, right ↔ bottom
    std::mem::swap(&mut style.margin.left, &mut style.margin.top);
    std::mem::swap(&mut style.margin.right, &mut style.margin.bottom);

    // 交换 padding: left ↔ top, right ↔ bottom
    std::mem::swap(&mut style.padding.left, &mut style.padding.top);
    std::mem::swap(&mut style.padding.right, &mut style.padding.bottom);

    // 交换 border: left ↔ top, right ↔ bottom
    std::mem::swap(&mut style.border.left, &mut style.border.top);
    std::mem::swap(&mut style.border.right, &mut style.border.bottom);

    // 交换 gap: column-gap ↔ row-gap
    // CSS Writing Modes §7.1：垂直书写模式中 gap 属性的轴也随主轴交换
    std::mem::swap(&mut style.gap.width, &mut style.gap.height);
}

/// 转换 display 属性。
fn convert_display(value: &DisplayValue) -> taffy::style::Display {
    match value {
        DisplayValue::Block => taffy::style::Display::Block,
        DisplayValue::Flex => taffy::style::Display::Flex,
        DisplayValue::InlineFlex => taffy::style::Display::Flex,
        DisplayValue::Grid => taffy::style::Display::Grid,
        DisplayValue::InlineGrid => taffy::style::Display::Grid,
        DisplayValue::None => taffy::style::Display::None,
        // table 相关类型：taffy 无原生 table 支持，映射为 Block 作为后备
        DisplayValue::Table
        | DisplayValue::InlineTable
        | DisplayValue::TableRow
        | DisplayValue::TableCell
        | DisplayValue::TableCaption
        | DisplayValue::TableColumn
        | DisplayValue::TableColumnGroup
        | DisplayValue::TableRowGroup
        | DisplayValue::TableHeaderGroup
        | DisplayValue::TableFooterGroup => taffy::style::Display::Block,
        // inline, inline-block, flow, flow-root, list-item, contents 都映射为 Block
        DisplayValue::Inline
        | DisplayValue::InlineBlock
        | DisplayValue::Flow
        | DisplayValue::FlowRoot
        | DisplayValue::ListItem
        | DisplayValue::Contents => taffy::style::Display::Block,
    }
}

/// 转换 position 属性。
///
/// - `Fixed` 映射为 `Absolute`：使元素脱离正常流，inset 相对于初始包含块（视口）。
///   后续由引擎后处理将坐标调整为视口相对。
/// - `Sticky` 映射为 `Relative`：taffy 无原生 sticky 支持，正常流布局，
///   由宿主层在滚动时动态调整偏移。
fn convert_position(value: &PositionValue) -> taffy::style::Position {
    match value {
        PositionValue::Absolute => taffy::style::Position::Absolute,
        // fixed 需要脱离正常流，使用 Absolute 让 taffy 应用 inset
        PositionValue::Fixed => taffy::style::Position::Absolute,
        // sticky 和 relative/static 一样参与正常流
        PositionValue::Sticky | PositionValue::Relative | PositionValue::Static => taffy::style::Position::Relative,
    }
}

/// 转换 overflow 属性。
fn convert_overflow(value: &OverflowValue) -> taffy::style::Overflow {
    match value {
        OverflowValue::Visible => taffy::style::Overflow::Visible,
        OverflowValue::Hidden => taffy::style::Overflow::Hidden,
        OverflowValue::Clip => taffy::style::Overflow::Clip,
        OverflowValue::Scroll | OverflowValue::Auto => taffy::style::Overflow::Scroll,
    }
}

/// 转换 box-sizing 属性。
fn convert_box_sizing(value: &BoxSizingValue) -> taffy::style::BoxSizing {
    match value {
        BoxSizingValue::ContentBox => taffy::style::BoxSizing::ContentBox,
        BoxSizingValue::BorderBox => taffy::style::BoxSizing::BorderBox,
    }
}

/// 转换 float 属性。
///
/// Taffy 0.7 不直接支持 float 布局，此函数将 FloatValue 映射为布尔值
/// 供布局引擎在构建布局树时判断元素是否需要浮动处理。
/// - `None` → 不浮动
/// - `Left` / `Right` / `InlineStart` / `InlineEnd` → 浮动
pub fn convert_float(value: &FloatValue) -> bool {
    !matches!(value, FloatValue::None)
}

/// 转换 clear 属性。
///
/// Taffy 0.7 不直接支持 clear 布局，此函数将 ClearValue 映射为布尔值
/// 供布局引擎在构建布局树时判断元素是否需要清除浮动。
/// - `None` → 不清除
/// - `Left` / `Right` / `Both` / `InlineStart` / `InlineEnd` → 清除
pub fn convert_clear(value: &ClearValue) -> bool {
    !matches!(value, ClearValue::None)
}

/// 解析视口相对单位（vw/vh/vmin/vmax）为像素值。
///
/// - `1vw` = 视口宽度的 1%（`vw / 100 * viewport_w`）
/// - `1vh` = 视口高度的 1%（`vh / 100 * viewport_h`）
/// - `1vmin` = min(vw, vh) 的 1%
/// - `1vmax` = max(vw, vh) 的 1%
///
/// 非 viewport 单位返回 None。
fn resolve_viewport_px(value: &LengthValue, vw: f32, vh: f32) -> Option<f32> {
    let vmin = vw.min(vh);
    let vmax = vw.max(vh);
    match value {
        LengthValue::Vw(v) => Some(*v as f32 * vw / 100.0),
        LengthValue::Vh(v) => Some(*v as f32 * vh / 100.0),
        LengthValue::Vmin(v) => Some(*v as f32 * vmin / 100.0),
        LengthValue::Vmax(v) => Some(*v as f32 * vmax / 100.0),
        _ => None,
    }
}

/// 将 LengthValue 转换为 taffy 的 Dimension。
///
/// em/rem 单位已由 style-system 解析为 px，所以统一用 Length。
/// Auto 映射为 Auto，Percentage 映射为 Percent。
fn convert_length_to_dimension(value: &LengthValue, vw: f32, vh: f32) -> taffy::style::Dimension {
    if let Some(px) = resolve_viewport_px(value, vw, vh) {
        return length(px);
    }
    match value {
        LengthValue::Px(v) => length(*v as f32),
        LengthValue::Em(v) => length(*v as f32),
        LengthValue::Rem(v) => length(*v as f32),
        LengthValue::Ch(v) => length(*v as f32),
        LengthValue::Percentage(v) => taffy::style::Dimension::percent((*v / 100.0) as f32),
        LengthValue::Auto => taffy::style::Dimension::auto(),
        // Calc 表达式：尝试提取简单的 P% ± Npx 模式，转为百分比。
        // calc(100% - 6px) → Percent(1.0)。精确的 px 偏移量在布局后处理中处理。
        LengthValue::Calc(expr) => {
            if let Some(pct) = extract_calc_percentage(expr) {
                taffy::style::Dimension::percent(pct as f32 / 100.0)
            } else {
                length(0.0_f32)
            }
        }
        // fit-content() 将内部值转换为 dimension
        LengthValue::FitContent(inner) => convert_length_to_dimension(inner, vw, vh),
        // min-content/max-content：塌缩为 0（与旧「resolve 为 Px(0)」行为中性），
        // 由 layout-engine 两趟固有宽度测量在可测时把容器宽度提升到 intrinsic。
        // 不能映射为 Auto——taffy 会把 width:auto 的块级容器拉伸到可用宽度（填充），
        // 违反 max-content/min-content 的 shrink-to-fit 语义（R181c 实测 net -5）。
        LengthValue::MinContent | LengthValue::MaxContent => length(0.0_f32),
        // viewport 单位已在上方 resolve_viewport_px 处理
        _ => taffy::style::Dimension::auto(),
    }
}

/// 将 max-width/max-height 的 LengthValue 转换为 Dimension。
///
/// max-width/max-height 默认值为 INFINITY，映射为 Auto。
fn convert_max_length_to_dimension(value: &LengthValue, vw: f32, vh: f32) -> taffy::style::Dimension {
    if let Some(px) = resolve_viewport_px(value, vw, vh) {
        return length(px);
    }
    match value {
        LengthValue::Px(v) => {
            let v = *v as f32;
            if v.is_infinite() {
                taffy::style::Dimension::auto()
            } else {
                length(v)
            }
        }
        LengthValue::Em(v) => length(*v as f32),
        LengthValue::Rem(v) => length(*v as f32),
        LengthValue::Ch(v) => length(*v as f32),
        LengthValue::Percentage(v) => taffy::style::Dimension::percent((*v / 100.0) as f32),
        LengthValue::Auto => taffy::style::Dimension::auto(),
        // Calc 表达式：提取百分比部分（与 convert_length_to_dimension 一致），
        // 非百分比 calc 回退 0.0。此前 calc() 被静默丢弃为 0.0（max-width/max-height 失效）。
        LengthValue::Calc(expr) => {
            if let Some(pct) = extract_calc_percentage(expr) {
                taffy::style::Dimension::percent(pct as f32 / 100.0)
            } else {
                length(0.0_f32)
            }
        }
        LengthValue::FitContent(inner) => convert_max_length_to_dimension(inner, vw, vh),
        LengthValue::MinContent | LengthValue::MaxContent => taffy::style::Dimension::auto(),
        _ => taffy::style::Dimension::auto(),
    }
}

/// 将 LengthValue 转换为 taffy 的 LengthPercentage。
///
/// 用于 padding、border、gap 等不接受 auto 的属性。
fn convert_length_to_lp(value: &LengthValue, vw: f32, vh: f32) -> taffy::style::LengthPercentage {
    if let Some(px) = resolve_viewport_px(value, vw, vh) {
        return length(px);
    }
    match value {
        LengthValue::Px(v) => length(*v as f32),
        LengthValue::Em(v) => length(*v as f32),
        LengthValue::Rem(v) => length(*v as f32),
        LengthValue::Ch(v) => length(*v as f32),
        LengthValue::Percentage(v) => taffy::style::LengthPercentage::percent((*v / 100.0) as f32),
        LengthValue::Auto => length(0.0_f32), // 不接受 auto 的属性，auto 视为 0
        // Calc 表达式：提取百分比部分（与 convert_length_to_dimension 一致），
        // 非百分比 calc 回退 0.0。此前 calc() 被静默丢弃为 0.0（padding/border/gap 失效）。
        LengthValue::Calc(expr) => {
            if let Some(pct) = extract_calc_percentage(expr) {
                taffy::style::LengthPercentage::percent(pct as f32 / 100.0)
            } else {
                length(0.0_f32)
            }
        }
        LengthValue::FitContent(inner) => convert_length_to_lp(inner, vw, vh),
        LengthValue::MinContent | LengthValue::MaxContent => length(0.0_f32),
        _ => length(0.0_f32),
    }
}

/// 将 LengthValue 转换为 taffy 的 LengthPercentageAuto。
///
/// 用于 margin、inset 等接受 auto 的属性。
/// `resolve_auto_as_zero` 为 true 时，将 Auto 解析为 0（用于浮动元素的左右 margin）。
fn convert_length_to_lpa(
    value: &LengthValue,
    resolve_auto_as_zero: bool,
    vw: f32,
    vh: f32,
) -> taffy::style::LengthPercentageAuto {
    if let Some(px) = resolve_viewport_px(value, vw, vh) {
        return length(px);
    }
    match value {
        LengthValue::Px(v) => length(*v as f32),
        LengthValue::Em(v) => length(*v as f32),
        LengthValue::Rem(v) => length(*v as f32),
        LengthValue::Ch(v) => length(*v as f32),
        LengthValue::Percentage(v) => taffy::style::LengthPercentageAuto::percent((*v / 100.0) as f32),
        LengthValue::Auto => {
            if resolve_auto_as_zero {
                length(0.0_f32)
            } else {
                taffy::style::LengthPercentageAuto::auto()
            }
        }
        // Calc 表达式：提取 P% ± Npx 的百分比部分（与 convert_length_to_dimension 一致）。
        // calc(50% - 0px) → Percent(0.5)；px 偏移量由布局后处理（同 dimension 路径注释）。
        // 此前 margin/inset 的 calc() 被静默丢弃为 0.0（grid-calc-margin 等用例）。
        LengthValue::Calc(expr) => {
            if let Some(pct) = extract_calc_percentage(expr) {
                taffy::style::LengthPercentageAuto::percent(pct as f32 / 100.0)
            } else {
                length(0.0_f32)
            }
        }
        LengthValue::FitContent(inner) => convert_length_to_lpa(inner, resolve_auto_as_zero, vw, vh),
        LengthValue::MinContent | LengthValue::MaxContent => length(0.0_f32),
        _ => length(0.0_f32),
    }
}

/// 转换 flex-direction 属性。
fn convert_flex_direction(value: &FlexDirectionValue) -> taffy::style::FlexDirection {
    match value {
        FlexDirectionValue::Row => taffy::style::FlexDirection::Row,
        FlexDirectionValue::RowReverse => taffy::style::FlexDirection::RowReverse,
        FlexDirectionValue::Column => taffy::style::FlexDirection::Column,
        FlexDirectionValue::ColumnReverse => taffy::style::FlexDirection::ColumnReverse,
    }
}

/// 转换 flex-wrap 属性。
fn convert_flex_wrap(value: &FlexWrapValue) -> taffy::style::FlexWrap {
    match value {
        FlexWrapValue::Nowrap => taffy::style::FlexWrap::NoWrap,
        FlexWrapValue::Wrap => taffy::style::FlexWrap::Wrap,
        FlexWrapValue::WrapReverse => taffy::style::FlexWrap::WrapReverse,
    }
}

/// 转换 flex-basis 属性。
fn convert_flex_basis(value: &FlexBasisValue, vw: f32, vh: f32) -> taffy::style::Dimension {
    match value {
        FlexBasisValue::Auto => taffy::style::Dimension::auto(),
        FlexBasisValue::Content => taffy::style::Dimension::auto(), // taffy 无 content，映射为 Auto
        FlexBasisValue::Length(lv) => convert_length_to_dimension(lv, vw, vh),
    }
}

/// 转换 AlignmentValue 到 taffy AlignItems。
fn convert_alignment_to_align_items(value: &AlignmentValue) -> Option<taffy::style::AlignItems> {
    match value {
        AlignmentValue::Auto => None, // align-items 不使用 auto
        AlignmentValue::FlexStart => Some(taffy::style::AlignItems::FLEX_START),
        AlignmentValue::FlexEnd => Some(taffy::style::AlignItems::FLEX_END),
        AlignmentValue::Center => Some(taffy::style::AlignItems::CENTER),
        AlignmentValue::Stretch => Some(taffy::style::AlignItems::STRETCH),
        AlignmentValue::Baseline => Some(taffy::style::AlignItems::BASELINE),
        AlignmentValue::Start => Some(taffy::style::AlignItems::START),
        AlignmentValue::End => Some(taffy::style::AlignItems::END),
        // space-between, space-around, space-evenly 不适用于 align-items
        AlignmentValue::SpaceBetween | AlignmentValue::SpaceAround | AlignmentValue::SpaceEvenly => None,
    }
}

/// 转换 AlignmentValue 到 taffy AlignSelf。
fn convert_alignment_to_align_self(value: &AlignmentValue) -> Option<taffy::style::AlignSelf> {
    // AlignSelf 是 AlignItems 的 type alias
    match value {
        AlignmentValue::Auto => None, // 继承容器 align-items
        AlignmentValue::FlexStart => Some(taffy::style::AlignSelf::FLEX_START),
        AlignmentValue::FlexEnd => Some(taffy::style::AlignSelf::FLEX_END),
        AlignmentValue::Center => Some(taffy::style::AlignSelf::CENTER),
        AlignmentValue::Stretch => Some(taffy::style::AlignSelf::STRETCH),
        AlignmentValue::Baseline => Some(taffy::style::AlignSelf::BASELINE),
        AlignmentValue::Start => Some(taffy::style::AlignSelf::START),
        AlignmentValue::End => Some(taffy::style::AlignSelf::END),
        AlignmentValue::SpaceBetween | AlignmentValue::SpaceAround | AlignmentValue::SpaceEvenly => None,
    }
}

/// 转换 AlignmentValue 到 taffy JustifyContent。
fn convert_alignment_to_justify_content(value: &AlignmentValue) -> Option<taffy::style::JustifyContent> {
    match value {
        AlignmentValue::Auto => None, // auto 不适用于 justify-content
        AlignmentValue::FlexStart => Some(taffy::style::JustifyContent::FLEX_START),
        AlignmentValue::FlexEnd => Some(taffy::style::JustifyContent::FLEX_END),
        AlignmentValue::Center => Some(taffy::style::JustifyContent::CENTER),
        AlignmentValue::SpaceBetween => Some(taffy::style::JustifyContent::SPACE_BETWEEN),
        AlignmentValue::SpaceAround => Some(taffy::style::JustifyContent::SPACE_AROUND),
        AlignmentValue::SpaceEvenly => Some(taffy::style::JustifyContent::SPACE_EVENLY),
        AlignmentValue::Start => Some(taffy::style::JustifyContent::START),
        AlignmentValue::End => Some(taffy::style::JustifyContent::END),
        AlignmentValue::Stretch => Some(taffy::style::JustifyContent::STRETCH),
        AlignmentValue::Baseline => None, // baseline 不适用于 justify-content
    }
}

/// `justify-content` 转换 + CSS Grid 默认值修正。
///
/// CSS Box Alignment §8.5：justify-content 的初始值 `normal` 在 **grid 容器**上行为等同
/// `stretch`（css-grid-2 §2.2 隐式 track 填充容器）；在 flex 容器上等同 `flex-start`。
/// ZW 的 `AlignmentValue` 不建模 `normal`，`default_impl` 用 `FlexStart` 作 `normal` 的
/// 代理（对 flex 正确）。但 `FlexStart` 经 `convert_alignment_to_justify_content` 映射到
/// `Some(FLEX_START)`，对 grid 容器**丢失了 stretch 语义** → 定宽 grid 容器中
/// max-content=0 的隐式 auto 列不吸收剩余空间 → 空 grid item 宽度=0、背景不绘制
/// （grid-calc-margin 实证：ZW 全白 0px vs chromium 20000px）。
///
/// 此处对 grid 容器把默认代理 `FlexStart`（= normal）改写为 `STRETCH`。仅
/// `display:Grid/InlineGrid` 生效；flex 默认 `flex-start` 不受影响（css-align §8.5 对 flex
/// 的 normal = flex-start，FlexStart 已正确）。作者显式声明的 justify-content
/// （start/end/center/space-*）不被覆盖。全 css-grid corpus 0 案显式 justify-content
/// （grep 实证），故 FlexStart→STRETCH 对 grid 无回归风险。
///
/// kill-switch `ZW_GRID_JUSTIFY_STRETCH=0`（default-on）。
fn grid_justify_content(value: &AlignmentValue, display: &DisplayValue) -> Option<taffy::style::JustifyContent> {
    let jc = convert_alignment_to_justify_content(value);
    if matches!(value, AlignmentValue::FlexStart)
        && matches!(display, DisplayValue::Grid | DisplayValue::InlineGrid)
        && std::env::var("ZW_GRID_JUSTIFY_STRETCH").as_deref() != Ok("0")
    {
        Some(taffy::style::JustifyContent::STRETCH)
    } else {
        jc
    }
}

/// 转换 AlignContentValue 到 taffy AlignContent。
///
/// 与 `convert_alignment_to_align_content` 类似，但接受 `AlignContentValue`
/// （CSS align-content 计算值类型，包含 Auto/Normal）。
fn convert_align_content(value: &AlignContentValue) -> Option<taffy::style::AlignContent> {
    match value {
        AlignContentValue::Auto => None,
        AlignContentValue::Normal => None,
        AlignContentValue::Start => Some(taffy::style::AlignContent::START),
        AlignContentValue::End => Some(taffy::style::AlignContent::END),
        AlignContentValue::Center => Some(taffy::style::AlignContent::CENTER),
        AlignContentValue::Stretch => Some(taffy::style::AlignContent::STRETCH),
        AlignContentValue::Baseline => None,
        AlignContentValue::SpaceBetween => Some(taffy::style::AlignContent::SPACE_BETWEEN),
        AlignContentValue::SpaceAround => Some(taffy::style::AlignContent::SPACE_AROUND),
        AlignContentValue::SpaceEvenly => Some(taffy::style::AlignContent::SPACE_EVENLY),
    }
}

/// 转换 JustifyItemsValue 到 taffy AlignItems。
///
/// taffy 的 justify_items 字段使用 Option<AlignItems> 类型。
/// Auto/Normal 表示使用默认行为。
fn convert_justify_items(value: &JustifyItemsValue) -> Option<taffy::style::AlignItems> {
    match value {
        JustifyItemsValue::Auto => None,
        JustifyItemsValue::Normal => None,
        JustifyItemsValue::Start => Some(taffy::style::AlignItems::START),
        JustifyItemsValue::End => Some(taffy::style::AlignItems::END),
        JustifyItemsValue::Center => Some(taffy::style::AlignItems::CENTER),
        JustifyItemsValue::Stretch => Some(taffy::style::AlignItems::STRETCH),
        JustifyItemsValue::Baseline => Some(taffy::style::AlignItems::BASELINE),
    }
}

/// 转换 JustifySelfValue 到 taffy AlignSelf。
///
/// taffy 的 justify_self 字段使用 Option<AlignSelf> 类型。
/// Auto/Normal 表示使用默认行为。
fn convert_justify_self(value: &JustifySelfValue) -> Option<taffy::style::AlignSelf> {
    match value {
        JustifySelfValue::Auto => None,
        JustifySelfValue::Normal => None,
        JustifySelfValue::Start => Some(taffy::style::AlignSelf::START),
        JustifySelfValue::End => Some(taffy::style::AlignSelf::END),
        JustifySelfValue::Center => Some(taffy::style::AlignSelf::CENTER),
        JustifySelfValue::Stretch => Some(taffy::style::AlignSelf::STRETCH),
        JustifySelfValue::Baseline => Some(taffy::style::AlignSelf::BASELINE),
    }
}

/// 解析 CSS grid track 定义字符串为 taffy TrackSizingFunction 列表。
///
/// 支持的值格式：
/// - `100px` — 固定长度
/// - `1fr` — 弹性轨道
/// - `auto` — 自动轨道
/// - `50%` — 百分比
/// - `minmax(100px, 1fr)` — 最小最大
/// - `repeat(3, 100px)` — 重复
/// - `repeat(auto-fill, 200px)` — 自动填充（传递给 taffy 原生 auto-fill）
fn parse_grid_tracks(value: &Option<String>) -> Vec<taffy::style::GridTemplateComponent<String>> {
    let Some(value) = value else {
        return vec![];
    };

    let tokens = tokenize_track_list(value);
    let mut result = Vec::new();

    for token in tokens {
        if let Some(inner) = token.strip_prefix("repeat(").and_then(|s| s.strip_suffix(')')) {
            result.extend(parse_repeat(inner));
        } else {
            result.push(parse_single_track(&token));
        }
    }

    result
}

/// 将 grid track 列表字符串拆分为独立的 token。
///
/// 与 `split_whitespace` 不同，此函数会识别括号边界，
/// 将 `repeat(...)` 和 `minmax(...)` 保持为单个 token。
fn tokenize_track_list(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth: u32 = 0;

    for ch in value.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ' ' | '\t' if depth == 0 => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// 解析 repeat() 函数内部内容为 track sizing function 列表。
///
/// 格式：`3, 100px` 或 `auto-fill, 200px` 或 `2, 1fr auto`。
///
/// 对于 auto-fill/auto-fit，生成 `TrackSizingFunction::Repeat` 变体，
/// 利用 taffy 原生的 auto-fill 支持，根据容器宽度自动计算轨道数量。
/// 对于固定次数，直接展开为对应数量的轨道。
fn parse_repeat(inner: &str) -> Vec<taffy::style::GridTemplateComponent<String>> {
    use taffy::style::{GridTemplateComponent, GridTemplateRepetition, RepetitionCount, TrackSizingFunction};

    // 找到第一个不在括号内的逗号
    let comma_pos = find_top_level_comma(inner);
    let Some(comma_pos) = comma_pos else {
        return vec![TrackSizingFunction::AUTO.into()];
    };

    let count_str = inner[..comma_pos].trim();
    let track_list_str = inner[comma_pos + 1..].trim();

    // 解析内部 track 列表为 TrackSizingFunction（非重复型，0.9.2 = MinMax）
    let inner_tokens = tokenize_track_list(track_list_str);
    let inner_tracks: Vec<TrackSizingFunction> = inner_tokens
        .iter()
        .map(|t| parse_single_track_as_non_repeated(t))
        .collect();

    if count_str.eq_ignore_ascii_case("auto-fill") {
        // 传递给 taffy 原生 auto-fill，自动根据容器宽度计算轨道数量
        return vec![GridTemplateComponent::Repeat(GridTemplateRepetition {
            count: RepetitionCount::AutoFill,
            tracks: inner_tracks,
            line_names: vec![],
        })];
    }

    if count_str.eq_ignore_ascii_case("auto-fit") {
        // 传递给 taffy 原生 auto-fit
        return vec![GridTemplateComponent::Repeat(GridTemplateRepetition {
            count: RepetitionCount::AutoFit,
            tracks: inner_tracks,
            line_names: vec![],
        })];
    }

    // 固定次数：展开为对应数量的轨道
    let Ok(count) = count_str.parse::<usize>() else {
        return vec![TrackSizingFunction::AUTO.into()];
    };

    let mut result = Vec::with_capacity(count * inner_tracks.len());
    for _ in 0..count {
        result.extend(inner_tracks.iter().map(|t| GridTemplateComponent::Single(*t)));
    }

    result
}

/// 将单个 track 值解析为 TrackSizingFunction。
///
/// 用于 repeat() 内部轨道列表的解析。
fn parse_single_track_as_non_repeated(s: &str) -> taffy::style::TrackSizingFunction {
    use taffy::style::TrackSizingFunction;

    let s = s.trim();

    if s.eq_ignore_ascii_case("auto") {
        return TrackSizingFunction::AUTO;
    }
    if s.ends_with("fr")
        && let Ok(flex) = s.trim_end_matches("fr").parse::<f32>()
    {
        return TrackSizingFunction::from_fr(flex);
    }
    if s.ends_with('%')
        && let Ok(pct) = s.trim_end_matches('%').parse::<f32>()
    {
        return TrackSizingFunction::from_percent(pct / 100.0);
    }
    if s.starts_with("minmax(") && s.ends_with(')') {
        return parse_minmax_as_non_repeated(&s[7..s.len() - 1]);
    }
    // fit-content() 函数
    if s.starts_with("fit-content(") && s.ends_with(')') {
        let inner = &s["fit-content(".len()..s.len() - 1];
        if let Some((val, is_pct)) = parse_length_percentage(inner.trim()) {
            return TrackSizingFunction {
                min: taffy::style::MinTrackSizingFunction::auto(),
                max: if is_pct {
                    taffy::style::MaxTrackSizingFunction::fit_content_percent(val)
                } else {
                    taffy::style::MaxTrackSizingFunction::fit_content_px(val)
                },
            };
        }
    }
    if s.ends_with("px")
        && let Ok(px) = s.trim_end_matches("px").parse::<f32>()
    {
        return TrackSizingFunction::from_length(px);
    }
    if let Ok(px) = s.parse::<f32>() {
        return TrackSizingFunction::from_length(px);
    }

    TrackSizingFunction::AUTO
}

/// 找到字符串中第一个不在括号内的逗号位置。
fn find_top_level_comma(s: &str) -> Option<usize> {
    let mut depth: u32 = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// 解析 grid-auto-rows/columns 的 track 定义为 TrackSizingFunction 列表。
///
/// 与 parse_grid_tracks 类似，但返回 TrackSizingFunction
/// （不包含 repeat 变体），用于 taffy 的 grid_auto_rows/grid_auto_columns 字段。
fn parse_grid_auto_tracks(value: &Option<String>) -> Vec<taffy::style::TrackSizingFunction> {
    let Some(value) = value else {
        return vec![];
    };

    value
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(parse_single_auto_track)
        .collect()
}

/// 解析单个 TrackSizingFunction 值。
fn parse_single_auto_track(s: &str) -> taffy::style::TrackSizingFunction {
    use taffy::style::TrackSizingFunction;

    let s = s.trim();

    if s.eq_ignore_ascii_case("auto") {
        return TrackSizingFunction::AUTO;
    }
    if s.ends_with("fr")
        && let Ok(flex) = s.trim_end_matches("fr").parse::<f32>()
    {
        return TrackSizingFunction::from_fr(flex);
    }
    if s.ends_with('%')
        && let Ok(pct) = s.trim_end_matches('%').parse::<f32>()
    {
        return TrackSizingFunction::from_percent(pct / 100.0);
    }
    if s.starts_with("minmax(") && s.ends_with(')') {
        return parse_minmax_as_non_repeated(&s[7..s.len() - 1]);
    }
    if s.ends_with("px")
        && let Ok(px) = s.trim_end_matches("px").parse::<f32>()
    {
        return TrackSizingFunction::from_length(px);
    }
    if let Ok(px) = s.parse::<f32>() {
        return TrackSizingFunction::from_length(px);
    }

    TrackSizingFunction::AUTO
}

/// 解析 minmax() 函数内部，返回 TrackSizingFunction。
fn parse_minmax_as_non_repeated(inner: &str) -> taffy::style::TrackSizingFunction {
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 2 {
        return taffy::style::TrackSizingFunction::AUTO;
    }

    let min = parse_min_track(parts[0].trim());
    let max = parse_max_track(parts[1].trim());

    taffy::geometry::MinMax { min, max }
}

/// 解析单个 grid track 值。
fn parse_single_track(s: &str) -> taffy::style::GridTemplateComponent<String> {
    use taffy::style::TrackSizingFunction;

    let s = s.trim();

    if s.eq_ignore_ascii_case("auto") {
        return TrackSizingFunction::AUTO.into();
    }
    if s.ends_with("fr")
        && let Ok(flex) = s.trim_end_matches("fr").parse::<f32>()
    {
        return TrackSizingFunction::from_fr(flex).into();
    }
    if s.ends_with('%')
        && let Ok(pct) = s.trim_end_matches('%').parse::<f32>()
    {
        return TrackSizingFunction::from_percent(pct / 100.0).into();
    }
    if s.starts_with("minmax(") && s.ends_with(')') {
        return parse_minmax(&s[7..s.len() - 1]).into();
    }
    // fit-content() 函数：映射为 taffy 的 FitContent 轨道尺寸
    if s.starts_with("fit-content(") && s.ends_with(')') {
        let inner = &s["fit-content(".len()..s.len() - 1];
        if let Some((val, is_pct)) = parse_length_percentage(inner.trim()) {
            return taffy::style::GridTemplateComponent::Single(taffy::geometry::MinMax {
                min: taffy::style::MinTrackSizingFunction::auto(),
                max: if is_pct {
                    taffy::style::MaxTrackSizingFunction::fit_content_percent(val)
                } else {
                    taffy::style::MaxTrackSizingFunction::fit_content_px(val)
                },
            });
        }
    }
    // 默认尝试解析为长度
    if s.ends_with("px")
        && let Ok(px) = s.trim_end_matches("px").parse::<f32>()
    {
        return TrackSizingFunction::from_length(px).into();
    }
    if let Ok(px) = s.parse::<f32>() {
        return TrackSizingFunction::from_length(px).into();
    }

    // 无法解析，默认 auto
    TrackSizingFunction::AUTO.into()
}

/// 解析长度或百分比，返回 (值, 是否百分比)。
///
/// 支持 px、%、纯数字（视为 px）。供 fit-content 轨道尺寸用（0.8.3 fit_content_px/percent 取 f32）。
fn parse_length_percentage(s: &str) -> Option<(f32, bool)> {
    let s = s.trim();
    if s.ends_with("px")
        && let Ok(px) = s.trim_end_matches("px").parse::<f32>()
    {
        return Some((px, false));
    }
    if s.ends_with('%')
        && let Ok(pct) = s.trim_end_matches('%').parse::<f32>()
    {
        return Some((pct / 100.0, true));
    }
    if let Ok(px) = s.parse::<f32>() {
        return Some((px, false));
    }
    None
}

fn parse_minmax(inner: &str) -> taffy::style::TrackSizingFunction {
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 2 {
        return taffy::style::TrackSizingFunction::AUTO;
    }

    let min = parse_min_track(parts[0].trim());
    let max = parse_max_track(parts[1].trim());

    taffy::geometry::MinMax { min, max }
}

/// 解析 minmax 的最小值。
///
/// 支持 auto、px、百分比（%）和纯数字。
fn parse_min_track(s: &str) -> taffy::style::MinTrackSizingFunction {
    use taffy::style::MinTrackSizingFunction;

    if s.eq_ignore_ascii_case("auto") {
        return MinTrackSizingFunction::auto();
    }
    if s.ends_with('%')
        && let Ok(pct) = s.trim_end_matches('%').parse::<f32>()
    {
        return MinTrackSizingFunction::percent(pct / 100.0);
    }
    if s.ends_with("px")
        && let Ok(px) = s.trim_end_matches("px").parse::<f32>()
    {
        return MinTrackSizingFunction::length(px);
    }
    if let Ok(px) = s.parse::<f32>() {
        return MinTrackSizingFunction::length(px);
    }

    MinTrackSizingFunction::auto()
}

/// 解析 minmax 的最大值。
///
/// 支持 auto、fr、px、百分比（%）和纯数字。
fn parse_max_track(s: &str) -> taffy::style::MaxTrackSizingFunction {
    use taffy::style::MaxTrackSizingFunction;

    if s.eq_ignore_ascii_case("auto") {
        return MaxTrackSizingFunction::auto();
    }
    if s.ends_with("fr")
        && let Ok(flex) = s.trim_end_matches("fr").parse::<f32>()
    {
        return MaxTrackSizingFunction::from_fr(flex);
    }
    if s.ends_with('%')
        && let Ok(pct) = s.trim_end_matches('%').parse::<f32>()
    {
        return MaxTrackSizingFunction::percent(pct / 100.0);
    }
    if s.ends_with("px")
        && let Ok(px) = s.trim_end_matches("px").parse::<f32>()
    {
        return MaxTrackSizingFunction::length(px);
    }
    if let Ok(px) = s.parse::<f32>() {
        return MaxTrackSizingFunction::length(px);
    }

    MaxTrackSizingFunction::auto()
}

/// 转换 grid-auto-flow 值。
fn convert_grid_auto_flow(value: &GridAutoFlowValue) -> taffy::style::GridAutoFlow {
    match value {
        GridAutoFlowValue::Row => taffy::style::GridAutoFlow::Row,
        GridAutoFlowValue::Column => taffy::style::GridAutoFlow::Column,
        GridAutoFlowValue::RowDense => taffy::style::GridAutoFlow::RowDense,
        GridAutoFlowValue::ColumnDense => taffy::style::GridAutoFlow::ColumnDense,
    }
}

/// 转换 GridLineValue 到 taffy GridPlacement。
///
/// Name 变体应已由 resolve_named_area 预处理为 Line，
/// 若仍有 Name 则 fallback 到 Auto。
fn convert_grid_line(value: &GridLineValue) -> taffy::style::GridPlacement {
    match value {
        GridLineValue::Auto => taffy::style::GridPlacement::Auto,
        GridLineValue::Line(n) => taffy::style::GridPlacement::from_line_index(*n),
        GridLineValue::Span(s) => taffy::style::GridPlacement::from_span(*s),
        GridLineValue::Name(_) => taffy::style::GridPlacement::Auto,
    }
}

/// 解析 grid-template-areas CSS 字符串为区域映射。
///
/// 输入格式：'"header header" "sidebar main" "sidebar footer"'
/// 返回：HashMap<区域名, (row_start, row_end, col_start, col_end)>
///
/// 行号和列号均为 1-based。区域占据的行/列为 [start, end)，
/// 即 row_end = row_start + span_rows。
///
/// 验证规则：
/// 1. 所有行的列数必须相同（矩形检查）
/// 2. 每个命名区域必须构成一个矩形（非矩形区域会记录警告并忽略）
pub fn parse_grid_template_areas(value: &str) -> GridAreaMap {
    let mut areas = std::collections::HashMap::new();
    let mut row = 1i16;
    // 收集每行的 token 列表，用于后续矩形校验
    let mut rows_tokens: Vec<Vec<String>> = Vec::new();

    // 按引号对分割出每行
    let mut chars = value.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch == '"' {
            chars.next(); // 消费开引号
            let mut line = String::new();
            while let Some(&c) = chars.peek() {
                if c == '"' {
                    chars.next(); // 消费闭引号
                    break;
                }
                line.push(c);
                chars.next();
            }

            // 解析行内 token
            let tokens: Vec<&str> = line.split_whitespace().collect();
            rows_tokens.push(tokens.iter().map(|s| s.to_string()).collect());

            for (col_idx, &token) in tokens.iter().enumerate() {
                // "." 表示空单元格，跳过（RFC 6265）
                if token == "." {
                    continue;
                }
                let col = (col_idx + 1) as i16;

                if let Some(entry) = areas.get_mut(token) {
                    // 扩展现有区域的 row_end 和 col_end
                    let (_, ref mut re, _, ref mut ce) = *entry;
                    if row + 1 > *re {
                        *re = row + 1;
                    }
                    if col + 1 > *ce {
                        *ce = col + 1;
                    }
                } else {
                    areas.insert(token.to_string(), (row, row + 1, col, col + 1));
                }
            }

            row += 1;
        } else {
            chars.next();
        }
    }

    // ── 矩形校验 ──

    // 1. 检查所有行的列数是否一致
    if rows_tokens.len() > 1 {
        let expected_cols = rows_tokens[0].len();
        for (i, tokens) in rows_tokens.iter().enumerate() {
            if tokens.len() != expected_cols {
                tracing::warn!(
                    "grid-template-areas: row {} has {} columns but expected {}, area map may be incorrect",
                    i + 1,
                    tokens.len(),
                    expected_cols
                );
                return areas;
            }
        }
    }

    // 2. 检查每个命名区域是否构成矩形
    //    对每个区域名，记录它在 grid 中出现的所有 (row, col)，
    //    然后验证这些位置是否构成一个完整的矩形。
    if !rows_tokens.is_empty() {
        let num_rows = rows_tokens.len() as i16;
        let num_cols = rows_tokens[0].len() as i16;

        for (name, &(rs, re, cs, ce)) in &areas {
            // 计算预期占据的单元格数
            let expected_count = ((re - rs) * (ce - cs)) as usize;
            // 统计实际出现次数
            let mut actual_count = 0usize;
            for (r, tokens) in rows_tokens.iter().enumerate() {
                let r1 = (r + 1) as i16;
                if r1 < rs || r1 >= re {
                    continue;
                }
                for (c, token) in tokens.iter().enumerate() {
                    let c1 = (c + 1) as i16;
                    if c1 < cs || c1 >= ce {
                        continue;
                    }
                    if token == name {
                        actual_count += 1;
                    }
                }
            }
            if actual_count != expected_count {
                tracing::warn!(
                    "grid-template-areas: area '{}' does not form a rectangle (expected {} cells, found {}), \
                     bounds=({},{},{},{}), grid_size={}x{}",
                    name,
                    expected_count,
                    actual_count,
                    rs,
                    re,
                    cs,
                    ce,
                    num_rows,
                    num_cols
                );
            }
        }
    }

    areas
}

/// 解析子元素的命名区域引用为具体的行号。
///
/// 当子元素的 grid-row-start/end 或 grid-column-start/end 为 Name 时，
/// 查找父级区域映射，将 Name 替换为 Line（区域边界）。
/// `which` 为 "row-start"、"row-end"、"col-start"、"col-end" 之一。
fn resolve_named_area(value: &GridLineValue, parent_areas: Option<&GridAreaMap>, which: &str) -> GridLineValue {
    match value {
        GridLineValue::Name(name) => {
            if let Some(areas) = parent_areas {
                if let Some(&(rs, re, cs, ce)) = areas.get(name) {
                    match which {
                        "row-start" => GridLineValue::Line(rs),
                        "row-end" => GridLineValue::Line(re),
                        "col-start" => GridLineValue::Line(cs),
                        "col-end" => GridLineValue::Line(ce),
                        _ => GridLineValue::Auto,
                    }
                } else {
                    GridLineValue::Auto
                }
            } else {
                GridLineValue::Auto
            }
        }
        other => other.clone(),
    }
}

/// 预处理子元素的 grid line 值，将 Name 引用解析为具体行号。
///
/// 返回解析后的 (row_start, row_end, col_start, col_end)。
pub fn resolve_grid_placement(
    style: &ComputedStyle,
    parent_areas: Option<&GridAreaMap>,
) -> (GridLineValue, GridLineValue, GridLineValue, GridLineValue) {
    let rs = resolve_named_area(&style.grid_row_start, parent_areas, "row-start");
    let re = resolve_named_area(&style.grid_row_end, parent_areas, "row-end");
    let cs = resolve_named_area(&style.grid_column_start, parent_areas, "col-start");
    let ce = resolve_named_area(&style.grid_column_end, parent_areas, "col-end");
    (rs, re, cs, ce)
}

/// 尝试从 calc 表达式中提取百分比值。
///
/// 对于 `calc(100% - 6px)` 这样的简单模式，提取出 `100.0`。
/// 这使得 taffy 能使用百分比进行布局。
/// 仅支持 `P% - Npx`、`P% + Npx`、`Npx - P%`、纯 `P%` 模式。
fn extract_calc_percentage(expr: &zero_css_parser::values::CalcExpr) -> Option<f64> {
    use zero_css_parser::values::{CalcExpr, CalcOp, LengthValue};
    match expr {
        CalcExpr::Length(LengthValue::Percentage(pct)) => Some(*pct),
        CalcExpr::BinaryOp(left, op, right) => {
            let left_pct = match left.as_ref() {
                CalcExpr::Length(LengthValue::Percentage(pct)) => Some(*pct),
                _ => None,
            };
            let right_pct = match right.as_ref() {
                CalcExpr::Length(LengthValue::Percentage(pct)) => Some(*pct),
                _ => None,
            };
            match (op, left_pct, right_pct) {
                (CalcOp::Add, Some(lp), None) => Some(lp),
                (CalcOp::Add, None, Some(rp)) => Some(rp),
                (CalcOp::Subtract, Some(lp), None) => Some(lp),
                (CalcOp::Subtract, None, Some(rp)) => Some(-rp),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
#[allow(unknown_lints)] // R1704: float_literal_f32_fallback 仅 rustc 1.96+ 存在；旧 toolchain 用 unknown_lints 抑制
#[allow(float_literal_f32_fallback)]
mod tests;

#[cfg(test)]
#[allow(unknown_lints)] // R1704: 同上
#[allow(float_literal_f32_fallback)]
mod inline_tests {
    use super::*;
    use zero_css_parser::values::{
        AlignmentValue, BoxSizingValue, ClearValue, DisplayValue, FlexDirectionValue, FlexWrapValue, FloatValue,
        LengthValue, OverflowValue, PositionValue, VisibilityValue,
    };
    use zero_style_system::{ComputedStyle, FlexBasisValue, GridAutoFlowValue, GridLineValue};

    // ── computed_style_to_taffy ─────────────────────────────────────────

    #[test]
    fn test_computed_style_to_taffy_default() {
        let style = ComputedStyle::default();
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.display, taffy::style::Display::Block);
    }

    #[test]
    fn test_computed_style_to_taffy_flex() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.display, taffy::style::Display::Flex);
    }

    #[test]
    fn test_computed_style_to_taffy_grid() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.display, taffy::style::Display::Grid);
    }

    #[test]
    fn test_computed_style_to_taffy_position_relative() {
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Relative;
        style.top = LengthValue::Px(10.0);
        style.left = LengthValue::Px(20.0);
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.position, taffy::style::Position::Relative);
        assert_eq!(result.inset.top, taffy::style::LengthPercentageAuto::length(10.0));
        assert_eq!(result.inset.left, taffy::style::LengthPercentageAuto::length(20.0));
    }

    #[test]
    fn test_computed_style_to_taffy_position_absolute() {
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Absolute;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.position, taffy::style::Position::Absolute);
    }

    #[test]
    fn test_computed_style_to_taffy_position_fixed() {
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Fixed;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.position, taffy::style::Position::Absolute); // taffy maps fixed to absolute
    }

    #[test]
    fn test_computed_style_to_taffy_padding() {
        let mut style = ComputedStyle::default();
        style.padding_top = LengthValue::Px(10.0);
        style.padding_right = LengthValue::Px(20.0);
        style.padding_bottom = LengthValue::Px(30.0);
        style.padding_left = LengthValue::Px(40.0);
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.padding.top, taffy::style::LengthPercentage::length(10.0));
        assert_eq!(result.padding.right, taffy::style::LengthPercentage::length(20.0));
        assert_eq!(result.padding.bottom, taffy::style::LengthPercentage::length(30.0));
        assert_eq!(result.padding.left, taffy::style::LengthPercentage::length(40.0));
    }

    #[test]
    fn test_table_padding_zeroed_in_border_collapse() {
        // CSS 2.1 §17.6.2：border-collapse:collapse 模式下 table 盒的 padding 不应用。
        // display:table + border-collapse:collapse → padding 归零（不论显式 padding 多大）。
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Table;
        style.border_collapse = BorderCollapseValue::Collapse;
        style.padding_top = LengthValue::Px(100.0);
        style.padding_right = LengthValue::Px(100.0);
        style.padding_bottom = LengthValue::Px(100.0);
        style.padding_left = LengthValue::Px(100.0);
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        let zero = taffy::style::LengthPercentage::length(0.0_f32);
        assert_eq!(result.padding.top, zero);
        assert_eq!(result.padding.right, zero);
        assert_eq!(result.padding.bottom, zero);
        assert_eq!(result.padding.left, zero);
    }

    #[test]
    fn test_table_padding_kept_in_border_separate() {
        // border-collapse:separate（默认）→ table padding 正常应用（§17.5：仅 collapse 模式禁用 table padding）。
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Table;
        // border_collapse 默认 Separate
        style.padding_top = LengthValue::Px(10.0);
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.padding.top, taffy::style::LengthPercentage::length(10.0));
    }

    #[test]
    fn test_computed_style_to_taffy_margin_auto() {
        let mut style = ComputedStyle::default();
        style.margin_left = LengthValue::Auto;
        style.margin_right = LengthValue::Auto;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.margin.left, taffy::style::LengthPercentageAuto::auto());
        assert_eq!(result.margin.right, taffy::style::LengthPercentageAuto::auto());
    }

    #[test]
    fn test_computed_style_to_taffy_size_percentage() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Percentage(50.0);
        style.height = LengthValue::Percentage(75.0);
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.size.width, taffy::style::Dimension::percent(0.5));
        assert_eq!(result.size.height, taffy::style::Dimension::percent(0.75));
    }

    #[test]
    fn test_computed_style_to_taffy_size_auto() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Auto;
        style.height = LengthValue::Auto;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.size.width, taffy::style::Dimension::auto());
        assert_eq!(result.size.height, taffy::style::Dimension::auto());
    }

    #[test]
    fn test_computed_style_to_taffy_min_max_size() {
        let mut style = ComputedStyle::default();
        style.min_width = LengthValue::Px(100.0);
        style.max_width = LengthValue::Px(500.0);
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.min_size.width, taffy::style::Dimension::length(100.0));
        assert_eq!(result.max_size.width, taffy::style::Dimension::length(500.0));
    }

    #[test]
    fn test_computed_style_to_taffy_overflow() {
        let mut style = ComputedStyle::default();
        style.overflow_x = OverflowValue::Hidden;
        style.overflow_y = OverflowValue::Scroll;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.overflow.x, taffy::style::Overflow::Hidden);
        assert_eq!(result.overflow.y, taffy::style::Overflow::Scroll);
    }

    #[test]
    fn test_computed_style_to_taffy_flex_direction() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        style.flex_direction = FlexDirectionValue::RowReverse;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.flex_direction, taffy::style::FlexDirection::RowReverse);
    }

    #[test]
    fn test_computed_style_to_taffy_flex_wrap() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        style.flex_wrap = FlexWrapValue::Wrap;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.flex_wrap, taffy::style::FlexWrap::Wrap);
    }

    #[test]
    fn test_computed_style_to_taffy_flex_basis_auto() {
        let mut style = ComputedStyle::default();
        style.flex_basis = FlexBasisValue::Auto;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.flex_basis, taffy::style::Dimension::auto());
    }

    #[test]
    fn test_computed_style_to_taffy_flex_basis_length() {
        let mut style = ComputedStyle::default();
        style.flex_basis = FlexBasisValue::Length(LengthValue::Px(200.0));
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.flex_basis, taffy::style::Dimension::length(200.0));
    }

    #[test]
    fn test_visibility_collapse_zeros_flex_main_size() {
        // CSS Flexbox §10.1：visibility:collapse 的 flex item 主尺寸归零（strut），
        // 即使声明了显式 flex-basis / flex-grow / flex-shrink 也被覆盖。
        // 对应上游 reftest: flexbox-collapsed-item-horiz-001/002。
        let mut style = ComputedStyle::default();
        style.visibility = VisibilityValue::Collapse;
        style.flex_basis = FlexBasisValue::Length(LengthValue::Px(200.0));
        style.flex_grow = 1.0;
        style.flex_shrink = 1.0;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.flex_basis, taffy::style::Dimension::length(0.0_f32));
        assert_eq!(result.flex_grow, 0.0);
        assert_eq!(result.flex_shrink, 0.0);
    }

    #[test]
    fn test_visibility_visible_preserves_flex() {
        // 非 collapse 时 flex 属性正常透传。
        let mut style = ComputedStyle::default();
        style.flex_basis = FlexBasisValue::Length(LengthValue::Px(200.0));
        style.flex_grow = 2.0;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.flex_basis, taffy::style::Dimension::length(200.0));
        assert_eq!(result.flex_grow, 2.0);
    }

    #[test]
    fn test_computed_style_to_taffy_align_items_center() {
        let mut style = ComputedStyle::default();
        style.align_items = AlignmentValue::Center;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.align_items, Some(taffy::style::AlignItems::CENTER));
    }

    #[test]
    fn test_computed_style_to_taffy_justify_content_space_between() {
        let mut style = ComputedStyle::default();
        style.justify_content = AlignmentValue::SpaceBetween;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(
            result.justify_content,
            Some(taffy::style::JustifyContent::SPACE_BETWEEN)
        );
    }

    #[test]
    fn test_computed_style_to_taffy_gap() {
        let mut style = ComputedStyle::default();
        style.gap = LengthValue::Px(20.0); // gap.width = style.gap
        style.row_gap = LengthValue::Px(10.0); // gap.height = style.row_gap
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.gap.width, taffy::style::LengthPercentage::length(20.0));
        assert_eq!(result.gap.height, taffy::style::LengthPercentage::length(10.0));
    }

    #[test]
    fn test_computed_style_to_taffy_box_sizing() {
        let mut style = ComputedStyle::default();
        style.box_sizing = BoxSizingValue::BorderBox;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.box_sizing, taffy::style::BoxSizing::BorderBox);
    }

    #[test]
    fn test_computed_style_to_taffy_display_none() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::None;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.display, taffy::style::Display::None);
    }

    // ── parse_grid_template_areas ───────────────────────────────────────

    #[test]
    fn test_parse_grid_template_areas_simple() {
        let input = r#""a a" "b b""#;
        let areas = parse_grid_template_areas(input);
        assert_eq!(areas.len(), 2);
        assert_eq!(areas.get("a"), Some(&(1, 2, 1, 3)));
        assert_eq!(areas.get("b"), Some(&(2, 3, 1, 3)));
    }

    #[test]
    fn test_parse_grid_template_areas_single_cell() {
        let areas = parse_grid_template_areas(r#""main""#);
        assert_eq!(areas.get("main"), Some(&(1, 2, 1, 2)));
    }

    #[test]
    fn test_parse_grid_template_areas_dot_is_skipped() {
        // "." is an empty cell marker and should not be stored in the area map
        let areas = parse_grid_template_areas(r#""a ." "b b""#);
        assert!(areas.contains_key("a"));
        assert!(areas.contains_key("b"));
        assert!(!areas.contains_key("."), "空单元格标记 '.' 不应存储到区域映射中");
    }

    #[test]
    fn test_parse_grid_template_areas_empty() {
        let areas = parse_grid_template_areas("");
        assert!(areas.is_empty());
    }

    #[test]
    fn test_parse_grid_template_areas_3x3() {
        let input = r#""h h h" "s m m" "s m m""#;
        let areas = parse_grid_template_areas(input);
        let h = areas.get("h").unwrap();
        assert_eq!(*h, (1, 2, 1, 4)); // row 1, cols 1-3
        let s = areas.get("s").unwrap();
        assert_eq!(*s, (2, 4, 1, 2)); // rows 2-3, col 1
        let m = areas.get("m").unwrap();
        assert_eq!(*m, (2, 4, 2, 4)); // rows 2-3, cols 2-3
    }

    // ── resolve_grid_placement ──────────────────────────────────────────

    #[test]
    fn test_resolve_grid_placement_auto() {
        let style = ComputedStyle::default();
        let (rs, re, cs, ce) = resolve_grid_placement(&style, None);
        assert_eq!(rs, GridLineValue::Auto);
        assert_eq!(re, GridLineValue::Auto);
        assert_eq!(cs, GridLineValue::Auto);
        assert_eq!(ce, GridLineValue::Auto);
    }

    #[test]
    fn test_resolve_grid_placement_with_line_numbers() {
        let mut style = ComputedStyle::default();
        style.grid_row_start = GridLineValue::Line(2);
        style.grid_row_end = GridLineValue::Line(4);
        style.grid_column_start = GridLineValue::Line(1);
        style.grid_column_end = GridLineValue::Line(3);
        let (rs, re, cs, ce) = resolve_grid_placement(&style, None);
        assert_eq!(rs, GridLineValue::Line(2));
        assert_eq!(re, GridLineValue::Line(4));
        assert_eq!(cs, GridLineValue::Line(1));
        assert_eq!(ce, GridLineValue::Line(3));
    }

    #[test]
    fn test_resolve_grid_placement_with_span() {
        let mut style = ComputedStyle::default();
        style.grid_row_start = GridLineValue::Line(1);
        style.grid_row_end = GridLineValue::Span(2);
        style.grid_column_start = GridLineValue::Span(3);
        style.grid_column_end = GridLineValue::Line(5);
        let (rs, re, cs, ce) = resolve_grid_placement(&style, None);
        assert_eq!(rs, GridLineValue::Line(1));
        assert_eq!(re, GridLineValue::Span(2));
        assert_eq!(cs, GridLineValue::Span(3));
        assert_eq!(ce, GridLineValue::Line(5));
    }

    #[test]
    fn test_resolve_grid_placement_named_area() {
        let mut style = ComputedStyle::default();
        style.grid_row_start = GridLineValue::Name("header".to_string());
        style.grid_row_end = GridLineValue::Name("header".to_string());
        style.grid_column_start = GridLineValue::Name("header".to_string());
        style.grid_column_end = GridLineValue::Name("header".to_string());

        let mut areas = GridAreaMap::new();
        areas.insert("header".to_string(), (1, 2, 1, 4));

        let (rs, re, cs, ce) = resolve_grid_placement(&style, Some(&areas));
        assert_eq!(rs, GridLineValue::Line(1)); // row-start
        assert_eq!(re, GridLineValue::Line(2)); // row-end
        assert_eq!(cs, GridLineValue::Line(1)); // col-start
        assert_eq!(ce, GridLineValue::Line(4)); // col-end
    }

    #[test]
    fn test_resolve_grid_placement_unknown_name_falls_to_auto() {
        let mut style = ComputedStyle::default();
        style.grid_row_start = GridLineValue::Name("nonexistent".to_string());

        let areas = GridAreaMap::new();
        let (rs, _, _, _) = resolve_grid_placement(&style, Some(&areas));
        assert_eq!(rs, GridLineValue::Auto);
    }

    // ── convert_float / convert_clear ───────────────────────────────────

    #[test]
    fn test_convert_float_none() {
        assert!(!convert_float(&FloatValue::None));
    }

    #[test]
    fn test_convert_float_inline_end() {
        assert!(convert_float(&FloatValue::InlineEnd));
    }

    #[test]
    fn test_convert_clear_both() {
        assert!(convert_clear(&ClearValue::Both));
    }

    #[test]
    fn test_convert_clear_inline_start() {
        assert!(convert_clear(&ClearValue::InlineStart));
    }

    // ── grid auto flow ──────────────────────────────────────────────────

    #[test]
    fn test_computed_style_grid_auto_flow() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        style.grid_auto_flow = GridAutoFlowValue::ColumnDense;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.grid_auto_flow, taffy::style::GridAutoFlow::ColumnDense);
    }

    // ── computed_style_to_taffy with border ─────────────────────────────

    #[test]
    fn test_computed_style_border() {
        let mut style = ComputedStyle::default();
        style.border_top_width = LengthValue::Px(1.0);
        style.border_right_width = LengthValue::Px(2.0);
        style.border_bottom_width = LengthValue::Px(3.0);
        style.border_left_width = LengthValue::Px(4.0);
        // border-style=Solid 方能使 border-width 进入布局盒（CSS §8.5.3：style=none→width=0）
        style.border_top_style = BorderStyleValue::Solid;
        style.border_right_style = BorderStyleValue::Solid;
        style.border_bottom_style = BorderStyleValue::Solid;
        style.border_left_style = BorderStyleValue::Solid;
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.border.top, taffy::style::LengthPercentage::length(1.0));
        assert_eq!(result.border.right, taffy::style::LengthPercentage::length(2.0));
        assert_eq!(result.border.bottom, taffy::style::LengthPercentage::length(3.0));
        assert_eq!(result.border.left, taffy::style::LengthPercentage::length(4.0));
    }

    // ── ComputedStyle with percentage padding/margin ────────────────────

    #[test]
    fn test_computed_style_padding_percentage() {
        let mut style = ComputedStyle::default();
        style.padding_top = LengthValue::Percentage(10.0);
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.padding.top, taffy::style::LengthPercentage::percent(0.1));
    }

    #[test]
    fn test_computed_style_margin_percentage() {
        let mut style = ComputedStyle::default();
        // R1058：测垂直 margin 机制须用 block 上下文（display 默认 Inline，§8.3 垂直 margin 归零）。
        style.display = DisplayValue::Block;
        style.margin_bottom = LengthValue::Percentage(25.0);
        let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(result.margin.bottom, taffy::style::LengthPercentageAuto::percent(0.25));
    }
}
