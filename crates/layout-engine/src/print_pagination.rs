//! Print 媒体分页 post-process（R1999 Phase P1a）。
//!
//! 当 `media_type == Print` 时，把文档主块流的直接 in-flow block 子按固定页高
//!（page box）分页，使 `page-break-before: always`（CSS2）/ `break-before: page`（CSS3）
//! 强制换页生效——Ctrl+P 打印预览显示分页内容（Print 模式此前仅套 @media print CSS、不分页）。
//!
//! # 相对定位分页模型（解决 R1998 缺口）
//!
//! `LayoutBox.y` 相对父内容区（types/mod.rs:30），故「遍历加偏移」会破坏兄弟相对顺序。
//! 本模块把分片**边界化到单一层级**（块流根的直接 in-flow block 子）：
//! - **sibling shift = gap 累加**：每个子 `child.y += accumulated_gap`，forced break 算出 gap
//!   把元素推到下一页边界；后续兄弟 gap 累加跟随。
//! - **后代自动跟随**：后代 y 相对子内容区；子整体下移（改 child.y）后后代相对结构不变。
//! - **无跨祖先问题**：不分片嵌套层（嵌套精确断 = P2，绝对坐标 remap）。
//!
//! 详见 `docs/goal/rendering-compat/print-layout-phase-p1-spec.md`。
//!
//! # 范围（P1a-M1）
//!
//! - ✅ FR-001 gate（调用方 engine.rs 判 media_type==Print + env `ZW_PRINT_PAGINATE=1`）
//! - ✅ FR-002 `page-break-before: always` / `break-before: page` 强制换页（own style）
//! - ✅ FR-005 默认页尺寸常量（A4 @96dpi）
//! - ⏳ FR-003 `page-break-after` / FR-004 自然页填充 / FR-006 嵌套提升 = P1a-M2
//!
//! # 输出模型
//!
//! 分页后子元素 abs 底部下移 → `layout_extent_y`（pipeline.rs:668）自动返回更大文档高 →
//! `paint_cull_viewport` 产出 taller cull rect → tall-framebuffer（页边界以空白间隔可见）。
//! 无需手动扩 body.height（子溢出 body box 但 extent 仍捕获）。

use std::collections::HashMap;

use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::{BreakValue, PageBreakValue};

use crate::types::LayoutBox;

/// A4 页高 @96dpi（297mm = 11.6929in × 96 ≈ 1122.5px）。P4 `@page { size }` 解析前的默认。
pub const PRINT_PAGE_HEIGHT_A4: f32 = 1122.5;

/// env kill-switch（首切片 default-off）：`ZW_PRINT_PAGINATE=1` 启用 Print 分页。
/// gate 由 engine.rs 组合 `media_type == Print && print_paginate_enabled()`。
pub fn print_paginate_enabled() -> bool {
    std::env::var("ZW_PRINT_PAGINATE").as_deref() == Ok("1")
}

/// 对 `root` 执行 Print 分页 post-process。
///
/// 下降到块流根（body 近似：含多个 in-flow block 子的容器），对其直接 in-flow block 子
/// 按 `page_height` 分页：forced break-before 的子被推到下一页边界，后续兄弟累加 gap 跟随。
/// Screen 路径不调用本函数（engine.rs gate）。
pub fn paginate_for_print(root: &mut LayoutBox, page_height: f32, styles: &HashMap<NodeId, ComputedStyle>) {
    if page_height <= 0.0 {
        return;
    }
    // Pass 1（不可变）：下降到块流根 + 累积其内容原点 abs y。
    let (path, flow_content_abs) = match find_flow_container_path(root) {
        Some(v) => v,
        None => return,
    };
    // Pass 2（可变）：沿 path 到达块流根，分页其子。
    let flow = path
        .iter()
        .fold(root as &mut LayoutBox, |node, &i| &mut node.children[i]);
    paginate_flow_children(flow, flow_content_abs, page_height, styles);
}

/// 下降路径 + 块流根内容原点 abs y。
///
/// 启发式：从 root 沿「唯一 in-flow block 子」下降，直到遇到含 **多个** in-flow block 子的
/// 容器（= 内容层，通常 body）。probe 构造 root=body（多子）时立即返回 root。生产 root=html
/// 时下降 html→body。TBD-1（body 缺失/结构异常）回退：0 in-flow 子时返回当前节点（无分页）。
fn find_flow_container_path(root: &LayoutBox) -> Option<(Vec<usize>, f32)> {
    let mut path: Vec<usize> = Vec::new();
    // root 内容原点 abs y：root 无父，parent_content_abs = 0。
    let mut content_abs = content_origin_abs(root, 0.0);
    let mut cur = root;
    loop {
        let in_flow = in_flow_block_indices(cur);
        if in_flow.len() > 1 {
            // cur = 块流根；其内容原点 = content_abs。
            return Some((path, content_abs));
        }
        match in_flow.into_iter().next() {
            Some(i) => {
                let child = &cur.children[i];
                content_abs = content_origin_abs(child, content_abs);
                path.push(i);
                cur = child;
            }
            None => {
                // 0 in-flow 子：当前节点作块流根（无子可分页，no-op）。
                return Some((path, content_abs));
            }
        }
    }
}

