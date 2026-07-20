//! 容器固有宽度（intrinsic / max-content）测量工具。
//!
//! 为 flex/grid 容器两趟固有宽度布局（见 `docs/goal/rendering-compat/flex-grid-two-pass-design.md`）
//! 提供测量基础。本模块**不参与布局**（compute() 不调用其改变布局的函数），
//! 仅提供纯计算函数 + 单元测试 + 可选诊断打印（env-gated），分轮渐进接线。
//!
//! 关键区别于 `table_shrink::block_max_content_width`：本模块的 `box_content_max_width`
//! 对「叶 block 显式 width」回退到自身显式宽度（R138 的函数对此返回 0，会漏测
//! `<div style="width:30px">` 这类叶盒），故 grid item 的固有宽度才能正确测量。

use std::collections::HashMap;

use zero_css_parser::values::{BoxSizingValue, DisplayValue, FlexDirectionValue, LengthValue, VisibilityValue};
use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::{ColumnSpanComputedValue, FlexBasisValue};

use crate::types::LayoutBox;

/// 计算一个盒的「内容最大宽度」（max-content）。
///
/// 递归规则（CSS intrinsic sizing）：
/// - inline 级子元素（含 inline-block）→ 水平求和（max-content 假设不换行）
/// - block 级子元素 → 取最大者的内容宽度
/// - **叶盒（无有效子元素贡献）且有显式 Px width → 回退到自身显式 width**
///   （这是与 `table_shrink::block_max_content_width` 的关键差异）
/// - **叶盒的文本内容**（Round C）：纯文本 item 此前测 0 致 flex/grid 容器 intrinsic
///   塌缩；此处按元素 font 度量逐字符累加文本宽度（Ahem 等宽=font_size）。
///   仅 max-content（不换行）；min-content（最宽词）独立子问题暂不实现。
///
/// 返回值含 box 自身的水平 padding+border（border-box 贡献）。
pub(crate) fn box_content_max_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    let mut inline_sum = 0.0f32;
    let mut block_max = 0.0f32;
    let mut has_in_flow_child = false;

    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        has_in_flow_child = true;
        let is_inline_level = child
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| {
                matches!(
                    s.display,
                    DisplayValue::Inline
                        | DisplayValue::InlineBlock
                        | DisplayValue::InlineFlex
                        | DisplayValue::InlineGrid
                        | DisplayValue::InlineTable
                )
            })
            .unwrap_or(false);
        let outer_w = child.width + child.margin_left + child.margin_right;
        // R1165：R109 §9.2.1.1 拆分 inline 父盒（is_r109_split，display:Inline 但已被拆成
        // 匿名块片段）的 max-content 须递归测其匿名块子（真实文本内容），而非用其
        // post-taffy 拉伸的 child.width。否则 table auto-layout 测含 split inline 的 cell
        // 时读到拉伸宽（td/table 链全宽）→ 表爆炸（block-in-inline-001：td 777px 应 ~50px，
        // R109=off 表 shrink-to-fit 93px ≈ oracle；R109=on 表 777px）。narrow gate：仅
        // is_r109_split 父盒（fragment_node_ids=None）递归，普通 inline 不变。
        let is_split_wrapper = child.is_r109_split && child.fragment_node_ids.is_none();
        if is_inline_level && !is_split_wrapper {
            // R1298：空 inline 元素（display:Inline，无元素子且无非空白文本，如 `<span></span>`）
            // 对父盒 max-content 宽贡献 0（无内容即零宽 inline 盒）。ZeroWeb 把 inline 映射为
            // taffy Block 拉伸到容器宽（R109/inline-stretch 已知多 session 缺口），空 inline 的
            // child.width 会被记成容器宽（如 200），若直接累入 inline_sum 会让含「空 inline +
            // block 子」的 inline-block（inline-block-baseline-015：`<span></span>` + 绿块）
            // 测得 intrinsic=容器宽 → shrink-to-fit 不触发 → 宽=容器 → 在 IFC 行中挤不下兄弟
            // → 换行错位。此处对**空 display:Inline** 贡献 0，绕过拉伸伪影（CSS 正确：空 inline
            // 盒零宽）。仅限 display:Inline：inline-block/inline-flex 等有自身盒模型（显式宽或
            // 独立 shrink-to-fit），child.width 真实有效（height-computed-001 的空 inline-block
            // 子 span[width:70px] 须贡献 70px，误判空会塌缩容器宽致回归）。
            let empty_inline = std::env::var("ZW_EMPTY_INLINE_WIDTH").as_deref() != Ok("0")
                && child
                    .node_id
                    .and_then(|cid| styles.get(&cid))
                    .is_some_and(|s| matches!(s.display, DisplayValue::Inline))
                && child.node_id.is_some_and(|cid| {
                    doc.child_nodes(cid).iter().all(|&gc| match doc.get(gc) {
                        Some(n) => match &n.kind {
                            zero_dom::NodeKind::Text(t) => t.content.trim().is_empty(),
                            zero_dom::NodeKind::Element(_) => false,
                            _ => true, // 注释/doctype 等不计为内容
                        },
                        None => true,
                    })
                });
            if empty_inline {
                // R1298：空 display:Inline 贡献 0（见上方注释）。
            } else if std::env::var("ZW_INLINE_INTRINSIC_CONTENT").as_deref() != Ok("0")
                && child
                    .node_id
                    .and_then(|cid| styles.get(&cid))
                    .is_some_and(|s| matches!(s.display, DisplayValue::Inline))
            {
                // R1479（R109 inline-box-model 首增量，kill-switch 默认关）：display:Inline
                // 子（非空、非 r109_split）按 **content-width 递归测量**，替代被 taffy Block
                // 拉伸到满宽的 outer_w。ZeroWeb 把 inline→taffy::Block（converter:337）致 inline
                // 子 child.width=容器宽，累入父 inline_sum 让含 inline 子的 inline-block 测得
                // intrinsic≥容器宽 → shrink-to-fit 不触发 → 满宽（vertical-align-122 的 8 个
                // wrapper 渲成单一满宽黑块的根因）。递归测真实内容宽（文本/嵌套 inline）让父正确
                // shrink。仅 display:Inline：inline-block/flex/grid 有自身盒模型，outer_w 真实有效
                //（height-computed-001 等），不走此路。default-off 待全量 A/B 验证 net≥0。
                inline_sum += box_content_max_width(child, doc, styles).max(0.0);
            } else {
                inline_sum += outer_w.max(0.0);
            }
        } else {
            block_max = block_max.max(box_content_max_width(child, doc, styles));
        }
    }

    let children_inner = inline_sum.max(block_max);
    // 叶盒回退：无有效子元素贡献时，用自身显式 Px width（content-box 语义）。
    // 显式 width 的叶盒（如 `<div style="width:50px">`）其 max-content 即该宽度。
    let own_explicit = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.width {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let inner = if !has_in_flow_child {
        // 叶盒：显式宽或文本内容宽（Round C）。纯文本 item（无 LayoutBox 子元素）
        // 之前测 0，现按 DOM 文本内容度量。取 max 避免显式宽被文本低估。
        let text_w = box_node
            .node_id
            .map_or(0.0, |id| text_content_max_width(id, doc, styles));
        own_explicit.max(text_w)
    } else if children_inner < own_explicit {
        own_explicit
    } else {
        children_inner
    };

    inner + box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right
}

