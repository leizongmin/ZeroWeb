//! Layout — 两遍布局（measure 自下而上 / arrange 自上而下，P0-2 拆分）。
//!
//! 内置容器（[`ContainerKind`]=Column/Row/Stack/ScrollVertical）走 host 级策略，
//! 叶子节点交给 widget 自带的 `Widget::layout`。

use zero_ui_core::binding::{PropsMap, Value};
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::prop_keys;
use zero_ui_core::widget::LayoutCtx;

use super::{ContainerKind, HostNode};

/// 从 props 读 `gap`（Column/Row 主轴间距）。接受 Int/Float，缺省 0。
fn gap_from_props(props: &PropsMap) -> f32 {
    match props.get(prop_keys::GAP) {
        Some(Value::Float(f)) => *f as f32,
        Some(Value::Int(i)) => *i as f32,
        _ => 0.0,
    }
}

/// 节点的容器布局种类：优先读 `props.layout`（`"column"`/`"row"`/`"stack"`，大小写不敏感），
/// 否则按组件名识别内置容器（`Column`/`Row`/`Stack`）。
///
/// 让任意组件名（如 `browser.DesktopBrowserShell`、`browser.ToolbarRow`）经 props 声明为容器，
/// 无需 host 硬编码 chrome/业务组件名 —— 保持 host 浏览器无关。
pub(super) fn node_container_kind(node: &HostNode) -> Option<ContainerKind> {
    if let Some(Value::Text(s)) = node.props.get(prop_keys::LAYOUT) {
        match s.as_str() {
            "column" | "Column" => return Some(ContainerKind::Column),
            "row" | "Row" => return Some(ContainerKind::Row),
            "stack" | "Stack" => return Some(ContainerKind::Stack),
            "scroll" | "scroll_vertical" | "ScrollVertical" | "ScrollView" => {
                return Some(ContainerKind::ScrollVertical);
            }
            _ => {}
        }
    }
    // DC-16 向后兼容：gallery 当前用 `props["scroll"] = "vertical"`（在 Column 上）标记滚动。
    // 识别此写法时升级为 ScrollVertical 容器（measure/arrange 走独立分支）。
    if is_scroll_vertical(&node.props) {
        return Some(ContainerKind::ScrollVertical);
    }
    ContainerKind::from_component(&node.component)
}

/// 是否为垂直滚动容器（DC-16 gallery scroll，向后兼容 gallery 现有 `scroll=vertical` 写法）。
fn is_scroll_vertical(props: &PropsMap) -> bool {
    matches!(props.get(prop_keys::SCROLL), Some(Value::Text(s)) if s == "vertical")
}

/// 从 props 读 `flex`（Row/Column 主轴弹性权重）。接受 Int/Float，缺省/负值 → 0。
fn flex_from_props(props: &PropsMap) -> f32 {
    match props.get(prop_keys::FLEX) {
        Some(Value::Float(f)) => (*f as f32).max(0.0),
        Some(Value::Int(i)) => (*i as f32).max(0.0),
        _ => 0.0,
    }
}

/// 从 props 提取浮点值（`Float` 或 `Int`），缺省返回 `default`。
fn float_from_props(props: &PropsMap, key: &str, default: f32) -> f32 {
    match props.get(key) {
        Some(Value::Float(f)) => *f as f32,
        Some(Value::Int(i)) => *i as f32,
        _ => default,
    }
}

/// 从子节点 props 读取 min/max 约束（缺省：min = 0, max = f32::MAX，即不约束）。
fn child_constraints_from_props(props: &PropsMap) -> (f32, f32, f32, f32) {
    let min_w = float_from_props(props, prop_keys::MIN_WIDTH, 0.0).max(0.0);
    let max_w = float_from_props(props, prop_keys::MAX_WIDTH, f32::MAX).max(min_w);
    let min_h = float_from_props(props, prop_keys::MIN_HEIGHT, 0.0).max(0.0);
    let max_h = float_from_props(props, prop_keys::MAX_HEIGHT, f32::MAX).max(min_h);
    (min_w, max_w, min_h, max_h)
}