/// 节点内容盒原点 abs y（子 y 相对此点）。
///
/// `border_box_top = parent_content_abs + node.y + node.margin_top`；
/// `content_box_top = border_box_top + node.border_top + node.padding_top`。
fn content_origin_abs(node: &LayoutBox, parent_content_abs: f32) -> f32 {
    let border_box_top = parent_content_abs + node.y + node.margin_top;
    border_box_top + node.border_top + node.padding_top
}

/// 直接 in-flow block 子的索引（排除 abspos/fixed）。
fn in_flow_block_indices(node: &LayoutBox) -> Vec<usize> {
    node.children
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.is_absolute && !c.is_fixed)
        .map(|(i, _)| i)
        .collect()
}

/// 对块流根的直接 in-flow block 子施加分页 gap。
fn paginate_flow_children(
    flow: &mut LayoutBox,
    flow_content_abs: f32,
    page_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    let mut gap = 0.0f32;
    for child in flow.children.iter_mut() {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        if has_forced_break_before(child, styles) {
            // 子当前 abs border-box 顶（含已累积 gap）。
            let cur_abs = flow_content_abs + child.y + gap + child.margin_top;
            // 仅当当前页已有前导内容时才强制换页（镜像 multicol `current_col_height > 0`
            // 守卫）：子已在页顶（cur_abs % page_height ≈ 0）时不换页，避免首页被推空。
            let within_page = (cur_abs % page_height).abs();
            if within_page > 0.5 {
                let target = next_page_boundary(cur_abs, page_height);
                let extra = target - cur_abs;
                if extra > 0.0 {
                    gap += extra;
                }
            }
        }
        // 累积 gap 下移此子（相对块流根内容区；后代因相对此子而自动跟随）。
        child.y += gap;
    }
}

/// 元素自身声明 forced break-before（CSS2 `page-break-before: always|left|right` 或
/// CSS3 `break-before: page`）。M1 仅查自身样式（FR-006 子树提升 = M2）。
fn has_forced_break_before(child: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    child.node_id.is_some_and(|id| {
        styles.get(&id).is_some_and(|s| {
            matches!(s.break_before, BreakValue::Page)
                || matches!(
                    s.page_break_before,
                    PageBreakValue::Always | PageBreakValue::Left | PageBreakValue::Right
                )
        })
    })
}

