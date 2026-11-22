//! gCS top/right/bottom/left resolved-value echo——刀 CSSOM-INSETS-USEDECHO
//!（2026-11-22，wptfyi-css OM 流）。从 [`super::computed_style_cache`] 的
//! used-size 族拆出独立模块（inset 族 CB 链解析面自成一体，控制 cache 文件行数）。
//!
//! CSSOM-1 §resolved value：定位元素（position 非 static）的 top/right/bottom/left
//! resolved value = used value——此前静态级联串化（"10%"/"calc(10% - 1px)"/"auto"
//! 直通）是引擎级缺口（insets 语料池 census 44668230656：getComputedStyle-insets-*
//! 六页 761F）。css-position-3 §inset 属性同款 resolved 特例。
//! https://drafts.csswg.org/cssom-1/#resolved-value-special-case-property-like-top
//!
//! v1 face（本刀预注册范围）：position ∈ {absolute, fixed} 语料族
//!（getComputedStyle-insets-absolute/fixed 432F 池）。geometry = 布局落位 rect 读回
//!（[`with_generation_rects_for_selector`] 全节点 rect 表 + cached doc 祖先链 CB
//! 解析）：used inset = 元素 margin-edge 与 CB padding-edge 的物理差——落位 rect
//! 已含布局全部语义（% 基/stretch/static 位/over-constraint 钳），WM 矩阵免疫
//!（inset 是物理轴属性）。sticky/relative/grid/relpos-inline 族留 v2 census 批
//!（relative 残量语义与 sticky CB 口径未定谳，
//! .acceptance/wptfyi-css/NOTES-knife-cssom-insets-usedecho-claim.md）。
//!
//! P2F-safe 门：computed 值含 Percentage ∨ Calc ∨ Auto 叶才咨询——现役通过池
//!（纯 px computed，绝对/定页各 108）静态串化即 resolved，零接触不回读。
//!
//! v2（census 根因③）：over-constraint 臂——insets 致布局忽略一侧时（CSS2
//! §10.3.7/§10.6.4：ltr 忽略 right、rtl 忽略 left、垂直恒忽略 bottom）该侧
//! rect 读回 used≠spec 口径，回落 **computed 绝对化**（% 按 CB 对应轴解析）。

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, LengthValue, PositionValue, TransformValue};
use zero_dom::{Document, NodeId};
use zero_style_system::{BorderStyleValue, ComputedStyle};

use super::DomMutation;
use super::computed_style_cache::{gcs_viewport_dims, with_generation_rects_for_selector};

/// gCS top/right/bottom/left resolved-value echo selector 通道门（v1：absolute/fixed）。
/// 门集任一不满足返 None 回落静态串化零漂移：
/// - prop ∈ {top, right, bottom, left}；display 非 none（CSSOM resolved = computed）；
/// - position ∈ {absolute, fixed}（v1；sticky/relative 留 v2 census 批）；
/// - computed 值 Percentage ∨ Calc ∨ Auto（P2F-safe：纯 px 静态串化即 resolved）。
pub(crate) fn resolved_inset_for_selector(
    sel: &str,
    prop: &str,
    style: &ComputedStyle,
    html: &str,
    drain_gen: usize,
    style_version: usize,
    mutations: impl FnOnce() -> Vec<DomMutation>,
) -> Option<String> {
    if !inset_prop_gate(prop, style) {
        return None;
    }
    let (vw, vh) = gcs_viewport_dims();
    with_generation_rects_for_selector(
        sel,
        html,
        drain_gen,
        style_version,
        mutations,
        |doc, styles, nid, rects| {
            resolved_inset_from_rects(prop, style, doc, styles, nid, &rects, vw as f32, vh as f32)
        },
    )
}

/// 共用门集（selector/handle 双通道同语义）：prop ∈ inset 四属性 + display 非 none
/// + position ∈ {absolute, fixed}（v1）+ computed 值 Percentage ∨ Calc ∨ Auto。
fn inset_prop_gate(prop: &str, style: &ComputedStyle) -> bool {
    if !matches!(prop, "top" | "right" | "bottom" | "left") {
        return false;
    }
    if matches!(style.display, DisplayValue::None) {
        return false;
    }
    if !matches!(style.position, PositionValue::Absolute | PositionValue::Fixed) {
        return false;
    }
    let computed = match prop {
        "top" => &style.top,
        "right" => &style.right,
        "bottom" => &style.bottom,
        _ => &style.left,
    };
    matches!(
        computed,
        LengthValue::Percentage(_) | LengthValue::Calc(_) | LengthValue::Auto
    )
}