/// 把尺寸钳到 `(min_w, max_w, min_h, max_h)` 范围内。
fn clamp_size(s: Size, min_w: f32, max_w: f32, min_h: f32, max_h: f32) -> Size {
    Size::new(s.width.clamp(min_w, max_w), s.height.clamp(min_h, max_h))
}

/// 从 props 读交叉轴对齐（`cross_axis_align`：`"start"`/`"center"`/`"end"`，大小写不敏感；
/// Row 也接受 `"top"`/`"bottom"`、Column 也接受 `"left"`/`"right"`）。
///
/// 缺省 [`CrossAxisAlignment::Start`]（向后兼容历史顶/左对齐行为）。
fn cross_axis_alignment_from_props(props: &PropsMap) -> CrossAxisAlignment {
    if let Some(Value::Text(s)) = props.get(prop_keys::CROSS_AXIS_ALIGN) {
        match s.to_ascii_lowercase().as_str() {
            "center" => return CrossAxisAlignment::Center,
            "end" | "bottom" | "right" => return CrossAxisAlignment::End,
            "start" | "top" | "left" => return CrossAxisAlignment::Start,
            _ => {}
        }
    }
    CrossAxisAlignment::Start
}

/// 从 props 读主轴对齐（`main_axis_align`，大小写不敏感；`-`/`_`/无分隔均可）：
/// `"start"` / `"center"` / `"end"` / `"space_between"` / `"space_around"` / `"space_evenly"`
/// （Row 也接受 `"left"`/`"right"`、Column 也接受 `"top"`/`"bottom"`）。
///
/// 缺省 [`MainAxisAlignment::Start`]（向后兼容历史左/顶打包行为）。需容器主轴有剩余空间才生效
/// （fill-sizing 或父 tight/exact 约束）；弹性子节点消费剩余空间时主轴对齐无可见效果。
fn main_axis_alignment_from_props(props: &PropsMap) -> MainAxisAlignment {
    if let Some(Value::Text(s)) = props.get(prop_keys::MAIN_AXIS_ALIGN) {
        let norm: String = s
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_ascii_lowercase();
        match norm.as_str() {
            "center" => return MainAxisAlignment::Center,
            "end" | "right" | "bottom" => return MainAxisAlignment::End,
            "spacebetween" => return MainAxisAlignment::SpaceBetween,
            "spacearound" => return MainAxisAlignment::SpaceAround,
            "spaceevenly" => return MainAxisAlignment::SpaceEvenly,
            "start" | "left" | "top" => return MainAxisAlignment::Start,
            _ => {}
        }
    }
    MainAxisAlignment::Start
}

/// 线性容器（Row/Column）的主轴方向。
#[derive(Clone, Copy, PartialEq)]
enum MainAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum CrossAxisAlignment {
    #[default]
    Start,
    Center,
    End,
}

