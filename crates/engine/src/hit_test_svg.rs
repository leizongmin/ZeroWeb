//! SVG 形状子元素屏幕矩形合成 — gBCR rect 快照的 svg 形状注入（M2-S201）。
//!
//! 布局树只为 `<svg>` 建替换盒（usvg/resvg 整体栅格化，[`crate::svg`] 渲染路径），
//! 形状子元素（`rect`/`circle`/`ellipse`…）的盒是**退化体**（探针实证高度 0，几何不
//! 参与布局）——`fill_rect_from_layout_box` 的 rect 快照（gBCR 数据源）因此对形状
//! 不可用，runner 侧最小面积命中扫描与 `elementFromPoint` 在形状上回落 `<svg>` 本体
//! （inert-svg-hittest 第 2 子测根因，探针实证 `hits=[svg,wrapper,body]`）。
//!
//! 本模块在 rect 快照构建期从 **DOM 形状属性 + viewBox 映射**合成形状屏幕矩形：
//! 纯快照补齐（gBCR/命中扫描数据源），不改布局树、不建盒、不影响绘制。
//! 命中面与 gBCR 面同源纪律不变（形状矩形 = 合成 gBCR 面）。
//!
//! 建模面（浏览器对齐最小集，按语料需求扩）：
//! - `viewBox` meet 映射 + `preserveAspectRatio` 缺省 `xMidYMid`（居中）；
//!   无/非法 viewBox 时用户单位 = CSS 像素 1:1。
//! - `rect`（含缺省 x/y=0）、`circle`、`ellipse` 的几何包围盒（命中面按包围盒，
//!   与 HTML 盒级命中模型一致；SVG 几何级点拾取挂账）。
//!
//! 未建模（跳过，不产生矩形）：嵌套 `<svg>` 子树（viewport 需级联映射）、
//! `path`/`polygon`/`polyline`/`line`/`text`/`use`/`foreignObject` 等几何。
//! 百分比/`em` 等非纯数值属性解析失败按缺省 0 处理（rect 宽高 ≤0 即跳过）。
//!
//! https://www.w3.org/TR/SVG11/coords.html#ViewBoxAttribute
//! https://www.w3.org/TR/SVG11/coords.html#PreserveAspectRatioAttribute
//! https://www.w3.org/TR/SVG11/shapes.html

use std::collections::HashMap;

use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::LayoutBox;

use crate::hit_test::{child_origin, node_id_to_u64};

/// 形状子树下探深度上限（防御病态/环状 DOM，正常文档远达不到）。
const SHAPE_WALK_MAX_DEPTH: usize = 64;

/// 遍历布局树（绝对坐标累计语义与 `fill_rect_from_layout_box` 一致），对每个
/// `<svg>` 元素盒调用 [`svg_shape_screen_rects`]，把形状屏幕矩形并入 `out`。
///
/// 形状布局盒是退化体（高度 0，几何不参与布局），调用方（`HitTestCache` rect
/// 快照填充）须让本表**覆盖**布局值。
pub(crate) fn svg_shape_rects_from_layout(
    doc: &Document,
    box_node: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
    out: &mut HashMap<u64, crate::rect_bridge::Rect4>,
) {
    // inline 上报 y/h 与 fill_rect_from_layout_box 同源（svg 替换盒通常无上报值）。
    let (reported_y, reported_h) = box_node.inline_reported_rect.unwrap_or((box_node.y, box_node.height));
    let box_x = abs_x + box_node.x;
    let box_y = abs_y + reported_y;
    if box_node.width > 0.0
        && reported_h > 0.0
        && let Some(id) = box_node.node_id
        && is_svg_element(doc, id)
    {
        for (shape_id, rect) in svg_shape_screen_rects(doc, id, box_x, box_y, box_node.width, reported_h) {
            out.entry(node_id_to_u64(shape_id)).or_insert(rect);
        }
    }
    let (child_x, child_y) = child_origin(box_node, box_x, abs_y + box_node.y);
    for child in &box_node.children {
        svg_shape_rects_from_layout(doc, child, child_x, child_y, out);
    }
}