/// handle 通道 gCS inset 族 resolved-value echo（s161 createElement 产物 `__n{n}`
/// 身份，gCS selector 缺席走 [`super::computed_style_cache::computed_style_by_handle`]
/// ——insets 语料页面 DOM 全为动态构建，selector 通道不覆盖本池）。门集同
/// [`resolved_inset_for_selector`]（[`inset_prop_gate`]），几何取 pending 视图槽
/// 联合 rect 表（[`super::computed_style_cache::with_pending_view_rects_for_handle`]）；
/// miss 回落 None 静态串化零漂移。
pub(crate) fn resolved_inset_for_handle(
    handle: &str,
    prop: &str,
    style: &ComputedStyle,
    html: &str,
    drain_gen: usize,
    style_version: usize,
    view_gen: usize,
    applied: usize,
    pending: impl FnOnce() -> Vec<DomMutation>,
) -> Option<String> {
    if !inset_prop_gate(prop, style) {
        return None;
    }
    let (vw, vh) = gcs_viewport_dims();
    super::computed_style_cache::with_pending_view_rects_for_handle(
        handle,
        html,
        drain_gen,
        style_version,
        view_gen,
        applied,
        pending,
        |doc, styles, nid, rects| resolved_inset_from_rects(prop, style, doc, styles, nid, rects, vw as f32, vh as f32),
    )
}

/// 布局落位 rect → used inset px（元素 margin-edge − CB padding-edge 物理差）。
/// 任一几何件缺盒/非 Px 回落 None（静态串化零漂移）。
/// v2（census 根因③）：过约束形先经 [`over_constrained_computed_fallback`]
/// 回落 computed 绝对化，不落 rect 读回。
fn resolved_inset_from_rects(
    prop: &str,
    style: &ComputedStyle,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    nid: NodeId,
    rects: &HashMap<u64, (f32, f32, f32, f32)>,
    vw: f32,
    vh: f32,
) -> Option<String> {
    let (x, y, w, h) = *rects.get(&crate::hit_test::node_id_to_u64(nid))?;
    let (cb, cb_rtl) = containing_block_padding_rect(&style.position, nid, doc, styles, rects, vw, vh)?;
    if let Some(v) = over_constrained_computed_fallback(prop, style, cb, cb_rtl) {
        return Some(v);
    }
    // margin edge：四臂全 Px 才展开——abspos auto margin 吸收/居中的 used 形样式侧
    // 不可知，% margin 基未解析，错展开比不展开更糟。
    let mt = length_px(&style.margin_top)?;
    let mr = length_px(&style.margin_right)?;
    let mb = length_px(&style.margin_bottom)?;
    let ml = length_px(&style.margin_left)?;
    let (mx, my, mw, mh) = (x - ml as f32, y - mt as f32, w + (ml + mr) as f32, h + (mt + mb) as f32);
    let (cx, cy, cw, ch) = cb;
    let v = match prop {
        "top" => my - cy,
        "left" => mx - cx,
        "right" => cx + cw - (mx + mw),
        _ => cy + ch - (my + mh),
    };
    Some(format_inset_px(v))
}

