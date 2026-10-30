//! 浮动定位与收缩后处理。
//!
//! 从 `engine.rs` 抽出（R342，2000 行规则 + Phase A Phase 5 准备）。
//! 包含：float 元素重新定位（CSS 2.1 §9.5）、clear clearance、BFC 浮动排斥、
//! 垂直书写模式块收缩、inline-block shrink-to-fit、匿名 table 根标记。

use std::collections::HashMap;
use zero_css_parser::values::{ColorValue, DisplayValue, FlexDirectionValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_style_system::{ComputedStyle, WritingModeValue};

use crate::types::LayoutBox;

/// 调整 float 元素的位置，并处理 clear 属性。
///
/// taffy 将 float 元素当作普通 block 处理（按正常流排列）。
/// 此后处理步骤将 float 元素重新定位到容器的左侧或右侧，
/// 支持水平并排放置、clear 属性（含 float 元素自身的 clear）。
///
/// ## 实现要点
///
/// - 同侧 float 水平并排放置（CSS 2.1 §9.5.1），空间不足时换行
/// - float 元素的 clear 属性正确生效
/// - 非 float 块元素的 clear 使用 max(normal_flow_Y, float_bottom)
/// - 非 float、非 clear 元素的 Y 偏移扣除 float 元素占据的垂直空间
///
/// R4235：float walk 坐标约定开关（content-rel 默认开；`ZW_FLOAT_CLAMP_CONTENT_REL=0`
/// 回退 border-rel）。LazyLock 进程级缓存——walk 按容器逐次调用，逐次 env::var 兼作
/// LLVM 优化栅栏有净负前科（R4214）。
static FLOAT_CLAMP_CONTENT_REL: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_FLOAT_CLAMP_CONTENT_REL").as_deref() != Ok("0"));

/// R5215（CSS2 §9.2.1.1）：float walk 第二阶段把原子 inline-level 子（inline-block
/// 族 / replaced）当作流参与者（匿名块盒占位）——`ZW_R5215_ATOMIC_FLOW=0` 回退。
static R5215_ATOMIC_FLOW: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_R5215_ATOMIC_FLOW").as_deref() != Ok("0"));

/// R5228（CSS2 §8.3.1 × §9.5.1）：float 前置容器的首流内子重放不再重放已与父折叠的
/// margin-top——`ZW_FLOAT_REPLAY_FIRST_MT=0` 回退。
static FLOAT_REPLAY_FIRST_MT: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_FLOAT_REPLAY_FIRST_MT").as_deref() != Ok("0"));

pub(crate) fn adjust_float_positions(
    box_node: &mut LayoutBox,
    doc: &zero_dom::Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    // R5307：子树含 float 旗标预扫——递归循环的嵌套 float 收集器按旗标 O(1) 跳过
    // 无 float 子树，避免每层重复下探使纯块级文档付出 O(n·depth) 重遍历（bench
    // block_layout_1000/deep_nesting 域）。旗标只做保守裁剪：收集器在无 float 子树
    // 本就返回空，跳过与收集逐值等价。全树无 float 时只做单趟布尔预扫不建表
    //（None = 收集器全跳过），无分配零开销路径。
    let content_abs_y = box_node.y + box_node.content_y;
    let mut flags: HashMap<NodeId, bool> = HashMap::new();
    let float_subtrees = if tree_has_float(box_node) {
        collect_float_subtree_flags(box_node, &mut flags);
        Some(&flags)
    } else {
        None
    };
    adjust_float_positions_with_context(box_node, content_abs_y, 0.0, 0.0, &[], doc, styles, float_subtrees);
}

/// R5307：全树是否含浮动元素（含自身），布尔单趟——无 float 文档零分配快速路径。
fn tree_has_float(node: &LayoutBox) -> bool {
    if !matches!(node.float, FloatValue::None) {
        return true;
    }
    node.children.iter().any(tree_has_float)
}

/// R5307：标记每棵子树是否含浮动元素（含自身）。无 id 匿名盒不入表（查询侧对
/// 无 id 子保守回退为收集）。
fn collect_float_subtree_flags(node: &LayoutBox, flags: &mut HashMap<NodeId, bool>) -> bool {
    let mut has = !matches!(node.float, FloatValue::None);
    for c in &node.children {
        if collect_float_subtree_flags(c, flags) {
            has = true;
        }
    }
    if let Some(id) = node.node_id {
        flags.insert(id, has);
    }
    has
}

/// R1733：inline-block（atomic inline-level BFC，`is_flow_root && !is_block_level`）float 排斥
/// **终末 pass**——在 compute() 所有重定位 pass 之后运行，确保 x-fix 不被后续 pass 重置
///（R1732 实证：在 adjust_float_positions 内做 x-fix 被 inline-level `y -= float_y_offset`
/// 降级 + 后续 pass 重置 → 不存活）。近似 IFC line-box shortening：inline-block 与同容器 float
/// 垂直重叠时 shift x 到 float 旁（左 float 推右、右 float 收缩宽），使 border-box 不重叠 float。
///
/// floats-wrap-top-below-bfc l 变体 REF（inline-block 旁 float 应 x=161 非 x=11）。
/// 非 R109 匿名块重写（RFC Slice 1+2 高风险）的低风险终末近似——终末跑使坐标已最终
///（y 已匹配 chromium），仅 x 需修正且不被覆盖。kill-switch `ZW_BFC_INLINEBLOCK_AVOID=0`。
pub(crate) fn apply_inline_block_float_avoidance(box_node: &mut LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    use zero_css_parser::values::FloatValue;
    // 收集本容器直接 float 子几何（与子同坐标系）。
    let floats: Vec<(FloatValue, f32, f32, f32, f32, f32)> = box_node
        .children
        .iter()
        .filter(|c| !matches!(c.float, FloatValue::None))
        .map(|c| (c.float.clone(), c.x, c.y, c.width, c.height, c.margin_right))
        .collect();
    if !floats.is_empty() {
        let container_w = box_node.content_width;
        for child in &mut box_node.children {
            // 仅 atomic inline-level BFC（inline-block；flow_root 含 FlowRoot 但后者 block-level）。
            if !(child.is_flow_root
                && !child.is_block_level
                && matches!(child.float, FloatValue::None)
                && !child.is_absolute
                && !child.is_fixed)
            {
                continue;
            }
            let ctop = child.y;
            let cbottom = child.y + child.height;
            // 收集与本 inline-block 垂直重叠的 float。
            let overlapping: Vec<&(FloatValue, f32, f32, f32, f32, f32)> = floats
                .iter()
                .filter(|(_, _, fy, _, fh, _)| {
                    let fbottom = *fy + *fh;
                    ctop < fbottom && cbottom > *fy
                })
                .collect();
            if overlapping.len() >= 2 {
                // R1733 续：多-float 协调（mirror R1730）——逐 float 独立 shift/shrink 会 over-shrink
                //（floats-wrap-top-below-inline-002r：left+right float 致 inline-block 被错缩到右侧）。
                // 计算可行 x 区间 [x_lo, x_hi]（左 float 推 x_lo 右移、右 float 推 x_hi 左移），
                // 可行则置 x_lo；不可行（float 占满致旁无空）则**保持原位**（勿错位，如 002r-inline
                // 原 x=[11,210] 匹配 chromium，per-float 错移到 [311,410]）。
                let used_width = child.declared_width_px.unwrap_or(child.width);
                // R3857：替换元素（CSS2 §10.3.2/§10.3.4：replaced 用 width 指定值，不可收缩）
                // 旁 float 放不下时**下推到 float 底**（§9.5 replaced 排斥的 clear 语义），
                // 不收缩宽度。width:auto 的 BFC 仍可收缩（R1733 原语义，floats-wrap-bfc-* 簇）；
                // definite-width BFC 块维持既有收缩/复位行为（floats-wrap-top-below-bfc-* 调优域，
                // 贸然 push-below 曾致 001r/003r/intrinsic-size 回归）。
                let is_definite = child.is_replaced;
                let mut x_lo = 0.0_f32;
                let mut x_hi = (container_w - used_width).max(0.0);
                for (fd, fx, _fy, fw, _fh, fmr) in &overlapping {
                    match fd {
                        FloatValue::Left => x_lo = x_lo.max(*fx + *fw + *fmr),
                        FloatValue::Right => x_hi = x_hi.min(*fx - used_width),
                        _ => {}
                    }
                }
                if x_lo <= x_hi + 0.5 {
                    // R5030：可行区间内按 text-align 取位——Left/Start → x_lo（区间左缘，
                    // 旧行为），Right/End → x_hi（区间右缘），Center → 区间中点。
                    // chromium 001r（text-align:right + 右 float 群）：span2 落区间右缘
                    // x=100（页面 111），旧 x_lo=0 错位左缘。justify 按左缘（多 float 带
                    // 内两端对齐的基线位=左缘，与单行文本同口径）。
                    // kill-switch `ZW_IFC_ALIGN_PLACE=0` 关闭（default-on）。
                    let align_place = std::env::var("ZW_IFC_ALIGN_PLACE").as_deref() != Ok("0");
                    let align = if align_place {
                        child
                            .node_id
                            .map(|id| crate::inline_finalization::resolve_text_align(styles.get(&id)))
                    } else {
                        None
                    };
                    child.x = match align {
                        Some(crate::inline::TextAlign::Right) => x_hi,
                        Some(crate::inline::TextAlign::Center) => (x_lo + x_hi) / 2.0,
                        _ => x_lo,
                    };
                    let max_w = (container_w - child.x).max(0.0);
                    if used_width > max_w {
                        if is_definite {
                            // 放不下且不可收缩 → 下推到 float 群底（保持全宽）。
                            child.y = overlapping
                                .iter()
                                .map(|(_, _, fy, _, fh, _)| fy + fh)
                                .fold(child.y, f32::max);
                        } else {
                            child.width = max_w;
                            shrink_bfc_content_width(child);
                        }
                    } else {
                        child.width = used_width;
                        shrink_bfc_content_width(child);
                    }
                } else if let Some(width) = child.declared_width_px {
                    child.width = width;
                    shrink_bfc_content_width(child);
                    // R5026：不可行（左右 float 夹空 < 盒宽）→ 盒下推到 float 群底并
                    // 回到流位 x=0（CSS 2.1 §9.5「a line box ... placed below the floated
                    // boxes」：declared-width 盒收缩救不了可行性；chromium 002r oracle
                    // 实证 span2 须 x=流位 0、y=下推 float 底——y 下推后 float 约束解除，
                    // x 也须解除）。R1733「不可行保持原位」校准只对了 x、漏了 y（当时
                    // 渲染恰未暴露 y 差）。kill-switch ZW_FLOAT_PUSH_BELOW=1 启用
                    //（**default-off**，独立于高度门——A/B 归因：case 级中性，随高度门
                    // 一同待 cleared-br 行位修清后启用）。
                    if std::env::var("ZW_FLOAT_PUSH_BELOW").as_deref() == Ok("1") {
                        child.x = 0.0;
                        child.y = overlapping
                            .iter()
                            .map(|(_, _, fy, _, fh, _)| fy + fh)
                            .fold(child.y, f32::max);
                        // 下推后容器 auto 高须覆盖新内容底（行盒高不再代表内容底，
                        // chromium 002r：body 底框随 span2 下探到 float 群底之下）。
                        // definite height 容器不涨（R1616：显式高不被 float 子覆盖）。
                        let is_auto = box_node
                            .node_id
                            .and_then(|id| styles.get(&id))
                            .is_some_and(|s| matches!(s.height, LengthValue::Auto));
                        if is_auto {
                            let needed = child.y + child.height + child.margin_bottom;
                            if needed > box_node.content_height + 0.5 {
                                let delta = needed - box_node.content_height;
                                box_node.height += delta;
                                box_node.content_height = needed;
                            }
                        }
                    }
                }
                // 不可行（无 declared width）→ 不动（避免错位）
            } else if overlapping.len() == 1 {
                // 单 float：per-float（左推右、右收缩宽）。
                let (fd, fx, _fy, fw, _fh, fmr) = overlapping[0];
                let used_width = child.declared_width_px.unwrap_or(child.width);
                // R3857：同上——仅替换元素走 push-below，BFC 保持既有收缩语义。
                let is_definite = child.is_replaced;
                match fd {
                    FloatValue::Left => {
                        let avoidance_x = *fx + *fw + *fmr;
                        if avoidance_x > child.x {
                            let max_w = (container_w - avoidance_x).max(0.0);
                            if used_width > max_w && is_definite {
                                // 放不下且不可收缩 → 下推到 float 底（保持全宽）。
                                child.y = (_fy + _fh).max(child.y);
                            } else {
                                child.x = avoidance_x;
                                if used_width > max_w {
                                    child.width = max_w;
                                } else {
                                    child.width = used_width;
                                }
                                shrink_bfc_content_width(child);
                            }
                        }
                    }
                    FloatValue::Right if *fx < child.x + used_width => {
                        let new_w = (*fx - child.x).max(0.0);
                        if used_width > new_w && is_definite {
                            // 右 float 侧放不下且不可收缩 → 下推到 float 底（保持全宽）。
                            child.y = (_fy + _fh).max(child.y);
                        } else if used_width > new_w {
                            child.width = new_w;
                            shrink_bfc_content_width(child);
                        } else {
                            child.width = used_width;
                            shrink_bfc_content_width(child);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    for child in &mut box_node.children {
        apply_inline_block_float_avoidance(child, styles);
    }
}

/// 浮动几何元组：(dir, x, y, width, height+margin_bottom, margin_right)。
/// 坐标相对「持有该浮动的容器的 border-box 原点」。供 BFC 排斥 + 嵌套透传共用。
type FloatGeom = (FloatValue, f32, f32, f32, f32, f32, f32);

/// R1623：BFC 被 float 排斥收缩 width 后，同步收缩 content_width（= width - frame）。
/// 否则内层 adjust_float_positions 递归用旧（大）content_width 做 container_width，
/// 致 BFC 内 float 不按收缩后宽换行/堆叠（floats-bfc-003 inner floats 溢出 BFC）。
fn shrink_bfc_content_width(child: &mut LayoutBox) {
    let frame = child.border_left + child.border_right + child.padding_left + child.padding_right;
    child.content_width = (child.width - frame).max(0.0);
}

/// R5217（CSS2 §9.5 规则 2 × §10.3.3）：width:auto BFC 旁置 float 的 margin-aware
/// 钉位解（chromium FloatAvoider 语义，floats-wrap-bfc-with-margin 族 68 行几何探针实证）。
///
/// flow 盒（§10.3.3 全 CB 解）为 `[ml, CB − mr]`；float 旁带为 `[band_left, band_right]`
/// （左 float = [float margin-box 右缘, CB]，右 float = [0, float margin-box 左缘]）。
/// 旁置钉位：left = max(ml, band_left)、right = min(CB − mr, band_right)——面向侧正
/// margin 被钉位吃掉（lf ml=5 → x=band_left w=gap，而非 x=band_left+ml）、背侧 margin
/// 收窄宽（rf ml=5 → x=5 w=70 @ CB=100 gap=75）。左右镜像不对称（chromium 实测）：
/// 左 float 对 right 取 `max(left + frame)` 下限再夹回 band，钉后永不因正 margin 下绕
/// （lf mr=15/22/28 全旁置 [14,2]）；右 float 钉后宽 < frame 则下绕（rf ml=15 → 下绕）。
/// 下绕几何（两 float 一致）= x=ml、w=CB − margin_h、y 到 float 底。触发条件：
/// ① 负 margin 阈 `margin_h < −(gap + 0.5)`（gap=16：−17 下绕、−16 旁置，WebKit bug
/// 239976 非互操作带取 chromium 阈）；② 仅右 float：钉后宽不足 frame（003 的
/// rf ml=15/22/28）。小负 margin（−gap ≤ margin_h < 0）由钉位自然落 `x=band_left、
/// w=gap`（同 margin:0，002-ref flex mock 不设 margin）。
/// 返回 (旁置 x, 旁置 border-box 宽, 是否下绕)。
fn solve_auto_bfc_margin_beside(
    is_left_float: bool,
    ml: f32,
    mr: f32,
    frame: f32,
    band_left: f32,
    band_right: f32,
    container_width: f32,
) -> (f32, f32, bool) {
    let margin_h = ml + mr;
    let gap = (band_right - band_left).max(0.0);
    let mut wrap = margin_h < -(gap + 0.5);
    let left = ml.max(band_left);
    let right = (container_width - mr).min(band_right);
    let right = if is_left_float {
        // 左 float：band 钉 border-box 左缘，right 取 frame 下限后夹回 band。
        right.max(left + frame).min(band_right)
    } else {
        // 右 float：右缘钉 float margin-box 左缘，钉后宽不足 frame → 下绕。
        if !wrap && right - left < frame {
            wrap = true;
        }
        right
    };
    let width = if wrap { 0.0 } else { (right - left).max(0.0) };
    (left, width, wrap)
}

/// 纯文本 float shrink-to-fit（CSS §10.3.5）预补 pass。
///
/// `adjust_float_positions` 的收缩分支仅处理含 block 级 / replaced 子元素的 float
/// （`content_child_widths`）；**纯文本 float**（无 block/replaced 子，仅直接文本）保持
/// taffy 全宽——旧注释「shrink-to-fit 需 IFC 测量，留后续」。本 pass 用
/// `text_content_max_width` 测量纯文本 float 的 max-content 宽度并收缩（仅当窄于 taffy
/// 宽度），修 `font-size: 0` float 未收缩（应 0 宽，font-size-zero-3）+ 短文本 float 撑满
/// 全宽。须在 `adjust_float_positions` **之前**运行，使后续 float 定位 / 排斥用收缩后宽度。
pub(crate) fn shrink_pure_text_floats(
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    advance_source: Option<&crate::inline::AdvanceSourceHandle>,
    font_resolver: Option<&std::rc::Rc<HashMap<String, u32>>>,
) {
    // 先递归子树（深度优先），使嵌套容器内的 float 也被处理。
    for child in &mut box_node.children {
        shrink_pure_text_floats(child, doc, styles, advance_source, font_resolver);
    }
    // 仅处理 width:auto 的 float（非替换）。非 auto 宽度 / 非 float / replaced 由别处处理。
    if !box_node.declared_width_auto || matches!(box_node.float, FloatValue::None) || box_node.is_replaced {
        return;
    }
    // 仅水平书写模式：text_content_max_width 度量水平文本宽，垂直模式的 float 其 inline 轴
    // 为垂直（block-size），物理 width 语义不同——R4462：vertical 纯文本 float 的物理宽
    // = **Σ 列宽**（ceil(文本深度/列深) × line-height）+ 横向 frame。旧路径依赖
    // shrink_vertical_blocks_to_content 后置 pass 按 max 子盒 extent 测内容宽，纯文本
    // float 无 LayoutBox 子 → content_extent=0 → float 宽塌缩 0（line-box-direction-slr-048
    // 实证），且塌缩在 step 5 float 定位**之后**（定位用 taffy 胖宽 → 多 float 误纵向堆叠）。
    // 本臂在定位前落定宽度（hyphens-vertical-* 域 depth=0 / 单列 no-op，零扰动）。
    if !matches!(box_node.writing_mode, WritingModeValue::HorizontalTb) {
        let Some(dom_id) = box_node.node_id else {
            return;
        };
        let Some(style) = styles.get(&dom_id) else {
            return;
        };
        let depth = crate::intrinsic_sizing::text_content_max_width(dom_id, doc, styles);
        if depth <= 0.0 {
            return;
        }
        // 列深（行内 extent）= definite author 高度用 content_height，auto 无约束（单列）。
        let col_depth = if !matches!(style.height, LengthValue::Auto) && box_node.content_height > 0.5 {
            box_node.content_height
        } else {
            depth
        };
        // R4463：真实 vertical IFC 测量替代算术估列——R4462 的 ceil(文本深度/列深) 不做
        // 列首空白折叠/列断尾随空白折叠，每列多计 → float 宽 +1 列/float
        //（line-box-direction-slr-048 残 9.62% 实证：float 120 应 100）。IFC 行断与
        // paint 同源（同 font resolver + 全局 measure 回调），lines.len() = 真实列数。
        let mut ifc = crate::inline::InlineFormattingContext::new(col_depth)
            .with_vertical(true)
            .with_vertical_rtl(matches!(box_node.writing_mode, WritingModeValue::VerticalRl))
            .with_no_wrap(false)
            .with_preserve_whitespace(false);
        if let Some(fr) = font_resolver {
            ifc = ifc.with_font_resolver(fr.clone());
        }
        ifc.layout(doc, dom_id, styles);
        let columns_width: f32 = ifc.lines.iter().map(|l| l.height).sum();
        if columns_width <= 0.0 {
            return;
        }
        let frame_h = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
        let shrink_border_box = columns_width + frame_h;
        if shrink_border_box < box_node.width || box_node.width <= 0.5 {
            box_node.width = shrink_border_box;
            box_node.content_width = (shrink_border_box - frame_h).max(0.0);
        }
        return;
    }
    // R4920 Σ 臂开关在本臂前段（R5242 inline-table 早退门）也要消费，提前读一次。
    let float_inline_sum_on = std::env::var("ZW_FLOAT_INLINE_SUM").as_deref() != Ok("0");
    // 已有 block 级 / replaced 子元素的 float 由 adjust_float_positions 收缩分支处理，
    // 此处不重复（避免双重 shrink / 与 block 子宽度冲突）。
    // R5242（CSS2 §9.2.2/§10.3.5）：inline-table 是行内级原子，其 LayoutBox 的
    // is_block_level 旗标是块叠装配的既有简化（postprocess.rs R5011 注在案）——float
    // shrink-to-fit 语义不认：definite 宽 inline-table 子不算 block-level，不早退，
    // 落入下方 R4920 Σ 臂（inline_children_non_text_width 的 InlineTable 分支按
    // width+margin 计盒外尺寸），preferred = 行内级内容 Σ 而非块叠 max
    //（table-vertical-align-baseline-008：wrapper 收缩 50 应 100 实证）。auto 宽
    // inline-table 不豁免（float_sigma_eligible_inline_table 谓词文档）；本豁免随
    // ZW_FLOAT_INLINE_SUM 一同关断（OFF 时 Σ 臂不可达，维持旧早退）。
    let has_block_or_replaced = box_node.children.iter().any(|c| {
        !c.is_absolute
            && !c.is_fixed
            && (c.is_block_level || c.is_replaced)
            && !(float_inline_sum_on
                && c.node_id
                    .and_then(|id| styles.get(&id))
                    .is_some_and(crate::intrinsic_sizing::float_inline_table_exempt))
    });
    if has_block_or_replaced {
        return;
    }
    // 纯文本 float：用 text_content_max_width 测 max-content（font-size:0 → 0，无文本 → 0）。
    let Some(dom_id) = box_node.node_id else {
        return;
    };
    let adjusted_text_max_w = styles
        .get(&dom_id)
        .filter(|style| !matches!(style.font_size_adjust, zero_style_system::FontSizeAdjustValue::None))
        .and_then(|_| adjusted_text_max_width(dom_id, doc, styles, advance_source?, font_resolver?));
    // R4921（css-sizing-3 §max-content）：定宽原子 inline 子停止文本递归——本臂 Σ 侧
    // inline_children_non_text_width 已计其盒外尺寸，旧 walk 再递归内文即双计（实证：
    // float > inline-block(100px) > 文本 → 80+100=180 vs 规范真值 100，R4920 组合新
    // 引入的过测；auto/百分比宽原子不定宽不受 gate，内文仍由 walk 计入单边口径）。
    // gate 仅本臂启用——text_content_max_width 其余消费方（legend/leaf/垂直 float 臂）
    // 无 Σ 配对侧，维持旧口径不扩波及面。kill-switch `ZW_INTRINSIC_ATOMIC_GATE=0` 回退。
    let text_max_w = adjusted_text_max_w.unwrap_or_else(|| {
        crate::intrinsic_sizing::text_content_max_width_scoped(
            dom_id,
            doc,
            styles,
            std::env::var("ZW_INTRINSIC_ATOMIC_GATE").as_deref() != Ok("0"),
        )
    });
    // R4920（css-sizing-3 §max-content + CSS2 §10.3.5 shrink-to-fit）：float preferred
    // width = 文本 max-content（R4919 per-font walk；R4921 起定宽原子内文经 gate 排除）
    // + **inline-level 子 margin-box 水平求和**（定宽原子 inline-block 族 + 纯 inline 子
    // frame/margin；纯 inline 子文本仍在 text_max_w 内，勿双计）。baidu「换一换」float
    //（a.hot-refresh）内 i.c-icon（inline-block width:16px）+ span（margin-left:2px）：
    // Σ = 16+2+42 = 60。taffy native float 收缩用**块流 max-content = max(子 margin-box)**
    //（ZW 把 inline 子映射为 taffy Block 节点，块流竖排取 max 语义，且 inline 子不经
    // measure 闭包）= max(16, 44) = 44 → float 窄于单行真值 60 → 内层 IFC 把「一换」折
    // 到第二行竖排（实证）。故 taffy 估值 < Σ 时须**扩**到 Σ（块流取 max 与 IFC 求和的
    // 真实偏差）；taffy 估值 ≥ Σ 时照旧收缩。adjusted_text_max_w 臂（font-size-adjust
    // 语境）已断言无元素子，Σ=0 恒不触发扩臂。
    // kill-switch `ZW_FLOAT_INLINE_SUM=0` = R4920 逐字节回退：Σ 不计入（收缩阈值还原为
    // text_max_w+frame）且扩臂关（R4921 gate 不受本开关影响，其归因用
    // `ZW_INTRINSIC_ATOMIC_GATE`）。
    let non_text_w = if adjusted_text_max_w.is_none() && float_inline_sum_on {
        // R5243（CSS2 §10.3.5）：auto 宽 inline-table 子按 [`inline_table_auto_max_content`]
        // 独立测量入 Σ（definite 部分仍由 inline_children_non_text_width 计 width+margin）。
        // 表 c.width 此刻可能是 taffy 拉伸伪影（R4032）不可用；纯 DOM 测量无伪影。主 walk
        // 对 inline-table 已停（R4921 gate R5243 扩臂），内文不再双计。
        // kill-switch 随 `ZW_FLOAT_INLINE_TABLE_MEASURE=0`（豁免谓词同关，块叠口径复原）。
        let auto_table_sum: f32 = if crate::intrinsic_sizing::float_inline_table_measure_on() {
            box_node
                .children
                .iter()
                .filter_map(|c| {
                    let id = c.node_id?;
                    let cs = styles.get(&id)?;
                    if !matches!(cs.display, DisplayValue::InlineTable) {
                        return None;
                    }
                    if crate::intrinsic_sizing::resolve_intrinsic_real_length(&cs.width, cs).is_some() {
                        return None; // definite 已由 Σ 函数按 width+margin 计
                    }
                    let ml = crate::intrinsic_sizing::resolve_intrinsic_real_length(&cs.margin_left, cs).unwrap_or(0.0);
                    let mr =
                        crate::intrinsic_sizing::resolve_intrinsic_real_length(&cs.margin_right, cs).unwrap_or(0.0);
                    // R5243（margin-right-applies-to-014 实证）：表自身 border/padding 入账
                    // ——max-content 只测内容，表框是 margin-box 的一部分；框值来自盒构建
                    // 期样式解析（非 taffy 拉伸伪影，可靠）。
                    let frame = c.border_left + c.border_right + c.padding_left + c.padding_right;
                    Some(
                        (ml + frame + crate::intrinsic_sizing::inline_table_auto_max_content(id, doc, styles) + mr)
                            .max(0.0),
                    )
                })
                .sum()
        } else {
            0.0
        };
        crate::intrinsic_sizing::inline_children_non_text_width(dom_id, doc, styles) + auto_table_sum
    } else {
        0.0
    };
    let content_max_w = text_max_w + non_text_w;
    let shrink_border_box =
        content_max_w + box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    // 仅当内容确实更窄时才收缩（对内容更宽或显式宽度为 no-op）。扩臂开关已在 non_text_w
    // 处统一门控（关断 Σ=0 → `non_text_w > 0.0` 恒 false），此处不再重复读环境。
    let expanded_adjusted_text = adjusted_text_max_w.is_some() && shrink_border_box > box_node.width;
    let expand_inline_sum = adjusted_text_max_w.is_none() && non_text_w > 0.0 && shrink_border_box > box_node.width;
    if adjusted_text_max_w.is_some() || shrink_border_box < box_node.width || expand_inline_sum {
        box_node.width = shrink_border_box;
        box_node.content_width = content_max_w;
    }
    if expanded_adjusted_text {
        let (_, line_height) = crate::inline::resolve_font_metrics(styles.get(&dom_id));
        let frame = box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
        box_node.content_height = line_height;
        box_node.height = line_height + frame;
    }
}

fn adjusted_text_max_width(
    node_id: NodeId,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    advance_source: &crate::inline::AdvanceSourceHandle,
    font_resolver: &std::rc::Rc<HashMap<String, u32>>,
) -> Option<f32> {
    if doc.child_nodes(node_id).iter().any(|child_id| {
        doc.get(*child_id)
            .is_some_and(|node| matches!(node.kind, zero_dom::NodeKind::Element(_)))
    }) {
        return None;
    }
    let style = styles.get(&node_id)?;
    let font_ids = crate::font_resolution::resolve_font_ids_for_style(
        font_resolver,
        &style.font_family,
        &style.font_weight,
        &style.font_style,
        style.font_stretch,
    );
    let text = crate::inline::collapse_whitespace(&doc.text_content(node_id)?);
    let font_size = crate::inline::resolve_inline_font_size_px(style);
    Some(advance_source.measure_text_with_font_context(
        &text,
        &font_ids,
        font_size,
        false,
        &style.font_size_adjust,
        &style.font_variation_settings,
    ))
}

/// 垂直书写模式下 width:auto 块级元素收缩到内容（CSS §10.3.3 + CSS Writing Modes §7.1）。
///
/// 规范：垂直书写模式（vertical-rl/lr）中，块级元素的 block-size 为物理 width。
/// block-size:auto 时应基于内容收缩（同水平模式下的 height:auto），而非填满包含块。
///
/// 当前架构的轴交换以**父元素**书写模式为键（converter/tree.rs 与 engine.rs
/// extract_layout）：仅当父元素为垂直模式时才交换子元素几何，使 taffy 以水平模型布局。
/// 但元素**自身**书写模式决定其 width 是 block-size 还是 inline-size。当元素自身为垂直
/// 模式而其父元素为水平模式时（典型场景：body 内一个 `writing-mode: vertical-rl` 的
/// div），轴交换不触发，taffy 把 width:auto 当作行内填充（填满容器宽度），违反规范。
///
/// 此后处理在 float 定位之后遍历布局树，对这类块按内容块轴跨度（最右侧流内子元素
/// margin-box 右缘）收缩 width，并尊重 min-width/max-width。
///
/// **自限性**：仅当内容右缘窄于当前 width 时才收缩，对子元素已正确铺满到内容右缘的
/// 垂直块为 no-op，从而对正确布局的用例零回归。
pub(crate) fn shrink_vertical_blocks_to_content(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    parent_writing_mode: &WritingModeValue,
) {
    let own_vertical = matches!(
        box_node.writing_mode,
        WritingModeValue::VerticalRl | WritingModeValue::VerticalLr
    );
    let parent_horizontal = matches!(parent_writing_mode, WritingModeValue::HorizontalTb);

    if own_vertical && parent_horizontal && box_node.is_block_level && !box_node.is_absolute && !box_node.is_fixed {
        // R4462：纯文本 float（无流内子）跳过本收缩——内容块轴跨度 = Σ 列宽（IFC 域），
        // 无 LayoutBox 子时 extent=0 会把宽度塌缩 0（line-box-direction-slr-048 float
        // 实证）；其宽已由 shrink_pure_text_floats 的 R4462 前置臂（ceil(文本深度/列深)
        // × line-height）在 float 定位前落定。非 float 空盒塌缩语义维持旧状（零风险面）。
        let is_float = !matches!(box_node.float, FloatValue::None);
        let has_in_flow_children = box_node.children.iter().any(|c| !c.is_absolute && !c.is_fixed);
        if is_float && !has_in_flow_children {
            for child in &mut box_node.children {
                shrink_vertical_blocks_to_content(child, styles, &box_node.writing_mode);
            }
            return;
        }
        // R4194（css-sizing-3 §fit-content）：bare fit-content 经 parser 映射 MaxContent
        //（R1018），converter 在 vertical + inline-size containment 下把 block 轴关键字
        // 映射 Auto（R4194 vertical_inline 臂）——taffy Auto block-size fill CB（800），
        // 须同 auto 一样收缩到内容块轴跨度（fit-content ≡ shrink-to-fit 语义）。
        let width_auto = box_node.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
            matches!(
                s.width,
                LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_)
            )
        });
        // R4997（css-sizing-4 §4.1）：CSS aspect-ratio 元素的 shrink-to-fit 有**地板**——
        // 固有 inline 尺寸经 ratio 传递的 block 尺寸（definite height × w/h，taffy leaf
        // 已落）不得被内容块轴跨度抹 0（flex-aspect-ratio-006：vertical-lr item ar 1/2 +
        // height:100 无内容 → 50 塌 0，chromium 50）；反过来内容也不被传递值钳制（§4.1
        // automatic minimum：block-aspect-ratio-014：ar 1/2 + h:100 + 子 w:100 → 内容跨度
        // 100 胜传递建议 50，chromium 100×100）。故 ar 盒取 max(内容跨度, taffy 传递宽)。
        let has_ar = box_node
            .node_id
            .and_then(|id| styles.get(&id))
            .is_some_and(|s| matches!(s.aspect_ratio, Some(r) if r > 0.0));
        if width_auto {
            // 内容块轴跨度 = 最右侧流内子元素 margin-box 右缘（相对父 border-box）。
            // R4194 注：勿改回 max(子自身宽)——多列 block 子（block-flow-direction-vrl-005
            // 等 S 形多列）的块轴跨度 = Σ 列宽，max 会过收缩（vrl-005/006 +7pp 回归实证）。
            let content_extent = box_node
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed)
                .map(|c| c.x + c.width + c.margin_right)
                .fold(0.0f32, f32::max);
            // 解析 min-width/max-width（style 解析阶段 em/rem 等已解析为 Px）。
            let (min_w, max_w) = box_node
                .node_id
                .and_then(|id| styles.get(&id))
                .map(|s| {
                    let lo = match &s.min_width {
                        LengthValue::Px(v) => *v as f32,
                        _ => 0.0,
                    };
                    let hi = match &s.max_width {
                        LengthValue::Px(v) => *v as f32,
                        _ => f32::MAX,
                    };
                    (lo, hi)
                })
                .unwrap_or((0.0, f32::MAX));
            // R4450：content-based **双向**回写（原 `.min(box_node.width)` 单向收缩）——
            // taffy 对正交 vertical 容器 auto 块轴给 fill/任意值时，content_extent（子
            // margin-box 右缘，相对本 border-box）是确定性 content 尺寸， 小于 taffy 值
            // 收缩、大于则须 grow 回（line-box-slr-060 ul：li x_local=40 + w=380 =
            // 420 > taffy 400，被 min 挡住残 6.87% 实证）。min/max-width 钳制保留。
            // R4475：坐标口径分裂 deriv（R4474 实验：+frame 对 vrl-019/srl-059 6.92→
            // 5.83 部分收敛，而 slr-054/srl-052 float 列族 9.68/8.25 翻红）——
            // R4473 extract 帧归一**仅施于非 float 容器**：其子坐标 = content-box 相对，
            // border-box 宽 = content_extent + frame；float 容器（R4473 排除臂）子坐标
            // 仍 border-box 相对，frame 已含于 content_extent（旧口径）。
            let container_is_float = !matches!(box_node.float, FloatValue::None);
            let frame_bb =
                box_node.border_left + box_node.border_right + box_node.padding_left + box_node.padding_right;
            let frame_delta = if container_is_float { 0.0 } else { frame_bb };
            let new_width = (content_extent + frame_delta).max(min_w).min(max_w);
            // R4997：ar 盒传递建议地板——max(内容跨度, taffy ratio 传递宽)（§4.1 双向语义，
            // 见上方 has_ar 注）。非 ar 盒维持既有单向 content-based 语义（R4450）。
            let new_width = if has_ar {
                new_width.max(box_node.width)
            } else {
                new_width
            };
            // R4195：table 有专属 vertical 路径（table_vertical.rs step 8 自管
            // table_block_extent + caption 逻辑 block 轴定位）——本收缩臂的宽度改写会被
            // step 8 覆盖，但**右锚平移改写子 x** 与 step 8 的 caption/行盒定位打架
            //（vrl caption-side:bottom 表 caption.x 0→20 回归实证），table 域整臂跳过。
            let is_table_container = box_node.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
                matches!(
                    s.display,
                    DisplayValue::Table
                        | DisplayValue::InlineTable
                        | DisplayValue::TableRowGroup
                        | DisplayValue::TableHeaderGroup
                        | DisplayValue::TableFooterGroup
                        | DisplayValue::TableRow
                        | DisplayValue::TableCell
                        | DisplayValue::TableCaption
                )
            });
            // R4450：双向门（原 `new_width + 0.5 < box_node.width` 单向）——grow 与 shrink
            // 同走本臂；vrl 右锚平移对 grow 情形同样成立（新右缘右移，子须右移同量）。
            if (new_width - box_node.width).abs() > 0.5 && !is_table_container {
                let frame =
                    box_node.border_left + box_node.border_right + box_node.padding_left + box_node.padding_right;
                // R4194（css-writing-modes-3 §7.1）：vertical-rl 的第一列贴**右**缘——
                // taffy 按收缩前 fill-CB 宽把子放在 content 起点（x=frame 侧），收缩后
                // 须把子重锚到新 content 右缘：右锚平移量 = (新宽 content 右缘) −
                // (子当前右缘)，先取新宽再平移（新右缘基于 new_width）。
                // R4475：锚坐标系随子坐标口径分裂——非 float 容器子 = content-box 相对，
                // 新 content 右缘（content 系）= new_width − frame_bb；float 容器子 =
                // border-box 相对，旧式 new_width − border_right。
                box_node.width = new_width;
                box_node.content_width = (new_width - frame).max(0.0);
                if box_node.writing_mode == WritingModeValue::VerticalRl {
                    let new_right_edge = if container_is_float {
                        new_width - box_node.border_right
                    } else {
                        new_width - frame_bb
                    };
                    for child in box_node.children.iter_mut() {
                        if child.is_absolute || child.is_fixed {
                            continue;
                        }
                        let child_right = child.x + child.width + child.margin_right;
                        child.x += new_right_edge - child_right;
                    }
                }
            }
        }
    }

    let pw = box_node.writing_mode.clone();
    for child in &mut box_node.children {
        shrink_vertical_blocks_to_content(child, styles, &pw);
    }
}

/// 后处理：`width:auto` 的 inline-block 收缩到内容宽度（CSS §10.3.9 shrink-to-fit）。
///
/// taffy 0.7 把 width:auto 的 inline-block 拉伸到可用宽度（如同 block），违反
/// inline-block 应 shrink-to-fit 到 max-content 的规范。此处读取流内 block 级子元素
/// 已布局的宽度（margin-box），取最大值作为内容宽度，仅在内容确实更窄时收缩
/// （内容更宽或显式宽度时为 no-op）。与 R129 float-shrink / R138 table-shrink 同谱系，
/// 但作用对象是 inline-block——其子元素已是正确尺寸（如显式 width 的 block），
/// 故仅收缩盒尺寸本身，不重排子元素。
pub(crate) fn shrink_inline_blocks_to_content(
    box_node: &mut LayoutBox,
    doc: &zero_dom::Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    inline_fonts: crate::inline_finalization::InlineFontContext<'_>,
    parent_wm: &WritingModeValue,
) {
    let own_horizontal = matches!(box_node.writing_mode, WritingModeValue::HorizontalTb);
    if !own_horizontal
        && box_node.writing_mode.is_vertical_block_flow()
        && std::env::var("ZW_ORTHO_SHRINK").as_deref() != Ok("0")
    {
        // R4437：vertical inline-block shrink-to-fit——旧入口 `own_horizontal` gate 把
        // 竖排 inline-block 全排除在本 pass 外，taffy（InlineBlock→Block 映射）拉伸宽
        // 直通终局（text-overflow-scroll-vertical-rl-001 div w=784 fill vs chromium
        // 200 实证）。收缩目标 = **列流 IFC Σ 列宽**（块轴 extent，R4432 cell /
        // R4436 caption 同模型）：列断由 definite 行内尺寸（CSS height Px，auto 回落
        // border-box−frame，再 fallback 单列）驱动。ZW_ORTHO_SHRINK=0 kill-switch。
        let is_shrinkable = !box_node.is_absolute
            && !box_node.is_fixed
            && box_node.node_id.is_some_and(|id| {
                styles.get(&id).is_some_and(|s| {
                    matches!(s.display, DisplayValue::InlineBlock)
                    // R57/R4290 同款：replaced 元素 width:auto 固有尺寸走属性/缺省尺寸路径，
                    // 不得按内容列流收缩（intrinsic-percent-replaced-029 / flexbox-vert-lr-with-img）。
                    && !box_node.node_id.is_some_and(|id2| {
                        doc.get(id2).is_some_and(|n| {
                            matches!(
                                &n.kind,
                                zero_dom::NodeKind::Element(e) if matches!(
                                    e.local_name(),
                                    "canvas" | "video" | "audio" | "iframe" | "embed" | "object" | "applet" | "img" | "svg"
                                )
                            )
                        })
                    })
                    // R4034b 同语义：contain:size → 元素按 CIS/0 sized，内容不参与
                    //（contain-intrinsic-size-017）。
                    && !s.contain.has_size()
            })
        });
        let width_auto = box_node.node_id.is_some_and(|id| {
            styles.get(&id).is_some_and(|s| {
                matches!(
                    s.width,
                    LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_)
                )
            })
        });
        // definite min/max-width 约束盒跳过（R4234/R4389 同款：恢复声明宽会打掉钳制）。
        let has_w_constraint = box_node.node_id.is_some_and(|id| {
            styles.get(&id).is_some_and(|s| {
                crate::intrinsic_sizing::resolve_intrinsic_real_length(&s.min_width, s).is_some()
                    || crate::intrinsic_sizing::resolve_intrinsic_real_length(&s.max_width, s).is_some()
            })
        });
        if is_shrinkable
            && width_auto
            && !has_w_constraint
            && let Some(id) = box_node.node_id
            && let Some(cs) = styles.get(&id)
        {
            use zero_css_parser::values::LengthValue;
            let frame_h = box_node.border_top + box_node.border_bottom + box_node.padding_top + box_node.padding_bottom;
            let depth = match &cs.height {
                LengthValue::Px(v) if v.is_finite() && *v > 0.0 => Some(*v as f32),
                _ => {
                    let fallback = box_node.height - frame_h;
                    (fallback > 0.5).then_some(fallback)
                }
            };
            let depth = depth.unwrap_or(f32::INFINITY);
            let mut col_ctx = crate::inline::InlineFormattingContext::new(depth)
                .with_vertical(true)
                .with_vertical_rtl(matches!(box_node.writing_mode, WritingModeValue::VerticalRl));
            col_ctx = crate::inline_finalization::configure_inline_fonts(col_ctx, inline_fonts, false);
            col_ctx.layout(doc, id, styles);
            // R4491：**orthogonal 限定**——列流 IFC 空（子全为 block-level 盒的竖排
            // inline-block：inline 内容为 0，"列" 即块级子自身，如 line-box-direction-
            // slr-054 的 span 块列）时回落**流内子块轴 extent 和**（CSS2 §10.6.3 auto
            // block-size 旋转类比：vertical 容器物理宽 = Σ 流内子 margin-box 块轴
            // extent；taffy 对 orthogonal 子给出的正是 Σ 子 border-box 宽初值）。旧回落
            // 单行字宽把 taffy 已正确的 120（Σ span 列宽）收缩成 20 → 父 IFC 原子步进
            // 同缩 → 兄弟盒横向重叠。
            // R4492：Σ 参与**扩展到 viv**（父亦 vertical）但**排除 sideways-lr**：
            // inline-block-alignment 家族全谱 A/B 实证——vrl（002/004/006）、vlr
            //（003/005/007）、sideways-rl（srl-008）七案由 viv Σ（.max 修正 under-measure
            // 的列流行测）翻绿 0.00-2.66%，唯独 sideways-lr（slr-009）翻红 6.06%——
            // slr 域列流/字形测量未接（R4443 谱系），其列宽参与信不过，域修复前排除。
            let orthogonal = matches!(parent_wm, WritingModeValue::HorizontalTb);
            let viv_sigma_eligible = !orthogonal
                && !(box_node.writing_mode_sideways_lr
                    && matches!(box_node.writing_mode, WritingModeValue::VerticalLr));
            let sigma_eligible = orthogonal || viv_sigma_eligible;
            let children_block_extent: f32 = if sigma_eligible {
                box_node
                    .children
                    .iter()
                    .filter(|c| {
                        !c.is_absolute && !c.is_fixed && matches!(c.float, zero_css_parser::values::FloatValue::None)
                    })
                    .map(|c| c.margin_left + c.width + c.margin_right)
                    .sum()
            } else {
                0.0
            };
            // R4491：total_height 退化（=0：竖排 IFC 对块级子的行高语义未接——slr-054
            // 探针实锤 lines=4 block_ext=0）时以 Σ 子列宽替代；R4492：Σ 合格盒（见上）
            // 非空行臂取 max（viv 行测 under-measure 修正，alignment 七案收割）；
            // slr 域单信 total_height（A/B：max 误伤 6.06%）。
            let col_total = col_ctx.total_height();
            let block_extent = if col_total > 0.5 {
                if sigma_eligible {
                    col_total.max(children_block_extent)
                } else {
                    col_total
                }
            } else if children_block_extent > 0.5 {
                children_block_extent
            } else if col_ctx.lines.is_empty() {
                let (fs, lh) = crate::inline::resolve_font_metrics(Some(cs));
                lh.max(fs)
            } else {
                col_total
            };
            let frame_w = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
            // Auto 单向收缩语义与 horizontal 臂一致（防拉伸伪影回写放大）。
            // R4491：**orthogonal**（父 horizontal-tb）臂改**双向**回写（R4445 block-ifc 臂
            // 同款 fit-content 论证：block-size:auto = fit-content）——taffy 对 orthogonal
            // inline-block 给 inline-axis 初值（行高/内容列和，缺 block 轴 frame），
            // shrink-only 时 140 content+frame 被「140 < 120 不动」守卫挡住 → 父 IFC 原子
            // 步进缺 frame → 兄弟盒重叠（slr-054 实证步进 120 应 140）。拉伸伪影方向不受
            // 影响（784 → content+frame 仍收缩）。viv（父亦 vertical）维持 shrink-only
            //（taffy 交换帧轴语义天然正确，grow 误伤 inline-block-alignment-slr-009 实证）。
            let shrink_border_box = block_extent + frame_w;
            let should_write = if orthogonal {
                (shrink_border_box - box_node.width).abs() > 0.5
            } else {
                shrink_border_box + 0.5 < box_node.width
            };
            if should_write {
                box_node.width = shrink_border_box;
                box_node.content_width = block_extent;
            }
        }
        // R4445：vertical block 容器纯 IFC 文本臂（css-writing-modes-3 §7.3 orthogonal
        // 'auto' sizing）。5.5 shrink_vertical_blocks 的 child-box 度量对纯 IFC 内容恒 0
        //（content_extent=0 → new_width=0 → w 塌 0：line-box-direction vrl-002/vlr-003/
        // slr-043 div w=0.0 vs chromium 420 实证，vrl/vlr/srl/slr 24 案同源带）。
        // 块轴尺寸 = Σ 列宽（definite 行内尺寸驱动列断，R4432 cell / R4436 caption /
        // R4437 inline-block 同模型）；回写 content-based 双向（block-size:auto =
        // fit-content 语义，须能从 5.5 塌 0 grow 回）——与 inline-block 臂的单向收缩
        //（防拉伸伪影）有意分叉。仅 definite CSS height（auto 高容器行内尺寸未定，
        // 列断深度不可信，如 line-box-slr-060 li h=704 fill 病理，另账）；无 line box
        //（真空 div，5.5 塌 0+frame ≈ chromium 无行盒块轴 0）不补偿不回写。
        let is_block_ifc = box_node.node_id.is_some_and(|id| {
            styles.get(&id).is_some_and(|s| {
                matches!(s.display, DisplayValue::Block | DisplayValue::ListItem)
                    && !box_node.node_id.is_some_and(|id2| {
                        doc.get(id2).is_some_and(|n| {
                            matches!(
                                &n.kind,
                                zero_dom::NodeKind::Element(e) if matches!(
                                    e.local_name(),
                                    "canvas" | "video" | "audio" | "iframe" | "embed" | "object" | "applet" | "img" | "svg"
                                )
                            )
                        })
                    })
                    && !s.contain.has_size()
            })
        });
        if is_block_ifc
            // 纯 Auto 限定（≠ inline-block 臂的 Auto|Min/Max/FitContent 宽集）：converter 对
            // vertical 元素按逻辑轴存储（s.width=inline-size、s.height=block-size），显式
            // inline-size 关键字（max-content 等）落 s.width——transform-3d-scales-different-
            // x-y-dynamic-001 .test（inline-size:max-content）被本臂改写后 dynamic 捕获相位
            // diff +10.06pp 实证；本臂语义 = auto block-size 收缩，显式 inline-size 另账。
            && matches!(
                box_node.node_id.and_then(|id| styles.get(&id)),
                Some(s) if matches!(s.width, LengthValue::Auto)
            )
            && !has_w_constraint
            // R4469：放开到「流内子全 inline 级且含原子子」容器——taffy 交换帧对含原子子
            // 的 inline 内容容器块轴给 Σ 子物理宽（inline-table-alignment-002 rl-mixed 450
            // = 表 300 + span 150），应 = Σ 列宽（IFC 域，单列 = max 列宽 300）。块级子在
            // 场仍排除（块子沿块轴堆叠，Σ 子宽语义成立，R4194 域）。inline 级按 style
            // display 判（extract 的 LayoutBox.is_block_level 含 Table/InlineTable，不可用）。
            // 纯文本容器（无原子子）维持原 R4445 域（definite height 限定）——font-size-
            // adjust-ic-height auto 高容器 probe 列宽与 size-adjusted 渲染度量分歧（144.8
            // vs 200）回写翻红实证。
            && (box_node.children.is_empty()
                || ({
                    let in_flow: Vec<&LayoutBox> = box_node
                        .children
                        .iter()
                        .filter(|c| !c.is_absolute && !c.is_fixed)
                        .collect();
                    in_flow.iter().all(|c| {
                        c.node_id.and_then(|cid| styles.get(&cid)).is_some_and(|s| {
                            matches!(
                                s.display,
                                DisplayValue::Inline
                                    | DisplayValue::InlineBlock
                                    | DisplayValue::InlineTable
                                    | DisplayValue::InlineFlex
                                    | DisplayValue::InlineGrid
                            )
                        })
                    }) && in_flow.iter().any(|c| {
                        c.node_id.and_then(|cid| styles.get(&cid)).is_some_and(|s| {
                            matches!(
                                s.display,
                                DisplayValue::InlineBlock
                                    | DisplayValue::InlineTable
                                    | DisplayValue::InlineFlex
                                    | DisplayValue::InlineGrid
                            )
                        })
                    })
                }))
            // float 容器排除：邻居 float 堆叠用改写前宽度定位（本 pass 晚于 float 流
            // 定位），content-based 回写引发重叠（line-box-vlr-008/slr-048 float:right
            // 容器 22.50→28.30pp 回归实证；x 平移补偿右缘无效——恶化源非自身锚定）。
            // float 自身 shrink-to-fit（§10.3.5）本应同式度量，需 float 感知回写时序，
            // 另账 slice（line-box-vrl-005/srl-045 float:left 容器待收）。
            && matches!(box_node.float, FloatValue::None)
            && let Some(id) = box_node.node_id
            && let Some(cs) = styles.get(&id)
        {
            // 列断深度：definite CSS height 优先（原 R4445 域）；原子子容器 auto 高回落
            // border-box−frame（R4437 同款，rl-mixed 450 junk 高可用——内容深度不超即不折
            // 列）；纯文本 auto 高容器 None → 整臂跳过（原 R4445 语义，见上 gate 注）。
            let has_atomic_child = box_node.children.iter().any(|c| {
                !c.is_absolute
                    && !c.is_fixed
                    && c.node_id.and_then(|cid| styles.get(&cid)).is_some_and(|s| {
                        matches!(
                            s.display,
                            DisplayValue::InlineBlock
                                | DisplayValue::InlineTable
                                | DisplayValue::InlineFlex
                                | DisplayValue::InlineGrid
                        )
                    })
            });
            let frame_h = box_node.border_top + box_node.border_bottom + box_node.padding_top + box_node.padding_bottom;
            let depth = match &cs.height {
                zero_css_parser::values::LengthValue::Px(v) if v.is_finite() && *v > 0.0 => Some(*v as f32),
                _ if has_atomic_child => {
                    let fallback = box_node.height - frame_h;
                    (fallback > 0.5).then_some(fallback)
                }
                _ => None,
            };
            if let Some(depth) = depth {
                // R4469：原子 inline-level 子尺寸注入——无注入时 CSS auto 原子子在探针 IFC
                // 降级为扁平文本（Σ 列宽虚增），与 compute_final 存储路径（R4468 注入）同源
                // 对齐，列宽度量才一致。
                let ib_sizes: HashMap<NodeId, (f32, f32)> = box_node
                    .children
                    .iter()
                    .filter(|c| !c.is_absolute && !c.is_fixed)
                    .filter_map(|c| {
                        let node_id = c.node_id?;
                        let is_atomic = styles.get(&node_id).is_some_and(|s| {
                            matches!(
                                s.display,
                                DisplayValue::InlineBlock
                                    | DisplayValue::InlineTable
                                    | DisplayValue::InlineFlex
                                    | DisplayValue::InlineGrid
                            )
                        });
                        is_atomic.then_some((node_id, (c.width, c.height)))
                    })
                    .collect();
                let mut col_ctx = crate::inline::InlineFormattingContext::new(depth)
                    .with_vertical(true)
                    .with_vertical_rtl(matches!(box_node.writing_mode, WritingModeValue::VerticalRl))
                    .with_inline_block_sizes(ib_sizes)
                    // R4469：块轴基线装配——Σ 列宽口径与 compute_final 存储路径一致（行
                    // extent = max(ascent)+max(descent)）。
                    .with_vertical_central_baselines(crate::inline_finalization::collect_vertical_central_baselines(
                        box_node, styles,
                    ));
                col_ctx = crate::inline_finalization::configure_inline_fonts(col_ctx, inline_fonts, false);
                col_ctx.layout(doc, id, styles);
                if !col_ctx.lines.is_empty() {
                    let block_extent = col_ctx.total_height();
                    let frame_w =
                        box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
                    let border_box = block_extent + frame_w;
                    if (border_box - box_node.width).abs() > 0.5 {
                        box_node.width = border_box;
                        box_node.content_width = block_extent;
                    }
                }
            }
        }
    } else if own_horizontal && !box_node.is_absolute && !box_node.is_fixed {
        // R372：除 inline-block 外，**带非默认 background 的 inline 元素**（如 morning.work
        // `.item-tag` 徽章 span：display:inline + background-color + padding）也应 shrink-to-fit。
        // ZeroWeb 把 inline 映射为 Block 拉到满宽（满宽色条），此处按 intrinsic 内容宽收缩
        // （仅 width 维度；元素仍是 block 堆叠——完整 inline-box 模型属 Phase A 多会话）。
        // 仅对有 background 的 inline 触发，避免影响纯文本 inline span（其文本经 IFC 收集，
        // 无盒装饰，收缩无意义且可能干扰）。
        // R57（M3）：replaced 元素（canvas/video/audio/iframe/embed/object/applet/img）的
        // width:auto 固有尺寸来自 HTML 属性（taffy 已解析为固有尺寸，如 canvas 400×400），
        // 其 DOM 子内容只是 fallback（canvas 的 `<p class="fallback">`）——此处按内容宽收缩
        // 会把 canvas 压到 fallback 文本宽（~188px），位图被横向压缩 0.47
        // （2d.reset.render.global_composite_operation oracle A/B 6.7%）。
        let is_replaced = box_node.node_id.is_some_and(|id| {
            doc.get(id).is_some_and(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => matches!(
                    e.local_name(),
                    // R4290：+ svg——inline `<svg>` 为 replaced 元素，width:auto 走
                    // R4000 缺省尺寸（viewBox-only 隐式 100%），不得按 DOM 子树内容宽
                    // shrink（defs/filter 等非渲染子盒被 box_content_max_width 度量 →
                    // 784 塌 20，fecolormatrix-negative @90.46% / svg-feimage-001/002
                    // @19.35% 实证；同 R57 canvas fallback 内容收缩教训）。
                    "canvas" | "video" | "audio" | "iframe" | "embed" | "object" | "applet" | "img" | "svg"
                ),
                _ => false,
            })
        });
        let is_shrinkable = !is_replaced
            && box_node.node_id.is_some_and(|id| {
                styles.get(&id).is_some_and(|s| match s.display {
                    // R783：inline-flex / inline-grid 同 inline-block 一样是 inline-level 容器，
                    // width:auto 应 shrink-to-fit（CSS §10.3.10/§10.3.11），但 taffy 0.7 把它们
                    // 当 block 拉伸到可用宽（满宽）。此前仅 inline-block 收缩，inline-flex/inline-grid
                    // 漏处理→aspect-ratio-intrinsic-size-001/003/006/008 等内联弹性盒被拉到 784px。
                    // 多 item flex-row 的 main-axis 求和语义此处用 max 近似（单 item 等价；多 item
                    // 罕见且满宽→max 仍优于 784px 拉伸）。
                    DisplayValue::InlineBlock | DisplayValue::InlineFlex | DisplayValue::InlineGrid => true,
                    DisplayValue::Inline => {
                        // R372：带非默认 background 的 inline shrink-to-fit。R1480（R109 增量）：
                        // 带 border 的 inline（如 WPT border-width-applies-to-008：display:inline +
                        // border-width:90px）亦应 shrink——否则 inline→taffy::Block 拉满宽，border
                        // 画在满宽 box（应 content-width = 内容 + 左右 border）。
                        let has_bg = s.background_color != ColorValue::Transparent;
                        let has_border = matches!(&s.border_top_width, LengthValue::Px(v) if *v > 0.0)
                            || matches!(&s.border_bottom_width, LengthValue::Px(v) if *v > 0.0)
                            || matches!(&s.border_left_width, LengthValue::Px(v) if *v > 0.0)
                            || matches!(&s.border_right_width, LengthValue::Px(v) if *v > 0.0);
                        has_bg || has_border
                    }
                    _ => false,
                })
            });
        // R4389：content 关键字（min/max/fit-content）与 auto 同为 content-based 收缩语境
        //（css-sizing-3 §4）——converter 把关键字映射 length(0)，taffy 无从测内容 →
        // inline-block 塌到 frame（ruby-intrinsic-isize-001：min/max-content div w=2.0
        // 纯边框，ruby 内容 63.1 全溢出）。
        let width_auto = box_node.node_id.is_some_and(|id| {
            styles.get(&id).is_some_and(|s| {
                matches!(
                    s.width,
                    LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_)
                )
            })
        });
        if is_shrinkable && width_auto {
            // 内容最大宽度（max-content）。R1017：InlineFlex/InlineGrid 当 box_content_max_width
            // 测得 0（aspect-ratio 空 item 等 box_content 无法度量）时，fallback 到专用 flex_intrinsic
            //（含 aspect-ratio transferred + container-cross 推导）；否则保留 box_content_max_width
            //（覆盖 gap/abspos/文本等有 content 案，避免回归）。
            let intrinsic_border_box =
                crate::intrinsic_font_measure::downloaded_text_border_width(box_node, doc, styles, inline_fonts)
                    .unwrap_or_else(|| crate::intrinsic_sizing::box_content_max_width(box_node, doc, styles));
            let intrinsic_border_box = if intrinsic_border_box > 0.5 {
                intrinsic_border_box
            } else {
                let flex_intrinsic = box_node.node_id.and_then(|id| styles.get(&id)).and_then(|s| {
                    if !matches!(s.display, DisplayValue::InlineFlex | DisplayValue::InlineGrid) {
                        return None;
                    }
                    if matches!(s.display, DisplayValue::InlineGrid) {
                        crate::intrinsic_sizing::grid_intrinsic_width(box_node, doc, styles)
                    } else if matches!(
                        s.flex_direction,
                        FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
                    ) {
                        crate::intrinsic_sizing::flex_column_intrinsic_width(box_node, doc, styles)
                    } else {
                        crate::intrinsic_sizing::flex_row_intrinsic_width(box_node, doc, styles)
                    }
                });
                flex_intrinsic.unwrap_or(0.0)
            };
            let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
            let content_max_w = (intrinsic_border_box - frame).max(0.0);
            // R4033（CSS2 §10.3.9 shrink-to-fit）：空内容 inline-block（content_max_w=0）
            // 的 shrink 目标 = frame（padding+border）而非跳过——旧 `> 0.0` guard 使
            // 「只有边框/padding 的 inline-block」保持 taffy 拉伸宽，边框画在拉伸盒右缘
            //（padding-right-applies-to-012：blue/orange 竖条渲染在容器右缘 722/782，
            // 应 8/68）。仅 frame==0 且内容==0 的纯空盒维持原状（避免空 span 塌缩面）。
            // R4406：carve-out 收窄（R4408 升 default-on）——DOM **非空**（有非空白文本/
            // 元素子，R1298 empty_inline 同款判据取反）而测得 intrinsic 0 的盒仍写 shrink
            //（目标 = frame = 0）：零宽内容（ZWSP 等）的真值就是宽 0，保持 taffy 拉伸 =
            // 满宽 bg 带伪影（improperly-contained-annotation：REF 页内层 ruby 内容仅 ZWSP +
            // display:none rt，测 0 → 跳写 → 盒宽保持拉伸 784，lightblue 带画满页宽 7.51%
            // 实证；TEST 页无内层 ruby 对称无损）。真空盒（纯空白文本/无子）维持原状——
            // whitespace-only span 的 bg 面与空 span 塌缩面都不在本 slice。
            let dom_nonempty = box_node.node_id.is_some_and(|id| {
                std::env::var("ZW_MIXED_BARE_TEXT").as_deref() != Ok("0")
                    && doc.child_nodes(id).iter().any(|&gc| match doc.get(gc) {
                        Some(n) => match &n.kind {
                            zero_dom::NodeKind::Text(t) => !t.content.trim().is_empty(),
                            zero_dom::NodeKind::Element(_) => true,
                            _ => false,
                        },
                        None => false,
                    })
            });
            if content_max_w > 0.0 || frame > 0.0 || dom_nonempty {
                let shrink_border_box = content_max_w + frame;
                // R4389：content 关键字（Min/Max/FitContent）= **定值语义**（converter length(0)
                // 伪影须双向修正——taffy 已塌到 frame 时无「收缩」可言，等值/放大同样写入）；
                // Auto 保持既有单向收缩（防拉伸伪影回写）。
                // 带 definite min/max-width 约束的元素不双向写（R4234 同款排除——
                // 恢复声明宽会打掉钳制：intrinsic-size-min-content-box-sizing
                // min-width:200 + width:min-content，taffy 已按 200 sizing）。
                let width_keyword = box_node.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
                    matches!(
                        s.width,
                        LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_)
                    )
                });
                let has_w_constraint = box_node.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
                    crate::intrinsic_sizing::resolve_intrinsic_real_length(&s.min_width, s).is_some()
                        || crate::intrinsic_sizing::resolve_intrinsic_real_length(&s.max_width, s).is_some()
                });
                // R5348（CSS2 §10.3.9 shrink-to-fit）：Auto 单向收缩的 grow 补腿。taffy 0.7 把
                // width:auto 的 inline-block 拉伸到可用宽，内容更宽时旧「shrink-only」守卫保持
                // 拉伸值——intrinsic-percent-replaced-014（inner inline-block + padding-top:100%
                // + canvas h:100% 链 → 子 transferred 100，父应 100 溢出 25 容器，实绘 25）实证。
                // grow 目标 = 流内块级子**已布局** margin-box 最大者与可用宽取大——definite/
                // transferred 子在 taffy 终局已按自然尺寸布局（DOM 域 intrinsic 测量解不了 %
                // 高链，不重测）；可用宽保下限对应 shrink-to-fit 的 min-content 下界：可换行
                // 文本 IFC 行宽 ≤ 拉伸宽不触发（不强迫 grow）。仅 InlineBlock（flex/grid 与
                // bg-inline 族另有 R1842/专用 intrinsic 路径）。ZW_IB_SHRINK_GROW=0 还原单向
                // 收缩。嵌套 inline-block 链父先于子递归处理，内层 grow 不反哺外层（FIXME）。
                let ib_grow_target = if width_keyword
                    || std::env::var("ZW_IB_SHRINK_GROW").as_deref() == Ok("0")
                    || !box_node.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
                        // R4034b 同语义（vertical 臂同款）：contain:size → 元素按
                        // CIS/0 sized，内容不参与尺寸（contain-size-button-002/
                        // inline-block-001 回归实证）。
                        matches!(s.display, DisplayValue::InlineBlock) && !s.contain.has_size()
                    }) {
                    0.0
                } else {
                    let avail = (box_node.width - frame).max(0.0);
                    box_node
                        .children
                        .iter()
                        .filter(|c| !c.is_absolute && !c.is_fixed && matches!(c.float, FloatValue::None))
                        .map(|c| c.margin_left + c.width + c.margin_right)
                        .fold(avail, f32::max)
                };
                let mut write_w = shrink_border_box;
                let mut write_cw = content_max_w;
                let mut grew_by_laid_out = false;
                let should_write = if width_keyword && has_w_constraint {
                    // R4389：约束关键字盒完全跳过——taffy 已按 definite min/max sizing
                    //（intrinsic-size-min-content-box-sizing：min-width:200 → 当前 200，
                    // max-content 目标 22 写入即打掉钳制，双向/单向皆害）。
                    false
                } else if width_keyword {
                    (shrink_border_box - box_node.width).abs() > 0.5
                } else if shrink_border_box + 0.5 < box_node.width {
                    true
                } else if ib_grow_target > box_node.width + 0.5 {
                    write_w = ib_grow_target;
                    write_cw = (ib_grow_target - frame).max(0.0);
                    grew_by_laid_out = true;
                    true
                } else {
                    false
                };
                if should_write {
                    box_node.width = write_w;
                    box_node.content_width = write_cw;
                    // 父 gate：仅 inline-block 族（§10.3.9-11 真 shrink-to-fit 容器）同步子宽。
                    // R372 bg-inline 的收缩是 ZW 绘制近似（CSS 层子仍是 inline 语义），其「块级子」
                    // 是 ZW 映射伪影，同步反而破坏 list-item/margin 链（008 等 5 案回归实证）。
                    let parent_is_ib = box_node.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
                        matches!(
                            s.display,
                            DisplayValue::InlineBlock | DisplayValue::InlineFlex | DisplayValue::InlineGrid
                        )
                    });
                    if parent_is_ib {
                        // R4023（CSS2 §10.3.3 + §10.3.9）：inline-block shrink-to-fit 收缩自身后，
                        // 其 in-flow 块级子的包含块宽随之变小——子 CSS width:auto 应 = 新内容宽
                        //（taffy 第一趟按收缩前可用宽拉伸子到 784，之后无重排，子保持拉伸值；
                        // 子带背景时溢出收缩后的父盒可见：inline-block-zorder-005 黄条 791）。
                        // 仅同步「CSS width:auto 且当前宽 > 新内容宽」的块级子（收缩方向；显式宽、
                        // inline-level 子、定位子不动），并同步 content_width 供后续 pass 使用。
                        for child in &mut box_node.children {
                            if child.is_absolute || child.is_fixed {
                                continue;
                            }
                            let child_is_block = child.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
                                !matches!(
                                    s.display,
                                    DisplayValue::Inline
                                        | DisplayValue::InlineBlock
                                        | DisplayValue::InlineFlex
                                        | DisplayValue::InlineGrid
                                        | DisplayValue::InlineTable
                                )
                            });
                            if !child_is_block {
                                continue;
                            }
                            let child_auto_width = child
                                .node_id
                                .and_then(|id| styles.get(&id))
                                .is_some_and(|s| matches!(s.width, LengthValue::Auto));
                            if !child_auto_width {
                                continue;
                            }
                            let child_frame =
                                child.padding_left + child.padding_right + child.border_left + child.border_right;
                            let target_border_box = write_cw + child.margin_left + child.margin_right + child_frame;
                            // R5348：grow 腿双向同步（auto 块级子 = 新包含块宽，§10.3.3）；
                            // 收缩路径维持 R4023 单向（child 宽 > 新内容才写）零扰动。
                            if (grew_by_laid_out && (child.width - target_border_box).abs() > 0.5)
                                || (!grew_by_laid_out && child.width > target_border_box + 0.5)
                            {
                                child.width = target_border_box;
                                child.content_width = (target_border_box - child_frame).max(0.0);
                            }
                        }
                    }
                }
            }
            // R1842 grow：inline-grid + width:auto 须确保容器宽 >= grid intrinsic（含 definite
            // track 之和）。taffy 对空/小内容 inline-grid 给 content=0（仅 frame），上方 shrink
            // 只缩不扩，致容器欠扩（grid-container-baseline-synthesized：grid-template-columns:60px
            // + 空子 → 容器 w=6 应 66）。仅 InlineGrid + 当前 border-box < intrinsic 时 grow（窄条件，
            // 不影响 inline-flex/inline-block 或已正确尺寸的 inline-grid）。A/B 守 net≥0。
            if box_node
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.display, DisplayValue::InlineGrid))
                && let Some(intrinsic_bb) = crate::intrinsic_sizing::grid_intrinsic_width(box_node, doc, styles)
                && intrinsic_bb > box_node.content_width + frame + 0.5
            {
                box_node.content_width = (intrinsic_bb - frame).max(0.0);
                box_node.width = intrinsic_bb;
            }
        }

        // R3794c（css-flexbox §9.4.4 + css-sizing-4 §4.1）：column flex 容器的 item transferred
        // cross。taffy 第一趟在容器宽度尚未收缩（auto→可用宽）时做 cross-stretch，item 宽被拉到
        // 可用宽（784）；上方 shrink 只缩容器自身（读 item CSS main=50），不回写 item cross——
        // flex-aspect-ratio-013/014/024：容器缩到 50/100 但 item 仍 784。此处对**列方向** flex
        // 容器的 item（aspect-ratio + cross CSS auto）按 flex 主轴拉伸后的 main（= 容器内容高，
        // flex-grow≥1 拉伸到行尺寸）transferred：cross = stretched_main × ratio；容器 cross 取
        // max(item cross)。013：item h 拉伸 100 → w=100，容器 100×100 绿方块。
        // 仅 item cross CSS auto（显式宽由 converter 处理）；仅水平书写模式。
        let is_column_flex = box_node.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
            // 仅 inline-level flex 容器（shrink-to-fit 语义，§10.3.10）：block-level flex
            // width:auto 拉满可用宽（R1015 契约——r1015_block_flex_column_auto_width_no_shrink），
            // item cross = 容器 stretch，不 transferred、不收缩容器。
            matches!(s.display, DisplayValue::InlineFlex)
                && matches!(
                    s.flex_direction,
                    FlexDirectionValue::Column | FlexDirectionValue::ColumnReverse
                )
        });
        if is_column_flex && own_horizontal {
            // 容器宽度 CSS definite（显式 Px 等）时 item cross 以容器为上限（align-stretch 语义）；
            // auto（shrink-to-fit）时 transferred 可超当前宽并把容器撑到 max(item cross)。
            let container_width_definite = box_node
                .node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| !matches!(s.width, LengthValue::Auto));
            let inner_w = box_node.content_width;
            let mut max_cross = 0.0f32;
            for item in &mut box_node.children {
                let Some(item_id) = item.node_id else { continue };
                let Some(item_style) = styles.get(&item_id) else {
                    continue;
                };
                if !matches!(item_style.width, LengthValue::Auto) {
                    continue;
                }
                // transferred 来源：item 解析后的主轴（border-box 高，taffy 拉伸后）× ratio。
                let Some(ratio) = item_style.aspect_ratio.filter(|&r| r > 0.0) else {
                    continue;
                };
                let item_frame = item.padding_left + item.padding_right + item.border_left + item.border_right;
                let main =
                    (item.height - item.padding_top - item.padding_bottom - item.border_top - item.border_bottom)
                        .max(0.0);
                if main <= 0.5 {
                    continue;
                }
                // box-sizing 决定 aspect-ratio 作用盒（css-sizing-4 §4.1）：border-box →
                // ratio 作用于 border-box，cross_bb = main_bb × ratio（main 已是 border-box 高，
                // 不加 frame——flex-item-transferred-sizes-padding-border-sizing：border-box +
                // padL/R 25 + min-height 100 + ratio 1 应 100 宽，+frame 会双计 padding 150）。
                // content-box → ratio 作用于 content-box，cross_bb = main_content × ratio + frame。
                let cross_raw = if matches!(
                    item_style.box_sizing,
                    zero_style_system::property::types::BoxSizingValue::BorderBox
                ) {
                    main * ratio
                } else {
                    main * ratio + item_frame
                };
                let cross = if container_width_definite {
                    cross_raw.min(inner_w.max(1.0))
                } else {
                    cross_raw
                };
                if cross > 0.5 && (item.width - cross).abs() > 0.5 {
                    item.width = cross;
                    item.content_width = (cross - item_frame).max(0.0);
                }
                max_cross = max_cross.max(cross);
            }
            // 容器 shrink-to-fit 收敛到 item transferred cross 的最大者（双向：013 容器 50 应
            // 100 需 grow；definite 容器不动）。
            if !container_width_definite && max_cross > 0.5 && (box_node.width - max_cross).abs() > 0.5 {
                let container_frame =
                    box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
                box_node.width = max_cross;
                box_node.content_width = (max_cross - container_frame).max(0.0);
            }
        }
    }

    let own_wm = box_node.writing_mode.clone();
    for child in &mut box_node.children {
        shrink_inline_blocks_to_content(child, doc, styles, inline_fonts, &own_wm);
    }
}

