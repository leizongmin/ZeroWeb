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
//! # 范围（P1a-M2）
//!
//! - ✅ FR-001 gate（调用方 engine.rs 判 media_type==Print + env `ZW_PRINT_PAGINATE` default-on）
//! - ✅ FR-002 `page-break-before: always` / `break-before: page` 强制换页
//! - ✅ FR-003 `page-break-after: always` / `break-after: page`（下一兄弟换页）
//! - ✅ FR-004 自然页填充（子越页边界且整页装得下→推下页顶；oversized 留原位 deferred）
//! - ✅ FR-006 嵌套 break 子树提升（后代 break-before→整单元换页，P1a 近似）
//! - ✅ FR-005 页尺寸 + 垂直页边距（default A4 + 0 边距；R2010 P4 `@page { size }` + R2011 `@page { margin }` 解析后由 pipeline 注入覆盖；margin_top/bottom 驱动分页内容区）
//! - ✅ R2013 layout-width-for-print（Print 模式根布局 containing block 宽 = 页内容盒宽 `print_content_width`；@page size 宽度 + 水平 margin 完整生效；default A4 宽）
//! - ⏳ oversized 单元真分片（fragment_y_offset 拆多页）/ P2 嵌套精确断 / P3 inside:avoid / P5 输出模型
//!
//! # 输出模型
//!
//! 分页后子元素 abs 底部下移 → `layout_extent_y`（pipeline.rs:748）自动返回更大文档高 →
//! `paint_cull_viewport` 产出 taller cull rect → tall-framebuffer（页边界以空白间隔可见）。
//! 无需手动扩 body.height（子溢出 body box 但 extent 仍捕获）。

use std::collections::HashMap;

use zero_dom::NodeId;
use zero_style_system::ComputedStyle;
use zero_style_system::property::types::{BreakValue, PageBreakValue};

use crate::types::LayoutBox;

/// A4 页高 @96dpi（297mm = 11.6929in × 96 ≈ 1122.5px）。P4 `@page { size }` 解析前的默认。
pub const PRINT_PAGE_HEIGHT_A4: f32 = 1122.5;

/// A4 页宽 @96dpi（210mm = 8.2677in × 96 ≈ 793.7px）。R2013 layout-width-for-print：
/// Print 模式根布局宽默认 = A4 内容宽（与页高默认 A4 一致），`@page { size }` 解析后由 pipeline 注入覆盖。
pub const PRINT_PAGE_WIDTH_A4: f32 = 210.0 / 25.4 * 96.0;

/// Print 页内容盒宽（px）= 页宽 − 水平边距。R2013：Print 模式根布局 containing block 宽。
///
/// 退化守卫：若水平边距吃掉绝大部分页宽（usable ≤ 1px），回退页宽本身（边距归零），
/// 避免负可用宽致布局塌缩（镜像 `paginate_for_print` 的垂直 usable 守卫）。
pub fn print_content_width(page_width: f32, margin_left: f32, margin_right: f32) -> f32 {
    let usable = page_width - margin_left - margin_right;
    if usable < 1.0 { page_width.max(0.0) } else { usable }
}

/// 一页的几何信息（R2014 Phase P5a page-sequence 元数据）。
///
/// `physical_top/bottom` = 物理页框边界（k×page_height … (k+1)×page_height）；
/// `content_top/bottom` = 页内容盒边界（physical ± 垂直 margin），即该页内容可占据的 abs y 区间。
/// 物理边界供页边界分隔线（`inject_print_page_dividers`）+ 未来 P5b per-page clip；
/// 内容盒边界供 P5b 内容裁剪 + 未来 fragmentation。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrintPage {
    /// 页索引（0-based）。
    pub index: usize,
    /// 物理页框顶 abs y（= index × page_height）。
    pub physical_top: f32,
    /// 物理页框底 abs y（= (index + 1) × page_height）。
    pub physical_bottom: f32,
    /// 页内容盒顶 abs y（= physical_top + margin_top）。
    pub content_top: f32,
    /// 页内容盒底 abs y（= physical_bottom − margin_bottom）。
    pub content_bottom: f32,
}

