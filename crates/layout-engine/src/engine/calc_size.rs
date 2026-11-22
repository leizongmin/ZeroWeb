//! calc-size() 非 definite 值布局期解析 pass（css-values-5 §8 `calc-size()`）。
//!
//! S2b intrinsic 刀的 layout-engine 半边：style-system 写面把 intrinsic 关键词
//! /`%`/嵌套 indefinite basis 的 `size` 词元形声明原文承载进
//! [`ComputedStyle::calc_size_slots`]（写面 [`calc_size_slot_capture`]），本 pass
//! 在首趟布局后的 sizing pass 组消费——basis 表→px（min/max-content 用
//! intrinsic_sizing 既有测量、fit-content 公式、stretch/%/auto 依父 content
//! 轴长）、sum 内 `size` 叶代换 basis px、`parse_calc`+`CalcContext` 数学求值，
//! 恒 px 写 taffy 尺寸（flex_basis/size/min_size/max_size）后统一重跑。
//!
//! 与 R54xx `apply_flex_item_base_authority` 的次序：本 pass 在其后——base
//! authority 门读 CSS `FlexBasisValue::Auto`（calc-size 原文 parse 失败保持
//! Auto），若先行会以 content 基准覆写本 pass 的解析 basis；后行则本 pass
//! 写入胜出（slot 形声明优先级高于 auto-base 权威修正）。
//!
//! kill-switch：`ZW_CALC_SIZE_BASIS=0` 整 pass 跳过（回退现行为）；
//! `ZW_CALC_SIZE_FLEX_WRAP_FREEZE=0` 仅关 flex 主轴 wrap 单行冻结（H2 刀）。

use super::*;

/// 电池 FACE-MARKER（代码字面 ASCII，b0 剥 face 后 binary 0 命中/b1 命中；
/// 与 style-system 写面捕获侧同串——双臂判别只看 0/非 0）。
#[used]
static CALC_SIZE_S2B_LAYOUT_MARKER: &str = "CALCSIZE-S2 basis-resolve";

/// H 刀 FACE-MARKER（高度轴 intrinsic 测量机器，b0 剥 face 0 命中/b1 命中；
/// ASCII 代码字面——FACE-MARKER 必须代码字面 ASCII 教训）。
#[used]
static H_KNIFE_HEIGHT_MARKER: &str = "HKNIFE-H block-content-height";

/// H2 刀 FACE-MARKER（flex 主轴 wrap 单行冻结，b0 剥 face 0 命中/b1 命中；
/// ASCII 代码字面——FACE-MARKER 必须代码字面 ASCII 教训）。
#[used]
static H2_KNIFE_WRAP_FREEZE_MARKER: &str = "H2KNIFE flex-main-wrap-freeze";

/// 栈帧：盒 + 父 content-box 轴长（inline/block）+ 父级样式（flex 主轴判定）。
type Frame<'a> = (&'a LayoutBox, f32, f32, Option<&'a ComputedStyle>);

