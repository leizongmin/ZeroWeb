//! R5214：float 摆放 earliest-fit + table 内部盒载体豁免。
//!
//! 两个 load-bearing 锚（对应 floats-wrap-bfc-004 上游案）：
//! 1. earliest-fit 带步进（§9.5.1 规则 5 as high as possible）：放不进当前区间时下移到
//!    相交 float 的**最近**底边逐带试探，替代旧行推进（整行下移到当前行 max float 底）。
//!    chromium CDP 探针定谳（R5213）：float 与 BFC 盒同用 earliest-fit。
//! 2. R5070 载体修正豁免 table 内部盒：td（非 block_level 非 flow_root）旧判定误作
//!    inline 载体，cell 内 right float 被「重定」到表右缘覆盖先落位 float
//!    （CSS 2.1 §10.1 float 包含块 = 最近 block-level/table-cell/table-caption 祖先）。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_style_system::ComputedStyle;

/// earliest-fit 带步进：300 容器内左 float 100×20（blue）+ 右 float 100×6（silver）+
/// 右 float 150×20。首带 y=0 区间 [blue 右缘 100, silver 左缘 200] 可用 100 < 150
/// 放不下 → earliest-fit 下移到相交 float 最近底边 6（silver 底；blue 底 20 更深）：
/// band_y=6 时仅 blue 带相交（[0,20)），区间 [100,300] 可用 200 ≥ 150 → 落
/// x=150 y=6。旧行推进模型一步到行底（line_max_height=20）得 y=20——y=6 即锚。
#[test]
fn r5214_right_float_earliest_fit_nearest_band_bottom() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let blue = doc.create_element("div");
    doc.append_child(container, blue).unwrap();
    let silver = doc.create_element("div");
    doc.append_child(container, silver).unwrap();
    let right = doc.create_element("div");
    doc.append_child(container, right).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);
    let mut sb = ComputedStyle::default();
    sb.display = DisplayValue::Block;
    sb.float = FloatValue::Left;
    sb.width = LengthValue::Px(100.0);
    sb.height = LengthValue::Px(20.0);
    styles.insert(blue, sb);
    let mut ss = ComputedStyle::default();
    ss.display = DisplayValue::Block;
    ss.float = FloatValue::Right;
    ss.width = LengthValue::Px(100.0);
    ss.height = LengthValue::Px(6.0);
    styles.insert(silver, ss);
    let mut sr = ComputedStyle::default();
    sr.display = DisplayValue::Block;
    sr.float = FloatValue::Right;
    sr.width = LengthValue::Px(150.0);
    sr.height = LengthValue::Px(20.0);
    styles.insert(right, sr);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let right_box = find_child_by_node_id(&result.root, right).expect("right float found");
    assert!(
        (right_box.y - 6.0).abs() <= 0.5,
        "right float 应 earliest-fit 落最近带底 6（silver 底），行推进回归会推 y=20，实际 y={}",
        right_box.y
    );
    assert!(
        (right_box.x - 150.0).abs() <= 0.5,
        "right float 应贴容器右缘（x=150），实际 x={}",
        right_box.x
    );
}

/// 双 right float 栈（300 容器，blue 100×20 + silver 100×6）：silver 应 beside blue
/// （x=100），不重叠（x=200 即回归——旧行推进 right_used_width 叠加错误或载体重写）。
#[test]
fn r5214_right_float_stack_beside_not_overlap() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let blue = doc.create_element("div");
    doc.append_child(container, blue).unwrap();
    let silver = doc.create_element("div");
    doc.append_child(container, silver).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);
    let mut sb = ComputedStyle::default();
    sb.display = DisplayValue::Block;
    sb.float = FloatValue::Right;
    sb.width = LengthValue::Px(100.0);
    sb.height = LengthValue::Px(20.0);
    styles.insert(blue, sb);
    let mut ss = ComputedStyle::default();
    ss.display = DisplayValue::Block;
    ss.float = FloatValue::Right;
    ss.width = LengthValue::Px(100.0);
    ss.height = LengthValue::Px(6.0);
    styles.insert(silver, ss);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let blue_box = find_child_by_node_id(&result.root, blue).expect("blue found");
    let silver_box = find_child_by_node_id(&result.root, silver).expect("silver found");
    assert!((blue_box.x - 200.0).abs() <= 0.5, "blue x=200，实际 {}", blue_box.x);
    assert!(
        (silver_box.x - 100.0).abs() <= 0.5,
        "silver 应 beside blue 左侧（x=100），重叠 x=200 即回归，实际 x={}",
        silver_box.x
    );
}

/// R5070 载体豁免：真 td（display:table-cell，非 block_level 非 flow_root）内双 right
/// float 不被终末载体修正重写到表右缘（floats-wrap-bfc-004 S4：silver x=100 非 200）。
#[test]
fn r5214_td_right_floats_not_carrier_rewritten() {
    let src = "<html><body style=\"margin:0\"><table width=\"300\" style=\"border-spacing:0\"><tbody><tr><td>\
  <div style=\"float:right; width:100px; height:20px\"></div>\
  <div style=\"float:right; width:100px; height:6px\"></div>\
</td></tr></tbody></table></body></html>";
    let doc = zero_dom::parse_html(src);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    fn collect_floats(b: &LayoutBox, out: &mut Vec<(f32, f32, f32)>) {
        if b.float != zero_css_parser::values::FloatValue::None {
            out.push((b.x, b.y, b.height));
        }
        for c in &b.children {
            collect_floats(c, out);
        }
    }
    let mut floats = Vec::new();
    collect_floats(&result.root, &mut floats);
    assert_eq!(floats.len(), 2, "应有两个 right float，实际 {:?}", floats);
    let (bx, by, bh) = floats[0];
    let (sx, sy, sh) = floats[1];
    assert!((bh - 20.0).abs() < 0.5 && (sh - 6.0).abs() < 0.5, "顺序 {:?}", floats);
    assert!((bx - 200.0).abs() <= 0.5, "blue 贴 td 右缘 x=200，实际 {}", bx);
    assert!(
        (by - 0.0).abs() <= 0.5 && (sy - 0.0).abs() <= 0.5,
        "同带 beside，实际 {:?}",
        floats
    );
    assert!(
        (sx - 100.0).abs() <= 0.5,
        "silver 应 beside blue（x=100）；被载体修正重写 x=200 即回归，实际 x={}",
        sx
    );
}
