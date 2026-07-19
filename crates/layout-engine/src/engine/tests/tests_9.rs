// 边界条件和极端值测试 — engine 模块私有函数。
use super::*;
use crate::types::{LayoutBox, OverflowClip};
use zero_css_parser::values::OverflowValue;

// ── convert_overflow_to_clip 边界条件 ──

/// 测试 convert_overflow_to_clip：Visible 映射为 Visible。
#[test]
fn test_overflow_visible_round_trip() {
    let result = convert_overflow_to_clip(&OverflowValue::Visible);
    assert_eq!(result, OverflowClip::Visible);
    assert_eq!(result, OverflowClip::Visible, "Visible 应可复制且比较");
}

/// 测试 convert_overflow_to_clip：所有变体映射正确。
#[test]
fn test_overflow_all_variants_complete_mapping() {
    // Visible
    assert_eq!(convert_overflow_to_clip(&OverflowValue::Visible), OverflowClip::Visible);
    // Hidden
    assert_eq!(convert_overflow_to_clip(&OverflowValue::Hidden), OverflowClip::Hidden);
    // Clip
    assert_eq!(convert_overflow_to_clip(&OverflowValue::Clip), OverflowClip::Clip);
    // Scroll
    assert_eq!(convert_overflow_to_clip(&OverflowValue::Scroll), OverflowClip::Scroll);
    // Auto → Scroll
    assert_eq!(convert_overflow_to_clip(&OverflowValue::Auto), OverflowClip::Scroll);
}

// ── adjust_fixed_to_viewport 边界条件 ──

/// 测试 adjust_fixed_to_viewport：传入零偏移量时 fixed 元素坐标不变。
#[test]
fn test_adjust_fixed_zero_parent_offset() {
    let mut root = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);
    assert!((root.x - 0.0).abs() < 0.001, "零偏移 + 零坐标 = 0");
    assert!((root.y - 0.0).abs() < 0.001);
}

