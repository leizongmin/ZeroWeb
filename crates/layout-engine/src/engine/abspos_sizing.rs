//! abspos 尺寸/居中修正 pass（R831 拆分续：R4988 从 abspos.rs 分出——2000 行规则）。
//!
//! 定位域（CB 解析/insets/static 位/stretch，见 [`super::abspos`]）与本模块分离；
//! 这里收「内容决定型尺寸」修正：height 内容关键字（R3910/R4037）、aspect-ratio
//! 三臂传递（R4987，css-sizing-4 §5.2 + §4.1 automatic minimum·OOF 语义域）、
//! 垂直 margin:auto 居中（R2062/R2068）。

use std::collections::HashMap;

use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use super::abspos::resolve_abspos_real_length;
use crate::types::LayoutBox;

fn resolve_abspos_vcenter_inset(
    value: &zero_css_parser::values::LengthValue,
    font_size: &zero_css_parser::values::LengthValue,
    percentage_basis: f32,
    viewport_width: f32,
    viewport_height: f32,
) -> Option<f32> {
    use zero_css_parser::values::LengthValue;
    match value {
        LengthValue::Percentage(p) => Some(*p as f32 / 100.0 * percentage_basis),
        other => resolve_abspos_real_length(other, font_size, viewport_width, viewport_height),
    }
}

/// R3910：abspos + height 内容关键字（`fit-content`〔bare，parser 映射 MaxContent〕/
/// `max-content` / `min-content` / `fit-content(…)`）shrink-to-fit 修复。
///
/// converter 把 height 关键字映射为 `length(0)`（convert_length_to_dimension 的
/// MinContent/MaxContent 臂，R181c width-gate 语义）→ abspos 盒塌缩为零高
///（driving: css-sizing div-fit-content-auto-margin ×20 @4.79%，`inset:0; margin:auto;
/// block-size:fit-content` 应 shrink-to-fit 到内容高度 200px 后由 auto margin 居中）。
///
/// 修：taffy 后（children 已定位）content_h = max child bottom 相对 content origin，
/// box height < content 高度则 lift 到 content 高度（shrink-to-fit）。须在
/// `recenter_abspos_margin_auto_vertically` **之前**执行——居中方程
/// `leftover = CB − top − bottom − height` 依赖修复后的 height。
/// max-height 关键字 cap（拉伸过高方向）仍归 R2057（apply_calc_size_adjustments）。
pub(super) fn fix_abspos_height_content_keyword(box_node: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    use zero_css_parser::values::LengthValue;
    for child in &mut box_node.children {
        if child.is_absolute
            && let Some(id) = child.node_id
            && let Some(s) = styles.get(&id)
            && matches!(
                &s.height,
                LengthValue::FitContent(_) | LengthValue::MaxContent | LengthValue::MinContent
            )
        {
            let content_top = child.padding_top + child.border_top;
            let pb = child.padding_top + child.padding_bottom + child.border_top + child.border_bottom;
            // R4037（css-position-3 §3 abspos auto-size）：内容底改**子树递归**——中间层
            // 百分比高子（height:100%）在 taffy 首趟因父高塌 0 而自身塌 0，其内容
            // （固定高孙）溢出在外但直接子 bottom = 0，旧实现测得 content_h=0 不抬升
            //（abspos-auto-sizing-fit-content-percentage ×17 @2.08% 同值簇：fit-content
            // 语义下百分比子按 indefinite 解析，fit-content 应取到固定高孙的 100px）。
            // fit-content/max-content 本就按全部内容度量——溢出的固定高后代照样贡献。
            let content_h = subtree_content_bottom(child, content_top);
            let target = content_h + pb;
            if target > 0.0 && child.height < target - 0.5 {
                child.height = target;
                child.content_height = content_h;
            }
        }
        fix_abspos_height_content_keyword(child, styles);
    }
}

/// R4037：子树内容底（相对 `content_top` 的 max(y+height)，仅 in-flow 可见盒）——
/// 供 [`fix_abspos_height_content_keyword`] 度量 fit-content 内容高。跳过
/// abspos/fixed 后代（脱流，不贡献固有高）。
fn subtree_content_bottom(box_node: &LayoutBox, content_top: f32) -> f32 {
    let mut bottom = (box_node.y + box_node.height - content_top).max(0.0);
    for c in &box_node.children {
        if c.is_absolute || c.is_fixed {
            continue;
        }
        bottom = bottom.max(subtree_content_bottom(c, content_top));
    }
    bottom
}