/// R1018：block-level 容器的 max-content 宽度，对 flex/grid **子容器**分发到专用 intrinsic 函数。
///
/// 区别于 [`box_content_max_width`] 的通用递归：当 block 的子元素本身是 flex/grid 容器时，
/// flex/grid 容器的 intrinsic 宽度须用专用测量（`flex_row_intrinsic_width` 等，含 transferred
/// sizing / aspect-ratio 推导），而非通用递归（通用递归对 aspect-ratio 空 item 测 0）。
///
/// 用于 `width:max-content`/`fit-content` block 的 shrink-to-fit（CSS css-sizing-3）。返回 border-box。
/// 仅水平书写模式。leaf 文本/显式宽回退同 [`box_content_max_width`]。
pub(crate) fn block_max_content_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    let mut inline_sum = 0.0f32;
    let mut block_max = 0.0f32;
    // R1431 L3②：spanner-aware multicol intrinsic sizing 须区分 spanner / 非 spanner block 子。
    // `nonspanner_block_max` = 非 column-span:all block 子 intrinsic max（multicol 列内容宽）；
    // `spanner_max` = column-span:all 子 intrinsic max（spanner 跨全宽驱动宽度）。`block_max`
    //（含 spanner）保留供非 multicol `children_inner`（spanner 对普通 block 容器即普通子）。
    let mut nonspanner_block_max = 0.0f32;
    let mut spanner_max = 0.0f32;
    let mut has_in_flow_child = false;

    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        has_in_flow_child = true;
        let child_style = child.node_id.and_then(|id| styles.get(&id));
        let is_inline_level = child_style
            .map(|s| {
                matches!(
                    s.display,
                    DisplayValue::Inline
                        | DisplayValue::InlineBlock
                        | DisplayValue::InlineFlex
                        | DisplayValue::InlineGrid
                        | DisplayValue::InlineTable
                )
            })
            .unwrap_or(false);
        if is_inline_level {
            // inline-level 子：用 outer_w（已布局宽度）求和。inline-flex/inline-grid 的
            // intrinsic 测量由 shrink_inline_blocks_to_content（R180/R1017）路径处理，此处不重复。
            inline_sum += (child.width + child.margin_left + child.margin_right).max(0.0);
            continue;
        }
        let is_spanner = child_style.is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All));
        // block-level 子：若是 flex/grid 容器，dispatch 到专用 intrinsic 函数（R1018 关键）。
        let child_intrinsic = child_style
            .map(|s| match s.display {
                DisplayValue::Flex | DisplayValue::InlineFlex => {
                    let base = if matches!(
                        s.flex_direction,
                        FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
                    ) {
                        flex_column_intrinsic_width(child, doc, styles)
                    } else {
                        flex_row_intrinsic_width(child, doc, styles)
                    };
                    base.unwrap_or(0.0)
                }
                DisplayValue::Grid | DisplayValue::InlineGrid => {
                    grid_intrinsic_width(child, doc, styles).unwrap_or(0.0)
                }
                _ => box_content_max_width(child, doc, styles),
            })
            .unwrap_or_else(|| box_content_max_width(child, doc, styles));
        let with_margins = child_intrinsic + child.margin_left + child.margin_right;
        block_max = block_max.max(with_margins);
        if is_spanner {
            spanner_max = spanner_max.max(with_margins);
        } else {
            nonspanner_block_max = nonspanner_block_max.max(with_margins);
        }
    }

    let children_inner = inline_sum.max(block_max);

    // leaf 回退同 box_content_max_width：显式 Px width 或文本内容宽。
    let own_explicit = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.width {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let inner = if !has_in_flow_child {
        let text_w = box_node
            .node_id
            .map_or(0.0, |id| text_content_max_width(id, doc, styles));
        own_explicit.max(text_w)
    } else if children_inner < own_explicit {
        own_explicit
    } else {
        children_inner
    };

    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;

    // R1431 L3②：spanner-aware multicol intrinsic sizing（替 R1020 proxy）。
    // CSS Multicol §3.4 + §6.1：multicol 容器 max-content 宽 = max(column-driven, spanner-driven)。
    //   N            = column-count:Number(n) ? n : 1            // col-width-only → shrink-to-fit 下 1 列
    //   col_content  = column-width:Length(w) ? w : nonspanner_block_max   // col-width 设定则子溢出
    //   column_driven = N × col_content + (N-1) × gap
    //   spanner_driven = spanner_max                            // column-span:all 子跨全宽
    // 6 case 验证见 docs/goal/rendering-compat/spanner-aware-multicol-intrinsic-sizing.md。
    // R1020 proxy 两处错：① col-width 不参与（col-width-only 走 inner+frame 用 max 子宽）；
    // ② spanner 被 N× 误放大。本算法解两处。
    let mc_style = box_node.node_id.and_then(|id| styles.get(&id));
    let col_count_n = mc_style.and_then(|s| match s.column_count {
        zero_style_system::ColumnCountComputedValue::Number(n) => Some(n as usize),
        _ => None,
    });
    let col_width_set = mc_style.and_then(|s| match &s.column_width {
        zero_style_system::ColumnWidthComputedValue::Length(LengthValue::Px(w)) => Some(*w as f32),
        _ => None,
    });
    if col_count_n.is_some() || col_width_set.is_some() {
        // 仅当所有 in-flow 子 leaf（无元素孙）时应用 spanner-aware 算法——leaf 保证无**嵌套** spanner
        //（spanner 须在元素孙辈以下），N×column_driven 安全。非 leaf 子（含嵌套 spanner，如
        // intrinsic-size-003 div>div>div>column-span:all）破坏列流成 region，N× 不适用 → 回落 inner+frame
        //（intrinsic-size-003 旧 inner+frame=100 恰正确）。此 leaf 守卫 = R1020 原「无元素孙」判定。
        let no_nested_spanner = box_node
            .children
            .iter()
            .filter(|c| !c.is_absolute && !c.is_fixed)
            .all(|c| c.children.iter().all(|gc| gc.is_absolute || gc.is_fixed));
        if no_nested_spanner {
            let n = col_count_n.unwrap_or(1).max(1);
            let gap_px = mc_style
                .and_then(|s| match &s.column_gap {
                    LengthValue::Px(g) => Some(*g as f32),
                    _ => None,
                })
                .unwrap_or(0.0);
            // col_content：col-width 设定则用之（子溢出，不撑宽 multicol），否则取最宽非 spanner 子 intrinsic。
            let col_content = col_width_set.unwrap_or(nonspanner_block_max);
            let column_driven = n as f32 * col_content + (n as f32 - 1.0) * gap_px;
            // multicol intrinsic = max(column-driven, spanner-driven)。非 spanner 子是列内容（fit 或溢出），
            // 不额外撑宽（不加 .max(inner)——否则 col-width 案中宽于列的 block 会错误撑宽，如 width-005 case 1）。
            let mc_inner = column_driven.max(spanner_max);
            return mc_inner + frame;
        }
    }

    inner + frame
}