impl LayoutEngine {
    /// calc-size 承载槽布局期解析（css-values-5 §8 `calc-size()`，S2b）。
    ///
    /// 首趟布局后走 LayoutBox 树：节点样式带 calc_size_slots 时逐槽解析
    /// basis→px、sum 代换求值，写 taffy 对应尺寸字段（后声明胜出已由写面
    /// 清位口径保证槽新鲜）。abspos/fixed 盒跳过（abspos 尺寸链另有 §10.3.x
    /// 修正 pass，basis 语境不同）。返回是否有节点被修改（调用方并入
    /// sizing pass 组统一重跑）。
    pub(super) fn apply_calc_size_basis_resolution(
        taffy_tree: &mut TaffyTree<NodeId>,
        root: &LayoutBox,
        dom_to_taffy: &HashMap<NodeId, taffy::NodeId>,
        styles: &HashMap<NodeId, ComputedStyle>,
        doc: &Document,
        viewport_width: f32,
        viewport_height: f32,
    ) -> bool {
        if std::env::var("ZW_CALC_SIZE_BASIS").as_deref() == Ok("0") {
            return false;
        }
        let mut changed = false;
        let mut stack: Vec<Frame> = vec![(root, root.width.max(0.0), viewport_height.max(0.0), None)];
        while let Some((b, cb_w, cb_h, parent_style)) = stack.pop() {
            let inline_frame = b.padding_left + b.padding_right + b.border_left + b.border_right;
            let block_frame = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
            let child_cb_w = (b.width - inline_frame).max(0.0);
            let child_cb_h = (b.height - block_frame).max(0.0);
            let my_style = b.node_id.and_then(|id| styles.get(&id));
            stack.extend(b.children.iter().map(|c| (c, child_cb_w, child_cb_h, my_style)));
            // abspos/fixed 盒跳过（尺寸链另有专 pass；basis 语境不同）
            if b.is_absolute || b.is_fixed {
                continue;
            }
            let Some(id) = b.node_id else { continue };
            let Some(s) = styles.get(&id) else { continue };
            let Some(slots) = s.calc_size_slots.as_ref() else {
                continue;
            };
            let Some(&tid) = dom_to_taffy.get(&id) else { continue };
            let Ok(mut st) = taffy_tree.style(tid).cloned() else {
                continue;
            };
            // sum/basis 内 % 与 em 的求值语境：em 用本元素字号；% 依解析轴
            // 父 content 长度（width/min/max-width/flex-basis row = cb_w，
            // height/min/max-height/flex-basis column = cb_h）。
            let font_px = zero_style_system::computed::resolve_length(&s.font_size, 16.0, None, None);
            // flex-basis 主轴：父 flex column/reverse-column 主轴 = block（cb_h）
            let flex_main_cb_h = parent_style.is_some_and(|ps| {
                matches!(
                    ps.display,
                    zero_css_parser::values::DisplayValue::Flex | zero_css_parser::values::DisplayValue::InlineFlex
                ) && matches!(
                    ps.flex_direction,
                    zero_css_parser::values::FlexDirectionValue::Column
                        | zero_css_parser::values::FlexDirectionValue::ColumnReverse
                )
            });
            let mut touched = false;
            let set = |px: f32, slot: SlotField, st: &mut taffy::style::Style| {
                let dim = taffy::style::Dimension::length(px);
                match slot {
                    SlotField::FlexBasis => st.flex_basis = dim,
                    SlotField::SizeW => st.size.width = dim,
                    SlotField::SizeH => {
                        st.size.height = dim;
                        // H2 刀（flex 主轴 wrap 单行冻结，css-values-5 §8 ×
                        // css-flexbox-1 §9.9.1）：calc-size() 主尺寸在
                        // available main size 判定语境不定形 ⇒ 换行不可发
                        // 生（全 item 单行）；taffy 把本 pass 写入的
                        // definite 主尺寸直接当换行空间，列向 wrap 容器会
                        // 假换行（calc-size-flex-008：min(size,100px) 定尺
                        // 100 → 60+60 假两行 ✗2.08%；不定形语义应为单行
                        // ＋行内依最终 definite 高收缩 50/50）。行向容器
                        // 换行空间来自父级可用宽度，与本写入正交不受影响。
                        if flex_main_wrap_freeze(s) {
                            st.flex_wrap = taffy::style::FlexWrap::NoWrap;
                        }
                    }
                    SlotField::MinW => st.min_size.width = dim,
                    SlotField::MinH => st.min_size.height = dim,
                    SlotField::MaxW => st.max_size.width = dim,
                    SlotField::MaxH => st.max_size.height = dim,
                }
            };
            for (raw, field, axis_h) in [
                (slots.flex_basis.as_ref(), SlotField::FlexBasis, flex_main_cb_h),
                (slots.width.as_ref(), SlotField::SizeW, false),
                (slots.height.as_ref(), SlotField::SizeH, true),
                (slots.min_width.as_ref(), SlotField::MinW, false),
                (slots.min_height.as_ref(), SlotField::MinH, true),
                (slots.max_width.as_ref(), SlotField::MaxW, false),
                (slots.max_height.as_ref(), SlotField::MaxH, true),
            ] {
                let Some(raw) = raw else { continue };
                let cb = if axis_h { cb_h } else { cb_w };
                let Some(px) = resolve_slot_px(
                    b,
                    doc,
                    styles,
                    s,
                    raw,
                    cb,
                    font_px,
                    axis_h,
                    field == SlotField::FlexBasis,
                    parent_cb_definite_h(parent_style, font_px),
                    viewport_width as f64,
                    viewport_height as f64,
                ) else {
                    continue;
                };
                set(px, field, &mut st);
                touched = true;
            }
            if touched {
                let _ = taffy_tree.set_style(tid, st);
                let _ = taffy_tree.mark_dirty(tid);
                changed = true;
            }
        }
        changed
    }
}