/// R4468：vertical 容器 inline-level 子**行内 extent（物理高）拉伸清除**。
///
/// 病理（inline-table-alignment-002 实证）：vertical 容器的 inline-level 子（inline-table/
/// inline-block/span）在 taffy 交换帧被按 block 子拉伸——taffy block 布局对 auto 宽
///（= 交换帧子的物理行内 extent 槽，tree.rs apply_vertical_writing_mode 交换 +
/// engine.rs extract_layout 交换还原）子施加 stretch fill（taffy compute/block.rs
/// `width.unwrap_or(stretch_width)`），stretch 参照 = 容器 taffy inner width：
/// - 顶层 vertical 容器（HorizontalTb 父，自身样式不交换）= 容器 pre-restack 物理宽
///   （rl-mixed 784 = body 内容宽）；
/// - 嵌套 vertical 容器（样式已交换）= 容器物理高（inline extent 槽，自自身拉伸值级联）。
///
/// 全子树行内 extent 滞留同一拉伸值（inline-table/row/span h=784，应 30-120），下游
/// IFC 原子项/行盒 extent 以拉伸 LayoutBox 为测源（垃圾进垃圾出，R4467 定谳）。
///
/// 规范（css-writing-modes-3 §7.1 + CSS2 §10.3.9/§17.4）：inline-level 盒行内尺寸
/// shrink-to-fit，**不**填充包含块行内尺寸（块级子仅在 definite 行内尺寸容器 fill，
/// R4460/vrl-021 域）。本 pass 在 IFC 消费者（5.6 inline-block shrink / 6.x adjust /
/// 6.5 remeasure / 12 compute_final）之前运行，从源头切断污染。
///
/// 泄漏签名（防误伤）：容器 vertical + 子 in-flow 非替换 + 子 CSS height Auto（definite
/// 尊重声明）+ `child.height ≈ 泄漏参照`（taffy stretch 值，leak_ref 推导见 walk）。
/// 目标类型二分：
/// - **原子 inline-level / inline 叶**（InlineBlock/InlineTable/InlineFlex/InlineGrid/
///   Inline）：行内 extent = 有流内块级子时 max(子 outer extent)（行/块沿块轴堆叠），
///   否则纯文本叶 = 自身字体 strut（resolve_font_metrics CSS line-height）；
/// - **原子容器（inline-table/inline-block）的块级子**（table-row / display:block span）：
///   仅当容器 CSS 行内尺寸 Auto（definite 容器块级子 fill 语义正确，不触）且子为纯
///   inline 内容（叶）→ strut extent。
///
/// 单向收缩（new < old，同 R4437「防拉伸伪影回写放大」）。`ZW_VIV_SIZING` 同族
/// kill-switch。页级 fast path：无 vertical 样式整臂跳过（R4460 预扫描先例）。
pub(crate) fn shrink_vertical_stretched_inline_extents(
    root: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    inline_fonts: crate::inline_finalization::InlineFontContext<'_>,
) {
    if std::env::var("ZW_VIV_SIZING").as_deref() == Ok("0") {
        return;
    }
    // 页级 fast path：无 vertical 书写模式的页整臂跳过。
    if !styles.values().any(|s| s.writing_mode.is_vertical_block_flow()) {
        return;
    }
    let provider = inline_fonts.metric_provider;
    walk_stretched_inline_extents(
        root,
        &WritingModeValue::HorizontalTb,
        0.0,
        false,
        false,
        doc,
        styles,
        provider,
    );
}