/// 测量一个 DOM 元素的文本内容 max-content 宽度（Round C：纯文本 flex/grid item 测量）。
///
/// 遍历 DOM 后代收集全部文本（`Document::text_content`），按 CSS 白空格折叠规则折叠后，
/// 用元素 font 度量逐字符累加宽度（复用 IFC 的 `estimate_char_width`：Ahem 等宽=font_size，
/// 其它字体按字符近似宽）。仅 max-content（假设不换行）；min-content（最宽词）独立子问题。
pub(crate) fn text_content_max_width(node_id: NodeId, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) -> f32 {
    let style = styles.get(&node_id);
    let (font_size, _line_height) = crate::inline::resolve_font_metrics(style);
    let is_ahem = style.is_some_and(|s| s.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem")));
    // R1747：`<br>` 是强制换行（CSS css-sizing-3：forced break 产生独立 line，max-content
    // 取最宽 line 而非全文本累加）。旧实现用 `doc.text_content`（递归扁平化，br 折成空）把
    // "short<br>much longer line<br>mid" 测成单行 201.6px（应 max-line 131.2px），致 inline-block
    // / float / leaf block shrink-to-fit 过宽。改为递归遍历 DOM 子树，按 `<br>` 切段，取最宽段。
    let mut segments: Vec<f32> = vec![0.0];
    text_max_width_walk(node_id, doc, font_size, is_ahem, &mut segments);
    segments.into_iter().fold(0.0f32, f32::max)
}

/// R1747：测量 `node_id` 自身（文本节点直接量；元素递归子树），把文本字符宽累入当前段，
/// 遇 `<br>` 元素开新段（嵌套 br 同样切段）。`segments` 每项 = 一段宽。
/// R1748：改为处理 node 自身（text/br/element 三态），使 fragment_node_ids（可能是文本
/// 节点）亦可用此函数。
fn text_max_width_walk(node_id: NodeId, doc: &Document, font_size: f32, is_ahem: bool, segments: &mut Vec<f32>) {
    let Some(node) = doc.get(node_id) else { return };
    match &node.kind {
        zero_dom::NodeKind::Text(t) => {
            let collapsed = crate::inline::collapse_whitespace(&t.content);
            if !collapsed.is_empty() {
                let w: f32 = collapsed
                    .chars()
                    .map(|ch| crate::inline::estimate_char_width(ch, font_size, is_ahem))
                    .sum();
                *segments.last_mut().expect("segments 非空") += w;
            }
        }
        zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br") => {
            segments.push(0.0);
        }
        zero_dom::NodeKind::Element(_) => {
            for child in doc.child_nodes(node_id) {
                text_max_width_walk(child, doc, font_size, is_ahem, segments);
            }
        }
        _ => {}
    }
}

/// R109 §9.2.1.1：测量 split inline 的一个匿名块片段的 inline 内容 max-content 宽度。
///
/// 片段内的 DOM 子节点（文本节点 + inline-level 元素）按 inline 级求和（max-content
/// 假设不换行），字体度量取自 split inline 自身（片段继承其 font-family/size）。
/// 用于匿名块收缩到文本宽，使 inline 的 border/background 落在文本宽而非全宽
/// （inline-box-001 等 §9.2.1.1 用例）。返回 0 = 不可测（无文本）。
///
/// R1748：br-aware（同 R1747 text_content_max_width）——片段内含 `<br>` 时按最宽行而非
/// 全文本累加（forced break 产生独立 line）。无 br 片段行为不变（单段 = 累加）。
pub(crate) fn fragment_inline_max_width(
    inline_style: &ComputedStyle,
    fragment_node_ids: &[NodeId],
    doc: &Document,
) -> f32 {
    let (font_size, _line_height) = crate::inline::resolve_font_metrics(Some(inline_style));
    let is_ahem = inline_style.font_family.iter().any(|f| f.eq_ignore_ascii_case("Ahem"));
    // R1748：br-aware —— fragment_node_ids 共享一组 segments（同片段 inline 级内容按序累入
    // 当前段，遇 br 切段），取最宽段。无 br 时单段 = 全文本累加（行为同旧 total）。
    let mut segments: Vec<f32> = vec![0.0];
    for nid in fragment_node_ids {
        text_max_width_walk(*nid, doc, font_size, is_ahem, &mut segments);
    }
    segments.into_iter().fold(0.0f32, f32::max)
}