/// R4987（css-sizing-4 §5.2 transferred size suggestion + shrink-to-fit·abspos 臂）：
/// abspos + aspect-ratio 的 auto 轴传递三臂——taffy 0.12 对 OOF 盒 AR 语义不完整：
/// 007 双 auto 塌 19×18（shrink-to-fit + 传递全缺）、013 definite 高 + auto 宽只传
/// 高×ratio=50 不与内容 shrink-to-fit 取大（应 100）、018 definite 宽 + auto 高被
/// 内容撑 200（abspos auto 块轴 = 纯传递，内容不撑，012 在册语义）、008 双 auto +
/// max-height:100% 钳高后未按 ratio 回传宽（200→钳 100→回传 100）。
///
/// spec 映射：① definite 高 + auto 宽 → 宽 = max(高×ratio, automatic minimum)，
/// 地板仅 min-width:auto 时生效（013：max(50, 100)=100；017 min-width:0 纯传递
/// 100，内容 200 不撑）；② definite 宽 + auto 高 → 高 = 宽/ratio **纯传递**
/// （abspos 无 content-based automatic minimum 高地板——012 现绿即此语义，
/// 018 同式收敛）；③ 双 auto → 宽 = shrink-to-fit(max-content 钳 CB 宽)，高 =
/// 宽/ratio，max-height（% 按 CB 高解析）钳高后按 ratio 回传宽（008）。
///
/// 范围限定：非替换（替换走 attr/AR 固有路径）；`contain:size` 不触（内容不参与
/// 尺寸，014 在册）；insets 拉伸（auto + 对侧 inset 全 definite）不触（§10.3.7/
/// §10.6.4 归既有拉伸 pass，021 在册）；fixed 不触（CB=视口，归
/// adjust_fixed_to_viewport 域）；百分比尺寸不触（归 R1227/R1227b 解析 pass，009
/// 在册）。box-sizing:border-box 时 ratio 作用在 border 盒（§4.2），010/011 在册
/// 幂等。内容度量用 `block_max_content_width`（max-content 语境：inline 同行求和 +
/// block 取大）。
/// kill-switch `ZW_ABSPOS_AR=0`。
///
/// // https://drafts.csswg.org/css-sizing-4/#aspect-ratio-auto-sizes
pub(super) fn fix_abspos_aspect_ratio_auto_sizes(
    box_node: &mut LayoutBox,
    doc: &zero_dom::Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    cb: Option<(f32, f32)>,
    viewport: (f32, f32),
) {
    if std::env::var("ZW_ABSPOS_AR").as_deref() == Ok("0") {
        return;
    }
    use zero_css_parser::values::{BoxSizingValue, FloatValue, LengthValue};
    let (vp_w, vp_h) = viewport;
    for child in &mut box_node.children {
        if child.is_absolute
            && !child.is_fixed
            && !child.is_replaced
            && let Some(id) = child.node_id
            && let Some(style) = styles.get(&id)
            && let Some(ratio) = style.aspect_ratio.filter(|&r| r > 0.0)
            && !style.contain.has_size()
        {
            let (cb_w, cb_h) = cb.unwrap_or(viewport);
            let frame_h = child.padding_left + child.padding_right + child.border_left + child.border_right;
            let frame_v = child.padding_top + child.padding_bottom + child.border_top + child.border_bottom;
            let is_bb = matches!(style.box_sizing, BoxSizingValue::BorderBox);
            let definite = |v: &LengthValue| resolve_abspos_real_length(v, &style.font_size, vp_w, vp_h).is_some();
            let stretched_w =
                matches!(style.width, LengthValue::Auto) && definite(&style.left) && definite(&style.right);
            let stretched_h =
                matches!(style.height, LengthValue::Auto) && definite(&style.top) && definite(&style.bottom);
            // max-height 定值（Px 等 real length；% 按 CB 高另行解析）。
            let max_h = |cb_basis: f32| -> Option<f32> {
                resolve_abspos_real_length(&style.max_height, &style.font_size, vp_w, vp_h).or_else(|| {
                    match &style.max_height {
                        LengthValue::Percentage(p) => Some(*p as f32 / 100.0 * cb_basis),
                        _ => None,
                    }
                })
            };
            let w_definite = definite(&style.width);
            let h_definite = definite(&style.height);
            if !stretched_w && !stretched_h && !w_definite && h_definite {
                // 臂①（013/017/019/020）：transferred 宽 = 高×ratio，仅 min-width:auto
                // 时以 content-based automatic minimum 取大（§4.1；min-width:0 无地板
                // ——017 内容 200 不撑应纯传递 100）。地板度量暂用 max-content 近似
                // min-content（二者在在册案同值；可换行 IFC + min-width:auto + 传递值
                // < min-content 场景会过估，RFC 域）。shrink 钳 CB 可用宽。
                let main_h = if is_bb {
                    child.height
                } else {
                    (child.height - frame_v).max(0.0)
                };
                let transferred_basis_w = main_h * ratio;
                let transferred_content_w = if is_bb {
                    (transferred_basis_w - frame_h).max(0.0)
                } else {
                    transferred_basis_w
                };
                let mut target_content_w = transferred_content_w;
                if matches!(style.min_width, LengthValue::Auto) {
                    let floor = crate::intrinsic_sizing::block_max_content_width(child, doc, styles).min(cb_w.max(0.0));
                    target_content_w = target_content_w.max(floor);
                }
                let new_w = target_content_w + frame_h;
                if (child.width - new_w).abs() > 0.5 {
                    child.width = new_w;
                    child.content_width = target_content_w;
                }
            } else if w_definite && !stretched_h && matches!(style.height, LengthValue::Auto) {
                // 臂②（012/018/010/011）：高 = 宽/ratio 传递，仅 min-height:auto 时以
                // content-based automatic minimum 取大（§4.1 近似 = in-flow 子底边，
                // 同 R4986 vertical 臂口径；012 auto 地板 100 胜传递 50，018
                // min-height:0 无地板纯传递 100、内容 200 不撑）。max-height 钳后落盒。
                let basis_w = if is_bb {
                    child.width
                } else {
                    (child.width - frame_h).max(0.0)
                };
                let transferred_basis_h = basis_w / ratio;
                let mut target_content_h = if is_bb {
                    (transferred_basis_h - frame_v).max(0.0)
                } else {
                    transferred_basis_h
                };
                if matches!(style.min_height, LengthValue::Auto) {
                    let child_bottom = child
                        .children
                        .iter()
                        .filter(|c| !c.is_absolute && !c.is_fixed && matches!(c.float, FloatValue::None))
                        .map(|c| c.y + c.height)
                        .fold(0.0_f32, f32::max);
                    target_content_h = target_content_h.max(child_bottom);
                }
                if let Some(mh) = max_h(cb_h) {
                    let max_content_h = if is_bb { (mh - frame_v).max(0.0) } else { mh };
                    target_content_h = target_content_h.min(max_content_h);
                }
                let new_h = target_content_h + frame_v;
                if (child.height - new_h).abs() > 0.5 {
                    child.height = new_h;
                    child.content_height = target_content_h;
                }
            } else if matches!(style.width, LengthValue::Auto)
                && matches!(style.height, LengthValue::Auto)
                && !stretched_w
                && !stretched_h
            {
                // 臂③（007/008）：双 auto shrink-to-fit + 传递 + max-height 钳高回传宽。
                let maxc = crate::intrinsic_sizing::block_max_content_width(child, doc, styles);
                let mut target_content_w = maxc.min(cb_w.max(0.0));
                let basis_w = if is_bb {
                    target_content_w + frame_h
                } else {
                    target_content_w
                };
                let mut target_content_h = if is_bb {
                    (basis_w / ratio - frame_v).max(0.0)
                } else {
                    basis_w / ratio
                };
                if let Some(mh) = max_h(cb_h) {
                    let border_h = target_content_h + frame_v;
                    if border_h > mh + 0.5 {
                        // 钳高后按 ratio 回传宽（min-content 地板不低于此处 shrink 值）。
                        let basis_h = if is_bb { mh } else { (mh - frame_v).max(0.0) };
                        let basis_w2 = basis_h * ratio;
                        target_content_w = if is_bb { (basis_w2 - frame_h).max(0.0) } else { basis_w2 };
                        target_content_h = if is_bb { (mh - frame_v).max(0.0) } else { mh };
                    }
                }
                let new_w = target_content_w + frame_h;
                let new_h = target_content_h + frame_v;
                if (child.width - new_w).abs() > 0.5 || (child.height - new_h).abs() > 0.5 {
                    child.width = new_w;
                    child.content_width = target_content_w;
                    child.height = new_h;
                    child.content_height = target_content_h;
                }
            }
        }
        // 递归：abspos CB 子（is_abspos_cb）成为其后代最近 CB（padding-box 内缘），
        // 否则继承；坐标仅取宽高（本 pass 不动定位）。
        let child_cb = if child.is_abspos_cb {
            Some((
                (child.width - child.border_left - child.border_right).max(0.0),
                (child.height - child.border_top - child.border_bottom).max(0.0),
            ))
        } else {
            cb
        };
        fix_abspos_aspect_ratio_auto_sizes(child, doc, styles, child_cb, viewport);
    }
}