/// >= cur_abs 的最小页边界（k * page_height）。
fn next_page_boundary(cur_abs: f32, page_height: f32) -> f32 {
    (cur_abs / page_height).ceil() * page_height
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_dom::Document;
    use zero_style_system::property::types::{BreakValue, PageBreakValue};

    /// 构造一个最小 ComputedStyle（仅设 break 字段，其余 default）。
    fn style_with_break_before(break_before: BreakValue, page_break_before: PageBreakValue) -> ComputedStyle {
        let mut s = ComputedStyle::default();
        s.break_before = break_before;
        s.page_break_before = page_break_before;
        s
    }

    /// 构造块流根（root=body，内容原点 abs=0）+ 3 个 in-flow block 子 [A, B*, C]。
    /// A: y=0 h=100；B: y=100 h=50（可设 break-before）；C: y=150 h=30。
    fn body_with_three_children(
        doc: &mut Document,
        b_style: ComputedStyle,
    ) -> (LayoutBox, HashMap<NodeId, ComputedStyle>) {
        let id_a = doc.create_element("div");
        let id_b = doc.create_element("div");
        let id_c = doc.create_element("div");
        let id_root = doc.create_element("div");
        let mut styles = HashMap::new();
        styles.insert(id_b, b_style);
        let a = LayoutBox {
            node_id: Some(id_a),
            y: 0.0,
            height: 100.0,
            ..Default::default()
        };
        let b = LayoutBox {
            node_id: Some(id_b),
            y: 100.0,
            height: 50.0,
            ..Default::default()
        };
        let c = LayoutBox {
            node_id: Some(id_c),
            y: 150.0,
            height: 30.0,
            ..Default::default()
        };
        let root = LayoutBox {
            node_id: Some(id_root),
            y: 0.0,
            children: vec![a, b, c],
            ..Default::default()
        };
        (root, styles)
    }

    #[test]
    fn r1999_print_paginate_forced_break_pushes_to_page_boundary() {
        // FR-002：B(page-break-before:always) 推到下一页顶 (y=H)；C 跟随到 H+50；A 不动。
        let mut doc = Document::new();
        let (mut root, styles) = body_with_three_children(
            &mut doc,
            style_with_break_before(BreakValue::Auto, PageBreakValue::Always),
        );
        let h = 1000.0;
        paginate_for_print(&mut root, h, &styles);
        let a = &root.children[0];
        let b = &root.children[1];
        let c = &root.children[2];
        // A 在第1页原位（A 先处理、无 break → gap=0 → A.y 不变）。
        assert_eq!(a.y, 0.0, "A 应留在第1页原位 y=0");
        // B 推到第2页顶 = H。
        assert_eq!(b.y, h, "B(page-break-before) 应被推到页边界 y=H={}", h);
        // C 跟随 B（原 y=150 + gap(H-100=900) = 1050 = H+50）。
        assert_eq!(c.y, h + 50.0, "C 应跟随 B 到 H+50");
    }

    #[test]
    fn r1999_print_paginate_break_before_page_equivalent() {
        // FR-002 CSS3 变体：break-before:page 等价 page-break-before:always。
        let mut doc = Document::new();
        let (mut root, styles) = body_with_three_children(
            &mut doc,
            style_with_break_before(BreakValue::Page, PageBreakValue::Auto),
        );
        let h = 1000.0;
        paginate_for_print(&mut root, h, &styles);
        assert_eq!(root.children[1].y, h, "break-before:page 应等价推到 H");
    }

    #[test]
    fn r1999_print_paginate_no_forced_break_preserves_order() {
        // 无 forced break：单元顺序与原位置一致（gap 恒 0）。
        let mut doc = Document::new();
        let (mut root, styles) = body_with_three_children(
            &mut doc,
            style_with_break_before(BreakValue::Auto, PageBreakValue::Auto),
        );
        let h = 1000.0;
        paginate_for_print(&mut root, h, &styles);
        assert_eq!(root.children[0].y, 0.0);
        assert_eq!(root.children[1].y, 100.0);
        assert_eq!(root.children[2].y, 150.0);
    }

    #[test]
    fn r1999_print_paginate_killswitch_disables_pass() {
        // NFR-002：env ZW_PRINT_PAGINATE != "1" 时 print_paginate_enabled() 返 false（default-off）。
        // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。
        unsafe {
            std::env::remove_var("ZW_PRINT_PAGINATE");
            assert!(!print_paginate_enabled(), "未设 env 时须 default-off");
            std::env::set_var("ZW_PRINT_PAGINATE", "0");
            assert!(!print_paginate_enabled(), "=0 时须关闭");
            std::env::set_var("ZW_PRINT_PAGINATE", "1");
            assert!(print_paginate_enabled(), "=1 时须启用");
            std::env::remove_var("ZW_PRINT_PAGINATE");
        }
    }

    #[test]
    fn r1999_print_paginate_default_page_height_a4() {
        // FR-005：默认页高常量 = A4 @96dpi ≈ 1122.5。
        assert!((PRINT_PAGE_HEIGHT_A4 - 1122.5).abs() < 0.01, "A4 页高须 ≈ 1122.5px");
    }

    #[test]
    fn r1999_print_paginate_descendants_auto_follow() {
        // 核心不变量：shift 子 y 后，后代（相对子内容区）自动跟随——后代相对 y 不变，
        // 但后代 abs 位置随子整体下移。证「相对定位 sibling-shift 边界化到单层」成立。
        let mut doc = Document::new();
        let id_a = doc.create_element("div");
        let id_b = doc.create_element("div");
        let id_d = doc.create_element("div");
        let id_root = doc.create_element("div");
        let mut styles = HashMap::new();
        styles.insert(id_b, style_with_break_before(BreakValue::Auto, PageBreakValue::Always));
        // B 含后代 D（相对 B 内容区 y=10）；B 无 margin/border/padding → 内容原点 = B.y。
        let d = LayoutBox {
            node_id: Some(id_d),
            y: 10.0,
            height: 5.0,
            ..Default::default()
        };
        let b = LayoutBox {
            node_id: Some(id_b),
            y: 100.0,
            height: 50.0,
            children: vec![d],
            ..Default::default()
        };
        let a = LayoutBox {
            node_id: Some(id_a),
            y: 0.0,
            height: 100.0,
            ..Default::default()
        };
        let root = LayoutBox {
            node_id: Some(id_root),
            y: 0.0,
            children: vec![a, b],
            ..Default::default()
        };
        let mut root = root;
        let h = 1000.0;
        paginate_for_print(&mut root, h, &styles);
        let b = &root.children[1];
        let d = &b.children[0];
        assert_eq!(b.y, h, "B 推到 H");
        assert_eq!(d.y, 10.0, "后代 D 相对 B 的 y 不变（=10）");
        // D 的 abs 顶 = B 内容原点 + D.y = H + 10。
        assert!((b.y + d.y - (h + 10.0)).abs() < 0.01, "D abs 顶随 B 整体下移到 H+10");
    }
}
