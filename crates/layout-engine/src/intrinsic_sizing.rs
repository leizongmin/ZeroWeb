//! 容器固有宽度（intrinsic / max-content）测量工具。
//!
//! 为 flex/grid 容器两趟固有宽度布局（见 `docs/goal/rendering-compat/flex-grid-two-pass-design.md`）
//! 提供测量基础。本模块**不参与布局**（compute() 不调用其改变布局的函数），
//! 仅提供纯计算函数 + 单元测试 + 可选诊断打印（env-gated），分轮渐进接线。
//!
//! 关键区别于 `table_shrink::block_max_content_width`：本模块的 `box_content_max_width`
//! 对「叶 block 显式 width」回退到自身显式宽度（R138 的函数对此返回 0，会漏测
//! `<div style="width:30px">` 这类叶盒），故 grid item 的固有宽度才能正确测量。

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::FlexBasisValue;

use crate::types::LayoutBox;

/// 计算一个盒的「内容最大宽度」（max-content）。
///
/// 递归规则（CSS intrinsic sizing）：
/// - inline 级子元素（含 inline-block）→ 水平求和（max-content 假设不换行）
/// - block 级子元素 → 取最大者的内容宽度
/// - **叶盒（无有效子元素贡献）且有显式 Px width → 回退到自身显式 width**
///   （这是与 `table_shrink::block_max_content_width` 的关键差异）
///
/// 返回值含 box 自身的水平 padding+border（border-box 贡献）。
fn box_content_max_width(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> f32 {
    let mut inline_sum = 0.0f32;
    let mut block_max = 0.0f32;
    let mut has_in_flow_child = false;

    for child in &box_node.children {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        has_in_flow_child = true;
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
        let outer_w = child.width + child.margin_left + child.margin_right;
        if is_inline_level {
            inline_sum += outer_w.max(0.0);
        } else {
            block_max = block_max.max(box_content_max_width(child, styles));
        }
    }

    let children_inner = inline_sum.max(block_max);

    // 叶盒回退：无有效子元素贡献时，用自身显式 Px width（content-box 语义）。
    // 显式 width 的叶盒（如 `<div style="width:50px">`）其 max-content 即该宽度。
    let own_explicit = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.width {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let inner = if !has_in_flow_child || children_inner < own_explicit {
        own_explicit
    } else {
        children_inner
    };

    inner + box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right
}

/// 计算 flex item 的主轴 base size（CSS Flexbox §9.2 flex base size）。
///
/// 优先级：`flex-basis` 显式长度 > `width` 显式长度 > 内容 max-content。
/// - `flex-basis: auto`/`content` → 回退到 width 或内容
/// - 无法确定（无显式值且内容为 0）→ 返回 0.0（调用方应作 no-op 处理）
///
/// 返回 border-box 贡献（含 item 自身 padding+border，不含 margin——margin 由容器求和时加）。
fn flex_item_base_size(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> f32 {
    let style = box_node.node_id.and_then(|id| styles.get(&id));
    // 1. flex-basis 显式长度优先
    if let Some(s) = style
        && let FlexBasisValue::Length(len) = &s.flex_basis
    {
        if let LengthValue::Px(v) = len {
            let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
            return (*v as f32) + frame;
        }
    }
    // 2. width 显式长度
    if let Some(s) = style
        && let LengthValue::Px(v) = &s.width
    {
        let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
        return (*v as f32) + frame;
    }
    // 3. 内容 max-content
    box_content_max_width(box_node, styles)
}

/// 计算一个**水平 flex 行容器**的固有宽度（max-content 主尺寸）。
///
/// = Σ flex item base size + item margins + gaps + 容器水平 padding/border。
/// 仅对 `display:flex`/`inline-flex` 且主轴为水平（flex-direction: row/row-reverse）的容器有意义。
/// 返回 None 表示无法确定（如无流内 item）。
pub(crate) fn flex_row_intrinsic_width(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> Option<f32> {
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
            sum += flex_item_base_size(child, styles) + child.margin_left + child.margin_right;
        }
    }
    if count == 0 {
        return None;
    }
    let gap = box_node
        .node_id
        .and_then(|id| styles.get(&id))
        .and_then(|s| match &s.gap {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    Some(sum + gap * (count - 1) as f32 + frame)
}

/// 计算一个 **grid 容器**的固有宽度（max-content 主尺寸）。
///
/// 近似实现（taffy 0.7 无原生 grid auto-track 扩展，此处用 item base size 估算）：
/// - `grid-auto-flow: column`（item 水平排列）→ Σ item base size + gaps
/// - 其它（默认 row，item 垂直堆叠）→ max item base size
///
/// 其中 item base size = `box_content_max_width`（含叶显式宽回退，故 `.item > .content(50px)`
/// 会测为 50+frame）。返回 None 表示无流内 item。
pub(crate) fn grid_intrinsic_width(box_node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> Option<f32> {
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
        .and_then(|s| match &s.column_gap {
            LengthValue::Px(v) => Some(*v as f32),
            _ => None,
        })
        .unwrap_or(0.0);
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
            let base = box_content_max_width(child, styles) + child.margin_left + child.margin_right;
            sum += base;
            max_w = max_w.max(base);
        }
    }
    if count == 0 {
        return None;
    }
    let frame = box_node.padding_left + box_node.padding_right + box_node.border_left + box_node.border_right;
    let inner = if is_column_flow {
        sum + gap * (count - 1) as f32
    } else {
        max_w
    };
    Some(inner + frame)
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
pub(crate) fn debug_dump_shrink_candidates(root: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
    fn walk(b: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) {
        let Some(id) = b.node_id else {
            for c in &b.children {
                walk(c, styles);
            }
            return;
        };
        let Some(s) = styles.get(&id) else {
            for c in &b.children {
                walk(c, styles);
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
                    grid_intrinsic_width(b, styles)
                } else {
                    flex_row_intrinsic_width(b, styles)
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
            walk(c, styles);
        }
    }
    walk(root, styles);
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
        flex_row_intrinsic_width(target, &styles)
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
        grid_intrinsic_width(target, &styles)
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
}