/// 计算打印页序列（R2014 Phase P5a FR-P5-001）。
///
/// 从文档布局 `layout_extent`（abs y 跨度）+ 页尺寸 + 垂直边距算页序列：
/// `page_count = ceil(extent / page_height)`，每页含物理边界 + 内容盒边界。
/// 与 `inject_print_page_dividers`（分隔线）/ `paginate_for_print`（分页）同源页边界——
/// 单一真相，避免三处页边界计算发散。退化守卫镜像 `paginate_for_print`：
/// 垂直边距吃光页高（usable ≤ 1px）→ 边距归零。
///
/// 返回至少 1 页（空文档 extent=0 → 1 页）。
pub fn compute_print_page_sequence(
    layout_extent: f32,
    page_height: f32,
    margin_top: f32,
    margin_bottom: f32,
) -> Vec<PrintPage> {
    let page_height = page_height.max(1.0);
    // 退化守卫：边距吃光页高 → 边距归零（镜像 paginate_for_print usable 守卫）。
    let usable = page_height - margin_top - margin_bottom;
    let (mt, mb) = if usable < 1.0 {
        (0.0, 0.0)
    } else {
        (margin_top.max(0.0), margin_bottom.max(0.0))
    };
    let extent = layout_extent.max(0.0);
    let page_count = (extent / page_height).ceil() as usize;
    (0..page_count.max(1))
        .map(|k| {
            let physical_top = k as f32 * page_height;
            let physical_bottom = (k as f32 + 1.0) * page_height;
            PrintPage {
                index: k,
                physical_top,
                physical_bottom,
                content_top: physical_top + mt,
                content_bottom: physical_bottom - mb,
            }
        })
        .collect()
}

/// env kill-switch（R2000 default-on）：Print 分页默认启用；`ZW_PRINT_PAGINATE=0` 紧急关闭。
/// gate 由 engine.rs 组合 `media_type == Print && print_paginate_enabled()`（Screen 永不触发）。
/// default-on 依据：11 单元测试 + 端到端真实 HTML 管线测试（R2000）证分页正确 + Screen 零影响。
pub fn print_paginate_enabled() -> bool {
    std::env::var("ZW_PRINT_PAGINATE").as_deref() != Ok("0")
}

/// 对 `root` 执行 Print 分页 post-process。
///
/// 下降到块流根（body 近似：含多个 in-flow block 子的容器），对其直接 in-flow block 子
/// 按 `page_height` 分页：forced break-before 的子被推到下一页边界，后续兄弟累加 gap 跟随。
/// Screen 路径不调用本函数（engine.rs gate）。
pub fn paginate_for_print(
    root: &mut LayoutBox,
    page_height: f32,
    margin_top: f32,
    margin_bottom: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    if page_height <= 0.0 {
        return;
    }
    // R2011：退化为 0 边距若垂直边距吃掉绝大部分页高（usable ≤ 1px）——避免负可用高死循环。
    let usable = page_height - margin_top - margin_bottom;
    let (margin_top, margin_bottom) = if usable < 1.0 {
        (0.0, 0.0)
    } else {
        (margin_top.max(0.0), margin_bottom.max(0.0))
    };
    // Pass 1（不可变）：下降到块流根 + 累积其内容原点 abs y。
    let (path, flow_content_abs) = match find_flow_container_path(root) {
        Some(v) => v,
        None => return,
    };
    // Pass 2（可变）：沿 path 到达块流根，分页其子。
    let flow = path
        .iter()
        .fold(root as &mut LayoutBox, |node, &i| &mut node.children[i]);
    paginate_flow_children(flow, flow_content_abs, page_height, margin_top, margin_bottom, styles);
}