pub(super) fn recenter_abspos_margin_auto_vertically(
    box_node: &mut LayoutBox,
    cb_height: f32,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    if std::env::var("ZW_ABSPOS_VCENTER").as_deref() == Ok("0") {
        return;
    }
    recenter_abspos_vcenter_inner(box_node, cb_height, viewport_width, viewport_height, styles);
}

fn recenter_abspos_vcenter_inner(
    box_node: &mut LayoutBox,
    cb_height: f32,
    viewport_width: f32,
    viewport_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    use zero_css_parser::values::LengthValue;
    for child in &mut box_node.children {
        if (child.is_absolute || child.is_fixed)
            && let Some(s) = child.node_id.and_then(|id| styles.get(&id))
        {
            let mt_auto = matches!(s.margin_top, LengthValue::Auto);
            let mb_auto = matches!(s.margin_bottom, LengthValue::Auto);
            // R2068：仅处理「两侧 margin 均 auto」的垂直居中（taffy 不解此场景，absolute-tables-016
            // 等 driving test 依赖）。单边 auto（mt_auto xor mb_auto）的 §10.6.4 margin 分配
            // **taffy 已正确求解**——禁用本 pass（ZW_ABSPOS_VCENTER=0）实证 max-height-004 单边
            // auto 0.06% PASS；启用则双重应用（taffy 已下移 + 本 pass 再叠加 leftover）致元素
            // 贴底（absolute-non-replaced-max-height-002/003/004/007/009/011 簇）。故单边 auto
            // 交回 taffy，本 pass 仅 both-auto。
            // R2072：放开 height_definite gate——height:auto + top+bottom Px + both-auto 也处理。
            // SET（top_px+half）对两种 height:auto 子场景都对：① 无 max-height stretch 填满 CB
            //（child.height=CB-top-bottom，leftover=0，half=0，SET=top_px no-op，元素已填满 ✓）；
            // ② max-height cap（child.height=capped<stretch，leftover>0，SET 下移居中 ✓）。解
            // max-height-002/007/009/011 簇（height:auto+cap+both-auto，旧 gate 跳过）。
            // R2083：扩到 position:fixed——§10.6.4 对 fixed 同样适用（both-auto + top+bottom Px
            // 垂直居中），CB = 初始包含块（视口，§10.1；transform 祖先例外 ZW 暂不处理，同
            // adjust_fixed_to_viewport 假设）。taffy 对 fixed both-auto 给不一致结果（probe 实证
            // abs_y=0 + mt=100 未居中），故本 pass 接管。effective_cb_height：fixed→viewport，
            // absolute→inherited cb_height（positioned 祖先 padding-box 或 root ICB）。
            // R2085：扩到 Percentage inset——CSS2.1 §10.6.4 百分比 top/bottom 相对 CB height 解析后
            // 与 Px 等价参与方程（absolute-non-replaced-height-013：top/bottom:50%+height:100+
            // margin:auto in 100px CB）。旧 both_v_inset 仅 Px 漏 Percentage → 013 recenter no-op。
            // R3596：扩到 residual real length（em/ch/rem/vw/...），但仍显式拒绝 auto 和 intrinsic；
            // Percentage 继续按 effective_cb_height，而非 resolve_length 的 viewport basis。
            let effective_cb_height = if child.is_fixed { viewport_height } else { cb_height };
            if mt_auto
                && mb_auto
                && let (Some(top_px), Some(bottom_px)) = (
                    resolve_abspos_vcenter_inset(
                        &s.top,
                        &s.font_size,
                        effective_cb_height,
                        viewport_width,
                        viewport_height,
                    ),
                    resolve_abspos_vcenter_inset(
                        &s.bottom,
                        &s.font_size,
                        effective_cb_height,
                        viewport_width,
                        viewport_height,
                    ),
                )
            {
                // §10.6.4：leftover = CB_height − top − bottom − element border-box height
                //（child.height 是 border-box，已含 border/padding，对 box-sizing 均正确）。
                // 两侧 margin 均 auto → 各取 leftover/2。R2085：leftover **不钳零**——CSS2.1
                // §10.6.4 "solve the equation under the constraint that the two margins get equal
                // values" 不限符号；over-constrained（显式 height > CB−insets）时 margin 取负值仍
                // 居中。旧 `.max(0.0)` 把负 leftover 钳零致元素贴 top（013：leftover=−100 钳零→
                // y=50 留上半红；应 mt=mb=−50→y=0 填满 CB）。height:auto stretch/cap 场景 leftover
                // 恒 ≥0，去钳零对其无影响（max-height-002/003/007/009/011 簇零回归）。
                let leftover = effective_cb_height - top_px - bottom_px - child.height;
                let half = leftover / 2.0;
                child.margin_top = half;
                child.margin_bottom = half;
                // R2069：SET（非 +=）目标居中位 child.y = top_px + half。旧 `+= half` 假设 taffy
                // 把元素放在静态位（child.y = top_px），仅对 taffy 不居中的场景（height keyword
                // stretch / table）正确；对 height:Px regular div，taffy 已居中（child.y = top_px
                // + half），+= half 双重应用致贴底（max-height-003）。SET 对两种 taffy 起点都对。
                child.y = top_px + half;
            }
        }
        // 递归：若 child 自身 positioned，其后代 CB = child padding-box height（§10.1）。
        let child_cb_height = if child.is_abspos_cb {
            (child.height - child.border_top - child.border_bottom).max(0.0)
        } else {
            cb_height
        };
        recenter_abspos_vcenter_inner(child, child_cb_height, viewport_width, viewport_height, styles);
    }
}

