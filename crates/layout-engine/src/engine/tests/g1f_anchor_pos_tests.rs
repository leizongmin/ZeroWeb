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
    // 显式 stretch（WPT basic oracle 同形）：g35 对齐面后 default 走 §4.1 区域
    // 默认表（end/start/center/anchor-center + 收缩尺寸），满填需显式声明。
    anchored_style.align_self = zero_css_parser::values::AlignmentValue::Stretch;
    anchored_style.justify_self = zero_style_system::JustifySelfValue::Stretch;

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
    // 显式 stretch 同 anchor_fixture（g35 对齐面 default 表化）。
    anchored_style.align_self = zero_css_parser::values::AlignmentValue::Stretch;
    anchored_style.justify_self = zero_style_system::JustifySelfValue::Stretch;

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

/// g35 对齐面：无显式 align/justify-self → §4.1 区域默认表。`top left` 仅
/// start 轨 → 双轴 END：10×10 元素 margin-box 右/底缘钉区域右/底缘
///（x = 100-10 = 90、y = 160-10 = 150，css-anchor-position-1 §4.1）。
#[test]
fn g1f_position_area_default_table_end_pin() {
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
    anchored_style.position_area = Some("top left".into());
    anchored_style.width = LengthValue::Px(10.0);
    anchored_style.height = LengthValue::Px(10.0);

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
    // 仅 start 轨 → 默认 END：margin-box 右缘钉列右缘（锚左缘 100）、底缘钉
    // 行底缘（锚顶缘 160）；尺寸保持显式 10×10 不满填。
    assert!((a.x - 90.0).abs() < 0.5, "default table x {} 应为 100-10 = 90", a.x);
    assert!((a.y - 150.0).abs() < 0.5, "default table y {} 应为 160-10 = 150", a.y);
    assert!(
        (a.width - 10.0).abs() < 0.5,
        "显式尺寸不满填：width {} 应为 10",
        a.width
    );
}

