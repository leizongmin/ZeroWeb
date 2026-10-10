//! 容器固有宽度（intrinsic / max-content）测量工具。
//!
//! 为 flex/grid 容器两趟固有宽度布局（见 `docs/goal/rendering-compat/flex-grid-two-pass-design.md`）
//! 提供测量基础。本模块**不参与布局**（compute() 不调用其改变布局的函数），
//! 仅提供纯计算函数 + 单元测试 + 可选诊断打印（env-gated），分轮渐进接线。
//!
//! 关键区别于 `table_shrink::block_max_content_width`：本模块的 `box_content_max_width`
//! 对「叶 block 显式 width」回退到自身显式宽度（R138 的函数对此返回 0，会漏测
//! `<div style="width:30px">` 这类叶盒），故 grid item 的固有宽度才能正确测量。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use zero_css_parser::values::{
    BoxSizingValue, ClearValue, DisplayValue, FlexDirectionValue, FloatValue, LengthValue, OverflowValue,
    VisibilityValue,
};
use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::{ColumnSpanComputedValue, FlexBasisValue, WhiteSpaceValue};

use crate::types::LayoutBox;

// R4367：intrinsic 文本测量的真实 advance 源（线程本地，随布局线程注入）。
//
// 历史：`accumulate_text_width` / `DomWalkState` 恒走 `estimate_char_width`
// 启发式（0.55em/字母，sans 谱系标定）——R4365/R4366 把 layout/paint advance 源
// 统一到真实 serif hmtx 后，intrinsic 路径成为最后一处 estimate 分裂点
//（flexbox_flex-0-0-0 实证：同词 flex min-content=23.0（hmtx）vs inline-block
// shrink-to-fit=26.4（=3×0.55em 纯估计））。`LayoutEngine::set_advance_source`
// /`set_font_resolver` 发布线程本地；未发布（单测/无 pipeline 语境）回落估计。
// 杀开关 `ZW_INTRINSIC_REAL_ADVANCE=0`。
thread_local! {
    static INTRINSIC_ADVANCE: RefCell<Option<crate::inline::AdvanceSourceHandle>> = const { RefCell::new(None) };
    static INTRINSIC_RESOLVER: RefCell<Option<Rc<HashMap<String, u32>>>> = const { RefCell::new(None) };
}

/// 布局线程注入 intrinsic 测量用的 advance 源与 resolver（幂等，同值覆盖）。
pub fn publish_intrinsic_font_context(advance: crate::inline::AdvanceSourceHandle, resolver: Rc<HashMap<String, u32>>) {
    if std::env::var("ZW_INTRINSIC_REAL_ADVANCE").as_deref() == Ok("0") {
        return;
    }
    INTRINSIC_ADVANCE.with(|cell| *cell.borrow_mut() = Some(advance));
    INTRINSIC_RESOLVER.with(|cell| *cell.borrow_mut() = Some(resolver));
}

/// intrinsic 文本测量：真实 advance 源优先，未注入回落 estimate。
fn measure_intrinsic_char(ch: char, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
    INTRINSIC_ADVANCE.with(|cell| {
        cell.borrow().as_ref().map_or_else(
            || crate::inline::estimate_char_width(ch, font_size, is_ahem),
            |handle| handle.0.measure(ch, font_id, font_size, is_ahem),
        )
    })
}

/// 按元素 computed style 解析 intrinsic 测量 font_id（走 font_resolution 同一
/// 语义——含 R4365 空 family serif initial 臂）；无全局 resolver 时 None = estimate。
fn intrinsic_font_id(style: Option<&ComputedStyle>) -> Option<u32> {
    let style = style?;
    INTRINSIC_RESOLVER.with(|cell| {
        let resolver = cell.borrow();
        let resolver = resolver.as_ref()?;
        crate::font_resolution::resolve_font_ids_for_style(
            resolver,
            &style.font_family,
            &style.font_weight,
            &style.font_style,
            style.font_stretch,
        )
        .first()
        .copied()
    })
}

pub(crate) fn resolve_intrinsic_real_length(value: &LengthValue, style: &ComputedStyle) -> Option<f32> {
    match value {
        LengthValue::Auto
        | LengthValue::Percentage(_)
        | LengthValue::MinContent
        | LengthValue::MaxContent
        | LengthValue::FitContent(_) => None,
        LengthValue::Px(v) if *v == f64::INFINITY => None,
        other => {
            let font_size_px = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
            let px = zero_style_system::computed::resolve_length(other, font_size_px, None, None);
            px.is_finite().then_some((px as f32).max(0.0))
        }
    }
}

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
/// 返回值含 box 自身的水平 padding+border（border-box 贡献）；子盒水平 margin 的
/// intrinsic 贡献见下方 [`intrinsic_margin_contribution`]（% 分量记 0）。
///
/// 子盒水平 margin 的 intrinsic 贡献（css-sizing-3 #intrinsic-sizes + csswg #823066）。
///
/// 固有尺寸测量中**百分比 margin 解析为 0**：post-taffy 的 `child.margin_*` 对普通
/// 长度照用（已定值），对 `Calc(P% ± Npx)` 只取 px 部分（% 在内容定尺寸语境无基准，
/// 应记 0）。driving: WPT css-sizing calc-margins-block（min-content 容器内
/// `margin-left: calc(10% + 100px)` 的子，容器 max-content 应 = 100px 而非 0/膨胀值）。
fn intrinsic_margin_contribution(child: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> (f32, f32) {
    /// calc 表达式的 px 偏移部分（同 engine/postprocess 的提取语义，P%±Npx 模式）。
    /// intrinsic 语境 % → 0，仅 px 部分计入（css-sizing-3 #intrinsic-sizes）。
    fn calc_px_offset(expr: &zero_css_parser::values::CalcExpr) -> f32 {
        use zero_css_parser::values::{CalcExpr, CalcOp};
        match expr {
            CalcExpr::Length(_) => 0.0,
            CalcExpr::BinaryOp(left, op, right) => {
                let left_px = match left.as_ref() {
                    CalcExpr::Length(LengthValue::Px(v)) => Some(*v as f32),
                    _ => None,
                };
                let right_px = match right.as_ref() {
                    CalcExpr::Length(LengthValue::Px(v)) => Some(*v as f32),
                    _ => None,
                };
                let has_pct = |e: &CalcExpr| matches!(e, CalcExpr::Length(LengthValue::Percentage(_)));
                match (op, left_px, right_px) {
                    (CalcOp::Add, Some(px), None) if has_pct(right) => px,
                    (CalcOp::Add, None, Some(px)) if has_pct(left) => px,
                    (CalcOp::Subtract, Some(px), None) if has_pct(right) => -px,
                    (CalcOp::Subtract, None, Some(px)) if has_pct(left) => px,
                    _ => 0.0,
                }
            }
            _ => 0.0,
        }
    }
    let calc_px_margin = |value: &LengthValue| -> f32 {
        match value {
            LengthValue::Calc(expr) => calc_px_offset(expr).max(0.0),
            _ => 0.0,
        }
    };
    let Some(style) = child.node_id.and_then(|id| styles.get(&id)) else {
        return (child.margin_left, child.margin_right);
    };
    let resolve = |post_taffy: f32, css: &LengthValue| -> f32 {
        match css {
            LengthValue::Calc(_) => calc_px_margin(css),
            _ => post_taffy,
        }
    };
    (
        resolve(child.margin_left, &style.margin_left),
        resolve(child.margin_right, &style.margin_right),
    )
}

pub(crate) fn box_content_max_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    kw_aware_max_width(box_node, doc, styles, |b, d, s| {
        box_content_max_width_inner(b, d, s, true)
    })
}

/// R5037：content-only max-content——不含自身 definite width 的 own_explicit 抬升
///（fit-content() 公式的 W_max 口径：内在尺寸按内容基测量，specified 宽在使用侧钳制）。
/// 仅 R5037 贡献钳制消费。
pub(crate) fn box_content_max_width_content_only(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    kw_aware_max_width(box_node, doc, styles, |b, d, s| {
        box_content_max_width_inner(b, d, s, false)
    })
}

/// R4151（css-sizing-3 §5.2）：content 关键字 max/min-width 参与固有贡献——子盒带
/// `max-width:min-content/max-content` 时，其对父 max-content 的贡献 = min(自身
/// max-content, 自身 min-content 关键字语义)（dynamic-012：中段 div `max-width:
/// min-content; width:200px` 内 canvas 100 → 贡献 100 而非 200，float 链 shrink-to-fit
/// 才能收对）；`min-width:min-content` 时贡献下限 = min-content。递归测量 min-content
/// 用 max-content 近似（R1304 同口径）+ 显式定宽子收窄（R4149 守卫同源）。
fn kw_aware_max_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    measure: impl Fn(&LayoutBox, &Document, &HashMap<NodeId, ComputedStyle>) -> f32,
) -> f32 {
    let own = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .map(|s| (s.min_width.clone(), s.max_width.clone()));
    let Some((min_w, max_w)) = own else {
        return measure(box_node, doc, styles);
    };
    let is_kw = |v: &LengthValue| matches!(v, LengthValue::MinContent | LengthValue::MaxContent);
    if !is_kw(&min_w) && !is_kw(&max_w) {
        return measure(box_node, doc, styles);
    }
    // 自身 content 尺寸（剥 frame 前的 inner 由 measure 返回 border-box；此处用同口径）。
    let full = measure(box_node, doc, styles);
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    // min-content 近似：子的最小贡献 = 显式 CSS 定宽（R4149 收窄守卫同法），无显式宽
    // 时回退第一趟已解析盒宽（替换元素 canvas/img 的 attr/AR 传递尺寸在第一趟已解析
    // ——dynamic-012 canvas h:100%→100 AR→w=100；非替换 auto 子的 child.width 为 taffy
    // 拉伸伪影时可能偏大，只会让 cap 偏弱 no-op，安全方向）。
    let content_min = box_node
        .children
        .iter()
        .filter(|c| !(c.is_absolute || c.is_fixed))
        .filter_map(|c| {
            let cs = c.node_id.and_then(|cid| styles.get(&cid))?;
            let f = c.padding_left + c.padding_right + c.border_left + c.border_right;
            let w = resolve_kw_real_length(&cs.width, cs).unwrap_or(c.width);
            Some(w + f)
        })
        .fold(0.0_f32, f32::max)
        .max(frame);
    // 仅 `max-width:min-content` 收缩贡献（cap = min-content 近似）；`max-width:
    // max-content` 的 cap 就是 max-content 自身（= full），无需收缩——border-box-and-
    // max-content-002 的 .item（无子、max-width:max-content）曾误 cap 到 frame 塌盒。
    if matches!(max_w, LengthValue::MinContent) {
        let capped = full.min(content_min);
        return capped.max(min_resolved(&min_w, frame, full));
    }
    full
}

fn min_resolved(min_w: &LengthValue, frame: f32, full: f32) -> f32 {
    if matches!(min_w, LengthValue::MinContent | LengthValue::MaxContent) {
        full
    } else {
        frame
    }
}

/// 定值长度解析（intrinsic 语境：仅 Px/Em/Rem/Ch 等 real length）。
fn resolve_kw_real_length(value: &LengthValue, style: &zero_style_system::ComputedStyle) -> Option<f32> {
    match value {
        LengthValue::Auto | LengthValue::Percentage(_) | LengthValue::MinContent | LengthValue::MaxContent => None,
        LengthValue::Px(v) if *v == f64::INFINITY => None,
        other => {
            let font_size_px = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
            let px = zero_style_system::computed::resolve_length(other, font_size_px, None, None);
            px.is_finite().then_some(px.max(0.0) as f32)
        }
    }
}