/// 泄漏参照（leak_ref）= 本盒子的 taffy inner width——子的物理行内 extent（taffy 宽槽）
/// 被 stretch 到该值：
/// - 顶层 vertical 盒（HorizontalTb 父，自身未交换）：taffy 宽 = 物理宽 → content_width；
/// - 嵌套 vertical 盒（自身已交换）：taffy 宽 = 物理高 → height − 行内轴 frame
///   （taffy 水平 frame 经交换 = CSS top/bottom padding+border）。
fn inline_extent_leak_ref(b: &LayoutBox, parent_wm: &WritingModeValue) -> f32 {
    if matches!(parent_wm, WritingModeValue::HorizontalTb) {
        b.content_width
    } else {
        b.height - b.padding_top - b.padding_bottom - b.border_top - b.border_bottom
    }
}

/// 纯 inline 内容盒的行内 extent（content 域）= 自身字体 strut（CSS line-height，
/// 含 provider 真实度量；Ahem A+D=font-size 与 line-height:1 同值）。
fn inline_content_strut_extent(
    style: &ComputedStyle,
    provider: Option<&crate::inline::FontMetricProviderHandle>,
) -> f32 {
    let (_, line_height) = crate::inline::resolve_font_metrics_with_provider(Some(style), provider);
    line_height.max(0.0)
}

#[allow(clippy::too_many_arguments)]
fn walk_stretched_inline_extents(
    b: &mut LayoutBox,
    parent_wm: &WritingModeValue,
    leak_ref: f32,
    parent_is_atomic_inline: bool,
    parent_inline_size_auto: bool,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    provider: Option<&crate::inline::FontMetricProviderHandle>,
) {
    let own_vertical = b.writing_mode.is_vertical_block_flow();
    // 本盒子的子的泄漏参照（自本盒原始几何推导——本盒自身高度由父层在本 walk 返回后修复，
    // 此刻仍为 taffy 原值）。
    let child_ref = if own_vertical {
        inline_extent_leak_ref(b, parent_wm)
    } else {
        0.0
    };
    let own_style = b.node_id.and_then(|id| styles.get(&id));
    let own_is_atomic_inline = own_style.is_some_and(|s| {
        matches!(
            s.display,
            DisplayValue::InlineBlock | DisplayValue::InlineTable | DisplayValue::InlineFlex | DisplayValue::InlineGrid
        )
    });
    // vertical 盒的 CSS 行内尺寸 = height：definite 时块级子 fill 语义正确（R4460 域），
    // 块级子臂不触；Auto 时 taffy stretch 参照级联自祖先泄漏值 = 垃圾。
    let own_inline_size_auto = own_style.is_some_and(|s| matches!(s.height, LengthValue::Auto));
    for c in &mut b.children {
        walk_stretched_inline_extents(
            c,
            &b.writing_mode,
            child_ref,
            own_is_atomic_inline,
            own_inline_size_auto,
            doc,
            styles,
            provider,
        );
    }

    // 修复本盒：仅 vertical 父下的目标类型（父 wm 已随递归传入）。
    if !parent_wm.is_vertical_block_flow() || leak_ref <= 0.5 {
        return;
    }
    let Some(id) = b.node_id else { return };
    let Some(style) = styles.get(&id) else { return };
    // 通用 gate：in-flow / 非替换 / CSS height Auto（definite 尊重声明）。
    if b.is_absolute
        || b.is_fixed
        || !matches!(b.float, FloatValue::None)
        || b.is_replaced
        || !matches!(style.height, LengthValue::Auto)
        || !b.height.is_finite()
        || (b.height - leak_ref).abs() > 0.5
    {
        return;
    }
    let is_inline_leaf = matches!(style.display, DisplayValue::Inline)
        && !doc.child_nodes(id).iter().any(|&c| {
            doc.get(c)
                .is_some_and(|n| matches!(n.kind, zero_dom::NodeKind::Element(_)))
        });
    // 原子容器的块级子（table-row 在 extract 的 is_block_level 名单外，按 display 判）。
    // 容器 CSS 行内尺寸 definite 时块级子 fill 正确（R4460/slr-054 域）不触。
    let is_atomic_container_child = parent_is_atomic_inline
        && parent_inline_size_auto
        && (b.is_block_level
            || matches!(
                style.display,
                DisplayValue::TableRow
                    | DisplayValue::TableRowGroup
                    | DisplayValue::TableHeaderGroup
                    | DisplayValue::TableFooterGroup
                    | DisplayValue::TableCell
                    | DisplayValue::TableCaption
            ));
    let extent = if !is_inline_leaf && !is_atomic_container_child {
        let is_atomic = matches!(
            style.display,
            DisplayValue::InlineBlock | DisplayValue::InlineTable | DisplayValue::InlineFlex | DisplayValue::InlineGrid
        );
        if !is_atomic {
            return;
        }
        // 原子容器：有沿块轴堆叠的流内子（block 级 / table-internal，行内 extent = 最高子，
        // 子已自底向上修复）→ max(子 outer extent)；纯文本叶 → strut；其余（嵌套 inline
        // 元素子）保守跳过。
        let is_blockish = |c: &LayoutBox| -> bool {
            c.node_id
                .and_then(|cid| styles.get(&cid))
                .map(|s| {
                    !matches!(
                        s.display,
                        DisplayValue::Inline
                            | DisplayValue::InlineBlock
                            | DisplayValue::InlineFlex
                            | DisplayValue::InlineGrid
                            | DisplayValue::InlineTable
                    )
                })
                .unwrap_or(c.is_block_level)
        };
        let max_block_child = b
            .children
            .iter()
            .filter(|c| is_blockish(c) && !c.is_absolute && !c.is_fixed && matches!(c.float, FloatValue::None))
            .map(|c| c.height + c.margin_top + c.margin_bottom)
            .fold(0.0_f32, f32::max);
        if max_block_child > 0.5 {
            max_block_child
        } else if doc.child_nodes(id).iter().any(|&c| {
            doc.get(c)
                .is_some_and(|n| matches!(n.kind, zero_dom::NodeKind::Element(_)))
        }) {
            return;
        } else {
            inline_content_strut_extent(style, provider)
        }
    } else {
        // inline 叶（无元素子）/ 原子容器的纯 inline 内容块级子 → strut。
        inline_content_strut_extent(style, provider)
    };
    if extent <= 0.5 {
        return;
    }
    let frame_v = b.padding_top + b.padding_bottom + b.border_top + b.border_bottom;
    let new_h = extent + frame_v;
    // 单向收缩：仅在显著小于拉伸值时回写（防 extent 高估放大）。
    if new_h + 0.5 < b.height {
        b.height = new_h;
        b.content_height = extent.max(0.0);
    }
}