/// taffy 尺寸字段槽位（写面物理属性名一一对应）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum SlotField {
    FlexBasis,
    SizeW,
    SizeH,
    MinW,
    MinH,
    MaxW,
    MaxH,
}

/// 单槽解析：basis 表→px ＋ sum `size` 代换 ＋ 数学求值（§8 used-value 期）。
///
/// 任一步不可解（intrinsic 高度无测量机器、% basis 无父长语境等）返回
/// None——该槽保持现行为（fail-closed 零漂移），不做近似。
#[allow(clippy::too_many_arguments)]
fn resolve_slot_px(
    b: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    s: &ComputedStyle,
    raw: &str,
    cb: f32,
    font_px: f64,
    axis_h: bool,
    is_flex_basis_slot: bool,
    parent_cb_definite: bool,
    vw: f64,
    vh: f64,
) -> Option<f32> {
    let (basis, sum) = zero_style_system::calc_size_parts(raw)?;
    let basis_px = resolve_basis_px(
        b,
        doc,
        styles,
        s,
        &basis,
        cb,
        font_px,
        axis_h,
        is_flex_basis_slot,
        parent_cb_definite,
        vw,
        vh,
    )?;
    // H 刀（高度轴 % 语境）：父高 indefinite 时 sum 内 % 解析 0
    //（css-values-5 §calc-size；auto 容器 `calc-size(any, 31%)` 期望
    // 0px 实证）——旧径以首趟布局内容高近似语境（31% × 内容 10px = 3px
    // 伪值）。basis % 形既有 None 守卫不动（`calc-size(31%, size)` auto
    // 容器回落 auto 行为＝期望 10px）。
    let sum_cb = if axis_h && !parent_cb_definite { 0.0 } else { cb };
    eval_sum_px(&sum, basis_px, sum_cb, font_px, vw, vh)
}