#[cfg(test)]
mod r2062_tests {
    use super::*;
    // 拆分前同文件作用域遗留：本 mod 亦覆盖定位域函数（stretch/CB 判定，居 abspos.rs）。
    use crate::engine::abspos::*;
    use zero_css_parser::values::LengthValue;
    use zero_style_system::ComputedStyle;

    /// 构造 abspos img（definite height 100）作为 positioned 父（CB height 200）的子。
    /// 函数处理 `box_node.children`，故 img 必须包在父盒里再对父盒调用。
    fn make_parent_with_abspos_img(
        margin_top: LengthValue,
        margin_bottom: LengthValue,
        height: LengthValue,
    ) -> (LayoutBox, HashMap<NodeId, ComputedStyle>) {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, img);
        let mut styles = HashMap::new();
        let mut sp = ComputedStyle::default();
        sp.position = zero_style_system::property::types::PositionValue::Relative;
        styles.insert(parent, sp);
        let mut si = ComputedStyle::default();
        si.top = LengthValue::Px(0.0);
        si.bottom = LengthValue::Px(0.0);
        si.height = height;
        si.margin_top = margin_top;
        si.margin_bottom = margin_bottom;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_absolute: true,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        let parent_box = LayoutBox {
            node_id: Some(parent),
            is_relative: true,
            height: 200.0,
            children: vec![img_box],
            ..Default::default()
        };
        (parent_box, styles)
    }

    /// R2062：abspos + top+bottom Px + height definite + 两侧 auto margin → 居中下移 leftover/2。
    #[test]
    fn r2062_abspos_definite_height_both_auto_margins_center() {
        let (mut parent, styles) =
            make_parent_with_abspos_img(LengthValue::Auto, LengthValue::Auto, LengthValue::Px(100.0));
        // 顶层 cb_height = 父 padding-box = 200（父 positioned，其子的 CB）。
        recenter_abspos_margin_auto_vertically(&mut parent, 200.0, 800.0, 600.0, &styles);
        let img = &parent.children[0];
        // leftover = 200 − 100 = 100；mt=mb 均 auto → 各 50；img 下移 50。
        assert_eq!(img.y, 50.0, "img should shift down by leftover/2 = 50");
        assert_eq!(img.margin_top, 50.0);
        assert_eq!(img.margin_bottom, 50.0);
        assert_eq!(img.height, 100.0, "img height unchanged");
    }

    /// R3910：height 内容关键字（fit-content → MaxContent）塌缩的 abspos 盒在
    /// recenter 之前 lift 到内容高度，随后 both-auto margin 正确居中。
    #[test]
    fn r3910_abspos_height_content_keyword_lift_then_center() {
        let (mut parent, styles) =
            make_parent_with_abspos_img(LengthValue::Auto, LengthValue::Auto, LengthValue::MaxContent);
        // 模拟 taffy 对 height 关键字的 length(0) 映射：盒塌缩为 0 高。
        parent.children[0].height = 0.0;
        // 子内容 200px 高（content bottom = 200）。
        let inner = LayoutBox {
            height: 200.0,
            y: 0.0,
            ..Default::default()
        };
        parent.children[0].children = vec![inner];

        fix_abspos_height_content_keyword(&mut parent, &styles);
        assert_eq!(
            parent.children[0].height, 200.0,
            "collapsed abspos lifts to content height"
        );

        recenter_abspos_margin_auto_vertically(&mut parent, 200.0, 800.0, 600.0, &styles);
        let img = &parent.children[0];
        // leftover = 200 − 0 − 0 − 200 = 0 → 居中 no-op（贴 top）。
        assert_eq!(img.y, 0.0);
    }

    /// R3910：height 内容关键字但盒未塌缩（taffy 拉伸 ≥ 内容高）→ no-op（cap 归 R2057）。
    #[test]
    fn r3910_abspos_height_content_keyword_stretched_untouched() {
        let (mut parent, styles) =
            make_parent_with_abspos_img(LengthValue::Auto, LengthValue::Auto, LengthValue::MaxContent);
        parent.children[0].height = 300.0; // 已被 taffy 拉伸超过内容高
        let inner = LayoutBox {
            height: 200.0,
            y: 0.0,
            ..Default::default()
        };
        parent.children[0].children = vec![inner];

        fix_abspos_height_content_keyword(&mut parent, &styles);
        assert_eq!(parent.children[0].height, 300.0, "stretched box left to R2057 cap");
    }

    /// R2068：仅 margin-top auto（mb 非 auto）→ 本 pass **不再处理**（交回 taffy）。
    /// taffy 已正确解 §10.6.4 单边 auto margin 分配；旧实现双重应用致 max-height-004 簇
    /// 贴底（ZW_ABSPOS_VCENTER=0 实证 taffy 单独 0.06% PASS）。守 recenter 对单边 auto no-op。
    #[test]
    fn r2068_abspos_single_auto_margin_left_to_taffy_top() {
        let (mut parent, styles) =
            make_parent_with_abspos_img(LengthValue::Auto, LengthValue::Px(0.0), LengthValue::Px(100.0));
        recenter_abspos_margin_auto_vertically(&mut parent, 200.0, 800.0, 600.0, &styles);
        let img = &parent.children[0];
        assert_eq!(
            img.y, 0.0,
            "single-auto (mt auto, mb=0) is left to taffy — recenter no-op"
        );
        assert_eq!(img.margin_top, 0.0);
        assert_eq!(img.margin_bottom, 0.0);
    }

    /// R2068：仅 margin-bottom auto → 同上，recenter no-op（交回 taffy）。
    #[test]
    fn r2068_abspos_single_auto_margin_left_to_taffy_bottom() {
        let (mut parent, styles) =
            make_parent_with_abspos_img(LengthValue::Px(0.0), LengthValue::Auto, LengthValue::Px(100.0));
        recenter_abspos_margin_auto_vertically(&mut parent, 200.0, 800.0, 600.0, &styles);
        let img = &parent.children[0];
        assert_eq!(
            img.y, 0.0,
            "single-auto (mt=0, mb auto) is left to taffy — recenter no-op"
        );
        assert_eq!(img.margin_top, 0.0);
        assert_eq!(img.margin_bottom, 0.0);
    }

    /// R2072：height:auto 现也处理（放开 height_definite gate）。synthetic config
    ///（top=0, bottom=0, height=100, cb=200）→ leftover=200-0-0-100=100, half=50,
    /// SET child.y = top_px(0) + half(50) = 50。真实 stretch 场景 taffy 会让 child.height
    /// = CB-top-bottom（填满）→ leftover=0 → SET=top_px no-op（无中心化需要）。
    #[test]
    fn r2072_abspos_height_auto_now_processed() {
        let (mut parent, styles) = make_parent_with_abspos_img(LengthValue::Auto, LengthValue::Auto, LengthValue::Auto);
        recenter_abspos_margin_auto_vertically(&mut parent, 200.0, 800.0, 600.0, &styles);
        let img = &parent.children[0];
        // top=0, bottom=0, child.height=100, cb=200 → leftover=100, half=50, SET y=0+50=50.
        assert_eq!(img.y, 50.0, "height:auto both-auto now centered via SET (R2072)");
        assert_eq!(img.margin_top, 50.0);
        assert_eq!(img.margin_bottom, 50.0);
    }

    /// R2062：递归——positioned 祖先的 padding-box（border-box − border）成为后代 CB。
    #[test]
    fn r2062_recursive_cb_uses_positioned_ancestor_padding_box() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let container = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, container);
        let _ = doc.append_child(container, img);
        let mut styles = HashMap::new();
        // container: relative，border-box height 220，border_top/bottom 各 10 → padding-box 200。
        let mut cs = ComputedStyle::default();
        cs.position = zero_style_system::property::types::PositionValue::Relative;
        styles.insert(container, cs);
        let mut si = ComputedStyle::default();
        si.top = LengthValue::Px(0.0);
        si.bottom = LengthValue::Px(0.0);
        si.height = LengthValue::Px(100.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_absolute: true,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        let container_box = LayoutBox {
            node_id: Some(container),
            is_relative: true,
            // R3902：engine.rs 提取时 positioned 盒 is_abspos_cb=true（手工构造须显式设，
            // Default 为 false——CB height 传播按旗标判定）。
            is_abspos_cb: true,
            height: 220.0,
            border_top: 10.0,
            border_bottom: 10.0,
            children: vec![img_box],
            ..Default::default()
        };
        // root（非 positioned）→ container。顶层 cb_height=999（模拟 viewport），
        // 递归进 container（positioned）后其子（img）CB 应为 container padding-box 200。
        let mut root_box = LayoutBox {
            children: vec![container_box],
            ..Default::default()
        };
        recenter_abspos_margin_auto_vertically(&mut root_box, 999.0, 800.0, 600.0, &styles);
        let img = &root_box.children[0].children[0];
        assert_eq!(
            img.y, 50.0,
            "img centered against padding-box CB 200 (not border-box 220)"
        );
        assert_eq!(img.margin_top, 50.0);
    }

    /// R2083：position:fixed + top+bottom Px + 两侧 auto margin → 垂直居中，CB = 视口
    ///（§10.6.4 + §10.1，非父 cb_height）。R2082 probe 实证旧实现（recenter 仅 is_absolute）
    /// 对 fixed both-auto 给不一致结果（abs_y=0 + mt=100 未居中）。本测试守 fixed 走 viewport CB。
    #[test]
    fn r2083_fixed_both_auto_margins_center_against_viewport() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, img);
        let mut styles = HashMap::new();
        let mut si = ComputedStyle::default();
        si.top = LengthValue::Px(0.0);
        si.bottom = LengthValue::Px(0.0);
        si.height = LengthValue::Px(100.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        si.position = zero_style_system::property::types::PositionValue::Fixed;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_fixed: true,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        // parent height 300；顶层 cb_height=300（模拟 positioned 父），viewport=600（fixed CB）。
        // 若误用 cb_height(300)：leftover=300-0-0-100=200, half=100, y=100。
        // 正确（viewport 600）：leftover=600-0-0-100=500, half=250, y=250。
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            height: 300.0,
            children: vec![img_box],
            ..Default::default()
        };
        recenter_abspos_margin_auto_vertically(&mut parent_box, 300.0, 800.0, 600.0, &styles);
        let img = &parent_box.children[0];
        assert_eq!(
            img.y, 250.0,
            "fixed both-auto centers against viewport (600), not parent cb_height (300)"
        );
        assert_eq!(img.margin_top, 250.0);
        assert_eq!(img.margin_bottom, 250.0);
    }

    /// R3593：position:fixed + auto size + real-length opposing insets stretch against viewport.
    /// The fixed stretch pass previously only accepted Px, so residual `em` insets left the
    /// Taffy-sized fallback box unchanged.
    #[test]
    fn r3593_fixed_auto_size_stretches_with_relative_insets() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let div = doc.create_element("div");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, div);

        let mut styles = HashMap::new();
        let mut style = ComputedStyle::default();
        style.position = zero_style_system::property::types::PositionValue::Fixed;
        style.font_size = LengthValue::Px(20.0);
        style.width = LengthValue::Auto;
        style.height = LengthValue::Auto;
        style.left = LengthValue::Em(1.0);
        style.right = LengthValue::Em(2.0);
        style.top = LengthValue::Em(0.5);
        style.bottom = LengthValue::Em(1.0);
        styles.insert(div, style);

        let div_box = LayoutBox {
            node_id: Some(div),
            is_fixed: true,
            width: 10.0,
            height: 10.0,
            ..Default::default()
        };
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            children: vec![div_box],
            ..Default::default()
        };

        stretch_fixed_to_viewport_size(&mut parent_box, 800.0, 600.0, &styles, false, None);

        let div = &parent_box.children[0];
        assert_eq!(div.width, 740.0, "800 - 20px - 40px");
        assert_eq!(div.height, 570.0, "600 - 10px - 20px");
    }

    /// R4122（csswg #10544）：containment 祖先（contain:layout）捕获 fixed 后代为其 CB——
    /// auto + inset:0 的 fixed 子按 CB padding-box（100×100）stretch，且
    /// adjust_fixed_to_viewport 不扣祖先偏移（x/y 保持树相对 → paint 累加得 CB 位置）。
    /// driving: css-conditional/container-queries no-layout-containment-fixedpos
    ///（97.92%→0.00%）。对照锚：无 containment 时（under=false）视口 stretch 语义不变。
    #[test]
    fn r4122_fixed_stretches_against_containment_cb() {
        use zero_css_parser::values::{DisplayValue, LengthValue};
        use zero_style_system::property::types::PositionValue;

        let mut doc = zero_dom::Document::new();
        let parent = doc.create_element("div");
        let div = doc.create_element("div");

        let mut styles = HashMap::new();
        // containment CB 祖先（contain:layout）
        let mut cb_style = ComputedStyle::default();
        cb_style.contain = zero_style_system::property::types::ContainComputedValue::Layout;
        cb_style.display = DisplayValue::Block;
        styles.insert(parent, cb_style);
        // fixed 子：inset:0 + auto 尺寸
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Fixed;
        style.width = LengthValue::Auto;
        style.height = LengthValue::Auto;
        style.left = LengthValue::Px(0.0);
        style.right = LengthValue::Px(0.0);
        style.top = LengthValue::Px(0.0);
        style.bottom = LengthValue::Px(0.0);
        styles.insert(div, style);

        let fixed_box = LayoutBox {
            node_id: Some(div),
            is_fixed: true,
            width: 100.0,
            height: 0.0,
            fixed_x_insets_all_auto: false,
            fixed_y_insets_all_auto: false,
            ..Default::default()
        };
        // containment CB：100×100 @ 树相对 (8,51)
        let cb_box = LayoutBox {
            node_id: Some(parent),
            x: 8.0,
            y: 51.0,
            width: 100.0,
            height: 100.0,
            children: vec![fixed_box],
            ..Default::default()
        };
        let mut root_box = LayoutBox {
            children: vec![cb_box],
            ..Default::default()
        };

        // adjust：under=true 链（root→cb_box 捕获）→ fixed 子不扣偏移、偏移累积不归零。
        adjust_fixed_to_viewport(&mut root_box, 0.0, 0.0, &styles, false);
        let cb = &root_box.children[0];
        let fixed = &cb.children[0];
        assert!((fixed.x - 0.0).abs() < 0.001, "gated fixed x 保留树相对值");
        assert!((fixed.y - 0.0).abs() < 0.001, "gated fixed y 保留树相对值");

        // stretch：按 CB padding-box 100×100 解析（非视口 800×600）。
        stretch_fixed_to_viewport_size(&mut root_box, 800.0, 600.0, &styles, false, None);
        let cb = &root_box.children[0];
        let fixed = &cb.children[0];
        assert!(
            (fixed.width - 100.0).abs() < 0.001,
            "width = CB 宽 100，实际 {}",
            fixed.width
        );
        assert!(
            (fixed.height - 100.0).abs() < 0.001,
            "height = CB 高 100，实际 {}",
            fixed.height
        );
    }

    /// R2085：Percentage top/bottom inset 被接受并参与居中（相对 effective_cb_height 解析）。
    /// absolute-non-replaced-height-013 谱系：top/bottom:25% + height:50 + margin:auto in CB 200
    /// → top=bottom=50, leftover=200-50-50-50=50, half=25, y=75。旧实现 both_v_inset 仅 Px → no-op。
    #[test]
    fn r2085_abspos_percentage_inset_centers() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, img);
        let mut styles = HashMap::new();
        let mut sp = ComputedStyle::default();
        sp.position = zero_style_system::property::types::PositionValue::Relative;
        styles.insert(parent, sp);
        let mut si = ComputedStyle::default();
        si.top = LengthValue::Percentage(25.0);
        si.bottom = LengthValue::Percentage(25.0);
        si.height = LengthValue::Px(50.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        si.position = zero_style_system::property::types::PositionValue::Absolute;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_absolute: true,
            width: 100.0,
            height: 50.0,
            ..Default::default()
        };
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            is_relative: true,
            height: 200.0,
            children: vec![img_box],
            ..Default::default()
        };
        // cb_height=200：top=bottom=50（25% of 200），leftover=200-50-50-50=50，half=25，y=75。
        recenter_abspos_margin_auto_vertically(&mut parent_box, 200.0, 800.0, 600.0, &styles);
        let img = &parent_box.children[0];
        assert_eq!(
            img.y, 75.0,
            "percentage inset 25% of 200 = 50; centers at top(50)+half(25)=75"
        );
        assert_eq!(img.margin_top, 25.0);
        assert_eq!(img.margin_bottom, 25.0);
    }

    /// R3596：real-length top/bottom inset should participate in abspos vertical
    /// both-auto margin centering after resolving against the element font context.
    #[test]
    fn r3596_abspos_relative_length_inset_centers() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, img);
        let mut styles = HashMap::new();
        let mut sp = ComputedStyle::default();
        sp.position = zero_style_system::property::types::PositionValue::Relative;
        styles.insert(parent, sp);
        let mut si = ComputedStyle::default();
        si.font_size = LengthValue::Px(20.0);
        si.top = LengthValue::Em(1.0);
        si.bottom = LengthValue::Em(2.0);
        si.height = LengthValue::Px(50.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        si.position = zero_style_system::property::types::PositionValue::Absolute;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_absolute: true,
            width: 100.0,
            height: 50.0,
            ..Default::default()
        };
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            is_relative: true,
            height: 200.0,
            children: vec![img_box],
            ..Default::default()
        };
        // top=20, bottom=40, leftover=200-20-40-50=90, half=45, y=65.
        recenter_abspos_margin_auto_vertically(&mut parent_box, 200.0, 800.0, 600.0, &styles);
        let img = &parent_box.children[0];
        assert_eq!(img.y, 65.0);
        assert_eq!(img.margin_top, 45.0);
        assert_eq!(img.margin_bottom, 45.0);
    }

    /// R3596：fixed uses the viewport as its vertical centering CB while resolving
    /// residual real-length insets with the element font context.
    #[test]
    fn r3596_fixed_relative_length_inset_centers_against_viewport() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let img = doc.create_element("img");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, img);
        let mut styles = HashMap::new();
        let mut si = ComputedStyle::default();
        si.font_size = LengthValue::Px(20.0);
        si.top = LengthValue::Em(0.5);
        si.bottom = LengthValue::Em(1.0);
        si.height = LengthValue::Px(100.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        si.position = zero_style_system::property::types::PositionValue::Fixed;
        styles.insert(img, si);
        let img_box = LayoutBox {
            node_id: Some(img),
            is_fixed: true,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            height: 300.0,
            children: vec![img_box],
            ..Default::default()
        };
        // viewport=600: top=10, bottom=20, leftover=470, half=235, y=245.
        recenter_abspos_margin_auto_vertically(&mut parent_box, 300.0, 800.0, 600.0, &styles);
        let img = &parent_box.children[0];
        assert_eq!(img.y, 245.0);
        assert_eq!(img.margin_top, 235.0);
        assert_eq!(img.margin_bottom, 235.0);
    }

    /// R2085：over-constrained（显式 height > CB−insets）both-auto 居中——CSS2.1 §10.6.4
    /// "solve with equal margins" 不钳零；leftover 为负时 margin 取负值仍居中。
    /// absolute-non-replaced-height-013 精确复现：CB 100, top/bottom:50%→50/50, height:100
    /// → leftover=100-50-50-100=−100, half=−50, y=0（填满 CB 顶部）。
    #[test]
    fn r2085_abspos_over_constrained_centers_with_negative_margins() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let parent = doc.create_element("div");
        let div = doc.create_element("div");
        let _ = doc.append_child(root, parent);
        let _ = doc.append_child(parent, div);
        let mut styles = HashMap::new();
        let mut sp = ComputedStyle::default();
        sp.position = zero_style_system::property::types::PositionValue::Relative;
        styles.insert(parent, sp);
        let mut si = ComputedStyle::default();
        si.top = LengthValue::Percentage(50.0);
        si.bottom = LengthValue::Percentage(50.0);
        si.height = LengthValue::Px(100.0);
        si.margin_top = LengthValue::Auto;
        si.margin_bottom = LengthValue::Auto;
        si.position = zero_style_system::property::types::PositionValue::Absolute;
        styles.insert(div, si);
        let div_box = LayoutBox {
            node_id: Some(div),
            is_absolute: true,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        let mut parent_box = LayoutBox {
            node_id: Some(parent),
            is_relative: true,
            height: 100.0,
            children: vec![div_box],
            ..Default::default()
        };
        // cb_height=100：top=bottom=50（50%），height=100，leftover=100-50-50-100=−100，
        // half=−50，y=50+(−50)=0（green 填满 100×100 red CB 顶部，no red 可见）。旧 .max(0.0)
        // 钳零 → half=0 → y=50（仅覆盖下半，上半红可见 → 013 FAIL）。
        recenter_abspos_margin_auto_vertically(&mut parent_box, 100.0, 800.0, 600.0, &styles);
        let child = &parent_box.children[0];
        assert_eq!(
            child.y, 0.0,
            "over-constrained both-auto centers via negative margins: y=0"
        );
        assert_eq!(child.margin_top, -50.0);
        assert_eq!(child.margin_bottom, -50.0);
        assert_eq!(child.height, 100.0, "height unchanged");
    }

    /// R3902（CSS Containment §3.1/§4.1）：contain:layout 盒进入 abspos CB 链——
    /// nested-CB 重定位线程把 CB 更新为该盒 padding-box（positioned 祖先同等）。
    /// driving: contain-layout-006（contain 盒内 abspos bottom:0/right:0 应贴 contain 盒）。
    #[test]
    fn r3902_contain_layout_box_updates_cb_chain() {
        let mut doc = zero_dom::Document::new();
        let root = doc.root();
        let mid = doc.create_element("div"); // contain:layout 的 static 中间层
        let abs = doc.create_element("div");
        let _ = doc.append_child(root, mid);
        let _ = doc.append_child(mid, abs);
        let mut styles = HashMap::new();
        let mut sm = ComputedStyle::default();
        sm.contain = zero_style_system::property::types::ContainComputedValue::Layout;
        styles.insert(mid, sm);
        let mut sa = ComputedStyle::default();
        sa.position = zero_style_system::property::types::PositionValue::Absolute;
        styles.insert(abs, sa);
        let abs_box = LayoutBox {
            node_id: Some(abs),
            is_absolute: true,
            width: 50.0,
            height: 50.0,
            ..Default::default()
        };
        let mut mid_box = LayoutBox {
            node_id: Some(mid),
            is_abspos_cb: true, // engine.rs 提取：contain:layout（非 positioned）
            width: 100.0,
            height: 100.0,
            children: vec![abs_box],
            ..Default::default()
        };
        // cb = (10,10,100,100)（外层 positioned 祖先 padding-box）；abspos 无 inset →
        // static 位不重定位，但 CB 链须穿过 contain 盒更新为其 padding-box（零 border/padding
        // 时 = (0,0)+mid 偏移）。此处直接验证 child_cb 判定分支：is_abspos_cb 的 contain 盒
        // 使 CB 更新发生（旧判据 positioned-only 时 cb 原样继承）。
        resolve_abspos_against_nested_cb_inner(&mut mid_box, 0.0, 0.0, Some((10.0, 10.0, 100.0, 100.0)), &styles, true);
        // contain 盒非 positioned 且有 cb → 重定位分支对 child 生效（child.is_absolute）。
        // top/bottom 均 Auto → 无 inset 重定位，坐标不变；断言递归正常完成且 CB 链可达子节点
        //（无 panic/跳过）即为链路贯通。
        assert_eq!(mid_box.children[0].x, 0.0);
        assert_eq!(mid_box.children[0].y, 0.0);
    }

    /// R4295（filter-effects-1 §3 / filter-effects-2 #BackdropFilterProperty / CSS Transforms
    /// §3 / css-will-change §3）：非 none 的 filter/backdrop-filter/transform/perspective
    ///（含 will-change 提示）建立 abspos/fixed 后代包含块。driving:
    /// backdrop-filter-containing-block（backdrop-filter 容器捕获 fixed/absolute 子）。
    #[test]
    fn r4295_visual_cb_predicates() {
        use zero_css_parser::values::parse_transform::TransformValue;
        use zero_style_system::property::types::{FilterComputedValue, WillChangeValue};

        // 全默认：不建 CB。
        assert!(!creates_cb_for_abspos_descendants(&ComputedStyle::default()));
        assert!(!is_fixed_cb_ancestor(&ComputedStyle::default()));
        // filter / backdrop-filter 非 none。
        let mut s = ComputedStyle::default();
        s.filter = vec![FilterComputedValue::Invert(1.0)];
        assert!(creates_cb_for_abspos_descendants(&s));
        let mut s = ComputedStyle::default();
        s.backdrop_filter = vec![FilterComputedValue::Invert(1.0)];
        assert!(creates_cb_for_abspos_descendants(&s));
        assert!(is_fixed_cb_ancestor(&s));
        // transform 非 none / perspective 非 0。
        let mut s = ComputedStyle::default();
        s.transform = TransformValue::List(Vec::new());
        assert!(creates_cb_for_abspos_descendants(&s));
        let mut s = ComputedStyle::default();
        s.perspective = LengthValue::Px(100.0);
        assert!(creates_cb_for_abspos_descendants(&s));
        // will-change 提示：transform/perspective/filter/backdrop-filter 建 CB，
        // 其他属性提示（opacity）与 auto 不建。
        for prop in ["transform", "perspective", "filter", "backdrop-filter"] {
            let mut s = ComputedStyle::default();
            s.will_change = vec![WillChangeValue::Custom(prop.to_string())];
            assert!(creates_cb_for_abspos_descendants(&s), "will-change: {prop}");
        }
        let mut s = ComputedStyle::default();
        s.will_change = vec![WillChangeValue::Custom("opacity".into())];
        assert!(!creates_cb_for_abspos_descendants(&s));
        // containment 臂经 is_fixed_cb_ancestor 仍生效（R4122 语义保持）；display 须
        // 非 inline——containment 对非原子 inline 整体忽略（R4070）。
        let mut s = ComputedStyle::default();
        s.display = zero_css_parser::values::DisplayValue::Block;
        s.contain = zero_style_system::property::types::ContainComputedValue::Layout;
        assert!(is_fixed_cb_ancestor(&s));
    }
}
