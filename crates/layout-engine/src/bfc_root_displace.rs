//! R5477：块级 BFC 根 float 位移终末校正（CSS2 §9.5「establishes a new block
//! formatting context 的普通流内元素 border box 不得与同 BFC float margin box 重叠」，
//! chromium FloatAvoider 语义）。
//!
//! 既有 BFC float 避让（float_positioning.rs adjust_float_positions_with_context 的
//! R1369/R1728/R5217/R5221 域）按「旁置优先、装不下下绕」处理绝大多数形，但对
//! contain:inline-size（width 解为 0）等零宽/窄带形会停在仍与后继带 float 重叠的位置
//!（contain-inline-size-bfc-floats-001：#contain 停 x=250/y=float1 底，与 float3 带
//! 重叠）。chromium 定谳模型（google-chrome oracle V0–V7 八形态逐位吻合）：
//! 候选位 = 逐 float 底边界层位（含当前位）× 候选 x {流内原点 = 已解 margin_left,
//! 旁置 = 左 float margin-box 右缘最大值}，逐层升序首配即停；重叠判据 = x 区间交
//!（零宽盒左缘按 float_left ≤ x < float_right 计入）× y 全盒高带交。
//!
//! 本 pass 终末运行（R1733 先例：终末位不被后续 pass 重置），**只校正仍重叠者**：
//! 不重叠的既有旁置/收缩位一概不触碰（floats-wrap-bfc-* 绿域不变式）；只推不拉
//!（dy ≥ 0，R5470 纪律）；推下后 auto 高容器涨高覆盖（R5026 镜像）＋后继流内
//! 兄弟随推（sibling.y ≥ 旧位才推）。kill-switch `ZW_BFC_ROOT_FLOAT_DISPLACE=0`。

use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_css_parser::values::{ClearValue, FloatValue, LengthValue};
use zero_dom::NodeId;
use zero_style_system::WritingModeValue;
use zero_style_system::property::types::{ColumnCountComputedValue, ColumnWidthComputedValue, ComputedStyle};

/// float margin-box 几何：(dir, left, top, right, bottom)。
type FloatBox = (FloatValue, f32, f32, f32, f32);

/// 零宽盒左缘计入判据（chromium oracle 定谳）：w>0 标准 x 区间交（严格）；
/// w=0 时 float_left ≤ x < float_right（点入区间）。
fn x_overlaps(child_left: f32, child_width: f32, float_left: f32, float_right: f32) -> bool {
    let child_right = child_left + child_width;
    if child_width > 0.0 {
        child_left < float_right && child_right > float_left
    } else {
        child_left >= float_left && child_left < float_right
    }
}

fn y_overlaps(child_top: f32, child_bottom: f32, float_top: f32, float_bottom: f32) -> bool {
    child_top < float_bottom && child_bottom > float_top
}

/// 终末校正入口（engine.rs 终末 pass 链调用）。kill-switch `ZW_BFC_ROOT_FLOAT_DISPLACE=0`。
/// env 入口读一次边界快照传递（guideline #9：稳定开关禁逐节点热路径查询）。
///
/// R5478 escaped-float 臂（**默认开**，kill-switch `ZW_BFC_ROOT_FLOAT_DISPLACE_ESCAPED=0`）：
/// 非 BFC 根流内块级直接子的子树 float 逃逸收集（CSS2 §9.5 同 BFC 不重叠语义的
/// escaped 形；chromium oracle V1–V4 四形态定谳，旁置装得下 x=左 float 右缘 max /
/// 装不下下绕 float 底，与直接子形同 FloatAvoider 模型）。
///
/// R5479 auto 宽 beside 收缩形（**默认开**，kill-switch `ZW_BFC_AUTO_BESIDE_MODEL=0`）：
/// beside＋width:auto 子形收缩到 float 旁可用带宽旁置并写宽（chromium oracle 九形
/// 定谳：V1 (100,0) 宽 350、右 float (0,0) 宽 350、双侧夹 (100,0) 宽 250；收缩带宽
/// < 框架和 border+padding 时下绕 float 底全 CB——tight 门）。主链 beside 域
/// （float_positioning R1369/R5217）先于子树递归定谳，拿不到权威 escaped 几何，
/// 时序不可行，故在本终末 pass 补域（终末链之后无 auto 宽回填，写入不被重置）。
pub(crate) fn apply_bfc_root_float_displace(box_node: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    if std::env::var("ZW_BFC_ROOT_FLOAT_DISPLACE").as_deref() == Ok("0") {
        return;
    }
    let escaped = std::env::var("ZW_BFC_ROOT_FLOAT_DISPLACE_ESCAPED").as_deref() != Ok("0");
    let auto_beside = std::env::var("ZW_BFC_AUTO_BESIDE_MODEL").as_deref() != Ok("0");
    displace_in_container(box_node, styles, escaped, auto_beside);
    for child in &mut box_node.children {
        displace_subtree(child, styles, escaped, auto_beside);
    }
}