/// 过约束检测臂（v2 census 根因③）：CSS2 §10.3.7/§10.6.4——对侧 inset definite
/// 且四 margin 全非 auto 时，本侧 inset（ltr 忽略 right、rtl 忽略 left、垂直恒
/// 忽略 bottom）进入定位方程；尺寸 auto 时由方程解出（解为负钳 0，等式即破），
/// 尺寸 definite 时三件定和≠CB 即破——两种形布局均忽略本侧，rect 读回 used
///（slack 残差/钳零残差）非 CSSOM 口径，resolved 回落 **computed 绝对化**
///（% 按 CB 对应轴、Calc 同 R5465 parent_length 基）。等式恰好成立（非过约束）
/// 时 fallback 值＝rect 读回值（used 本侧=computed 绝对化），双臂同值零漂移；
/// 对侧 auto（吸收 slack）或本侧 computed auto（auto 的 resolved = used rect 读回）
/// 不触。返回 None 落 rect 读回既有臂。
/// https://drafts.csswg.org/cssom-1/#resolved-value-special-case-property-like-top
fn over_constrained_computed_fallback(
    prop: &str,
    style: &ComputedStyle,
    cb: (f32, f32, f32, f32),
    cb_rtl: bool,
) -> Option<String> {
    // 被忽略侧：垂直恒 bottom；水平 ltr=right、rtl=left（CSS2 §10.3.7 过约束）。
    let ignored = match prop {
        "bottom" => true,
        "right" => !cb_rtl,
        "left" => cb_rtl,
        _ => false,
    };
    if !ignored {
        return None;
    }
    let (opposite, axis) = match prop {
        "left" | "right" => (&style.left, cb.2),
        _ => (&style.top, cb.3),
    };
    if matches!(opposite, LengthValue::Auto)
        || [
            &style.margin_left,
            &style.margin_right,
            &style.margin_top,
            &style.margin_bottom,
        ]
        .iter()
        .any(|m| matches!(m, LengthValue::Auto))
    {
        return None;
    }
    let computed = match prop {
        "left" => &style.left,
        "right" => &style.right,
        "top" => &style.top,
        _ => &style.bottom,
    };
    match computed {
        LengthValue::Percentage(p) => Some(format_inset_px(*p as f32 / 100.0 * axis)),
        LengthValue::Calc(expr) => {
            let ctx = zero_css_parser::values::CalcContext {
                parent_length: Some(axis as f64),
                ..Default::default()
            };
            zero_css_parser::values::eval_calc_with_context(expr, &ctx).map(|v| format_inset_px(v as f32))
        }
        // Px 到不了本臂（P2F-safe 门纯 px 静态串化即 resolved，上游已排除）；
        // Auto 的 resolved = used（rect 读回既有臂）。
        _ => None,
    }
}

/// 定位元素包含块 padding 盒 rect（物理轴）：absolute = 最近 positioned/transformed
/// 祖先；fixed = 最近 transformed 祖先（css-transforms-2 §transform-established
/// containing block——positioned 静态链不停 fixed）；链上无 → 初始包含块 = 布局
/// 视口。padding edge = border 盒收 border 宽（四臂 Px 门；border-style none/hidden
/// used 0，csswg #2768 同 used-size 口径）。v2 附带 CB direction（§10.3.7 过约束
/// 忽略侧判定用）。
/// https://drafts.csswg.org/css-position-3/#containing-block
fn containing_block_padding_rect(
    position: &PositionValue,
    nid: NodeId,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    rects: &HashMap<u64, (f32, f32, f32, f32)>,
    vw: f32,
    vh: f32,
) -> Option<((f32, f32, f32, f32), bool)> {
    let mut cur = doc.parent_node(nid);
    while let Some(id) = cur {
        if let Some(cs) = styles.get(&id) {
            let establishes = match position {
                PositionValue::Absolute => cs.position != PositionValue::Static || cs.transform != TransformValue::None,
                PositionValue::Fixed => cs.transform != TransformValue::None,
                _ => false,
            };
            if establishes {
                let (x, y, w, h) = *rects.get(&crate::hit_test::node_id_to_u64(id))?;
                let bt = border_px(&cs.border_top_width, &cs.border_top_style)?;
                let br = border_px(&cs.border_right_width, &cs.border_right_style)?;
                let bb = border_px(&cs.border_bottom_width, &cs.border_bottom_style)?;
                let bl = border_px(&cs.border_left_width, &cs.border_left_style)?;
                let rtl = matches!(cs.direction, zero_style_system::property::types::DirectionValue::Rtl);
                return Some((
                    (x + bl as f32, y + bt as f32, w - (bl + br) as f32, h - (bt + bb) as f32),
                    rtl,
                ));
            }
        }
        cur = doc.parent_node(id);
    }
    Some(((0.0, 0.0, vw, vh), false))
}

/// Px 有限值提取（非 Px/非有限回落 None）。
fn length_px(v: &LengthValue) -> Option<f64> {
    match v {
        LengthValue::Px(p) if p.is_finite() => Some(*p),
        _ => None,
    }
}

/// border-style none/hidden → used 0；其余 Px 门。
fn border_px(wv: &LengthValue, sv: &BorderStyleValue) -> Option<f64> {
    match sv {
        BorderStyleValue::None | BorderStyleValue::Hidden => Some(0.0),
        _ => length_px(wv),
    }
}

/// used inset px 串化——整值 i64（布局 used 取整口径，gBCR/used-size 同）；inset
/// 允许负值，不钳 0。
fn format_inset_px(v: f32) -> String {
    format!("{}px", v.round() as i64)
}