/// 标记孤立 table-internal 元素（CSS Tables §2.4）为匿名 table 根。
///
/// 当 `display:table-row-group/table-row/table-cell/...` 出现在非 table 上下文中
///（父元素非 table/table-internal）时，CSS 规范应为其生成匿名 table 包装盒。
/// 此预遍历近似该行为：把这类孤立元素的 `is_anon_table_root` 置真，使其在
/// `establishes_bfc` 中被视为匿名 table（建立 BFC，隔离 margin 折叠 + 包含浮动）。
/// 在 `adjust_float_positions` 之前运行，确保 float exclusion 识别这些容器。
pub(crate) fn mark_anonymous_table_roots(
    box_node: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    in_table_context: bool,
) {
    let display = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .map(|s| s.display.clone());
    let is_table = matches!(display, Some(DisplayValue::Table | DisplayValue::InlineTable));
    let is_table_internal = matches!(
        display,
        Some(
            DisplayValue::TableRowGroup
                | DisplayValue::TableHeaderGroup
                | DisplayValue::TableFooterGroup
                | DisplayValue::TableRow
                | DisplayValue::TableCell
                | DisplayValue::TableCaption
        )
    );

    if is_table_internal && !in_table_context {
        box_node.is_anon_table_root = true;
        // 孤立 table-internal 充当匿名 table（块级），使其被 adjust_float_positions
        // 的 clear / BFC float-exclusion 逻辑（要求 is_block_level）正确处理。
        box_node.is_block_level = true;
    }

    let child_context = in_table_context || is_table || is_table_internal;
    for child in &mut box_node.children {
        mark_anonymous_table_roots(child, styles, child_context);
    }
}

/// R1392：递归收集非 BFC 后代中的浮动元素外底边（content-relative 到外层容器）。
///
/// CSS §9.5：clear 须清除同 BFC 上下文内的所有浮动。ZW 的 `active_left/right_float_bottom`
/// 仅收集**直接** float 子，嵌套在非 BFC 后代中的 float 对后续 clear 兄弟不可见 →
/// clear 失效（adjoining-float-before-clearance：float 嵌在 wrapper 内，clear:left 看不到它）。
/// BFC 后代建立独立浮动上下文（其内 float 不外溢），递归在 BFC 边界停止。
///
/// `child_border_y` = child 的 y 值累加基点：content-rel 模式（默认）下为
/// content-box 相对，border-rel 兼容模式下为 border-box 相对（R1324/R4235 帧约定）。
/// `outer_content_y_offset` = 外层 border→content 偏移，用于换算 content-relative。
///
/// R1392 余项：content-rel 模式（R1324/R4235）下所有子 y 均为 content-box 相对，
/// 递归下降时 c_border_y 除累加 child 的 y 外还须加上 child 自身的 border_top +
/// padding_top（frame 分量），否则中间非 BFC 容器的 frame 把嵌套浮动底边虚减，
/// 后续 clear 兄弟与嵌套浮动叠压。border-rel 兼容模式 child y 已含 frame，不重复加。
/// https://www.w3.org/TR/CSS22/visuren.html#clearance
fn nested_float_bottoms(child: &LayoutBox, child_border_y: f32, outer_content_y_offset: f32) -> (f32, f32) {
    use zero_css_parser::values::FloatValue;
    let mut left = 0.0f32;
    let mut right = 0.0f32;
    // 下降基点：child 的 content-box 原点（content-rel 模式相对外层 content-box；
    // border-rel 模式 child y 已含 frame，基点即 child 的 border-box 顶）。
    let child_content_y = child_border_y
        + if *FLOAT_CLAMP_CONTENT_REL {
            child.border_top + child.padding_top
        } else {
            0.0
        };
    for c in &child.children {
        if c.is_absolute || c.is_fixed {
            continue;
        }
        // content-rel 模式 c.y 相对 child 的 content-box 原点（border-rel 模式相对
        // border-box 原点，此时 child_content_y == child_border_y）。
        let c_border_y = child_content_y + c.y;
        if !matches!(c.float, FloatValue::None) {
            let bottom = c_border_y - outer_content_y_offset + c.height + c.margin_bottom;
            match c.float {
                FloatValue::Left => left = left.max(bottom),
                FloatValue::Right => right = right.max(bottom),
                _ => {}
            }
        } else if !crate::margin_collapse::establishes_bfc(c) {
            // 非 BFC 后代：其内浮动与外层同 BFC，继续递归收集。
            let (l, r) = nested_float_bottoms(c, c_border_y, outer_content_y_offset);
            if l > left {
                left = l;
            }
            if r > right {
                right = r;
            }
        }
        // BFC 后代：独立浮动上下文，停止递归。
    }
    (left, right)
}

/// R5307：递归收集非 BFC 后代中的浮动元素几何（[`FloatGeom`]，x/y 为外层容器
/// content-box 相对；y 为 border-box 顶、高含 margin_bottom，与 `float_geometries`
/// 同约定）。
///
/// CSS §9.5.1：同 BFC 上下文内 float 不得与更早 float 重叠。`float_geometries` 仅
/// 收集直接 float 子，嵌套在非 BFC wrapper 内的 float 对兄弟子树的 float 不可见 →
/// 兄弟块内的后继 float 与其叠压（position-absolute-dynamic-relayout-004：float2
/// 落 y=0 与 wrapper 内 float1 全叠，chromium 须落到 float1 底下方）。与
/// [`nested_float_bottoms`] 同帧约定与 BFC 边界（BFC 后代独立上下文，停止递归），
/// 供递归透传把嵌套 float 并入后继兄弟的继承带集（earliest-fit 带试探避开）。
/// 注：[`FloatGeom`] 不含 margin_top，带顶以 border-box 顶近似（mt>0 时带略窄；
/// x 区间为 margin-box 精确）。
/// https://www.w3.org/TR/CSS22/visuren.html#float-position
fn nested_float_geoms(
    child: &LayoutBox,
    child_border_x: f32,
    child_border_y: f32,
    outer_content_y_offset: f32,
) -> Vec<FloatGeom> {
    use zero_css_parser::values::FloatValue;
    let mut geoms = Vec::new();
    // 下降基点：child 的 content-box 原点（帧约定同 [`nested_float_bottoms`]，
    // X 同理：content-rel 模式下 child.x/y 为外层 content-box 相对，加 frame 分量；
    // border-rel 兼容模式 child x/y 已含 frame，不重复加）。
    let (child_content_x, child_content_y) = if *FLOAT_CLAMP_CONTENT_REL {
        (
            child_border_x + child.border_left + child.padding_left,
            child_border_y + child.border_top + child.padding_top,
        )
    } else {
        (child_border_x, child_border_y)
    };
    for c in &child.children {
        if c.is_absolute || c.is_fixed {
            continue;
        }
        let (c_border_x, c_border_y) = (child_content_x + c.x, child_content_y + c.y);
        if !matches!(c.float, FloatValue::None) {
            geoms.push((
                c.float.clone(),
                c_border_x,
                c_border_y - outer_content_y_offset,
                c.width,
                c.height + c.margin_bottom,
                c.margin_right,
                c.margin_left,
            ));
        } else if !crate::margin_collapse::establishes_bfc(c) {
            // 非 BFC 后代：其内浮动与外层同 BFC，继续递归收集。
            geoms.extend(nested_float_geoms(c, c_border_x, c_border_y, outer_content_y_offset));
        }
        // BFC 后代：独立浮动上下文，停止递归。
    }
    geoms
}

/// R5210（CSS2 §8.3.1 × §9.5.2）：首流内子链的 clear 阻断折叠值重解析。
/// 自 `box_node` 起沿「父↔首流内子」可折叠对下行，收集各环 declared mt 的折叠值；
/// 返回 `Some((folded, break_mt))` = 链上存在 clear 阻断点（阻断点 mt 留本级、
/// 不计入链值），`folded` 为阻断点以上链值、`break_mt` 为阻断点字段 mt（供击发
/// 门核对 taffy 确已把整链折进 `box_node` 字段）。返回 `None` = 链在抵达 clear
/// 阻断点前遇到不可穿越边界（顶 border/padding、BFC、空块自折叠、非块级首子
/// 即行内内容、无流内子）→ 不激活，维持既有字段语义（零行为变化）。
/// https://www.w3.org/TR/CSS22/visuren.html#flow-control
fn chain_clear_break_fold(box_node: &LayoutBox) -> Option<(f32, f32)> {
    use zero_css_parser::values::{ClearValue, FloatValue};
    let no_clear = |c: &ClearValue| matches!(c, ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd);
    let mut folded = box_node.declared_margin_top;
    let mut cur = box_node;
    loop {
        // 可穿越环：普通块级、无顶框边、非 BFC、非空块（空块自折叠走 R1317 链）。
        if cur.border_top != 0.0
            || cur.padding_top != 0.0
            || crate::margin_collapse::establishes_bfc(cur)
            || !cur.is_block_level
            || crate::margin_collapse::is_empty_block(cur)
        {
            return None;
        }
        // 首个流内子（定位/浮动不入对）；首个流内子非块级 = 行内内容，链断。
        let first = cur
            .children
            .iter()
            .find(|c| !c.is_absolute && !c.is_fixed && matches!(c.float, FloatValue::None))?;
        if !first.is_block_level {
            return None;
        }
        if !no_clear(&first.clear) {
            // clear 阻断点：§8.3.1「the child has no clearance」——其 mt 留本级。
            return Some((folded, first.margin_top));
        }
        folded = crate::margin_collapse::collapse_two_margins(folded, first.declared_margin_top);
        cur = first;
    }
}

/// R5210 链值重解析开关（`ZW_CLEAR_CHAIN_MT_REFOLD=0` 关闭，default-on）。
/// OPTIMIZATION：kill-switch 进程级单读（R5108 教训：per-node env::var 致
/// bench-gate 微基准超预算）。
static CLEAR_CHAIN_MT_REFOLD: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_CLEAR_CHAIN_MT_REFOLD").as_deref() != Ok("0"));
/// R5212（CSS2 §9.5.2 forced clearance）：direct-float 邻接吸收 kill-switch，
/// default-on，`ZW_FORCED_CLEAR_ABSORB=0` 回退。
static FORCED_CLEAR_ABSORB: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_FORCED_CLEAR_ABSORB").as_deref() != Ok("0"));

/// R5214：float 摆放 earliest-fit（CSS 2.1 §9.5.1 规则 5「as high as possible」）——
/// float 放不进当前可用区间时下移到与其纵向带相交 float 的**最近**底边逐带试探，
/// 替代旧行推进（整行下移到当前行 max float 底）。chromium CDP 探针（R5213）定谳
/// float 与 BFC 盒同用 earliest-fit；floats-wrap-bfc-004 ref 侧 purple float 期望
/// (100,6)（silver 底 6 带 beside blue，非行底 20）。kill-switch
/// `ZW_FLOAT_EARLIEST_FIT=0` 回退行推进。
static FLOAT_EARLIEST_FIT: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("ZW_FLOAT_EARLIEST_FIT").as_deref() != Ok("0"));

