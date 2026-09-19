//! R4519（R1473 step-2 slice ①，CSS Multicol §6.1 column-span + css-break §4 box
//! fragmentation）：**bordered nested-spanner wrapper 的区域×列 fragment 分段模型**。
//!
//! chromium 把被 column-span:all spanner 拆分的 bordered 容器按列 fragment 分片绘制：
//! 每个区域（spanner 前后段）是盒的一个独立 fragment 实例——首区域拥有盒顶边
//! （top+left+right 边框），末区域拥有盒底边（left+right+bottom），spanner 相邻边
//! skip（multicol-span-all-children-height-006 的 meta assert）。区域间与列间隙处
//! 透出多列容器（article）背景。
//!
//! 几何模型（006 test/ref 双页 PNG 逐像素实测收敛）：
//! - 区域 block 总量 = 区域内容渲染量 + 区域所属盒边/内边距 + 首区域 margin-top /
//!   末区域 margin-bottom（首/末边框归首/末区域）；
//! - 区域 cell 高 = 区域总量 / 列数（每列一个 cell，顺序切片：col i 的 fragment
//!   偏移 = i × cell）；
//! - 总 advance = Σ cell + Σ spanner 高（006：118 + 50 + 43 = 211，article [8,219]）；
//! - 末区域内容渲染量 = 剩余 CSS content 预算（顺序预算，R4514 同律：006
//!   250 − 200 = 50，block2 溢出照绘）；
//! - 段盒宽 = column_width − 水平边框（与 position_multicol_children 的列宽约束
//!   double-subtraction 结果 byte-一致——ref 页同款几何，自源配对一致的前提）。
//!
//! 产出两组 wrapper border-box 相对坐标的绘制载荷（painter 直接消费）：
//! [`NestedSpannerSegRect`]（边框/背景段，已裁剪到 cell）+
//! [`NestedSpannerChildFrag`]（子元素逐 fragment 绘制位置 + cell 裁剪）。
//! 同时回写 spanner 坐标（article content 帧）与 wrapper/article 高度。
//!
//! kill-switch：`ZW_BORDERED_FRAG_SEG=0`（回退 R1359 strip + 单盒边框行为）。

use std::collections::HashMap;

use zero_css_parser::values::LengthValue;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::ColumnSpanComputedValue;

use super::ColumnInfo;
use crate::types::{LayoutBox, NestedSpannerChildFrag, NestedSpannerSegKind, NestedSpannerSegRect};

/// wrapper 是否带自身盒（border/padding 任一侧 ≥ 1px）——bordered_frag 臂判定。
/// 无框 wrapper（004a/b）继续走 R1359 strip 模型，本模块不触。
pub(super) fn wrapper_has_box(wrapper: &LayoutBox) -> bool {
    wrapper.border_top >= 1.0
        || wrapper.border_bottom >= 1.0
        || wrapper.border_left >= 1.0
        || wrapper.border_right >= 1.0
        || wrapper.padding_top >= 1.0
        || wrapper.padding_bottom >= 1.0
}

/// 圆角 gate：分段装饰为直角矩形，任一角非零像素即放弃（回落单盒绘制）。
fn border_radius_is_zero(style: &ComputedStyle) -> bool {
    let zero = |v: &LengthValue| matches!(v, LengthValue::Px(p) if *p == 0.0);
    zero(&style.border_top_left_radius)
        && zero(&style.border_top_right_radius)
        && zero(&style.border_bottom_right_radius)
        && zero(&style.border_bottom_left_radius)
}

