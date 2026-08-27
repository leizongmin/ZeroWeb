//! CSS content-visibility 样式管线测试（R2251，CSS Containment Module Level 2）。
//!
//! 验证 content-visibility 解析进入 ComputedStyle（apply 分发）。

use super::super::*;

#[test]
fn test_content_visibility_apply_hidden() {
    let mut style = ComputedStyle::default();
    // 初始值 = Visible
    assert!(matches!(style.content_visibility, ContentVisibilityValue::Visible));
    assert!(apply_property_value(&mut style, "content-visibility", "hidden"));
    assert!(matches!(style.content_visibility, ContentVisibilityValue::Hidden));
}

#[test]
fn test_content_visibility_apply_auto_and_visible() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "content-visibility", "auto"));
    assert!(matches!(style.content_visibility, ContentVisibilityValue::Auto));
    assert!(apply_property_value(&mut style, "content-visibility", "visible"));
    assert!(matches!(style.content_visibility, ContentVisibilityValue::Visible));
}

#[test]
fn test_content_visibility_apply_invalid_rejected() {
    let mut style = ComputedStyle::default();
    // 非法值不应用（保持默认 Visible），返回 false
    assert!(!apply_property_value(&mut style, "content-visibility", "inherit"));
    assert!(matches!(style.content_visibility, ContentVisibilityValue::Visible));
}

#[test]
/// content-visibility:hidden 的「跳过内容」效果仅对 size-containment 适用盒生效（CSS Containment 2）。
/// 无主盒（display:none/contents）与非替换 inline 盒无效。WPT driving：
/// content-visibility-on-display-contents / content-visibility-on-ruby / content-visibility-073。
fn test_content_visibility_hidden_effective_display_gate() {
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "content-visibility", "hidden");

    // 默认 display:inline → 非原子 inline，content-visibility:hidden 无视觉效果。
    assert!(!style.content_visibility_hidden_effective(), "inline: 无效果");

    style.display = DisplayValue::Block;
    assert!(style.content_visibility_hidden_effective(), "block: 生效");

    style.display = DisplayValue::InlineBlock;
    assert!(
        style.content_visibility_hidden_effective(),
        "inline-block（atomic inline）: 生效"
    );

    style.display = DisplayValue::Contents;
    assert!(!style.content_visibility_hidden_effective(), "contents: 无主盒无效");

    style.display = DisplayValue::None;
    assert!(!style.content_visibility_hidden_effective(), "none: 无主盒无效");

    // visible/auto 永不生效
    style.display = DisplayValue::Block;
    apply_property_value(&mut style, "content-visibility", "visible");
    assert!(!style.content_visibility_hidden_effective());
    apply_property_value(&mut style, "content-visibility", "auto");
    assert!(
        !style.content_visibility_hidden_effective(),
        "auto 静态等价 visible，不跳过"
    );
}

#[test]
/// R2256 contain-intrinsic-size 解析（CSS Sizing 4）。
fn test_contain_intrinsic_size_parse() {
    use zero_css_parser::values::LengthValue;
    let mut style = ComputedStyle::default();
    assert!(style.contain_intrinsic_width.is_none() && style.contain_intrinsic_height.is_none());

    // 1 length → 双维
    assert!(apply_property_value(&mut style, "contain-intrinsic-size", "100px"));
    assert_eq!(style.contain_intrinsic_width, Some(LengthValue::Px(100.0)));
    assert_eq!(style.contain_intrinsic_height, Some(LengthValue::Px(100.0)));

    // 2 lengths → width height
    assert!(apply_property_value(
        &mut style,
        "contain-intrinsic-size",
        "111px 222px"
    ));
    assert_eq!(style.contain_intrinsic_width, Some(LengthValue::Px(111.0)));
    assert_eq!(style.contain_intrinsic_height, Some(LengthValue::Px(222.0)));

    assert!(!apply_property_value(
        &mut style,
        "contain-intrinsic-size",
        "100px bogus"
    ));
    assert_eq!(style.contain_intrinsic_width, Some(LengthValue::Px(111.0)));
    assert_eq!(style.contain_intrinsic_height, Some(LengthValue::Px(222.0)));
    for value in [
        "-1px",
        "10%",
        "thin",
        "min-content",
        "fit-content(10px)",
        "infpx",
        "NaNpx",
    ] {
        assert!(!apply_property_value(&mut style, "contain-intrinsic-size", value));
        assert_eq!(style.contain_intrinsic_width, Some(LengthValue::Px(111.0)));
        assert_eq!(style.contain_intrinsic_height, Some(LengthValue::Px(222.0)));
    }

    // none → 清空
    assert!(apply_property_value(&mut style, "contain-intrinsic-size", "none"));
    assert!(style.contain_intrinsic_width.is_none() && style.contain_intrinsic_height.is_none());

    // longhands
    assert!(apply_property_value(&mut style, "contain-intrinsic-width", "50px"));
    assert_eq!(style.contain_intrinsic_width, Some(LengthValue::Px(50.0)));
    assert!(apply_property_value(&mut style, "contain-intrinsic-height", "75px"));
    assert_eq!(style.contain_intrinsic_height, Some(LengthValue::Px(75.0)));
    assert!(apply_property_value(&mut style, "contain-intrinsic-width", "none"));
    assert!(style.contain_intrinsic_width.is_none());
    assert!(apply_property_value(&mut style, "contain-intrinsic-width", "50px"));
    for value in ["-1px", "10%", "thin", "min-content", "infpx", "NaNpx"] {
        assert!(!apply_property_value(&mut style, "contain-intrinsic-width", value));
        assert_eq!(style.contain_intrinsic_width, Some(LengthValue::Px(50.0)));
    }
}