fn box_content_max_width_inner(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    include_own_width: bool,
) -> f32 {
    // R4946（css-flexbox §9.9 intrinsic main size）：被通用递归测到的 flex/grid 容器
    //（典型：flex item 自身是 flex 行容器 → `flex_item_base_size` 第 3 步落到本函数；
    // R4032 inline-block 族递归遇 inline-flex 同理）须用专用 intrinsic 测量——flex 行
    // = Σ item base size，通用块递归按「块子各自成行取 max」会把行容器塌成单项宽。
    // `block_max_content_width` 已有同款分发（R1018），本函数此前缺位。
    // None（无流内 item）→ 落回通用路径保留叶/文本回退。
    if let Some(s) = box_node.node_id.and_then(|id| styles.get(&id)) {
        match s.display {
            DisplayValue::Flex | DisplayValue::InlineFlex => {
                let base = if matches!(
                    s.flex_direction,
                    FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
                ) {
                    flex_column_intrinsic_width(box_node, doc, styles)
                } else {
                    flex_row_intrinsic_width(box_node, doc, styles)
                };
                if let Some(w) = base {
                    return w;
                }
            }
            DisplayValue::Grid | DisplayValue::InlineGrid => {
                if let Some(w) = grid_intrinsic_width(box_node, doc, styles) {
                    return w;
                }
            }
            _ => {}
        }
    }
    let mut inline_sum = 0.0f32;
    let mut block_max = 0.0f32;
    // R4397：float 子横向叠加 + clear 强制换行（同 block_max_content_width loop 头注
    // ——max-content 语境 float 与 inline 内容同行并排；fit-content-contribution-001
    // float 行 0.000 精确实证）。
    let mut float_row = 0.0f32;
    let mut float_max = 0.0f32;
    let mut has_in_flow_child = false;
    // R4407：ZW_MIXED_BARE_TEXT 旗标 hoist（循环内每子一次 env::var → 每容器一次）。
    // R4408 开门：交错 walk 升 default-on（第七次实测 flag-on 14914 vs default 14900 =
    // 净 +14 零损失，达零新翻红 bar）；`ZW_MIXED_BARE_TEXT=0` 回退旧行为（opt-out）。
    let mixed_walk_on = std::env::var("ZW_MIXED_BARE_TEXT").as_deref() != Ok("0");

    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        has_in_flow_child = true;
        // R4407：匿名文本盒（块容器直接文本的第一趟产物，node_id=文本节点）贡献 0——
        // 其文本由本轮 dom_inline_text_max_width DOM walk 统一计入。旧路径 anon 盒经
        // box_content_max_width(child)（node_id=文本节点 → text_content_max_width）与
        // walk **双计同一文本**（margin-collapse-101/105 flag-on：div.b 内 [anon "B"]
        // 33.3 + walk 33.3 = 66.7，td 链测得 2× 文本宽、列宽翻倍实证）；default 臂
        //（walk 不运行）维持 anon 盒计宽不变。
        if mixed_walk_on && child.is_anonymous_text_item {
            continue;
        }
        // R4136：固有测量中 Calc(P%±Npx) margin 只取 px 部分（% → 0）；普通 margin 照用。
        let (ml, mr) = intrinsic_margin_contribution(child, styles);
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
        let outer_w = child.width + ml + mr;
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
                // R4134（CSS2 §10.3.9 + inline 盒模型）：空 inline 但带 frame（padding/border，
                // 如 word-spacing-characters-001 ref 页 `.spacer{padding-left:4em}` 空嵌套
                // span）时其 max-content 贡献 = frame——空 inline 盒的内容宽为 0，但 padding
                // 是盒几何的一部分（chromium 实测 spacer 撑开 4em）。纯空 span（frame=0）
                // 维持 R1298 的 0 贡献不变。旧实现一概 0 → 含空 padded inline 的 bg-bearing
                // inline 父测得 intrinsic 0 → R4033 guard 跳过 shrink → taffy 拉伸满宽
                // （外层 span 蓝条画满 767px）。
                inline_sum +=
                    (child.padding_left + child.padding_right + child.border_left + child.border_right + ml + mr)
                        .max(0.0);
            } else if std::env::var("ZW_MIXED_BARE_TEXT").as_deref() != Ok("0")
                && child
                    .node_id
                    .and_then(|cid| styles.get(&cid))
                    .is_some_and(|s| matches!(s.display, DisplayValue::Inline))
            {
                // R4395（ZW_MIXED_BARE_TEXT 交错 walk 配对臂；R4408 升 default-on）：display:Inline
                // 子仅计 frame（padding/border + margin）——其文本由本轮末尾的 `dom_inline_text_max_width`
                // DOM 直读统一计入（R4355 同款配对：block_max_content_width 的 is_plain_inline
                // 臂 + 末尾 walk，双计免疫由「loop 计 frame、walk 计文本」的分工保证）。
                // ruby run margin/emoji padding 等盒模型经 frame+ml+mr 入账（ruby-intrinsic-isize-003
                // 的 ruby margin/padding/border 参与面）。`ZW_MIXED_BARE_TEXT=0` opt-out 走下方
                // R1479 content-width 递归。
                let frame = child.padding_left + child.padding_right + child.border_left + child.border_right;
                inline_sum += (frame + ml + mr).max(0.0);
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
            } else if std::env::var("ZW_IB_DESTRETCH").as_deref() == Ok("1")
                && child.node_id.and_then(|cid| styles.get(&cid)).is_some_and(|s| {
                    matches!(
                        s.display,
                        DisplayValue::InlineBlock
                            | DisplayValue::InlineFlex
                            | DisplayValue::InlineGrid
                            | DisplayValue::InlineTable
                    )
                })
                && !child.node_id.and_then(|cid| doc.get(cid)).is_some_and(|n| {
                    matches!(
                        &n.kind,
                        zero_dom::NodeKind::Element(e)
                            if matches!(
                                e.local_name(),
                                "img" | "video" | "audio" | "canvas" | "iframe" | "embed"
                                    | "object" | "svg"
                            )
                    )
                })
            {
                // R4032（CSS Sizing §5.1 max-content）：inline-block 族子元素在 auto 容器
                // 中被 taffy 同步拉伸（容器拉伸 → 子填满容器），outer_w 是伪影而非真实
                // max-content——按 DOM 递归测量（与 R1479 对 Inline 子同语义，扩展到
                // inline-block 族）。R1298 的「outer_w 真实有效」假设仅在容器不被拉伸时
                // 成立；shrink-to-fit 语境（本函数的所有调用方）恰恰是拉伸发生的地方
                //（padding-right-applies-to-012：inline-block 嵌套被推到容器右缘实证）。
                // inline-block 族有真盒模型 → 递归返回其 border-box max-content，直接
                // 可比 outer_w。
                inline_sum += (box_content_max_width(child, doc, styles) + ml + mr).max(0.0);
            } else {
                inline_sum += outer_w.max(0.0);
            }
        } else if !matches!(child.float, FloatValue::None) {
            // R4397：float 子横向叠加 + clear 换行（见 loop 头注）。
            if !matches!(child.clear, ClearValue::None) {
                float_max = float_max.max(float_row);
                float_row = 0.0;
            }
            float_row += (box_content_max_width(child, doc, styles) + ml + mr).max(0.0);
        } else {
            // R4568（CSS2 §10.3.3 max-content）：definite CSS 宽**块级**子贡献 = 自身
            // definite outer width，不深递归文本重测——正交垂直子（vert-block-size-1
            // 容器 `width:200` definite）被按水平文本模型深测出 234（>200）污染 wrapper
            // shrink-to-fit（238 vs chromium 204，wrapper2 x 偏移 10px 实证）。
            // auto/percent 宽子维持深递归（taffy 拉伸伪影须按内容测，R1298 语境）；
            // inline 级落块支仅 R109 split wrapper（R1165 契约 = 递归匿名子，不截断）。
            // env ZW_IB_MAXW_DEFWIDTH=0 回退深递归（kill-switch，default-on）。
            let definite_child_w = std::env::var("ZW_IB_MAXW_DEFWIDTH").as_deref() != Ok("0")
                && !is_inline_level
                && child
                    .node_id
                    .and_then(|cid| styles.get(&cid))
                    .and_then(|cs| resolve_intrinsic_real_length(&cs.width, cs))
                    .is_some();
            if definite_child_w {
                block_max = block_max.max(outer_w.max(0.0));
            } else {
                block_max = block_max.max(box_content_max_width(child, doc, styles));
            }
        }
    }

    // R4397：float 行与 block 行正交取 max。
    let float_contribution = float_max.max(float_row);
    let children_inner = inline_sum.max(block_max).max(float_contribution + inline_sum);
    // 叶盒回退：无有效子元素贡献时，用自身显式 Px width（content-box 语义）。
    // 显式 width 的叶盒（如 `<div style="width:50px">`）其 max-content 即该宽度。
    let own_style = box_node.node_id.and_then(|id| styles.get(&id));
    // R4008（css-sizing-4 §intrinsic-size-override）：自身 contain:size 且 width 为
    // content-based 关键字（min/max/fit-content/Auto）时，intrinsic 宽 = CIS 替代值
    //（004：width:min-content + CIS 111 → 111 应 0）。显式 Px width 不受影响
    //（containment 不覆盖显式尺寸）。
    let own_cis = own_style
        .filter(|s| s.contain.has_size())
        .filter(|s| {
            matches!(
                s.width,
                LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_)
            )
        })
        .and_then(|s| {
            s.contain_intrinsic_width
                .as_ref()
                .and_then(|v| resolve_intrinsic_real_length(v, s))
        })
        .unwrap_or(0.0);
    let own_explicit = own_style
        .and_then(|s| resolve_intrinsic_real_length(&s.width, s))
        .filter(|_| include_own_width)
        .unwrap_or(0.0)
        .max(own_cis);
    // R3792：aspect-ratio transferred width（css-sizing-4 §4.1 transferred size）——
    // width:auto + aspect_ratio + definite height（Px）的块盒，其 intrinsic 宽贡献 =
    // height × ratio（intrinsic-size-001：`height:100px; aspect-ratio:1/1` 子对
    // `width:min-content` 父的 min-content 贡献应 100px，旧测 0 → 父回退 Auto 满宽）。
    // flex item 路径已有同逻辑（flex_item_base_size 2.5，R1015），此处补块内路径。
    // css-sizing-4：「aspect-ratio does not apply to internal table boxes」——table 内部盒
    //（cell/row/row-group/caption/column 等）排除（table-element-001：td height:50px +
    // ratio 4/1 不 transferred 200px 宽红条）。
    let own_ar = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .filter(|s| {
            // R3794：intrinsic 尺寸关键字（min/max/fit-content）与 auto 同为 content-based
            // 宽——transferred 同样适用（intrinsic-size-014：`width:min-content; height:100px;
            // aspect-ratio:1/1` 子对 min-content 父应贡献 100px，旧 gate 仅认 Auto → 子测 0
            // → 父满宽）。converter 把关键字映射 length(0)，非 transferred 不可。
            // R5040：% 宽在固有测量语境必为循环（本函数仅 shrink-to-fit 链调用，CB 即被测
            // 父）→ 按 auto 处理，transferred 适用（grid-aspect-ratio-025：min-content 网格
            // 内 `width:50%; height:100px; aspect-ratio:1/1` item 应 transferred 100，旧测 0
            // → 容器塌 0 宽）。
            matches!(
                s.width,
                LengthValue::Auto
                    | LengthValue::MinContent
                    | LengthValue::MaxContent
                    | LengthValue::FitContent(_)
                    | LengthValue::Percentage(_)
            ) && !matches!(
                s.display,
                DisplayValue::TableRow
                    | DisplayValue::TableRowGroup
                    | DisplayValue::TableHeaderGroup
                    | DisplayValue::TableFooterGroup
                    | DisplayValue::TableCell
                    | DisplayValue::TableCaption
                    | DisplayValue::TableColumn
                    | DisplayValue::TableColumnGroup
            )
        })
        .and_then(|s| s.aspect_ratio.filter(|&r| r > 0.0))
        .and_then(|ratio| {
            let s = box_node.node_id.and_then(|id| styles.get(&id));
            // R3794：definite main 来源扩百分比 height——CSS height:100% 在第一趟 taffy 已对
            // definite-CB 链解析到 LayoutBox.height（border-box）。intrinsic-size-006/008：
            // `height:100%; aspect-ratio:1/1` 叶盒 CSS 解析 None → 旧测 0 → min-content 父
            // 满宽。R1018 flex_row 容器 cross 同款回退（读第一趟 border-box 减 frame）。
            // 仅叶盒回退：有 in-flow 子时子内容才决定 main（上方 children_inner 分支），
            // 避免把 taffy 对整棵子树解析的 border-box 高误当 transferred 源。
            let main = s.and_then(|s| resolve_intrinsic_real_length(&s.height, s)).or_else(|| {
                if !has_in_flow_child {
                    let vframe =
                        box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
                    let resolved = (box_node.height - vframe).max(0.0);
                    (resolved > 0.5).then_some(resolved)
                } else {
                    None
                }
            });
            main.map(|main| aspect_ratio_transferred_width(s.unwrap(), box_node, main, ratio))
        })
        .unwrap_or(0.0);
    let inner = if !has_in_flow_child {
        // 叶盒：显式宽或文本内容宽（Round C）。纯文本 item（无 LayoutBox 子元素）
        // 之前测 0，现按 DOM 文本内容度量。取 max 避免显式宽被文本低估。
        let text_w = box_node
            .node_id
            .map_or(0.0, |id| text_content_max_width(id, doc, styles));
        own_explicit.max(text_w)
    } else {
        // R4389：混合内容（裸文本 + inline 元素子）的**裸文本段**贡献——子循环只累计
        // 元素子（R1479 递归/outer_w），div「XYZ <ruby>..</ruby> XYZ」型两侧 XYZ 裸文本
        // 此前丢失（ruby-intrinsic-isize-002 1.23→0.14 翻绿实证）。与 R4355 的整子树
        // walk 不同：**跳过已入 box_children 的元素子树**（其文本已经 R1479 计入——
        // 整树 walk 双计，line-break 族 −16 实证），仅累计裸文本段之和。
        // **opt-in（`ZW_MIXED_BARE_TEXT=1`）**：默认关。R4401 门收窄重估（R4393 重入条件
        // 清偿后实测）：三态门（未设=ruby 容器自动开放）在加固后的交错 walk 上实测
        // corpus 14893 净 −1（isize-002 +1 / improper-annotation −1 / intra-base −1）——
        // 劣化面已不在 walker（=R4033 纯空盒再拉伸 + ruby IFC narrow-width 分化两个深
        // ruby 域），门收窄继续挂账至该两案清偿。
        // R4395 交错 walk 切换：flag-on 时
        // 改用 R4355 `dom_inline_text_max_width`（ruby base 经 ruby 分支递归计入、rt/rp/
        // rtc display:none 任意深度跳过、br 分段取最宽段、R4357 注音 extra opt-in 参与；
        // 与上方 Inline 子 frame-only 臂配对=block_max_content_width 同款分工）——R4389
        // bare walker 只计裸文本段、ruby base 完全缺席（ruby 盒→R1479→anon→0），inline-block
        // shrink 对 ruby 页竖排坍塌（nested-ruby-pairing-001 双页 PNG 实证）即此缺口。
        // 默认关：bare-text 计入改变 shrink 决策的阈值骑墙带（R4394 A/B −10 记录），
        // 重估默认待本轮 flag-on 全量 A/B。
        // **definite width 守卫**：own_explicit>0（width 非 auto 的定值盒）时 walk 文本
        // 不再叠加——定值盒 max-content = 自身声明宽，内容在盒内折行不外延（line-break
        // normal-014 族 p.test{width:10.2em} + span 文本：无守卫时 163+224 双计，
        // wrapper shrink 被 walk 文本撑爆实证）。bare walker 时代靠 boxed-skip 免疫
        // （span 文本不经 walker），交错 walk 计全部文本后须显式守卫。
        let dom_text = if mixed_walk_on && own_explicit <= 0.5 {
            dom_inline_text_max_width(box_node, doc, styles)
        } else {
            0.0
        };
        // R4407：行感知合成——dom_text 是本容器 **inline 行**的文本宽（walk 分段取 max），
        // 与 block_max（块级子各自成行）是**不同行**，取 max 而非相加；与 inline_sum
        //（inline 子 frame/原子盒外宽）同行相加 ✓。dom_text==0（flag-off / 定值盒守卫）
        // 时回落 children_inner 逐字节不变。旧 `children_inner.max(own_explicit) + dom_text`
        // 在含块级子的容器上把块行宽与文本行宽跨行相加（margin-collapse-101/105：td 链
        // 66.7 = 2×33.3 实证）。
        if dom_text > 0.0 {
            block_max
                .max(inline_sum + dom_text)
                .max(float_contribution + inline_sum)
                .max(own_explicit)
        } else {
            children_inner.max(own_explicit)
        }
    }
    .max(own_ar);

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
/// R4015：塌 0 的 abspos replaced 叶的 max-content 宽（css-sizing-3 default object size）。
///
/// svg（R4000 三件套谓词：no-ratio 无宽 → 300；% 宽 → 300；ratio-only → 0；attr/CSS abs
/// 宽走既有路径不塌 0 不会进到这里）。非 svg replaced（embed/object/applet 等）回退
/// `block_max_content_width`（内部递归）。仅用于 R3929 abspos shrink-to-fit 的
/// replaced-collapse 例外臂。
pub(crate) fn abspos_replaced_max_content(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    if let Some(id) = box_node.node_id
        && let Some(node) = doc.get(id)
        && let zero_dom::NodeKind::Element(elem) = &node.kind
        && elem.local_name() == "svg"
        && let Some(style) = box_node.node_id.and_then(|nid| styles.get(&nid))
        && let Some(contribution) = crate::svg_default_size::svg_max_content_contribution(elem, style)
    {
        let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
        return (contribution + frame).max(0.0);
    }
    block_max_content_width(box_node, doc, styles)
}

pub(crate) fn block_max_content_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    block_intrinsic_width(box_node, doc, styles, false)
}

/// R5036（css-sizing-3 §min-content）：block 容器的 min-content 内容宽——软换行
/// 机会全部使用。与 max-content 共用 [`block_intrinsic_width`] 结构，语义分叉：
/// inline 级子（含原子盒/float 行）由「同行求和」改「独立断点取 max」（每个原子盒
/// /不可断词可独占一行）；文本经 `dom_inline_text_min_width`（词级）；叶文本经
/// `text_content_min_width`。block/flex/grid 子仍取 max（各自成行，两模式同；
/// flex/grid 的真 min-content 独立子问题，沿用 max 口径=过宽安全向，FIXME）。
pub(crate) fn block_min_content_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    block_intrinsic_width(box_node, doc, styles, true)
}

/// R5037：content-only min-content——不含自身 definite width 的 own_explicit 抬升
///（fit-content() 公式的 W_min 口径：内在尺寸按内容基测量，specified 宽在使用侧钳制，
/// 不进公式）。仅 R5037 贡献钳制消费。
pub(crate) fn block_min_content_width_content_only(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    block_intrinsic_width_ex(box_node, doc, styles, true, false)
}

/// R5037（css-sizing-3 #intrinsic-contribution + #valdef-width-fit-content-length-percentage）：
/// fit-content 尺寸参与 min/max-content 贡献传播——block 子对父固有宽的贡献须被其自身
/// fit-content 宽钳制（fit-content-length-percentage-011..016 取证：width:fit-content(100px)
/// 子贡献 = 公式值 100 而非内容 max 124；width:50px + min-width:fit-content(100px) 子贡献 =
/// max(50, 100) = 100；width:200px + max-width:fit-content(100px) 子贡献 = min(200, 100) =
/// 100）。仅 width/min/max-width 含 `fit-content()` 时触发（blast radius 窄）；content
/// 关键字（min-content/max-content min/max-width）仍归 R4151 kw_aware 路径不重复处理。
///
/// base（specified 宽主导，R4149 同口径）：definite width → 该宽 + frame；
/// width:fit-content(definite) → 公式值；width:fit-content(%) / auto / content 关键字 →
/// 内容 intrinsic（min 模式 = 真 min-content，max 模式 = raw max-content）。
///
/// 循环百分比语境（% arg 固有测量中无 CB 可依；WPT 011/012/013 案注释明示规则）：
/// width:fit-content(%) → 按 auto；min-width:fit-content(%) → floor 到 min-content
///（fit-content(0) 公式退化 = W_min）；max-width:fit-content(%) → cap 到 max-content
///（= W_max，无额外收缩）。
///
/// kill-switch `ZW_FIT_CONTRIBUTION=0` 回退。
fn clamp_fit_content_child_contribution(
    child: &LayoutBox,
    cs: &ComputedStyle,
    raw: f32,
    min_mode: bool,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    if std::env::var("ZW_FIT_CONTRIBUTION").as_deref() == Ok("0") {
        return raw;
    }
    let width_fit = matches!(cs.width, LengthValue::FitContent(_));
    let min_fit = matches!(cs.min_width, LengthValue::FitContent(_));
    let max_fit = matches!(cs.max_width, LengthValue::FitContent(_));
    if !width_fit && !min_fit && !max_fit {
        return raw;
    }
    let frame = child.padding_left + child.padding_right + child.border_left + child.border_right;
    // 子盒自身 content intrinsic（border-box 口径同 raw）。fit-content() 公式的 W_max/W_min
    // 是**内容基**内在尺寸（LayoutNG 同语义：specified 宽在使用侧钳制，不进公式）——子带
    // definite width 时 raw/max 测量含 own_explicit 抬升（max(内容, 定宽)），须换 content-only
    // 测量；无定宽时两者同值，直接复用 raw 免重测。W_min 缺测/塌 0 → None（公式退化不塌）。
    let has_definite_width = resolve_intrinsic_real_length(&cs.width, cs).is_some();
    let w_max = if has_definite_width {
        box_content_max_width_content_only(child, doc, styles)
    } else {
        raw
    };
    let w_min_measured = if has_definite_width {
        block_min_content_width_content_only(child, doc, styles)
    } else {
        block_min_content_width(child, doc, styles)
    };
    let w_min = (w_min_measured > 0.5).then_some(w_min_measured);
    // fit-content(arg) 公式：min(W_max, max(W_min, arg))；W_min 缺测退化 min(W_max, arg)
    //（R3925 旧行为同款兼容）。
    let fit_formula = |arg: f32| -> f32 { w_max.min(w_min.map_or(arg, |min_c| min_c.max(arg))) };
    // fit-content arg：definite → Some(Some(len))；percent（循环语境）→ Some(None)。
    let fit_arg = |v: &LengthValue| -> Option<Option<f32>> {
        match v {
            LengthValue::FitContent(inner) => Some(match inner.as_ref() {
                LengthValue::Percentage(_) => None,
                other => resolve_intrinsic_real_length(other, cs),
            }),
            _ => None,
        }
    };
    let base = if let Some(w) = resolve_intrinsic_real_length(&cs.width, cs) {
        w + frame
    } else if width_fit {
        match fit_arg(&cs.width) {
            Some(Some(arg)) => fit_formula(arg),
            // fit-content(%)（循环 → auto 语义）及其余 content-based 宽。
            _ => {
                if min_mode {
                    w_min.unwrap_or(raw)
                } else {
                    raw
                }
            }
        }
    } else if min_mode {
        w_min.unwrap_or(raw)
    } else {
        raw
    };
    // floor/cap：definite min/max-width → len + frame；fit-content(definite) → 公式值；
    // fit-content(%) → 循环语境退化（floor = W_min / cap = W_max）；content 关键字 →
    // 不在此钳（R4151 域）。
    let limit = |v: &LengthValue, is_fit: bool, cyclic_value: Option<f32>| -> Option<f32> {
        if let Some(len) = resolve_intrinsic_real_length(v, cs) {
            return Some(len + frame);
        }
        if !is_fit {
            return None;
        }
        match fit_arg(v) {
            Some(Some(arg)) => Some(fit_formula(arg)),
            Some(None) => cyclic_value,
            None => None,
        }
    };
    let floor = limit(&cs.min_width, min_fit, w_min);
    let cap = limit(&cs.max_width, max_fit, Some(w_max));
    base.max(floor.unwrap_or(0.0)).min(cap.unwrap_or(f32::MAX))
}