/// 计算子节点在交叉轴上的偏移量（相对交叉轴起点）。
fn cross_offset(align: CrossAxisAlignment, container_cross: f32, child_cross: f32) -> f32 {
    let free = (container_cross - child_cross).max(0.0);
    match align {
        CrossAxisAlignment::Start => 0.0,
        CrossAxisAlignment::Center => free * 0.5,
        CrossAxisAlignment::End => free,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum MainAxisAlignment {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// 主轴分布：返回 `(起始偏移, 子节点间额外间距)`（CSS `justify-content` 语义）。
fn main_axis_layout(align: MainAxisAlignment, extra: f32, n: usize) -> (f32, f32) {
    if extra <= 0.0 || n == 0 {
        return (0.0, 0.0);
    }
    let nf = n as f32;
    match align {
        MainAxisAlignment::Start => (0.0, 0.0),
        MainAxisAlignment::Center => (extra * 0.5, 0.0),
        MainAxisAlignment::End => (extra, 0.0),
        MainAxisAlignment::SpaceBetween => {
            if n > 1 {
                (0.0, extra / (nf - 1.0))
            } else {
                (0.0, 0.0)
            }
        }
        MainAxisAlignment::SpaceAround => (extra / (2.0 * nf), extra / nf),
        MainAxisAlignment::SpaceEvenly => {
            let per = extra / (nf + 1.0);
            (per, per)
        }
    }
}

/// Row/Column 共用的弹性布局：两遍 measure。
fn measure_linear(node: &mut HostNode, lctx: &mut LayoutCtx, constraints: Constraints, axis: MainAxis) -> Size {
    let gap = gap_from_props(&node.props);
    let n = node.children.len();
    let (min_main, max_main, min_cross, max_cross) = match axis {
        MainAxis::Horizontal => (
            constraints.min_width,
            constraints.max_width,
            constraints.min_height,
            constraints.max_height,
        ),
        MainAxis::Vertical => (
            constraints.min_height,
            constraints.max_height,
            constraints.min_width,
            constraints.max_width,
        ),
    };
    let mut child_main = vec![0.0_f32; n];
    let mut child_cross = vec![0.0_f32; n];

    let flexes: Vec<f32> = node.children.iter().map(|c| flex_from_props(&c.props)).collect();
    let total_flex: f32 = flexes.iter().sum();

    // Pass 1：非弹性子节点。
    let mut cursor = 0.0_f32;
    let scroll_overflow =
        axis == MainAxis::Vertical && node_container_kind(node) == Some(ContainerKind::ScrollVertical);
    for i in 0..n {
        if flexes[i] > 0.0 {
            continue;
        }
        let (min_w, max_w, min_h, max_h) = child_constraints_from_props(&node.children[i].props);
        let remaining = if scroll_overflow {
            f32::MAX
        } else {
            (max_main - cursor).max(0.0)
        };
        let child_c = match axis {
            MainAxis::Horizontal => Constraints {
                min_width: 0.0,
                max_width: remaining,
                min_height: 0.0,
                max_height: max_cross,
            },
            MainAxis::Vertical => Constraints {
                min_width: 0.0,
                max_width: max_cross,
                min_height: 0.0,
                max_height: remaining,
            },
        };
        let s = clamp_size(
            measure(&mut node.children[i], lctx, child_c),
            min_w,
            max_w,
            min_h,
            max_h,
        );
        let (m, c) = match axis {
            MainAxis::Horizontal => (s.width, s.height),
            MainAxis::Vertical => (s.height, s.width),
        };
        child_main[i] = m;
        child_cross[i] = c;
        cursor += m + gap;
    }

    // Pass 2：弹性子节点。
    let gaps_total = gap * n.saturating_sub(1) as f32;
    let used_nonflex: f32 = child_main
        .iter()
        .enumerate()
        .filter(|(i, _)| flexes[*i] <= 0.0)
        .map(|(_, m)| *m)
        .sum();
    if total_flex > 0.0 {
        let free = (max_main - gaps_total - used_nonflex).max(0.0);
        for i in 0..n {
            if flexes[i] <= 0.0 {
                continue;
            }
            let share = free * (flexes[i] / total_flex);
            let (min_w, max_w, min_h, max_h) = child_constraints_from_props(&node.children[i].props);
            let child_c = match axis {
                MainAxis::Horizontal => Constraints {
                    min_width: share.max(min_w),
                    max_width: share.max(min_w).min(max_w),
                    min_height: 0.0,
                    max_height: max_cross,
                },
                MainAxis::Vertical => Constraints {
                    min_width: 0.0,
                    max_width: max_cross,
                    min_height: share.max(min_w),
                    max_height: share.max(min_w).min(max_h),
                },
            };
            let s = clamp_size(
                measure(&mut node.children[i], lctx, child_c),
                min_w,
                max_w,
                min_h,
                max_h,
            );
            let (m, c) = match axis {
                MainAxis::Horizontal => (s.width, s.height),
                MainAxis::Vertical => (s.height, s.width),
            };
            child_main[i] = m;
            child_cross[i] = c;
        }
    }

    let content_main = (child_main.iter().sum::<f32>() + gaps_total).max(0.0);
    let total_main = content_main.max(min_main).min(max_main);
    let content_cross = child_cross.iter().copied().fold(0.0_f32, f32::max).max(0.0);
    let total_cross = content_cross.max(min_cross).min(max_cross);
    match axis {
        MainAxis::Horizontal => Size::new(total_main, total_cross),
        MainAxis::Vertical => Size::new(total_cross, total_main),
    }
}

/// measure：自下而上算每节点尺寸，写入 `cached_size`，返回本节点尺寸。
pub(super) fn measure(node: &mut HostNode, lctx: &mut LayoutCtx, constraints: Constraints) -> Size {
    let size = match node_container_kind(node) {
        Some(ContainerKind::Column) => measure_linear(node, lctx, constraints, MainAxis::Vertical),
        Some(ContainerKind::Row) => measure_linear(node, lctx, constraints, MainAxis::Horizontal),
        Some(ContainerKind::ScrollVertical) => measure_linear(node, lctx, constraints, MainAxis::Vertical),
        Some(ContainerKind::Stack) => {
            let mut max_w = 0.0_f32;
            let mut max_h = 0.0_f32;
            for child in node.children.iter_mut() {
                let s = measure(
                    child,
                    lctx,
                    Constraints::loose(Size::new(constraints.max_width, constraints.max_height)),
                );
                max_w = max_w.max(s.width);
                max_h = max_h.max(s.height);
            }
            Size::new(max_w.min(constraints.max_width), max_h.min(constraints.max_height))
        }
        None => {
            if let Some(w) = node.widget.as_mut() {
                w.layout(lctx, constraints)
            } else {
                Size::new(constraints.max_width, constraints.max_height)
            }
        }
    };
    node.cached_size = size;
    size
}

/// arrange：自上而下按 `cached_size` 定每节点绝对 `cached_rect`。
pub(super) fn arrange(node: &mut HostNode, origin: Point) {
    node.cached_rect = Rect::from_origin_size(origin, node.cached_size);
    match node_container_kind(node) {
        Some(ContainerKind::Column) => arrange_linear(node, origin, Axis::Vertical),
        Some(ContainerKind::ScrollVertical) => arrange_linear(node, origin, Axis::VerticalScroll),
        Some(ContainerKind::Row) => arrange_linear(node, origin, Axis::Horizontal),
        Some(ContainerKind::Stack) | None => {
            for child in node.children.iter_mut() {
                arrange(child, origin);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
    VerticalScroll,
}

fn arrange_linear(node: &mut HostNode, origin: Point, axis: Axis) {
    let gap = gap_from_props(&node.props);
    let cross = cross_axis_alignment_from_props(&node.props);
    let main = main_axis_alignment_from_props(&node.props);
    let horizontal = matches!(axis, Axis::Horizontal);
    let scroll = matches!(axis, Axis::VerticalScroll);

    let (container_main, container_cross) = if horizontal {
        (node.cached_size.width, node.cached_size.height)
    } else {
        (node.cached_size.height, node.cached_size.width)
    };

    let n = node.children.len();
    let child_mains: Vec<f32> = node
        .children
        .iter()
        .map(|c| {
            if horizontal {
                c.cached_size.width
            } else {
                c.cached_size.height
            }
        })
        .collect();
    let content: f32 = child_mains.iter().sum();
    let gaps_min = gap * n.saturating_sub(1) as f32;

    if scroll {
        node.content_height = content + gaps_min;
    }

    let extra = (container_main - content - gaps_min).max(0.0);
    let (main_offset, between_extra) = main_axis_layout(main, extra, n);
    let spacing = gap + between_extra;
    let start = origin_main(origin, horizontal) + main_offset - if scroll { node.scroll_offset } else { 0.0 };
    let mut cursor = start;

    for (i, child) in node.children.iter_mut().enumerate() {
        let child_cross = if horizontal {
            child.cached_size.height
        } else {
            child.cached_size.width
        };
        let cross_o = cross_offset(cross, container_cross, child_cross);
        let main_o = cursor;
        let p = if horizontal {
            Point::new(main_o, origin_cross(origin, horizontal) + cross_o)
        } else {
            Point::new(origin_cross(origin, horizontal) + cross_o, main_o)
        };
        arrange(child, p);
        cursor += child_mains[i] + spacing;
    }
    let _ = n;
}

#[inline]
fn origin_main(origin: Point, horizontal: bool) -> f32 {
    if horizontal { origin.x } else { origin.y }
}

#[inline]
fn origin_cross(origin: Point, horizontal: bool) -> f32 {
    if horizontal { origin.y } else { origin.x }
}
