//! R5139：碎片化 multicol 中全-auto inset OOF（fixed/absolute）后代的静态位置列流映射。
//!
//! 模型（2026-10-15 headless Chrome probe 系定谳，probeA/C/E/F + transform-CB/
//! static-pos corpus 六案全中）：OOF 后代四 inset 全 auto 时位置 = 静态位置；其
//! 静态位置链根在碎片化 multicol 流内、而 containing block 在该 multicol 之外时，
//! Chrome 把「假设全静态流组合出的 flow offset」（taffy 累加即该值）映射到列流
//! 坐标——落点 = (所在列 x + 行内偏移, 容器顶 + 列内偏移)，flow offset 恰在列界
//! 时取前一列底（css-position-3 §10.3.7/§10.6.4 static position + css-break-3
//! 碎片映射）。probe 数据：cursor 800/550/800（列高 100/100/500）落 col7 底/
//! col5 y50/col1 y300；corpus 负 margin 设计使映射点与 CB 原点重合（绿方盖红方）。
//!
//! ZW 布局树此前的累加坐标即「未映射流组合」（fixed-in-multicol-with-transform-
//! container 实测 fixed 落 (-342, 750.6) = 容器顶+800），缺的只是列映射这一步。
//! 本 pass 在 `adjust_multicol_layout` 之后按容器**自底向上**（后序）执行：嵌套
//! multicol 时内层映射产物（行内偏移吸收列偏移）作为外层流上输入点，两级组合
//! 与 Chrome 逐级映射一致（fixed-in-nested-…-transform-container 实测两级后
//! (275, 100) 容器相对 = Chrome (8, 50.6) 页面绝对）。
//!
//! 改写同时置 `oof_static_mapped` 旗标，paint 侧据此在 multicol 片段循环 k>0
//! 迭代与 R5137 续片重绘中抑制该子树（其 CB 在容器外，不参与碎片复制）。

use std::collections::HashMap;

use zero_css_parser::values::LengthValue;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::engine::abspos::{creates_cb_for_abspos_descendants, is_fixed_cb_ancestor};
use crate::multicol::compute_column_info;
use crate::types::LayoutBox;

/// kill-switch：`ZW_MULTICOL_OOF_STATIC=0` 关闭列流映射（A/B 隔离与紧急回滚用）。
fn oof_static_enabled() -> bool {
    std::env::var("ZW_MULTICOL_OOF_STATIC").as_deref() != Ok("0")
}

/// absolute 元素四 inset 全 auto（静态位置生效前提；fixed 用预计算 per-dim 旗标）。
fn abs_insets_all_auto(style: Option<&ComputedStyle>) -> bool {
    let Some(s) = style else {
        return false;
    };
    matches!(s.top, LengthValue::Auto)
        && matches!(s.bottom, LengthValue::Auto)
        && matches!(s.left, LengthValue::Auto)
        && matches!(s.right, LengthValue::Auto)
}

/// 对整棵布局树按 multicol 容器自底向上执行 OOF 静态位置列流映射。
pub(crate) fn remap_oof_static_positions(root: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if !oof_static_enabled() {
        return;
    }
    remap_rec(root, styles);
}

fn remap_rec(box_node: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    // 后序：内层 multicol 先映射，其产物（行内偏移含列偏移）供外层组合。
    for child in box_node.children.iter_mut() {
        remap_rec(child, styles);
    }

    if !box_node.is_multicol {
        return;
    }
    let Some(style) = box_node.node_id.and_then(|id| styles.get(&id)) else {
        return;
    };
    let Some(info) = compute_column_info(style, box_node.content_width) else {
        return;
    };
    let pitch = info.column_width + info.gap;
    let col_h = box_node.content_height;
    if info.count == 0 || pitch <= 0.0 || col_h <= 0.0 {
        return;
    }

    // 自容器内容原点下钻：ctx.acc = 祖先链 x/y 累加（children 坐标为父内容相对）；
    // ctx.cb_abs/cb_fixed = 严格祖先链上是否已出现对应 OOF 族的 containing block
    //（CB 在 multicol 之内的 OOF 位置由 inset 语义决定，不经列流映射）。
    walk_children(
        box_node,
        &WalkCtx {
            styles,
            pitch,
            col_h,
            cb_abs: false,
            cb_fixed: false,
            acc_x: 0.0,
            acc_y: 0.0,
        },
    );
}

/// 单个 multicol 容器的下钻上下文：pitch/col_h 全程不变（容器常量）；
/// acc/cb 随下钻演化（祖先链 x/y 累加与 CB 旗标）。
struct WalkCtx<'a> {
    styles: &'a HashMap<NodeId, ComputedStyle>,
    pitch: f32,
    col_h: f32,
    cb_abs: bool,
    cb_fixed: bool,
    acc_x: f32,
    acc_y: f32,
}