fn block_intrinsic_width(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    min_mode: bool,
) -> f32 {
    block_intrinsic_width_ex(box_node, doc, styles, min_mode, true)
}

/// `include_own_width = false`（R5037 content-only 口径）时不把自身 definite CSS 宽计入
/// intrinsic（`own_explicit` 的 width 臂置 0）——containment CIS 替代与 aspect-ratio
/// transferred 仍保留（同属内容基内在尺寸语义）。
fn block_intrinsic_width_ex(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    min_mode: bool,
    include_own_width: bool,
) -> f32 {
    let mut inline_sum = 0.0f32;
    let mut block_max = 0.0f32;
    // R1431 L3②：spanner-aware multicol intrinsic sizing 须区分 spanner / 非 spanner block 子。
    // `nonspanner_block_max` = 非 column-span:all block 子 intrinsic max（multicol 列内容宽）；
    // `spanner_max` = column-span:all 子 intrinsic max（spanner 跨全宽驱动宽度）。`block_max`
    //（含 spanner）保留供非 multicol `children_inner`（spanner 对普通 block 容器即普通子）。
    let mut nonspanner_block_max = 0.0f32;
    let mut spanner_max = 0.0f32;
    // R4397（css-sizing-3 §max-content；outline-028 取证）：float 子横向叠加不进
    // block_max（max 语义）——max-content 语境下 float 与后续 inline 内容**同行并排**，
    // 容器宽 = float 外宽和 + inline 内容宽（与 block 子「各自成行取 max」正交）。
    // 旧实现 float 落 block 分支与 inline 取 max → outline-028 的 max-content 容器
    // 25 测 15（float 15 吞掉 span 10），outline 不含 span 盒 1.656% 实证。
    // clear 强制换行：cleared float 开新行（bidi-breaking-001 的 .set{clear:both}
    // 堆叠行形态实证——无 clear 行模型时行内求和把堆叠宽误加），float 贡献 =
    // 各行和的最大值。
    let mut float_row = 0.0f32;
    let mut float_max = 0.0f32;
    let mut has_in_flow_child = false;

    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        has_in_flow_child = true;
        // R4136：固有测量中 Calc(P%±Npx) margin 只取 px 部分（% → 0）；普通 margin 照用。
        let (ml, mr) = intrinsic_margin_contribution(child, styles);
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
            // R4000（css-sizing-3 §intrinsic-sizes + csswg #1801581）：inline `<svg>`
            // 的 max-content 贡献不取已布局宽度（taffy 对 %/auto 无 CB 时给 0/塌缩）
            // 而按 default object size 规则：viewBox/ar-only → 0；width % 或无来源 →
            // 300。attr/CSS abs 宽仍走既有 outer_w。kill-switch `ZW_SVG_DEFAULT_SIZE=0`。
            if let Some(id) = child.node_id
                && let Some(node) = doc.get(id)
                && let zero_dom::NodeKind::Element(elem) = &node.kind
                && elem.local_name() == "svg"
                && let Some(style) = child_style
                && let Some(contribution) = crate::svg_default_size::svg_max_content_contribution(elem, style)
            {
                if min_mode {
                    inline_sum = inline_sum.max((contribution + ml + mr).max(0.0));
                } else {
                    inline_sum += (contribution + ml + mr).max(0.0);
                }
                continue;
            }
            // R4355（css-sizing-3 §max-content；ruby-overhang-spaces-002 取证）：intrinsic
            // 测量趟裸文本不生成 LayoutBox、纯 inline 子在 taffy 前宽为 0——旧实现对
            // 「文本 + <span>/<ruby> + 文本」容器 max-content 塌缩（实测 text+ruby+text
            // div 测 20px 应 ~100px，整行逐字竖排坍塌）。纯 inline（span/ruby/em 等）改为
            // 仅计 frame（padding/border + margin），其文本经循环后 dom_inline_text_max_width
            // 统一计入 DOM 直读（ruby 的 base 文本收进父 IFC、LayoutBox 树仅空 anon 盒，
            // 只有 DOM 直读可见；rt/rp display:none 子树在 walk 中跳过，rt 宽度绝不计入）。
            // 原子/盒模型 inline（inline-block 族/replaced）维持既有 child.width 口径
            //（replaced 的 attr/AR 宽第一趟已解析进 child.width；DOM 递归对无文本原子盒
            // 测 0 会塌——intrinsic-percent-replaced-001/012/013/dynamic-004 实证回退）。
            let is_plain_inline = std::env::var("ZW_INLINE_DOM_INTRINSIC").as_deref() != Ok("0")
                && child_style.is_some_and(|s| matches!(s.display, DisplayValue::Inline));
            if is_plain_inline {
                let frame = child.padding_left + child.padding_right + child.border_left + child.border_right;
                // FIXME(R5036)：min 模式透明 inline 的 frame 未与其内容词并合（近似偏窄），
                // fit-content 族案面均为无 frame inline，frame 并合后续 slice。
                if min_mode {
                    inline_sum = inline_sum.max((frame + ml + mr).max(0.0));
                } else {
                    inline_sum += (frame + ml + mr).max(0.0);
                }
            } else if min_mode {
                // 原子盒独立断点（UAX14：atomic inline 前后可断）——取 max 非求和。
                inline_sum = inline_sum.max((child.width + ml + mr).max(0.0));
            } else {
                inline_sum += (child.width + ml + mr).max(0.0);
            }
            continue;
        }
        let is_spanner = child_style.is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All));
        // R4008（css-sizing-4 §intrinsic-size-override）：contain:size 子的固有尺寸由
        // contain-intrinsic-size 替代（内容不参与）——max-content 计入 CIS 宽（002：CIS
        // 111 宽子 → max-content 父 113 应 4）。CIS 缺失/非 definite → 0（同旧行为）。
        if let Some(cs) = child_style
            && cs.contain.has_size()
            && let Some(cis_st) = child.node_id.and_then(|cid| styles.get(&cid))
            && let Some(cis_w) = cs
                .contain_intrinsic_width
                .as_ref()
                .and_then(|v| resolve_intrinsic_real_length(v, cis_st))
        {
            block_max = block_max.max(cis_w + ml + mr);
            continue;
        }
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
                    grid_intrinsic_width_ex(child, doc, styles, min_mode).unwrap_or(0.0)
                }
                _ => box_content_max_width(child, doc, styles),
            })
            .unwrap_or_else(|| box_content_max_width(child, doc, styles));
        // R5037：fit-content 尺寸参与贡献传播（仅 block 子——flex/grid 的主轴贡献由专用
        // intrinsic 自算，关键词域已有独立接线，不在此交叉钳制）。
        let is_flex_grid_child = child_style.is_some_and(|s| {
            matches!(
                s.display,
                DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
            )
        });
        let child_intrinsic = if is_flex_grid_child {
            child_intrinsic
        } else {
            child_style.map_or(child_intrinsic, |cs| {
                clamp_fit_content_child_contribution(child, cs, child_intrinsic, min_mode, doc, styles)
            })
        };
        let with_margins = child_intrinsic + ml + mr;
        // R4397：float 子横向叠加 + clear 换行（见 loop 头注）；与 spanner 记账正交
        //（column-span:all float 非常规形态，不参与 spanner_max）。
        if !matches!(child.float, FloatValue::None) {
            if !matches!(child.clear, ClearValue::None) {
                float_max = float_max.max(float_row);
                float_row = 0.0;
            }
            if min_mode {
                float_row = float_row.max(with_margins.max(0.0));
            } else {
                float_row += with_margins.max(0.0);
            }
            continue;
        }
        block_max = block_max.max(with_margins);
        if is_spanner {
            spanner_max = spanner_max.max(with_margins);
        } else {
            nonspanner_block_max = nonspanner_block_max.max(with_margins);
        }
    }

    // R4355：裸文本与纯 inline 后代（span/ruby 等）的文本固有宽（见 inline 分支注）。
    // 杀开关 `ZW_INLINE_DOM_INTRINSIC=0` 整体回退（含 inline 分支的 DOM 递归）。
    if std::env::var("ZW_INLINE_DOM_INTRINSIC").as_deref() != Ok("0") {
        if min_mode {
            // min 模式文本与原子盒同为独立断点——取 max 非求和。
            inline_sum = inline_sum.max(dom_inline_text_min_width(box_node, doc, styles));
        } else {
            inline_sum += dom_inline_text_max_width(box_node, doc, styles);
        }
    }

    // R4397：float 行（float 外宽和 + inline 内容同行并排）与 block 行（max）正交取 max。
    let float_contribution = float_max.max(float_row);
    let children_inner = inline_sum.max(block_max).max(float_contribution + inline_sum);

    // leaf 回退同 box_content_max_width：显式 Px width 或文本内容宽。
    // R4008：自身 contain:size + content-based width 关键字 → CIS 替代（同 box_content_max_width）。
    let own_style = box_node.node_id.and_then(|id| styles.get(&id));
    let own_cis = own_style
        .filter(|s| s.contain.has_size())
        .filter(|s| {
            matches!(
                s.width,
                LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_)
            )
        })
        .and_then(|s| {
            s.contain_intrinsic_width
                .as_ref()
                .and_then(|v| resolve_intrinsic_real_length(v, s))
        })
        .unwrap_or(0.0);
    let own_explicit = own_style
        .and_then(|s| resolve_intrinsic_real_length(&s.width, s))
        .filter(|_| include_own_width)
        .unwrap_or(0.0)
        .max(own_cis);
    let inner = if !has_in_flow_child {
        let text_w = box_node.node_id.map_or(0.0, |id| {
            if min_mode {
                text_content_min_width(id, doc, styles)
            } else {
                text_content_max_width(id, doc, styles)
            }
        });
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
        zero_style_system::ColumnWidthComputedValue::Length(width) => resolve_intrinsic_real_length(width, s),
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
                .and_then(|s| resolve_intrinsic_real_length(&s.column_gap, s))
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
    let is_ahem = style.is_some_and(|s| {
        s.font_family
            .iter()
            .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"))
    });
    // R1747：`<br>` 是强制换行（CSS css-sizing-3：forced break 产生独立 line，max-content
    // 取最宽 line 而非全文本累加）。旧实现用 `doc.text_content`（递归扁平化，br 折成空）把
    // "short<br>much longer line<br>mid" 测成单行 201.6px（应 max-line 131.2px），致 inline-block
    // / float / leaf block shrink-to-fit 过宽。改为递归遍历 DOM 子树，按 `<br>` 切段，取最宽段。
    let white_space = styles
        .get(&node_id)
        .map(|s| s.white_space.clone())
        .unwrap_or(WhiteSpaceValue::Normal);
    let font_id = intrinsic_font_id(styles.get(&node_id));
    let tab = intrinsic_tab_metrics(styles.get(&node_id), font_size, is_ahem, font_id);
    let mut segments: Vec<f32> = vec![0.0];
    let mut state = DomWalkState {
        pending_space: false,
        line_has_content: false,
        font_size,
        is_ahem,
        font_id,
        min_mode: false,
    };
    text_max_width_walk(
        node_id,
        doc,
        font_size,
        is_ahem,
        &white_space,
        Some(styles),
        font_id,
        tab,
        &mut segments,
        &mut state,
        per_font_intrinsic_on(),
        // 既有消费方（float 垂直臂/legend/leaf intrinsic/单测）维持旧口径——原子 gate
        // 仅在「Σ 侧同时计盒宽」的 float 纯文本臂（text_content_max_width_scoped）启用。
        false,
    );
    segments.into_iter().fold(0.0f32, f32::max)
}

/// R4921 scoped 变体：`stop_at_definite_atomic = true` 时对**定宽**原子 inline 子停止
/// 文本递归。仅供 float 纯文本收缩臂（float_positioning shrink_pure_text_floats）使用
/// ——该臂的 Σ 侧 `inline_children_non_text_width` 同时计原子盒外尺寸，walk 再递归内文
/// 即双计（R4920 组合新引入过测，实证 float 180 vs 规范真值 100）。其余 `text_content_
/// max_width` 消费方（legend/leaf intrinsic/垂直 float 臂）无 Σ 配对侧，维持旧 walk
/// 口径（其原子形态的既有欠测/过测与本修复无关，不扩波及面）。
pub(crate) fn text_content_max_width_scoped(
    node_id: NodeId,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    stop_at_definite_atomic: bool,
) -> f32 {
    let style = styles.get(&node_id);
    let (font_size, _line_height) = crate::inline::resolve_font_metrics(style);
    let is_ahem = style.is_some_and(|s| {
        s.font_family
            .iter()
            .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"))
    });
    let white_space = styles
        .get(&node_id)
        .map(|s| s.white_space.clone())
        .unwrap_or(WhiteSpaceValue::Normal);
    let font_id = intrinsic_font_id(styles.get(&node_id));
    let tab = intrinsic_tab_metrics(styles.get(&node_id), font_size, is_ahem, font_id);
    let mut segments: Vec<f32> = vec![0.0];
    let mut state = DomWalkState {
        pending_space: false,
        line_has_content: false,
        font_size,
        is_ahem,
        font_id,
        min_mode: false,
    };
    text_max_width_walk(
        node_id,
        doc,
        font_size,
        is_ahem,
        &white_space,
        Some(styles),
        font_id,
        tab,
        &mut segments,
        &mut state,
        per_font_intrinsic_on(),
        stop_at_definite_atomic,
    );
    segments.into_iter().fold(0.0f32, f32::max)
}

/// R5036（css-sizing-3 §min-content）：[`text_content_max_width`] 的 min-content 变体
/// ——折叠空格为软换行断点、文本按不可断词取最宽（state.min_mode 驱动，见
/// walk_collapsible_text）；pre 系保留空白无断点 = max-content。原子 gate 口径与
/// scoped max 版一致（stop_at_definite_atomic 恒 false——本函数无 Σ 配对侧）。
pub(crate) fn text_content_min_width(node_id: NodeId, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) -> f32 {
    let style = styles.get(&node_id);
    let (font_size, _line_height) = crate::inline::resolve_font_metrics(style);
    let is_ahem = style.is_some_and(|s| {
        s.font_family
            .iter()
            .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"))
    });
    let white_space = styles
        .get(&node_id)
        .map(|s| s.white_space.clone())
        .unwrap_or(WhiteSpaceValue::Normal);
    let font_id = intrinsic_font_id(styles.get(&node_id));
    let tab = intrinsic_tab_metrics(styles.get(&node_id), font_size, is_ahem, font_id);
    let mut segments: Vec<f32> = vec![0.0];
    let mut state = DomWalkState {
        pending_space: false,
        line_has_content: false,
        font_size,
        is_ahem,
        font_id,
        min_mode: true,
    };
    text_max_width_walk(
        node_id,
        doc,
        font_size,
        is_ahem,
        &white_space,
        Some(styles),
        font_id,
        tab,
        &mut segments,
        &mut state,
        per_font_intrinsic_on(),
        false,
    );
    segments.into_iter().fold(0.0f32, f32::max)
}