/// 单个 `<svg>` 视口盒 → 形状子元素屏幕矩形列表（文档序）。
///
/// viewBox 映射：`scale = min(box_w/vw, box_h/vh)`（meet），内容在视口内居中
/// （`xMidYMid`），再平移 `-min·scale` 把用户单位原点对到屏幕。
pub(crate) fn svg_shape_screen_rects(
    doc: &Document,
    svg_id: NodeId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Vec<(NodeId, crate::rect_bridge::Rect4)> {
    let (scale, origin_x, origin_y) = match doc.get_attribute(svg_id, "viewBox").and_then(|v| parse_view_box(&v)) {
        Some((min_x, min_y, view_w, view_h)) => {
            let scale = (w / view_w).min(h / view_h);
            (
                scale,
                x + (w - view_w * scale) / 2.0 - min_x * scale,
                y + (h - view_h * scale) / 2.0 - min_y * scale,
            )
        }
        None => (1.0, x, y),
    };
    let mut shapes = Vec::new();
    walk_shapes(doc, svg_id, scale, origin_x, origin_y, &mut shapes, 0);
    shapes
}

/// 先序遍历 svg 子树合成形状包围盒；用户单位 → 屏幕坐标 = `origin + value·scale`。
fn walk_shapes(
    doc: &Document,
    id: NodeId,
    scale: f32,
    origin_x: f32,
    origin_y: f32,
    out: &mut Vec<(NodeId, crate::rect_bridge::Rect4)>,
    depth: usize,
) {
    if depth > SHAPE_WALK_MAX_DEPTH {
        return;
    }
    for child in doc.child_nodes(id) {
        let Some(data) = doc.get(child) else { continue };
        let NodeKind::Element(elem) = &data.kind else { continue };
        match elem.local_name().to_ascii_lowercase().as_str() {
            // FIXME: 嵌套 `<svg>` viewport 几何（级联 viewBox 映射）未建模，子树跳过。
            "svg" => continue,
            "rect" => {
                let x = attr_f32(doc, child, "x").unwrap_or(0.0);
                let y = attr_f32(doc, child, "y").unwrap_or(0.0);
                let w = attr_f32(doc, child, "width").unwrap_or(0.0);
                let h = attr_f32(doc, child, "height").unwrap_or(0.0);
                if w > 0.0 && h > 0.0 {
                    push_shape(out, child, x, y, w, h, scale, origin_x, origin_y);
                }
            }
            "circle" => {
                let cx = attr_f32(doc, child, "cx").unwrap_or(0.0);
                let cy = attr_f32(doc, child, "cy").unwrap_or(0.0);
                let r = attr_f32(doc, child, "r").unwrap_or(0.0);
                if r > 0.0 {
                    push_shape(out, child, cx - r, cy - r, 2.0 * r, 2.0 * r, scale, origin_x, origin_y);
                }
            }
            "ellipse" => {
                let cx = attr_f32(doc, child, "cx").unwrap_or(0.0);
                let cy = attr_f32(doc, child, "cy").unwrap_or(0.0);
                let rx = attr_f32(doc, child, "rx").unwrap_or(0.0);
                let ry = attr_f32(doc, child, "ry").unwrap_or(0.0);
                if rx > 0.0 && ry > 0.0 {
                    push_shape(
                        out,
                        child,
                        cx - rx,
                        cy - ry,
                        2.0 * rx,
                        2.0 * ry,
                        scale,
                        origin_x,
                        origin_y,
                    );
                }
            }
            // FIXME: path/polygon/polyline/line/text/use/foreignObject 几何未建模。
            _ => {}
        }
        walk_shapes(doc, child, scale, origin_x, origin_y, out, depth + 1);
    }
}

fn push_shape(
    out: &mut Vec<(NodeId, crate::rect_bridge::Rect4)>,
    id: NodeId,
    user_x: f32,
    user_y: f32,
    user_w: f32,
    user_h: f32,
    scale: f32,
    origin_x: f32,
    origin_y: f32,
) {
    out.push((
        id,
        (
            origin_x + user_x * scale,
            origin_y + user_y * scale,
            user_w * scale,
            user_h * scale,
        ),
    ));
}

fn is_svg_element(doc: &Document, id: NodeId) -> bool {
    doc.get(id).is_some_and(
        |data| matches!(&data.kind, NodeKind::Element(elem) if elem.local_name().eq_ignore_ascii_case("svg")),
    )
}

fn attr_f32(doc: &Document, id: NodeId, name: &str) -> Option<f32> {
    doc.get_attribute(id, name)?
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|v| v.is_finite())
}