/// calc-size basis 表→px（css-values-5 §8 `<calc-size-basis>`）。
///
/// - 嵌套 calc-size basis（唯一合法函数形）：内层先解——内层 basis 表解析
///   ＋内层 sum `size` 代换取 px，作为外层 basis px（§8 内层嵌套先求值）。
/// - `any` = 0；`stretch` = 父 content 轴长；`fit-content` =
///   min(max-content, max(min-content, 父 content))（css-sizing-3 fit-content
///   公式，arg = 100% ≡ 父 content）；`min-content`/`max-content`/`content`
///   = intrinsic_sizing 既有测量（宽度轴）；高度轴＝block_content_height
///   内容高测量（H 刀，css-sizing-3 intrinsic block-size；1521 b1r 实证
///   宽度测量代高度翻 P2F——禁宽度代高）。
/// - `auto`：本轴声明 definite px（css-flexbox §flex-basis:auto = 主轴尺寸
///   属性，height 槽/column flex-basis 读 height 声明）；flex-basis 槽无
///   definite 声明落 max-content 测量（automatic main size 近似）；width
///   槽 block 盒 = stretch-fit＝父 content 宽（css-sizing-3 §5.3），其余
///   盒型 shrink-to-fit（fit-content 公式）；height 槽无 definite 声明
///   走 block_content_height 内容高测量（H 刀）。
/// - `<length-percentage>`：parse 后经 `CalcContext` 求值（%/em/ch 等语境
///   齐全，负 basis 已被写面文法门拒收，此处双保险钳非负）；高度轴且父
///   高度 indefinite 的 % 形 fail-closed None（§8 不可解 % 使整个
///   calc-size indefinite → 回落属性自动行为，即 b0 现行为——防
///   auto-container 页形以错值覆写致 P2F）。
#[allow(clippy::too_many_arguments)]
fn resolve_basis_px(
    b: &LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    s: &ComputedStyle,
    basis: &str,
    cb: f32,
    font_px: f64,
    axis_h: bool,
    is_flex_basis_slot: bool,
    parent_cb_definite: bool,
    vw: f64,
    vh: f64,
) -> Option<f32> {
    let lb = basis.trim();
    if lb.starts_with("calc-size(") {
        let (inner_basis, inner_sum) = zero_style_system::calc_size_parts(lb)?;
        let inner_px = resolve_basis_px(
            b,
            doc,
            styles,
            s,
            &inner_basis,
            cb,
            font_px,
            axis_h,
            is_flex_basis_slot,
            parent_cb_definite,
            vw,
            vh,
        )?;
        return eval_sum_px(&inner_sum, inner_px, cb, font_px, vw, vh);
    }
    match lb {
        "any" => Some(0.0),
        "stretch" => Some(cb.max(0.0)),
        "max-content" | "content" if !axis_h => Some(crate::intrinsic_sizing::block_max_content_width(b, doc, styles)),
        "min-content" if !axis_h => Some(crate::intrinsic_sizing::block_min_content_width(b, doc, styles)),
        "fit-content" if !axis_h => {
            let min_c = crate::intrinsic_sizing::block_min_content_width(b, doc, styles);
            let max_c = crate::intrinsic_sizing::block_max_content_width(b, doc, styles);
            Some(max_c.min(min_c.max(cb.max(0.0))))
        }
        // H 刀（高度轴 intrinsic 测量机器）：块容器 intrinsic block-size＝
        // 流内块子高和（css-sizing-3 #intrinsic-sizes-_block）；fit-content
        // v1 塌缩内容高（分片语境 FIXME 随 fragmentation 刀扩）。
        "max-content" | "content" | "min-content" | "fit-content" if axis_h => {
            let content = block_content_height(b, styles, font_px)?;
            let frame = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
            Some(basis_h_in_own_dimension(content, s, frame))
        }
        "auto" => {
            // 本轴声明 definite px（auto basis = 主轴尺寸属性语义）。
            let decl = if axis_h { &s.height } else { &s.width };
            if let Some(px) = declared_len_px(decl, font_px) {
                return Some(px.max(0.0) as f32);
            }
            // flex-basis 槽 auto = automatic main size（content 基准近似）；
            // width 槽 block 盒 auto = stretch-fit＝父 content 宽（css-sizing-3
            // §5.3 stretch-fit），inline-block/浮动 = shrink-to-fit（fit-content
            // 公式）；height 槽无 definite 声明 fail-closed None（块高度 auto
            // 依内容流，无测量机器——见上 FIXME，回落 b0 现行为防错值 P2F）。
            if !is_flex_basis_slot {
                if axis_h {
                    // H 刀：块 auto 高＝内容高测量（css-sizing-3 §5.3 auto
                    // block-size content-based；基按 box-sizing 语义维——
                    // box-sizing 页 border-box 19px 锚）。
                    let content = block_content_height(b, styles, font_px)?;
                    let frame = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
                    return Some(basis_h_in_own_dimension(content, s, frame));
                }
                if matches!(s.display, zero_css_parser::values::DisplayValue::Block) {
                    return Some(cb.max(0.0));
                }
                let min_c = crate::intrinsic_sizing::block_min_content_width(b, doc, styles);
                let max_c = crate::intrinsic_sizing::block_max_content_width(b, doc, styles);
                return Some(max_c.min(min_c.max(cb.max(0.0))));
            }
            Some(crate::intrinsic_sizing::block_max_content_width(b, doc, styles))
        }
        _ => {
            if axis_h && !parent_cb_definite && lb.contains('%') {
                return None;
            }
            let expr = zero_css_parser::values::parse_calc(&format!("calc({lb})"))?;
            let ctx = zero_css_parser::values::CalcContext {
                parent_length: Some(cb as f64),
                font_size: Some(font_px),
                viewport_width: Some(vw),
                viewport_height: Some(vh),
                ..Default::default()
            };
            let px = zero_css_parser::values::eval_calc_with_context(&expr, &ctx)?;
            (px.is_finite() && px >= 0.0).then_some(px as f32)
        }
    }
}