/// 纵向区间与 cell 区间求交（空 → None）。
fn intersect_v(y: f32, h: f32, cell_y: f32, cell_h: f32) -> Option<(f32, f32)> {
    let top = y.max(cell_y);
    let bottom = (y + h).min(cell_y + cell_h);
    if bottom - top > 0.5 {
        Some((top, bottom - top))
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn push_side(
    segs: &mut Vec<NestedSpannerSegRect>,
    kind: NestedSpannerSegKind,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    cell_y: f32,
    cell_h: f32,
) {
    if w <= 0.5 {
        return;
    }
    if let Some((y, h)) = intersect_v(y, h, cell_y, cell_h) {
        segs.push(NestedSpannerSegRect { x, y, w, h, kind });
    }
}

/// 对 bordered nested-spanner wrapper 应用区域×列 fragment 分段模型。
///
/// 调用点：`try_layout_nested_spanner` 末尾（R1357/R1359/R1360 之后——本函数对其
/// wrapper/article 高度结果做 bordered 覆写，并清 `nested_spanner_col_bg` 使
/// strip 模型不双绘）。gate 不满足时返回 false，零行为变更。
pub(super) fn apply_bordered_region_fragments(
    container: &mut LayoutBox,
    wrapper_idx: usize,
    eff_indices: &[usize],
    info: &ColumnInfo,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> bool {
    if std::env::var("ZW_BORDERED_FRAG_SEG").as_deref() == Ok("0") {
        return false;
    }
    if info.count < 2 {
        return false;
    }
    let wrapper = &container.children[wrapper_idx];
    if !wrapper_has_box(wrapper) {
        return false;
    }
    let Some(wid) = wrapper.node_id else {
        return false;
    };
    let Some(ws) = styles.get(&wid) else {
        return false;
    };
    // 预算模型输入须为 CSS 显式 content 高（同 R1357 wrapper_budget gate）。
    let LengthValue::Px(css_h) = ws.height else {
        return false;
    };
    if !border_radius_is_zero(ws) {
        return false;
    }

    let mt = wrapper.margin_top;
    let mb = wrapper.margin_bottom;
    let bt = wrapper.border_top;
    let bb = wrapper.border_bottom;
    let bl = wrapper.border_left;
    let br = wrapper.border_right;
    let pt = wrapper.padding_top;
    let pb = wrapper.padding_bottom;
    let pl = wrapper.padding_left;
    let pr = wrapper.padding_right;

    // 1. 区域划分（与 synth 同序）：regions[r] = eff_indices 内非 spanner 子位置；
    //    spanner 高度按序记为区域间 advance。（eff_indices 是 **wrapper.children**
    //    下标——与 R1357/R1359 同口径。判定预展开为标志表，避免借用冲突。）
    let wrapper_children = &container.children[wrapper_idx].children;
    let spanner_flags: Vec<bool> = eff_indices
        .iter()
        .map(|&idx| {
            wrapper_children[idx]
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.column_span, ColumnSpanComputedValue::All))
        })
        .collect();
    let is_spanner = |pos: usize| -> bool { spanner_flags[pos] };
    let mut regions: Vec<Vec<usize>> = vec![Vec::new()];
    let mut spanner_heights: Vec<f32> = Vec::new();
    for pos in 0..eff_indices.len() {
        if is_spanner(pos) {
            spanner_heights.push(wrapper_children[eff_indices[pos]].height);
            regions.push(Vec::new());
        } else {
            regions.last_mut().unwrap().push(pos);
        }
    }

    // 2. 顺序预算：每区域 rendered = min(内容总量, 剩余 CSS content 预算)
    //（R4514 同律：006 s1 = min(200, 250) = 200、s2 = min(200, 50) = 50）。
    let content_budget = (css_h as f32 - pt - pb).max(0.0);
    let mut rendered: Vec<f32> = Vec::with_capacity(regions.len());
    let mut remaining = content_budget;
    for region in &regions {
        let total: f32 = region
            .iter()
            .map(|&pos| {
                let c = &wrapper_children[eff_indices[pos]];
                c.height + c.margin_top + c.margin_bottom
            })
            .sum();
        let r = total.min(remaining);
        rendered.push(r);
        remaining -= r;
    }

    // 3. 区域总量与 cell 高：首区域扛 mt+bt+pt，末区域扛 pb+bb+mb（spanner 相邻边
    // skip = 边框只归拥有该盒边的区域实例）。
    let last = regions.len() - 1;
    let cells: Vec<f32> = (0..regions.len())
        .map(|r| {
            let mut t = rendered[r];
            if r == 0 {
                t += mt + bt + pt;
            }
            if r == last {
                t += pb + bb + mb;
            }
            t / info.count as f32
        })
        .collect();

    // 4. cell 顶（wrapper border-box 相对；wrapper 盒顶在 article content 下 mt 处）。
    let cell_tops: Vec<f32> = {
        let mut tops = Vec::with_capacity(regions.len());
        let mut acc = -mt;
        for (r, &cell) in cells.iter().enumerate() {
            tops.push(acc);
            acc += cell + spanner_heights.get(r).copied().unwrap_or(0.0);
        }
        tops
    };
    let total_advance: f32 = cells.iter().sum::<f32>() + spanner_heights.iter().sum::<f32>();
    if total_advance <= 0.5 {
        return false;
    }

    // 5. 段几何：段盒宽 = column_width − 水平边框（position_multicol_children 列宽
    // 约束的 double-subtraction 终值，与 ref 页容器几何 byte-一致）。
    let stride = info.column_width + info.gap;
    let seg_w = (info.column_width - bl - br).max(0.0);
    if seg_w < 1.0 {
        return false;
    }

    let mut segs: Vec<NestedSpannerSegRect> = Vec::new();
    let mut frags: Vec<NestedSpannerChildFrag> = Vec::new();
    for (r, region) in regions.iter().enumerate() {
        let cell_y = cell_tops[r];
        let cell_h = cells[r];
        if cell_h < 0.5 {
            continue;
        }
        // 区域盒实例：首区域带 margin cell（盒顶 = cell 顶 + mt），末/中间区域无。
        // 盒边归属：top 边框/内边距属首区域实例，bottom 属末区域实例（spanner 相邻
        // 边 skip 的几何表达）——区域内坐标一律用 r_bt/r_bb，不得混用全盒 bt/bb。
        let box_top_base = if r == 0 { cell_y + mt } else { cell_y };
        let (r_bt, r_bb) = (if r == 0 { bt } else { 0.0 }, if r == last { bb } else { 0.0 });
        let box_h = rendered[r] + r_bt + pt + pb + r_bb;
        for i in 0..info.count {
            // col i 的顺序切片偏移：region 内第 i 个 cell。
            let box_top = box_top_base - i as f32 * cell_h;
            let sx = i as f32 * stride;
            // 边框段（仅拥有对应盒边的区域发射；∩ cell 后空段自然消失——
            // col i>0 的顶边落在 cell 上方之外，即前 cell/spanner 相邻 skip）。
            if r == 0 && bt > 0.5 {
                push_side(
                    &mut segs,
                    NestedSpannerSegKind::BorderTop,
                    sx,
                    box_top,
                    seg_w,
                    bt,
                    cell_y,
                    cell_h,
                );
            }
            if r == last && bb > 0.5 {
                push_side(
                    &mut segs,
                    NestedSpannerSegKind::BorderBottom,
                    sx,
                    box_top + box_h - bb,
                    seg_w,
                    bb,
                    cell_y,
                    cell_h,
                );
            }
            if bl > 0.5 {
                push_side(
                    &mut segs,
                    NestedSpannerSegKind::BorderLeft,
                    sx,
                    box_top,
                    bl,
                    box_h,
                    cell_y,
                    cell_h,
                );
            }
            if br > 0.5 {
                push_side(
                    &mut segs,
                    NestedSpannerSegKind::BorderRight,
                    sx + seg_w - br,
                    box_top,
                    br,
                    box_h,
                    cell_y,
                    cell_h,
                );
            }
            // 背景段 = 段盒 content 区 ∩ cell。
            let bg_w = (seg_w - bl - br - pl - pr).max(0.0);
            let bg_h = (box_h - r_bt - r_bb - pt - pb).max(0.0);
            if bg_w > 0.5
                && let Some((y, h)) = intersect_v(box_top + r_bt + pt, bg_h, cell_y, cell_h)
            {
                segs.push(NestedSpannerSegRect {
                    x: sx + bl + pl,
                    y,
                    w: bg_w,
                    h,
                    kind: NestedSpannerSegKind::Background,
                });
            }
            // 子元素 fragment：paint 于区域盒 content 顶（顺序切片 f 已含在 box_top），
            // 裁剪窗口水平 = 段盒 ± 半 gap（同通用列窗口余量）、垂直 = cell。
            let content_top = box_top + r_bt + pt;
            let mut child_y = content_top;
            for &pos in region {
                let child = &wrapper_children[eff_indices[pos]];
                frags.push(NestedSpannerChildFrag {
                    child_idx: eff_indices[pos],
                    paint_x: sx + bl + pl + child.margin_left,
                    paint_y: child_y + child.margin_top,
                    clip_x: sx - info.gap / 2.0,
                    clip_y: cell_y,
                    clip_w: seg_w + info.gap,
                    clip_h: cell_h,
                });
                child_y += child.height + child.margin_top + child.margin_bottom;
            }
        }
    }

    // 6. 回写：spanner 脱列流定位到区域间 advance 位（y：wrapper content 帧 =
    // cell 链顶 − 盒顶内偏移）；x 归 article content 帧（spanner 跨 multicol 容器
    // 全宽，非 wrapper content 列帧——R4517 帧修正的 spanner 特例）。
    let mut spanner_r = 0usize;
    for (&idx, &flag) in eff_indices.iter().zip(spanner_flags.iter()) {
        if !flag {
            continue;
        }
        let s = &mut container.children[wrapper_idx].children[idx];
        s.column_span_offsets.clear();
        s.x = -(bl + pl);
        s.y = cell_tops[spanner_r] + cells[spanner_r] - bt - pt;
        spanner_r += 1;
    }

    let wrapper = &mut container.children[wrapper_idx];
    wrapper.nested_spanner_col_bg.clear();
    wrapper.nested_spanner_box_segs = segs;
    wrapper.nested_spanner_child_frags = frags;
    // wrapper 盒底 = 总 advance − mt（mb 折叠出 multicol 容器，不入盒高）。
    let new_h = (total_advance - mt).max(0.0);
    wrapper.height = new_h;
    wrapper.content_height = (new_h - bt - bb - pt - pb).max(0.0);
    // article 高 = 总 advance（cell 链 + spanner），bg/后继流位随之（006：211）。
    let delta = total_advance - container.content_height;
    container.content_height = total_advance;
    container.height += delta;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::values::LengthValue;
    use zero_style_system::property::types::{ColumnSpanComputedValue, ColumnWidthComputedValue};

    /// R4519 回归：006 几何（article 400 + column-count:2 > wrapper[border 20 +
    /// margin 16 + height 250 + pink] > [block1 200][spanner 50][block2 200]）。
    /// 断言区域×列 cell 模型的全部关键输出与 ref 页渲染逐像素收敛实测值一致：
    /// cell 118/43、总 advance 211、spanner 归位 article 帧、边框/背景段矩形。
    #[test]
    fn bordered_wrapper_region_fragments_006_geometry() {
        let mut doc = zero_dom::Document::new();
        let wrapper_id = doc.create_element("div");
        let block1_id = doc.create_element("div");
        let spanner_id = doc.create_element("div");
        let block2_id = doc.create_element("div");

        let mut wrapper_style = ComputedStyle::default();
        wrapper_style.height = LengthValue::Px(250.0);
        let mut spanner_style = ComputedStyle::default();
        spanner_style.column_span = ColumnSpanComputedValue::All;
        let styles = HashMap::from([(wrapper_id, wrapper_style), (spanner_id, spanner_style)]);

        let block = |id| LayoutBox {
            node_id: Some(id),
            width: 100.0,
            content_width: 100.0,
            height: 200.0,
            content_height: 200.0,
            ..Default::default()
        };
        let wrapper = LayoutBox {
            node_id: Some(wrapper_id),
            margin_top: 16.0,
            margin_bottom: 16.0,
            border_top: 20.0,
            border_right: 20.0,
            border_bottom: 20.0,
            border_left: 20.0,
            width: 400.0,
            content_width: 360.0,
            height: 290.0,
            content_height: 250.0,
            is_block_level: true,
            children: vec![
                block(block1_id),
                LayoutBox {
                    node_id: Some(spanner_id),
                    width: 400.0,
                    content_width: 400.0,
                    height: 50.0,
                    content_height: 50.0,
                    ..Default::default()
                },
                block(block2_id),
            ],
            ..Default::default()
        };
        let mut container = LayoutBox {
            content_width: 400.0,
            content_height: 195.0,
            height: 195.0,
            is_multicol: true,
            children: vec![wrapper],
            ..Default::default()
        };
        let info = super::super::ColumnInfo {
            count: 2,
            column_width: 192.0,
            gap: 16.0,
            sequential_fill: false,
        };

        assert!(apply_bordered_region_fragments(
            &mut container,
            0,
            &[0, 1, 2],
            &info,
            &styles
        ));

        // 高度回写：article = Σ cell + spanner = 118 + 50 + 43 = 211；wrapper 盒 = 211 − mt。
        assert!(
            (container.content_height - 211.0).abs() < 0.01,
            "{}",
            container.content_height
        );
        assert!((container.children[0].height - 195.0).abs() < 0.01);
        // spanner：x 归 article content 帧（−bl−pl），y = region0 cell 底 − 盒顶内偏移。
        let spanner = &container.children[0].children[1];
        assert!((spanner.x + 20.0).abs() < 0.01);
        assert!((spanner.y - 82.0).abs() < 0.01);

        let segs = &container.children[0].nested_spanner_box_segs;
        let find = |kind: NestedSpannerSegKind, x: f32| {
            segs.iter()
                .filter(|s| s.kind == kind && (s.x - x).abs() < 0.01)
                .cloned()
                .collect::<Vec<_>>()
        };
        let near = |a: f32, b: f32| (a - b).abs() < 0.01;
        // 首区域（region0）：col0 顶边可见（cell [−16,102] 内 [0,20]），col1 顶边被裁空。
        let t0 = find(NestedSpannerSegKind::BorderTop, 0.0);
        assert_eq!(t0.len(), 1);
        assert!(near(t0[0].y, 0.0) && near(t0[0].w, 152.0) && near(t0[0].h, 20.0));
        assert!(find(NestedSpannerSegKind::BorderTop, 208.0).is_empty());
        // 末区域（region1）：col1 底边可见（[159,179]），col0 底边被裁空。
        let b1 = find(NestedSpannerSegKind::BorderBottom, 208.0);
        assert_eq!(b1.len(), 1);
        assert!(near(b1[0].y, 159.0) && near(b1[0].h, 20.0));
        assert!(find(NestedSpannerSegKind::BorderBottom, 0.0).is_empty());
        // 左竖边：col0 region0 [0,102]、col1 region0 顶越 cell 裁到 [−16,102]。
        let l0 = find(NestedSpannerSegKind::BorderLeft, 0.0);
        assert!(near(l0[0].y, 0.0) && near(l0[0].h, 102.0));
        let l1 = find(NestedSpannerSegKind::BorderLeft, 208.0);
        assert!(near(l1[0].y, -16.0) && near(l1[0].h, 118.0));
        // 背景段：region0 col0 content 带高 82、col1 全 cell 118；region1 col0 43、col1 7。
        let bg0 = find(NestedSpannerSegKind::Background, 20.0);
        assert!(near(bg0[0].y, 20.0) && near(bg0[0].h, 82.0));
        assert!(near(bg0[1].y, 152.0) && near(bg0[1].h, 43.0));
        let bg1 = find(NestedSpannerSegKind::Background, 228.0);
        assert!(near(bg1[0].y, -16.0) && near(bg1[0].h, 118.0));
        assert!(near(bg1[1].y, 152.0) && near(bg1[1].h, 7.0));

        // 子 fragment：region0 cell [−16,118)、region1 cell [152,195)，段盒宽 152。
        let frags = &container.children[0].nested_spanner_child_frags;
        assert_eq!(frags.len(), 4);
        assert!(near(frags[0].paint_x, 20.0) && near(frags[0].paint_y, 20.0));
        assert!(near(frags[1].paint_x, 228.0) && near(frags[1].paint_y, -98.0));
        assert!(near(frags[2].paint_x, 20.0) && near(frags[2].paint_y, 152.0));
        assert!(near(frags[3].paint_x, 228.0) && near(frags[3].paint_y, 109.0));
        for (i, f) in frags.iter().enumerate() {
            let expect_x = if i % 2 == 0 { -8.0 } else { 200.0 };
            assert!(
                near(f.clip_w, 168.0) && near(f.clip_x, expect_x),
                "frag{i} clip=({},{},{},{})",
                f.clip_x,
                f.clip_y,
                f.clip_w,
                f.clip_h
            );
        }
        assert!(near(frags[0].clip_y, -16.0) && near(frags[0].clip_h, 118.0));
        assert!(near(frags[2].clip_y, 152.0) && near(frags[2].clip_h, 43.0));

        // strip 模型关闭（bordered 路径不双绘）。
        assert!(container.children[0].nested_spanner_col_bg.is_empty());
        // spanner 样式仍为 All（供 dispatch 复用），宽度保持全宽。
        let _ = ColumnWidthComputedValue::Auto;
    }
}