fn displace_subtree(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    escaped: bool,
    auto_beside: bool,
) {
    displace_in_container(box_node, styles, escaped, auto_beside);
    for child in &mut box_node.children {
        displace_subtree(child, styles, escaped, auto_beside);
    }
}

/// 容器内样式门：本 pass 只处理水平书写、非 multicol 的普通块流容器。
fn container_gate(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    box_node.node_id.and_then(|id| styles.get(&id)).is_none_or(|s| {
        matches!(s.writing_mode, WritingModeValue::HorizontalTb)
            && matches!(s.column_count, ColumnCountComputedValue::Auto)
            && matches!(s.column_width, ColumnWidthComputedValue::Auto)
    })
}

/// escaped floats 收集：从非 BFC 根流内块级节点子树收集 float margin box（容器
/// 内容系坐标，沿 `content_x/content_y` 链累加）。停止边界：float（自身建 BFC，
/// 整体随 float margin box）、定位/r109、BFC 根（其内 float 不逃逸）。
fn collect_escaped_floats(node: &LayoutBox, content_ox: f32, content_oy: f32, out: &mut Vec<FloatBox>) {
    for child in &node.children {
        let child_left = content_ox + child.x;
        let child_top = content_oy + child.y;
        if !matches!(child.float, FloatValue::None) {
            // 与直接子收集同口径：top = border-box y（不减 margin_top），bottom 含 margin_bottom。
            out.push((
                child.float.clone(),
                child_left - child.margin_left,
                child_top,
                child_left + child.width + child.margin_right,
                child_top + child.height + child.margin_bottom,
            ));
            continue;
        }
        if child.is_absolute || child.is_fixed || child.is_r109_split || child.is_flow_root {
            continue;
        }
        collect_escaped_floats(child, child_left + child.content_x, child_top + child.content_y, out);
    }
}