#[allow(clippy::too_many_arguments)]
pub(crate) fn adjust_float_positions_with_context(
    box_node: &mut LayoutBox,
    box_content_abs_y: f32,
    inherited_left_bottom_abs: f32,
    inherited_right_bottom_abs: f32,
    // R1618/R1619 Slice 2：祖先 BFC 上下文内的浮动几何（已转换到本容器 border-box 帧）。
    // 非 BFC 子递归时透传，使嵌套 BFC（如 overflow:hidden 在 margin-div 内）能避开
    // 祖先 float（CSS §9.5：BFC border-box 不重叠同 BFC 上下文内任意 float）。
    // env ZW_NESTED_BFC_FLOAT_AVOID=0 关闭（kill-switch，default-on）。
    inherited_floats: &[FloatGeom],
    doc: &zero_dom::Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    // R5307：子树含 float 旗标表（[`adjust_float_positions`] 入口预扫）；`None` =
    // 全树无 float（布尔单趟判定），收集器全跳过。
    float_subtrees: Option<&HashMap<NodeId, bool>>,
) {
    use zero_css_parser::values::ClearValue;
    use zero_css_parser::values::FloatValue;

    // R1277 ②：inline 级子（is_block_level=false，如 `<span>` display:inline）不推进
    // flow_bottom——CSS §9.5.1 仅 block-level 前置内容约束 float 的 outer top 边。
    // 旧实现把 inline→taffy::Block 映射的 span 当 block 累入 flow_bottom，致后续 float
    // 被推到 inline 内容之下（floats-006 float 落 rel_y=100 而非 0）。须与 ④（显式高度
    // 守卫）协调上 default：② 单独 net -3 floats-clear（R1272），②+④ 同码 A/B 全 10 dir
    // NET 0 + floats-006 11.54→4.79（R1277）。env `ZW_FLOAT_LIFT_INLINE=0` 可关闭
    //（kill-switch，default-on；与 ④ 绑定，单独关闭会重现 -3）。
    let lift_inline = std::env::var("ZW_FLOAT_LIFT_INLINE").as_deref() != Ok("0");

    // 容器的内容区域宽度
    let container_width = box_node.content_width;

    // R4235：本 walk 的读/写按 **border 盒相对** y 约定写就（child.y ± content_y_offset），
    // 但引擎提取层（engine.rs extract_layout）已把全部流内子节点 y 换算为**内容盒相对**
    // ——border/padding 非零的容器内，float 的 line_y 定位写点与钳制假设流位各偏
    // +content_y_offset（c414-flt-fit-001：border 5px 容器内 float p 整体 +5px，与无
    // float 的 ref 页不对称暴露）。content-rel 模式（默认开）在两处写点/钳制读点去掉
    // offset；kill-switch `ZW_FLOAT_CLAMP_CONTENT_REL=0` 回退旧 border-rel 约定。
    // LazyLock 缓存（R4214 教训：walk 热路径逐次 env::var 兼作 LLVM 优化栅栏，净负）。
    let clamp_content_rel = *FLOAT_CLAMP_CONTENT_REL;

    // CSS Flexbox §4 / Grid §4 / Tables §2.4：flex/grid/table 容器的流内子元素
    //（即布局项）其 `float` 与 `clear` 不产生浮动或清除效果——`float` 计算为 `none`。
    // taffy 内部已据此布局，但 ZeroWeb 的浮动后处理（本函数）按 `child.float` 重新
    // 定位，会把带 `float:right` 的 flex item 误推到容器右缘。此处对布局容器父级的
    // 直接子元素将 `float` 归零，使后处理（含 paint 的 float 排斥/绘制）一致忽略它。
    if box_node.is_layout_container {
        for child in &mut box_node.children {
            child.float = FloatValue::None;
        }
    }

    // taffy 子元素的 Y 坐标是相对于父元素的 border-box 原点，
    // 而 flow_bottom / line_y 等追踪变量是相对于 content area 原点。
    // 当容器有 border-top 或 padding-top 时，需要加上偏移量。
    let content_y_offset = box_node.border_top + box_node.padding_top;
    let inherited_left_bottom = (inherited_left_bottom_abs - box_content_abs_y).max(0.0);
    let inherited_right_bottom = (inherited_right_bottom_abs - box_content_abs_y).max(0.0);

    // CSS §8.3.1 修正：float 的 margin 不与父容器折叠。但 taffy 把 float 当作普通
    // block 排列，当容器的首个流内子元素是 float 且容器无 border-top/padding-top
    //（margin 可与子元素折叠）时，容器的 margin-top 会被折叠到该 float 的 margin
    //（取 max），使容器（及其全部内容）整体偏低。此处把多折叠的量从容器 y 中扣除
    // 并恢复 margin_top。
    //
    // 精确门控（四重条件，确保只修真正的 float-margin 折叠，排除 taffy 把 float 当
    // block 引起的其它膨胀）：
    //   1. 容器无 border-top/padding-top（margin 可与首个子元素折叠）
    //   2. 容器布局 margin_top > 声明值（发生了膨胀）
    //   3. 容器 margin_top == 首个 float 子元素 margin_top（容器 mt 被折叠到该 float mt）
    //   4. 该 float 子元素的 margin_top == 其声明值（float 的 mt 自身未被 taffy 膨胀）
    if content_y_offset == 0.0 && box_node.margin_top > box_node.declared_margin_top + 0.01 {
        if let Some(fc) = box_node
            .children
            .iter()
            .find(|c| !c.is_absolute && !c.is_fixed)
            .filter(|c| !matches!(c.float, FloatValue::None))
        {
            let container_absorbed_float_mt = (box_node.margin_top - fc.margin_top).abs() < 0.01;
            let float_mt_is_clean = (fc.margin_top - fc.declared_margin_top).abs() < 0.01;
            if container_absorbed_float_mt && float_mt_is_clean {
                let over_collapse = box_node.margin_top - box_node.declared_margin_top;
                box_node.y -= over_collapse;
                box_node.margin_top = box_node.declared_margin_top;
            }
        }
    }

    // R4140（CSS §8.3.1）：**inline-level 子的 margin 不与块容器折叠**。taffy 把
    // inline-block 等原子行内级映射为 Block 子，当容器首个流内子是原子且容器无
    // border/padding-top 时，容器 margin-top 被折叠抬升到该原子的 mt（取 max）；
    // 而原子自身的行内定位（IFC run.y，R3904/R3809）又应用一次 mt → 双计（容器及
    // 全部后代整体偏低 over_collapse）。探针实证（debug_r4140_layout_dump）：body
    // margin:8 + 首子 inline-block mt:32 → body y=32 应 8；原子 y 相对容器已由 IFC
    // 正确摆位（=mt），故只需把容器多折叠的量从 y 扣除并恢复 margin_top——与上方
    // float 修正同一形状。门控（五重）：
    //   1. 容器无 border-top/padding-top（margin 可与首个子元素折叠）
    //   2. 容器布局 margin_top > 声明值（发生了膨胀）
    //   3. 首个流内子为 inline-level（!is_block_level；含 anonymous text item 时
    //      其无 LayoutBox 或 mt=0，天然不匹配）
    //   4. 容器 margin_top == 该子 margin_top（被折叠到该子的 mt）
    //   5. 该子 margin_top == 声明值（自身未被 taffy 膨胀）
    //   kill-switch：env `ZW_INLINE_FIRST_MT_UNHOIST=0` 关闭（default-on）。
    unhoist_inline_first_atom_mt(box_node, content_y_offset);

    // 第一阶段：重新定位 float 元素，记录每个 float 在 taffy 布局中占据的垂直空间
    //
    // CSS 2.1 §9.5.1 float 定位规则：
    // 1. Float 必须尽可能高（"as high as possible"）
    // 2. Float 的 outer top 不得高于前面元素生成的块的 outer top
    // 3. Float 不参与 margin 折叠
    //
    // 关键：min_line_y 必须基于正常流内容的实际位置（考虑 margin 折叠），
    // 而不是 taffy 的 Y（taffy 将 float 当作 block，包含了前元素的 margin 折叠）。
    let mut line_y = 0.0f32;
    let mut line_max_height = 0.0f32;
    let mut left_used_width = 0.0f32;
    let mut right_used_width = 0.0f32;
    let mut left_float_bottom = inherited_left_bottom;
    let mut right_float_bottom = inherited_right_bottom;

    // 正常流的垂直位置追踪（用于计算 float 的最小 Y）
    // 关键：flow_bottom 必须独立于 taffy 的 Y 来计算。
    // taffy 将 float 当作 block 排列，导致后续正常流元素的 Y 偏移过大。
    // 我们通过累加正常流元素的高度 + margin 折叠来独立追踪 flow_bottom。
    let mut flow_bottom = 0.0f32; // 上一个正常流元素的 border-bottom（相对于容器 content area）
    let mut last_flow_mb = 0.0f32; // 上一个正常流元素的 margin-bottom
    // R5227：尾随匿名片段不计入 float 行游标的 (flow_bottom, last_flow_mb) 快照——
    // R109 纯 inline 匿名块盒（fragment_node_ids 记账，tree/extract 强制
    // is_block_level=true）语义上是原子 inline-level 内容（§9.2.1.1），§9.5.1 float
    // 外顶与其前置行盒**顶边**齐平（R5215 floats-placement-006/007/008 同模型；
    // chromium 探针 between-float-and-text float rel_y = anon_top + mt = 20）。
    // 片段推进前重拍（float 落该行顶）；非片段流内子推进后重拍为推进后状态
    // （[文本行, 块, float] 的 float 仍取块后流游标 = 50，R5226
    // r5226_float_after_block_cursor_anchor 同值）。无片段史时与
    // (flow_bottom, last_flow_mb) 恒等，既有锚零扰动。
    let mut flow_snap: (f32, f32) = (0.0, 0.0);
    // 第一个流内子元素的 margin-top 会与无 border-top/padding-top 的父容器折叠
    //（CSS §8.3.1），此时子元素位于容器 content 原点，其 margin-top 不应计入 flow_bottom，
    // 否则后续 float 的 min_float_y 会把该 margin-top 双重计入，使 float 偏低。
    let mut first_in_flow = true;

    // 记录每个 float 子元素在 taffy 布局中的 Y 和高度，用于后续偏移修正
    let mut float_taffy_y: Vec<(usize, f32, f32)> = Vec::new(); // (index, taffy_y, outer_height)
    // R5214：已放置 float 的 margin-box 带记录（is_right, margin 左缘, margin 右缘,
    // 带顶, 带底），earliest-fit 带试探用。kill-switch 关闭时不 push，仅空 Vec。
    // R5307：以前序同上下文 inherited float 带初始化（§9.5.1 float 不得与更早 float
    // 重叠）——嵌套非 BFC 兄弟块内的 float 收到的 inherited 带此前不参与落位试探，
    // 放不进 inherited float 旁时仍从 y=0 落位叠压。帧换算：inherited geom 为本容器
    // border-box 帧，带集为 content 帧（减 content_y_offset）；x 区间 margin-box
    //（x−ml, x+w+mr，同下方 placed push 约定）。带顶以 geom border-box 顶近似
    //（FloatGeom 无 margin_top，mt>0 时带略窄，见 nested_float_geoms 注）。
    // https://www.w3.org/TR/CSS22/visuren.html#float-position
    let mut placed_bands: Vec<(bool, f32, f32, f32, f32)> = inherited_floats
        .iter()
        .map(|(d, fx, fy, fw, fh, fmr, fml)| {
            (
                matches!(d, FloatValue::Right),
                *fx - *fml,
                *fx + *fw + *fmr,
                *fy - content_y_offset,
                *fy - content_y_offset + *fh,
            )
        })
        .collect();

    for (idx, child) in box_node.children.iter_mut().enumerate() {
        // 跳过绝对定位和 fixed 元素
        if child.is_absolute || child.is_fixed {
            continue;
        }

        // 正常流元素：独立更新 flow_bottom
        // 不使用 taffy 的 Y（包含 float 垂直空间），而是自行累加
        if matches!(child.float, FloatValue::None) && !child.is_absolute && !child.is_fixed {
            // 第一个流内子元素的 margin-top 与无 border-top/padding-top 的父容器折叠：
            // margin-top 上浮到父容器外，子元素位于 content 原点，不计入 flow_bottom。
            let parent_collapses_first = first_in_flow && content_y_offset == 0.0;
            first_in_flow = false;
            // R1277 ②：inline 级子（is_block_level=false）不推进 flow_bottom——
            // CSS §9.5.1 仅 block-level 前置内容约束 float 的 outer top。
            if !lift_inline || child.is_block_level {
                // R5227：匿名片段推进前先拍快照（float 的行顶锚基准，见 flow_snap 注释）。
                if child.fragment_node_ids.is_some() {
                    flow_snap = (flow_bottom, last_flow_mb);
                }
                // 独立计算正常流位置：使用 margin 折叠
                if crate::margin_collapse::is_empty_block(child) {
                    let collapsed_self_margin =
                        crate::margin_collapse::collapse_two_margins(child.margin_top, child.margin_bottom);
                    last_flow_mb = crate::margin_collapse::collapse_two_margins(last_flow_mb, collapsed_self_margin);
                } else {
                    let collapsed_margin = if parent_collapses_first {
                        0.0
                    } else {
                        crate::margin_collapse::collapse_two_margins(last_flow_mb, child.margin_top)
                    };
                    let child_y = flow_bottom + collapsed_margin;
                    let child_border_bottom = child_y + child.height;
                    flow_bottom = child_border_bottom;
                    last_flow_mb = child.margin_bottom;
                }
                if child.fragment_node_ids.is_none() {
                    flow_snap = (flow_bottom, last_flow_mb);
                }
            }
            // 处理非 float 元素的 clear 属性（延迟到第二阶段）
            if matches!(child.float, FloatValue::None) {
                // R1833：X-staircase fix。非-float clear 元素（如 clear:both 空 div）原被
                // 此处 continue 跳过，Phase 1 float X 定位用的 left/right_used_width 永不
                // reset → 后续行 float 继承累积宽度 = X-staircase（flexbox-collapsed-item-
                // horiz-001 fill x=13/25/57/89/121/153/205 阶梯，-001 cluster 谱系，R1823
                // 锁定）。fix：clear 元素须 reset left/right_used_width（Y clearance 仍延迟
                // Phase 2，line 569 注释原意保留）。kill-switch `ZW_FLOAT_XSTAIRCASE_FIX=0`。
                if std::env::var("ZW_FLOAT_XSTAIRCASE_FIX").as_deref() != Ok("0") {
                    match child.clear {
                        ClearValue::Left if left_float_bottom > line_y => {
                            line_y = left_float_bottom;
                            left_used_width = 0.0;
                            right_used_width = 0.0;
                            line_max_height = 0.0;
                        }
                        ClearValue::Right if right_float_bottom > line_y => {
                            line_y = right_float_bottom;
                            left_used_width = 0.0;
                            right_used_width = 0.0;
                            line_max_height = 0.0;
                        }
                        ClearValue::Both => {
                            let clear_y = left_float_bottom.max(right_float_bottom);
                            if clear_y > line_y {
                                line_y = clear_y;
                                left_used_width = 0.0;
                                right_used_width = 0.0;
                                line_max_height = 0.0;
                            }
                        }
                        _ => {}
                    }
                }
                continue;
            }
        }

        // 记录 float 元素的 taffy Y 位置和高度
        let child_outer_height = child.margin_top + child.height + child.margin_bottom;
        float_taffy_y.push((idx, child.y, child_outer_height));

        // CSS §10.3.5：width:auto 的浮动非替换元素应 shrink-to-fit 到内容宽度。
        // taffy 把 float 当作普通 block（填满可用宽度），此处对 width:auto 且有
        // 块级子元素的 float 收缩到子元素最大 border-box 宽度（仅当窄于当前宽度）。
        // 纯文本内容（无块级子元素）的 float 保持 taffy 宽度——其 shrink-to-fit 需
        // IFC 测量，留作后续。仅当内容确实更窄时才收缩，对内容更宽或显式宽度的 float 为 no-op。
        // R4060（css-contain-1 §containment-size）：contain:size float 的内容不参与
        // sizing——shrink-to-fit 内容宽 = 0，浮动按钮收缩到 chrome（padding+border），
        // 非子底边宽（contain-size-button-002 floatLBasic：144 应 44）。
        let content_suppressed = child.has_size_containment;
        if child.declared_width_auto {
            // 内容宽度候选：块级子元素取最大 border-box 宽度，**inline-level replaced
            // 子元素**（img/video 等原子 inline 盒，已有确定 used 宽度）也纳入——R180 教训
            //「content 宽须 inline 级求和 + block 级取最大」。旧实现仅 `is_block_level` 致
            // float div 仅含 `<img>`（inline-level）时 content_child_widths 为空→跳过收缩→
            // float 撑满全宽，img 无法覆盖 div 背景（max-width-110，red 68400px 外露）。
            // replaced 的 used width 已解析（含 max-width 等约束），无需 IFC 测量。
            // 纯文本 float（无 block 级、无 replaced 子元素）仍保持 taffy 宽度（需 IFC，留后续）。
            // R5242（与 shrink_pure_text_floats 同口径）：definite 宽 inline-table 子是
            // 行内级原子，其 preferred 宽已由 5a.1 的 R4920 Σ 臂先行落定——不入块叠 max
            //（块叠 max 会把 Σ 结果重收缩回 max(子)，撤掉行内求和；008 实证 100→50）。
            // R5243：auto 宽 inline-table 同豁免（Σ 补源见 5a.1；豁免后 content_child_widths
            // 为空时本臂跳过收缩，float 保持 5a.1 扩臂值——块叠 max(50) 不再撤 Σ(100)）。
            // ZW_FLOAT_INLINE_SUM=0 两处同关；`ZW_FLOAT_INLINE_TABLE_MEASURE=0` 收窄回
            // R5242 definite 语义（float_inline_table_exempt 内部门控）。
            let float_inline_sum_on = std::env::var("ZW_FLOAT_INLINE_SUM").as_deref() != Ok("0");
            let content_child_widths: Vec<f32> = child
                .children
                .iter()
                .filter(|c| {
                    !c.is_absolute
                        && !c.is_fixed
                        && (c.is_block_level || c.is_replaced)
                        && !(float_inline_sum_on
                            && c.node_id
                                .and_then(|id| styles.get(&id))
                                .is_some_and(crate::intrinsic_sizing::float_inline_table_exempt))
                })
                .map(|c| c.width)
                .collect();
            let content_max_w = content_child_widths.iter().copied().fold(0.0f32, f32::max);
            // https://www.w3.org/TR/CSS21/visudet.html#float-width
            // In the max-content measurement used by shrink-to-fit, floated siblings may
            // share a line.  Their combined outer widths therefore contribute to the
            // containing float's preferred width; using only the widest child makes a
            // floated navigation list wrap one item per line.
            let floated_children_width = child
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed && !matches!(c.float, FloatValue::None))
                .map(|c| c.margin_left + c.width + c.margin_right)
                .sum::<f32>();
            let content_max_w = content_max_w.max(floated_children_width);
            // 有块级、replaced 或浮动子元素时，按首选内容宽度计算 used width。
            // **content_max_w 可能为 0**（如 visibility:collapse 的 flex item 主尺寸归零，
            // 或空内容块）——旧条件 `content_max_w > 0.0` 在此跳过收缩致 float 撑满全宽
            //（flexbox-collapsed-item-horiz-001 根因，R300）。改为「有内容子元素即收缩」：
            // 空内容 float 收缩到 padding+border（最小盒），仍比全宽更接近 shrink-to-fit 语义。
            // 纯文本 float（无 block 级/replaced 子元素）保持 taffy 宽度——其 shrink-to-fit
            // 需 IFC 测量，留后续。
            if content_suppressed {
                // contain:size：内容宽 0 → chrome-only 盒。
                let preferred_border_box =
                    child.padding_left + child.padding_right + child.border_left + child.border_right;
                let available_border_box = (container_width - child.margin_left - child.margin_right).max(0.0);
                let used_border_box = preferred_border_box.min(available_border_box).max(0.0);
                if used_border_box < child.width {
                    let delta_w = child.width - used_border_box;
                    child.width = used_border_box;
                    child.content_width = (used_border_box
                        - child.padding_left
                        - child.padding_right
                        - child.border_left
                        - child.border_right)
                        .max(0.0);
                    let _ = delta_w;
                }
            } else if !content_child_widths.is_empty() || floated_children_width > 0.0 {
                let preferred_border_box =
                    content_max_w + child.padding_left + child.padding_right + child.border_left + child.border_right;
                let available_border_box = (container_width - child.margin_left - child.margin_right).max(0.0);
                // R5044（css-sizing-3 shrink-to-fit = min(max(min-content, available),
                // preferred)）：available < min-content 时地板胜（width:0 容器内带块级子的
                // float 不塌 0）。仅限本臂（有 block/replaced/floated 子——preferred 可信；
                // 纯文本/空 float 的 preferred 未测不入——R5043 变体 A 教训）；min-content
                // 经 R5044 min 模式递归真测量（fit-content 中间层 50 非 max 近似 150）。
                let min_content_border_box = (crate::intrinsic_sizing::block_min_content_width(child, doc, styles)
                    + child.padding_left
                    + child.padding_right
                    + child.border_left
                    + child.border_right)
                    .max(0.0);
                let used_border_box = preferred_border_box
                    .min(available_border_box)
                    .max(min_content_border_box);
                if used_border_box < child.width || floated_children_width > 0.0 {
                    child.width = used_border_box;
                    child.content_width = content_max_w;
                }
            } else if child.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
                // R5045（css-sizing-4 §4.1）：叶/空 float 的 AR transferred preferred——
                // ratio + definite height + width:auto 的 preferred = transfer 宽
                //（intrinsic-size-003：height:100px + ratio 1/1 → shrink-to-fit =
                // min(max(100, available=0), 100) = 100 而非 taffy 塌 0）。preferred 与
                // min-content 同源可测（block_max/min_content_width 叶 own_ar，R5042）。
                // 纯文本 float（无 ratio）仍不入——preferred 需 IFC 整行测量
                //（R5043 变体 A 教训：preferred=0 会让地板塌成词宽）。
                s.aspect_ratio.filter(|&r| r > 0.0).is_some()
                    && crate::intrinsic_sizing::resolve_intrinsic_real_length(&s.height, s).is_some()
                    && matches!(s.width, LengthValue::Auto)
            }) {
                let frame = child.padding_left + child.padding_right + child.border_left + child.border_right;
                let preferred_border_box = crate::intrinsic_sizing::block_max_content_width(child, doc, styles) + frame;
                let available_border_box = (container_width - child.margin_left - child.margin_right).max(0.0);
                let min_content_border_box =
                    crate::intrinsic_sizing::block_min_content_width(child, doc, styles) + frame;
                let used_border_box = preferred_border_box
                    .min(available_border_box)
                    .max(min_content_border_box)
                    .max(0.0);
                if (used_border_box - child.width).abs() > 0.5 {
                    child.width = used_border_box;
                    child.content_width = (used_border_box - frame).max(0.0);
                }
            }
        }

        // 计算浮动元素的总占用尺寸（含 margin）
        let child_outer_width = child.margin_left + child.width + child.margin_right;

        // CSS 2.1 §9.5.1 float 定位约束：
        // 1. Float 的 outer top 不得高于前面正常流元素生成的块盒的 outer top
        // 2. Float 不参与 margin 折叠
        //
        // CSS 2.1 §9.5.1：float 的 margin box 顶边按正常流规则确定。
        // 这里的 `line_y` / `child.y` 都追踪的是 border-box 顶边，因此用于
        // 约束 float 最小垂直位置时，不能再次把当前元素的 margin-top 加进去，
        // 否则后续 `child.y = line_y + margin_top` 会双计 margin-top。
        // R5227：min_float_y 取快照（尾随匿名片段不推进 float 行游标，见 flow_snap 注释）。
        let min_float_y = flow_snap.0 + flow_snap.1;
        if min_float_y > line_y {
            line_y = min_float_y;
            left_used_width = 0.0;
            right_used_width = 0.0;
            line_max_height = 0.0;
        }

        // 处理 float 元素自身的 clear 属性
        match child.clear {
            ClearValue::Left => {
                if left_float_bottom > line_y {
                    line_y = left_float_bottom;
                    left_used_width = 0.0;
                    right_used_width = 0.0;
                    line_max_height = 0.0;
                }
            }
            ClearValue::Right => {
                if right_float_bottom > line_y {
                    line_y = right_float_bottom;
                    left_used_width = 0.0;
                    right_used_width = 0.0;
                    line_max_height = 0.0;
                }
            }
            ClearValue::Both => {
                let clear_y = left_float_bottom.max(right_float_bottom);
                if clear_y > line_y {
                    line_y = clear_y;
                    left_used_width = 0.0;
                    right_used_width = 0.0;
                    line_max_height = 0.0;
                }
            }
            ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd => {}
        }

        // R5214（CSS 2.1 §9.5.1 规则 5「as high as possible」）：earliest-fit 带试探——
        // 从单调 floor（line_y，已含 min_float_y 与 clear 抬升）起，放不进
        // [左系 float 右缘, 右系 float 左缘] 区间就下移到与 [y, y+h) 相交 float 的
        // 最近底边重试，替旧行推进（整行下移到当前行 max float 底，R1612 表 C 步
        // 曾对齐本模型；R5213 表 C 步已改 earliest-fit，本处补齐 float 侧同模型）。
        // chromium CDP 探针定谳两侧同模型（R5213；floats-wrap-bfc-004 ref purple
        // float 期望 (100,6)：silver 底 6 带 beside blue，非行底 20）。
        // 带记录 = margin-box（x−ml, x+w+mr, top, bottom）；零高度 float 带底==带顶
        // 恒不相交、不约束后续（旧行模型消耗行宽，差异记 corpus 仲裁）。
        // R5307 墙模型（§9.5.1 规则 1/2/3）：带可来自 inherited 透传（含外层
        // containing block 外的 float，x 区间可负/超容器），区间墙不再默认钳容器宽——
        // 左系 float 左墙锚容器左缘（规则 1），右墙仅当左侧确有相交左系带（规则 2
        // 「another left-floating box to its left」）时取容器右缘，否则无墙、仅受相交
        // 右系带左缘约束（规则 3，可溢出容器右缘：floats-rule3-outside-left-001 蓝
        // 425 宽入 400 容器贴 (0,0)，右系带 450..500 不相交右缘 425）；右系 float
        // 镜像（右墙锚容器右缘=规则 1、有相交右系带时左墙钳容器左缘=规则 2 镜像、
        // 相交左系带右缘无 0 钳=规则 3 镜像：floats-rule3-outside-right-001 蓝
        // 425 宽 x=−25 越容器左缘）。旧行为（双墙恒容器宽）下这些案被误判放不下
        // 而下移。https://www.w3.org/TR/CSS22/visuren.html#float-position
        let earliest_fit = if *FLOAT_EARLIEST_FIT {
            let mut band_y = line_y;
            let mut resolved: Option<(f32, f32)> = None; // (左缘, 右缘)
            let is_right_float = matches!(child.float, FloatValue::Right);
            for _ in 0..(placed_bands.len() + 2) {
                // 相交带联合墙：左系带右缘最大值 / 右系带左缘最小值（±∞ 无相交）。
                let mut left_wall = f32::NEG_INFINITY;
                let mut right_wall = f32::INFINITY;
                let mut has_left_neighbor = false;
                let mut has_right_neighbor = false;
                for b in &placed_bands {
                    if b.3 < band_y + child_outer_height && band_y < b.4 {
                        if b.0 {
                            // 右系 float：其 margin 左缘（b.1）收紧区间右界。
                            has_right_neighbor = true;
                            right_wall = right_wall.min(b.1);
                        } else {
                            // 左系 float：其 margin 右缘（b.2）收紧区间左界。
                            has_left_neighbor = true;
                            left_wall = left_wall.max(b.2);
                        }
                    }
                }
                let (left_edge, right_edge) = if is_right_float {
                    (
                        left_wall.max(if has_right_neighbor { 0.0 } else { f32::NEG_INFINITY }),
                        container_width.min(right_wall),
                    )
                } else {
                    (
                        left_wall.max(0.0),
                        right_wall.min(if has_left_neighbor {
                            container_width
                        } else {
                            f32::INFINITY
                        }),
                    )
                };
                if child_outer_width <= (right_edge - left_edge).max(0.0) + 0.5 {
                    resolved = Some((left_edge, right_edge));
                    break;
                }
                // 放不下 → 推进到相交 float 的最近底边（earliest-fit）。相交条件
                // 保证各相交 band 底 > band_y，最小值必存在；band_y 严格单调推进，
                // 迭代数以 float 数为上界（+2 余量）。
                let next_band = placed_bands
                    .iter()
                    .filter(|b| b.3 < band_y + child_outer_height && band_y < b.4)
                    .map(|b| b.4)
                    .fold(f32::INFINITY, f32::min);
                if !next_band.is_finite() {
                    // 无相交 float 仍放不下（宽超容器）：原位贴边放置（旧行模型
                    // line_max_height==0 同语义，§9.5.1 左/右外缘贴包含块边）。
                    resolved = Some((left_edge, right_edge));
                    break;
                }
                band_y = next_band;
            }
            resolved.map(|(left_edge, right_edge)| {
                let x = match child.float {
                    FloatValue::Left => left_edge + child.margin_left,
                    FloatValue::Right => right_edge - child_outer_width + child.margin_left,
                    _ => child.x,
                };
                (x, band_y)
            })
        } else {
            None
        };

        if let Some((placed_x, band_y)) = earliest_fit {
            if matches!(child.float, FloatValue::Left | FloatValue::Right) {
                child.x = placed_x;
                child.y = if clamp_content_rel {
                    band_y + child.margin_top
                } else {
                    content_y_offset + band_y + child.margin_top
                };
                // 规则 4：后继 float 外顶不得高于更早 float 外顶——floor 单调取本
                // float 落位顶（行推进模型同性质，line_y 单调不降）。
                line_y = band_y;
                match child.float {
                    FloatValue::Left => {
                        let new_bottom = band_y + child_outer_height;
                        left_float_bottom = left_float_bottom.max(new_bottom);
                    }
                    FloatValue::Right => {
                        let new_bottom = band_y + child_outer_height;
                        right_float_bottom = right_float_bottom.max(new_bottom);
                    }
                    _ => {}
                }
                placed_bands.push((
                    matches!(child.float, FloatValue::Right),
                    child.x - child.margin_left,
                    child.x + child.width + child.margin_right,
                    band_y,
                    band_y + child_outer_height,
                ));
            }
        } else {
            // 检查当前行是否有足够空间放置此浮动元素
            // FIXME: 行模型（earliest-fit kill-switch 回退路径）无 inherited 带集——
            // R5307 带初始化仅earliest-fit 路径消费，回退下嵌套兄弟块内 float 仍可能与
            // 同上下文前序 float 叠压（默认配置 earliest-fit 开，不影响 corpus）。
            let available_width = container_width - left_used_width - right_used_width;
            if child_outer_width > available_width && line_max_height > 0.0 {
                line_y += line_max_height;
                left_used_width = 0.0;
                right_used_width = 0.0;
                line_max_height = 0.0;
            }

            match child.float {
                FloatValue::Left => {
                    child.x = left_used_width + child.margin_left;
                    child.y = if clamp_content_rel {
                        line_y + child.margin_top
                    } else {
                        content_y_offset + line_y + child.margin_top
                    };

                    left_used_width += child_outer_width;
                    let new_bottom = line_y + child_outer_height;
                    left_float_bottom = left_float_bottom.max(new_bottom);
                }
                FloatValue::Right => {
                    right_used_width += child_outer_width;
                    child.x = container_width - right_used_width + child.margin_left;
                    child.y = if clamp_content_rel {
                        line_y + child.margin_top
                    } else {
                        content_y_offset + line_y + child.margin_top
                    };

                    let new_bottom = line_y + child_outer_height;
                    right_float_bottom = right_float_bottom.max(new_bottom);
                }
                FloatValue::InlineStart | FloatValue::InlineEnd | FloatValue::None => {}
            }
        }

        // CSS 2.1 §9.5.1：零高度浮动元素（margin-box 高度为 0）不推进 line_y。
        // 一个没有内容、没有 border、没有 padding 的空浮动元素不应占据垂直空间，
        // 后续浮动元素应从相同的 line_y 开始。
        if child_outer_height > 0.0 {
            line_max_height = line_max_height.max(child_outer_height);
        }
    }

    // 第二阶段：修正非 float 子元素的 Y 位置 + BFC 浮动排斥
    // CSS 规范中 float 元素脱离正常流，不应占据垂直空间。
    // taffy 将 float 当作正常 block 排列，导致后续非 float 元素的 Y 偏移过大。
    // 策略：维护一个 float_y_offset（累积 float 在 taffy 中占据的垂直空间），
    // 对每个非 float 子元素从 Y 中扣除 offset；clear 元素消耗 offset。
    //
    // BFC 浮动排斥（CSS 2.1 §9.5）：建立 BFC 的块级元素不得与浮动元素重叠。
    // 当一个非 float 块级元素建立 BFC 且与浮动元素垂直重叠时，
    // 需将其水平位置偏移到浮动元素旁边。
    let has_active_float_context =
        !float_taffy_y.is_empty() || inherited_left_bottom > 0.0 || inherited_right_bottom > 0.0;
    // R1393：adjoining-float 吸收是否在本容器触发（触发后须跑 containment 收缩容器高度，
    // 否则被吸收的 margin 仍留在容器高度里显红）。函数级——在 if has_active_float_context
    // 块内设置，containment gate（块外）读取。
    let mut ran_adjoining_clearance = false;
    // R1393：adjoining-float——容器无直接 float 子，但非 BFC 后代内嵌套 float + 有 clear 子时，
    // 须走主 clearance 路径（else 分支 R1389 按「无 float context」处理，clear 看不到嵌套
    // 浮动）。窄 gate：仅同时有 clear 子 + 嵌套浮动才扩 has_active_float_context，避免影响
    // 普通容器。env `ZW_ADJOINING_FLOAT_CLEARANCE=0` 关闭（kill-switch，default-on）。
    // R5028：clear 子认领扩到 **cleared inline-level br 盒**（display:inline，
    // is_block_level=false 被 R1393 旧门拒之门外——p1{float 子}+br{clear}+p2 的 body
    // 走 else 路径，br 的 clear 整体旁路，p2 比 chromium 低一个 strut）。br-only 位移
    // 块零高度由 R5027 折叠臂（ZW_BR_CLEAR_ZEROH）承接。env
    // `ZW_BR_CLEAR_ELSE_PATH=0` 关闭（kill-switch，default-on）。
    let br_clear_gate = std::env::var("ZW_BR_CLEAR_ELSE_PATH").as_deref() != Ok("0");
    let has_active_float_context = has_active_float_context
        || (std::env::var("ZW_ADJOINING_FLOAT_CLEARANCE").as_deref() != Ok("0")
            && box_node.children.iter().any(|c| {
                (c.is_block_level
                    || (br_clear_gate
                        && c.node_id.is_some_and(|id| {
                            doc.get(id).is_some_and(
                    |n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br")),
                )
                        })))
                    && !c.is_absolute
                    && !c.is_fixed
                    && !matches!(
                        c.clear,
                        ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
                    )
            })
            && box_node.children.iter().any(|c| {
                !c.is_absolute
                    && !c.is_fixed
                    && matches!(c.float, FloatValue::None)
                    && !crate::margin_collapse::establishes_bfc(c)
                    && {
                        let (nl, nr) = nested_float_bottoms(c, 0.0, 0.0);
                        nl > 0.0 || nr > 0.0
                    }
            }));
    let mut child_float_contexts: Vec<(f32, f32)> =
        vec![(inherited_left_bottom, inherited_right_bottom); box_node.children.len()];

    // R1316/R1317：以下流追踪变量在子循环（has_active_float_context 块）内维护，
    // 并被后面的容器高度 containment 计算（§8.3.1）读取，故声明在函数级。
    let mut flow_bottom = 0.0f32; // 上一个非 float 流内元素的 border-bottom（content-relative）
    let mut last_flow_mb = 0.0f32; // 上一个非 float 流内元素的 margin-bottom（折叠链）
    // R5215（CSS2 §9.5.1）：float 钳制下界单独跟踪，仅 block-level 流内子推进——
    // 原子 inline-level 子（匿名块盒）推进 flow_bottom（后继流内子按 §9.2.1.1 叠于
    // 其下）但不推进本值：floats-placement-006/007/008（[inline-block, float-right,
    // float-left]）chromium 实证 float 外顶与前置匿名块盒**顶边**齐平（§9.5.1 规则 4
    // 「may not be higher than the outer top of any block … earlier」），不叠其底——
    // 旧 flow_bottom 全量推进会把 float 推到匿名块盒之下（50→150 露红）。
    let mut float_floor_bottom = 0.0f32;
    let mut float_floor_last_mb = 0.0f32;
    // R1316：一旦本容器内任一 in-flow 子被应用了正 clearance，taffy 的布局
    // 即与正确流位置发散（taffy 不建模 clearance）。此后所有非 clear 块级
    // 兄弟须以 flow_bottom 为权威基准重定位，而非沿用 taffy 的（错误）位置。
    let mut had_clearance = false;
    // R1318 §8.3.1 containment：是否有**空块**（collapse-through）被应用了正 clearance。
    // containment 公式（flow_bottom + chain − consumed mt）仅对 empty cleared block
    // 验证正确（margin-collapse-clear-012/013、margin-collapse-033/034/035 谱系）；
    // 非空 cleared（如 replaced element）的 containment 几何不同，本轮不处理（避回归）。
    let mut had_empty_clearance = false;
    // R1317 §8.3.1 containment：被 clearance「消耗」的 cleared 元素 margin-top
    // 累积（每个正 clearance 子贡献其 margin_top）。计算 contained parent height
    // 时须从 trailing 折叠链中扣除，避免 clearance-absorbed margin 双计。
    let mut clearance_consumed_mt = 0.0f32;
    // R3808：容器 collapsed-mt 泄漏撤销只执行一次的标记（首个命中正 clearance 的
    // 流内块子撤完即止，防多子连环误撤）。
    let mut parent_mt_reverted = false;

    // 收集本容器自身的 float 子元素几何（border-box 帧），用于 BFC 排斥 + 嵌套透传。
    // 注意：c.y 已含 margin_top（Phase 1 定位），故 float_h 只需 height + margin_bottom。
    // R5160：同步记录各 float 的子索引——BFC 排斥须只消费**文档顺序在前**的 float
    //（CSS 2.1 §9.5：float 顶边不得高于其假设流位，后序 float 的正确位恒在前序块
    // 之下，对更早的块级 BFC 不构成避让约束；Phase 1 的临时 y 可早于真实流位，
    // 据它对前序块做排斥会把后序 float 误当现役约束——contain-size-block-002
    // block2 被后置 float 群缩进 x=68/w=716 实证）。kill-switch
    // `ZW_BFC_FLOAT_ORDER_GATE=0` 回退无序消费。
    let float_order_gate = std::env::var("ZW_BFC_FLOAT_ORDER_GATE").as_deref() != Ok("0");
    let mut float_child_indices: Vec<usize> = Vec::new();
    let mut float_geometries: Vec<FloatGeom> = box_node
        .children
        .iter()
        .enumerate()
        .filter(|(_, c)| !matches!(c.float, FloatValue::None))
        .map(|(i, c)| {
            float_child_indices.push(i);
            (
                c.float.clone(),
                c.x,
                c.y,
                c.width,
                c.height + c.margin_bottom,
                c.margin_right,
                c.margin_left,
            )
        })
        .collect();
    // R1619 Slice 2：祖先 float（已在本容器 border-box 帧）供 BFC 排斥段（块内）
    // 与递归透传（块外）使用——使嵌套在非 BFC 后代内的 BFC 能避开祖先 float。
    // env ZW_NESTED_BFC_FLOAT_AVOID=0 关闭（kill-switch，default-on）。
    // （R5307 起透传改在循环内按文档序过滤合并，见 child_inherited。）
    let nested_avoid_on = std::env::var("ZW_NESTED_BFC_FLOAT_AVOID").as_deref() != Ok("0");
    // R5217 margin-aware 钉位总开关，循环外取一次（热路径逐对 env 查询有可测开销）。
    let margin_solve_on = std::env::var("ZW_BFC_MARGIN_SOLVE").as_deref() != Ok("0");
    // R5221：下绕 BFC 的流游标修正（chromium 交错布局语义，007 探针定谳）。Phase 1
    // 用 BFC 避让**前**的流位放置后继 float 与兄弟预位；auto 宽 BFC 在 Phase 2 下绕
    // 到 float 底后其真实底边下移，后继 float 的流位置 ≥ 该底（007：float2 从
    // 50 抬到 100）、后续 BFC 兄弟预位同步抬底才能读到抬升后的 float 带进入正确
    // 避让臂（bfc2 预位 50→100 后按 float2 新带 100..150 下推 150）。记录
    //（生效子索引, 新底）：float 子在循环顶应用；BFC 子在避让块前应用（定位臂
    // 可能覆写）；float_geometries 快照就地抬升供后续子排斥消费。
    let mut wrap_floors: Vec<(usize, f32)> = Vec::new();

    if has_active_float_context {
        let mut float_y_offset = 0.0f32;
        // 追踪正常流内容的位置，用于 clearance 假设位置计算
        let mut active_left_float_bottom = inherited_left_bottom;
        let mut active_right_float_bottom = inherited_right_bottom;
        // R1393：追踪来自「非 BFC 后代嵌套浮动」的底边（区别于直接 float 子），用于
        // adjoining-float 吸收判定——clear 清除的是嵌套浮动时，其 margin 经 wrapper 与该
        // 浮动 adjoining（§8.3.1+§9.5.2），须吸收而非正常折叠。
        let mut nested_left_bottom: f32 = 0.0;
        let mut nested_right_bottom: f32 = 0.0;

        // float_geometries 已在 if 块外（函数作用域）定义，使 BFC 排斥段（块内）
        // 与递归透传（块外）都能访问。

        // R1369：预计算每个子是否有「后续 in-flow block 同胞」（definite-width BFC 推到 float
        // 下方时，若有后续 block 同胞会留空隙/错位 → 仅无后续同胞时才安全推下）。
        let n_children = box_node.children.len();
        let has_following_block_sibling: Vec<bool> = (0..n_children)
            .map(|i| {
                box_node.children[i + 1..]
                    .iter()
                    .any(|s| s.is_block_level && !s.is_absolute && !s.is_fixed && matches!(s.float, FloatValue::None))
            })
            .collect();
        // R5210：各子是否为首个流内子（前置同胞全为浮动/定位）。taffy 仅对首流内子
        // 执行「父↔子顶边距折叠」，链值字段语义只在此成立——非首子的字段 mt 另有
        // 兄弟折叠语义，不参与链重解析。
        let is_first_in_flow: Vec<bool> = {
            let mut seen_in_flow = false;
            box_node
                .children
                .iter()
                .map(|c| {
                    let in_flow = !c.is_absolute && !c.is_fixed && matches!(c.float, FloatValue::None);
                    let first = !seen_in_flow && in_flow;
                    seen_in_flow |= in_flow;
                    first
                })
                .collect()
        };

        for (idx, child) in box_node.children.iter_mut().enumerate() {
            child_float_contexts[idx] = (active_left_float_bottom, active_right_float_bottom);
            if child.is_absolute || child.is_fixed {
                continue;
            }

            if !matches!(child.float, FloatValue::None) {
                // R5221：前序下绕 BFC 的流游标底——float 的流位置 ≥ 该底（007：
                // float2 预位 50 抬到 bfc1 新底 100；§9.5.1 float 顶不低于假设流位）。
                let cursor_floor = wrap_floors
                    .iter()
                    .filter(|(from, _)| *from <= idx)
                    .map(|(_, b)| *b)
                    .fold(f32::NEG_INFINITY, f32::max);
                if cursor_floor.is_finite() && child.y < cursor_floor {
                    child.y = cursor_floor;
                }
                // float 元素：将其 taffy 高度加入 offset
                // CSS 2.1：零高度浮动元素不占据垂直空间

                // CSS 2.1 §9.5.1：float 元素不应高于正常流内容的位置。
                // Phase 1 定位 float 时不知道 normal flow 的位置，
                // 这里修正：将 float 的 Y 推到至少与当前流位置齐平。
                // 注意：flow_bottom 与 child.y 均按 content-relative 读写的分支内约定
                // （R4235：引擎提取层 extract_layout 已把子节点 y 换算为**内容盒相对**，
                // 旧实现此处按 border-relative 再减 content_y_offset → 假设流位虚减
                // border+padding，首个 float 恒触发钳制并写回 +content_y_offset（border
                // 双计）→ c414-flt-fit-001 有 border 容器内 float p 整体 +5px，与无 float
                // 的 ref 页不对称暴露）。kill-switch `ZW_FLOAT_CLAMP_CONTENT_REL=0` 回退。
                // R4176（§9.5.1 + chromium 实证 negative-block-margin-pushing-float-
                // out-of-block-formatting-context）：钳制基准是 float 的**假设流位置**
                //（Phase 1 的 line_y，即 margin 应用前的 border-box 目标位），非 margin
                // 应用后的实际位——负 margin-top 是 float 相对流位的**合法偏移**
                //（chromium：BFC 内唯一 float `mt:-100` 渲染于流位上移 100，不被钳回）。
                // 旧实现拿 mt 应用后的 y 与 flow_bottom 比较，负 mt 恒触发钳制
                //（y 被拉回 flow_bottom，负 margin 整体失效）。
                let child_hypothetical_y = if clamp_content_rel {
                    child.y - child.margin_top
                } else {
                    child.y - content_y_offset - child.margin_top
                };
                // R5077（CSS 2.1 §9.5.1 + §8.3.1）：float 的 margin 不折叠，其 margin-box
                // 顶边不得高于前一流内兄弟的**底边距边缘** = flow_bottom + last_flow_mb。
                // 旧 floor 只用 flow_border 底——正 clearance 后 flow_bottom 虽已权威
                //（R1316），但缺 last_flow_mb：float 被放在前驱 border 底而非 margin 底
                //（clear-on-replaced-element：canvas2 clear mt:10 mb:10 后，canvas3 float
                // mt:10 margin-box 顶 30 应 40、border 顶 40 应 50，后续 clear 链整体 -10）。
                // Phase 1 的 min_float_y 公式同此（flow_bottom + last_flow_mb），但其
                // flow_bottom 是 clearance 前假设值无从修正——本钳是权威修正点。负 mb
                // 不抬 floor（.max(0) 严守旧下界；负 margin-top 合法偏移归 R4176 域）。
                // 无正 clearance 时 Phase 1 已按同式定位 → 本钳 no-op，爆炸半径限于
                // clearance 后浮点链。kill-switch `ZW_FLOAT_PREV_MB_FLOOR=0` 回退。
                let float_floor = if std::env::var("ZW_FLOAT_PREV_MB_FLOOR").as_deref() != Ok("0") {
                    // R5215：floor 用 block-only 跟踪值（float_floor_bottom/floor_last_mb），
                    // 原子 inline-level 子的推进不入 floor（§9.5.1 规则 4 顶边语义，
                    // floats-placement-006/007/008）。
                    float_floor_bottom + float_floor_last_mb.max(0.0)
                } else {
                    float_floor_bottom
                };
                if child_hypothetical_y < float_floor {
                    child.y = if clamp_content_rel {
                        float_floor + child.margin_top
                    } else {
                        content_y_offset + float_floor + child.margin_top
                    };
                    // R1832：修复 float/clear Y-staircase。旧代码在此 `active_*_float_bottom
                    // += shift`，但 active_*_float_bottom 追踪「前序 float 底边」非本 float
                    // 原始位置——把 shift（本 float 被下推量）加到前序底边 = 双重计数：
                    // side-by-side 同行 float 共享 flow_bottom，第二个 float 的 shift 把
                    // active_L 再抬一个行高（probe 实测指数增长 52→99→146→287→428→851），
                    // clear:both 后 flow_bottom 跳到膨胀 active_L = Y-staircase。本 float
                    // 真实底边已由下方 `.max(child_bottom)` 正确更新。kill-switch
                    // `ZW_FLOAT_YSTAIRCASE_FIX=0` 恢复旧行为。
                    if std::env::var("ZW_FLOAT_YSTAIRCASE_FIX").as_deref() == Ok("0") {
                        let shift = float_floor - child_hypothetical_y;
                        match child.float {
                            FloatValue::Left => active_left_float_bottom += shift,
                            FloatValue::Right => active_right_float_bottom += shift,
                            _ => {}
                        }
                    }
                }

                let float_total_height = child.margin_top + child.height + child.margin_bottom;
                float_y_offset += float_total_height;
                // R4235：child_bottom 同分支内约定（content-rel 输入，不再减 offset）。
                let child_bottom = if clamp_content_rel {
                    child.y + child.height + child.margin_bottom
                } else {
                    child.y - content_y_offset + child.height + child.margin_bottom
                };
                match child.float {
                    FloatValue::Left => active_left_float_bottom = active_left_float_bottom.max(child_bottom),
                    FloatValue::Right => active_right_float_bottom = active_right_float_bottom.max(child_bottom),
                    _ => {}
                }
                continue;
            }

            // 保存 taffy 的原始 Y（调整前）
            let original_taffy_y = child.y;

            // CSS 规范：clear 属性仅适用于块级元素（CSS 2.1 §13.5）
            // R5028：cleared br 盒豁免本 early-continue——br 自成块级载体盒（ZW 树：
            // w=全宽 h=strut）须入 §9.5.2 clearance 路径（配合 R1393 门扩认领 +
            // R5027 位移后零高度折叠）。env 同 ZW_BR_CLEAR_ELSE_PATH 门。
            let is_cleared_br_box = br_clear_gate
                && !matches!(child.clear, ClearValue::None)
                && child
                    .node_id
                    .is_some_and(|id| {
                        doc.get(id).is_some_and(|n| {
                            matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br"))
                        })
                    });
            // R5215（CSS2 §9.2.1.1）：原子 inline-level 子（inline-block 族 / replaced）
            // 是块容器的流参与者——「When a block container box contains both block-level
            // and inline-level boxes, anonymous block boxes are generated around the
            // consecutive runs of inline-level boxes」，匿名块盒按正常流占位。taffy
            // native float 已把此类子排在真实流位（floats-wrap-top-below-002l-ref：
            // span 0/50/150 基线正确），本阶段 flow 重定位必须把其高度计入
            // flow_bottom，否则后继 block 兄弟被重定位到原子盒之上（ref 页 div
            // 50→0、span2 150→100）。此类子落入下方主路径（ClearValue::None 臂的
            // correct_y 重定位 + flow 推进，clear 只认 None——clearance 不适用于
            // inline-level，CSS 2.1 §13.5/clear-applies-to）；纯 inline / 文本叶维持
            // 旧口径（R1277 ②：§9.5.1 float 顶边不受 inline 内容约束；其行盒几何由
            // IFC 面自持）。kill-switch `ZW_R5215_ATOMIC_FLOW=0` 回退。
            let child_display_inline_atomic = child.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
                matches!(
                    s.display,
                    DisplayValue::InlineBlock
                        | DisplayValue::InlineTable
                        | DisplayValue::InlineFlex
                        | DisplayValue::InlineGrid
                )
            }) || child.is_replaced;
            let is_atomic_inline_flow_child = *R5215_ATOMIC_FLOW
                && !child.is_block_level
                && matches!(child.float, FloatValue::None)
                && matches!(child.clear, ClearValue::None)
                && child_display_inline_atomic;
            if !child.is_block_level && !is_atomic_inline_flow_child && !is_cleared_br_box {
                // 非块级元素（如 inline）：扣除 float offset，不处理 clear
                if float_y_offset > 0.0 {
                    child.y -= float_y_offset;
                }
                // R1277 ②：inline 级子不推进 flow_bottom（CSS §9.5.1 仅 block-level
                // 前置内容约束 float 顶边）。否则 lifted float 后的 inline 内容会把
                // 后续 float 再度推下。
                if !lift_inline {
                    flow_bottom = flow_bottom.max(if clamp_content_rel {
                        child.y + child.height
                    } else {
                        child.y - content_y_offset + child.height
                    });
                }
                continue;
            }

            // R5210（CSS2 §8.3.1 × §9.5.2）：float 上下文重定位的链值 mt 归位。
            // taffy 把「父↔首流内子」可折叠对的整条下行链值折进链顶盒字段
            // margin_top（内层成员 y 不再含各自 mt；无 float 时提取层保序正确）。
            // 本重定位若直接用字段 mt，会在内层成员本级重放已上折的 margin
            //（逐级堆叠泄漏：clear-on-child-with-margins-2 的 B1/B2/CL 三层各
            // +16px、容器 132 应 100），且祖先帧整体下移令递归透传的 float 底边
            // 在嵌套帧内失真——clearance 假设位对比失准，正 clearance 判成负。
            // 击发门（五重，未击发零行为变化）：kill-switch 开、本子无 clear、
            // 本子为首个流内子、下行链存在 clear 阻断点（chain_clear_break_fold）、
            // 且字段 mt == collapse(重解析值, 阻断点字段 mt)（防 taffy 未折叠的
            // 树形误归位）。归位动作：字段 mt 置重解析值（R3808 撤销语义同款，
            // 撤销后 R3808 两臂首条件自然不再击发，防双计）+ extent 缩减记录
            //（R5109 祖先传播同款）。clear 阻断点的两臂语义（§9.5.2 C1/C2）：
            // 零/负 clearance 时其 mt 留本级与折叠世界几何重合（中间链 mt 和为
            // 零），正 clearance 时即本臂目标——mt 参与假设位/不折叠位对比。
            // kill-switch `ZW_CLEAR_CHAIN_MT_REFOLD=0` 关闭（default-on）。
            let child_placement_mt = if *CLEAR_CHAIN_MT_REFOLD
                && matches!(
                    child.clear,
                    ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
                )
                && is_first_in_flow[idx]
            {
                match chain_clear_break_fold(child) {
                    Some((folded, break_mt))
                        if (crate::margin_collapse::collapse_two_margins(folded, break_mt) - child.margin_top)
                            .abs()
                            < 0.01 =>
                    {
                        let over = child.margin_top - folded;
                        if over.abs() > 0.01 {
                            child.margin_top = folded;
                            child.clearance_extent_reduction += over;
                        }
                        folded
                    }
                    _ => child.margin_top,
                }
            } else {
                child.margin_top
            };

            // CSS 2.1 §9.5.2 Clearance 计算
            // 假设位置（hypothetical position）：无 clear 时元素应在的位置，
            // 基于正常流的 flow_bottom + margin 折叠计算。
            //
            // CSS 2.1 §9.5.2 clearance 算法：
            // 1. 计算 hypothetical position（假设 clear:none，含 margin 折叠）
            // 2. 计算 clearance = max(0, clear_bottom - hypothetical_position)
            // 3. 如果 clearance > 0：元素推到 clear_bottom
            // 4. 如果 clearance == 0：clear 仍阻止 margin 折叠
            //    元素放在 flow_bottom + margin_top（不折叠）
            // R1316：本子是否被应用了正 clearance（破坏 collapse-through）。
            let mut clearance_applied = false;
            match child.clear {
                ClearValue::Left | ClearValue::Right | ClearValue::Both => {
                    let clear_bottom = match child.clear {
                        ClearValue::Left => active_left_float_bottom,
                        ClearValue::Right => active_right_float_bottom,
                        _ => active_left_float_bottom.max(active_right_float_bottom),
                    };
                    // 假设位置：基于正常流的 flow_bottom + margin 折叠
                    // CSS 2.1 §9.5.2：「as if 'clear' were 'none'」
                    // R5210：假设位用链值归位后的本级 mt（clear 子未击发重解析，
                    // 恒等字段值；链上祖先已归位——见 child_placement_mt 注释）。
                    let collapsed_margin =
                        crate::margin_collapse::collapse_two_margins(last_flow_mb, child_placement_mt);
                    let hypothetical_y = flow_bottom + collapsed_margin;

                    if clear_bottom > hypothetical_y {
                        // 正 clearance：margin 不折叠
                        // CSS 2.1 §9.5.2 C1/C2 双路径算法：
                        // C1（含 margin 折叠）：clearance = clear_bottom - hypothetical_y
                        // C2（不含 margin 折叠）：clearance = clear_bottom - (flow_bottom + child.margin_top)
                        // 最终位置 = max(clear_bottom, flow_bottom + child.margin_top)
                        // 当元素自身 margin-top 足够大时，即使不折叠也已在浮动之下
                        let uncollapsed_pos = flow_bottom + child_placement_mt;
                        // R4235：content-rel 模式（同 line_y/钳制分支）——写点去 offset。
                        child.y = if clamp_content_rel {
                            clear_bottom.max(uncollapsed_pos)
                        } else {
                            content_y_offset + clear_bottom.max(uncollapsed_pos)
                        };
                        // R3808：正 clearance 吸收了该子的 margin-top（位置 = clear_bottom，
                        // margin 不再下推自身），但 taffy 已把该 mt 折叠进容器（§8.3.1
                        // parent-child collapse——首个流内块子的 mt 上浮），容器被多推下
                        // 一段（margin-collapse-142：td 内 container 被 .clear 的 4em mt
                        // 推下 64px 露 td 红底；chromium container 顶在 cell content 顶）。
                        // 泄漏签名与上方 float-mt 恢复（§8.3.1 float mt 不折叠）同构：
                        // 容器无 border/padding-top + 容器 mt 膨胀到 == 该子 mt + 该子 mt
                        // 为声明值 + 该子是首个非 float 流内块子 → 撤销多折叠量。
                        // 只撤一次（首个命中子），后续兄弟不再触发。
                        if !parent_mt_reverted
                            && std::env::var("ZW_CLEAR_MT_LEAK_REVERT").as_deref() != Ok("0")
                            && content_y_offset == 0.0
                            && box_node.margin_top > box_node.declared_margin_top + 0.01
                            && (box_node.margin_top - child.margin_top).abs() < 0.01
                            // R5109：子 mt 允许被其自身子树折叠膨胀（max(声明, 末链)——
                            // margin-collapse-157 row2：.clear 自身 mt 声明 0 但被内部
                            // empty 自折叠 16 抬到 16）。§8.3.1「child has clearance」阻止
                            // 该组与容器 top 折叠，组应以子的自身 margin 留父内——撤销容器
                            // 膨胀量仍正确（组不消失，仅从容器 margin 槽归位）。仅拒绝
                            // 子 mt 小于声明（负向异常不属本签名）。
                            && child.margin_top >= child.declared_margin_top - 0.01
                        {
                            let over_collapse = box_node.margin_top - box_node.declared_margin_top;
                            box_node.y -= over_collapse;
                            box_node.margin_top = box_node.declared_margin_top;
                            // R5109：容器 extent（含 mt）缩减 over_collapse，记录供祖先
                            // 传播（taffy 祖先高按膨胀前的 margin-box 计）。
                            box_node.clearance_extent_reduction += over_collapse;
                            parent_mt_reverted = true;
                        }
                        // R1316 defect ①/②：正 clearance 被应用 → 该子的 margin 不再
                        // collapse-through（§8.3.1），它建立流位置，且其后所有兄弟
                        // 须以 flow_bottom 重定位（taffy 未知 clearance）。
                        clearance_applied = true;
                        // R5027（CSS 2.1 §9.4.2 零行盒 + §9.5.2）：**仅含强制换行的清除
                        // 块**（br 自成块级盒）被正 clearance 位移后高度归零——行盒被
                        // 推到 float 底时 strut 不再加高（chromium：p1(float)+br{clear}+p2
                        // 的 p2 top = float margin-box 底 89；ZW 旧 = 89 + strut 19 = 108，
                        // floats-no-content-beside-001 / floats-wrap-top-below 系连坐）。
                        // 未位移（无 float）时 strut 高保持（chromium 变体 A 实证 +19）。
                        // env ZW_BR_CLEAR_ZEROH=0 关闭（kill-switch，default-on）。
                        let is_br_box = child
                            .node_id
                            .and_then(|id| doc.get(id))
                            .is_some_and(|n| {
                                matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("br"))
                            });
                        if is_br_box
                            && child.children.is_empty()
                            && std::env::var("ZW_BR_CLEAR_ZEROH").as_deref() != Ok("0")
                        {
                            child.height = 0.0;
                            child.content_height = 0.0;
                        }
                        // R1318 §8.3.1 containment：clearance 「消耗」了空 cleared 块的 margin-top
                        //（hypothetical 用它定位，clearance 填充余下到 clear_bottom 的间隙）。
                        // 计算 contained parent height 时须从 trailing 折叠链扣除，避免双计。
                        // 仅对 empty cleared block（collapse-through）—— 非空 cleared（replaced
                        // 等）containment 几何不同，本轮不触（避 clear-on-replaced-element 回归）。
                        if crate::margin_collapse::is_empty_block(child) {
                            clearance_consumed_mt += child.margin_top;
                            had_empty_clearance = true;
                        }
                    } else if (clear_bottom - hypothetical_y).abs() < 0.001 {
                        // 零 clearance（hypothetical_y ≈ clear_bottom）：
                        // CSS 2.1 §9.5.2：clearance 引入后，位置 = hypothetical + clearance。
                        // clearance = max(clear_bottom - P, H - P)，其中 P 为不折叠边距位置。
                        // 当 H == clear_bottom 时，P + clearance = H = clear_bottom。
                        // 因此元素位置 = hypothetical_y（使用折叠边距计算的假设位置）。
                        // 零 clearance 仍阻止 margin 折叠（CSSWG resolution），
                        // 但视觉位置与假设位置相同。
                        child.y = if clamp_content_rel {
                            hypothetical_y
                        } else {
                            content_y_offset + hypothetical_y
                        };
                        // R3808：零 clearance 同样使该子 margin 不与容器折叠链合并
                        //（collapse 被 clearance 打断）——taffy 已折叠进容器的 mt 按同一
                        // 泄漏签名撤销（与正 clearance 臂同构）。
                        if !parent_mt_reverted
                            && std::env::var("ZW_CLEAR_MT_LEAK_REVERT").as_deref() != Ok("0")
                            && content_y_offset == 0.0
                            && box_node.margin_top > box_node.declared_margin_top + 0.01
                            && (box_node.margin_top - child.margin_top).abs() < 0.01
                            // R5109：同正 clearance 臂——子 mt 允许被其自身子树折叠膨胀
                            //（§8.3.1「child has clearance」语义下撤销容器膨胀仍正确）。
                            && child.margin_top >= child.declared_margin_top - 0.01
                        {
                            let over_collapse = box_node.margin_top - box_node.declared_margin_top;
                            box_node.y -= over_collapse;
                            box_node.margin_top = box_node.declared_margin_top;
                            // R5109：同正 clearance 臂——记录供祖先传播。
                            box_node.clearance_extent_reduction += over_collapse;
                            parent_mt_reverted = true;
                        }
                    } else {
                        // hypothetical_y > clear_bottom：元素（含 margin）已过浮动。
                        // R1393 adjoining-float 吸收：若 clear 清除的是「嵌套在非 BFC 后代中的
                        // 浮动」（clear_bottom 由 nested_bottom 决定），则 clear 的 margin 经
                        // 该 wrapper 与浮动 adjoining（§8.3.1+§9.5.2）——须 apply clearance 把
                        // clear 定到 clear_bottom 并吸收 margin，否则 margin 把 clear 推到
                        // hypothetical 留下红间隙（adjoining-float-before-clearance：clear mt:400
                        // 应落 float 底 50 而非 450）。须配合 R1392 nested-float 可见性。
                        // env `ZW_ADJOINING_FLOAT_CLEARANCE=0` 关闭（kill-switch，default-on）。
                        let nested_for_side = match child.clear {
                            ClearValue::Left => nested_left_bottom,
                            ClearValue::Right => nested_right_bottom,
                            _ => nested_left_bottom.max(nested_right_bottom),
                        };
                        // adjoining 信号：clear 清除的是「嵌套在非 BFC 后代中的浮动」
                        //（clear_bottom 由 nested_bottom 决定）。direct-float 变体（new-fc 谱系）
                        // 用 clear_bottom>flow_bottom 信号会过冲（4 案回归），故仅 gate 嵌套浮动。
                        let adjoining = std::env::var("ZW_ADJOINING_FLOAT_CLEARANCE").as_deref() != Ok("0")
                            && nested_for_side > 0.0
                            && (clear_bottom - nested_for_side).abs() < 0.5;
                        // R5212（CSS2 §9.5.2 forced clearance × §8.3.1）：direct-float
                        // 邻接变体。clear 盒为本帧首流内子时，其邻接 top-margin 链
                        //（塌穿链 + 自身 mt，含 R5210 折入链值）经帧顶上折、在帧上方
                        // 物化——帧内 float 连同链顶盒被整体下拉（链与 float adjoining，
                        // Chromium adjoining-float-* 测试族互操作语义：「No matter how
                        // large the margin is, it should still be just below the
                        // float」）——引入 forced clearance：float 保持原位，clear 盒
                        // border edge 贴 float 底，整条邻接链吸收。非首流内子时链与
                        // 前驱兄弟折叠（float 置于自身 DOM 点、不被后续兄弟 margin
                        // 拉动，tests_10 empty-cleared 形态），帧顶有 border/padding
                        // 时链不出帧（content_y_offset>0，同 R3808 首门），均不吸收。
                        // kill-switch `ZW_FORCED_CLEAR_ABSORB=0` 关闭（default-on）。
                        let forced_absorb = *FORCED_CLEAR_ABSORB
                            && is_first_in_flow[idx]
                            && content_y_offset == 0.0
                            && clear_bottom > flow_bottom + 0.01;
                        if adjoining || forced_absorb {
                            child.y = if clamp_content_rel {
                                clear_bottom
                            } else {
                                content_y_offset + clear_bottom
                            };
                            clearance_applied = true;
                            ran_adjoining_clearance = true;
                            // R5212：标记邻接链已吸收——首流内子的本级重放抑制门。
                            child.clearance_chain_absorbed = true;
                        } else {
                            // 普通情形：margin 正常折叠，clear 留在 hypothetical。
                            child.y = if clamp_content_rel {
                                hypothetical_y
                            } else {
                                content_y_offset + hypothetical_y
                            };
                        }
                    }
                    float_y_offset = (original_taffy_y - child.y).max(0.0);
                    // R1316 defect ①：正 clearance 使流发散，标记后续兄弟须重定位。
                    if clearance_applied {
                        had_clearance = true;
                    }
                }
                ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd => {
                    // 非 clear 的普通元素：使用独立的 flow_bottom 追踪计算正确位置
                    // 简单的 child.y -= float_y_offset 无法正确处理 margin 折叠，
                    // 因为 taffy 将 float 当作 block 排列，其 margin 折叠方式
                    // 与 float 不存在时的折叠方式不同。
                    //
                    // R1316 defect ①：除残留 float 空间（float_y_offset>0）外，
                    // 若本容器此前出现过正 clearance（had_clearance），taffy 对本
                    // 子的定位同样不可信 —— 须以 flow_bottom 重定位。
                    if float_y_offset > 0.0 || had_clearance {
                        // R5212：吸收链首子重放抑制。box_node（clear 盒）被 forced/
                        // nested clearance 吸收邻接链落 float 底后，首流内子的字段 mt
                        // 已随整链折入 box_node 的吸收 margin（§8.3.1 parent-child
                        // 折叠），本级重放须归零——否则在 B 内以折叠链值重放（
                        // adjoining-float-nested-forced-clearance：G 在 B 内重放 100，
                        // 盒树双重下移）。膨胀签名（字段 mt > 声明 mt）同 R3808。
                        let replay_mt = if box_node.clearance_chain_absorbed
                            && is_first_in_flow[idx]
                            && box_node.margin_top > box_node.declared_margin_top + 0.01
                        {
                            0.0
                        } else if is_first_in_flow[idx]
                            && content_y_offset == 0.0
                            && box_node.margin_top > box_node.declared_margin_top + 0.01
                            && *FLOAT_REPLAY_FIRST_MT
                        {
                            // R5228：首流内子的 mt 已被父容器吸收折叠（§8.3.1；膨胀签名
                            // 同 R5212 门：box_node.margin_top > declared——engine 折叠
                            // pass 把 max(parent_mt, 链顶 mt) 折进容器 margin，子盒应位于
                            // content 原点）。taffy 把前序 float 当 block 排列致
                            // float_y_offset>0 触发本重放时，再以字段 mt 重放 = 双计
                            //（border-right-width-095-ref [float,p,float]：p +16、
                            // flow_bottom 快照污染 → 后继 float floor 36→52、square
                            // 52→68）。签名不成立时不归零：BFC/有 padding-top 容器的链顶
                            // 值 16 合法留本级（R5210 既有语义，不重解析）。
                            // kill-switch `ZW_FLOAT_REPLAY_FIRST_MT=0` 回退。
                            0.0
                        } else {
                            child_placement_mt
                        };
                        // CSS 2.1：非 clear 元素的位置 = 正常流位置（假设 float 不存在）
                        // R5210：mt 用链值归位后的本级值（已上折的首子链 margin 不在
                        // 本级重放——旧字段值使 B1/B2/CL 逐级堆叠 +16px）。
                        let collapsed_margin = crate::margin_collapse::collapse_two_margins(last_flow_mb, replay_mt);
                        let correct_y = flow_bottom + collapsed_margin;
                        child.y = if clamp_content_rel {
                            correct_y
                        } else {
                            content_y_offset + correct_y
                        };
                        // 更新 float_y_offset 以反映 taffy Y 与正确 Y 的差异
                        float_y_offset = (original_taffy_y - child.y).max(0.0);
                    }
                }
            }

            // 空块自身的上下 margin 会自折叠，并继续传递给后继兄弟；
            // 它自身不应把 flow_bottom 往下推进一段“实心高度”。
            //
            // R1316 defect ②：但被应用了正 clearance 的元素（即便高度为 0）破坏
            // collapse-through（§8.3.1）—— 它建立流位置，flow_bottom 须推进到其
            // border-box 底边，其 margin-bottom 不再 collapse-through 传给后继。
            // 否则 cleared 空块之后的首个兄弟会用陈旧的 flow_bottom 定位，违反
            // DOM 顺序（出现在 cleared 元素之前）。
            let establishes_flow_position = !crate::margin_collapse::is_empty_block(child) || clearance_applied;
            if !establishes_flow_position {
                let collapsed_self_margin =
                    crate::margin_collapse::collapse_two_margins(child.margin_top, child.margin_bottom);
                last_flow_mb = crate::margin_collapse::collapse_two_margins(last_flow_mb, collapsed_self_margin);
            } else {
                // 更新流内容追踪（使用 content-relative 坐标）
                // R4235：content-rel 模式下 child.y 已是内容盒相对，不再减 offset。
                flow_bottom = if clamp_content_rel {
                    child.y + child.height
                } else {
                    child.y - content_y_offset + child.height
                };
                last_flow_mb = child.margin_bottom;
                // R5215：float 钳制 floor 只随 block-level 子推进（原子 inline-level
                // 子按 §9.5.1 规则 4 只以顶边约束 float，见 float_floor_bottom 声明处）。
                // R5227：R109 匿名片段盒同语义——taffy/extract 强制 is_block_level=true
                // 但内容是原子 inline-level（§9.2.1.1），float 外顶与其顶边齐平、不以其
                // 底抬 floor（between-float-and-text：floor 不入 → Phase 1 行顶锚 20
                // 不被钳到 40）。
                if child.is_block_level && child.fragment_node_ids.is_none() {
                    float_floor_bottom = flow_bottom;
                    float_floor_last_mb = last_flow_mb;
                }
            }

            // R5221：BFC 子避让前预位再抬——clearance/flow 重放臂可能覆写 loop 顶
            // floor（bfc2 预位被 FloatValue::None 臂重放回 50），避让重叠检查前回
            // 本帧权威底（仅 establishes_bfc 子，普通块流位语义未探不入；float 子
            // 已在分支 continue，不达此处）。
            if !wrap_floors.is_empty() && crate::margin_collapse::establishes_bfc(child) {
                let cursor_floor = wrap_floors
                    .iter()
                    .filter(|(from, _)| *from <= idx)
                    .map(|(_, b)| *b)
                    .fold(f32::NEG_INFINITY, f32::max);
                if cursor_floor.is_finite() && child.y < cursor_floor {
                    child.y = cursor_floor;
                }
            }

            // R4240（css-sizing-4 §6.1 + csswg #4028）：块级 replaced 的 width:stretch —
            // stretch size respects float avoidance：与垂直重叠 float 并置时，box 放
            // float 右缘、宽取剩余可用宽（replaced-next-to-float-1：canvas
            // width:stretch 应 100 宽避让，旧实现 200 宽压 float 且 h 按固有比翻倍）。
            // 不重叠（自然位已满宽）与非 replaced stretch（flow-root 窄化语义）不入本臂。
            if clamp_content_rel
                && child.declared_width_stretch
                && child.is_replaced
                && !child.is_absolute
                && !child.is_fixed
            {
                let child_top = child.y;
                let child_bottom = child.y + child.height;
                let mut x_lo = if child.margin_left_auto { 0.0 } else { child.margin_left };
                let mut overlapped = false;
                for g in &float_geometries {
                    let (fd, fx, fy, fw, fh, fmr, _fml) = g;
                    let fbottom = *fy + *fh;
                    if child_top < fbottom && child_bottom > *fy {
                        if matches!(fd, FloatValue::Left) {
                            x_lo = x_lo.max(*fx + *fw + *fmr);
                        }
                        overlapped = true;
                    }
                }
                if overlapped {
                    let avail = (container_width - x_lo).max(0.0);
                    if (child.x - x_lo).abs() > 0.5 || (child.width - avail).abs() > 0.5 {
                        let old_w = child.width;
                        child.x = x_lo;
                        child.width = avail;
                        // R4240：auto 高 replaced 的高由固有比随宽解析（taffy 期按旧宽
                        // 200 算出 h=200）——宽缩后高按同比例重算（1:1 attr canvas
                        // 200/200 → 100/100）。
                        if child.declared_height_auto && old_w > 0.5 {
                            child.height *= avail / old_w;
                        }
                        shrink_bfc_content_width(child);
                    }
                }
                continue;
            }

            // BFC 浮动排斥（CSS 2.1 §9.5）：
            // 建立 BFC 的块级元素不得与同容器的浮动元素重叠。
            // 当 BFC 元素的垂直范围与浮动元素重叠时，水平偏移以避开浮动。
            if child.is_block_level
                && !child.is_absolute
                && !child.is_fixed
                && matches!(child.float, FloatValue::None)
                && crate::margin_collapse::establishes_bfc(child)
            {
                let child_top = child.y;
                let child_bottom = child.y + child.height;

                // R1730 Slice 5（RFC §10.2）：多-float BFC 协调。当 BFC 子同时垂直重叠 ≥2 个
                // 同容器 float 时，per-float 循环（R1369/R1722/R1728）独立逐 float pushdown/squeeze
                // 会 over-push（floats-wrap-top-below-bfc-003l span2 被推到 float R 底 164 而非
                // float L 底 89）。协调：收集所有垂直重叠 float，按候选 y（自然 y ∪ 各重叠 float
                // bottom，升序）找首个使 BFC 不重叠任何 float 的 y（该 y 处所有现役 float 联合约束
                // 出可行 x 区间 [x_lo, x_hi]，BFC 宽能放下即可行），取 x_lo（尊 BFC 既有 margin_left
                // 左对齐）；找不到可行 y 则下到最晚 float bottom。margin-left:auto 的右对齐特化须
                // margin_auto 字段（LayoutBox 暂无），001r 此版落 x=margin_left（与现状一致非回归）。
                // kill-switch ZW_BFC_MULTIFLOAT_COORD=0（default-on）。scope gate：≥2 垂直重叠 float
                // + 无后续 in-flow block 同胞——单 float 走既有 per-float 循环，零回归基线。
                let coord_on = std::env::var("ZW_BFC_MULTIFLOAT_COORD").as_deref() != Ok("0");
                let mut coord_handled = false;
                if coord_on
                    && !has_following_block_sibling[idx]
                    && !child.declared_width_auto
                    && !child.is_layout_container
                {
                    let overlapping: Vec<&FloatGeom> = float_geometries
                        .iter()
                        .enumerate()
                        .filter(|(j, g)| {
                            // R5160：仅文档顺序在前的 float 参与排斥（见 float_geometries 注）。
                            if float_order_gate && float_child_indices[*j] >= idx {
                                return false;
                            }
                            let (_, _, fy, _, fh, _, _) = g;
                            let fbottom = *fy + *fh;
                            child_top < fbottom && child_bottom > *fy
                        })
                        .map(|(_, g)| g)
                        .collect();
                    if overlapping.len() >= 2 {
                        let w = child.declared_width_px.unwrap_or(child.width);
                        let h = child.height;
                        // 候选 y：自然 y + 各重叠 float bottom（> 自然 y），升序去重。
                        let mut y_candidates: Vec<f32> = vec![child.y];
                        for g in &overlapping {
                            let (_, _, fy, _, fh, _, _) = g;
                            let fb = *fy + *fh;
                            if fb > child.y + 0.5 {
                                y_candidates.push(fb);
                            }
                        }
                        y_candidates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        y_candidates.dedup_by(|a, b| (*a - *b).abs() < 0.5);
                        let mut placed: Option<(f32, f32)> = None;
                        for &cand_y in &y_candidates {
                            // 该 y 处可行 x 区间：左 float 推 x_lo 右移，右 float 推 x_hi 左移。
                            // margin-left:auto 时 margin_left 是 taffy 已解析的 auto 值（大），
                            // 作左界会误判不可行——auto margin 的 BFC 可放 [0, x_hi] 任意处（右对齐 x_hi），
                            // 故 auto 时 x_lo 从 0 起；非 auto 时尊固定 margin_left。
                            let mut x_lo = if child.margin_left_auto { 0.0 } else { child.margin_left };
                            let mut x_hi = (container_width - w).max(child.margin_left);
                            for g in &overlapping {
                                let (fd, fx, fy, fwidth, fh, fmargin_r, _fmargin_l) = g;
                                let fbottom = *fy + *fh;
                                if !(cand_y < fbottom && cand_y + h > *fy) {
                                    continue;
                                }
                                match fd {
                                    FloatValue::Left => {
                                        // BFC 须在 float margin-box 右侧：x >= fx + width + margin_r。
                                        x_lo = x_lo.max(fx + fwidth + fmargin_r);
                                    }
                                    FloatValue::Right => {
                                        // BFC 须在 float 左侧：x + w <= fx（float 左 margin 未存，用 border-box 左）。
                                        x_hi = x_hi.min(fx - w);
                                    }
                                    _ => {}
                                }
                            }
                            if x_lo <= x_hi + 0.5 {
                                // margin-left:auto → 右对齐到最左 obstructing float 左缘（x_hi）；
                                // 否则左对齐 x_lo（floats-wrap-top-below-bfc-001r span2 margin-auto）。
                                let chosen_x = if child.margin_left_auto { x_hi } else { x_lo };
                                placed = Some((chosen_x, cand_y));
                                break;
                            }
                        }
                        if let Some((px, py)) = placed {
                            child.x = px;
                            child.y = py;
                            if let Some(width) = child.declared_width_px {
                                child.width = width;
                                shrink_bfc_content_width(child);
                            }
                        } else {
                            // 无可行 y（BFC 宽放不下任何候选处）→ 下到最晚重叠 float bottom。
                            let max_bottom = overlapping
                                .iter()
                                .map(|g| {
                                    let (_, _, fy, _, fh, _, _) = g;
                                    *fy + *fh
                                })
                                .fold(child.y, f32::max);
                            child.y = max_bottom;
                            child.x = child.margin_left;
                            if let Some(width) = child.declared_width_px {
                                child.width = width;
                                shrink_bfc_content_width(child);
                            }
                        }
                        coord_handled = true;
                    }
                }

                // R5221：下绕触发的快照抬升——per-float 循环借用 float_geometries，
                // 延后到循环后应用（后继 float 带抬到下绕 BFC 新底）。
                let mut pending_geom_bump: Option<f32> = None;
                if !coord_handled {
                    for (gi, (float_dir, float_x, float_y, float_border_w, float_h, float_margin_r, float_margin_l)) in
                        float_geometries.iter().enumerate()
                    {
                        // R5160：仅文档顺序在前的 float 参与排斥（见 float_geometries 注）。
                        if float_order_gate && float_child_indices[gi] >= idx {
                            continue;
                        }
                        let float_top = *float_y;
                        let float_bottom = *float_y + *float_h;

                        // 检查垂直重叠
                        if !(child_top < float_bottom && child_bottom > float_top) {
                            continue;
                        }
                        match float_dir {
                            FloatValue::Left => {
                                // 左浮动：将 BFC 元素推到浮动元素的 margin-box 右侧
                                // float_x 是边框盒左边，加上边框宽度和右 margin
                                let avoidance_x = float_x + float_border_w + float_margin_r;
                                // R3807：零位移左 float（avoidance_x ≤ 0）不构成 BFC 回避约束
                                //（CSS2 §9.5 chromium 实证：0 宽左 float 在容器左缘时
                                // float offset 与无 float 同值 → BFC 走普通 §10.3.3 流内求解，
                                // 负 margin 求解语义保留——zero-width-floats.html：float 0 宽
                                // 时 BFC 仍应 x=ml=-50、w=cb−ml−mr=200，而非被钉在 band）。
                                if avoidance_x <= 0.0 {
                                    continue;
                                }
                                let declared_or_layout_width = child.declared_width_px.unwrap_or(child.width);
                                // R1369：definite-width BFC（width 未填满容器）若 overflow 容器
                                //（child.x + width > container_width），应推到 float 下方（CSS §9.5：
                                // BFC border-box 不重叠 float；definite 宽度保持不 shrink）。
                                // **关键**：taffy 0.12 native float 可能已把 BFC 推到 float 右
                                //（child.x == avoidance_x），故本检查须在 `avoidance_x > child.x`
                                // 之外做（否则 taffy 已推时整块 skip）。auto-width BFC（填满容器）
                                // 仍走 shrink-to-fit（else 分支）。仅无后续 in-flow block 同胞时推下。
                                // R1728：补充「放不下 float 旁」判定。原 R1369 gate 仅查「溢出容器」，
                                // 漏「float 占满宽致其右可用宽 < BFC 声明宽」——此时 BFC 同样无法旁置，
                                // 须推到 float 下方（CSS §9.5：BFC border-box 不重叠 float；definite
                                // 宽度不 shrink）。floats-wrap-top-below-bfc-002r span2：float:left 300
                                // 宽，span2（overflow:hidden）声明宽 200 放不下其右 [300,400]=100 → 应
                                // pushdown 到 float 下方（原行为是 squeeze 到 x=300/w=100，错）。
                                // **关键**：仅对「声明宽（非 auto）」BFC 触发——auto 宽 BFC（如
                                // floats-bfc-003 的 #bfc、new-fc-beside-float）须 shrink-to-fit 旁置
                                //（spec：BFC 占 float 旁可用宽），用 declared_width_auto 区分（width
                                // 已 shrink 的 auto BFC 其 child.width < container_width 但非「definite」）。
                                // kill-switch ZW_BFC_LEFT_FIT_PUSHBELOW=0 回退纯溢出 gate。
                                let overflows = child.x + declared_or_layout_width > container_width + 0.5;
                                let is_definite_width =
                                    child.declared_width_px.is_some() || child.width < container_width - 0.5;
                                let avail_beside = (container_width - avoidance_x).max(0.0);
                                let fits_beside = declared_or_layout_width <= avail_beside + 0.5;
                                let left_fit_pushbelow =
                                    std::env::var("ZW_BFC_LEFT_FIT_PUSHBELOW").as_deref() != Ok("0");
                                let must_pushdown = overflows
                                    || (left_fit_pushbelow
                                        && is_definite_width
                                        && !child.declared_width_auto
                                        && !fits_beside);
                                // R5217（CSS2 §9.5 规则 2 × §10.3.3）：width:auto BFC 且水平
                                // margin 非零时 margin-aware 钉位解——flow 盒 [ml, CB − mr] 被
                                // band [avoidance_x, CB] 钉位：面向侧正 margin 被吃掉
                                //（lf ml=5 → x=avoidance w=gap）、背侧 margin 收窄、frame 下限
                                // 防塌缩；margin_h < −(gap+0.5) 下绕 float 底全 CB 重解。
                                // kill-switch `ZW_BFC_MARGIN_SOLVE=0` 回退旧「填满 gap」臂。
                                let ml = if child.margin_left_auto { 0.0 } else { child.margin_left };
                                let mr = if child.margin_right_auto {
                                    0.0
                                } else {
                                    child.margin_right
                                };
                                let margin_h = ml + mr;
                                let frame =
                                    child.border_left + child.border_right + child.padding_left + child.padding_right;
                                let margin_aware = margin_solve_on && child.declared_width_auto && margin_h.abs() > 0.5;
                                // R5221：sibling 门移除——007 chromium 探针定谳多 float 纵向堆叠
                                // 语义：auto 宽 BFC 的 beside fit 用 margin box（w−|负 margin|）
                                // 对 beside 带判定，装不下即下绕（floats-wrap-bfc-with-margin-007：
                                // bfc1 w=CB−mr=100、margin box 50 > 带 25 → 下绕 y=float 底
                                // x=0 w=100；float2/bfc2 顺序下移一行），与本钉位解 wrap 分支
                                //（margin_h < −(gap+0.5)）一致。旧臂（taffy 用 beside 带解
                                // §10.3.3 → w=75、x=float 右缘）偏离 chromium。
                                // 右 float 镜像门未同步开（无 rf 侧多 float 探针，挂账）。
                                // 入门：margin-aware 即行动——钉位解与 §10.3.3 flow 盒在无
                                // 钳制时相等（幂等覆写），而 ml ≥ avoidance 时 taffy 自身
                                // float 收缩可能已把宽塌到 0（lf ml=85 探针：taffy w=0，
                                // chromium 15），必须由钉位解覆写回 flow 几何。
                                let margin_enter = margin_aware;
                                if margin_enter {
                                    let (beside_x, beside_w, wrap) = solve_auto_bfc_margin_beside(
                                        true,
                                        ml,
                                        mr,
                                        frame,
                                        avoidance_x,
                                        container_width,
                                        container_width,
                                    );
                                    if wrap {
                                        // 下绕：float 约束解除 → 全 CB 重解宽、x 回流位。
                                        if float_bottom > child.y {
                                            child.y = float_bottom;
                                        }
                                        child.x = ml;
                                        child.width = (container_width - margin_h).max(0.0);
                                        shrink_bfc_content_width(child);
                                        // R5221：下绕后本盒真实底边下移——记流游标底，
                                        // 后续兄弟（float 带与 BFC 预位）在各自处理点抬升。
                                        let new_bottom = child.y + child.height;
                                        wrap_floors.push((idx + 1, new_bottom));
                                        pending_geom_bump = Some(new_bottom);
                                    } else {
                                        // 旁置钉位（含小负 margin 落 x=avoidance、w=gap）。
                                        child.x = beside_x;
                                        child.width = beside_w;
                                        shrink_bfc_content_width(child);
                                    }
                                } else if is_definite_width && !has_following_block_sibling[idx] && must_pushdown {
                                    if float_bottom > child.y {
                                        child.y = float_bottom;
                                    }
                                    if let Some(width) = child.declared_width_px {
                                        child.width = width;
                                        shrink_bfc_content_width(child);
                                    }
                                    // 回正常流位置（taffy float push 前）：block border-box 左 =
                                    // 父 content-box 左 + margin_left（child.x 相对父 content-box）。
                                    child.x = child.margin_left;
                                } else if avoidance_x > child.x {
                                    // taffy 未推 → ZW 推到 float 右 + shrink-to-fit（原行为）
                                    child.x = avoidance_x;
                                    // 缩小宽度以不超出容器
                                    let max_width = container_width - child.x;
                                    if child.width > max_width {
                                        child.width = max_width.max(0.0);
                                        shrink_bfc_content_width(child);
                                    }
                                }
                            }
                            FloatValue::Right => {
                                let declared_or_layout_width = child.declared_width_px.unwrap_or(child.width);
                                // R1722：float:right definite-width BFC 放不下 float 左侧可用宽
                                //（child.x + width > float_x）→ 推到 float 下方（mirror of R1369 左
                                // float overflows 推下，CSS §9.5：BFC border-box 不重叠 float；definite
                                // 宽度保持不 shrink）。仅 definite-width + 无后续 in-flow block 同胞时推下，
                                // 否则保持 shrink-to-fit（原行为）。kill-switch ZW_BFC_RIGHT_PUSHBELOW=0。
                                // R5217：width:auto BFC 水平 margin 非零 → margin-aware
                                // 钉位解（镜像左 float 臂；band = [0, float margin-box 左缘]，
                                // 右缘钉位 + 钉后宽 < frame 下绕，003 rf ml=15/22/28 实证）。
                                // 门置于「右缘不达 float → continue」之前：taffy 按 §10.3.3
                                // 全 CB 求解的 border-box 右（= CB − mr）在大正 mr 时可能已
                                // 不达 float_x（001 margin-right=14：右缘 16 < float_x 18，
                                // 旧门漏触发，宽停在 CB 解 16 应 2）。kill-switch
                                // `ZW_BFC_MARGIN_SOLVE=0` 回退旧「缩到 float 左缘」臂。
                                let ml = if child.margin_left_auto { 0.0 } else { child.margin_left };
                                let mr = if child.margin_right_auto {
                                    0.0
                                } else {
                                    child.margin_right
                                };
                                let margin_h = ml + mr;
                                let frame =
                                    child.border_left + child.border_right + child.padding_left + child.padding_right;
                                let margin_aware = margin_solve_on && child.declared_width_auto && margin_h.abs() > 0.5;
                                // R5222：sibling 门移除——rf 镜像探针定谳（/tmp/r5222_007rf_probe.mjs，
                                // float:right + clear:right + ml=−50 镜像 007）：bfc1 下绕 y=50
                                // x=ml=−50 w=100、float2 y=100（流游标级联同左）、bfc2 y=150 w=75
                                // x=−50、wrapper h=200——与左 float 侧完全对称，R5217 阈值模型
                                //（margin_h < −(gap+0.5)）rf 侧同成立。
                                let is_definite_width =
                                    child.declared_width_px.is_some() || child.width < container_width - 0.5;
                                if !margin_aware && child.x + declared_or_layout_width <= *float_x {
                                    continue;
                                }
                                // R3807：零位移右 float（float_x ≥ 容器宽）不构成 BFC 回避约束
                                //（CSS2 §9.5 chromium 实证：0 宽右 float 在容器右缘时
                                // float offset 与无 float 同值 → 普通流内求解；
                                // zero-width-floats.html 的两个 0 宽 float 均零位移 → BFC 保持
                                // x=ml=-50、w=cb−ml−mr=200 的流内几何）。
                                if *float_x >= container_width - 0.5 {
                                    continue;
                                }
                                if margin_aware {
                                    let band_right = (*float_x - *float_margin_l).max(0.0);
                                    let (beside_x, beside_w, wrap) = solve_auto_bfc_margin_beside(
                                        false,
                                        ml,
                                        mr,
                                        frame,
                                        0.0,
                                        band_right,
                                        container_width,
                                    );
                                    if wrap {
                                        // 下绕：全 CB 重解宽、x 回流位。
                                        if float_bottom > child.y {
                                            child.y = float_bottom;
                                        }
                                        child.x = ml;
                                        child.width = (container_width - margin_h).max(0.0);
                                        shrink_bfc_content_width(child);
                                        // R5222：rf 镜像同左——下绕后记流游标底与快照抬升
                                        //（探针 float2 y=100 级联实证，机制与左侧共用）。
                                        let new_bottom = child.y + child.height;
                                        wrap_floors.push((idx + 1, new_bottom));
                                        pending_geom_bump = Some(new_bottom);
                                    } else {
                                        // 旁置钉位（含小负 margin 落 x=0、w=gap）。
                                        child.x = beside_x;
                                        child.width = beside_w;
                                        shrink_bfc_content_width(child);
                                    }
                                } else if std::env::var("ZW_BFC_RIGHT_PUSHBELOW").as_deref() != Ok("0")
                                    && is_definite_width
                                    && !has_following_block_sibling[idx]
                                    && !child.is_layout_container
                                {
                                    if float_bottom > child.y {
                                        child.y = float_bottom;
                                    }
                                    if let Some(width) = child.declared_width_px {
                                        child.width = width;
                                        shrink_bfc_content_width(child);
                                    }
                                    child.x = child.margin_left;
                                } else {
                                    // 右浮动：缩小 BFC 元素宽度以不重叠 float 的 margin-box。
                                    // R5217：float_x 是 float border-box 左缘，§9.5 约束边是
                                    // margin-box 左缘（001/002 v=0 cells 实证 ref 宽 16 =
                                    // float_x − margin_l，旧算式给 18 多留 float 左 margin）。
                                    let new_width = float_x - *float_margin_l - child.x;
                                    child.width = new_width.max(0.0);
                                    shrink_bfc_content_width(child);
                                }
                            }
                            _ => {}
                        }
                    }
                } // end if !coord_handled（R1730 Slice 5：coord_handled 时跳过 per-float 循环）

                // R5221：应用快照抬升——float 子索引在本盒之后的 float 带底抬到
                // 下绕 BFC 新底（007：float2 带 50..100 → 100..150，bfc2 避让据此
                // 下推 150）。
                if let Some(nb) = pending_geom_bump {
                    for (gi2, &ci) in float_child_indices.iter().enumerate() {
                        if ci > idx && float_geometries[gi2].2 < nb {
                            float_geometries[gi2].2 = nb;
                        }
                    }
                }

                // R1619 Slice 2（嵌套 BFC 祖先 float 下沉）：直接同胞 float 由上方 float_geometries
                // 循环（R1369 左 / 右 shrink）处理；此处处理**祖先 float**（经 inherited_floats 透传，
                // 即嵌套在非 BFC 后代内的 BFC 看到的外层 float）。declared-width BFC（width 非 auto）
                // 放不下 float 旁可用宽时，下沉到 float 底（CSS §9.5），而非被 shrink 到 0/负宽。
                // 与 R1369 分离：R1369 用 `width < container_width` 代理，嵌套+margin 上下文下 BFC
                // 溢出窄父失效（with-margin-008/009）；此处用 `!declared_width_auto` 精确判定。
                // env ZW_NESTED_BFC_FLOAT_AVOID=0 关闭（kill-switch，default-on）。
                if nested_avoid_on
                    && !inherited_floats.is_empty()
                    && !has_following_block_sibling[idx]
                    && !child.declared_width_auto
                {
                    let ctop = child.y;
                    let cbottom = child.y + child.height;
                    for (fdir, fx, fy, fbw, fh, fmr, _fml) in inherited_floats.iter() {
                        let fbottom = fy + fh;
                        // 垂直重叠
                        if !(ctop < fbottom && cbottom > *fy) {
                            continue;
                        }
                        // float 旁可用宽（本容器 border-box 帧）
                        let available = match fdir {
                            FloatValue::Left => container_width - (fx + fbw + fmr),
                            FloatValue::Right => fx - child.x,
                            _ => continue,
                        };
                        let declared_or_layout_width = child.declared_width_px.unwrap_or(child.width);
                        if declared_or_layout_width > available + 0.5 {
                            if fbottom > child.y {
                                child.y = fbottom;
                            }
                            if let Some(width) = child.declared_width_px {
                                child.width = width;
                                shrink_bfc_content_width(child);
                            }
                            // 回正常流左对齐（float 在侧，BFC 下沉后不与 float 同行）。
                            child.x = child.margin_left;
                            break;
                        }
                    }
                }
            }

            // R1392：nested-float clearance——非 BFC 子内的嵌套浮动须对后续 clear 兄弟可见
            //（§9.5：clear 清除同 BFC 上下文内所有浮动）。ZW 的 active_*_float_bottom 仅收集
            // 直接 float 子，嵌套在非 BFC wrapper 内的 float（adjoining-float-before-clearance）
            // 对 clear 不可见。此处在子定位完成后扫描其非 BFC 后代的浮动底边，并入追踪。
            // env `ZW_NESTED_FLOAT_CLEARANCE=0` 关闭（kill-switch，default-on）。
            if std::env::var("ZW_NESTED_FLOAT_CLEARANCE").as_deref() != Ok("0")
                && !child.is_absolute
                && !child.is_fixed
                && matches!(child.float, FloatValue::None)
                && !crate::margin_collapse::establishes_bfc(child)
            {
                let cy = child.y;
                // R4235 帧约定：content-rel 模式（default-on）下 child.y 已是内容盒相对，
                // active_*_float_bottom 同帧（direct float 子的 child_bottom = y+h+mb 不减
                // offset）——嵌套收集须同帧（offset 0），否则带 border/padding-top 的容器
                // 嵌套浮动底边整体虚减 content_y_offset，后续 clear 兄弟 clearance 不足、
                // 停在 float 底边之上（baidu hotsearch wrapper pt:21 → ::after 落 93 应 114）。
                // border-rel 兼容模式（ZW_FLOAT_CLAMP_CONTENT_REL=0）保留旧 offset 换算。
                // https://www.w3.org/TR/CSS22/visuren.html#clearance
                let (nl, nr) = nested_float_bottoms(child, cy, if clamp_content_rel { 0.0 } else { content_y_offset });
                if nl > 0.0 {
                    nested_left_bottom = nested_left_bottom.max(nl);
                }
                if nr > 0.0 {
                    nested_right_bottom = nested_right_bottom.max(nr);
                }
                if nl > active_left_float_bottom {
                    active_left_float_bottom = nl;
                }
                if nr > active_right_float_bottom {
                    active_right_float_bottom = nr;
                }
            }
        }
    } else if content_y_offset == 0.0 && std::env::var("ZW_CLEAR_NO_FLOAT_CONTEXT").as_deref() != Ok("0") {
        // R1389：has_active_float_context=false（容器无直接 float 子，且 inherited float
        // bottom 已 clamp 到 0 = 容器已在所有祖先 float 下方）时，clear 子元素**无浮动可清除**。
        // 但 taffy 0.12 仍基于同 BFC 的祖先 float 对 clear 子误 apply clearance，把 clear 子
        // 推到其 flow 位置之下并膨胀容器高度（no-clearance-due-to-large-margin：red h=83 应 20，
        // clear 在 red 底部而非顶部）。此处对窄情形（容器无 border-top/padding-top + 唯一 in-flow
        // block 子为 clear 元素 + auto-height）将 clear 子重定位到容器 content top（其 margin-top
        // 经无 border/padding-top 的容器折叠穿出，位置本应如此），并按 in-flow 子 border-box 收缩
        // 容器高度。env `ZW_CLEAR_NO_FLOAT_CONTEXT=0` 关闭（kill-switch，default-on）。
        let clear_idx = box_node.children.iter().position(|c| {
            !c.is_absolute
                && !c.is_fixed
                && c.is_block_level
                && !matches!(
                    c.clear,
                    ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
                )
        });
        if let Some(idx) = clear_idx {
            // clear 子须为唯一 in-flow block 子（无其它 block 兄弟），避免重定位影响兄弟流位置
            //（多 block 兄弟情形须完整 flow 跟踪，留多 session）。
            let in_flow_block_count = box_node
                .children
                .iter()
                .filter(|c| !c.is_absolute && !c.is_fixed && c.is_block_level)
                .count();
            if in_flow_block_count == 1 {
                let respect_explicit_height = std::env::var("ZW_FLOAT_RESPECT_HEIGHT").as_deref() != Ok("0");
                let auto_height = !respect_explicit_height || box_node.declared_height_auto;
                if auto_height {
                    // 重定位 clear 子到 content top（margin-top 折叠穿出容器，border-top 落在
                    // 容器 content 原点；content_y_offset 此分支恒为 0）。
                    box_node.children[idx].y = content_y_offset;
                    // 按 in-flow 子 border-box 收缩容器（clear 的 margin-top 不计入高度）。
                    let content_bottom =
                        box_node
                            .children
                            .iter()
                            .filter(|c| !c.is_absolute && !c.is_fixed)
                            .fold(0.0f32, |max_y, c| {
                                let bottom = c.y + c.height + c.margin_bottom;
                                max_y.max(bottom)
                            });
                    let new_content_height = (content_bottom - content_y_offset).max(0.0);
                    if new_content_height < box_node.content_height {
                        box_node.content_height = new_content_height;
                        let new_total = new_content_height
                            + box_node.padding_top
                            + box_node.padding_bottom
                            + box_node.border_top
                            + box_node.border_bottom;
                        if new_total < box_node.height {
                            box_node.height = new_total;
                        }
                    }
                }
            }
        }
    }

    // 调整容器高度：
    // CSS 2.1 §10.6.7：建立 BFC 的容器必须包含浮动元素的外底边
    // （margin-box bottom）。非 BFC 容器中，浮动元素不贡献高度，
    // 因此需要收缩容器以去除 taffy 将 float 当作 block 时多计算的空间。
    //
    // 注意：子元素的 x/y 坐标是相对于父元素 content area 的，
    // 所以 content_bottom 也是相对于 content area 顶部的。
    // 不需要减去 content_y（那是相对于自身 border-box 的局部偏移，不含位置量）。
    if !float_taffy_y.is_empty() || ran_adjoining_clearance {
        let establishes_bfc = crate::margin_collapse::establishes_bfc(box_node);
        // 多列容器虽然建立 BFC（阻止 margin 折叠），
        // 但其内容通过列分布，不应使用 BFC 的浮动包含高度逻辑。
        // 使用非 BFC 路径按正常流内容高度收缩。
        let use_bfc_float_containment = establishes_bfc && !box_node.is_multicol;

        if use_bfc_float_containment {
            // BFC 容器：浮动元素已包含在高度中（taffy 正确计算了含 float 的 auto height）
            // 不需要收缩。但需要确保高度至少覆盖到最低浮动元素的外底边。
            let float_bottom = box_node
                .children
                .iter()
                .filter(|c| !matches!(c.float, FloatValue::None) && !c.is_absolute && !c.is_fixed)
                .fold(0.0f32, |max_y, c| {
                    let bottom = c.y + c.height + c.margin_bottom;
                    max_y.max(bottom)
                });
            if float_bottom > box_node.content_height {
                box_node.content_height = float_bottom;
                let new_total = float_bottom
                    + box_node.padding_top
                    + box_node.padding_bottom
                    + box_node.border_top
                    + box_node.border_bottom;
                if new_total > box_node.height {
                    box_node.height = new_total;
                }
            }
        } else {
            // 非 BFC 容器：浮动元素不贡献高度，收缩容器
            //
            // R1277 ④：显式高度（definite height）容器不应被收缩。CSS §10.5：显式高度的
            // used height 即显式值，float 重定位后 content_bottom 下降（如 floats-006 float
            // 上提后 content_bottom 100 < height 200）不应改变容器高度——内容仅溢出/不足。
            // 旧实现无此守卫，致 `#div1{height:200px}` 在 float 上提后被错误塌缩到 100
            //（R1273 实证，R1277 经 DIV1_TRACE 二分定位塌缩源=本函数 L779 非 BFC 收缩路径，
            // 非 exclude_floats_from_non_bfc_auto_height）。env `ZW_FLOAT_RESPECT_HEIGHT=0`
            // 可关闭（kill-switch，default-on；同 R109_BACKFILL 约定）。
            let respect_explicit_height = std::env::var("ZW_FLOAT_RESPECT_HEIGHT").as_deref() != Ok("0");
            if respect_explicit_height && !box_node.declared_height_auto {
                // 显式高度容器：跳过收缩（CSS §10.5 used height = 显式值）。
            } else {
                // R1323 §8.3.1：auto-height 非 BFC + clear 子（进入此块即有 float 上下文）
                // → clearance_active。覆盖非空 cleared（margin-collapse-clear-015：clear-left
                // 有子，empty-gate 排除 containment math 但仍须 sibling-shift）+ negative clearance
                //（hypothetical>clear_bottom，clearance 仍 stop collapse per 014 assert）。
                // sibling-shift leak 公式按 declared_margin_bottom 自门控（027 #div2 declared mb 安全）。
                // clearance_active 语义 =「in-flow clearance/containment 机制对本容器生效」，
                // 故仅统计会被子循环流内 clearance 分支处理的子（block-level、非 float、
                // 非定位）。float 子自身的 clear 由 Phase 1 float 摆位解决（CSS2 §9.5.1），
                // 不产生流内 clearance 链——旧实现把 float 子的 clear 也计入，容器被误标
                // 后 R1319 sibling-shift 把后续兄弟的合法 clearance 当「泄漏」撤销
                //（clearfix 伪元素 + float 行热榜形状：wrapper::after 被拉回 float 容器底，
                // 容器高度塌回 0/63，后续内容整体叠压，baidu hotsearch 叠字根因）。
                // https://www.w3.org/TR/CSS22/visuren.html#clearance
                let has_clear_child = box_node.children.iter().any(|c| {
                    c.is_block_level
                        && !c.is_absolute
                        && !c.is_fixed
                        && matches!(c.float, FloatValue::None)
                        && !matches!(
                            c.clear,
                            ClearValue::None | ClearValue::InlineStart | ClearValue::InlineEnd
                        )
                });
                if has_clear_child {
                    box_node.clearance_active = true;
                }
                if had_empty_clearance {
                    // R1317 §8.3.1 containment（empty cleared）：trailing collapse-through 链留父内。
                    // 父 content_height =（最后建立流位置子 border-box 底）+（trailing 链 − consumed mt）。
                    // 例 012：flow_bottom=100 + (last_flow_mb=140 − consumed=40) = 200。
                    let contained_chain = (last_flow_mb - clearance_consumed_mt).max(0.0);
                    let content_height = (flow_bottom + contained_chain).max(0.0);
                    if content_height > box_node.content_height {
                        box_node.content_height = content_height;
                        let new_total = content_height
                            + box_node.padding_top
                            + box_node.padding_bottom
                            + box_node.border_top
                            + box_node.border_bottom;
                        if new_total > box_node.height {
                            box_node.height = new_total;
                        }
                        // 仅当 containment 实际扩张时才标记 had_clearance（exclude_floats 跳过用）。
                        box_node.had_clearance = true;
                    }
                } else {
                    let content_bottom =
                        box_node
                            .children
                            .iter()
                            .filter(|c| !c.is_absolute && !c.is_fixed)
                            .fold(0.0f32, |max_y, c| {
                                let bottom = c.y + c.height + c.margin_bottom;
                                max_y.max(bottom)
                            });
                    // R1324：content_bottom 为 border-box 相对（c.y 含 content_y_offset =
                    // border_top + padding_top），须换算为 content 相对后再赋 content_height
                    //（后者不含 border/padding）。否则带 border-top/padding-top 的容器
                    //（margin-collapse-clear-015 border-top:1px）content_height 会多算一段
                    // content_y_offset，容器偏高 → 后续 in-flow 兄弟整体下移（015 残余 1px）。
                    // R4235：content-rel 模式下 content_bottom 已是内容盒相对（float 写点
                    // 与提取层同约定），不再减 offset（R1324 的减法按旧 border-rel 写点校准）。
                    let content_height = if clamp_content_rel {
                        content_bottom.max(0.0)
                    } else {
                        (content_bottom - content_y_offset).max(0.0)
                    };
                    // 如果内容区域实际高度小于 taffy 计算的高度，收缩容器
                    if content_height < box_node.content_height {
                        // R5109：clearance 相关容器（clearance_active 已置位）的高度收缩
                        // 须记录缩减量——祖先盒的 taffy 高度按收缩前的子 extent 计算，
                        // 不向上传播则祖先残留泄漏段（margin-collapse-157 row1-b 的
                        // .test 多 16px / margin-collapse-142 的 td 多 64px 露红底）。
                        // 非 clearance 容器（纯 float 子收缩）不记录，行为不变。
                        let shrink = box_node.content_height - content_height;
                        box_node.content_height = content_height;
                        // 更新总高度（包含 padding + border）
                        let new_total = content_height
                            + box_node.padding_top
                            + box_node.padding_bottom
                            + box_node.border_top
                            + box_node.border_bottom;
                        // 仅当新高度更小时才更新（不扩大容器）
                        if new_total < box_node.height {
                            box_node.height = new_total;
                        }
                        if box_node.clearance_active {
                            box_node.clearance_extent_reduction += shrink;
                        }
                    }
                }
            }
        }
    }

    // R1318/R1319：had_clearance 仅在上方 containment 分支实际扩张 auto-height 容器时置 true
    //（非显式高度、且确有 empty-block clearance）。供 exclude_floats 跳过收缩 + sibling-shift
    // 位移后续兄弟。box_node.had_clearance 默认 false（literal/Default 初始化）。

    // 递归处理子容器
    // R5307：前序非 BFC 兄弟子树内嵌套 float 的同上下文透传累加器（§9.5：同 BFC
    // 上下文内后继 float/兄弟子树须避开更早 float）。递归循环内边递归边收集——子树
    // 内 float 的最终位由其自身 walk 定谳，须递归返回后读取才权威；收集结果只并入
    // **后继**兄弟的继承上下文（R5160 文档顺序原则：更早子不被更晚 float 约束）。
    // 底边并入 left_ctx/right_ctx（gate 开时 R1392 已按 taffy 期位置并入 active 底，
    // 此处按定谳位置 max 补齐，幂等）；带集并入 child_inherited（此前嵌套 float 带
    // 从不透传，兄弟块内 float 落位试探看不到 → position-absolute-dynamic-relayout-004）。
    // https://www.w3.org/TR/CSS22/visuren.html#float-position
    let mut nested_left_acc = 0.0f32;
    let mut nested_right_acc = 0.0f32;
    let mut nested_geoms_acc: Vec<FloatGeom> = Vec::new();
    for (idx, child) in box_node.children.iter_mut().enumerate() {
        let (left_ctx, right_ctx) = child_float_contexts
            .get(idx)
            .copied()
            .unwrap_or((inherited_left_bottom, inherited_right_bottom));
        let left_ctx = left_ctx.max(nested_left_acc);
        let right_ctx = right_ctx.max(nested_right_acc);
        let child_content_abs_y = box_content_abs_y + child.y + child.content_y;
        if crate::margin_collapse::establishes_bfc(child) {
            // BFC 子：独立浮动上下文，祖先 float 不透传。
            adjust_float_positions_with_context(child, child_content_abs_y, 0.0, 0.0, &[], doc, styles, float_subtrees);
        } else {
            // 非 BFC 子：其内 float 与外层同 BFC 上下文。R1619 Slice 2 透传祖先带 +
            // R5307 文档序在前的直接 float 带 + 前序兄弟嵌套 float 带（nested_geoms_acc，
            // 递归返回后收集，恒文档序在前）到子 border-box 帧（减 child.x/child.y），使
            // 嵌套 BFC 后代能避开外层 float、兄弟块内 float 能避开前序 float。直接 float
            // 带按 R5160 文档序过滤（R5307：带集自 R5307 起约束嵌套 walk 的 float 落位，
            // 后序 float 带漏进前序子会把前序子树内 float 误推到其下方——
            // position-absolute-dynamic-relayout 结构里 wrapper 内 float1 被后序 float2
            // 带顶到 50 实证）。
            let has_prior_floats = !inherited_floats.is_empty()
                || !nested_geoms_acc.is_empty()
                || float_child_indices.iter().any(|ci| *ci < idx);
            let child_inherited: Vec<FloatGeom> = if nested_avoid_on && has_prior_floats {
                // float_geometries 借用 box_node.children；先取出 child 偏移再 map（避免与 child mut 借用冲突）。
                let (cx, cy) = (child.x, child.y);
                inherited_floats
                    .iter()
                    .chain(
                        float_geometries
                            .iter()
                            .zip(float_child_indices.iter())
                            .filter(|(_, ci)| **ci < idx)
                            .map(|(g, _)| g),
                    )
                    .chain(nested_geoms_acc.iter())
                    .map(|(d, fx, fy, w, h, mr, fml)| (d.clone(), fx - cx, fy - cy, *w, *h, *mr, *fml))
                    .collect()
            } else {
                Vec::new()
            };
            adjust_float_positions_with_context(
                child,
                child_content_abs_y,
                box_content_abs_y + left_ctx,
                box_content_abs_y + right_ctx,
                &child_inherited,
                doc,
                styles,
                float_subtrees,
            );
        }
        // 递归定谳后收集该子非 BFC 子树内的嵌套 float（BFC/float/定位子独立上下文
        // 或外溢隔离，不入累加器；帧约定同 nested_float_bottoms/nested_float_geoms）。
        // R5307 性能门：无 float 子树跳过（旗标预扫；收集器在无 float 子树本就返回空，
        // 跳过与收集逐值等价）。无 id 匿名盒查不到旗标，保守回退为收集。
        if !child.is_absolute
            && !child.is_fixed
            && matches!(child.float, FloatValue::None)
            && !crate::margin_collapse::establishes_bfc(child)
            && float_subtrees.is_some_and(|m| child.node_id.is_none_or(|id| m.get(&id).copied().unwrap_or(true)))
        {
            let outer_content_y_offset = if clamp_content_rel { 0.0 } else { content_y_offset };
            let (nl, nr) = nested_float_bottoms(child, child.y, outer_content_y_offset);
            nested_left_acc = nested_left_acc.max(nl);
            nested_right_acc = nested_right_acc.max(nr);
            nested_geoms_acc.extend(nested_float_geoms(child, child.x, child.y, outer_content_y_offset));
        }
    }

    // https://www.w3.org/TR/CSS21/visudet.html#root-height
    // A floated box (and every other flow root) establishes a block formatting
    // context, so its auto height contains its floated descendants. Descendant
    // float positions are finalized by the recursion above; recompute afterwards
    // so an initially stacked row of floated navigation items does not leave its
    // parent at the stale pre-float height.
    if box_node.declared_height_auto
        && !box_node.is_absolute
        && !box_node.is_fixed
        && !box_node.is_replaced
        // R3903（CSS Flexbox §4 / Grid §4）：布局项的尺寸由 flex/grid 算法决定——
        // taffy align-stretch 定高的 item（如 inline-block item）被本重算按「无子 →
        // content_bottom=0」清零（flexbox_flex-1-* 24 案 10.41% 簇）。排除。
        && !box_node.is_flex_grid_item
        // R4060（css-contain-1 §containment-size）：contain:size 容器的 taffy 高已是
        // definite（converter 折 CIS-or-0），本「BFC auto-height 含浮动后代」重算
        // 不适用——重算把 definite-0 撑回子底边（contain-size-button-002：contained
        // button 34 被撑到 117）。同 R4057 R1743 豁免：definite used size 胜内容回填。
        && !box_node.has_size_containment
        && (box_node.is_flow_root || matches!(box_node.float, FloatValue::Left | FloatValue::Right))
    {
        // 本重算的语义是「BFC 的 auto height 包含其浮动后代」，仅适用于参与文档流的
        // 普通容器，须排除两类盒子：
        // ① absolute/fixed 定位（脱离文档流，高度由 inset stretch/内容决定——R1781
        //    abspos input stretch 94px 被压成 8）；
        // ② replaced 元素（canvas/img/video/iframe/embed/object/svg/applet 等）：auto
        //    height 由固有尺寸或 CSS 决定，无子元素时 content_bottom fold 初始 0.0 会把
        //    固有高度压成 0（R3268 canvas 显示链路全平台 FAIL）。
        // R4069（CSS §8.3.1 + chromium 151 oracle 2026-09-06）：空块子 collapse-through
        // 后 mb 与父底边折叠，不计父高（同 R4068 R1743 fold 修正语义；oracle：BFC 父 +
        // 空子 m30 → 父高 30 非 60）。有内容子/float 子保持 mb 计入（oracle：p mt30 h20
        // mb30 → 父 80 = 30+20+30）。
        recompute_bfc_auto_height(box_node, content_y_offset);
    }
}

