//! R5217：width:auto BFC 旁置 float 的 margin-aware 钉位解（CSS2 §9.5 规则 2 × §10.3.3）。
//!
//! chromium FloatAvoider 语义（68 行几何探针实证，floats-wrap-bfc-with-margin 族）：
//! flow 盒 [ml, CB − mr] 被 float-free band 钉位——面向侧正 margin 被吃掉、背侧 margin
//! 收窄宽；左右镜像不对称（左 float frame 下限永不正 margin 下绕，右 float 钉后宽
//! < frame 下绕）；负 margin 阈 `margin_h < −(gap + 0.5)` 下绕（WebKit bug 239976
//! 非互操作带取 chromium 阈）。探针几何：容器 100、float 25×50、无 border/padding。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue, OverflowValue};
use zero_style_system::ComputedStyle;

/// 搭建 容器(width=100) + float(side, 25×50) + overflow:hidden BFC(margin-left/right) 场景。
fn pin_case(margin_left: f64, margin_right: f64, float_side: FloatValue) -> (LayoutBox, LayoutEngine) {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float = doc.create_element("div");
    doc.append_child(container, float).unwrap();
    let bfc_div = doc.create_element("div");
    doc.append_child(container, bfc_div).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(100.0);
    styles.insert(container, cont);

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = float_side;
    fl.width = LengthValue::Px(25.0);
    fl.height = LengthValue::Px(50.0);
    styles.insert(float, fl);

    let mut bd = ComputedStyle::default();
    bd.display = DisplayValue::Block;
    bd.overflow_x = OverflowValue::Hidden;
    bd.overflow_y = OverflowValue::Hidden;
    bd.margin_left = LengthValue::Px(margin_left);
    bd.margin_right = LengthValue::Px(margin_right);
    bd.height = LengthValue::Px(20.0);
    styles.insert(bfc_div, bd);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let bfc_box = find_child_by_node_id(&result.root, bfc_div).expect("bfc div found");
    (bfc_box.clone(), engine)
}

/// 左 float 面向侧正 margin 被钉位吃掉：lf ml=5 → x=25、w=75（chromium 探针同值，
/// 非 §10.3.3 字面 x=30 w=70）。
#[test]
fn r5217_lf_beside_drops_facing_positive_margin() {
    let (bfc_box, _) = pin_case(5.0, 0.0, FloatValue::Left);
    assert!(
        (bfc_box.x - 25.0).abs() < 0.5,
        "左缘应钉 band 左缘 25，实际 {}",
        bfc_box.x
    );
    assert!(
        (bfc_box.width - 75.0).abs() < 0.5,
        "宽应填满 band（75），实际 {}",
        bfc_box.width
    );
}

/// 左 float 小负 margin 落 band 钉位（同 margin:0）：lf ml=−30 → x=25、w=75
///（chromium 探针同值）。
#[test]
fn r5217_lf_beside_small_negative_pins_to_band() {
    let (bfc_box, _) = pin_case(-30.0, 0.0, FloatValue::Left);
    assert!(
        (bfc_box.x - 25.0).abs() < 0.5,
        "左缘应钉 band 左缘 25，实际 {}",
        bfc_box.x
    );
    assert!(
        (bfc_box.width - 75.0).abs() < 0.5,
        "宽应填满 band（75），实际 {}",
        bfc_box.width
    );
}

/// 左 float 负 margin 超阈（−80 < −(75+0.5)）下绕：x=ml=−80、w=CB−margin_h=180、
/// y=float 底 50。
#[test]
fn r5217_lf_negative_beyond_gap_wraps() {
    let (bfc_box, _) = pin_case(-80.0, 0.0, FloatValue::Left);
    assert!(
        (bfc_box.x - (-80.0)).abs() < 0.5,
        "下绕 x 应回 ml=−80，实际 {}",
        bfc_box.x
    );
    assert!(
        (bfc_box.width - 180.0).abs() < 0.5,
        "下绕宽应全 CB 解 180，实际 {}",
        bfc_box.width
    );
    assert!(
        (bfc_box.y - 50.0).abs() < 0.5,
        "下绕 y 应到 float 底 50，实际 {}",
        bfc_box.y
    );
}