fn displace_in_container(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    escaped: bool,
    auto_beside: bool,
) {
    if !container_gate(box_node, styles) {
        return;
    }
    // 同容器 float 子 margin-box 几何收集（R1733 同口径：直接子 float）。
    let mut floats: Vec<FloatBox> = box_node
        .children
        .iter()
        .filter(|c| !matches!(c.float, FloatValue::None))
        .map(|c| {
            (
                c.float.clone(),
                c.x - c.margin_left,
                c.y,
                c.x + c.width + c.margin_right,
                c.y + c.height + c.margin_bottom,
            )
        })
        .collect();
    // R5478 escaped 臂：escaped floats 并入候选域（默认开，kill-switch ESCAPED=0）。
    if escaped {
        for child in &box_node.children {
            let escapes = matches!(child.float, FloatValue::None)
                && child.is_block_level
                && !child.is_flow_root
                && !child.is_absolute
                && !child.is_fixed
                && !child.is_r109_split;
            if escapes {
                collect_escaped_floats(child, child.x + child.content_x, child.y + child.content_y, &mut floats);
            }
        }
    }
    if floats.is_empty() {
        return;
    }
    for idx in 0..box_node.children.len() {
        // 触发门：块级 BFC 根、非浮动/定位、非 r109 匿名块、无 clearance、
        // 水平书写、非 multicol 源。
        let (child_w, child_h, child_top, child_ml, is_displaceable) = {
            let c = &box_node.children[idx];
            let style = c.node_id.and_then(|id| styles.get(&id));
            let gate = c.is_flow_root
                && c.is_block_level
                && matches!(c.float, FloatValue::None)
                && !c.is_absolute
                && !c.is_fixed
                && !c.is_r109_split
                && matches!(c.clear, ClearValue::None)
                && style.is_none_or(|s| {
                    matches!(s.writing_mode, WritingModeValue::HorizontalTb)
                        && matches!(s.clear, ClearValue::None)
                        && matches!(s.column_count, ColumnCountComputedValue::Auto)
                        && matches!(s.column_width, ColumnWidthComputedValue::Auto)
                });
            let overlaps = gate
                && floats.iter().any(|(_, fl, ft, fr, fb)| {
                    y_overlaps(c.y, c.y + c.height, *ft, *fb) && x_overlaps(c.x, c.width, *fl, *fr)
                });
            (c.width, c.height, c.y, c.margin_left, overlaps)
        };
        if !is_displaceable {
            continue;
        }
        // chromium FloatAvoider 定谳模型：逐 float 底边界层位（含当前位）升序 ×
        // 候选 x {流内原点 ml, 旁置 = 左 float margin-box 右缘 max}，首配即停。
        // 旁置候选须装得下（§9.5 "adjacent to such floats if there is sufficient
        // space"）：beside + 盒宽 ≤ 容器内容右缘，装不下走下绕层（oracle beside
        // 形定谳：300 容器 150 float + 200 宽 BFC 根 → 下绕 float 底，不旁置溢出）。
        let content_right = box_node.border_left + box_node.padding_left + box_node.content_width;
        // R5479：auto 宽 beside 收缩形（chromium headless oracle 九形定谳）：
        // beside＋width:auto 子形 chromium 把 BFC 根收缩到 float 旁可用宽旁置（V1
        // (100,0) 宽 350、E4 右 float (0,0) 宽 350、E3 双侧 (100,0) 宽 250），收缩带
        // 宽 < 框架和（border+padding）时下绕 float 底全 CB（E1/e5/e6/e7 定谳 (0,50)
        // 宽 450）。主链 beside 域（float_positioning R1369/R5217）看不到 escaped
        // floats（排斥循环先于递归定谳，时序拿不到权威几何），故在本终末 pass 补域：
        // 候选 x 处收缩带宽 = min(容器内容右缘，层位 y 交带内右 float margin-box 左缘
        // min) − x，≥ 框架和可行并随位移写宽；declared-width 盒维持原判据。
        let (child_is_auto, child_frame, child_mr) = {
            let c = &box_node.children[idx];
            (
                c.declared_width_auto && escaped && auto_beside,
                c.border_left + c.border_right + c.padding_left + c.padding_right,
                if c.margin_right_auto { 0.0 } else { c.margin_right },
            )
        };
        // declared-width 盒维持原全局 beside（R5477 八形调优域零回归）；auto 宽收缩形
        // beside/右界按层位 y 交带算（E3 双侧夹 (100,0) 宽 250 定谳须带内右 float 界）。
        let beside_global = floats
            .iter()
            .filter(|(d, ..)| matches!(d, FloatValue::Left))
            .map(|(_, _, _, fr, _)| *fr)
            .fold(f32::NEG_INFINITY, f32::max);
        let mut levels: Vec<f32> = floats.iter().map(|(_, _, _, _, fb)| *fb).collect();
        levels.push(child_top);
        levels.retain(|&l| l >= child_top - 0.5);
        levels.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        levels.dedup();
        let mut resolved: Option<(f32, f32, Option<f32>)> = None;
        for &level in &levels {
            // 层位 y 交带（beside 左缘与右 float 收缩右界都按本层位带算）。
            let band: Vec<&FloatBox> = floats
                .iter()
                .filter(|(_, _, ft, _, fb)| y_overlaps(level, level + child_h, *ft, *fb))
                .collect();
            let beside = if child_is_auto {
                band.iter()
                    .filter(|(d, ..)| matches!(d, FloatValue::Left))
                    .map(|(_, _, _, fr, _)| *fr)
                    .fold(f32::NEG_INFINITY, f32::max)
            } else {
                beside_global
            };
            let beside_usable = beside.is_finite()
                && if child_is_auto {
                    beside >= child_ml - 0.5 && (content_right - beside) >= child_frame - 0.5
                } else {
                    beside + child_w <= content_right + 0.5
                };
            for cx in [child_ml, beside] {
                if !cx.is_finite() || (cx != child_ml && !beside_usable) {
                    continue;
                }
                let (w_eff, shrink_w) = if child_is_auto {
                    let band_right = band
                        .iter()
                        .filter(|(d, ..)| matches!(d, FloatValue::Right))
                        .map(|(_, fl, ..)| *fl)
                        .fold(content_right, f32::min);
                    // R5482：收缩右界夹 margin-right（chromium oracle M1–M3 混合形
                    // escaped＋margin≠0 定谳——R5217 margin-aware 钉位解原样适用
                    // escaped 域：right = min(CB − mr, band_right)、w = right − cx；
                    // mr=0 时与旧公式同值（R5479 九形 margin:0 域零变化）。M2 形
                    // chromium (100,0) 宽 340＝band 350 − mr 10（pre-fix 350 缺口）；
                    // wrap 层 band 空时 w = CB − mr − cx = CB − margin_h（M1 (0,50)
                    // 宽 440 同式）。
                    let w = ((content_right - child_mr).min(band_right) - cx).max(0.0);
                    (w, if w < child_w - 0.5 { Some(w) } else { None })
                } else {
                    (child_w, None)
                };
                if child_is_auto && w_eff < child_frame - 0.5 {
                    // 收缩带宽不足框架和 → 本候选不可行（走下绕层位）。
                    continue;
                }
                let hit = band.iter().any(|(_, fl, _, fr, _)| x_overlaps(cx, w_eff, *fl, *fr));
                if !hit {
                    resolved = Some((cx, level, shrink_w));
                    break;
                }
            }
            if resolved.is_some() {
                break;
            }
        }
        let Some((target_x, target_y, shrink_w)) = resolved else {
            continue;
        };
        // 只推不拉（R5470 纪律）：目标位须不低于现位（x 重钉仅伴随下推）。
        let dy = target_y - child_top;
        if dy < -0.5 {
            continue;
        }
        let old_top = child_top;
        {
            let child = &mut box_node.children[idx];
            child.x = target_x;
            child.y = target_y;
            // R5479：auto 宽 beside 收缩形写宽（float_positioning shrink_bfc_content_width
            // 同式同步 content_width；终末链之后无 auto 宽回填 pass，写入不被重置）。
            if let Some(w) = shrink_w {
                child.width = w;
                child.content_width = (child.width - child_frame).max(0.0);
            }
        }
        // 后继流内兄弟随推（sibling.y ≥ 旧位才推；float/定位/r109 兄弟不动）。
        if dy > 0.5 {
            for sib in box_node.children[idx + 1..].iter_mut() {
                let in_flow =
                    matches!(sib.float, FloatValue::None) && !sib.is_absolute && !sib.is_fixed && !sib.is_r109_split;
                if in_flow && sib.y >= old_top - 0.5 {
                    sib.y += dy;
                }
            }
            // auto 高容器涨高覆盖新内容底（R5026 镜像：显式高不涨）。needed 取位移盒
            // 与被推兄弟 margin-box 底的最大值（auto 高须包住全部流内内容底；行内
            // 内容已计入 content_height，grow-only max 不会回缩）。
            let is_auto = box_node
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.height, LengthValue::Auto));
            if is_auto {
                let mut needed = {
                    let child = &box_node.children[idx];
                    child.y + child.height + child.margin_bottom
                };
                for sib in &box_node.children[idx + 1..] {
                    let in_flow = matches!(sib.float, FloatValue::None)
                        && !sib.is_absolute
                        && !sib.is_fixed
                        && !sib.is_r109_split;
                    if in_flow {
                        needed = needed.max(sib.y + sib.height + sib.margin_bottom);
                    }
                }
                if needed > box_node.content_height + 0.5 {
                    let delta = needed - box_node.content_height;
                    box_node.height += delta;
                    box_node.content_height = needed;
                }
            }
        }
    }
}