/// width 声明 definite px 读取（auto basis 臂）：关键字/`%`/Calc 混合面
/// None（Calc 混合 % 需容器语境，declared 读法只收可静态解析的定长）。
fn declared_len_px(v: &zero_css_parser::values::LengthValue, font_px: f64) -> Option<f64> {
    match v {
        LengthValue::Auto
        | LengthValue::Percentage(_)
        | LengthValue::MinContent
        | LengthValue::MaxContent
        | LengthValue::FitContent(_)
        | LengthValue::Stretch
        | LengthValue::Calc(_) => None,
        other => {
            let px = zero_style_system::computed::resolve_length(other, font_px, None, None);
            (px.is_finite() && px >= 0.0).then_some(px)
        }
    }
}

/// H 刀：块容器内容高测量（css-sizing-3 #intrinsic-sizes-_block）——流内
/// 块级子盒 border-box 高累加。definite px 子直读（box-sizing 语义维）；
/// auto 子递归内容和＋自身 frame；inline 级/浮动/含 calc-size 槽子/匿名盒
/// v1 fail-closed None（无行盒机器不近似；1521 b1r 实证宽度代高翻 P2F——
/// 禁宽度代高）。子 margin v1 不计（collapse 未建模，靶面无 margin 形）。
fn block_content_height(b: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>, font_px: f64) -> Option<f32> {
    let mut total = 0.0f32;
    for child in &b.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        let cid = child.node_id?;
        let cs = styles.get(&cid)?;
        if matches!(cs.display, zero_css_parser::values::DisplayValue::None) {
            continue;
        }
        if matches!(
            cs.position,
            zero_css_parser::values::PositionValue::Absolute | zero_css_parser::values::PositionValue::Fixed
        ) {
            continue;
        }
        if !matches!(cs.display, zero_css_parser::values::DisplayValue::Block)
            || cs.float != zero_css_parser::values::FloatValue::None
            || cs.calc_size_slots.is_some()
        {
            // inline 级原子盒/浮动/calc-size 槽子：v1 无机器，fail-closed。
            return None;
        }
        let frame = child.padding_top + child.padding_bottom + child.border_top + child.border_bottom;
        let h = if let Some(px) = declared_len_px(&cs.height, font_px) {
            if matches!(cs.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox) {
                // border-box 声明即 border-box 高（frame 不足钳底 frame）。
                (px as f32).max(frame)
            } else {
                px as f32 + frame
            }
        } else {
            block_content_height(child, styles, font_px)? + frame
        };
        total += h;
    }
    Some(total)
}