fn walk_children(box_node: &mut LayoutBox, ctx: &WalkCtx) {
    for i in 0..box_node.children.len() {
        let style = box_node.children[i].node_id.and_then(|id| ctx.styles.get(&id));
        // 候选：OOF + 全 auto inset（静态位置生效前提）+ 严格祖先链无本族 CB。
        let is_candidate = {
            let child = &box_node.children[i];
            let inside = if child.is_fixed { ctx.cb_fixed } else { ctx.cb_abs };
            let insets_auto = if child.is_fixed {
                child.fixed_x_insets_all_auto && child.fixed_y_insets_all_auto
            } else {
                abs_insets_all_auto(style)
            };
            (child.is_absolute || child.is_fixed) && insets_auto && !inside
        };
        if is_candidate {
            let fy = ctx.acc_y + box_node.children[i].y;
            if fy > ctx.col_h {
                // flow offset 恰在列界取前一列底（cursor 800 列高 100 → col7 底）。
                let k = ((fy / ctx.col_h).ceil() as i64).max(1) - 1;
                let y_within = fy - k as f32 * ctx.col_h;
                let child = &mut box_node.children[i];
                child.x += k as f32 * ctx.pitch;
                child.y += y_within - fy;
                child.oof_static_mapped = true;
            }
        }
        // 下钻旗标：child 自身为 CB 属性时其后代入 CB 内（absolute 族：positioned
        //（is_abspos_cb 含 positioned/containment）∪ transform/filter 等视觉 CB；
        // fixed 族：containment ∪ 视觉 CB，positioned 非 containment 不在内——
        // fixed 穿透普通 positioned 祖先）。
        let cb_abs_c =
            ctx.cb_abs || box_node.children[i].is_abspos_cb || style.is_some_and(creates_cb_for_abspos_descendants);
        let cb_fixed_c = ctx.cb_fixed || style.is_some_and(is_fixed_cb_ancestor);
        let (cx, cy) = (ctx.acc_x + box_node.children[i].x, ctx.acc_y + box_node.children[i].y);
        walk_children(
            &mut box_node.children[i],
            &WalkCtx {
                styles: ctx.styles,
                pitch: ctx.pitch,
                col_h: ctx.col_h,
                cb_abs: cb_abs_c,
                cb_fixed: cb_fixed_c,
                acc_x: cx,
                acc_y: cy,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_dom::Document;
    use zero_style_system::property::types::ColumnCountComputedValue;

    /// 构造 100×100 双列容器（列宽 50、gap 0 → pitch 50、列高 100），
    /// abs 子带 node_id + 默认样式（inset 全 auto，静态位置生效前提）。
    fn setup(children: Vec<LayoutBox>) -> (LayoutBox, HashMap<NodeId, ComputedStyle>) {
        let mut doc = Document::new();
        let root_id = doc.create_element("div");
        let mut style = ComputedStyle::default();
        style.column_count = ColumnCountComputedValue::Number(2);
        style.column_gap = LengthValue::Px(0.0);
        let mut styles = HashMap::from([(root_id, style)]);
        let mut root = LayoutBox {
            node_id: Some(root_id),
            is_multicol: true,
            width: 100.0,
            content_width: 100.0,
            height: 100.0,
            content_height: 100.0,
            ..Default::default()
        };
        for mut child in children {
            if child.is_absolute {
                let id = doc.create_element("div");
                styles.insert(id, ComputedStyle::default());
                child.node_id = Some(id);
            }
            root.children.push(child);
        }
        (root, styles)
    }

    /// 列中流偏移映射：fy=120（列高 100）→ 落 col1 列内 y20（k=1、y_within=20），
    /// x 加一个 pitch，旗标置位；同层 in-flow 兄弟不动。
    #[test]
    fn remap_maps_static_position_across_column_flow() {
        let (mut root, styles) = setup(vec![
            LayoutBox {
                y: 100.0,
                height: 100.0,
                ..Default::default()
            },
            LayoutBox {
                y: 120.0,
                is_absolute: true,
                ..Default::default()
            },
        ]);

        remap_oof_static_positions(&mut root, &styles);

        let abs = &root.children[1];
        assert!((abs.x - 50.0).abs() < 0.01, "x = col1（+1 pitch）实得 {}", abs.x);
        assert!((abs.y - 20.0).abs() < 0.01, "y = 列内偏移 20 实得 {}", abs.y);
        assert!(abs.oof_static_mapped, "映射旗标置位");
        assert!((root.children[0].y - 100.0).abs() < 0.01, "in-flow 兄弟不受映射");
    }

    /// flow offset 恰在列界（fy=200=2×列高）取前一列底：k=1、y_within=100 →
    /// col1 列内 y100（css-break-3 碎片映射边界规则；probe cursor 800/列高 100 →
    /// col7 底=列内 y100 同律）。
    #[test]
    fn remap_at_exact_column_boundary_lands_previous_column_bottom() {
        let (mut root, styles) = setup(vec![LayoutBox {
            y: 200.0,
            is_absolute: true,
            ..Default::default()
        }]);

        remap_oof_static_positions(&mut root, &styles);

        let abs = &root.children[0];
        assert!((abs.x - 50.0).abs() < 0.01, "x = col1 实得 {}", abs.x);
        assert!((abs.y - 100.0).abs() < 0.01, "y = 100（前一列列底）实得 {}", abs.y);
        assert!(abs.oof_static_mapped);
    }

    /// CB 在 multicol 之内（positioned 祖先 is_abspos_cb）不经列流映射——
    /// 其位置由 inset 语义决定（css-position-3），保持原位、不置旗标。
    #[test]
    fn remap_skips_absolute_inside_positioned_ancestor() {
        let (mut root, styles) = setup(vec![LayoutBox {
            is_abspos_cb: true,
            ..Default::default()
        }]);
        root.children[0].children.push(LayoutBox {
            y: 120.0,
            is_absolute: true,
            ..Default::default()
        });

        remap_oof_static_positions(&mut root, &styles);

        let abs = &root.children[0].children[0];
        assert!((abs.x - 0.0).abs() < 0.01 && (abs.y - 120.0).abs() < 0.01);
        assert!(!abs.oof_static_mapped, "CB 内不映射");
    }
}