/// 下降路径 + 块流根内容原点 abs y。
///
/// 启发式：从 root 沿「唯一 **node_id-bearing** in-flow 子」下降（跳过匿名包装盒 node_id=None），
/// 直到遇到含 **多个** node_id-bearing in-flow 子的容器（= 内容层，通常 body）。
/// probe 构造 root=body（多子）时立即返回 root。生产 layout root 常为 `[anon, html/body]`
///（anon 无 node_id）→ 过滤 anon 后下降到真实内容层。0 真实子时返回当前节点（无分页）。
fn find_flow_container_path(root: &LayoutBox) -> Option<(Vec<usize>, f32)> {
    let mut path: Vec<usize> = Vec::new();
    // root 内容原点 abs y：root 无父，parent_content_abs = 0。
    let mut content_abs = content_origin_abs(root, 0.0);
    let mut cur = root;
    loop {
        let real = real_in_flow_child_indices(cur);
        if real.len() > 1 {
            // cur = 块流根；其内容原点 = content_abs。
            return Some((path, content_abs));
        }
        match real.into_iter().next() {
            Some(i) => {
                let child = &cur.children[i];
                content_abs = content_origin_abs(child, content_abs);
                path.push(i);
                cur = child;
            }
            None => {
                // 0 真实子：当前节点作块流根（无子可分页，no-op）。
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

/// 直接 in-flow 且 **有 node_id**（非匿名包装盒）的子索引——用于下降决策。
///
/// layout root 常含匿名包装盒（node_id=None，h=0），若计入会让启发式停在 root 而非下降到
/// 真实内容层（body）。仅数 node_id-bearing 子确保下降穿过匿名包装层。
fn real_in_flow_child_indices(node: &LayoutBox) -> Vec<usize> {
    node.children
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.is_absolute && !c.is_fixed && c.node_id.is_some())
        .map(|(i, _)| i)
        .collect()
}

/// 对块流根的直接 in-flow block 子施加分页 gap（P1a-M2：forced before/after + 自然填充）。
///
/// 三类换页触发（皆仅在「当前页已有前导内容」即子非页顶时生效，避免首页被推空）：
/// - **FR-002 forced before**：子自身或后代声明 `break-before:page` / `page-break-before:always`
///   （FR-006 后代 break 经子树扫描**提升**到整单元）→ 推到下一页边界。
/// - **FR-003 break-after**：上一兄弟声明 `break-after:page` / `page-break-after:always`
///   → 当前子推到下一页边界（`pending_break_after` 标志传递）。
/// - **FR-004 自然填充**：子放不下当前页剩余空间且**整页装得下**（outer_h ≤ page_height）
///   → 推到下一页顶。**oversized**（outer_h > page_height）M2 不分片，留原位（overflow），deferred。
fn paginate_flow_children(
    flow: &mut LayoutBox,
    flow_content_abs: f32,
    page_height: f32,
    margin_top: f32,
    margin_bottom: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    // R2011：初始 gap 把页 0 内容对齐到 `margin_top`（页内容盒顶）。若块流根已超过 margin_top
    //（flow_content_abs > margin_top，罕见）则不前推。后续页的内容顶 = k×page_height + margin_top
    // 由下方 forced/FR-004 推页时 +margin_top 保证。
    let mut gap = (margin_top - flow_content_abs).max(0.0);
    let mut pending_break_after = false;
    for child in flow.children.iter_mut() {
        if child.is_absolute || child.is_fixed {
            continue;
        }
        // 子当前 abs border-box 顶（含已累积 gap）。
        let top_abs = flow_content_abs + child.y + gap + child.margin_top;
        let outer_h = child.height + child.margin_top + child.margin_bottom;
        // 镜像 multicol `current_col_height > 0` 守卫：子在页内容顶（(top_abs - margin_top) % page_height ≈ 0）不换页。
        let at_page_top = ((top_abs - margin_top).max(0.0) % page_height).abs() < 0.5;

        let forced = has_forced_break_before(child, styles) || pending_break_after;
        if forced {
            if !at_page_top {
                // 推到下一页内容顶 = 下一物理页边界 + margin_top。
                let target = next_page_boundary(top_abs, page_height) + margin_top;
                let extra = target - top_abs;
                if extra > 0.0 {
                    gap += extra;
                }
            }
        } else if !at_page_top && outer_h <= page_height + 0.5 {
            // FR-004 自然填充：整页装得下但放不下当前页内容区剩余空间 → 推到下一页内容顶。
            let bottom_abs = top_abs + child.height + child.margin_bottom;
            let phys_bottom = page_bottom_for(top_abs, page_height);
            let content_bottom = phys_bottom - margin_bottom;
            if bottom_abs > content_bottom + 0.5 {
                gap += phys_bottom + margin_top - top_abs;
            }
        }
        // 累积 gap 下移此子（相对块流根内容区；后代因相对此子而自动跟随）。
        child.y += gap;
        // FR-003：记录此子 break-after 供下一兄弟换页。
        pending_break_after = has_forced_break_after(child, styles);
    }
}

/// 元素声明 forced break-before（FR-006：自身 **或** 后代子树，后代 break 提升到整单元）。
fn has_forced_break_before(child: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    if let Some(id) = child.node_id {
        if let Some(s) = styles.get(&id) {
            if matches!(s.break_before, BreakValue::Page)
                || matches!(
                    s.page_break_before,
                    PageBreakValue::Always | PageBreakValue::Left | PageBreakValue::Right
                )
            {
                return true;
            }
        }
    }
    // FR-006 子树提升：任一后代声明 forced break-before → 整单元换页（P1a 近似，P2 才精确断）。
    subtree_has_forced_break_before(child, styles)
}

/// 元素自身声明 forced break-after（CSS2 `page-break-after: always|left|right` 或
/// CSS3 `break-after: page`）。break-after 不做子树提升（after 是元素之后的断，后代无关）。
fn has_forced_break_after(child: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    child.node_id.is_some_and(|id| {
        styles.get(&id).is_some_and(|s| {
            matches!(s.break_after, BreakValue::Page)
                || matches!(
                    s.page_break_after,
                    PageBreakValue::Always | PageBreakValue::Left | PageBreakValue::Right
                )
        })
    })
}

/// 子树（不含 node 自身）是否含 forced break-before（FR-006 提升）。
fn subtree_has_forced_break_before(node: &LayoutBox, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    for c in &node.children {
        if c.is_absolute || c.is_fixed {
            continue;
        }
        if let Some(id) = c.node_id {
            if let Some(s) = styles.get(&id) {
                if matches!(s.break_before, BreakValue::Page)
                    || matches!(
                        s.page_break_before,
                        PageBreakValue::Always | PageBreakValue::Left | PageBreakValue::Right
                    )
                {
                    return true;
                }
            }
        }
        if subtree_has_forced_break_before(c, styles) {
            return true;
        }
    }
    false
}

/// 包含 `top_abs` 的页的底部边界（= 下一页顶）。`top_abs` 在页顶时返该页底（非自身）。
fn page_bottom_for(top_abs: f32, page_height: f32) -> f32 {
    let page_idx = (top_abs / page_height).floor();
    (page_idx + 1.0) * page_height
}

/// >= cur_abs 的最小页边界（k * page_height）。用于 forced 推页（mid-page 时与 page_bottom_for 同）。
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
        paginate_for_print(&mut root, h, 0.0, 0.0, &styles);
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
        paginate_for_print(&mut root, h, 0.0, 0.0, &styles);
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
        paginate_for_print(&mut root, h, 0.0, 0.0, &styles);
        assert_eq!(root.children[0].y, 0.0);
        assert_eq!(root.children[1].y, 100.0);
        assert_eq!(root.children[2].y, 150.0);
    }

    #[test]
    fn r1999_print_paginate_killswitch_disables_pass() {
        // NFR-002：R2000 default-on——未设/="0" 时 print_paginate_enabled() 返 true；="0" 关闭。
        // Rust 2024：set_var/remove_var 为 unsafe（进程全局可变）。
        unsafe {
            std::env::remove_var("ZW_PRINT_PAGINATE");
            assert!(print_paginate_enabled(), "未设 env 时须 default-on");
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
        paginate_for_print(&mut root, h, 0.0, 0.0, &styles);
        let b = &root.children[1];
        let d = &b.children[0];
        assert_eq!(b.y, h, "B 推到 H");
        assert_eq!(d.y, 10.0, "后代 D 相对 B 的 y 不变（=10）");
        // D 的 abs 顶 = B 内容原点 + D.y = H + 10。
        assert!((b.y + d.y - (h + 10.0)).abs() < 0.01, "D abs 顶随 B 整体下移到 H+10");
    }

    /// 构造一个声明 forced break-after 的 ComputedStyle（CSS2 page-break-after:always）。
    fn style_with_break_after() -> ComputedStyle {
        let mut s = ComputedStyle::default();
        s.page_break_after = PageBreakValue::Always;
        s
    }

    #[test]
    fn r2000_print_paginate_break_after_pushes_next_sibling() {
        // FR-003：A(page-break-after:always) 之后强制换页 → 下一兄弟 B 起于新页顶 (y=H)。
        let mut doc = Document::new();
        let id_a = doc.create_element("div");
        let id_b = doc.create_element("div");
        let id_root = doc.create_element("div");
        let mut styles = HashMap::new();
        styles.insert(id_a, style_with_break_after());
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
        let mut root = LayoutBox {
            node_id: Some(id_root),
            y: 0.0,
            children: vec![a, b],
            ..Default::default()
        };
        let h = 1000.0;
        paginate_for_print(&mut root, h, 0.0, 0.0, &styles);
        assert_eq!(root.children[0].y, 0.0, "A 留在第1页原位");
        assert_eq!(root.children[1].y, h, "B 因 A 的 break-after 被推到 H");
    }

    #[test]
    fn r2000_print_paginate_natural_fill_overflows_to_next_page() {
        // FR-004：无 forced break，但 B 放不下当前页剩余空间 → 自然推到下一页顶。
        // A h=950（占第1页 0..950），B h=100 起于 y=950 → 底 1050 越页边界 1000 → 推到 1000。
        let mut doc = Document::new();
        let id_a = doc.create_element("div");
        let id_b = doc.create_element("div");
        let id_root = doc.create_element("div");
        let styles = HashMap::new();
        let a = LayoutBox {
            node_id: Some(id_a),
            y: 0.0,
            height: 950.0,
            ..Default::default()
        };
        let b = LayoutBox {
            node_id: Some(id_b),
            y: 950.0,
            height: 100.0,
            ..Default::default()
        };
        let mut root = LayoutBox {
            node_id: Some(id_root),
            y: 0.0,
            children: vec![a, b],
            ..Default::default()
        };
        let h = 1000.0;
        paginate_for_print(&mut root, h, 0.0, 0.0, &styles);
        assert_eq!(root.children[0].y, 0.0, "A 留第1页");
        assert_eq!(root.children[1].y, h, "B 自然填充推到下一页顶 H（原 950 + gap 50）");
    }

    #[test]
    fn r2000_print_paginate_nested_break_promoted_to_top_unit() {
        // FR-006：section 自身无 break，但其后代 h1 声明 break-before:page →
        // 整个 section 单元被提升换页（推到下一页顶）。P1a 近似（P2 才精确断在 h1）。
        let mut doc = Document::new();
        let id_a = doc.create_element("div");
        let id_section = doc.create_element("section");
        let id_h1 = doc.create_element("h1");
        let id_c = doc.create_element("div");
        let id_root = doc.create_element("div");
        let mut styles = HashMap::new();
        // h1 后代声明 break-before:page（CSS3）。
        let mut h1_style = ComputedStyle::default();
        h1_style.break_before = BreakValue::Page;
        styles.insert(id_h1, h1_style);
        let h1 = LayoutBox {
            node_id: Some(id_h1),
            y: 0.0,
            height: 20.0,
            ..Default::default()
        };
        let section = LayoutBox {
            node_id: Some(id_section),
            y: 100.0,
            height: 50.0,
            children: vec![h1],
            ..Default::default()
        };
        let a = LayoutBox {
            node_id: Some(id_a),
            y: 0.0,
            height: 100.0,
            ..Default::default()
        };
        let c = LayoutBox {
            node_id: Some(id_c),
            y: 200.0,
            height: 30.0,
            ..Default::default()
        };
        let mut root = LayoutBox {
            node_id: Some(id_root),
            y: 0.0,
            children: vec![a, section, c],
            ..Default::default()
        };
        let h = 1000.0;
        paginate_for_print(&mut root, h, 0.0, 0.0, &styles);
        assert_eq!(root.children[0].y, 0.0, "A 留第1页");
        assert_eq!(root.children[1].y, h, "section 因后代 h1 break 提升到 H");
        assert_eq!(
            root.children[2].y,
            h + 100.0,
            "C 跟随 section（原 200 + gap 900 = H+100）"
        );
    }

    #[test]
    fn r2000_print_paginate_oversized_left_in_place() {
        // FR-004 oversized（outer_h > page_height）：M2 不分片，留原位（overflow），deferred。
        // B h=100（页顶），A h=1500（> page_h=1000）起于 y=100 → 不自然填充（整页装不下）→ 留原位。
        let mut doc = Document::new();
        let id_b = doc.create_element("div");
        let id_a = doc.create_element("div");
        let id_root = doc.create_element("div");
        let styles = HashMap::new();
        let b = LayoutBox {
            node_id: Some(id_b),
            y: 0.0,
            height: 100.0,
            ..Default::default()
        };
        let a = LayoutBox {
            node_id: Some(id_a),
            y: 100.0,
            height: 1500.0,
            ..Default::default()
        };
        let mut root = LayoutBox {
            node_id: Some(id_root),
            y: 0.0,
            children: vec![b, a],
            ..Default::default()
        };
        let h = 1000.0;
        paginate_for_print(&mut root, h, 0.0, 0.0, &styles);
        assert_eq!(root.children[0].y, 0.0, "B 留页顶");
        assert_eq!(root.children[1].y, 100.0, "oversized A 留原位（未分片，deferred）");
    }

    #[test]
    fn r2000_print_paginate_descends_html_to_body() {
        // 真实文档形（root=html[margin_top=8] → body → [A, B*, C]）：验证 find_flow_container
        // 下降启发式落地到 body（html 单子→下降，body 多子→停）+ content_origin_abs 累积
        // html margin（8）使 B 的 abs 顶精确落页边界 1000（若 origin 算错会落 992）。
        let mut doc = Document::new();
        let id_html = doc.create_element("html");
        let id_body = doc.create_element("body");
        let id_a = doc.create_element("div");
        let id_b = doc.create_element("div");
        let id_c = doc.create_element("div");
        let mut styles = HashMap::new();
        styles.insert(id_b, style_with_break_before(BreakValue::Auto, PageBreakValue::Always));
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
        let body = LayoutBox {
            node_id: Some(id_body),
            y: 0.0,
            children: vec![a, b, c],
            ..Default::default()
        };
        // html margin_top=8 模拟 UA 默认 margin；html 单 in-flow 子 body。
        let mut html = LayoutBox {
            node_id: Some(id_html),
            y: 0.0,
            margin_top: 8.0,
            children: vec![body],
            ..Default::default()
        };
        let h = 1000.0;
        paginate_for_print(&mut html, h, 0.0, 0.0, &styles);
        let body = &html.children[0];
        let a = &body.children[0];
        let b = &body.children[1];
        assert_eq!(a.y, 0.0, "A 留原位");
        // B 的 abs 顶须 == 页边界 1000（content_origin 累积 html margin=8 正确）。
        // abs = body_content_abs(8) + B.y + 0 = 8 + B.y == 1000 → B.y == 992。
        assert_eq!(8.0 + b.y, h, "B abs 顶须精确落页边界 H=1000（验证 html margin 累积）");
    }

    /// R2011：`margin_top` 把页 0 内容从 y=0 下移到 margin_top（页内容盒顶）。
    /// flow 容器须 ≥2 node_id 子才被 find_flow_container 识别（单子会下降到叶）。
    #[test]
    fn r2011_print_paginate_margin_top_offsets_page_zero_content() {
        let mut doc = Document::new();
        let id_a = doc.create_element("div");
        let id_b = doc.create_element("div");
        let id_root = doc.create_element("div");
        let styles = HashMap::new();
        let a = LayoutBox {
            node_id: Some(id_a),
            y: 0.0,
            height: 50.0,
            ..Default::default()
        };
        let b = LayoutBox {
            node_id: Some(id_b),
            y: 50.0,
            height: 50.0,
            ..Default::default()
        };
        let mut root = LayoutBox {
            node_id: Some(id_root),
            y: 0.0,
            children: vec![a, b],
            ..Default::default()
        };
        // page_h=1000, margin_top=100 → 页 0 内容起于 y=100；A、B 均下移 100。
        paginate_for_print(&mut root, 1000.0, 100.0, 0.0, &styles);
        assert_eq!(root.children[0].y, 100.0, "A 下移到 margin_top=100");
        assert_eq!(root.children[1].y, 150.0, "B 同步下移 100（50+100）");
    }

    /// R2011：内容超出页内容区（[margin_top, page_h - margin_bottom]）→ 推到下一页内容顶
    /// （phys_boundary + margin_top），非物理页边界本身。B 不在页顶（A 占位）触发 FR-004。
    #[test]
    fn r2011_print_paginate_margin_overflow_lands_at_next_page_content_top() {
        let mut doc = Document::new();
        let id_a = doc.create_element("div");
        let id_b = doc.create_element("div");
        let id_root = doc.create_element("div");
        let styles = HashMap::new();
        // page_h=1000, margin_top=100, margin_bottom=100 → 内容区 [100, 900]（usable 800）。
        // A(h=50) 在页顶；B(y=50, h=760) abs 顶=150，底=910 > 内容底 900 → 推到下一页内容顶 1100。
        let a = LayoutBox {
            node_id: Some(id_a),
            y: 0.0,
            height: 50.0,
            ..Default::default()
        };
        let b = LayoutBox {
            node_id: Some(id_b),
            y: 50.0,
            height: 760.0,
            ..Default::default()
        };
        let mut root = LayoutBox {
            node_id: Some(id_root),
            y: 0.0,
            children: vec![a, b],
            ..Default::default()
        };
        paginate_for_print(&mut root, 1000.0, 100.0, 100.0, &styles);
        assert_eq!(
            root.children[1].y, 1100.0,
            "B 底 910 > 内容底 900 → 推到下一页内容顶 = page_h + margin_top = 1100"
        );
    }

    /// R2011：margin 0（默认）= 旧行为零回归——内容留原位，按物理页边界。
    #[test]
    fn r2011_print_paginate_zero_margin_matches_legacy_behavior() {
        let mut doc = Document::new();
        let id_a = doc.create_element("div");
        let id_b = doc.create_element("div");
        let id_root = doc.create_element("div");
        let styles = HashMap::new();
        let a = LayoutBox {
            node_id: Some(id_a),
            y: 0.0,
            height: 50.0,
            ..Default::default()
        };
        let b = LayoutBox {
            node_id: Some(id_b),
            y: 50.0,
            height: 50.0,
            ..Default::default()
        };
        let mut root = LayoutBox {
            node_id: Some(id_root),
            y: 0.0,
            children: vec![a, b],
            ..Default::default()
        };
        paginate_for_print(&mut root, 1000.0, 0.0, 0.0, &styles);
        assert_eq!(root.children[0].y, 0.0, "margin=0 → A 留 y=0（旧行为）");
        assert_eq!(root.children[1].y, 50.0, "margin=0 → B 留 y=50（旧行为）");
    }

    /// R2013：`PRINT_PAGE_WIDTH_A4` = 210mm @96dpi ≈ 793.7px（与 `PRINT_PAGE_HEIGHT_A4` 同源）。
    #[test]
    fn r2013_print_page_width_a4_is_210mm_at_96dpi() {
        let expected = 210.0 / 25.4 * 96.0;
        assert!(
            (PRINT_PAGE_WIDTH_A4 - expected).abs() < 0.01,
            "A4 width {expected}, got {PRINT_PAGE_WIDTH_A4}"
        );
    }

    /// R2013：`print_content_width` = 页宽 − 水平边距。正常区间。
    #[test]
    fn r2013_print_content_width_subtracts_horizontal_margins() {
        // A4 (793.7) − 1in(96) left − 1in(96) right = 601.7
        let w = print_content_width(PRINT_PAGE_WIDTH_A4, 96.0, 96.0);
        let expected = PRINT_PAGE_WIDTH_A4 - 192.0;
        assert!((w - expected).abs() < 0.01, "content width {expected}, got {w}");
    }

    /// R2013：退化守卫——水平边距吃掉绝大部分页宽（usable ≤ 1px）→ 回退页宽本身（边距归零），
    /// 避免负可用宽致布局塌缩（镜像垂直 usable 守卫）。
    #[test]
    fn r2013_print_content_width_degenerate_falls_back_to_page_width() {
        // 页宽 100，左右边距各 60 → usable = -20 < 1 → 回退页宽 100。
        let w = print_content_width(100.0, 60.0, 60.0);
        assert!((w - 100.0).abs() < 0.01, "degenerate → page width 100, got {w}");
        // 边界：usable 恰 = 1 不触发回退（>1 才正常减）。
        let wb = print_content_width(101.0, 50.0, 50.0);
        assert!((wb - 1.0).abs() < 0.01, "usable=1 boundary → 1.0, got {wb}");
    }

    /// R2018 P5a：页序列按 ceil(extent/page_h) 计页数 + 物理边界 = k×page_h。
    #[test]
    fn r2018_page_sequence_extent_to_page_count() {
        // extent 2500 / page_h 1122.5 → ceil = 3 页。
        let pages = compute_print_page_sequence(2500.0, 1122.5, 0.0, 0.0);
        assert_eq!(pages.len(), 3, "extent 2500 / 1122.5 → 3 pages");
        assert_eq!(pages[0].index, 0);
        assert!((pages[0].physical_top - 0.0).abs() < 0.1);
        assert!((pages[1].physical_top - 1122.5).abs() < 0.1, "page 1 physical_top");
        assert!((pages[2].physical_top - 2245.0).abs() < 0.1, "page 2 physical_top");
        // 无边距时 content == physical。
        assert_eq!(pages[1].content_top, pages[1].physical_top);
        assert_eq!(pages[1].content_bottom, pages[1].physical_bottom);
    }

    /// R2018 P5a：垂直边距驱动内容盒边界（content_top = physical_top + mt，content_bottom = physical_bottom − mb）。
    #[test]
    fn r2018_page_sequence_content_box_uses_margins() {
        // page_h 1000, mt 100, mb 80 → 内容盒高 820，content_top = k×1000 + 100。
        let pages = compute_print_page_sequence(2500.0, 1000.0, 100.0, 80.0);
        assert_eq!(pages.len(), 3);
        assert!(
            (pages[1].content_top - 1100.0).abs() < 0.1,
            "page 1 content_top = 1000+100"
        );
        assert!(
            (pages[1].content_bottom - 1920.0).abs() < 0.1,
            "page 1 content_bottom = 2000-80"
        );
        // 物理边界不受 margin 影响。
        assert!((pages[1].physical_top - 1000.0).abs() < 0.1);
        assert!((pages[1].physical_bottom - 2000.0).abs() < 0.1);
    }

    /// R2018 P5a：退化守卫——垂直边距吃光页高（usable ≤ 1）→ 边距归零（content == physical）。
    #[test]
    fn r2018_page_sequence_degenerate_margins_zeroed() {
        // page_h 100, mt 60, mb 60 → usable = -20 < 1 → 边距归零。
        let pages = compute_print_page_sequence(250.0, 100.0, 60.0, 60.0);
        assert_eq!(
            pages[1].content_top, pages[1].physical_top,
            "degenerate → margins zeroed"
        );
        assert_eq!(pages[1].content_bottom, pages[1].physical_bottom);
    }

    /// R2018 P5a：空文档（extent ≤ 0）至少 1 页。
    #[test]
    fn r2018_page_sequence_empty_doc_one_page() {
        let pages = compute_print_page_sequence(0.0, 1122.5, 0.0, 0.0);
        assert_eq!(pages.len(), 1, "empty doc → 1 page");
        assert_eq!(pages[0].index, 0);
    }
}