/// H 刀：基值换算到属性 box-sizing 语义维（css-values-5 §calc-size basis
/// 按对应 sizing 属性同法解析；box-sizing 页 border-box `calc-size(auto,
/// size)` 期望 19px＝内容 7＋frame 12 实证基含 frame；intrinsic 关键词同
/// 规则——fit-content border-box 期望 38px＝19×2）。content-box 维原样。
fn basis_h_in_own_dimension(content: f32, s: &ComputedStyle, frame: f32) -> f32 {
    if matches!(s.box_sizing, zero_css_parser::values::BoxSizingValue::BorderBox) {
        (content + frame).max(0.0)
    } else {
        content.max(0.0)
    }
}

/// H2 刀：flex 主轴 wrap 容器 height 槽 calc-size 写入的单行冻结判定
/// （css-values-5 §8 `calc-size()` × css-flexbox-1 §9.9.1）。列向 wrap
/// flex 容器 = 主轴换行空间来自自身 definite 高，calc-size 派生的定尺须
/// 冻结单行（换行不定形语义）；行向容器换行空间来自父级可用宽，不冻结。
/// kill-switch：`ZW_CALC_SIZE_FLEX_WRAP_FREEZE=0` 关闭（回退 H 刀行为）。
fn flex_main_wrap_freeze(s: &ComputedStyle) -> bool {
    if std::env::var("ZW_CALC_SIZE_FLEX_WRAP_FREEZE").as_deref() == Ok("0") {
        return false;
    }
    use zero_css_parser::values::{DisplayValue, FlexDirectionValue, FlexWrapValue};
    matches!(s.display, DisplayValue::Flex | DisplayValue::InlineFlex)
        && matches!(
            s.flex_direction,
            FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
        )
        && !matches!(s.flex_wrap, FlexWrapValue::Nowrap)
}

/// 父高度 definite 性（高度轴 % basis 语境）：父样式 height 可静态解析
/// 定长 = definite；auto/`%`/关键字/Calc 混合 = indefinite。根（无父样式）
/// 按 definite 处理（保守保增益，不额外 fail-close）。
fn parent_cb_definite_h(parent_style: Option<&ComputedStyle>, font_px: f64) -> bool {
    parent_style.is_none_or(|ps| declared_len_px(&ps.height, font_px).is_some())
}