/// 左 float 背侧正 margin 收窄宽：lf mr=15 → left 钉 25、right=CB−mr=85 → w=60
///（chromium gap=16 lf mr=5 → w=11 同模型）。
#[test]
fn r5217_lf_away_positive_margin_shrinks_width() {
    let (bfc_box, _) = pin_case(0.0, 15.0, FloatValue::Left);
    assert!(
        (bfc_box.x - 25.0).abs() < 0.5,
        "左缘应钉 band 左缘 25，实际 {}",
        bfc_box.x
    );
    assert!(
        (bfc_box.width - 60.0).abs() < 0.5,
        "宽应扣背侧 margin（85−25=60），实际 {}",
        bfc_box.width
    );
}

/// 左 float 大正面向 margin 越过 band 左缘仍旁置（左 float 无正 margin 下绕角）：
/// lf ml=85 → x=85、w=15（chromium gap=16 lf ml=28 → [28,2] 同模型）。
#[test]
fn r5217_lf_facing_positive_beyond_band_stays_beside() {
    let (bfc_box, _) = pin_case(85.0, 0.0, FloatValue::Left);
    assert!((bfc_box.x - 85.0).abs() < 0.5, "左缘应保持 ml=85，实际 {}", bfc_box.x);
    assert!(
        (bfc_box.width - 15.0).abs() < 0.5,
        "宽应扣至 CB（100−85=15），实际 {}",
        bfc_box.width
    );
}

/// 右 float 背侧（远离 float 一侧）正 margin 被尊重：rf ml=5 → x=5、right 钉 float 左缘
/// 75 → w=70（chromium 探针同值）。
#[test]
fn r5217_rf_beside_honors_away_margin() {
    let (bfc_box, _) = pin_case(5.0, 0.0, FloatValue::Right);
    assert!((bfc_box.x - 5.0).abs() < 0.5, "左缘应保持 ml=5，实际 {}", bfc_box.x);
    assert!(
        (bfc_box.width - 70.0).abs() < 0.5,
        "右缘应钉 float 左缘（75−5=70），实际 {}",
        bfc_box.width
    );
}

/// 右 float 大正 margin 使钉后宽 < frame → 下绕：rf ml=85 → x=85、w=15、y=float 底 50
///（chromium gap=16 rf ml=15 → 下绕 [15,15,12] 同模型；左 float 无此角，见上）。
#[test]
fn r5217_rf_positive_overrun_wraps() {
    let (bfc_box, _) = pin_case(85.0, 0.0, FloatValue::Right);
    assert!((bfc_box.x - 85.0).abs() < 0.5, "下绕 x 应回 ml=85，实际 {}", bfc_box.x);
    assert!(
        (bfc_box.width - 15.0).abs() < 0.5,
        "下绕宽应全 CB 解 15，实际 {}",
        bfc_box.width
    );
    assert!(
        (bfc_box.y - 50.0).abs() < 0.5,
        "下绕 y 应到 float 底 50，实际 {}",
        bfc_box.y
    );
}

/// 右 float 小负 margin 落 x=0、w=gap（同 margin:0）：rf ml=−30 → x=0、w=75。
#[test]
fn r5217_rf_beside_small_negative_pins_to_zero() {
    let (bfc_box, _) = pin_case(-30.0, 0.0, FloatValue::Right);
    assert!((bfc_box.x - 0.0).abs() < 0.5, "左缘应钳 0，实际 {}", bfc_box.x);
    assert!(
        (bfc_box.width - 75.0).abs() < 0.5,
        "宽应填满 gap（75），实际 {}",
        bfc_box.width
    );
}
