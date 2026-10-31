//! R5265：collapse cell 带 z 序相位间插 — CSS2.1 Appendix E / Blink
//! `PaintPhase::kCollapsedTableBorders`（「后代块背景 < 带 < inline 内容」）。
//!
//! cell 子树拆两遍 sweep：sweep A（带之前）沿 plain-block 链只绘装饰（相位
//! -1..3b：shadow/backdrop/背景/边框/outline）；随后带补发射（deco 相位置位）、
//! 自身 inline/替换内容；sweep B（带之后）plain-block 抑制装饰补文本/子树，
//! 其余子树（inline/atomic/浮动/SC）整体常规绘制。positioned 后代不入 sweep
//!（scope 步 2/6/7 照旧）；SC/浮动/overflow 裁剪/多列子树整体留 sweep B
//!（= 既有 z 序，拆分会破坏其 per-node 效果/裁剪窗口）。

use std::collections::HashMap;

use zero_css_parser::values::{FloatValue, VisibilityValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::LayoutBox;
use zero_layout_engine::types::OverflowClip;
use zero_style_system::property::types::DisplayValue;
use zero_style_system::{BorderCollapseValue, ComputedStyle, ContentVisibilityValue, WritingModeValue};

use super::super::helpers::PrimitiveCounts;
use super::{Painter, box_paints_in_transform_phase, is_positioned_child, ordered_child_indices};

/// R5265：collapse cell 相位间插 sweep 栈帧（Painter 递归旗标，None = 常规路径）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CollapseCellPhase {
    /// sweep A：仅绘装饰并沿 plain-block 链下探（带之前的后代块背景域）。
    DecoSweep,
    /// sweep B：抑制自身装饰，绘文本并分派子树（带之后的 inline 内容域）。
    ContentSweep,
}

/// R5265：cell 是否启用带相位间插驱动（deco 相位带置位，由子树分派处补发射）。
///
/// 门：TableCell + border-collapse:collapse + 水平书写模式（带模型域）+ 非多列
/// 容器 + 有子盒（text-only cell 自身文本已在带之上，无需拆分）+ 至少一条胜出
/// 边框（全零无带可插）+ 可见（hidden 时 deco/内容相位本就不发射）。
pub(crate) fn is_collapse_band_cell(box_node: &LayoutBox, style: &ComputedStyle) -> bool {
    matches!(style.display, DisplayValue::TableCell)
        && matches!(style.border_collapse, BorderCollapseValue::Collapse)
        && matches!(box_node.writing_mode, WritingModeValue::HorizontalTb)
        && !box_node.is_multicol
        && !box_node.is_nested_spanner_wrapper
        && !box_node.children.is_empty()
        && !matches!(style.visibility, VisibilityValue::Hidden | VisibilityValue::Collapse)
        && (box_node.border_top > 0.0
            || box_node.border_right > 0.0
            || box_node.border_bottom > 0.0
            || box_node.border_left > 0.0)
}

/// R5265：sweep 下探分类——plain-block 且无独立绘制/裁剪/效果域的 in-flow 子盒
/// 才拆两遍（creates_stacking_context 含 opacity/transform/filter 等 per-node
/// 效果窗口载体，拆分即窗口割裂；overflow 非 Visible 的 R793 裁剪在自身 paint
/// 末尾，DecoSweep 早返不经过；浮动/多列属既有相位不动）。
fn sweeps_as_plain_block(child: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    if child.creates_stacking_context
        || !matches!(child.float, FloatValue::None)
        || child.overflow_x != OverflowClip::Visible
        || child.overflow_y != OverflowClip::Visible
        || child.is_multicol
        || child.is_nested_spanner_wrapper
        || !child.column_span_offsets.is_empty()
    {
        return false;
    }
    child.node_id.and_then(|id| styles.get(&id)).is_some_and(|st| {
        matches!(
            st.display,
            DisplayValue::Block | DisplayValue::Flow | DisplayValue::FlowRoot | DisplayValue::ListItem
        )
    })
}

/// canvas 元素盒判定（fallback 内容不绘制，HTML §4.8.10——R57 主循环同款门）。
fn is_canvas_container(box_node: &LayoutBox, doc: Option<&Document>) -> bool {
    box_node
        .node_id
        .and_then(|id| doc.and_then(|d| d.get(id)))
        .is_some_and(|n| matches!(&n.kind, NodeKind::Element(e) if e.local_name() == "canvas"))
}

impl Painter {
    /// sweep A：对 in-flow 子盒中的 plain-block 链逐个以 [`CollapseCellPhase::DecoSweep`]
    /// 模式绘制（仅装饰；文本/子树内容/positioned flush 留给 sweep B）。
    pub(crate) fn paint_collapse_deco_sweep_children(
        &mut self,
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
        child_offset_x: f32,
        child_offset_y: f32,
        doc: Option<&Document>,
        in_flow: &[usize],
    ) {
        if is_canvas_container(box_node, doc) {
            return;
        }
        for child_idx in in_flow {
            let child = &box_node.children[*child_idx];
            if !sweeps_as_plain_block(child, styles) {
                continue;
            }
            let prev = self.collapse_cell_phase;
            self.collapse_cell_phase = Some(CollapseCellPhase::DecoSweep);
            self.paint_node(child, styles, child_offset_x, child_offset_y, doc, false);
            self.collapse_cell_phase = prev;
        }
    }