/// R4919：per-element 字体语境 intrinsic 测量开关（default-on；`ZW_INTRINSIC_PERFONT=0`
/// 回退「整棵子树沿用容器 font」旧行为，供回归归因 A/B）。
fn per_font_intrinsic_on() -> bool {
    std::env::var("ZW_INTRINSIC_PERFONT").as_deref() != Ok("0")
}

/// R4920（css-sizing-3 §max-content + CSS2 §10.3.5 shrink-to-fit preferred width）：
/// float 纯文本臂的**非文本**子贡献——定宽原子 inline 子（inline-block 族，CSS width
/// definite）贡献盒宽 + margin-box；纯 inline 子贡献 frame + margin（其文本由
/// text_content_max_width 的 DOM walk 计入，frame/margin 在此单边入账勿双计）。
/// 走 **DOM 直子**（与 measure_text_content 的 ib_sizes 同口径——float 的 LayoutBox
/// children 不含折叠进 IFC 的 inline 子）。auto 宽原子子不计宽（taffy 首趟拉伸伪影
/// R4032 语境；under-measure 安全方向，真值按内容递归测独立子问题）。
/// baidu「换一换」float（a.hot-refresh）内 i.c-icon（inline-block width:16px）+
/// span margin-left:2px 漏计入 → float 收缩到 42 < 单行真值 60 → 内层 IFC 把
/// 「一换」折到第二行竖排（实证）。
pub(crate) fn inline_children_non_text_width(
    dom_id: NodeId,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    let mut sum = 0.0f32;
    for child in doc.child_nodes(dom_id) {
        let Some(cs) = styles.get(&child) else {
            continue;
        };
        let resolve = |v: &LengthValue| resolve_intrinsic_real_length(v, cs).unwrap_or(0.0);
        match cs.display {
            DisplayValue::InlineBlock
            // R4921 注记：InlineFlex/InlineGrid/InlineTable 三臂在唯一调用路径（float
            // 纯文本臂）不可达——该三 display 经 engine.rs（is_block_level 判定）使
            // has_block_or_replaced 早退。保留为防御口径（调用方扩展时与 InlineBlock
            // 同语义），与 dom walk 原子 gate 四族对齐，非死代码删除对象。
            | DisplayValue::InlineFlex
            | DisplayValue::InlineGrid
            | DisplayValue::InlineTable => {
                let w = resolve(&cs.width);
                let ml = resolve(&cs.margin_left);
                let mr = resolve(&cs.margin_right);
                sum += (w + ml + mr).max(0.0);
            }
            DisplayValue::Inline => {
                // border-style None/Hidden 不渲染边框——UA initial border-width=medium(3px)
                // 恒在场（inline_finalization border_w 同口径），须按 style 门控。
                let border_w = |w: &LengthValue, s: &zero_style_system::property::types::BorderStyleValue| -> f32 {
                    match s {
                        zero_style_system::property::types::BorderStyleValue::None
                        | zero_style_system::property::types::BorderStyleValue::Hidden => 0.0,
                        _ => resolve(w),
                    }
                };
                let frame = resolve(&cs.padding_left)
                    + resolve(&cs.padding_right)
                    + border_w(&cs.border_left_width, &cs.border_left_style)
                    + border_w(&cs.border_right_width, &cs.border_right_style);
                let ml = resolve(&cs.margin_left);
                let mr = resolve(&cs.margin_right);
                sum += (frame + ml + mr).max(0.0);
            }
            _ => {}
        }
    }
    sum
}

/// R4416（CSS Text 3 §4.1.3）：tab 度量对 (unit, space_advance)——unit = 下一 tab stop
/// 的名义距离（Number(n) = n×space，Length = max(px, space)），与行内 break_lines 的
/// tab_unit 公式同源（相同 max 钳制）；preserve 臂 intrinsic 测量与 ruby 悬挂容量共用。
/// 旧实现 preserve 文本的 '\t' 走字体 hmtx advance（DejaVu ≈ 0.5em），与行内 stop
/// 推进分裂（tab-size:1ic 行内 20 vs intrinsic 10，overhang-spaces-006 三套度量分裂）。
fn intrinsic_tab_metrics(
    style: Option<&ComputedStyle>,
    font_size: f32,
    is_ahem: bool,
    font_id: Option<u32>,
) -> (f32, f32) {
    let space = measure_intrinsic_char(' ', font_id, font_size, is_ahem);
    let unit = match style.map(|s| &s.tab_size) {
        Some(zero_style_system::TabSizeValue::Number(n)) => (*n as f32).max(1.0) * space,
        Some(zero_style_system::TabSizeValue::Length(v)) => {
            resolve_intrinsic_real_length(v, style.expect("Length 臂 style 在场"))
                .unwrap_or(8.0 * space)
                .max(space)
        }
        None => 8.0 * space,
    };
    (unit, space)
}

/// R1747：测量 `node_id` 自身（文本节点直接量；元素递归子树），把文本字符宽累入当前段，
/// 遇 `<br>` 元素开新段（嵌套 br 同样切段）。`segments` 每项 = 一段宽。
/// R1748：改为处理 node 自身（text/br/element 三态），使 fragment_node_ids（可能是文本
/// 节点）亦可用此函数。
/// R4223（CSS Text 3 §4.1.3 + css-sizing-3）：保留换行模式（pre/pre-wrap/break-spaces/
/// pre-line）下 `\n` 与 `<br>` 同为强制换行——按行切段取最宽行；pre 系行内空格保留逐字
/// 计宽，pre-line 行内空格仍折叠。旧实现一律 `collapse_whitespace` 把 `\n` 折成空格测成
/// 单长行（text-group-align ref 页 `.group{inline-size:min-content}` 全文本测 262px、
/// 应最宽行 122px → 组盒溢出容器、margin-inline:auto 居中失效）。`styles` 供元素子查
/// 自身 white-space 覆盖（None = 沿用继承值，fragment 语境无样式表可用）。
/// 文本内容按 white-space 规则累入 segments（collapse 模式单段累加；保留换行/ pre-line
/// 模式按 `\n` 切段）。text_max_width_walk 与 dom_inline_text_walk 共用。
fn accumulate_text_width(
    content: &str,
    white_space: &WhiteSpaceValue,
    font_size: f32,
    is_ahem: bool,
    font_id: Option<u32>,
    tab: (f32, f32),
    segments: &mut Vec<f32>,
) {
    let preserve_spaces = matches!(
        white_space,
        WhiteSpaceValue::Pre | WhiteSpaceValue::PreWrap | WhiteSpaceValue::BreakSpaces
    );
    let forced_newline = preserve_spaces || matches!(white_space, WhiteSpaceValue::PreLine);
    if !forced_newline {
        let collapsed = crate::inline::collapse_whitespace(content);
        if !collapsed.is_empty() {
            let w: f32 = collapsed
                .chars()
                .map(|ch| measure_intrinsic_char(ch, font_id, font_size, is_ahem))
                .sum();
            *segments.last_mut().expect("segments 非空") += w;
        }
    } else {
        for (i, line) in content.split('\n').enumerate() {
            if i > 0 {
                segments.push(0.0);
            }
            if line.is_empty() {
                continue;
            }
            let measured = if preserve_spaces {
                line.to_string()
            } else {
                crate::inline::collapse_whitespace(line)
            };
            if measured.is_empty() {
                continue;
            }
            // R4416：preserve 臂 '\t' 按 tab stop 推进（CSS Text 3 §4.1.3——下一
            // tab_size 倍数、最小一个空格 advance；段宽即行内位置，与 break_lines
            // 同公式）。折叠臂不经过此处（\t 已折叠为空格）。
            let mut pos = *segments.last_mut().expect("segments 非空");
            for ch in measured.chars() {
                let w = if ch == '\t' && tab.0 > 0.0 {
                    (tab.0 - pos % tab.0).max(tab.1)
                } else {
                    measure_intrinsic_char(ch, font_id, font_size, is_ahem)
                };
                pos += w;
            }
            *segments.last_mut().expect("segments 非空") = pos;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn text_max_width_walk(
    node_id: NodeId,
    doc: &Document,
    font_size: f32,
    is_ahem: bool,
    white_space: &WhiteSpaceValue,
    styles: Option<&HashMap<NodeId, ComputedStyle>>,
    font_id: Option<u32>,
    tab: (f32, f32),
    segments: &mut Vec<f32>,
    state: &mut DomWalkState,
    per_font: bool,
    stop_at_definite_atomic: bool,
) {
    let Some(node) = doc.get(node_id) else { return };
    match &node.kind {
        zero_dom::NodeKind::Text(t) => {
            let preserve_spaces = matches!(
                white_space,
                WhiteSpaceValue::Pre | WhiteSpaceValue::PreWrap | WhiteSpaceValue::BreakSpaces
            );
            if preserve_spaces {
                // 保留换行模式：pre 系空白保留、无行缘折叠丢弃语义，维持 accumulate 逐字计宽。
                accumulate_text_width(&t.content, white_space, font_size, is_ahem, font_id, tab, segments);
                state.line_has_content = true;
                return;
            }
            // R4397（css-text-3 §white-space-phase-1）：pre-line 保留 \n 为强制换行——按 \n
            // 切段（段前 pending 随行尾丢弃），段内空白照常折叠。与 dom_inline_text_walk 同款。
            if matches!(white_space, WhiteSpaceValue::PreLine) && t.content.contains('\n') {
                for part in t.content.split('\n') {
                    walk_collapsible_text(part, segments, state);
                    // \n 强制断：行尾 pending 丢弃，开新段。
                    state.pending_space = false;
                    state.line_has_content = false;
                    segments.push(0.0);
                }
                return;
            }
            // R5034（css-text-3 §3 white-space processing phase 1/2 + css-sizing-3）：折叠语境
            // （normal/nowrap）行缘可折叠空白丢弃——行首缩进/行尾换行折叠出的空格不计入
            // max-content（nowrap 只禁软换行）。旧实现整段计宽（accumulate 折叠臂无行缘语义），
            // 源码跨行排版的容器 intrinsic 被行缘空白吹胀（slice/clone-nowrap-intrinsic-size
            // 四案 test 页 +10px 实证）。与 dom_inline_text_walk 共用 walk_collapsible_text
            // 跨节点折叠状态（pending_space/line_has_content）。
            walk_collapsible_text(&t.content, segments, state);
        }
        zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br") => {
            // 强制换行：行尾待定空白丢弃，开新段（同 dom_inline_text_walk）。
            state.pending_space = false;
            state.line_has_content = false;
            segments.push(0.0);
        }
        zero_dom::NodeKind::Element(_) => {
            for child in doc.child_nodes(node_id) {
                let child_style = styles.and_then(|m| m.get(&child));
                // R4921（css-sizing-3 §max-content：原子 inline 子贡献盒外尺寸，其内文在
                // 原子盒内自行折行不外溢）：**定宽**原子 inline 子停止文本递归——盒宽由
                // float 臂 Σ 侧 inline_children_non_text_width 单边入账，此处再递归内文即
                // 双计（实证：float > inline-block(100px) > 文本 → walk 计内文 80 + Σ 计
                // 盒宽 100 = 180 vs 规范真值 100，R4920 组合新引入的过测）。与 dom walk
                // 的原子 gate（dom_inline_text_walk「replaced / inline-block 族停止…避免
                // 文本双计」）同语义口径；本 walk 折叠臂在文本节点侧整段计宽（含边缘空格），
                // 文本与原子间折叠空格已入账——dom walk flush_space 落点同向。
                // **仅定宽**（width resolve definite）：auto/百分比宽原子不定宽，Σ 侧计 0
                //（欠测安全向，R4032 语境），内文仍由此 walk 近似其 max-content（既有口径
                // 不变）——定宽 gate 才是双计的精确面，gate 全体原子会使 auto 形态欠测
                //（float > inline-block(auto) > 文本收缩到 0 回归）。
                // 仅 float 纯文本臂经 text_content_max_width_scoped 启用（其余消费方无 Σ
                // 配对侧，维持旧口径）；`ZW_INTRINSIC_ATOMIC_GATE=0` 回退（回归归因 A/B）。
                if stop_at_definite_atomic
                    && let Some(cs) = child_style
                    && matches!(
                        cs.display,
                        DisplayValue::InlineBlock
                            | DisplayValue::InlineFlex
                            | DisplayValue::InlineGrid
                            | DisplayValue::InlineTable
                    )
                    && resolve_intrinsic_real_length(&cs.width, cs).is_some()
                {
                    continue;
                }
                let child_ws = child_style
                    .map(|s| s.white_space.clone())
                    .unwrap_or_else(|| white_space.clone());
                // R4919（css-sizing-3 #max-content-inline-size）：max-content 的 inline 文本段
                // 按**该段自身**的 computed font 度量——旧实现整棵子树沿用容器 font_size/
                // is_ahem（font_id 已按子刷新，R4043），嵌套异字号（如 12px 容器 > 13px 锚）
                // 被低估 → shrink-to-fit 盒窄于真实内容 → 内层 IFC 按真实字体二次布局折行
                //（baidu「更多」「换一换」竖排实证）。子无样式条目（styles=None 的 fragment
                // 语境，R4367）沿用父语境不变。`ZW_INTRINSIC_PERFONT=0` 回退旧行为。
                let (child_fs, child_ahem, child_font_id) = match child_style.filter(|_| per_font) {
                    Some(cs) => {
                        let (fs, _) = crate::inline::resolve_font_metrics(Some(cs));
                        (
                            fs,
                            cs.font_family
                                .iter()
                                .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem")),
                            intrinsic_font_id(Some(cs)).or(font_id),
                        )
                    }
                    // R4043 近似沿用：嵌套异字体后代按容器 font_id 计（本域已排除出
                    // stored IFC；原子后代宽由调用方 LayoutBox 分支负责）。
                    None => (
                        font_size,
                        is_ahem,
                        child_style.and_then(|cs| intrinsic_font_id(Some(cs))).or(font_id),
                    ),
                };
                // R4416：tab 度量随 white_space 同点解析（子样式覆盖，否则继承）。
                let child_tab = child_style
                    .map(|cs| intrinsic_tab_metrics(Some(cs), child_fs, child_ahem, child_font_id))
                    .unwrap_or(tab);
                // R4919：per-element 字体语境换入/换出（同 dom_inline_text_walk）——折叠
                // pending 空格的 flush 度量随当前子语境，递归返回恢复父语境。
                let (prev_fs, prev_ahem, prev_fid) = (state.font_size, state.is_ahem, state.font_id);
                state.font_size = child_fs;
                state.is_ahem = child_ahem;
                state.font_id = child_font_id;
                text_max_width_walk(
                    child,
                    doc,
                    child_fs,
                    child_ahem,
                    &child_ws,
                    styles,
                    child_font_id,
                    child_tab,
                    segments,
                    state,
                    per_font,
                    stop_at_definite_atomic,
                );
                state.font_size = prev_fs;
                state.is_ahem = prev_ahem;
                state.font_id = prev_fid;
            }
        }
        _ => {}
    }
}

/// R4355（css-sizing-3 §max-content；ruby-overhang-spaces-002 取证）：混合 inline 内容
/// 容器的「文本部分」max-content 宽。intrinsic 测量趟 inline 子尚未布局（child.width=0；
/// taffy 之后的调用点读到拉伸伪影宽），且裸文本片段不生成 LayoutBox——旧实现对
/// 「文本 + <span>/<ruby> + 文本」容器测得 ~0（实测 div w=20 应 ~100px，整行逐字
/// 竖排坍塌）。DOM 直读容器子树文本：
/// - display:none 子树跳过（**rt/rp**——ruby base 文本收进父 IFC 参与<Ruby>行宽，rt
///   文本绝不计入；annotation 宽度参与行预算 = 后续 slice）；
/// - 块级后代停止（其文本由 block 分支的子 intrinsic 负责，不属本 inline 流）；
/// - replaced / inline-block 族停止（其贡献由调用方 inline 分支的 LayoutBox 递归计，
///   避免文本双计；原子盒无文本）；
/// - 其余 inline 元素（span/ruby/em 等）递归；br 切段取最宽段（R1747 同语义）。
///
/// 字体度量取容器自身（嵌套异字体后代近似，R4043 已把该域排除出 stored IFC）。
fn dom_inline_text_max_width(box_node: &LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) -> f32 {
    let Some(id) = box_node.node_id else { return 0.0 };
    let Some(style) = styles.get(&id) else { return 0.0 };
    let (font_size, _line_height) = crate::inline::resolve_font_metrics(Some(style));
    let is_ahem = style
        .font_family
        .iter()
        .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));
    let white_space = style.white_space.clone();
    let font_id = intrinsic_font_id(Some(style));
    let tab = intrinsic_tab_metrics(Some(style), font_size, is_ahem, font_id);
    let mut segments: Vec<f32> = vec![0.0];
    let mut state = DomWalkState {
        pending_space: false,
        line_has_content: false,
        font_size,
        is_ahem,
        font_id,
        min_mode: false,
    };
    dom_inline_text_walk(
        id,
        doc,
        styles,
        &white_space,
        tab,
        &mut segments,
        &mut state,
        per_font_intrinsic_on(),
    );
    segments.into_iter().fold(0.0f32, f32::max)
}