/// sum 求值：`size` 叶代换 basis px（style-system 单源 ident 边界扫描）→
/// `parse_calc` → `CalcContext`（% = 父 content、em = 本元素字号）→ px。
/// 尺寸槽负值钳 0（§8 sum 可负、尺寸属性取 0）。
fn eval_sum_px(sum: &str, basis_px: f32, cb: f32, font_px: f64, vw: f64, vh: f64) -> Option<f32> {
    let patched = zero_style_system::calc_size_substitute_size(sum, basis_px as f64);
    let expr = zero_css_parser::values::parse_calc(&format!("calc({patched})"))?;
    let ctx = zero_css_parser::values::CalcContext {
        parent_length: Some(cb as f64),
        font_size: Some(font_px),
        viewport_width: Some(vw),
        viewport_height: Some(vh),
        ..Default::default()
    };
    let px = zero_css_parser::values::eval_calc_with_context(&expr, &ctx)?;
    (px.is_finite()).then_some((px.max(0.0)) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    // §8 basis 表六形＋sum 代换端到端算术钉（页面真值表：
    // calc-size-flex-basis-on-row.html，Ahem 30ch=600/20ch=400/容器 500px）
    #[test]
    fn s2b_sum_eval_px_forms() {
        // px basis：calc-size(83px, size * 3) → 249
        assert_eq!(eval_sum_px("size * 3", 83.0, 500.0, 16.0, 800.0, 600.0), Some(249.0));
        // stretch basis：size/2 - 10% → 500/2 - 50 = 200
        assert_eq!(
            eval_sum_px("size / 2 - 10%", 500.0, 500.0, 16.0, 800.0, 600.0),
            Some(200.0)
        );
        // fit-content basis：size/4 + 30px → 500/4 + 30 = 155
        assert_eq!(
            eval_sum_px("size / 4 + 30px", 500.0, 500.0, 16.0, 800.0, 600.0),
            Some(155.0)
        );
        // % 对容器：31% × 500 = 155
        assert_eq!(eval_sum_px("31%", 0.0, 500.0, 16.0, 800.0, 600.0), Some(155.0));
        // em 对字号：15em × 20px = 300
        assert_eq!(eval_sum_px("15em", 0.0, 500.0, 20.0, 800.0, 600.0), Some(300.0));
        // 负值钳 0
        assert_eq!(eval_sum_px("size - 1000px", 83.0, 500.0, 16.0, 800.0, 600.0), Some(0.0));
    }

    #[test]
    fn s2b_substitute_size_leaf_boundaries() {
        // ident 边界不误伤：font-size/min-size 不代换；--size 自定义属性形
        // 依写面口径不展开（ident 连段含 -- 前缀整体保留）。
        let patched = zero_style_system::calc_size_substitute_size("size * font-size", 83.0);
        assert_eq!(patched, "83px * font-size");
        let patched = zero_style_system::calc_size_substitute_size("min-size + size", 40.0);
        assert_eq!(patched, "min-size + 40px");
    }

    // H 刀：基值 box-sizing 语义维换算（box-sizing 页 19px 锚）。
    #[test]
    fn h_knife_basis_own_dimension() {
        let mut s = ComputedStyle::default();
        s.box_sizing = zero_css_parser::values::BoxSizingValue::ContentBox;
        // content-box：内容高原样。
        assert_eq!(basis_h_in_own_dimension(7.0, &s, 12.0), 7.0);
        s.box_sizing = zero_css_parser::values::BoxSizingValue::BorderBox;
        // border-box：内容高＋frame（19px 锚）。
        assert_eq!(basis_h_in_own_dimension(7.0, &s, 12.0), 19.0);
    }

    #[test]
    fn s2b_parts_split_nested() {
        let (basis, sum) =
            zero_style_system::calc_size_parts("calc-size(calc-size(min-content, 30px), size * 10)").unwrap();
        assert_eq!(basis, "calc-size(min-content, 30px)");
        assert_eq!(sum, "size * 10");
    }

    // H2 刀：列向 wrap flex 容器 height 槽冻结单行判定（calc-size-flex-008
    // 驱动形；row 向/nowrap/非 flex 容器不冻结——换行空间来源正交）。
    #[test]
    fn h2_flex_main_wrap_freeze_gate() {
        use zero_css_parser::values::{DisplayValue, FlexDirectionValue, FlexWrapValue};
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Flex;
        s.flex_direction = FlexDirectionValue::Column;
        s.flex_wrap = FlexWrapValue::Wrap;
        // 列向 wrap flex：冻结（驱动案 calc-size-flex-008 形）。
        assert!(flex_main_wrap_freeze(&s));
        s.flex_wrap = FlexWrapValue::WrapReverse;
        // wrap-reverse 同语义（换行空间同源）。
        assert!(flex_main_wrap_freeze(&s));
        s.flex_wrap = FlexWrapValue::Nowrap;
        // nowrap 本就单行，无需冻结。
        assert!(!flex_main_wrap_freeze(&s));
        s.flex_wrap = FlexWrapValue::Wrap;
        s.flex_direction = FlexDirectionValue::Row;
        // 行向：换行空间来自父级可用宽，冻结无意义。
        assert!(!flex_main_wrap_freeze(&s));
        s.flex_direction = FlexDirectionValue::Column;
        s.display = DisplayValue::Block;
        // 非 flex 容器：无 flex 行概念。
        assert!(!flex_main_wrap_freeze(&s));
    }
}