/// viewBox 属性解析：空白/逗号分隔的 4 个有限数值（`min_x min_y view_w view_h`）。
fn parse_view_box(value: &str) -> Option<(f32, f32, f32, f32)> {
    let nums: Vec<f32> = value
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.trim().parse::<f32>().ok().filter(|v| v.is_finite()))
        .collect();
    if nums.len() != 4 {
        return None;
    }
    Some((nums[0], nums[1], nums[2], nums[3]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hit_test::HitTestCache;

    /// 手工布局树：body 根盒 + 指定 rect 的子盒。
    fn tree(body: zero_dom::NodeId, child: zero_dom::NodeId, x: f32, y: f32, w: f32, h: f32) -> LayoutBox {
        let mut root = LayoutBox {
            node_id: Some(body),
            width: 800.0,
            height: 600.0,
            ..LayoutBox::default()
        };
        root.children.push(LayoutBox {
            node_id: Some(child),
            x,
            y,
            width: w,
            height: h,
            ..LayoutBox::default()
        });
        root
    }

    #[test]
    fn view_box_meet_mapping_centers_and_scales() {
        // 200×100 盒放 500×500 viewBox：scale = 0.2，内容垂直居中（(100−500·0.2)/2 = 0）。
        let doc = zero_dom::parse_html(
            r#"<body><svg id="s" viewBox="0 0 500 500"><rect id="t" width="500" height="500"/></svg></body>"#,
        );
        let svg = doc.get_element_by_id("s").expect("svg");
        let target = doc.get_element_by_id("t").expect("rect");
        let body = doc.get_elements_by_tag_name("body")[0];
        let root = tree(body, svg, 10.0, 20.0, 200.0, 100.0);
        let rects = svg_shape_screen_rects(&doc, svg, 10.0, 20.0, 200.0, 100.0);
        // scale=0.2，水平居中：(200 − 500·0.2)/2 = 50 → ox = 60。
        assert_eq!(rects, vec![(target, (60.0, 20.0, 100.0, 100.0))]);

        let cache = HitTestCache::from_document(&doc, &root, &HashMap::new());
        let snapshot = crate::rect_bridge::new_layout_rect_snapshot();
        cache.fill_layout_rect_snapshot(&snapshot);
        let map = snapshot.lock().expect("map");
        let svg_rect = map.get(&node_id_to_u64(svg)).copied().expect("svg box rect");
        assert_eq!(svg_rect, (10.0, 20.0, 200.0, 100.0));
        assert_eq!(map.get(&node_id_to_u64(target)), Some(&(60.0, 20.0, 100.0, 100.0)));
    }

    #[test]
    fn no_view_box_maps_user_units_one_to_one() {
        let doc =
            zero_dom::parse_html(r#"<body><svg id="s"><rect id="t" x="4" y="6" width="10" height="12"/></svg></body>"#);
        let svg = doc.get_element_by_id("s").expect("svg");
        let target = doc.get_element_by_id("t").expect("rect");
        let rects = svg_shape_screen_rects(&doc, svg, 10.0, 20.0, 200.0, 100.0);
        assert_eq!(rects, vec![(target, (14.0, 26.0, 10.0, 12.0))]);
    }

    #[test]
    fn circle_and_ellipse_bounding_boxes_use_center_radius() {
        let doc = zero_dom::parse_html(
            r#"<body><svg id="s" viewBox="0 0 100 100"><circle id="c" cx="50" cy="50" r="20"/><ellipse id="e" cx="50" cy="50" rx="10" ry="30"/></svg></body>"#,
        );
        let svg = doc.get_element_by_id("s").expect("svg");
        let circle = doc.get_element_by_id("c").expect("circle");
        let ellipse = doc.get_element_by_id("e").expect("ellipse");
        let rects = svg_shape_screen_rects(&doc, svg, 0.0, 0.0, 100.0, 100.0);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], (circle, (30.0, 30.0, 40.0, 40.0)));
        assert_eq!(rects[1], (ellipse, (40.0, 20.0, 20.0, 60.0)));
    }

    #[test]
    fn nested_svg_and_unmodeled_shapes_are_skipped() {
        let doc = zero_dom::parse_html(
            r#"<body><svg id="s"><rect id="keep" width="8" height="8"/><path id="p" d="M0 0"/><svg id="inner"><rect id="drop" width="4" height="4"/></svg></svg></body>"#,
        );
        let svg = doc.get_element_by_id("s").expect("svg");
        let keep = doc.get_element_by_id("keep").expect("keep");
        let rects = svg_shape_screen_rects(&doc, svg, 0.0, 0.0, 100.0, 100.0);
        assert_eq!(rects, vec![(keep, (0.0, 0.0, 8.0, 8.0))]);
    }

    #[test]
    fn zero_sized_and_invalid_attributes_are_skipped() {
        let doc = zero_dom::parse_html(
            r#"<body><svg id="s"><rect id="zero" width="0" height="8"/><rect id="pct" width="50%" height="8"/><rect id="ok" width="8" height="8"/></svg></body>"#,
        );
        let svg = doc.get_element_by_id("s").expect("svg");
        let ok = doc.get_element_by_id("ok").expect("ok");
        let rects = svg_shape_screen_rects(&doc, svg, 0.0, 0.0, 100.0, 100.0);
        assert_eq!(rects, vec![(ok, (0.0, 0.0, 8.0, 8.0))]);
    }

    #[test]
    fn invalid_view_box_falls_back_to_one_to_one() {
        let doc = zero_dom::parse_html(
            r#"<body><svg id="s" viewBox="abc 0 500"><rect id="t" width="5" height="5"/></svg></body>"#,
        );
        let svg = doc.get_element_by_id("s").expect("svg");
        let target = doc.get_element_by_id("t").expect("rect");
        let rects = svg_shape_screen_rects(&doc, svg, 7.0, 9.0, 100.0, 100.0);
        assert_eq!(rects, vec![(target, (7.0, 9.0, 5.0, 5.0))]);
    }
}