/// R5036（css-sizing-3 §min-content）：[`dom_inline_text_max_width`] 的 min-content
/// 变体——软换行机会全部使用：折叠空格为断点不计宽、文本按不可断词取最宽。pre 系
/// 保留空白无软断点（= max-content），由 DomWalkState.min_mode + walk 分派；pre-line
/// 的 \n 强制行切段语义两模式同。原子盒宽仍由调用方 loop 侧单边入账（分工同 max）。
fn dom_inline_text_min_width(box_node: &LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) -> f32 {
    let Some(id) = box_node.node_id else { return 0.0 };
    let Some(style) = styles.get(&id) else { return 0.0 };
    let (font_size, _line_height) = crate::inline::resolve_font_metrics(Some(style));
    let is_ahem = style
        .font_family
        .iter()
        .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));
    let white_space = style.white_space.clone();
    let font_id = intrinsic_font_id(Some(style));
    let tab = intrinsic_tab_metrics(Some(style), font_size, is_ahem, font_id);
    let mut segments: Vec<f32> = vec![0.0];
    let mut state = DomWalkState {
        pending_space: false,
        line_has_content: false,
        font_size,
        is_ahem,
        font_id,
        min_mode: true,
    };
    dom_inline_text_walk(
        id,
        doc,
        styles,
        &white_space,
        tab,
        &mut segments,
        &mut state,
        per_font_intrinsic_on(),
    );
    segments.into_iter().fold(0.0f32, f32::max)
}

/// R4355：跨节点空白折叠状态——CSS 白空格折叠跨 inline 盒边界连续（源内连续空白串
/// 折叠为一个空格；行首/行尾空白丢弃）。`pending_space` = 已见待定空白（尚不计宽）；
/// `line_has_content` = 本段已见非空白内容或原子盒（决定 pending 空白是否为行首丢弃）。
/// R5036：`min_mode` = min-content 口径（css-sizing-3 §min-content——软换行机会全部
/// 使用）：折叠空格是断点不计宽、文本按不可断词取最宽而非整段累加；pre 系保留空白
/// 无断点（= max-content），由调用方分派。
struct DomWalkState {
    pending_space: bool,
    line_has_content: bool,
    font_size: f32,
    is_ahem: bool,
    font_id: Option<u32>,
    min_mode: bool,
}

impl DomWalkState {
    fn flush_space(&mut self, segments: &mut [f32]) {
        if self.min_mode {
            // R5036：min-content 语境空格 = 软换行断点，永不入账。
            self.pending_space = false;
            return;
        }
        if self.pending_space && self.line_has_content {
            *segments.last_mut().expect("segments 非空") +=
                measure_intrinsic_char(' ', self.font_id, self.font_size, self.is_ahem);
        }
        self.pending_space = false;
    }
}

/// R4395：折叠语境（non-preserve）的文本段 walk——连续空白折叠态语义：
/// - 纯空白段只记 pending（与后续内容相接时计一个空格宽）——旧实现 collapsed=" "
///   非空 → 无条件逐字计宽，行首/行尾空白误计（R4394 bare walker 同缺陷前科，
///   intra-base-white-space 族）；
/// - 非空文本剥首尾空白计宽，边缘空白转 pending（行首无前置内容不计宽）。
fn walk_collapsible_text(content: &str, segments: &mut [f32], state: &mut DomWalkState) {
    let collapsed = crate::inline::collapse_whitespace(content);
    let trimmed = collapsed.trim_matches(' ');
    let leading = collapsed.starts_with(' ');
    let trailing = collapsed.ends_with(' ');
    if trimmed.is_empty() {
        if !collapsed.is_empty() || content.chars().any(char::is_whitespace) {
            state.pending_space = true;
        }
        return;
    }
    if leading {
        state.pending_space = true;
    }
    state.flush_space(segments);
    // R5036（css-sizing-3 §min-content）：min 模式按软换行断点（折叠空格）切词取最宽
    // 不可断单元；max 模式整段累加（不换行假设）。跨 inline 盒边界的词不合并（断点
    // 语义保守——本函数只处理单文本节点，跨节点词边界由空格 pending 状态自然分段）。
    let w: f32 = if state.min_mode {
        trimmed
            .split(' ')
            .filter(|word| !word.is_empty())
            .map(|word| {
                word.chars()
                    .map(|ch| measure_intrinsic_char(ch, state.font_id, state.font_size, state.is_ahem))
                    .sum::<f32>()
            })
            .fold(0.0f32, f32::max)
    } else {
        trimmed
            .chars()
            .map(|ch| measure_intrinsic_char(ch, state.font_id, state.font_size, state.is_ahem))
            .sum()
    };
    let last = segments.last_mut().expect("segments 非空");
    if state.min_mode {
        // 段语义切换：min = 本强制行内最宽不可断单元（fold max 取全行最宽）。
        *last = last.max(w);
    } else {
        *last += w;
    }
    state.line_has_content = true;
    if trailing {
        state.pending_space = true;
    }
}

#[allow(clippy::too_many_arguments)]
fn dom_inline_text_walk(
    node_id: NodeId,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    white_space: &WhiteSpaceValue,
    tab: (f32, f32),
    segments: &mut Vec<f32>,
    state: &mut DomWalkState,
    per_font: bool,
) {
    for child in doc.child_nodes(node_id) {
        let Some(node) = doc.get(child) else { continue };
        match &node.kind {
            zero_dom::NodeKind::Text(t) => {
                let preserve_spaces = matches!(
                    white_space,
                    WhiteSpaceValue::Pre | WhiteSpaceValue::PreWrap | WhiteSpaceValue::BreakSpaces
                );
                if preserve_spaces {
                    // 保留换行模式：逐字计宽（pre 系空白保留）；\n 切段由 accumulate 处理。
                    // 这里与折叠语义不同源，直接复用 accumulate 的 pre 臂（无跨节点折叠）。
                    let mut tmp: Vec<f32> = vec![0.0];
                    accumulate_text_width(
                        &t.content,
                        white_space,
                        state.font_size,
                        state.is_ahem,
                        state.font_id,
                        tab,
                        &mut tmp,
                    );
                    *segments.last_mut().expect("segments 非空") += tmp.into_iter().fold(0.0f32, f32::max);
                    state.line_has_content = true;
                    continue;
                }
                // R4397（css-text-3 §white-space-phase-1）：pre-line 保留 \n 为强制换行——
                // 按 \n 切段（段前 pending 随行尾丢弃——「collapsible spaces immediately
                // preceding a sequent break are removed」），段内空白照常折叠。
                if matches!(white_space, WhiteSpaceValue::PreLine) && t.content.contains('\n') {
                    for part in t.content.split('\n') {
                        walk_collapsible_text(part, segments, state);
                        // \n 强制断：行尾 pending 丢弃，开新段。
                        state.pending_space = false;
                        state.line_has_content = false;
                        segments.push(0.0);
                    }
                    continue;
                }
                walk_collapsible_text(&t.content, segments, state);
            }
            zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br") => {
                // 强制换行：行尾待定空白丢弃（CSS：行尾空白不渲染），开新段。
                state.pending_space = false;
                state.line_has_content = false;
                segments.push(0.0);
            }
            zero_dom::NodeKind::Element(e) => {
                let Some(cs) = styles.get(&child) else { continue };
                use zero_style_system::property::types::DisplayValue as DV;
                // display:none：rt/rp（ruby annotation 不计宽）、script/template 等——对
                // 白空格折叠透明（两侧空白串仍折叠为基准流内一个空格）。
                if matches!(cs.display, DV::None) {
                    continue;
                }
                // 块级后代：停止（不属本 inline 流）；其后空白为下一行行首（丢弃）。
                if matches!(
                    cs.display,
                    DV::Block
                        | DV::FlowRoot
                        | DV::ListItem
                        | DV::Table
                        | DV::TableRow
                        | DV::TableRowGroup
                        | DV::TableHeaderGroup
                        | DV::TableFooterGroup
                        | DV::TableCell
                        | DV::TableCaption
                        | DV::TableColumn
                        | DV::TableColumnGroup
                        | DV::Flex
                        | DV::Grid
                ) {
                    state.pending_space = false;
                    state.line_has_content = false;
                    continue;
                }
                // 原子 inline（盒模型/replaced 族）：调用方 LayoutBox 分支负责宽度，跳过
                // 文本；其与前后 inline 内容之间的折叠空格照常渲染。
                if matches!(
                    cs.display,
                    DV::InlineBlock | DV::InlineFlex | DV::InlineGrid | DV::InlineTable
                ) {
                    state.flush_space(segments);
                    state.line_has_content = true;
                    continue;
                }
                let tag_lower = e.local_name().to_ascii_lowercase();
                if matches!(
                    tag_lower.as_str(),
                    "img"
                        | "video"
                        | "audio"
                        | "canvas"
                        | "iframe"
                        | "embed"
                        | "object"
                        | "svg"
                        | "input"
                        | "button"
                        | "select"
                        | "textarea"
                        | "keygen"
                        | "progress"
                        | "meter"
                ) {
                    state.flush_space(segments);
                    state.line_has_content = true;
                    continue;
                }
                let child_ws = styles
                    .get(&child)
                    .map(|s| s.white_space.clone())
                    .unwrap_or_else(|| white_space.clone());
                let cs_child = styles.get(&child);
                // R4919（css-sizing-3 #max-content-inline-size）：max-content 的 inline 文本段
                // 按**该段自身** computed font 度量——旧实现整棵 DOM 子树沿用容器
                // font_size/is_ahem/font_id，嵌套异字号（如 baidu .mnav 12px 容器 > a.s-bri
                // 13px 锚）被低估（2×12=24 < 真实 26）→ shrink-to-fit 盒窄于内容 → 内层 IFC
                // 按真实字体二次布局折行竖排（「更多」「换一换」竖排实证）。子无样式条目沿用
                // 父语境；`ZW_INTRINSIC_PERFONT=0` 回退旧行为（回归归因 A/B）。
                // 边界折叠空格的 advance 记在 flush 落点语境（近似，跨异字号边界 ±1px 级）。
                let (child_fs, child_ahem, child_fid) = match cs_child.filter(|_| per_font) {
                    Some(s) => {
                        let (fs, _) = crate::inline::resolve_font_metrics(Some(s));
                        (
                            fs,
                            s.font_family
                                .iter()
                                .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem")),
                            intrinsic_font_id(Some(s)),
                        )
                    }
                    None => (state.font_size, state.is_ahem, state.font_id),
                };
                // R4416：tab 度量随 white_space 同点解析（子样式覆盖，否则继承）；R4919 起
                // 随子字体语境。
                let child_tab = cs_child
                    .map(|s| intrinsic_tab_metrics(Some(s), child_fs, child_ahem, child_fid))
                    .unwrap_or(tab);
                // R4357：ruby 注释行宽参与——max-content 侧与 collect run margin 同模型，
                // extra（box − base）计入段宽（两侧不同步则 intrinsic 与行宽分裂，
                // ruby-overhang-spaces-002 的 width:max-content 容器即此形态）。
                if e.local_name().eq_ignore_ascii_case("ruby") {
                    let annot = crate::inline::ruby_annotation_width_text(doc, child);
                    if !annot.is_empty() && std::env::var("ZW_RUBY_OVERHANG_MODEL").as_deref() != Ok("0") {
                        let rs = styles.get(&child);
                        let (ruby_fs, _) = crate::inline::resolve_font_metrics(rs);
                        let ruby_ahem = rs.is_some_and(|s| {
                            s.font_family
                                .iter()
                                .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"))
                        });
                        // R4415：ruby 前邻折叠空格先于 start_w 冲账——旧序下 pending_space
                        // 在 base 段 walk 的 flush_space 里入账，base_w 被空格宽污染
                        //（overhang-spaces-015 实证 25 vs collect trimmed 20，pads 12/13 vs
                        // 15/15 → max-content 95 < 行宽 100，尾部 あ 折行）。
                        state.flush_space(segments);
                        let start_w = *segments.last().expect("segments 非空");
                        // R4919：ruby base 段同 per-element 字体语境（见上注）。
                        let (prev_fs, prev_ahem, prev_fid) = (state.font_size, state.is_ahem, state.font_id);
                        state.font_size = child_fs;
                        state.is_ahem = child_ahem;
                        state.font_id = child_fid;
                        dom_inline_text_walk(child, doc, styles, &child_ws, child_tab, segments, state, per_font);
                        state.font_size = prev_fs;
                        state.is_ahem = prev_ahem;
                        state.font_id = prev_fid;
                        // R4415：base 内尾随空格在 pads 前入账——base_w = 完整折叠 base 宽
                        //（含前后边缘空格 advance，与 collect 侧 collapsed 口径同源）。
                        // ref 页形态（空格在 base 内）旧序把尾随空格漏到 pads 之后
                        //（base_w 25 vs 30，pads 35 vs 30），test/ref 双页 region 差 5px
                        //（60 vs 65）。空格已在段内（region 和不变），仅 pads 分母修正。
                        state.flush_space(segments);
                        let base_w = *segments.last().expect("segments 非空") - start_w;
                        // R4416：preserve 语境 capacity 走原始空白串（tab 按名义 unit，
                        // 与行内 stop 推进同量级；折叠语境 None = collapse 语义不变）。
                        let preserve = matches!(
                            child_ws,
                            WhiteSpaceValue::Pre | WhiteSpaceValue::PreWrap | WhiteSpaceValue::BreakSpaces
                        );
                        let (pl, pr) = crate::inline::ruby_overhang_pads(
                            doc,
                            child,
                            base_w,
                            &annot,
                            ruby_fs,
                            0.0,
                            ruby_ahem,
                            preserve.then_some(child_tab),
                        );
                        *segments.last_mut().expect("segments 非空") += pl + pr;
                        continue;
                    }
                }
                // R4919：per-element 字体语境换入/换出（见上注）——递归以子语境度量，返回
                // 恢复父语境（边界 pending 空格折叠态不随字体语境切换）。
                let (prev_fs, prev_ahem, prev_fid) = (state.font_size, state.is_ahem, state.font_id);
                state.font_size = child_fs;
                state.is_ahem = child_ahem;
                state.font_id = child_fid;
                dom_inline_text_walk(child, doc, styles, &child_ws, child_tab, segments, state, per_font);
                state.font_size = prev_fs;
                state.is_ahem = prev_ahem;
                state.font_id = prev_fid;
            }
            _ => {}
        }
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
    let is_ahem = inline_style
        .font_family
        .iter()
        .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));
    // R1748：br-aware —— fragment_node_ids 共享一组 segments（同片段 inline 级内容按序累入
    // 当前段，遇 br 切段），取最宽段。无 br 时单段 = 全文本累加（行为同旧 total）。
    // R4223：保留换行模式同样按 \n 切段（white-space 取 split inline 自身，片段内无样式表
    // 可查嵌套覆盖——传 None 沿用继承值）。
    let white_space = inline_style.white_space.clone();
    // R4367：片段语境无样式表可用（styles=None），font_id 走 inline_style 自身
    //（容器近似同 dom_inline_text_max_width 的 R4043 注记）。
    let font_id = intrinsic_font_id(Some(inline_style));
    let tab = intrinsic_tab_metrics(Some(inline_style), font_size, is_ahem, font_id);
    let mut segments: Vec<f32> = vec![0.0];
    // R5034：跨节点折叠状态随片段序列连续（同 dom_inline_text_max_width）——片段边界
    // 行缘可折叠空格不入账。
    let mut state = DomWalkState {
        pending_space: false,
        line_has_content: false,
        font_size,
        is_ahem,
        font_id,
        min_mode: false,
    };
    for nid in fragment_node_ids {
        text_max_width_walk(
            *nid,
            doc,
            font_size,
            is_ahem,
            &white_space,
            None,
            font_id,
            tab,
            &mut segments,
            &mut state,
            // R4919：fragment 语境无样式表可用（styles=None，R4367）——per-element 字体
            // 语境无从解析，维持 split inline 自身 font 近似。
            // R4921：原子 gate 需查子 display/width（styles=None 无从解析）且 split inline
            // 片段无 Σ 配对侧——恒关。
            false,
            false,
        );
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
    // R1840：mirror converter §10.1 visibility:collapse 逻辑（converter/mod.rs:312，R1834）。
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
        && let Some(v) = resolve_intrinsic_real_length(len, s)
    {
        let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
        return v + frame;
    }
    // 2. width 显式长度
    if let Some(s) = style
        && let Some(v) = resolve_intrinsic_real_length(&s.width, s)
    {
        let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
        return v + frame;
    }
    // 2.5 R1015/R1017：aspect-ratio transferred-size——width:auto + aspect_ratio + definite main。
    // main 来源优先级：(a) item 自身 height Px；(b) item min-height Px 地板；(c) R1017 container-
    // stretch cross（容器 definite height Px 拉伸 item，如 inline-flex height:100px；经
    // shrink_inline_blocks_to_content IFC 路径调用，绕过 R1016 的 taffy gate 墙）。
    if let Some(s) = style
        && matches!(s.width, LengthValue::Auto)
        && let Some(ratio) = s.aspect_ratio.filter(|&r| r > 0.0)
    {
        let main = resolve_intrinsic_real_length(&s.height, s)
            .or_else(|| resolve_intrinsic_real_length(&s.min_height, s))
            .or(container_cross);
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
    // R4252：水平 flex 主轴 gap = **column-gap** 长写（css-align §gap：row-gap 沿行轴、
    // column-gap 沿列轴/主轴）；legacy `gap` 字段在双值简写下不再下发（shorthand R4252），
    // Auto（normal）解析 0。
    let gap = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| resolve_intrinsic_real_length(&s.column_gap, s))
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
    grid_intrinsic_width_ex(box_node, doc, styles, false)
}