    /// sweep B：plain-block 子以 [`CollapseCellPhase::ContentSweep`] 模式绘制
    ///（装饰抑制、文本/子树补齐），其余子树整体常规绘制（相位域 inline/atomic/
    /// 浮动/SC，= 既有 z 序）。
    pub(crate) fn paint_collapse_content_sweep_children(
        &mut self,
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
        child_offset_x: f32,
        child_offset_y: f32,
        doc: Option<&Document>,
        in_flow: &[usize],
    ) {
        if is_canvas_container(box_node, doc) {
            return;
        }
        for child_idx in in_flow {
            let child = &box_node.children[*child_idx];
            let prev = self.collapse_cell_phase;
            if sweeps_as_plain_block(child, styles) {
                self.collapse_cell_phase = Some(CollapseCellPhase::ContentSweep);
            } else {
                self.collapse_cell_phase = None;
            }
            self.paint_node(child, styles, child_offset_x, child_offset_y, doc, false);
            self.collapse_cell_phase = prev;
        }
    }

    /// cell 自身 inline/替换内容相位（list marker / 替换内容 / 直属文本 /
    /// progress-meter 填充条）。R5265 自 paint_node_inner 原位抽出：collapse 驱动
    /// cell 重定位到带补发射之后发射（CSS2 App E：inline 内容在带之上）；
    /// 非驱动节点原位调用，行为不变。
    pub(crate) fn paint_inline_content_phase(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        styles: &HashMap<NodeId, ComputedStyle>,
        doc: Option<&Document>,
    ) {
        // R2251 content-visibility:hidden → 直属文本跳过绘制（元素自身盒装饰仍绘）。
        // kill-switch `ZW_CONTENT_VISIBILITY`，default-on。
        // R4058：UA display:inline 的**替换元素**（svg 根等）是 atomic inline——
        // content-visibility:hidden 对其生效（css-contain-2 §5）。
        let self_replaced = box_node.node_id.is_some_and(|id| {
            doc.is_some_and(|d| {
                d.get(id).is_some_and(|n| {
                    matches!(
                        &n.kind,
                        NodeKind::Element(e)
                            if matches!(
                                e.local_name(),
                                "img" | "canvas" | "video" | "embed" | "object" | "applet" | "iframe" | "svg"
                            )
                    )
                })
            })
        });
        let content_hidden = std::env::var("ZW_CONTENT_VISIBILITY").as_deref() != Ok("0")
            && (style.content_visibility_hidden_effective()
                || (self_replaced && matches!(style.content_visibility, ContentVisibilityValue::Hidden)));

        // CSS2 §visufx #visibility：hidden 容器**自身**盒装饰/markers/替换内容隐藏，
        // 但后代可用 visibility:visible 重置继承、仍须绘制——扁平 inline 后代（无独立
        // 盒）的片段由 paint_text 发射、经 R5098 owner 过滤器逐片段裁决（owner hidden →
        // 片段灭；owner visible → 片段绘）。旧实现在此整体早退，hidden 容器的可见重置
        // 后代一并灭绘（driving: CSS2/visufx/visibility-005——div[hidden] 内 span[visible]
        // 绿 X 全白）。kill-switch `ZW_INLINE_FRAG_VISIBILITY=0` 时回退旧整体早退（过滤
        // 器同灭，防泄漏）。content-visibility:hidden 按包含语义整树跳过，不在此列。
        if matches!(style.visibility, VisibilityValue::Hidden | VisibilityValue::Collapse) {
            if self.inline_frag_visibility_enabled && !content_hidden {
                self.paint_text(box_node, abs_x, abs_y, style, doc, Some(styles));
            }
            return;
        }

        // 4. 列表标记绘制（bullets/numbers，位于文本之前）
        if let Some(doc) = doc {
            self.paint_list_marker(box_node, abs_x, abs_y, style, doc);

            // R1686：<summary> disclosure 标记（▶闭合/▼开启），≡ list marker 同位。
            self.paint_summary_marker(box_node, abs_x, abs_y, style, doc);
        }

        // 4b/4b2/4b4/4c. 替换内容 + 表单控件值 + CSS content（与脏矩形走查共用）。
        self.paint_element_specials_before_text(box_node, abs_x, abs_y, style, styles, doc, content_hidden);

        // 5. 文本内容绘制（含 text-shadow，使用行内格式化上下文处理换行）
        // R2251 content-visibility:hidden → 直属文本跳过（元素盒装饰仍绘）。
        if !content_hidden {
            self.paint_text(box_node, abs_x, abs_y, style, doc, Some(styles));
        }

        // 5b3. <progress>/<meter> value 填充条（须在 paint_text 之后盖 fallback）。
        self.paint_element_specials_after_text(box_node, abs_x, abs_y, style, doc);
    }