/// 计算 flex item 的主轴 base size（CSS Flexbox §9.2 flex base size）。
///
/// 优先级：`flex-basis` 显式长度 > `width` 显式长度 > 内容 max-content。
/// - `flex-basis: auto`/`content` → 回退到 width 或内容
/// - 无法确定（无显式值且内容为 0）→ 返回 0.0（调用方应作 no-op 处理）
///
/// 返回 border-box 贡献（含 item 自身 padding+border，不含 margin——margin 由容器求和时加）。
fn flex_item_base_size(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    container_cross: Option<f32>,
) -> f32 {
    let style = box_node.node_id.and_then(|id| styles.get(&id));
    // R1840：mirror converter §10.1 visibility:collapse 逻辑（converter/mod.rs:241，R1834）。
    // flexible collapsed item（flex-grow>0，或 ③ kill-switch ZW_VC_NONFLEX_STRUT=0）→ flex_basis=0，
    // 主尺寸贡献仅 frame（border+padding），与 converter 设 taffy flex_basis=0 一致。
    // 非-flexible collapsed（③ ON，flex-grow==0）保留原 base 作 strut，走下方原逻辑。
    // 修 flexbox-collapsed-item-horiz-001 Row4：旧 intrinsic 对 collapsed flexible item 读
    // width:20px 返 20，与 converter flex_basis=0 不一致 → 容器 intrinsic=42（应 22）→
    // float shrink-to-fit（b.width > intrinsic）不触发 → flexible item grow 到 40（应 20）。
    if let Some(s) = style {
        let collapsed = matches!(s.visibility, VisibilityValue::Collapse);
        let nonflex_strut_off = std::env::var("ZW_VC_NONFLEX_STRUT").as_deref() == Ok("0");
        if collapsed && (nonflex_strut_off || (s.flex_grow as f32) > 0.0) {
            let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
            return frame;
        }
    }
    // 1. flex-basis 显式长度优先
    if let Some(s) = style
        && let FlexBasisValue::Length(len) = &s.flex_basis
    {
        if let LengthValue::Px(v) = len {
            let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
            return (*v as f32) + frame;
        }
    }
    // 2. width 显式长度
    if let Some(s) = style
        && let LengthValue::Px(v) = &s.width
    {
        let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
        return (*v as f32) + frame;
    }
    // 2.5 R1015/R1017：aspect-ratio transferred-size——width:auto + aspect_ratio + definite main。
    // main 来源优先级：(a) item 自身 height Px；(b) item min-height Px 地板；(c) R1017 container-
    // stretch cross（容器 definite height Px 拉伸 item，如 inline-flex height:100px；经
    // shrink_inline_blocks_to_content IFC 路径调用，绕过 R1016 的 taffy gate 墙）。
    if let Some(s) = style
        && matches!(s.width, LengthValue::Auto)
        && let Some(ratio) = s.aspect_ratio.filter(|&r| r > 0.0)
    {
        let main = match &s.height {
            LengthValue::Px(v) => Some(*v as f32),
            _ => match &s.min_height {
                LengthValue::Px(v) => Some(*v as f32),
                _ => container_cross,
            },
        };
        if let Some(main) = main {
            return aspect_ratio_transferred_width(s, box_node, main, ratio);
        }
    }
    // 3. 内容 max-content（Round C：含纯文本 item 的文本宽度）
    box_content_max_width(box_node, doc, styles)
}

/// R1015：aspect-ratio transferred width（非替换 item）。`main` = item definite main-size（height）
/// 的 Px 数值（border-box 或 content-box 由 `box-sizing` 决定）。返回 border-box width。
///
/// - `border-box`：aspect-ratio 作用于 border-box，width_bb = height_bb × ratio = main × ratio。
/// - `content-box`：aspect-ratio 作用于 content-box，width_content = main × ratio，
///   border-box width = width_content + 水平 frame。
fn aspect_ratio_transferred_width(s: &ComputedStyle, box_node: &LayoutBox, main: f32, ratio: f32) -> f32 {
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    if matches!(s.box_sizing, BoxSizingValue::BorderBox) {
        main * ratio
    } else {
        main * ratio + frame
    }
}

/// 计算一个**水平 flex 行容器**的固有宽度（max-content 主尺寸）。
///
/// = Σ flex item base size + item margins + gaps + 容器水平 padding/border。
/// 仅对 `display:flex`/`inline-flex` 且主轴为水平（flex-direction: row/row-reverse）的容器有意义。
/// 返回 None 表示无法确定（如无流内 item）。
pub(crate) fn flex_row_intrinsic_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Option<f32> {
    // R1017：容器 definite cross（height Px）作 item stretch 源——item 无自身 main 时，
    // width = container_content_height × ratio（inline-flex height:100px + item aspect-ratio:1/1）。
    // R1018：百分比/auto height 经 taffy 第一趟已解析到 LayoutBox.height（border-box），
    // 作 fallback container_cross（flex 子 height:100% 在 definite-height 父内已解析）。
    let container_cross = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.height {
            LengthValue::Px(v) => {
                let vframe =
                    box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
                let content = if matches!(s.box_sizing, BoxSizingValue::BorderBox) {
                    (*v as f32) - vframe
                } else {
                    *v as f32
                };
                Some(content.max(0.0))
            }
            _ => {
                // 非 Px（百分比/auto/em）：用 taffy 第一趟解析的 border-box height 减 frame。
                let vframe =
                    box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
                let resolved = (box_node.height - vframe).max(0.0);
                (resolved > 0.0).then_some(resolved)
            }
        });
    let mut sum = 0.0f32;
    let mut count = 0usize;
    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        // 仅统计直接 flex item（block 级流内子元素）
        let is_item = child
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| !matches!(s.display, DisplayValue::None | DisplayValue::Contents))
            .unwrap_or(true);
        if is_item && child.is_block_level {
            count += 1;
            sum += flex_item_base_size(child, doc, styles, container_cross) + child.margin_left + child.margin_right;
        }
    }
    if count == 0 {
        return None;
    }
    let gap = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.gap {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    Some(sum + gap * (count - 1) as f32 + frame)
}