/// `min_mode = true`（min-content 语境）：scroll container（overflow x/y auto/scroll）的
/// item min-content 贡献 = 0——scroller 可收缩到任意小，内容滚动（css-overflow-3 +
/// css-sizing-3 #scroll-container-intrinsic；grid-aspect-ratio-027 取证：`width:min-content`
/// 列流网格内 overflow:auto + ratio 2/1 item 的列宽应 0 而非 transfer 200）。max-content
/// 语境不适用（scroller 的 max-content 仍是内容 fit 宽）。
pub(crate) fn grid_intrinsic_width_ex(
    box_node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    min_mode: bool,
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
        .and_then(|s| resolve_intrinsic_real_length(&s.column_gap, s))
        .unwrap_or(0.0);
    // R4008（css-sizing-4 §intrinsic-size-override）：grid 容器自身 contain:size 时，
    // 固有宽 = CIS 替代（内容/tracks 不参与——024：width:max-content + CIS 200 应 200，
    // definite-track sum 170 不得抢先）。CIS 缺失/非 definite → None 走原路径。
    if style.is_some_and(|s| s.contain.has_size())
        && matches!(
            style.unwrap().width,
            LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_)
        )
        && let Some(cis) = style
            .unwrap()
            .contain_intrinsic_width
            .as_ref()
            .and_then(|v| resolve_intrinsic_real_length(v, style.unwrap()))
    {
        let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
        return Some(cis + frame);
    }
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
            // R5040：min-content 语境 scroller 内容贡献 = 0（css-overflow-3——scroller
            // 可收缩任意小、内容滚动），但 **definite min-width 仍地板**（027 item2：
            // ratio 1/2 + min-width:100px + overflow:auto → 列 100 而非 0/50）。
            let is_scroller = child.node_id.and_then(|id| styles.get(&id)).is_some_and(|cs| {
                matches!(cs.overflow_x, OverflowValue::Auto | OverflowValue::Scroll)
                    || matches!(cs.overflow_y, OverflowValue::Auto | OverflowValue::Scroll)
            });
            if min_mode && is_scroller {
                let scroller_floor = child
                    .node_id
                    .and_then(|id| styles.get(&id))
                    .and_then(|cs| {
                        resolve_intrinsic_real_length(&cs.min_width, cs).map(|w| {
                            w + child.padding_left + child.padding_right + child.border_left + child.border_right
                        })
                    })
                    .map(|w| (w + child.margin_left + child.margin_right).max(0.0))
                    .unwrap_or(0.0);
                sum += scroller_floor;
                max_w = max_w.max(scroller_floor);
                continue;
            }
            let base = box_content_max_width(child, doc, styles) + child.margin_left + child.margin_right;
            // R5040（css-grid-1 #algo-track-sizing + css-sizing-4 §4.1）：item 的
            // min/max-content 轨道贡献被其 definite min/max-width 钳——比例 transfer 宽
            // 超 max-width 收窄（024 item2：transfer 100 超 25 → 列 25）、不足 min-width
            // 抬升（024 item3：transfer 10 不足 25 → 列 25）。fit-content()/content
            // 关键字 min/max-width 有独立接线域，不在此重复处理。
            let clamped_base = child
                .node_id
                .and_then(|id| styles.get(&id))
                .map(|cs| {
                    let min_w = resolve_intrinsic_real_length(&cs.min_width, cs);
                    let max_w_def = resolve_intrinsic_real_length(&cs.max_width, cs);
                    base.max(min_w.unwrap_or(0.0)).min(max_w_def.unwrap_or(f32::MAX))
                })
                .unwrap_or(base);
            sum += clamped_base;
            max_w = max_w.max(clamped_base);
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
    // R1842：显式 definite-length grid-template-columns（如 `60px`、`100px 200px`）定义
    // 固定 track，grid 的 max-content 宽度 = definite track 之和（CSS Grid §11.2：item
    // 溢出不撑大 definite track）。仅当全部显式 track 为 definite Px 且 item 数 <= track
    // 数时启用，其余回落 item-content 测量。修 inline-grid + 空子 + `grid-template-columns:60px`
    // 旧路径返 item-content(0)+frame=6px（应 60px+frame）。A/B 守 net≥0。
    let inner = match style.and_then(sum_definite_grid_columns) {
        Some((def_sum, def_n)) if def_n >= count => def_sum + gap * (def_n - 1) as f32,
        _ if multi_column => sum + gap * (count - 1) as f32,
        _ => max_w,
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

/// 解析显式 `grid-template-columns` 中**全部 definite-length** track 的宽度之和。
///
/// 仅当所有显式 track 均为 definite Px 长度（如 `60px`、`100px 200px`）时返回
/// `Some((sum, count))`；遇到 `fr`/`auto`/`min-content`/`max-content`/`minmax()`/
/// `repeat()`/`fit-content()` 等非 definite track 返回 `None`（调用方回落到 item
/// content 测量）。line-names `[a]` 跳过。括号感知分词避免 `minmax(a, b)` 内空格误切。
/// 用于 [`grid_intrinsic_width`] 的 definite-track fast path（R1842）。
fn sum_definite_grid_columns(s: &ComputedStyle) -> Option<(f32, usize)> {
    let cols = s.grid_template_columns.as_deref()?.trim();
    if cols.is_empty() || cols.eq_ignore_ascii_case("none") {
        return None;
    }
    let mut tokens: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    for ch in cols.chars() {
        match ch {
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                depth -= 1;
                cur.push(ch);
            }
            c if c.is_whitespace() && depth == 0 => {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    let mut sum = 0.0f32;
    let mut n = 0usize;
    for t in &tokens {
        if t.starts_with('[') {
            continue; // line-names
        }
        let val = t
            .strip_suffix("px")
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .and_then(|x| x.parse::<f32>().ok());
        match val {
            Some(v) if v >= 0.0 => {
                sum += v;
                n += 1;
            }
            _ => return None, // 非 definite track（fr/auto/minmax/repeat/fit-content）→ 整体回落
        }
    }
    (n > 0).then_some((sum, n))
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

    // ── R4008（css-sizing-4 §intrinsic-size-override）：contain:size 的 CIS 替代测量 ──

    /// width:min-content 的 contain:size 叶，intrinsic 宽 = CIS 替代（004：CIS 111 → 111 应 0）。
    #[test]
    fn r4008_cis_overrides_min_content_leaf_intrinsic() {
        let w = compute_block_max_content(
            r#"<html><body><div id="t" style="width: min-content; contain: size; contain-intrinsic-size: 111px 222px;"></div></body></html>"#,
            "t",
        );
        assert!(
            (w.unwrap_or(0.0) - 111.0).abs() < 1.0,
            "CIS width must replace content-based intrinsic width, got {w:?}"
        );
    }

    /// max-content 父含 contain:size 子（CIS 宽）→ 父 intrinsic 计入 CIS（002：111+边框 2 → 113）。
    #[test]
    fn r4008_cis_child_contributes_to_parent_max_content() {
        let w = compute_block_max_content(
            r#"<html><body><div id="t" style="width: max-content; border: 1px solid black;">
<div style="contain: size; contain-intrinsic-size: 111px 222px;"></div>
</div></body></html>"#,
            "t",
        );
        assert!(
            (w.unwrap_or(0.0) - 113.0).abs() < 1.0,
            "parent max-content must include CIS child (111 + border 2), got {w:?}"
        );
    }

    // ── R4223（css-text-3 §4.1.3 + css-sizing-3）：保留换行的 \n 强制换行切段测量 ──

    /// pre 块 max-content = 最宽行（13 字符行），而非全文本折叠单行（28 字符）。
    /// text-group-align ref 页 `.group{inline-size:min-content}` 262px→114px 的根因锚。
    #[test]
    fn r4223_pre_newline_splits_intrinsic_segments() {
        let w = compute_block_max_content(
            r#"<html><body><div id="t" style="white-space: pre; font: 16px monospace">ABCDEFGHIJKLO
AAAAAAAA
AAAA</div></body></html>"#,
            "t",
        );
        // 13 字符行最宽（若 \n 被折叠成空格测单行，宽度 ≈ 13+1+8+1+4 = 27 字符）。
        let w13 = compute_block_max_content(
            r#"<html><body><div id="t" style="white-space: pre; font: 16px monospace">ABCDEFGHIJKLO</div></body></html>"#,
            "t",
        );
        let (a, b) = (w.unwrap_or(0.0), w13.unwrap_or(0.0));
        assert!(
            a > 0.0 && (a - b).abs() < 1.0,
            "pre max-content must equal widest line: multi={a}, single={b}"
        );
    }

    /// normal 白空间（\n 折叠为空格）行为不变：max-content = 全文本单行宽。
    #[test]
    fn r4223_normal_newline_still_collapses() {
        let multi = compute_block_max_content(
            r#"<html><body><div id="t" style="font: 16px monospace">ABCDEFGHIJKLO
AAAAAAAA</div></body></html>"#,
            "t",
        );
        let single = compute_block_max_content(
            r#"<html><body><div id="t" style="font: 16px monospace">ABCDEFGHIJKLO AAAAAAAA</div></body></html>"#,
            "t",
        );
        let (a, b) = (multi.unwrap_or(0.0), single.unwrap_or(0.0));
        assert!(
            a > 0.0 && (a - b).abs() < 1.0,
            "normal white-space must collapse \\n to space: multi={a}, single={b}"
        );
    }

    /// pre-line：\n 强制换行切段，行内空格折叠。
    #[test]
    fn r4223_preline_newline_splits_and_collapses_spaces() {
        let split = compute_block_max_content(
            r#"<html><body><div id="t" style="white-space: pre-line; font: 16px monospace">AAAAAAAA
AAAA</div></body></html>"#,
            "t",
        );
        let widest = compute_block_max_content(
            r#"<html><body><div id="t" style="white-space: pre-line; font: 16px monospace">AAAAAAAA</div></body></html>"#,
            "t",
        );
        let (a, b) = (split.unwrap_or(0.0), widest.unwrap_or(0.0));
        assert!(
            a > 0.0 && (a - b).abs() < 1.0,
            "pre-line max-content must equal widest line: split={a}, widest={b}"
        );
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

    #[test]
    fn r3632_multicol_intrinsic_resolves_residual_column_width_and_gap() {
        let mut doc = zero_dom::Document::new();
        let article_id = doc.create_element("article");
        let child_id = doc.create_element("div");

        let mut styles = HashMap::new();
        let mut article_style = ComputedStyle::default();
        article_style.font_size = LengthValue::Px(20.0);
        article_style.column_count = zero_style_system::ColumnCountComputedValue::Number(2);
        article_style.column_width = zero_style_system::ColumnWidthComputedValue::Length(LengthValue::Em(6.0));
        article_style.column_gap = LengthValue::Em(2.0);
        styles.insert(article_id, article_style);

        let mut child_style = ComputedStyle::default();
        child_style.display = DisplayValue::Block;
        child_style.width = LengthValue::Px(100.0);
        styles.insert(child_id, child_style);

        let article = LayoutBox {
            node_id: Some(article_id),
            children: vec![LayoutBox {
                node_id: Some(child_id),
                is_block_level: true,
                ..Default::default()
            }],
            ..Default::default()
        };

        let w = block_max_content_width(&article, &doc, &styles);
        assert_eq!(
            w, 280.0,
            "column-count:2 with column-width:6em and gap:2em at 20px should produce 2*120+40"
        );
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

    // ── R5040（css-grid-1 #algo-track-sizing + css-sizing-4 §4.1 + css-overflow-3）──

    /// 跑完整引擎后对 grid#t 直调 min-content 语境固有宽（R5040 断言 helper）。
    fn r5040_grid_intrinsic_min(html: &str) -> Option<f32> {
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
        let target = find("t", &doc, &result.root)?;
        grid_intrinsic_width_ex(target, &doc, &styles, true)
    }

    /// item 轨道贡献被 definite max-width 钳（grid-aspect-ratio-024 item2 同构）：
    /// height:100px + ratio 1/1 → transfer 宽 100，max-width:25px → 列 25。
    #[test]
    fn r5040_grid_item_contribution_clamped_by_max_width() {
        let w = r5040_grid_intrinsic_min(
            r#"<html><body><div id="t" style="display:grid;grid-auto-flow:column;width:min-content">
            <div style="height:100px;aspect-ratio:1/1;max-width:25px"></div></div></body></html>"#,
        );
        assert_eq!(w, Some(25.0), "transfer 宽 100 须被 max-width 钳至列 25");
    }

    /// item 轨道贡献被 definite min-width 抬升（grid-aspect-ratio-024 item3 同构）：
    /// ratio .1/1 → transfer 宽 10，min-width:25px → 列 25。
    #[test]
    fn r5040_grid_item_contribution_floored_by_min_width() {
        let w = r5040_grid_intrinsic_min(
            r#"<html><body><div id="t" style="display:grid;grid-auto-flow:column;width:min-content">
            <div style="height:100px;aspect-ratio:0.1/1;min-width:25px"></div></div></body></html>"#,
        );
        assert_eq!(w, Some(25.0), "transfer 宽 10 须被 min-width 抬至列 25");
    }

    /// scroller（overflow:auto）内容 min-content 贡献 = 0，definite min-width 仍地板
    ///（grid-aspect-ratio-027 同构）：item1 ratio 2/1 overflow:auto 无 min-width → 列 0；
    /// item2 ratio 1/2 + min-width:100px + overflow:auto → 列 100；总 100。
    #[test]
    fn r5040_grid_scroller_content_zero_min_width_floors() {
        let w = r5040_grid_intrinsic_min(
            r#"<html><body><div id="t" style="display:grid;grid-auto-flow:column;width:min-content">
            <div style="height:100px;aspect-ratio:2/1;overflow:auto"></div>
            <div style="height:100px;aspect-ratio:1/2;min-width:100px;overflow:auto"></div>
            </div></body></html>"#,
        );
        assert_eq!(w, Some(100.0), "scroller 列 0 + min-width 地板列 100，总 100");
    }

    /// % 宽在固有语境循环按 auto → ratio transferred（grid-aspect-ratio-025 同构）：
    /// width:50% + height:100px + ratio 1/1 → 贡献 100（旧测 0 容器塌 0）。
    #[test]
    fn r5040_grid_percent_width_cyclic_transfers_via_ratio() {
        let w = r5040_grid_intrinsic_min(
            r#"<html><body><div id="t" style="display:grid;width:min-content">
            <div style="height:100px;width:50%;aspect-ratio:1/1"></div></div></body></html>"#,
        );
        assert_eq!(w, Some(100.0), "% 宽循环按 auto → transferred 100");
    }

    #[test]
    fn r3631_grid_intrinsic_gap_resolves_residual_real_length() {
        let mut doc = zero_dom::Document::new();
        let grid_id = doc.create_element("div");
        let child_a_id = doc.create_element("div");
        let child_b_id = doc.create_element("div");

        let mut styles = HashMap::new();
        let mut grid_style = ComputedStyle::default();
        grid_style.display = DisplayValue::Grid;
        grid_style.grid_auto_flow = zero_style_system::property::types::GridAutoFlowValue::Column;
        grid_style.font_size = LengthValue::Px(20.0);
        grid_style.column_gap = LengthValue::Em(2.0);
        styles.insert(grid_id, grid_style);

        for child_id in [child_a_id, child_b_id] {
            let mut child_style = ComputedStyle::default();
            child_style.display = DisplayValue::Block;
            child_style.width = LengthValue::Px(50.0);
            styles.insert(child_id, child_style);
        }

        let child_box = |node_id| LayoutBox {
            node_id: Some(node_id),
            width: 50.0,
            is_block_level: true,
            ..Default::default()
        };
        let grid = LayoutBox {
            node_id: Some(grid_id),
            children: vec![child_box(child_a_id), child_box(child_b_id)],
            ..Default::default()
        };

        let width = grid_intrinsic_width(&grid, &doc, &styles).expect("grid intrinsic");
        assert_eq!(
            width, 140.0,
            "two 50px grid items plus column-gap:2em at 20px should produce 140px intrinsic width"
        );
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
    fn test_leaf_explicit_width_relative_length_fallback() {
        let mut doc = zero_dom::Document::new();
        let node = doc.create_element("div");

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);
        style.width = LengthValue::Em(5.0);
        styles.insert(node, style);

        let box_node = LayoutBox {
            node_id: Some(node),
            ..Default::default()
        };

        let w = box_content_max_width(&box_node, &doc, &styles);

        assert_eq!(w, 100.0, "5em at 20px should contribute a 100px max-content width");
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
    fn r3629_flex_row_intrinsic_gap_resolves_residual_real_length() {
        let mut doc = zero_dom::Document::new();
        let container_id = doc.create_element("div");
        let child_a_id = doc.create_element("div");
        let child_b_id = doc.create_element("div");

        let mut styles = HashMap::new();
        let mut container_style = ComputedStyle::default();
        container_style.display = DisplayValue::Flex;
        container_style.font_size = LengthValue::Px(20.0);
        // R4252：主轴 gap 消费 column_gap 长写（`gap: 2em` 经 shorthand expansion 落
        // row-gap/column-gap；legacy `gap` 字段双值简写下不再下发，主轴臂不再消费）。
        container_style.column_gap = LengthValue::Em(2.0);
        styles.insert(container_id, container_style);

        for child_id in [child_a_id, child_b_id] {
            let mut child_style = ComputedStyle::default();
            child_style.display = DisplayValue::Block;
            child_style.width = LengthValue::Px(50.0);
            styles.insert(child_id, child_style);
        }

        let child_box = |node_id| LayoutBox {
            node_id: Some(node_id),
            width: 50.0,
            is_block_level: true,
            ..Default::default()
        };
        let container = LayoutBox {
            node_id: Some(container_id),
            children: vec![child_box(child_a_id), child_box(child_b_id)],
            ..Default::default()
        };

        let width = flex_row_intrinsic_width(&container, &doc, &styles).expect("flex row intrinsic");
        assert_eq!(
            width, 140.0,
            "two 50px flex items plus gap:2em at 20px should produce 140px intrinsic width"
        );
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
    fn test_flex_basis_relative_length_overrides_width() {
        let mut doc = zero_dom::Document::new();
        let node = doc.create_element("div");

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);
        style.flex_basis = FlexBasisValue::Length(LengthValue::Em(5.0));
        style.width = LengthValue::Px(50.0);
        styles.insert(node, style);

        let box_node = LayoutBox {
            node_id: Some(node),
            ..Default::default()
        };

        let w = flex_item_base_size(&box_node, &doc, &styles, None);

        assert_eq!(w, 100.0, "flex-basis:5em at 20px should override width:50px");
    }

    #[test]
    fn r3637_flex_base_width_resolves_residual_real_length_before_content() {
        let mut doc = zero_dom::Document::new();
        let node = doc.create_element("div");
        let text = doc.create_text_node("XXXXXXXXXX");
        doc.append_child(node, text).unwrap();

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Block;
        style.font_family = vec!["Ahem".to_string()];
        style.font_size = LengthValue::Px(20.0);
        style.width = LengthValue::Em(5.0);
        styles.insert(node, style);

        let box_node = LayoutBox {
            node_id: Some(node),
            is_block_level: true,
            ..Default::default()
        };

        let w = flex_item_base_size(&box_node, &doc, &styles, None);

        assert_eq!(
            w, 100.0,
            "flex-basis:auto should use width:5em at 20px before 200px Ahem max-content"
        );
    }

    #[test]
    fn r3630_flex_item_aspect_ratio_transfers_residual_height() {
        let mut doc = zero_dom::Document::new();
        let node = doc.create_element("div");

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Block;
        style.font_size = LengthValue::Px(20.0);
        style.width = LengthValue::Auto;
        style.height = LengthValue::Em(5.0);
        style.aspect_ratio = Some(2.0);
        styles.insert(node, style);

        let box_node = LayoutBox {
            node_id: Some(node),
            is_block_level: true,
            ..Default::default()
        };

        let w = flex_item_base_size(&box_node, &doc, &styles, None);

        assert_eq!(
            w, 200.0,
            "width:auto + aspect-ratio:2 + height:5em at 20px should transfer to 200px base width"
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
        // R4408 开门注：交错 walk default-on 后，两原子盒之间的折叠空格计入 max-content
        //（70 + 空格 ≈4 + 70 = 144）——CSS max-content 语义（行内内容含原子间折叠空格，
        // chromium 同值），旧 R1479 臂丢空格测 140。
        assert!(
            (140.0..150.0).contains(&w),
            "R1298 guard: empty inline-block[70px] children must contribute; expected ~140+collapsed space, got {w}"
        );
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
        // R4568：四节点须同源 Document（fresh_id_boxcw 每次新 Document 的 NodeId 会
        // 数值碰撞——四 id 同值使 styles 键互相覆盖，split wrapper 的 Inline 样式被
        // anon 的 Block/Px(50) 覆盖，gate 类改动下测的不是契约本身）。
        let mut doc = zero_dom::Document::new();
        let wrapper_id = doc.create_element("div");
        let anon1_id = doc.create_element("div");
        let blk_id = doc.create_element("div");
        let anon2_id = doc.create_element("div");

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

    // ── R4395：交错 walk（R4355 dom_inline_text_max_width）连续空白折叠态 ──
    //
    // ZW_MIXED_BARE_TEXT=1 时 box_content_max_width_inner 换用该 walk（配对臂：
    // Inline 子 frame-only + walk 计文本）。直接调 `dom_inline_text_max_width`
    // （不经 env 门）断言空白态机与 ruby base 计入契约。

    /// 布局 `html` 并返回 `target_id` 元素上的 walk 测量值 + 同源度量环境
    /// （供期望值计算复用同一次 parse/compute，Node/字体上下文一致）。
    fn walk_fixture(html: &str, target_id: &str) -> (f32, zero_dom::Document, HashMap<NodeId, ComputedStyle>, NodeId) {
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
        let found = find(target_id, &doc, &result.root).expect("target box");
        let node_id = found.node_id.expect("target node id");
        (dom_inline_text_max_width(found, &doc, &styles), doc, styles, node_id)
    }

    /// 目标容器同源字体参数下的逐字符测量期望值（与 walk Text 臂同口径求和）。
    fn expected_measure(styles: &HashMap<NodeId, ComputedStyle>, id: NodeId, text: &str) -> f32 {
        let style = styles.get(&id).expect("target style");
        let (font_size, _line_height) = crate::inline::resolve_font_metrics(Some(style));
        let is_ahem = style
            .font_family
            .iter()
            .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));
        let font_id = intrinsic_font_id(Some(style));
        text.chars()
            .map(|ch| measure_intrinsic_char(ch, font_id, font_size, is_ahem))
            .sum()
    }

    fn assert_walk_eq(html: &str, target_id: &str, expected_text: &str) {
        let (total, _doc, styles, node_id) = walk_fixture(html, target_id);
        let expected = expected_measure(&styles, node_id, expected_text);
        assert!(
            (total - expected).abs() < 0.51,
            "inline walk {target_id}: got {total}, expected {expected} (=w({expected_text:?})); html: {}",
            html.trim()
        );
    }

    /// 容器首空白（行首）不计宽——`<div>  <em>ab</em> cd</div>` ≡ `ab cd`。
    #[test]
    fn r4395_inline_walk_leading_whitespace_free() {
        assert_walk_eq(
            r#"<html><body><div id="t">  <em>ab</em> cd</div></body></html>"#,
            "t",
            "ab cd",
        );
    }

    /// 跨段空白串折叠为**一个**空格宽——`a      b`（6 空格）≡ `a b`。
    #[test]
    fn r4395_inline_walk_whitespace_runs_collapse_to_one_space() {
        assert_walk_eq(
            r#"<html><body><div id="t"><em>a</em>      <em>b</em></div></body></html>"#,
            "t",
            "a b",
        );
    }

    /// 注音（rt/rp，display:none）在**任意嵌套深度**透明跳过且不扰空白折叠——
    /// `a<rt>x</rt>b` + `<rp>(</rp>` + ` c` ≡ `ab c`（intra-base-white-space /
    /// improperly-contained-annotation 形态的态机契约）。
    #[test]
    fn r4395_inline_walk_rt_rp_excluded_at_any_depth() {
        assert_walk_eq(
            r#"<html><body><div id="t"><em>a<rt>x</rt>b</em><rp>(</rp> c</div></body></html>"#,
            "t",
            "ab c",
        );
    }

    /// 段尾空白不计宽（无后续内容即行尾丢弃）——`<em>ab</em>   ` ≡ `ab`。
    #[test]
    fn r4395_inline_walk_trailing_whitespace_free() {
        assert_walk_eq(
            r#"<html><body><div id="t"><em>ab</em>   </div></body></html>"#,
            "t",
            "ab",
        );
    }

    /// 文本只计一次（walk 计文本 + 配对臂 frame-only 的分工契约）——
    /// `XYZ <span>ABC</span> XYZ` ≡ `XYZ ABC XYZ`（span 文本经 walk 单次入账）。
    #[test]
    fn r4395_inline_walk_text_counted_once() {
        assert_walk_eq(
            r#"<html><body><div id="t">XYZ <span>ABC</span> XYZ</div></body></html>"#,
            "t",
            "XYZ ABC XYZ",
        );
    }

    /// ruby base 文本计入（R4389 bare walker 缺席面——inline-block shrink 竖排坍塌轴）
    /// 且 rt 注音排除——`XYZ <ruby><rb>ABC</rb><rt>x</rt></ruby> XYZ` ≡ `XYZ ABC XYZ`。
    #[test]
    fn r4395_inline_walk_ruby_base_counted_annotation_excluded() {
        assert_walk_eq(
            r#"<html><body><div id="t">XYZ <ruby><rb>ABC</rb><rt>x</rt></ruby> XYZ</div></body></html>"#,
            "t",
            "XYZ ABC XYZ",
        );
    }

    // ── R4919：max-content 文本段按自身 computed font 度量（嵌套异字号） ──
    // R4919（css-sizing-3 #max-content-inline-size）

    /// 嵌套异字号：容器 12px > 子 13px，max-content = 2×13=26（旧实现 2×12=24 低估
    /// → shrink-to-fit 盒窄于内容 → 内层 IFC 二次折行竖排，baidu「更多」实证）。
    #[test]
    fn r4919_nested_font_size_measured_at_own_font() {
        let doc = zero_dom::parse_html(
            r#"<html><body><div id="t" style="font-size:12px"><a style="font-size:13px">更多</a></div></body></html>"#,
        );
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let tid = doc.get_element_by_id("t").expect("target #t");
        let w = text_content_max_width(tid, &doc, &styles);
        assert!(
            (w - 26.0).abs() < 1.0,
            "nested 13px text must measure 2×13=26 at its own font, got {w}"
        );
    }

    // ── R4920：float 纯文本臂的非文本子贡献（定宽原子 inline + inline frame/margin）──
    // R4920（css-sizing-3 §max-content + CSS2 §10.3.5）

    /// a.hot-refresh 形态：float 内 inline-block(width:16px) + span(margin-left:2px)，
    /// 非文本贡献 = 16+2=18（旧实现 0 → float 收缩过窄二次折行「换一换」竖排实证）。
    #[test]
    fn r4920_float_inline_children_non_text_width() {
        let doc = zero_dom::parse_html(
            r#"<html><body><a id="t" style="float:right"><i id="c" style="display:inline-block; width:16px; height:16px"></i><span id="s" style="margin-left:2px">换一换</span></a></body></html>"#,
        );
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let tid = doc.get_element_by_id("t").expect("target #t");
        let w = inline_children_non_text_width(tid, &doc, &styles);
        assert!(
            (w - 18.0).abs() < 1.0,
            "non-text contribution = icon 16 + span margin 2, got {w}"
        );
    }

    /// R4919 端到端（inline-block shrink 臂）：12px inline-block 容器内 13px `<a>更多</a>`，
    /// `block_max_content_width`（经 dom_inline_text_max_width）= 26（旧 24 → 收缩盒窄于
    /// 内层 IFC 行宽 26 → 「多」折行竖排，baidu 顶部导航实证）。
    #[test]
    fn r4919_inline_block_shrink_measures_descendant_font() {
        let w = compute_block_max_content(
            r#"<html><body><div id="t" style="font-size:12px"><a style="font-size:13px">更多</a></div></body></html>"#,
            "t",
        );
        let w = w.expect("inline-block target found");
        assert!(
            (w - 26.0).abs() < 1.0,
            "max-content must measure 2×13=26 at descendant font, got {w}"
        );
    }

    /// R4920 级联镜像：browser 同形 author stylesheet（class 选择器）下非文本贡献仍 = 18。
    #[test]
    fn r4920_cascade_stylesheet_shape() {
        let css = ".hot-refresh{float:right;height:16px}.c-icon{display:inline-block;width:16px;height:16px}.hot-refresh-text{font-size:14px;line-height:14px;margin-left:2px}";
        let doc = zero_dom::parse_html(
            r#"<html><head><style>.hot-refresh{float:right;height:16px}.c-icon{display:inline-block;width:16px;height:16px}.hot-refresh-text{font-size:14px;line-height:14px;margin-left:2px}</style></head><body><a id="t" class="hot-refresh"><i class="c-icon"></i><span class="hot-refresh-text">换一换</span></a></body></html>"#,
        );
        let ss = zero_css_parser::Parser::parse_stylesheet(css);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[ss]);
        let tid = doc.get_element_by_id("t").expect("target #t");
        let mut shape = String::new();
        for child in doc.child_nodes(tid) {
            if let Some(cs) = styles.get(&child) {
                shape.push_str(&format!(
                    "[{:?} d={:?} w={:?}]",
                    doc.get(child).map(|n| match &n.kind {
                        zero_dom::NodeKind::Element(e) => e.tag_name().clone(),
                        _ => "other".to_string(),
                    }),
                    cs.display,
                    cs.width
                ));
            } else {
                shape.push_str("[no-style]");
            }
        }
        let w = inline_children_non_text_width(tid, &doc, &styles);
        assert!((w - 18.0).abs() < 1.0, "cascade non_text={w}, children={shape}");
    }

    /// R4920/R4920b 端到端：float 内 inline-level 子 margin-box 水平求和（Σ=16+2+42=60）
    /// 须**扩**过 taffy 块流 max（子 margin-box）= max(16, 44) = 44 的低估
    /// （css-sizing-3 §shrink-to-fit），且 definite height（16px）的 float 自身子仍按
    /// 行内流单行排布（CSS2 §9.4.1/§9.5；§10.5 显式高不被内容行数覆写）——
    /// baidu「换一换」float 收缩过窄二次折行竖排实证。
    /// 负控制：`ZW_FLOAT_INLINE_SUM=0` → 宽 44（R4920 红）；`ZW_FLOAT_DEFINITE_H_REMEASURE=0`
    /// → 宽 60 但 icon/text 两行（R4920b 红）。
    #[test]
    fn r4920_float_inline_sum_expands_taffy_max_child_estimate() {
        let css = ".hot-refresh{float:right;height:16px}.c-icon{display:inline-block;width:16px;height:16px}.hot-refresh-text{font-size:14px;line-height:14px;margin-left:2px}";
        let doc = zero_dom::parse_html(
            r#"<html><head><style>.hot-refresh{float:right;height:16px}.c-icon{display:inline-block;width:16px;height:16px}.hot-refresh-text{font-size:14px;line-height:14px;margin-left:2px}</style></head><body><div class="hdr" style="width:784px"><a id="t" class="hot-refresh"><i id="ico" class="c-icon"></i><span id="txt" class="hot-refresh-text">换一换</span></a></div></body></html>"#,
        );
        let ss = zero_css_parser::Parser::parse_stylesheet(css);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[ss]);
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
        let float_box = find("t", &doc, &result.root).expect("float #t found");
        assert!(
            (float_box.width - 60.0).abs() < 1.0,
            "float width must expand to inline-sum 60 (taffy max-child 44), got {}",
            float_box.width
        );
        let ico = find("ico", &doc, &result.root).expect("icon #ico found");
        let txt = find("txt", &doc, &result.root).expect("text #txt found");
        assert!(
            (txt.y - ico.y).abs() < 1.0,
            "float 内 inline-level 子须单行排布（R4920b definite-height float 重测），got icon y={} text y={}",
            ico.y,
            txt.y
        );
    }

    // ── R4921：定宽原子 inline 子停止文本递归（Σ 双计修复）──
    // R4921（css-sizing-3 §max-content：原子 inline 贡献盒外尺寸，内文在原子盒内自行
    // 折行不外溢）

    /// major-1 钉（端到端）：float(width:auto) 内 span(display:inline-block;
    /// width:100px) 带文本——float 首选宽 = 原子 margin-box = 100（内文在原子盒内
    /// 折行不外溢）。
    /// 回归形态：R4920 Σ 侧计盒宽 100 + walk 递归内文 ≈80 = 180 过测（修复前该形态
    /// 欠测 80 → 方向由欠转超）。负控制：`ZW_INTRINSIC_ATOMIC_GATE=0` → walk 复计内文
    /// → float 过 100（红）；`ZW_FLOAT_INLINE_SUM=0` → Σ=0 → 收缩塌 0（红，异签名）。
    #[test]
    fn r4921_float_definite_atomic_no_text_double_count() {
        let css = ".fa{float:right}.ibx{display:inline-block;width:100px}";
        let doc = zero_dom::parse_html(
            r#"<html><head><style>.fa{float:right}.ibx{display:inline-block;width:100px}</style></head><body><div class="hdr" style="width:784px"><a id="t" class="fa"><span id="ibx" class="ibx">0123456789 abcdefghij</span></a></div></body></html>"#,
        );
        let ss = zero_css_parser::Parser::parse_stylesheet(css);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[ss]);
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
        let float_box = find("t", &doc, &result.root).expect("float #t found");
        assert!(
            (float_box.width - 100.0).abs() < 1.0,
            "float width must equal definite atomic margin-box 100 (inner text must not double-count), got {}",
            float_box.width
        );
    }

    /// auto 宽原子形态不变式：float > inline-block(width:auto) > 文本——Σ 侧计 0
    ///（欠测安全向），内文由 walk 单边计入 → float = 文本 max-content。gate 不得触及
    /// 本形态（定宽 gate 才是双计精确面；gate 全体原子会收缩塌 0）。
    #[test]
    fn r4921_float_auto_atomic_text_single_source() {
        let css = ".fa{float:right}.iby{display:inline-block}";
        let doc = zero_dom::parse_html(
            r#"<html><head><style>.fa{float:right}.iby{display:inline-block}</style></head><body><div class="hdr" style="width:784px"><a id="t" class="fa"><span id="iby" class="iby">northern light</span></a></div></body></html>"#,
        );
        let ss = zero_css_parser::Parser::parse_stylesheet(css);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[ss]);
        let iby_id = doc.get_element_by_id("iby").expect("span #iby");
        let expected = text_content_max_width(iby_id, &doc, &styles);
        assert!(expected > 0.0, "auto atomic text must measure > 0");
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
        let float_box = find("t", &doc, &result.root).expect("float #t found");
        assert!(
            (float_box.width - expected).abs() < 1.0,
            "float width must equal auto-atomic text max-content {expected} (single-source, gate must not fire), got {}",
            float_box.width
        );
    }

    /// F5-①（per-font walk 三层嵌套）：36px 容器 > 39px 中层 > 42px 叶文本——
    /// max-content 按叶自身 font 度量（2×42=84；旧整树容器口径 2×36=72）。
    #[test]
    fn r4919_per_font_three_level_nesting() {
        let doc = zero_dom::parse_html(
            r#"<html><body><div id="t" style="font-size:36px"><span style="font-size:39px"><a style="font-size:42px">更多</a></span></div></body></html>"#,
        );
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[]);
        let tid = doc.get_element_by_id("t").expect("target #t");
        let w = text_content_max_width(tid, &doc, &styles);
        assert!(
            (w - 84.0).abs() < 1.0,
            "3-level nested text must measure 2×42=84 at leaf font, got {w}"
        );
    }

    /// F5-②（R4920b 放行臂 float-only 排除面钉）：definite-height **inline-block**
    ///（非 float）不入 inline-only 重测放行——其 inline-level 子保持 taffy 块堆叠
    ///（icon 行 1 / 文本行 2，y 差 = icon 高 16）。钉排除面防未来改动误伤非 float；
    /// 块堆叠本身是 inline-block IFC 的挂账缺陷（slice12 前余项，vd 臂实证），此处
    /// 钉排除边界非背书堆叠——未来切片修复 inline-block IFC 时须同步更新本钉。
    #[test]
    fn r4921_definite_height_non_float_excluded_from_remeasure() {
        let css = ".ib{display:inline-block;height:16px}.c-icon{display:inline-block;width:16px;height:16px}.t{font-size:14px;line-height:14px;margin-left:2px}";
        let doc = zero_dom::parse_html(
            r#"<html><head><style>.ib{display:inline-block;height:16px}.c-icon{display:inline-block;width:16px;height:16px}.t{font-size:14px;line-height:14px;margin-left:2px}</style></head><body><div style="width:784px"><span id="t" class="ib"><i id="ico" class="c-icon"></i><span id="txt" class="t">换一换</span></span></div></body></html>"#,
        );
        let ss = zero_css_parser::Parser::parse_stylesheet(css);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[ss]);
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
        let ico = find("ico", &doc, &result.root).expect("icon #ico found");
        let txt = find("txt", &doc, &result.root).expect("text #txt found");
        let dy = txt.y - ico.y;
        assert!(
            (dy - 16.0).abs() < 1.0,
            "non-float definite-height inline-block must stay excluded from remeasure (children block-stacked, y diff = icon 16), got {dy}"
        );
    }

    // ── R5034（css-text-3 §3 + css-sizing-3）：max-content 不计行缘可折叠空白 ──
    //
    // slice/clone-nowrap-intrinsic-size 四案根因（R5032/R5033 定谳）：div 源码行首缩进
    // + 行尾换行折叠后各 1 空格（10px mono = 5px/个）被 max-content 计入（+10px），
    // test 页比 ref 页宽 10px → 盒位差。chromium 语义：行缘可折叠空白在 white-space
    // 处理 phase 1 丢弃（nowrap 只禁软换行，不禁行缘空白折叠）。
    //
    // 注意：`<style>` 元素不经本测试路径收集——样式表显式 parse_stylesheet 传入。

    /// 跑完整引擎后对 div#t 直调 block_max_content_width（R5034 观测/断言共用 helper）。
    fn r5034_block_max(html_body: &str, css: &str) -> f32 {
        let doc = zero_dom::parse_html(&format!(r#"<html><head></head><body>{html_body}</body></html>"#));
        let ss = zero_css_parser::Parser::parse_stylesheet(css);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[ss]);
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
        let target = find("t", &doc, &result.root).expect("div#t found");
        block_max_content_width(target, &doc, &styles)
    }

    const R5034_CSS: &str =
        "html,body{font:10px/1 monospace;margin:0;padding:0}div#t{white-space:nowrap;width:max-content}";

    /// 四案同构最小页：div 源码带行缘空白（ws1）vs 无空白（ws2）——max-content 须相等。
    #[test]
    fn r5034_max_content_excludes_line_edge_whitespace() {
        let ws1 = r5034_block_max(
            r#"<div id="t">
  <span>aaa</span><span>aaa</span>
</div>"#,
            R5034_CSS,
        );
        let ws2 = r5034_block_max(r#"<div id="t"><span>aaa</span><span>aaa</span></div>"#, R5034_CSS);
        assert!(
            (ws1 - ws2).abs() < 0.5,
            "line-edge collapsible whitespace must not contribute to max-content (ws1={ws1} ws2={ws2})"
        );
    }

    /// 四案真实 span 形态（padding+border frame）：带行缘空白页 max-content 仍须与
    /// 无空白版相等——frame 分工（loop 计 frame、walk 计文本）不引入行缘空白。
    #[test]
    fn r5034_max_content_excludes_line_edge_whitespace_with_span_frame() {
        let css = &format!("{R5034_CSS}span{{padding:0 10px 0 6px;border-width:0 8px 0 5px;border-style:solid}}");
        let ws1 = r5034_block_max(
            r#"<div id="t">
  <span>aaa</span><span>aaa</span>
</div>"#,
            css,
        );
        let ws2 = r5034_block_max(r#"<div id="t"><span>aaa</span><span>aaa</span></div>"#, css);
        assert!(
            (ws1 - ws2).abs() < 0.5,
            "span frame + text must not double-count nor include line-edge whitespace (ws1={ws1} ws2={ws2})"
        );
    }

    /// R5034 布局级守护：`width:max-content` div 的**布局后盒宽**（经
    /// apply_intrinsic_content_sizing 的 first-pass 叶测量路径 text_content_max_width）
    /// 不得含行缘空白。修复前该路径 ws1=38 vs ws2=33（+5）；最终树单次
    /// block_max_content_width 反而干净——缺陷只在 first-pass 树形态（div 无子）。
    /// 注意：`<style>` 元素不经此测试路径收集（样式表须显式 parse_stylesheet 传入）。
    #[test]
    fn r5034_max_content_layout_width_excludes_line_edge_whitespace() {
        let css = "html,body{font:10px/1 monospace;margin:0;padding:0}div{white-space:nowrap;width:max-content}";
        fn layout_w(html: &str, css: &str) -> f32 {
            let doc = zero_dom::parse_html(html);
            let ss = zero_css_parser::Parser::parse_stylesheet(css);
            let mut sys = zero_style_system::StyleSystem::new();
            sys.set_viewport(800.0, 600.0);
            let styles = sys.compute_styles(&doc, &[ss]);
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
            let t = find("t", &doc, &result.root).expect("div#t");
            t.width
        }
        let w1 = layout_w(
            r#"<html><head></head><body><div id="t">
  <span>aaa</span><span>aaa</span>
</div></body></html>"#,
            css,
        );
        let w2 = layout_w(
            r#"<html><head></head><body><div id="t"><span>aaa</span><span>aaa</span></div></body></html>"#,
            css,
        );
        assert!(
            (w1 - w2).abs() < 0.5,
            "width:max-content div must not include line-edge whitespace in laid-out width (ws1={w1} ws2={w2})"
        );
    }

    // ── R5036（css-sizing-3 §min-content）：真 min-content 测量 ──

    /// 跑完整引擎后对 div#t 直调 block_min_content_width（R5036 断言 helper）。
    fn r5036_block_min(html_body: &str, css: &str) -> f32 {
        let doc = zero_dom::parse_html(&format!(r#"<html><head></head><body>{html_body}</body></html>"#));
        let ss = zero_css_parser::Parser::parse_stylesheet(css);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[ss]);
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
        let target = find("t", &doc, &result.root).expect("div#t found");
        block_min_content_width(target, &doc, &styles)
    }

    const R5036_CSS: &str = "html,body{font:10px/1 monospace;margin:0;padding:0}div#t{white-space:nowrap}";

    /// 纯文本词级 min：min-content = 最宽词（< max-content 全串宽）。
    #[test]
    fn r5036_min_content_word_level_narrower_than_max() {
        let min_w = r5036_block_min(r#"<div id="t">aaa bb cccc</div>"#, R5036_CSS);
        let max_w = {
            let doc = zero_dom::parse_html(r#"<html><head></head><body><div id="t">aaa bb cccc</div></body></html>"#);
            let ss = zero_css_parser::Parser::parse_stylesheet(R5036_CSS);
            let mut sys = zero_style_system::StyleSystem::new();
            sys.set_viewport(800.0, 600.0);
            let styles = sys.compute_styles(&doc, &[ss]);
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
            let t = find("t", &doc, &result.root).unwrap();
            block_max_content_width(t, &doc, &styles)
        };
        assert!(
            min_w < max_w - 0.5,
            "min-content (widest word) must be narrower than max-content (full run), min={min_w} max={max_w}"
        );
    }

    /// fit-content-length-percentage-002 同构：两个 100px inline-block 连写，
    /// min-content = 单个原子盒宽（原子独立断点），width:fit-content(50px) 布局宽 = 100
    /// （公式 min(W_max=200, max(W_min=100, 50))）。
    #[test]
    fn r5036_fit_content_formula_floor_at_min_content() {
        let css = "html,body{font:10px/1 monospace;margin:0;padding:0}div#t{width:fit-content(50px)}";
        let doc = zero_dom::parse_html(
            r#"<html><head></head><body><div id="t"><div style="display:inline-block;width:100px"></div><div style="display:inline-block;width:100px"></div></div></body></html>"#,
        );
        let ss = zero_css_parser::Parser::parse_stylesheet(css);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[ss]);
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
        let t = find("t", &doc, &result.root).expect("div#t found");
        assert!(
            (t.width - 100.0).abs() < 1.0,
            "fit-content(50px) with two 100px inline-blocks must floor at min-content 100, got {}",
            t.width
        );
    }

    /// min-width: fit-content(100px) + width:50px：floor = min(W_max, max(W_min, 100))
    /// ——fit-content-length-percentage-012 同构（2×60 inline-block → 100）。
    #[test]
    fn r5036_min_width_fit_content_floors_formula() {
        let css = "html,body{font:10px/1 monospace;margin:0;padding:0}div#t{width:50px;min-width:fit-content(100px)}";
        let doc = zero_dom::parse_html(
            r#"<html><head></head><body><div id="t"><div style="display:inline-block;width:60px"></div><div style="display:inline-block;width:60px"></div></div></body></html>"#,
        );
        let ss = zero_css_parser::Parser::parse_stylesheet(css);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[ss]);
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
        let t = find("t", &doc, &result.root).expect("div#t found");
        assert!(
            (t.width - 100.0).abs() < 1.0,
            "min-width:fit-content(100px) with two 60px inline-blocks must floor at 100, got {}",
            t.width
        );
    }

    // ── R5037（css-sizing-3 #intrinsic-contribution）：fit-content 贡献传播 ──

    /// 跑完整引擎后取 div#p（min/max-content 父）布局宽（R5037 断言 helper）。
    fn r5037_parent_width(css: &str, inner_html: &str) -> f32 {
        let doc = zero_dom::parse_html(&format!(
            r#"<html><head></head><body><div id="p" style="{css}">{inner_html}</div></body></html>"#
        ));
        let ss = zero_css_parser::Parser::parse_stylesheet("html,body{font:10px/1 monospace;margin:0;padding:0}");
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[ss]);
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
        find("p", &doc, &result.root).expect("div#p found").width
    }

    /// 011 同构：min-content 父 + width:fit-content(100px) 子（2×60 inline-block）——
    /// 子贡献 = 公式值 min(W_max, max(W_min, 100)) = 100，父 min-content = 100 而非内容 max。
    #[test]
    fn r5037_fit_width_child_contributes_formula_value() {
        let w = r5037_parent_width(
            "width:min-content;height:50px",
            r#"<div style="width:fit-content(100px)"><div style="display:inline-block;width:60px"></div><div style="display:inline-block;width:60px"></div></div>"#,
        );
        assert!(
            (w - 100.0).abs() < 1.0,
            "min-content parent with width:fit-content(100px) child must size 100 (formula contribution), got {w}"
        );
    }

    /// 012 同构：min-content 父 + width:50px + min-width:fit-content(100px) 子——贡献 =
    /// max(50, 100) = 100。
    #[test]
    fn r5037_min_width_fit_child_contribution_floors() {
        let w = r5037_parent_width(
            "width:min-content;height:50px",
            r#"<div style="width:50px;min-width:fit-content(100px)"><div style="display:inline-block;width:60px"></div><div style="display:inline-block;width:60px"></div></div>"#,
        );
        assert!(
            (w - 100.0).abs() < 1.0,
            "min-content parent contribution of min-width:fit-content(100px) child must be 100, got {w}"
        );
    }

    /// 013 同构：min-content 父 + width:200px + max-width:fit-content(100px) 子——贡献 =
    /// min(200, 100) = 100。
    #[test]
    fn r5037_max_width_fit_child_contribution_caps() {
        let w = r5037_parent_width(
            "width:min-content;height:50px",
            r#"<div style="width:200px;max-width:fit-content(100px)"><div style="display:inline-block;width:60px"></div><div style="display:inline-block;width:60px"></div></div>"#,
        );
        assert!(
            (w - 100.0).abs() < 1.0,
            "min-content parent contribution of max-width:fit-content(100px) child must be 100, got {w}"
        );
    }

    /// 015 同构：max-content 父 + width:50px + min-width:fit-content(100px) 子——
    /// max-content 贡献同样被 min-width floor 到 100。
    #[test]
    fn r5037_max_content_parent_min_width_fit_child() {
        let w = r5037_parent_width(
            "width:max-content;height:50px",
            r#"<div style="width:50px;min-width:fit-content(100px)"><div style="display:inline-block;width:60px"></div><div style="display:inline-block;width:60px"></div></div>"#,
        );
        assert!(
            (w - 100.0).abs() < 1.0,
            "max-content parent contribution of min-width:fit-content(100px) child must be 100, got {w}"
        );
    }
}