/// R4140（CSS §8.3.1）：**inline-level 子的 margin 不与块容器折叠**。taffy 把
/// inline-block 等原子行内级映射为 Block 子，当容器首个流内子是原子且容器无
/// border/padding-top 时，容器 margin-top 被折叠抬升到该原子的 mt（取 max）；
/// 而原子自身的行内定位（IFC run.y，R3904/R3809）又应用一次 mt → 双计（容器及
/// 全部后代整体偏低 over_collapse）。探针实证（debug_r4140_layout_dump）：body
/// margin:8 + 首子 inline-block mt:32 → body y=32 应 8；原子 y 相对容器已由 IFC
/// 正确摆位（=mt），故只需把容器多折叠的量从 y 扣除并恢复 margin_top——与上方
/// float 修正同一形状。门控（五重）：
///   1. 容器无 border-top/padding-top（margin 可与首个子元素折叠）
///   2. 容器布局 margin_top > 声明值（发生了膨胀）
///   3. 首个流内子为 inline-level（!is_block_level；含 anonymous text item 时
///      其无 LayoutBox 或 mt=0，天然不匹配）
///   4. 容器 margin_top == 该子 margin_top（被折叠到该子的 mt）
///   5. 该子 margin_top == 声明值（自身未被 taffy 膨胀）
///
///   kill-switch：env `ZW_INLINE_FIRST_MT_UNHOIST=0` 关闭（default-on）。
///
/// R4217：从 [`adjust_float_positions_with_context`] 原样抽取为 helper——全 walk 与
/// 无 float 快速路径（[`adjust_no_float_page`]）共用，语义零变化。
fn unhoist_inline_first_atom_mt(box_node: &mut LayoutBox, content_y_offset: f32) {
    if std::env::var("ZW_INLINE_FIRST_MT_UNHOIST").as_deref() != Ok("0")
        && content_y_offset == 0.0
        && box_node.margin_top > box_node.declared_margin_top + 0.01
    {
        if let Some(fc) = box_node
            .children
            .iter()
            .find(|c| !c.is_absolute && !c.is_fixed && !c.is_anonymous_text_item)
            .filter(|c| matches!(c.float, FloatValue::None) && !c.is_block_level)
        {
            let container_absorbed_atom_mt = (box_node.margin_top - fc.margin_top).abs() < 0.01;
            let atom_mt_is_clean = (fc.margin_top - fc.declared_margin_top).abs() < 0.01;
            if container_absorbed_atom_mt && atom_mt_is_clean {
                let over_collapse = box_node.margin_top - box_node.declared_margin_top;
                box_node.y -= over_collapse;
                box_node.margin_top = box_node.declared_margin_top;
            }
        }
    }
}