    /// R793/R4541/R5155/R5201：overflow 子树图元裁剪到 padding box（含 shaped
    /// overflow 多边形改写、contain:paint / overflow 圆角剔除）。R5265 自
    /// paint_node_inner 主循环抽出：collapse 驱动 cell 的 sweep A 段同样属
    /// R792 内容域，需要同款裁剪。
    pub(crate) fn clip_overflow_children(
        &mut self,
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
        abs_x: f32,
        abs_y: f32,
        doc: Option<&Document>,
        counts: &PrimitiveCounts,
    ) {
        let needs_clip = super::compute_needs_clip(box_node, styles, doc);
        if !needs_clip || box_node.is_multicol_region_fragment || self.svg_filter_canvas_skip(box_node) {
            return;
        }
        // R793：CSS §11.1.1 — overflow 裁剪到 **padding box**（内容 + padding，
        // border 之内），非 content box。R2500：overflow:clip 轴按
        // overflow-clip-margin 扩展（CSS Overflow 3 §3）。
        let clip_rect = super::overflow_clip_rect(box_node, abs_x, abs_y);
        super::super::helpers::clip_all_primitives_to_rect(&mut self.primitives, counts, &clip_rect);
        // R4541（css-borders-4 §border-shape）：border-shape 声明时 overflow 内容
        // 裁剪到形状内缘。覆盖填充改写 + v2 条带裁剪与 clip-path polygon 臂共用
        // 同一入口。R4547（corner-shaping）：形角非初始态时裁到形角化 border-box
        // 轮廓。kill-switch `ZW_CORNER_SHAPE_OVERFLOW=0`（回退纯矩形裁剪）。
        let shaped_overflow = box_node.node_id.and_then(|node_id| {
            styles.get(&node_id).and_then(|style| {
                super::super::helpers::border_shape_overflow_polygon(style, box_node, abs_x, abs_y).or_else(|| {
                    if std::env::var("ZW_CORNER_SHAPE_OVERFLOW").as_deref() == Ok("0") {
                        return None;
                    }
                    // R5179（css-overflow-4 §3）：overflow:clip 轴按 overflow-clip-margin
                    // 外扩后套形角几何（半径不变）。R5183：半径百分比相对元素 border-box。
                    let mx = if box_node.overflow_x == OverflowClip::Clip {
                        box_node.overflow_clip_margin_length
                    } else {
                        0.0
                    };
                    let my = if box_node.overflow_y == OverflowClip::Clip {
                        box_node.overflow_clip_margin_length
                    } else {
                        0.0
                    };
                    super::super::helpers::shaped_corner_polygon_in_box(
                        style,
                        abs_x - mx,
                        abs_y - my,
                        box_node.width + 2.0 * mx,
                        box_node.height + 2.0 * my,
                        crate::paint::helpers::shaped_radius_spec(style, box_node.width, box_node.height),
                    )
                })
            })
        });
        if let Some(polygon) = shaped_overflow {
            super::super::helpers::clip_with_polygon_rewrite(&mut self.primitives, counts, &polygon);
        }
        // R5155：contain:paint 圆角裁剪——图片内容窗口追加四角圆角剔除。
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
            && let Some(radii) = super::paint_containment_clip_radii(box_node, style)
        {
            super::super::helpers::clip_images_to_rounded_rect(&mut self.primitives, counts, &clip_rect, radii);
        }
        // R5201：overflow 裁剪圆角——触及裁剪角的 fill 改写圆角矩形（仅 fill 面）。
        if let Some(node_id) = box_node.node_id
            && let Some(style) = styles.get(&node_id)
            && let Some(radii) = super::overflow_clip_radii(box_node, style, &clip_rect, abs_x, abs_y)
        {
            super::super::helpers::clip_fills_to_rounded_rect(&mut self.primitives, counts, &clip_rect, radii);
        }
    }
}

/// R5265：collapse sweep 主循环共用 in-flow 子集（与常规子循环同谓词：
/// positioned/transform-defer/OOF/multicol 片段排除——paint_node_inner 调用方
/// 预计算后传入，保证 sweep 集 ≡ 常规集）。
pub(crate) fn collapse_sweep_in_flow(
    box_node: &LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    paint_as_multicol: bool,
    defer_abspos: bool,
    transform_phase_defer: bool,
) -> Vec<usize> {
    ordered_child_indices(&box_node.children, |child| {
        (!paint_as_multicol || child.column_span_offsets.is_empty())
            && (!defer_abspos || (!child.is_absolute && !child.is_fixed))
            && !is_positioned_child(child)
            // R5176：变换子改由 scope 的 step 6 flush 施绘（App E step 8 组）。
            && !(transform_phase_defer && box_paints_in_transform_phase(child, styles))
    })
}