#[test]
/// R3751：contain-intrinsic-* 接受 CSS math 长度（CSS Sizing 4 `[<length> | auto]{1,2}`
/// 的 <length-percentage> math 形式；shorthand 组件切分须括号感知）。
fn r3751_contain_intrinsic_math_lengths() {
    use zero_css_parser::values::LengthValue;
    let mut style = ComputedStyle::default();

    // shorthand：math 函数内部空白不是组件边界。
    assert!(apply_property_value(
        &mut style,
        "contain-intrinsic-size",
        "calc(100px + 2em) min(300px, 50vh)"
    ));
    assert!(matches!(style.contain_intrinsic_width, Some(LengthValue::Calc(_))));
    assert!(matches!(style.contain_intrinsic_height, Some(LengthValue::Calc(_))));

    // longhand：math 写入；纯 number / 混类型 / 未闭合 math 拒绝且不覆盖。
    assert!(apply_property_value(
        &mut style,
        "contain-intrinsic-width",
        "clamp(100px, 50vw, 300px)"
    ));
    assert!(matches!(style.contain_intrinsic_width, Some(LengthValue::Calc(_))));

    let previous_w = style.contain_intrinsic_width.clone();
    let previous_h = style.contain_intrinsic_height.clone();
    for value in ["calc(5)", "clamp(100px, 5, 300px)", "calc(100px +"] {
        assert!(!apply_property_value(&mut style, "contain-intrinsic-width", value));
        assert_eq!(style.contain_intrinsic_width, previous_w);
        assert!(!apply_property_value(&mut style, "contain-intrinsic-height", value));
        assert_eq!(style.contain_intrinsic_height, previous_h);
    }
}

/// R2462 系统审计：`contain-intrinsic-width`/`-height` 长手属性非默认继承，但显式
/// `inherit` 关键字（CSS wide keyword）须经 inherit_property 从父元素复制（同 box-shadow
/// R2462 gap class）。driving: contain-intrinsic-size 子树用 `inherit` 的 case。
#[test]
fn test_contain_intrinsic_longhands_explicit_inherit_keyword() {
    use zero_css_parser::values::LengthValue;
    // 父元素分别设置 width / height 长手
    let mut parent = ComputedStyle::default();
    parent.contain_intrinsic_width = Some(LengthValue::Px(120.0));
    parent.contain_intrinsic_height = Some(LengthValue::Px(80.0));

    // 子元素显式 inherit：inherit_property 须识别两个长手并复制父值
    let mut child = ComputedStyle::default();
    assert!(
        inherit_property(&parent, &mut child, "contain-intrinsic-width"),
        "contain-intrinsic-width 须在 inherit_property 中有 case（显式 inherit 关键字）"
    );
    assert_eq!(child.contain_intrinsic_width, parent.contain_intrinsic_width);

    let mut child2 = ComputedStyle::default();
    assert!(
        inherit_property(&parent, &mut child2, "contain-intrinsic-height"),
        "contain-intrinsic-height 须在 inherit_property 中有 case（显式 inherit 关键字）"
    );
    assert_eq!(child2.contain_intrinsic_height, parent.contain_intrinsic_height);
}

/// R2468：contain-intrinsic-{inline,block}-size logical longhands（CSS Sizing 4
/// §intrinsic-size-override）。inline→width、block→height（水平书写模式等价；垂直模式轴
/// 交换由 converter swap_writing_mode_axes 负责，同 inline-size/block-size）。driving:
/// css-sizing/contain-intrinsic-size/contain-intrinsic-size-logical-001.html。
#[test]
fn test_contain_intrinsic_logical_longhands() {
    use zero_css_parser::values::LengthValue;
    let mut style = ComputedStyle::default();
    assert!(style.contain_intrinsic_width.is_none() && style.contain_intrinsic_height.is_none());

    // inline-size → width
    assert!(apply_property_value(
        &mut style,
        "contain-intrinsic-inline-size",
        "100px"
    ));
    assert_eq!(style.contain_intrinsic_width, Some(LengthValue::Px(100.0)));
    assert!(style.contain_intrinsic_height.is_none());

    // block-size → height
    assert!(apply_property_value(&mut style, "contain-intrinsic-block-size", "50px"));
    assert_eq!(style.contain_intrinsic_height, Some(LengthValue::Px(50.0)));

    // 可选 `auto` 前缀（静态无 remembered-size，按显式长度处理）
    assert!(apply_property_value(
        &mut style,
        "contain-intrinsic-inline-size",
        "auto 200px"
    ));
    assert_eq!(style.contain_intrinsic_width, Some(LengthValue::Px(200.0)));
    assert!(apply_property_value(
        &mut style,
        "contain-intrinsic-inline-size",
        "auto none"
    ));
    assert!(style.contain_intrinsic_width.is_none());
    assert!(apply_property_value(
        &mut style,
        "contain-intrinsic-inline-size",
        "auto 200px"
    ));
    for value in ["-1px", "10%", "thin", "max-content", "infpx", "NaNpx"] {
        assert!(!apply_property_value(
            &mut style,
            "contain-intrinsic-inline-size",
            value
        ));
        assert_eq!(style.contain_intrinsic_width, Some(LengthValue::Px(200.0)));
    }

    // 显式 inherit 关键字（logical longhands 与物理同 inherit_property 分支）
    let mut parent = ComputedStyle::default();
    parent.contain_intrinsic_width = Some(LengthValue::Px(300.0));
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "contain-intrinsic-inline-size"));
    assert_eq!(child.contain_intrinsic_width, parent.contain_intrinsic_width);
}