/// https://www.w3.org/TR/CSS21/visudet.html#root-height（BFC auto-height 含浮动后代
/// 重算，R4069 fold 语义）——原样抽取为 helper（R4217，同 [`unhoist_inline_first_atom_mt`]）。
/// `content_y_offset` = 容器 border_top + padding_top（调用方现场计算，与全 walk 一致）。
fn recompute_bfc_auto_height(box_node: &mut LayoutBox, content_y_offset: f32) {
    let content_bottom = box_node
        .children
        .iter()
        .filter(|child| !child.is_absolute && !child.is_fixed)
        .map(|child| {
            if crate::margin_collapse::is_empty_block(child) {
                child.y + child.height
            } else {
                child.y + child.height + child.margin_bottom
            }
        })
        .fold(0.0f32, f32::max);
    // R4235：content-rel 模式（同 walk 内各臂）——children y 已是内容盒相对，减
    // offset 会按 border/padding 少算容器高（R1324 的减法按旧 border-rel 写点校准）。
    let content_height = if *FLOAT_CLAMP_CONTENT_REL {
        content_bottom.max(0.0)
    } else {
        (content_bottom - content_y_offset).max(0.0)
    };
    box_node.content_height = content_height;
    box_node.height =
        content_height + box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
}

/// R4217（perf pullback slice 3）：**无 float 页面快速路径**。
///
/// 全 walk（[`adjust_float_positions_with_context`]）的 Phase 1/2 float 机制
/// （float 重定位、clearance、BFC float 回填）在无 float 页面上逐节点空转
/// （block_layout_1000 探针实测 ~0.15-0.29ms/布局）。无 float 时仍有行为的臂
/// 只有两个，本快速路径以**同序**（self 臂 → 递归子 → 高度重算）保留：
/// ① [`unhoist_inline_first_atom_mt`]（R4140，per 容器）；
/// ② [`recompute_bfc_auto_height`]（per flow-root 容器，R4069 fold 语义）。
/// 其余全 walk 臂在无 float 页面均为数值 no-op：is_layout_container float 归零
/// （float 恒 None）、float-mt 解折叠（需 float 子，不可触发）、Phase 1/2 的
/// float 定位 / clear 位移（float bottom 恒 0，max 无操作）。
pub(crate) fn adjust_no_float_page(box_node: &mut LayoutBox) {
    let content_y_offset = box_node.border_top + box_node.padding_top;
    unhoist_inline_first_atom_mt(box_node, content_y_offset);
    for child in &mut box_node.children {
        adjust_no_float_page(child);
    }
    if box_node.declared_height_auto
        && !box_node.is_absolute
        && !box_node.is_fixed
        && !box_node.is_replaced
        && !box_node.is_flex_grid_item
        && !box_node.has_size_containment
        && (box_node.is_flow_root || matches!(box_node.float, FloatValue::Left | FloatValue::Right))
    {
        recompute_bfc_auto_height(box_node, content_y_offset);
    }
}