/// 测试 adjust_fixed_to_viewport：fixed 元素在负坐标父级中。
#[test]
fn test_adjust_fixed_negative_parent_offset() {
    let fixed_child = LayoutBox {
        node_id: None,
        x: 10.0,
        y: 10.0,
        width: 50.0,
        height: 50.0,
        content_x: 10.0,
        content_y: 10.0,
        content_width: 50.0,
        content_height: 50.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let mut root = LayoutBox {
        node_id: None,
        x: -100.0,
        y: -200.0,
        width: 800.0,
        height: 600.0,
        content_x: -100.0,
        content_y: -200.0,
        content_width: 800.0,
        content_height: 600.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![fixed_child],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // R324：fixed child 扣除负父级偏移（视口相对）：x = 10 - (-100) = 110, y = 10 - (-200) = 210
    // （painter 累积后绝对 = -100+110=10 / -200+210=10 = CSS left/top 视口相对）
    let child = &root.children[0];
    assert!(
        (child.x - 110.0).abs() < 0.001,
        "fixed child x 应为 110，实际 {}",
        child.x
    );
    assert!(
        (child.y - 210.0).abs() < 0.001,
        "fixed child y 应为 210，实际 {}",
        child.y
    );
}

/// 测试 adjust_fixed_to_viewport：两个连续 fixed 元素互不影响。
#[test]
fn test_adjust_fixed_sibling_fixed_elements() {
    let fixed1 = LayoutBox {
        node_id: None,
        x: 5.0,
        y: 5.0,
        width: 50.0,
        height: 50.0,
        content_x: 5.0,
        content_y: 5.0,
        content_width: 50.0,
        content_height: 50.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let fixed2 = LayoutBox {
        node_id: None,
        x: 100.0,
        y: 200.0,
        width: 50.0,
        height: 50.0,
        content_x: 100.0,
        content_y: 200.0,
        content_width: 50.0,
        content_height: 50.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let mut root = LayoutBox {
        node_id: None,
        x: 50.0,
        y: 50.0,
        width: 800.0,
        height: 600.0,
        content_x: 50.0,
        content_y: 50.0,
        content_width: 800.0,
        content_height: 600.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![fixed1, fixed2],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // R324：fixed1 扣除父级偏移（视口相对）：x = 5 - 50 = -45, y = 5 - 50 = -45
    let c1 = &root.children[0];
    assert!((c1.x - (-45.0)).abs() < 0.001, "fixed1 x 应为 -45，实际 {}", c1.x);
    assert!((c1.y - (-45.0)).abs() < 0.001, "fixed1 y 应为 -45，实际 {}", c1.y);

    // R324：fixed2 扣除父级偏移（视口相对）：x = 100 - 50 = 50, y = 200 - 50 = 150
    let c2 = &root.children[1];
    assert!((c2.x - 50.0).abs() < 0.001, "fixed2 x 应为 50，实际 {}", c2.x);
    assert!((c2.y - 150.0).abs() < 0.001, "fixed2 y 应为 150，实际 {}", c2.y);
}

/// 测试 adjust_fixed_to_viewport：fixed 元素包含 absolute 子元素。
///
/// fixed 元素的子元素不应被视作 fixed，它们的坐标不变。
/// 但由于 fixed 元素的 offset 归零，子元素的后续偏移应从 0 开始。
#[test]
fn test_adjust_fixed_with_absolute_child() {
    let abs_grandchild = LayoutBox {
        node_id: None,
        x: 20.0,
        y: 30.0,
        width: 50.0,
        height: 50.0,
        content_x: 20.0,
        content_y: 30.0,
        content_width: 50.0,
        content_height: 50.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: true,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let fixed_parent = LayoutBox {
        node_id: None,
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 200.0,
        content_x: 10.0,
        content_y: 20.0,
        content_width: 200.0,
        content_height: 200.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![abs_grandchild],
        is_absolute: false,
        is_fixed: true,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    let mut root = LayoutBox {
        node_id: None,
        x: 100.0,
        y: 200.0,
        width: 800.0,
        height: 600.0,
        content_x: 100.0,
        content_y: 200.0,
        content_width: 800.0,
        content_height: 600.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![fixed_parent],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    adjust_fixed_to_viewport(&mut root, 0.0, 0.0);

    // R324：fixed parent 扣除累积祖先偏移（视口相对）：x = 10 - 100 = -90, y = 20 - 200 = -180
    let fp = &root.children[0];
    assert!((fp.x - (-90.0)).abs() < 0.001, "fixed parent x 应为 -90");
    assert!((fp.y - (-180.0)).abs() < 0.001, "fixed parent y 应为 -180");

    // absolute grandchild: offset 从 fixed 归零后重新累加
    // 由于 fixed parent offset 归零，absolute child 以 0 为基，
    // 它自身的 x=20 不变（不是 fixed，所以坐标不被修改）
    let gc = &root.children[0].children[0];
    assert!((gc.x - 20.0).abs() < 0.001, "absolute child x 应为 20，实际 {}", gc.x);
    assert!((gc.y - 30.0).abs() < 0.001, "absolute child y 应为 30，实际 {}", gc.y);
}

/// 测试 adjust_fixed_to_viewport：空 children 不 panic。
#[test]
fn test_adjust_fixed_empty_children_no_panic() {
    let mut root = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        content_x: 0.0,
        content_y: 0.0,
        content_width: 800.0,
        content_height: 600.0,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed: false,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        overflow_y: OverflowClip::Visible,
        clear: zero_css_parser::values::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        ..Default::default()
    };
    // 不应 panic
    adjust_fixed_to_viewport(&mut root, 100.0, 200.0);
    assert!((root.x - 0.0).abs() < 0.001);
}

// ── has_direct_text 边界条件 ──

/// 测试 has_direct_text：空元素返回 false。
#[test]
fn test_has_direct_text_empty_element() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    assert!(!has_direct_text(&doc, div), "空 div 不应有直接文本");
}

/// 测试 has_direct_text：仅有空白文本的元素返回 false。
#[test]
fn test_has_direct_text_whitespace_only() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    let text = doc.create_text_node("   ");
    doc.append_child(div, text).unwrap();

    assert!(!has_direct_text(&doc, div), "仅有空白文本的 div 不应被视为有直接文本");
}

/// 测试 has_direct_text：有非空文本的元素返回 true。
#[test]
fn test_has_direct_text_with_content() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    let text = doc.create_text_node("Hello");
    doc.append_child(div, text).unwrap();

    assert!(has_direct_text(&doc, div), "有文本内容的 div 应返回 true");
}

// ── measure_text_content 边界条件 ──

/// 测试 measure_text_content：无文本节点时返回 Size::ZERO。
#[test]
fn test_measure_text_content_no_text() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let styles = HashMap::new();
    let size = measure_text_content(
        &doc,
        &styles,
        div,
        taffy::geometry::Size {
            width: None,
            height: None,
        },
        taffy::geometry::Size {
            width: taffy::style::AvailableSpace::Definite(800.0),
            height: taffy::style::AvailableSpace::Definite(600.0),
        },
        &HashMap::new(),
        None,
    );
    assert_eq!(size.width, 0.0, "无文本节点宽度应为 0");
    assert_eq!(size.height, 0.0, "无文本节点高度应为 0");
}

/// 测试 measure_text_content：MinContent 宽度应为最宽不可拆单元（单词），
/// 而非整行 max-content。R542 修复：此前 inline-content 分支对 MinContent 也用
/// INFINITY 宽 → 全部单词排一行 → measured_width = max-content（偏大）。R428
/// min-size:auto 默认后，grid/flex item 的 min-width 被这个偏大值 floor →
/// 卡片过宽（welcome.html +7.65pp 回归，R541 实证 min-width:0 可恢复）。
#[test]
fn test_measure_text_content_min_content_is_widest_word() {
    use zero_css_parser::values::LengthValue;
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    // "XX YYYY"：Ahem 10px 等宽（每字符=font_size）→ "XX"=20px, "YYYY"=40px。
    let text = doc.create_text_node("XX YYYY");
    doc.append_child(div, text).unwrap();

    let mut style = ComputedStyle::default();
    style.font_family = vec!["Ahem".to_string()];
    style.font_size = LengthValue::Px(10.0_f64);
    let mut styles = HashMap::new();
    styles.insert(div, style);

    let none_size = taffy::geometry::Size {
        width: None,
        height: None,
    };
    let min_size = measure_text_content(
        &doc,
        &styles,
        div,
        none_size,
        taffy::geometry::Size {
            width: taffy::style::AvailableSpace::MinContent,
            height: taffy::style::AvailableSpace::Definite(600.0),
        },
        &HashMap::new(),
        None,
    );
    // MinContent ≈ 最宽词 "YYYY"(40px)，远小于整行 max-content(~70px)。
    assert!(
        min_size.width < 55.0,
        "MinContent 应为最宽词(~40px) 而非整行 max-content，实际 {}",
        min_size.width
    );
    let max_size = measure_text_content(
        &doc,
        &styles,
        div,
        none_size,
        taffy::geometry::Size {
            width: taffy::style::AvailableSpace::MaxContent,
            height: taffy::style::AvailableSpace::Definite(600.0),
        },
        &HashMap::new(),
        None,
    );
    // MaxContent ≈ 整行 "XX YYYY"(~70px)。
    assert!(
        max_size.width > 60.0,
        "MaxContent 应为整行(~70px)，实际 {}",
        max_size.width
    );
    assert!(
        min_size.width < max_size.width,
        "MinContent({}) 应严格小于 MaxContent({})（修复前两者相等=max-content）",
        min_size.width,
        max_size.width
    );
}

/// R1578：measure path 经 `img_intrinsic_sizes` 推导 auto-width img 的缺失维度，
/// 解「inline 元素包裹 auto-width img 致父容器塌缩」（wintertc footer
/// `<p><a><img class="h-6"></a></p>`）。load-bearing：env ON 时 `<a>` measured
/// 高度 ≈ img height（img 经固有比推导 w 后参与行盒），env OFF 时塌缩（h_on > h_off）。
#[test]
fn test_r1578_measure_uses_img_intrinsic_for_auto_width_img() {
    use zero_css_parser::values::LengthValue;
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let p = doc.create_element("p");
    doc.append_child(body, p).unwrap();
    let a = doc.create_element("a");
    doc.append_child(p, a).unwrap();
    let img = doc.create_element("img");
    doc.append_child(a, img).unwrap();

    // img CSS height=24、width=auto（auto-width img）；`<a>`/img 默认 display:Inline。
    let mut img_style = ComputedStyle::default();
    img_style.height = LengthValue::Px(24.0);
    let mut styles = HashMap::new();
    styles.insert(img, img_style);

    // img 固有尺寸 75×32（matrix.svg BothAbs）→ 期望推导 w = 24 × 75/32 = 56.25。
    let mut intrinsic: HashMap<zero_dom::NodeId, (f32, f32)> = HashMap::new();
    intrinsic.insert(img, (75.0, 32.0));

    let none_size = taffy::geometry::Size::<Option<f32>> {
        width: None,
        height: None,
    };
    let avail = taffy::geometry::Size {
        width: taffy::style::AvailableSpace::MaxContent,
        height: taffy::style::AvailableSpace::Definite(600.0),
    };

    // ON：img 分支用固有比推导 w=56.25 → 产 item → `<a>` measured h≈24
    // SAFETY: 测试单线程内同步设置/读取；无其他测试并发读 ZW_IFC_IMG_INTRINSIC。
    unsafe {
        std::env::set_var("ZW_IFC_IMG_INTRINSIC", "1");
    }
    let h_on = measure_text_content(&doc, &styles, a, none_size, avail, &intrinsic, None).height;
    // OFF：不推导 → img 不收集（w=0）→ `<a>` 塌缩
    unsafe {
        std::env::set_var("ZW_IFC_IMG_INTRINSIC", "0");
    }
    let h_off = measure_text_content(&doc, &styles, a, none_size, avail, &intrinsic, None).height;

    assert!(
        h_on > 20.0,
        "R1578 ON: <a> measured height should be ~24 (img drives line box), got {h_on}"
    );
    assert!(
        h_on > h_off + 4.0,
        "R1578 load-bearing: ON ({h_on}) must exceed OFF ({h_off}) by >4px (else fix is no-op)"
    );

    // 清理 env，避免污染同进程后续测试
    unsafe {
        std::env::remove_var("ZW_IFC_IMG_INTRINSIC");
    }
}

/// R1311e REFUTED 回归守卫：非-Ahem `line-height:normal` 单行文本块 IFC leaf-measure
/// 必须返回 `font_size × NORMAL_LINE_HEIGHT_RATIO(1.164)`（= chromium 真值），无 +0.4px 过冲。
///
/// R1311e（2026-07-12）曾记录 inline-fold ON 时 instruction `<p>` leaf-measure 返 19.0
/// （vs container 18.6），归因为「Phase A leaf-measure 0.4px 高度过冲」并据此关闭 inline-fold。
/// R1779 经验复测（measure_text_content 直接调用）：当前 leaf-measure 对非-Ahem 16px 文本
/// 返回 **18.624 = 16×1.164 = chromium 真值**，全 font-size 线性 ratio=1.164，**无过冲**。
/// R1311e 19.0 为 stale（此后 line-height ratio 已固化为 DejaVu hhea 1.164）。本测试锁定该
/// 事实，防未来 ratio 改动重新引入过冲被误判为 leaf-measure blocker。
#[test]
fn test_leaf_measure_nonahem_lineheight_no_overshoot() {
    let none_size = taffy::geometry::Size::<Option<f32>> {
        width: None,
        height: None,
    };
    let avail = taffy::geometry::Size {
        width: taffy::style::AvailableSpace::Definite(800.0),
        height: taffy::style::AvailableSpace::Definite(600.0),
    };
    // 非注释 instruction `<p>`（text + inline <strong>）leaf-measure：必须 = 16×1.164 = 18.624。
    let (doc, styles, p, _strong) = build_instruction_p(16.0);
    let h = measure_text_content(&doc, &styles, p, none_size, avail, &HashMap::new(), None).height;
    assert!(
        (h - 18.624).abs() < 0.001,
        "非-Ahem 16px leaf-measure 应为 18.624(=16×1.164, chromium 真值)，实际 {h}；若 >18.9 说明重新引入 R1311e 过冲"
    );
    // 纯文本块扫描：ratio 必须恒为 1.164（线性，无 fs 相关过冲）。
    for fs in [10.0_f32, 12.0, 14.0, 16.0, 20.0, 24.0, 32.0] {
        let (d, s, n) = build_nonahem_text_block("Test passes", fs);
        let m = measure_text_content(&d, &s, n, none_size, avail, &HashMap::new(), None).height;
        let expected = fs * 1.164;
        assert!(
            (m - expected).abs() < 0.001,
            "fs={fs}: leaf-measure {m} 应 = {expected}(×1.164)，ratio={:.5}",
            m / fs
        );
    }
}

fn build_nonahem_text_block(
    text: &str,
    fs: f32,
) -> (Document, HashMap<zero_dom::NodeId, ComputedStyle>, zero_dom::NodeId) {
    use zero_css_parser::values::LengthValue;
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();
    let text = doc.create_text_node(text);
    doc.append_child(div, text).unwrap();

    let mut style = ComputedStyle::default();
    style.font_family = vec!["DejaVu Sans".to_string()];
    style.font_size = LengthValue::Px(fs as f64);
    let mut styles = HashMap::new();
    styles.insert(div, style);
    (doc, styles, div)
}

fn build_instruction_p(
    fs: f32,
) -> (
    Document,
    HashMap<zero_dom::NodeId, ComputedStyle>,
    zero_dom::NodeId,
    zero_dom::NodeId,
) {
    use zero_css_parser::values::LengthValue;
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let p = doc.create_element("p");
    doc.append_child(body, p).unwrap();
    let t1 = doc.create_text_node("Test passes if there are 2 identical filled green rectangles and ");
    doc.append_child(p, t1).unwrap();
    let strong = doc.create_element("strong");
    doc.append_child(p, strong).unwrap();
    let t2 = doc.create_text_node("no red");
    doc.append_child(strong, t2).unwrap();
    let t3 = doc.create_text_node(".");
    doc.append_child(p, t3).unwrap();

    let mut p_style = ComputedStyle::default();
    p_style.font_family = vec!["DejaVu Sans".to_string()];
    p_style.font_size = LengthValue::Px(fs as f64);
    let mut strong_style = ComputedStyle::default();
    strong_style.font_family = vec!["DejaVu Sans".to_string()];
    strong_style.font_size = LengthValue::Px(fs as f64);
    let mut styles = HashMap::new();
    styles.insert(p, p_style);
    styles.insert(strong, strong_style);
    (doc, styles, p, strong)
}

/// 测试 adjust_absolute_pct_to_viewport：无 positioned ancestor 的 absolute 元素，
/// `top`/`left` 为长度（Px）时应解析为视口相对坐标（CSS 2.1 §10.1）。
///
/// taffy 用静态父作 containing block，会把 top:118px 解析为父相对坐标。
/// 修复后应转为视口相对：child.y = top_px - current_content_origin_y。
#[test]
fn test_adjust_absolute_length_top_to_viewport() {
    use std::collections::HashMap;
    use zero_css_parser::values::LengthValue;
    let (_doc, key_id) = make_doc_with_body();

    let abs_child = LayoutBox {
        node_id: Some(key_id),
        x: 8.0,
        y: 118.0, // taffy 设置的父相对坐标（= top 值）
        width: 100.0,
        height: 50.0,
        is_absolute: true,
        ..Default::default()
    };
    let body = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        children: vec![abs_child],
        ..Default::default()
    };
    let mut root = LayoutBox {
        children: vec![body],
        ..Default::default()
    };

    let mut style = ComputedStyle::default();
    style.top = LengthValue::Px(118.0);
    style.left = LengthValue::Px(8.0);
    let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
    styles.insert(key_id, style);

    // current_content_origin_y = 8（body margin），模拟视口偏移
    adjust_absolute_pct_to_viewport(&mut root, 0.0, 8.0, 800.0, 600.0, &styles, false);

    let abs = &root.children[0].children[0];
    // 视口相对：top:118px → child.y = 118 - 8(origin) = 110
    assert!(
        (abs.y - 110.0).abs() < 0.001,
        "abs Length top 应为视口相对（118 - origin 8 = 110），实际 y={}",
        abs.y
    );
    // left:8px → child.x = 8 - 0(origin) = 8
    assert!(
        (abs.x - 8.0).abs() < 0.001,
        "abs Length left 应为视口相对，实际 x={}",
        abs.x
    );
}

/// R1227：abspos（无 positioned 祖先，CB=viewport）`width:%`/`height:%` + border 须按
/// box-sizing 解析。content-box（默认）下 `width:50%` 指 content，border-box = content +
/// border；旧代码把 `%` 当 border-box 丢 border 致 border-box 偏小（abspos-containing-
/// block-initial-009e/009f：body abspos width:50% + border:10px 旧渲 400 非 420）。
#[test]
fn test_adjust_absolute_pct_box_sizing_border() {
    use std::collections::HashMap;
    use zero_css_parser::values::{BoxSizingValue, LengthValue};
    let (_doc, key_id) = make_doc_with_body();

    let abs_child = LayoutBox {
        node_id: Some(key_id),
        x: 50.0,
        y: 50.0,
        width: 0.0,
        height: 0.0,
        // taffy 已填的 border（10px 四边）—— postprocess 在 extract 后运行
        border_left: 10.0,
        border_right: 10.0,
        border_top: 10.0,
        border_bottom: 10.0,
        is_absolute: true,
        ..Default::default()
    };
    let body = LayoutBox {
        node_id: None,
        width: 800.0,
        height: 600.0,
        children: vec![abs_child],
        ..Default::default()
    };
    let mut root = LayoutBox {
        children: vec![body],
        ..Default::default()
    };

    let mut style = ComputedStyle::default();
    style.width = LengthValue::Percentage(50.0);
    style.height = LengthValue::Percentage(50.0);
    style.box_sizing = BoxSizingValue::ContentBox; // 默认：width:% 指 content
    let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
    styles.insert(key_id, style);

    adjust_absolute_pct_to_viewport(&mut root, 0.0, 0.0, 800.0, 600.0, &styles, false);

    let abs = &root.children[0].children[0];
    // content-box：content = 50%×800 = 400，border-box = 400 + 10+10 = 420
    assert!(
        (abs.width - 420.0).abs() < 0.001,
        "content-box width:50% + border 10px → border-box 420，实际 {}",
        abs.width
    );
    assert!(
        (abs.content_width - 400.0).abs() < 0.001,
        "content_width 应为 400（420 − 20 border），实际 {}",
        abs.content_width
    );
    // height:50% → content 300，border-box 320
    assert!(
        (abs.height - 320.0).abs() < 0.001,
        "content-box height:50% + border 10px → border-box 320，实际 {}",
        abs.height
    );
    assert!(
        (abs.content_height - 300.0).abs() < 0.001,
        "content_height 应为 300（320 − 20 border），实际 {}",
        abs.content_height
    );

    // border-box：width:% 指 border-box 直接，content = border-box − border
    let mut style = ComputedStyle::default();
    style.width = LengthValue::Percentage(50.0);
    style.height = LengthValue::Percentage(50.0);
    style.box_sizing = BoxSizingValue::BorderBox;
    styles.insert(key_id, style);
    // 重置初值
    root.children[0].children[0].width = 0.0;
    root.children[0].children[0].height = 0.0;
    adjust_absolute_pct_to_viewport(&mut root, 0.0, 0.0, 800.0, 600.0, &styles, false);
    let abs = &root.children[0].children[0];
    assert!(
        (abs.width - 400.0).abs() < 0.001,
        "border-box width:50% → border-box 400，实际 {}",
        abs.width
    );
    assert!(
        (abs.content_width - 380.0).abs() < 0.001,
        "border-box content_width 应为 380（400 − 20 border），实际 {}",
        abs.content_width
    );
}

/// 测试 §10.3.7：abspos（无 positioned 祖先，CB=viewport）width:auto + left+right +
/// max-width 时，max-width 钳制填满宽，两侧 auto-margin 居中（对应 WPT
/// absolute-non-replaced-width-025）。taffy 0.7 不钳 abspos inset-fill 宽。
#[test]
fn test_adjust_absolute_maxwidth_clamp_center() {
    use std::collections::HashMap;
    use zero_css_parser::values::LengthValue;
    let (_doc, key_id) = make_doc_with_body();

    let abs_child = LayoutBox {
        node_id: Some(key_id),
        x: 8.0,
        y: 0.0,
        width: 100.0, // 初值；函数会先 stretch 到 784 再 clamp
        height: 100.0,
        is_absolute: true,
        ..Default::default()
    };
    let body = LayoutBox {
        node_id: None,
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
        children: vec![abs_child],
        ..Default::default()
    };
    let mut root = LayoutBox {
        children: vec![body],
        ..Default::default()
    };

    let mut style = ComputedStyle::default();
    style.width = LengthValue::Auto;
    style.left = LengthValue::Px(8.0);
    style.right = LengthValue::Px(8.0);
    style.max_width = LengthValue::Px(100.0);
    style.margin_left = LengthValue::Auto;
    style.margin_right = LengthValue::Auto;
    let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
    styles.insert(key_id, style);

    adjust_absolute_pct_to_viewport(&mut root, 0.0, 0.0, 800.0, 600.0, &styles, false);

    let abs = &root.children[0].children[0];
    // max-width 钳制到 100
    assert!(
        (abs.width - 100.0).abs() < 0.001,
        "width 应被 max-width 钳到 100，实际 {}",
        abs.width
    );
    // leftover = 800-8-8-100 = 684，居中 → margin 342，target_viewport_x=8+342=350
    assert!(
        (abs.x - 350.0).abs() < 0.001,
        "两侧 auto-margin 应居中到 x=350，实际 {}",
        abs.x
    );
    assert!((abs.margin_left - 342.0).abs() < 0.001, "margin_left 应为 342");
    assert!((abs.margin_right - 342.0).abs() < 0.001, "margin_right 应为 342");
}

/// 测试 §10.3.7：max-width 钳制后，仅 margin-left auto 时吸收 leftover（右对齐，
/// 对应 WPT absolute-non-replaced-width-026）。
#[test]
fn test_adjust_absolute_maxwidth_clamp_margin_left_auto() {
    use std::collections::HashMap;
    use zero_css_parser::values::LengthValue;
    let (_doc, key_id) = make_doc_with_body();

    let abs_child = LayoutBox {
        node_id: Some(key_id),
        x: 8.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
        is_absolute: true,
        ..Default::default()
    };
    let body = LayoutBox {
        node_id: None,
        width: 800.0,
        height: 600.0,
        children: vec![abs_child],
        ..Default::default()
    };
    let mut root = LayoutBox {
        children: vec![body],
        ..Default::default()
    };

    let mut style = ComputedStyle::default();
    style.width = LengthValue::Auto;
    style.left = LengthValue::Px(8.0);
    style.right = LengthValue::Px(8.0);
    style.max_width = LengthValue::Px(100.0);
    style.margin_left = LengthValue::Auto;
    style.margin_right = LengthValue::Px(0.0); // 仅 margin-left auto
    let mut styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
    styles.insert(key_id, style);

    adjust_absolute_pct_to_viewport(&mut root, 0.0, 0.0, 800.0, 600.0, &styles, false);

    let abs = &root.children[0].children[0];
    assert!((abs.width - 100.0).abs() < 0.001, "width 钳到 100");
    // margin-left 吸收 leftover 684 → target_viewport_x = 8 + 684 = 692
    assert!(
        (abs.x - 692.0).abs() < 0.001,
        "仅 margin-left auto 应右对齐到 x=692，实际 {}",
        abs.x
    );
    assert!((abs.margin_left - 684.0).abs() < 0.001, "margin_left 应为 684");
}
