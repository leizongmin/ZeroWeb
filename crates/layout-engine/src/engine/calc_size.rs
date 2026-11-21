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
//! kill-switch：`ZW_CALC_SIZE_BASIS=0` 整 pass 跳过（回退现行为）。

use super::*;

/// 电池 FACE-MARKER（代码字面 ASCII，b0 剥 face 后 binary 0 命中/b1 命中；
/// 与 style-system 写面捕获侧同串——双臂判别只看 0/非 0）。
#[used]
static CALC_SIZE_S2B_LAYOUT_MARKER: &str = "CALCSIZE-S2 basis-resolve";

/// 栈帧：盒 + 父 content-box 轴长（inline/block）+ 父级样式（flex 主轴判定）。
type Frame<'a> = (
    &'a LayoutBox,
    f32,
    f32,
    Option<&'a ComputedStyle>,
);

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
        let mut stack: Vec<Frame> = vec![(
            root,
            root.width.max(0.0),
            viewport_height.max(0.0),
            None,
        )];
        while let Some((b, cb_w, cb_h, parent_style)) = stack.pop() {
            let inline_frame = b.padding_left + b.padding_right + b.border_left + b.border_right;
            let block_frame = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
            let child_cb_w = (b.width - inline_frame).max(0.0);
            let child_cb_h = (b.height - block_frame).max(0.0);
            let my_style = b.node_id.and_then(|id| styles.get(&id));
            stack.extend(
                b.children
                    .iter()
                    .map(|c| (c, child_cb_w, child_cb_h, my_style)),
            );
            // abspos/fixed 盒跳过（尺寸链另有专 pass；basis 语境不同）
            if b.is_absolute || b.is_fixed {
                continue;
            }
            let Some(id) = b.node_id else { continue };
            let Some(s) = styles.get(&id) else { continue };
            let Some(slots) = s.calc_size_slots.as_ref() else { continue };
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
                    zero_css_parser::values::DisplayValue::Flex
                        | zero_css_parser::values::DisplayValue::InlineFlex
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
                    SlotField::SizeH => st.size.height = dim,
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
    eval_sum_px(&sum, basis_px, cb, font_px, vw, vh)
}

/// calc-size basis 表→px（css-values-5 §8 `<calc-size-basis>`）。
///
/// - 嵌套 calc-size basis（唯一合法函数形）：内层先解——内层 basis 表解析
///   ＋内层 sum `size` 代换取 px，作为外层 basis px（§8 内层嵌套先求值）。
/// - `any` = 0；`stretch` = 父 content 轴长；`fit-content` =
///   min(max-content, max(min-content, 父 content))（css-sizing-3 fit-content
///   公式，arg = 100% ≡ 父 content）；`min-content`/`max-content`/`content`
///   = intrinsic_sizing 既有测量（仅宽度轴——高度轴 fail-closed None 回落
///   b0 现行为。FIXME: block 高度 intrinsic 测量机器挂账随高度 intrinsic
///   刀扩展；1521 b1r 实证宽度测量代高度在 nested/auto 形翻 P2F）。
/// - `auto`：本轴声明 definite px（css-flexbox §flex-basis:auto = 主轴尺寸
///   属性，height 槽/column flex-basis 读 height 声明）；flex-basis 槽无
///   definite 声明落 max-content 测量（automatic main size 近似）；width
///   槽 block 盒 = stretch-fit＝父 content 宽（css-sizing-3 §5.3），其余
///   盒型 shrink-to-fit（fit-content 公式）；height 槽无 definite 声明
///   fail-closed None（块 auto 高度依内容流，无测量机器不近似）。
/// - `<length-percentage>`：parse 后经 `CalcContext` 求值（%/em/ch 等语境
///   齐全，负 basis 已被写面文法门拒收，此处双保险钳非负）；高度轴且父
///   高度 indefinite 的 % 形 fail-closed None（§8 不可解 % 使整个
///   calc-size indefinite → 回落属性自动行为，即 b0 现行为——防
///   auto-container 页形以错值覆写致 P2F）。
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
        "max-content" | "content" if !axis_h => {
            Some(crate::intrinsic_sizing::block_max_content_width(b, doc, styles))
        }
        "min-content" if !axis_h => {
            Some(crate::intrinsic_sizing::block_min_content_width(b, doc, styles))
        }
        "fit-content" if !axis_h => {
            let min_c = crate::intrinsic_sizing::block_min_content_width(b, doc, styles);
            let max_c = crate::intrinsic_sizing::block_max_content_width(b, doc, styles);
            Some(max_c.min(min_c.max(cb.max(0.0))))
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
                    return None;
                }
                if matches!(
                    s.display,
                    zero_css_parser::values::DisplayValue::Block
                ) {
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

/// 父高度 definite 性（高度轴 % basis 语境）：父样式 height 可静态解析
/// 定长 = definite；auto/`%`/关键字/Calc 混合 = indefinite。根（无父样式）
/// 按 definite 处理（保守保增益，不额外 fail-close）。
fn parent_cb_definite_h(parent_style: Option<&ComputedStyle>, font_px: f64) -> bool {
    parent_style.map_or(true, |ps| declared_len_px(&ps.height, font_px).is_some())
}

/// sum 求值：`size` 叶代换 basis px（style-system 单源 ident 边界扫描）→
/// `parse_calc` → `CalcContext`（% = 父 content、em = 本元素字号）→ px。
/// 尺寸槽负值钳 0（§8 sum 可负、尺寸属性取 0）。
fn eval_sum_px(
    sum: &str,
    basis_px: f32,
    cb: f32,
    font_px: f64,
    vw: f64,
    vh: f64,
) -> Option<f32> {
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
        assert_eq!(eval_sum_px("size / 2 - 10%", 500.0, 500.0, 16.0, 800.0, 600.0), Some(200.0));
        // fit-content basis：size/4 + 30px → 500/4 + 30 = 155
        assert_eq!(eval_sum_px("size / 4 + 30px", 500.0, 500.0, 16.0, 800.0, 600.0), Some(155.0));
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

    #[test]
    fn s2b_parts_split_nested() {
        let (basis, sum) = zero_style_system::calc_size_parts("calc-size(calc-size(min-content, 30px), size * 10)").unwrap();
        assert_eq!(basis, "calc-size(min-content, 30px)");
        assert_eq!(sum, "size * 10");
    }
}