/// 计算一个**垂直 flex 列容器**的固有宽度（cross 轴 max-content）。
///
/// = max(item base size + item margins) + 容器水平 padding/border。列容器的主轴是垂直，
/// cross 轴（width）取最宽 item（非 row 的求和）。R1015：驱动案 flex-item-transferred-sizes-padding
///（float:left + flex-direction:column + item aspect-ratio:1/1 + min-height:100px）。
/// 返回 None 表示无法确定（如无流内 item）。
pub(crate) fn flex_column_intrinsic_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Option<f32> {
    let mut max = 0.0f32;
    let mut count = 0usize;
    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        let is_item = child
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| !matches!(s.display, DisplayValue::None | DisplayValue::Contents))
            .unwrap_or(true);
        if is_item && child.is_block_level {
            count += 1;
            // column：computing container width（cross）— container_cross = width 是循环，传 None。
            let base = flex_item_base_size(child, doc, styles, None) + child.margin_left + child.margin_right;
            if base > max {
                max = base;
            }
        }
    }
    if count == 0 {
        return None;
    }
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    Some(max + frame)
}

/// 计算一个 **grid 容器**的固有宽度（max-content 主尺寸）。
///
/// 近似实现（taffy 0.7 无原生 grid auto-track 扩展，此处用 item base size 估算）：
/// - `grid-auto-flow: column`（item 水平排列）→ Σ item base size + gaps
/// - 其它（默认 row，item 垂直堆叠）→ max item base size
///
/// 其中 item base size = `box_content_max_width`（含叶显式宽回退，故 `.item > .content(50px)`
/// 会测为 50+frame）。返回 None 表示无流内 item。
pub(crate) fn grid_intrinsic_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Option<f32> {
    let style = box_node.node_id.and_then(|id| styles.get(&id));
    let is_column_flow = style
        .map(|s| {
            // grid-auto-flow 含 "column" → item 水平排列
            matches!(
                s.grid_auto_flow,
                zero_style_system::property::types::GridAutoFlowValue::Column
                    | zero_style_system::property::types::GridAutoFlowValue::ColumnDense
            )
        })
        .unwrap_or(false);
    let gap = style
        .and_then(|s| match &s.column_gap {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let mut sum = 0.0f32;
    let mut max_w = 0.0f32;
    let mut count = 0usize;
    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        let is_item = child
            .node_id
            .and_then(|id| styles.get(&id))
            .map(|s| !matches!(s.display, DisplayValue::None | DisplayValue::Contents))
            .unwrap_or(true);
        if is_item && child.is_block_level {
            count += 1;
            let base = box_content_max_width(child, doc, styles) + child.margin_left + child.margin_right;
            sum += base;
            max_w = max_w.max(base);
        }
    }
    if count == 0 {
        return None;
    }
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    // 显式 grid-template-columns 时，每个 item 落入一个独立列，grid 的 max-content
    // 宽度 = 各列 max-content 之和（而非默认 row flow 单列取最大）。
    // 保守守卫：仅当显式 track 数 >= item 数时求和（每 item 独占一列），避免 item
    // 跨行换列导致过计。fit-content(L)/固定长度 track 的 L 钳制未建模（item 的
    // min-content 地板通常已 >= L，故不缩窄；残余边界由 reftest 验证）。
    let multi_column = is_column_flow || style.and_then(count_explicit_grid_columns).is_some_and(|n| n >= count);
    let inner = if multi_column {
        sum + gap * (count - 1) as f32
    } else {
        max_w
    };
    Some(inner + frame)
}

/// 统计显式 `grid-template-columns` 定义的 track 数（用于 grid 内在宽度测量）。
///
/// 括号感知按空白分割：`fit-content(30px)`、`minmax(a,b)`、`repeat(n, ...)` 各算 1 个
/// token（`repeat` 展开计数复杂，保守按 1 计——只会少计 track 数，不会误判为多列）。
/// 返回 `None` 表示无显式列定义（默认 None 或 `none`）。
fn count_explicit_grid_columns(s: &ComputedStyle) -> Option<usize> {
    let cols = s.grid_template_columns.as_deref()?.trim();
    if cols.is_empty() || cols.eq_ignore_ascii_case("none") {
        return None;
    }
    let mut count = 0usize;
    let mut depth = 0i32;
    let mut in_token = false;
    for ch in cols.chars() {
        match ch {
            '(' => {
                depth += 1;
                in_token = true;
            }
            ')' => depth -= 1,
            c if c.is_whitespace() && depth == 0 => {
                if in_token {
                    count += 1;
                    in_token = false;
                }
            }
            _ => in_token = true,
        }
    }
    if in_token {
        count += 1;
    }
    (count > 0).then_some(count)
}

/// 判断一个盒是否是 flex/grid 行容器（display:flex/inline-flex/grid/inline-grid）。
fn is_flex_grid_container(s: &ComputedStyle) -> bool {
    matches!(
        s.display,
        DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
    )
}

/// 诊断：遍历布局树，对 shrink-to-fit 候选容器打印测得的固有宽度 vs 当前宽度。
///
/// 候选 = flex/grid 容器且（width 为 auto/max-content/min-content，或容器本身是 inline-level
/// 或 float——这些应 shrink-to-fit 而非填满）。**仅 eprintln，不改变任何布局状态**（Round A）。
pub(crate) fn debug_dump_shrink_candidates(root: &LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    fn walk(b: &LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
        let Some(id) = b.node_id else {
            for c in &b.children {
                walk(c, doc, styles);
            }
            return;
        };
        let Some(s) = styles.get(&id) else {
            for c in &b.children {
                walk(c, doc, styles);
            }
            return;
        };
        if is_flex_grid_container(s) {
            let width_indefinite = matches!(
                s.width,
                LengthValue::Auto | LengthValue::MaxContent | LengthValue::MinContent
            );
            let is_inline = matches!(s.display, DisplayValue::InlineFlex | DisplayValue::InlineGrid);
            let is_float = !matches!(b.float, zero_css_parser::values::FloatValue::None);
            if width_indefinite || is_inline || is_float {
                let intrinsic = if matches!(s.display, DisplayValue::Grid | DisplayValue::InlineGrid) {
                    grid_intrinsic_width(b, doc, styles)
                } else {
                    flex_row_intrinsic_width(b, doc, styles)
                };
                if let Some(intrinsic) = intrinsic {
                    eprintln!(
                        "INTRINSIC_DBG: {:?} width={:?} float={:?} current_w={} intrinsic_w={} (delta={:.1})",
                        s.display,
                        s.width,
                        b.float,
                        b.width,
                        intrinsic,
                        b.width - intrinsic
                    );
                }
            }
        }
        for c in &b.children {
            walk(c, doc, styles);
        }
    }
    walk(root, doc, styles);
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_dom::NodeKind;

    /// 用 DOM 解析真实 HTML 计算样式，验证端到端测量。
    fn compute_intrinsic(html: &str, target_id: &str) -> Option<f32> {
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &zero_dom::Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let NodeKind::Element(e) = &n.kind
                && e.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let target = find(target_id, &doc, &result.root)?;
        flex_row_intrinsic_width(target, &doc, &styles)
    }

    /// 用 DOM 解析真实 HTML 计算样式，验证 grid 固有宽度测量（column flow 求和）。
    fn compute_grid_intrinsic(html: &str, target_id: &str) -> Option<f32> {
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &zero_dom::Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let NodeKind::Element(e) = &n.kind
                && e.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let target = find(target_id, &doc, &result.root)?;
        grid_intrinsic_width(target, &doc, &styles)
    }

    /// 用 DOM 解析真实 HTML，验证 block 容器（含 multicol）的 `block_max_content_width`。
    /// 复用 find 逻辑，目标盒调 `block_max_content_width`（multicol spanner-aware 路径）。
    fn compute_block_max_content(html: &str, target_id: &str) -> Option<f32> {
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &zero_dom::Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let NodeKind::Element(e) = &n.kind
                && e.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let target = find(target_id, &doc, &result.root)?;
        Some(block_max_content_width(target, &doc, &styles))
    }

    // ── R1431 L3② spanner-aware multicol intrinsic sizing（multicol-width-005 6 case）──

    /// case 1：column-width:80px + block(100px) + spanner(50px) → 80（col-width 设定，
    /// 宽于列的 block 溢出不撑宽；spanner 50 < 80）。N=1（count auto）。
    #[test]
    fn multicol_intrinsic_column_width_caps_overflowing_block() {
        let html = r#"<html><body style="margin:0">
          <article id="m" style="column-width:80px;column-gap:10px">
            <div style="width:100px">block1</div>
            <div style="column-span:all;width:50px">spanner</div>
          </article>
        </body></html>"#;
        let w = compute_block_max_content(html, "m").expect("multicol intrinsic");
        // col-width 80 → content 80，frame=0（article 无 border/padding）→ 80。
        assert!(
            (w - 80.0).abs() < 1.0,
            "case1: col-width:80 caps block100 → 80, got {}",
            w
        );
    }

    /// case 3：column-width:120px + spanner(150px) → 150（spanner 宽于 col-width 驱动）。
    #[test]
    fn multicol_intrinsic_spanner_wider_than_column_width_drives() {
        let html = r#"<html><body style="margin:0">
          <article id="m" style="column-width:120px;column-gap:10px">
            <div style="width:100px">block1</div>
            <div style="column-span:all;width:150px">spanner</div>
          </article>
        </body></html>"#;
        let w = compute_block_max_content(html, "m").expect("multicol intrinsic");
        assert!(
            (w - 150.0).abs() < 1.0,
            "case3: spanner150 > col-width120 → 150, got {}",
            w
        );
    }

    /// case 4：column-count:2 + block(100px) + spanner(narrow) → 2×100+10=210（N×非 spanner 子）。
    #[test]
    fn multicol_intrinsic_column_count_times_nonspanner_child() {
        let html = r#"<html><body style="margin:0">
          <article id="m" style="column-count:2;column-gap:10px">
            <div style="width:100px">block1</div>
            <div style="column-span:all">spanner</div>
          </article>
        </body></html>"#;
        let w = compute_block_max_content(html, "m").expect("multicol intrinsic");
        // 2×100(block) + 1×10(gap) = 210。spanner 文本窄不计。
        assert!((w - 210.0).abs() < 2.0, "case4: 2×100+10=210, got {}", w);
    }

    /// case 6：column-count:2 + block(100px) + spanner(250px) → 250（spanner 跨全宽不被 N×）。
    /// ★ 关键：旧 R1020 proxy 把 spanner 计入 children_inner 再 N× → 2×250=500（ZW 实测 514）。
    #[test]
    fn multicol_intrinsic_spanner_not_multiplied_by_column_count() {
        let html = r#"<html><body style="margin:0">
          <article id="m" style="column-count:2;column-gap:10px">
            <div style="width:100px">block1</div>
            <div style="column-span:all;width:250px">spanner</div>
          </article>
        </body></html>"#;
        let w = compute_block_max_content(html, "m").expect("multicol intrinsic");
        // column_driven=2×100+10=210，spanner_driven=250 → max=250。
        assert!((w - 250.0).abs() < 2.0, "case6: spanner250 not N× → 250, got {}", w);
    }

    /// case 5：column-count:2 + column-width:110px + block(100px) → 2×110+10=230
    ///（col-width 设定 → col_content=110 > block100）。
    #[test]
    fn multicol_intrinsic_column_width_overrides_block_when_set_with_count() {
        let html = r#"<html><body style="margin:0">
          <article id="m" style="column-count:2;column-width:110px;column-gap:10px">
            <div style="width:100px">block1</div>
            <div style="column-span:all">spanner</div>
          </article>
        </body></html>"#;
        let w = compute_block_max_content(html, "m").expect("multicol intrinsic");
        assert!((w - 230.0).abs() < 2.0, "case5: 2×110+10=230, got {}", w);
    }

    /// 嵌套 spanner（intrinsic-size-003：div>div>div>column-span:all）leaf 守卫拦截：
    /// 非 leaf 子（含元素孙）→ 不应用 N×column_driven，回落 inner+frame（spanner 嵌套破坏列流）。
    /// 验证不被错误放大（旧 proxy 拦截，新算法也须 leaf 守卫拦截）。
    #[test]
    fn multicol_intrinsic_nested_spanner_leaf_guard_no_multiply() {
        let html = r#"<html><body style="margin:0">
          <article id="m" style="column-count:3">
            <div><div><div>
              <div style="column-span:all"><div style="width:100px"></div></div>
            </div></div></div>
          </article>
        </body></html>"#;
        let w = compute_block_max_content(html, "m").expect("multicol intrinsic");
        // 嵌套 spanner → leaf 守卫失败 → 回落 inner+frame。inner = wrapper intrinsic ≈ 100（含 spanner 100 子）。
        // 不应被 3× 放大到 ~300。
        assert!(
            w < 150.0,
            "nested spanner: leaf guard must prevent 3× multiply, got {}",
            w
        );
    }

    #[test]
    fn test_grid_column_flow_sums_items() {
        // child-border-box-and-max-content 结构：grid-auto-flow:column，2 item，
        // 每个 item = .content(50) + padding 20×2 = 90 → grid 固有 = 180。
        let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid;grid-auto-columns:1fr;grid-auto-flow:column">
            <div style="padding:0 20px"><div style="width:50px"></div></div>
            <div style="padding:0 20px"><div style="width:50px"></div></div>
          </div>
        </body></html>"#;
        let w = compute_grid_intrinsic(html, "g").expect("grid intrinsic");
        assert!((w - 180.0).abs() < 2.0, "expected ~180px (2×(50+40)), got {}", w);
    }

    #[test]
    fn test_grid_row_flow_takes_max() {
        // 默认 grid-auto-flow:row → item 垂直堆叠 → 取最大 item 宽度（50）。
        let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid">
            <div style="width:30px"></div>
            <div style="width:50px"></div>
          </div>
        </body></html>"#;
        let w = compute_grid_intrinsic(html, "g").expect("grid intrinsic");
        assert!((w - 50.0).abs() < 1.0, "expected ~50px (max item), got {}", w);
    }

    #[test]
    fn test_grid_explicit_columns_sum_items() {
        // child-border-box-and-max-content-002 结构：显式 grid-template-columns
        // 2 个 fit-content track，2 item 各占一列 → grid 固有 = 各 item 求和（180），
        // 而非默认 row flow 的取最大（90）。item = .content(50) + padding 20×2 = 90。
        let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid;grid-template-columns:fit-content(30px) fit-content(80px)">
            <div style="padding:0 20px"><div style="width:50px"></div></div>
            <div style="padding:0 20px"><div style="width:50px"></div></div>
          </div>
        </body></html>"#;
        let w = compute_grid_intrinsic(html, "g").expect("grid intrinsic");
        assert!(
            (w - 180.0).abs() < 2.0,
            "expected ~180px (2×90, explicit columns sum), got {}",
            w
        );
    }

    #[test]
    fn test_grid_explicit_columns_fewer_tracks_takes_max() {
        // 显式 1 个 track，2 个 item → item 会换行到第 2 行复用同一列；
        // 保守取最大 item 宽度（不冒险过计），而非求和。
        let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid;grid-template-columns:100px">
            <div style="width:30px"></div>
            <div style="width:50px"></div>
          </div>
        </body></html>"#;
        let w = compute_grid_intrinsic(html, "g").expect("grid intrinsic");
        assert!(
            (w - 50.0).abs() < 1.0,
            "expected ~50px (max item, fewer tracks than items), got {}",
            w
        );
    }

    #[test]
    fn test_leaf_explicit_width_fallback() {
        // `.item > .content(width:50px)`：item max-content 应含 content 的 50px
        // （box_content_max_width 对叶 content 回退到 50）。
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex">
            <div style="width:50px"></div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        // 单 item width:50 → 50（无 padding/border）
        assert!((w - 50.0).abs() < 1.0, "expected ~50px, got {}", w);
    }

    #[test]
    fn test_flex_row_sum_two_items() {
        // 两个显式宽 item：30 + 50 = 80（行固有宽度）
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex">
            <div style="width:30px"></div>
            <div style="width:50px"></div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        assert!((w - 80.0).abs() < 1.0, "expected ~80px (30+50), got {}", w);
    }

    #[test]
    fn test_flex_basis_overrides_width() {
        // flex-basis 显式优先于 width：flex-basis:40px + width:50px → base 40
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex">
            <div style="flex-basis:40px;width:50px"></div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        assert!(
            (w - 40.0).abs() < 1.0,
            "flex-basis should win (expected ~40), got {}",
            w
        );
    }

    #[test]
    fn test_item_padding_adds_to_base() {
        // item 有 padding：width:50 + padding 10+10 = 70 border-box base
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex">
            <div style="width:50px;padding:0 10px"></div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        assert!((w - 70.0).abs() < 1.0, "expected ~70 (50+20 padding), got {}", w);
    }

    #[test]
    fn test_text_only_item_measured_round_c() {
        // Round C：纯文本 flex item（Ahem 10px 等宽）此前测 0，现按文本内容度量。
        // 5 字符 "XXXXX" × 10px = 50px（item 无 padding/border/margin）。
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex;font:10px/1 Ahem">
            <div>XXXXX</div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        assert!(
            (w - 50.0).abs() < 1.0,
            "expected ~50px (5×10px Ahem text, Round C), got {}",
            w
        );
    }

    #[test]
    fn test_nested_explicit_child_grid_like() {
        // grid item 场景：`.item(padding 20) > .content(width:50)` → item 内容 max = 50+40 = 90
        let html = r#"<html><body style="margin:0">
          <div id="c" style="display:flex">
            <div style="padding:0 20px"><div style="width:50px"></div></div>
          </div>
        </body></html>"#;
        let w = compute_intrinsic(html, "c").expect("flex row intrinsic");
        assert!(
            (w - 90.0).abs() < 1.0,
            "expected ~90 (50 content + 40 padding), got {}",
            w
        );
    }

    #[test]
    fn test_empty_container_returns_none() {
        let html = r#"<html><body style="margin:0"><div id="c" style="display:flex"></div></body></html>"#;
        let w = compute_intrinsic(html, "c");
        assert!(w.is_none(), "empty flex container should return None");
    }

    /// R1298：含「空 display:Inline 子 + block 子」的 inline-block 的 max-content 宽
    /// 应取 block 子宽（100），而非被空 inline 的 taffy 拉伸宽（容器宽）撑大。
    /// 修前：空 `<span></span>` 被拉伸到容器宽，inline_sum 累入 → intrinsic=容器宽
    /// → shrink-to-fit 不触发 → inline-block 宽=容器（inline-block-baseline-015 换行错位）。
    /// 修后：空 display:Inline 贡献 0 → intrinsic=100。
    #[test]
    fn test_empty_inline_child_does_not_stretch_max_content() {
        // inline-block(id=t) 宽 600 容器内：`<span></span>`（空 inline，会被 taffy 拉伸）
        // + `<div style="width:100px">`（block 子）。max-content 应 = 100。
        let html = r#"<html><body style="margin:0">
          <div style="width:600px">
            <div id="t" style="display:inline-block">
              <span></span>
              <div style="width:100px;height:50px"></div>
            </div>
          </div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &zero_dom::Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let NodeKind::Element(e) = &n.kind
                && e.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let target = find("t", &doc, &result.root).expect("inline-block target found");
        let w = box_content_max_width(target, &doc, &styles);
        assert!(
            (w - 100.0).abs() < 1.0,
            "R1298: empty inline child must contribute 0; expected ~100 (block child), got {w}"
        );
    }

    /// R1298：空 inline-**block**（display:InlineBlock，有显式宽）不可误判为「空 inline」
    /// 贡献 0——height-computed-001 的 `<span[display:inline-block][width:70px]>` 须贡献 70。
    #[test]
    fn test_empty_inline_block_with_explicit_width_still_contributes() {
        let html = r#"<html><body style="margin:0">
          <div style="width:600px">
            <div id="t" style="display:inline-block">
              <span style="display:inline-block;width:70px"></span>
              <span style="display:inline-block;width:70px"></span>
            </div>
          </div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &zero_dom::Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let NodeKind::Element(e) = &n.kind
                && e.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let target = find("t", &doc, &result.root).expect("inline-block target found");
        let w = box_content_max_width(target, &doc, &styles);
        assert!(
            (w - 140.0).abs() < 1.0,
            "R1298 guard: empty inline-block[70px] children must contribute; expected ~140, got {w}"
        );
    }

    /// R1165：取一个有效 NodeId（经 Document）。
    fn fresh_id_boxcw() -> zero_dom::NodeId {
        zero_dom::Document::new().create_element("div")
    }

    #[test]
    fn test_box_content_max_width_r109_split_wrapper_recurse_not_stretched() {
        // R1165：含 R109 拆分 inline 父盒（is_r109_split=true, display:Inline,
        // fragment_node_ids=None）的容器的 max-content 须递归测其匿名块子（真实内容 ~50），
        // 而非用 split 父盒 post-taffy 拉伸的 width（777，table auto-layout 链全宽）。
        // 复现 block-in-inline-001：td > span.inline(is_r109_split, w=777 stretched) >
        // [anon(Line1,w=50), block(Line2,w=50), anon(Line3,w=50)]。修前测 777（表爆炸），
        // 修后测 50（表 shrink-to-fit）。普通 inline（非 split）仍用拉伸宽（回归守卫）。
        use zero_css_parser::values::{DisplayValue, LengthValue};
        use zero_style_system::ComputedStyle;
        let wrapper_id = fresh_id_boxcw();
        let anon1_id = fresh_id_boxcw();
        let blk_id = fresh_id_boxcw();
        let anon2_id = fresh_id_boxcw();

        let mut wrapper = LayoutBox::default();
        wrapper.node_id = Some(wrapper_id);
        wrapper.is_r109_split = true; // R109 拆分 inline 父盒
        wrapper.fragment_node_ids = None; // 父盒非片段
        wrapper.width = 777.0; // post-taffy 拉伸宽（bug 源）

        for cid in [anon1_id, blk_id, anon2_id] {
            let mut anon = LayoutBox::default();
            anon.node_id = Some(cid);
            anon.is_block_level = true;
            anon.width = 50.0; // 真实内容宽（显式 width 叶盒）
            wrapper.children.push(anon);
        }

        let mut container = LayoutBox::default();
        container.children.push(wrapper);

        let mut styles = std::collections::HashMap::new();
        let mut inline_style = ComputedStyle::default();
        inline_style.display = DisplayValue::Inline; // split 父盒 display 仍是 Inline
        styles.insert(wrapper_id, inline_style);
        for cid in [anon1_id, blk_id, anon2_id] {
            let mut s = ComputedStyle::default();
            s.display = DisplayValue::Block;
            s.width = LengthValue::Px(50.0); // 显式宽叶盒
            styles.insert(cid, s);
        }
        let doc = zero_dom::Document::new();

        let w = box_content_max_width(&container, &doc, &styles);
        assert!(
            (w - 50.0).abs() < 1.0,
            "R109 split wrapper: recurse into anon children (~50), not stretched width (777); got {}",
            w
        );
    }

    // ── R1433 layout-time multicol balance height（multicol-rule-001 等）──

    /// balance multicol 容器（columns:2 + "1<br>2"）经 measure_text_content layout-time
    /// 返回均衡列高（ceil(L/N)×行高 = 1 行 = 20px），而非全宽 IFC 全高（2 行 = 40px）。
    /// 驱动案 multicol-rule-001（3.86→0.77 flip）。text-only（br 允许）+ overflow:visible + deterministic。
    #[test]
    fn multicol_balance_height_layout_time() {
        let html = r#"<html><body style="margin:0">
          <div id="m" style="columns:2;column-gap:0;font:20px/1 Ahem">1<br>2</div>
        </body></html>"#;
        let doc = zero_dom::parse_html(html);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        fn find<'a>(id: &str, doc: &zero_dom::Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
            if let Some(nid) = b.node_id
                && let Some(n) = doc.get(nid)
                && let NodeKind::Element(e) = &n.kind
                && e.get_attribute("id").as_deref() == Some(id)
            {
                return Some(b);
            }
            b.children.iter().find_map(|c| find(id, doc, c))
        }
        let div = find("m", &doc, &result.root).expect("multicol div");
        // 2 行 "1"/"2" 均衡到 2 列 → 1 行/列 = 20px（非未均衡的 40px）
        assert!(
            (div.content_height - 20.0).abs() < 1.0,
            "layout-time balance height: expected ~20 (1 line/col), got {}",
            div.content_height
        );
    }
}