/// g35 对齐面：span-all 三轨全选 → 默认 anchor-center（margin-box 中心对锚
/// 中心）：锚 (100..250, 160..235) 中心 (175, 197.5)，10×10 → (170, 192.5)。
#[test]
fn g1f_position_area_span_all_anchor_center() {
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
    anchored_style.position_area = Some("span-all".into());
    anchored_style.width = LengthValue::Px(10.0);
    anchored_style.height = LengthValue::Px(10.0);

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
    assert!((a.x - 170.0).abs() < 0.5, "anchor-center x {} 应为 175-5 = 170", a.x);
    assert!(
        (a.y - 192.5).abs() < 0.5,
        "anchor-center y {} 应为 197.5-5 = 192.5",
        a.y
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

// ── 刀3：anchor() inset 求值 ──────────────────────────────────────────────

/// anchor() 四 inset 全锚定拉伸（WPT anchor-position-writing-modes-001 oracle
/// 场景：container 600×400 CB 内 a1/a2 流内 100×100，target 四 inset 全
/// anchor()）→ [200,200,500,300]。物理边求值与写作模式正交。垂直偏移用
/// spacer 高度承载（taffy 流内 margin-top 折叠面既有缺口，刀2 fixture 同避
/// 法；anchor 几何以视口坐标折算 CB 相对，等价 oracle 几何）。
#[test]
fn g1f_anchor_inset_quad_stretch_writing_modes_oracle() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let spacer1 = doc.create_element("div");
    doc.append_child(container, spacer1).unwrap();
    let a1 = doc.create_element("div");
    doc.append_child(container, a1).unwrap();
    let spacer2 = doc.create_element("div");
    doc.append_child(container, spacer2).unwrap();
    let a2 = doc.create_element("div");
    doc.append_child(container, a2).unwrap();
    let target = doc.create_element("div");
    doc.append_child(container, target).unwrap();

    let mut container_style = ComputedStyle::default();
    // UA sheet div{display:block} 的 fixture 显式形（default display=Inline；
    // inline CB + 块级子 → abspos 目标盒不落树，见 anchor display 注）。
    container_style.display = zero_style_system::DisplayValue::Block;
    container_style.position = zero_style_system::PositionValue::Relative;
    container_style.width = LengthValue::Px(600.0);
    container_style.height = LengthValue::Px(400.0);

    let mut spacer_style = ComputedStyle::default();
    // Block 显式形：height 不作用于 inline 盒（块流中 inline spacer 高度塌为
    // 行高——UA div 为 Block 的真实几何）。
    spacer_style.display = zero_style_system::DisplayValue::Block;
    spacer_style.height = LengthValue::Px(100.0);

    let mut a1_style = ComputedStyle::default();
    // UA sheet div{display:block} 的 fixture 显式形（default display=Inline，
    // 会触发 inline 流锚介入门——真实页 div 经 UA 表为 Block）。
    a1_style.display = zero_style_system::DisplayValue::Block;
    a1_style.width = LengthValue::Px(100.0);
    a1_style.height = LengthValue::Px(100.0);
    a1_style.margin_left = LengthValue::Px(100.0);
    a1_style.anchor_name = Some("--a1".into());

    let mut a2_style = ComputedStyle::default();
    a2_style.display = zero_style_system::DisplayValue::Block;
    a2_style.width = LengthValue::Px(100.0);
    a2_style.height = LengthValue::Px(100.0);
    a2_style.margin_left = LengthValue::Px(500.0);
    a2_style.anchor_name = Some("--a2".into());

    let mut target_style = ComputedStyle::default();
    // abspos 盒落树需显式 Block（default Inline 不落块流盒——r4987 fixture 同形）。
    target_style.display = zero_style_system::DisplayValue::Block;
    target_style.position = zero_style_system::PositionValue::Absolute;
    target_style.anchor_inset_left = Some("anchor(--a1 right)".into());
    target_style.anchor_inset_top = Some("anchor(--a1 bottom)".into());
    target_style.anchor_inset_right = Some("anchor(--a2 left)".into());
    target_style.anchor_inset_bottom = Some("anchor(--a2 top)".into());

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(spacer1, spacer_style.clone());
    styles.insert(spacer2, spacer_style);
    styles.insert(a1, a1_style);
    styles.insert(a2, a2_style);
    styles.insert(target, target_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let t = find(&result.root, target).expect("target box");
    // spacer1 后 a1 (100..200, 100..200)，spacer2 后 a2 (500..600, 300..400)：
    // left inset = a1 右缘 200；right inset = CB 右 600 − a2 左缘 500 = 100 →
    // 宽 300（右缘 500）；top inset = a1 底缘 200；bottom inset = 400 − a2 顶
    // 缘 300 = 100 → 高 100（底缘 300）。oracle [200,200,500,300]。
    assert!((t.x - 200.0).abs() < 0.5, "target x {} 应为 200", t.x);
    assert!((t.y - 200.0).abs() < 0.5, "target y {} 应为 200", t.y);
    assert!((t.width - 300.0).abs() < 0.5, "target width {} 应为 300", t.width);
    assert!((t.height - 100.0).abs() < 0.5, "target height {} 应为 100", t.height);
}

/// anchor() 缺锚回落 fallback length；锚存在时 fallback 不消费。
#[test]
fn g1f_anchor_inset_fallback_on_missing_anchor() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let target = doc.create_element("div");
    doc.append_child(container, target).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.position = zero_style_system::PositionValue::Relative;
    container_style.width = LengthValue::Px(600.0);
    container_style.height = LengthValue::Px(400.0);

    let mut target_style = ComputedStyle::default();
    target_style.position = zero_style_system::PositionValue::Absolute;
    target_style.anchor_inset_left = Some("anchor(--missing left, 70px)".into());
    target_style.anchor_inset_top = Some("anchor(--missing top, 25%)".into());

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(target, target_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let t = find(&result.root, target).expect("target box");
    // --missing 无锚 → left 回落 70px；top 回落 25%×400=100。
    assert!((t.x - 70.0).abs() < 0.5, "target x {} 应为 fallback 70", t.x);
    assert!((t.y - 100.0).abs() < 0.5, "target y {} 应为 fallback 25% = 100", t.y);
}

/// 逻辑边关键字（start，需锚写作模式映射）第一刀不介入 → 槽位 Auto 静态形。
#[test]
fn g1f_anchor_inset_logical_side_not_applied() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let anchor = doc.create_element("div");
    doc.append_child(container, anchor).unwrap();
    let target = doc.create_element("div");
    doc.append_child(container, target).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.position = zero_style_system::PositionValue::Relative;
    container_style.width = LengthValue::Px(600.0);
    container_style.height = LengthValue::Px(400.0);
    let mut anchor_style = ComputedStyle::default();
    anchor_style.width = LengthValue::Px(100.0);
    anchor_style.height = LengthValue::Px(50.0);
    anchor_style.anchor_name = Some("--a".into());
    let mut target_style = ComputedStyle::default();
    target_style.position = zero_style_system::PositionValue::Absolute;
    target_style.anchor_inset_left = Some("anchor(--a start)".into());
    target_style.width = LengthValue::Px(10.0);
    target_style.height = LengthValue::Px(10.0);

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(anchor, anchor_style);
    styles.insert(target, target_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let t = find(&result.root, target).expect("target box");
    // 逻辑边不介入：left 槽 Auto → taffy 静态位（IFC 流内 anchor 后 y≈50、
    // x≈0——本 fixture 保 default-Inline 形，块流 abspos 静态 y 映射为 0 不
    // 能区分介入/静态），非锚左缘定位形。锚定形 x 应为 0 不可区分，钉 y。
    assert!(
        (t.y - 50.0).abs() < 0.5,
        "逻辑边不介入时 target 保持静态位 y {} ≈ 50",
        t.y
    );
}

/// 纯 inline 流锚（display:inline span）几何不可靠 → pass3 锚查找过滤为无锚
/// → fallback 消费钉面（介入形会取 span 静态盒几何 x≈0）。WPT 回归门：
/// anchor-position-inline-002（锚 #anchor1 行内 span，介入致 2P→0P）。
#[test]
fn g1f_anchor_inset_inline_flow_anchor_gated() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let anchor = doc.create_element("span");
    doc.append_child(container, anchor).unwrap();
    let target = doc.create_element("div");
    doc.append_child(container, target).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.position = zero_style_system::PositionValue::Relative;
    container_style.width = LengthValue::Px(600.0);
    container_style.height = LengthValue::Px(400.0);
    let mut anchor_style = ComputedStyle::default();
    anchor_style.display = zero_style_system::DisplayValue::Inline;
    anchor_style.anchor_name = Some("--a".into());
    let mut target_style = ComputedStyle::default();
    target_style.position = zero_style_system::PositionValue::Absolute;
    // fallback 70px 即介入门观测点：门控（锚过滤）→ 回落 70；未门控 → span
    // 几何（x≈0）。
    target_style.anchor_inset_left = Some("anchor(--a left, 70px)".into());
    target_style.width = LengthValue::Px(10.0);
    target_style.height = LengthValue::Px(10.0);

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(anchor, anchor_style);
    styles.insert(target, target_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let t = find(&result.root, target).expect("target box");
    assert!(
        (t.x - 70.0).abs() < 0.5,
        "inline 流锚门控后 target.x {} 应为 fallback 70（非锚几何）",
        t.x
    );
}

/// 滚动上下文失配：锚在 overflow:scroll 容器内、目标在其外 → 锚滚移而目标
/// 不动（引擎不建模 scroll offset，盒坐标为布局静态位）→ 坐标系失配不介入，
/// fallback 消费钉面（WPT anchor-scroll-js-expose 滚移断言挂账同源）。
#[test]
fn g1f_anchor_inset_scroll_context_mismatch_gated() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let scroller = doc.create_element("div");
    doc.append_child(container, scroller).unwrap();
    let anchor = doc.create_element("div");
    doc.append_child(scroller, anchor).unwrap();
    let target = doc.create_element("div");
    doc.append_child(container, target).unwrap();

    let mut container_style = ComputedStyle::default();
    container_style.position = zero_style_system::PositionValue::Relative;
    container_style.width = LengthValue::Px(600.0);
    container_style.height = LengthValue::Px(400.0);
    let mut scroller_style = ComputedStyle::default();
    scroller_style.height = LengthValue::Px(100.0);
    scroller_style.overflow_y = zero_css_parser::values::OverflowValue::Scroll;
    let mut anchor_style = ComputedStyle::default();
    // display 显式 Block：隔离滚动上下文门（不混入 inline 流门）。
    anchor_style.display = zero_style_system::DisplayValue::Block;
    anchor_style.width = LengthValue::Px(20.0);
    anchor_style.height = LengthValue::Px(20.0);
    anchor_style.anchor_name = Some("--a".into());
    let mut target_style = ComputedStyle::default();
    target_style.position = zero_style_system::PositionValue::Absolute;
    // 未门控形会取锚静态几何 x=0；门控 → fallback 70。
    target_style.anchor_inset_left = Some("anchor(--a left, 70px)".into());
    target_style.width = LengthValue::Px(10.0);
    target_style.height = LengthValue::Px(10.0);

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(scroller, scroller_style);
    styles.insert(anchor, anchor_style);
    styles.insert(target, target_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let t = find(&result.root, target).expect("target box");
    assert!(
        (t.x - 70.0).abs() < 0.5,
        "滚动上下文失配门控后 target.x {} 应为 fallback 70",
        t.x
    );
}

/// 同滚上下文允许求值：scroller 兼 positioned CB（relative），锚与目标均在
/// 其内 → 两侧盒坐标同基（同滚上下文内坐标一致性），anchor() 正常介入。
#[test]
fn g1f_anchor_inset_same_scroll_context_resolved() {
    let (mut doc, body) = make_doc_with_body();
    let scroller = doc.create_element("div");
    doc.append_child(body, scroller).unwrap();
    let anchor = doc.create_element("div");
    doc.append_child(scroller, anchor).unwrap();
    let target = doc.create_element("div");
    doc.append_child(scroller, target).unwrap();

    let mut scroller_style = ComputedStyle::default();
    scroller_style.position = zero_style_system::PositionValue::Relative;
    scroller_style.width = LengthValue::Px(600.0);
    scroller_style.height = LengthValue::Px(400.0);
    scroller_style.overflow_y = zero_css_parser::values::OverflowValue::Scroll;
    let mut anchor_style = ComputedStyle::default();
    // display 显式 Block：隔离同滚上下文允许形（inline 流门不误触）。
    anchor_style.display = zero_style_system::DisplayValue::Block;
    anchor_style.width = LengthValue::Px(20.0);
    anchor_style.height = LengthValue::Px(20.0);
    anchor_style.margin_left = LengthValue::Px(30.0);
    anchor_style.anchor_name = Some("--a".into());
    let mut target_style = ComputedStyle::default();
    // abspos 盒落树需显式 Block（default Inline 不落块流盒——r4987 fixture 同形）。
    target_style.display = zero_style_system::DisplayValue::Block;
    target_style.position = zero_style_system::PositionValue::Absolute;
    target_style.anchor_inset_left = Some("anchor(--a left)".into());
    target_style.width = LengthValue::Px(10.0);
    target_style.height = LengthValue::Px(10.0);

    let mut styles = HashMap::new();
    styles.insert(scroller, scroller_style);
    styles.insert(anchor, anchor_style);
    styles.insert(target, target_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let t = find(&result.root, target).expect("target box");
    // 锚左缘 = scroller 内容 x + margin 30 → left inset = 30。
    assert!(
        (t.x - 30.0).abs() < 0.5,
        "同滚上下文 target.x {} 应为锚左缘 30（介入形，非 fallback/静态）",
        t.x
    );
}

/// 有边框 containing block 的区域/inset 定位原点口径钉测：child.x 基准 =
/// CB padding box 原点（rect 提取与渲染递归按 parent border 偏移 child.x），
/// anchor 两通路不得把 CB border 烘进 child.x——曾双计 border 导致语料
/// offsetLeft 多 +2、满填区域宽/高各缩 2（g36 定谳，见 wptfyi-css master）。
#[test]
fn g1f_border_cb_child_origin() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let anchored = doc.create_element("div");
    doc.append_child(container, anchored).unwrap();
    let anchor = doc.create_element("div");
    doc.append_child(container, anchor).unwrap();

    let mut container_style = ComputedStyle::default();
    // fixture 全员显式 Block（default Inline 触发 inline 流锚介入门与行盒塌陷）。
    container_style.display = zero_style_system::DisplayValue::Block;
    container_style.position = zero_style_system::PositionValue::Absolute;
    container_style.width = LengthValue::Px(400.0);
    container_style.height = LengthValue::Px(400.0);
    container_style.border_top_width = LengthValue::Px(2.0);
    container_style.border_left_width = LengthValue::Px(2.0);
    container_style.border_right_width = LengthValue::Px(2.0);
    container_style.border_bottom_width = LengthValue::Px(2.0);
    container_style.border_top_style = zero_style_system::BorderStyleValue::Solid;
    container_style.border_left_style = zero_style_system::BorderStyleValue::Solid;
    container_style.border_right_style = zero_style_system::BorderStyleValue::Solid;
    container_style.border_bottom_style = zero_style_system::BorderStyleValue::Solid;

    let mut anchored_style = ComputedStyle::default();
    anchored_style.display = zero_style_system::DisplayValue::Block;
    anchored_style.position = zero_style_system::PositionValue::Absolute;
    anchored_style.position_anchor = Some("--anchor".into());
    anchored_style.position_area = Some("span-all".into());
    anchored_style.align_self = zero_css_parser::values::AlignmentValue::Stretch;
    anchored_style.justify_self = zero_style_system::JustifySelfValue::Stretch;

    let mut anchor_style = ComputedStyle::default();
    anchor_style.display = zero_style_system::DisplayValue::Block;
    anchor_style.width = LengthValue::Px(150.0);
    anchor_style.height = LengthValue::Px(75.0);
    anchor_style.anchor_name = Some("--anchor".into());
    anchor_style.margin_top = LengthValue::Px(150.0);
    anchor_style.margin_left = LengthValue::Px(100.0);

    let mut styles = HashMap::new();
    styles.insert(container, container_style);
    styles.insert(anchored, anchored_style.clone());
    styles.insert(anchor, anchor_style);

    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let a = find(&result.root, anchored).expect("anchored box");
    // 区域 = 全 padding box（style width 400 为 content 宽 → padding box
    // 400×400）；child.x = padding 原点相对 0，不含 border。
    assert!((a.x - 0.0).abs() < 0.5, "anchored x {} 应为 0（padding 原点相对）", a.x);
    assert!((a.y - 0.0).abs() < 0.5, "anchored y {} 应为 0（padding 原点相对）", a.y);
    assert!(
        (a.width - 400.0).abs() < 0.5,
        "anchored width {} 应为 padding box 宽 400",
        a.width
    );
    assert!(
        (a.height - 400.0).abs() < 0.5,
        "anchored height {} 应为 padding box 高 400",
        a.height
    );

    // plain abs 对照（无锚）：left:50/top:60 → padding 原点相对 50/60；
    // 两通路同口径，防漂移。
    let plain = doc.create_element("div");
    doc.append_child(container, plain).unwrap();
    let mut plain_style = ComputedStyle::default();
    plain_style.display = zero_style_system::DisplayValue::Block;
    plain_style.position = zero_style_system::PositionValue::Absolute;
    plain_style.left = LengthValue::Px(50.0);
    plain_style.top = LengthValue::Px(60.0);
    let mut styles2 = styles.clone();
    styles2.insert(plain, plain_style);
    let mut engine2 = crate::LayoutEngine::new(800.0, 600.0);
    let result2 = engine2.compute(&doc, &styles2);
    let p = find(&result2.root, plain).expect("plain box");
    assert!((p.x - 50.0).abs() < 0.5, "plain x {} 应为 50（padding 原点相对）", p.x);
    assert!((p.y - 60.0).abs() < 0.5, "plain y {} 应为 60（padding 原点相对）", p.y);

    // inset 通路（position-anchor + anchor() inset）同口径：锚 margin-left
    // 100/margin-top 150 → 锚左/上缘 CB 相对 100/150，锚定 inset 后 viewport
    // 应落在锚缘（padding 原点相对 100/150，不含 border）。
    let mut anchored2_style = anchored_style.clone();
    anchored2_style.position_area = None;
    anchored2_style.anchor_inset_left = Some("anchor(--anchor left)".into());
    anchored2_style.anchor_inset_top = Some("anchor(--anchor top)".into());
    let mut styles3 = styles.clone();
    styles3.insert(anchored, anchored2_style);
    let mut engine3 = crate::LayoutEngine::new(800.0, 600.0);
    let result3 = engine3.compute(&doc, &styles3);
    let a3 = find(&result3.root, anchored).expect("anchored3 box");
    assert!(
        (a3.x - 100.0).abs() < 0.5,
        "inset 通路 anchored x {} 应为 100（锚左缘，padding 原点相对，不含 border）",
        a3.x
    );
    assert!(
        (a3.y - 150.0).abs() < 0.5,
        "inset 通路 anchored y {} 应为 150（锚上缘，padding 原点相对，不含 border）",
        a3.y
    );
}