/// R5070（CSS §9.5/§10.3.5）：终末 inline 载体右浮 CB 修正——float 的包含块 = 最近
/// **块级**祖先容器 content box（inline 元素盒不构成包含块）。IFC 载体（span.nowrap
/// 型）内的 `float:right` 由 adjust_float_positions 的 Right 臂按载体盒自身宽右对齐
///（CB 选取错误），且该 pass 跑在载体 x 终值确定之前（abs 锚定不可行——R5070 首版
/// walk 内实现探针实证 box_abs_x=8 为 div 位）。本终末 pass 于全部重定位之后（载体
/// x 终值已定，R1733 终末位之后）重定：`float:right` 且直接父为 inline 载体
///（非 block_level 非 flow_root）→ border-box 左缘 = 最近块级 CB content 右缘 −
/// float margin-box 宽。kill-switch `ZW_FLOAT_CB_BLOCK=0`（default-on）。
pub(crate) fn fix_inline_carrier_right_float_abs(
    box_node: &mut LayoutBox,
    parent_content_abs_x: f32,
    cb_content_right_abs: Option<f32>,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    if std::env::var("ZW_FLOAT_CB_BLOCK").as_deref() == Ok("0") {
        return;
    }
    let content_left_abs =
        parent_content_abs_x + box_node.x + box_node.margin_left + box_node.border_left + box_node.padding_left;
    // R5214：table 内部盒（row-group/row/cell 等）既非 block_level 亦非 flow_root，旧
    // 判定误作 inline 载体——cell 内 right float 被「重定」到表右缘覆盖先落位的 float
    //（floats-wrap-bfc-004 S4/S8：silver 期望 x=100 被 300−100=200 覆盖）。CSS 2.1 §10.1
    // float 包含块 = 最近 block-level / **table-cell** / table-caption 祖先 content box：
    // table 内部盒按 CB 边界处理（升级 cb_right），其直接 float 子由摆放 pass 定位、
    // 载体修正不重写。
    let is_table_internal = box_node.node_id.and_then(|id| styles.get(&id)).is_some_and(|s| {
        matches!(
            s.display,
            DisplayValue::TableRow
                | DisplayValue::TableRowGroup
                | DisplayValue::TableHeaderGroup
                | DisplayValue::TableFooterGroup
                | DisplayValue::TableCell
                | DisplayValue::TableColumn
                | DisplayValue::TableColumnGroup
        )
    });
    // 本盒块级（BFC flow root 亦构成 float 包含块）→ 升级为最近块级 CB；inline 载体继承。
    let cb_right =
        if box_node.is_block_level || box_node.is_flow_root || is_table_internal || cb_content_right_abs.is_none() {
            Some(content_left_abs + box_node.content_width)
        } else {
            cb_content_right_abs
        };
    let is_inline_carrier = !box_node.is_block_level && !box_node.is_flow_root && !is_table_internal;
    for child in &mut box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        if is_inline_carrier
            && matches!(child.float, FloatValue::Right)
            && let Some(cr) = cb_right
        {
            let outer_w = child.width
                + child.margin_left
                + child.margin_right
                + child.border_left
                + child.border_right
                + child.padding_left
                + child.padding_right;
            child.x = cr - outer_w - content_left_abs;
        }
        fix_inline_carrier_right_float_abs(child, content_left_abs, cb_right, styles);
    }
}
