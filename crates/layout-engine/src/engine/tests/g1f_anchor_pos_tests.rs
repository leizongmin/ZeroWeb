//! wptfyi-css 组F 布局刀2：position-area 区域定位 pass 端到端
//!（css-anchor-position-1 §position-area；oracle 形 = WPT css-anchor-position/
//! position-area-basic.html 的 3×3 网格——container 400×400 内锚 margin 100/150、
//! 150×75，anchored 直挂 CB，区域格满填 stretch 语义）。
//!
//! `ZW_ANCHOR_AREA=0` kill-switch 的回退形由 env 控制不在单测覆盖（进程级
//! LazyLock 进程首用定格）；此处钉 default-on 形。

use super::*;
use std::collections::HashMap;
use zero_css_parser::values::LengthValue;

/// 搭 position-area-basic 场景：container（abspos 400×400 = CB）内 anchored
///（abspos + position-anchor/position-area）、spade（160px 流内占位——锚的
/// 垂直偏移载体；margin-top 在 taffy 折叠/传递面丢失——既有面缺口挂账，fixture
/// 避开 margin 垂直偏移）与 anchor（流内 margin-left 100、150×75、anchor-name）。
/// 锚顶缘 = spade 底 = 160。
/// 返回 (result.root, container, anchored, anchor)。
fn anchor_fixture(area: &str) -> (crate::LayoutResult, NodeId, NodeId, NodeId) {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let anchored = doc.create_element("div");
    doc.append_child(container, anchored).unwrap();
    let spade = doc.create_element("div");
    doc.append_child(container, spade).unwrap();
    let anchor = doc.create_element("div");
    doc.append_child(container, anchor).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.position = zero_style_system::PositionValue::Absolute;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(400.0);

    let mut anchored_style = ComputedStyle::default();
    anchored_style.position = zero_style_system::PositionValue::Absolute;
    anchored_style.position_anchor = Some("--anchor".into());
    anchored_style.position_area = Some(area.into());

    let mut spade_style = ComputedStyle::default();
    spade_style.height = LengthValue::Px(160.0);

    let mut anchor_style = ComputedStyle::default();
    anchor_style.width = LengthValue::Px(150.0);
    anchor_style.height = LengthValue::Px(75.0);
    anchor_style.margin_left = LengthValue::Px(100.0);
    anchor_style.anchor_name = Some("--anchor".into());

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(anchored, anchored_style);
    styles.insert(spade, spade_style);
    styles.insert(anchor, anchor_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (result, container, anchored, anchor)
}

fn find(root: &crate::LayoutBox, id: NodeId) -> Option<&crate::LayoutBox> {
    if root.node_id == Some(id) {
        return Some(root);
    }
    root.children.iter().find_map(|c| find(c, id))
}

/// 物理双词 `top left`（author 原文序，非 canonical 序）→ 区域 (行0, 列0)：
/// 格 (0..100, 0..150)。
#[test]
fn g1f_position_area_top_left_grid() {
    let (result, container, anchored, _anchor) = anchor_fixture("top left");
    let c = find(&result.root, container).expect("container box");
    let a = find(&result.root, anchored).expect("anchored box");
    // 区域宽 = 锚左缘（100）− CB 左缘（0）；高 = 锚顶缘（160）。
    assert!((a.width - 100.0).abs() < 0.5, "anchored width {} 应为列宽 100", a.width);
    assert!(
        (a.height - 160.0).abs() < 0.5,
        "anchored height {} 应为行高 160",
        a.height
    );
    // anchored 定位在区域原点（container 无 border/padding → 父相对 x/y = 0）。
    assert!(a.x.abs() < 0.5, "anchored x {} 应为 0", a.x);
    assert!(a.y.abs() < 0.5, "anchored y {} 应为 0", a.y);
    let _ = c;
}

/// 折叠形 `bottom`（隐式轴 span-all）→ 区域 (行2, 列全)：
/// (x 0..400, y 235..400)。
#[test]
fn g1f_position_area_bottom_folded_row_span_all() {
    let (result, _container, anchored, _anchor) = anchor_fixture("bottom");
    let a = find(&result.root, anchored).expect("anchored box");
    assert!(
        (a.width - 400.0).abs() < 0.5,
        "anchored width {} 应为 CB 全宽 400",
        a.width
    );
    assert!(
        (a.height - 165.0).abs() < 0.5,
        "anchored height {} 应为行高 165",
        a.height
    );
    // 底行起点 = 锚底缘 235（spade 10 + margin 150 + 高 75）。
    assert!((a.y - 235.0).abs() < 0.5, "anchored y {} 应为 235", a.y);
}

/// `center center` 折叠 `center` → 中央格 (150×75 at 100,150)。
#[test]
fn g1f_position_area_center_cell() {
    let (result, _container, anchored, _anchor) = anchor_fixture("center");
    let a = find(&result.root, anchored).expect("anchored box");
    assert!((a.width - 150.0).abs() < 0.5, "anchored width {} 应为中列 150", a.width);
    assert!(
        (a.height - 75.0).abs() < 0.5,
        "anchored height {} 应为中行 75",
        a.height
    );
    assert!((a.x - 100.0).abs() < 0.5, "anchored x {} 应为 100", a.x);
    assert!((a.y - 160.0).abs() < 0.5, "anchored y {} 应为 160", a.y);
}

/// `span-all` → 全 CB 满填 (400×400 at 0,0)。
#[test]
fn g1f_position_area_span_all_full_cb() {
    let (result, _container, anchored, _anchor) = anchor_fixture("span-all");
    let a = find(&result.root, anchored).expect("anchored box");
    assert!((a.width - 400.0).abs() < 0.5, "anchored width {} 应为 400", a.width);
    assert!((a.height - 400.0).abs() < 0.5, "anchored height {} 应为 400", a.height);
    assert!(
        a.x.abs() < 0.5 && a.y.abs() < 0.5,
        "anchored 原点定位，got ({},{})",
        a.x,
        a.y
    );
}

/// 元素 rtl：self 族 inline 轴镜像（ambiguous-self 页同形）。`self-start` 折叠
/// 形 ltr = 行 START × 列 START；rtl 时列镜像 END：x = 锚右缘 250、宽 150。
#[test]
fn g1f_position_area_rtl_self_start_mirror() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let anchored = doc.create_element("div");
    doc.append_child(container, anchored).unwrap();
    let spade = doc.create_element("div");
    doc.append_child(container, spade).unwrap();
    let anchor = doc.create_element("div");
    doc.append_child(container, anchor).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.position = zero_style_system::PositionValue::Absolute;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(400.0);

    let mut anchored_style = ComputedStyle::default();
    anchored_style.position = zero_style_system::PositionValue::Absolute;
    anchored_style.direction = zero_style_system::DirectionValue::Rtl;
    anchored_style.position_anchor = Some("--anchor".into());
    anchored_style.position_area = Some("self-start".into());

    let mut spade_style = ComputedStyle::default();
    spade_style.height = LengthValue::Px(160.0);

    let mut anchor_style = ComputedStyle::default();
    anchor_style.width = LengthValue::Px(150.0);
    anchor_style.height = LengthValue::Px(75.0);
    anchor_style.margin_left = LengthValue::Px(100.0);
    anchor_style.anchor_name = Some("--anchor".into());

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(anchored, anchored_style);
    styles.insert(spade, spade_style);
    styles.insert(anchor, anchor_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let a = find(&result.root, anchored).expect("anchored box");
    // 列镜像 END：col[2] = 锚右缘 250 → x 250、宽 400-250=150；行 START 不动。
    assert!((a.x - 250.0).abs() < 0.5, "rtl self-start x {} 应为 250", a.x);
    assert!(
        (a.width - 150.0).abs() < 0.5,
        "rtl self-start width {} 应为 150",
        a.width
    );
    assert!((a.y - 0.0).abs() < 0.5, "rtl self-start y {} 应为 0", a.y);
    assert!(
        (a.height - 160.0).abs() < 0.5,
        "rtl self-start height {} 应为 160",
        a.height
    );
}

/// 无 position-anchor（默认锚未建模）→ 不介入（taffy 静态形保持）。
#[test]
fn g1f_position_area_without_anchor_not_applied() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let anchored = doc.create_element("div");
    doc.append_child(container, anchored).unwrap();
    let anchor = doc.create_element("div");
    doc.append_child(container, anchor).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.position = zero_style_system::PositionValue::Absolute;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(400.0);
    let mut anchored_style = ComputedStyle::default();
    anchored_style.position = zero_style_system::PositionValue::Absolute;
    anchored_style.position_area = Some("top left".into());
    let mut anchor_style = ComputedStyle::default();
    anchor_style.anchor_name = Some("--anchor".into());

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(anchored, anchored_style);
    styles.insert(anchor, anchor_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let a = find(&result.root, anchored).expect("anchored box");
    // 未介入：不覆写为区域格（0×150 形不出现；abspos 无宽内容塌缩 0 或静态形）。
    assert!(
        (a.height - 150.0).abs() > 0.5,
        "无 position-anchor 时区域定位不得介入（height {} 非格高 150）",
        a.height
    );
}
