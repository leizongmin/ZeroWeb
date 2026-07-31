//! CSS 解析器覆盖率补充测试：color.rs, parse_transform.rs, types.rs, parse_extended.rs。

use crate::values::hwb_to_rgba;
use crate::values::{
    ColorValue, LengthValue, QuotesValue, parse_appearance, parse_background_attachment, parse_background_clip,
    parse_background_origin, parse_background_repeat, parse_background_repeat_list, parse_background_size,
    parse_background_size_list, parse_border_collapse, parse_border_image_outset, parse_border_image_repeat,
    parse_border_image_slice, parse_border_image_source, parse_border_image_width, parse_box_shadow, parse_caret_color,
    parse_color, parse_column_count, parse_column_width, parse_contain, parse_content, parse_counter_list,
    parse_filter, parse_filter_list, parse_gradient, parse_grid_area, parse_hyphens, parse_line_clamp,
    parse_list_style_image, parse_mix_blend_mode, parse_object_fit, parse_quotes, parse_scrollbar_gutter,
    parse_scrollbar_width, parse_table_layout, parse_text_overflow, parse_text_shadow, parse_text_wrap,
    parse_will_change,
};

// ═══════════════════════════════════════════════════════════════════════
// color.rs — 十六进制边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_hex_invalid_lengths() {
    assert!(parse_color("#1").is_none());
    assert!(parse_color("#12").is_none());
    assert!(parse_color("#1234").is_some()); // 4-char = RGBA
    assert!(parse_color("#12345").is_none());
    assert!(parse_color("#1234567").is_none());
    assert!(parse_color("#123456789").is_none());
}

#[test]
fn test_hex_invalid_chars() {
    // 非 hex 字符 → hex 解析失败返回 None（from_str_radix.ok()? 会失败）
    // 注意：hex_char_to_byte 用 unwrap_or(0) 所以不会 panic，但 parse_hex_color 的 6-char 路径用的是 .ok()?
    let c = parse_color("#xyzabc");
    assert!(c.is_none());
}

#[test]
fn test_hex_3_char_various() {
    let c = parse_color("#0f0").unwrap();
    assert_eq!(c, ColorValue::Rgba(0, 255, 0, 255));
    let c = parse_color("#00f").unwrap();
    assert_eq!(c, ColorValue::Rgba(0, 0, 255, 255));
}

#[test]
fn test_hex_8_char_with_alpha() {
    let c = parse_color("#ff000040").unwrap();
    if let ColorValue::Rgba(r, g, b, a) = c {
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
        assert_eq!(a, 64);
    } else {
        panic!("expected Rgba");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// color.rs — rgb() 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_rgb_percentage_clamp_high() {
    let c = parse_color("rgb(200%, 200%, 200%)").unwrap();
    if let ColorValue::Rgba(r, g, b, _a) = c {
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 255);
    }
}

#[test]
fn test_rgb_value_clamp() {
    let c = parse_color("rgb(300, 300, 300)").unwrap();
    if let ColorValue::Rgba(r, g, b, _a) = c {
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 255);
    }
}

#[test]
fn test_rgb_alpha_percentage() {
    let c = parse_color("rgba(255, 0, 0, 50%)").unwrap();
    if let ColorValue::Rgba(_r, _g, _b, a) = c {
        assert_eq!(a, 128); // 50% of 255 = 127.5 → 128
    }
}

#[test]
fn test_rgb_with_spaces() {
    let c = parse_color("rgb( 255 , 128 , 0 )").unwrap();
    if let ColorValue::Rgba(r, g, b, _a) = c {
        assert_eq!(r, 255);
        assert_eq!(g, 128);
        assert_eq!(b, 0);
    }
}

#[test]
fn test_rgb_invalid_non_numeric() {
    assert!(parse_color("rgb(abc, def, ghi)").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// color.rs — hsl() 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_hsl_with_deg() {
    let c = parse_color("hsl(90deg, 50%, 50%)").unwrap();
    if let ColorValue::Hsla(h, s, l, a) = c {
        assert_eq!(h, 90.0);
        assert_eq!(s, 50.0);
        assert_eq!(l, 50.0);
        assert_eq!(a, 1.0);
    }
}

#[test]
fn test_hsla_with_alpha() {
    let c = parse_color("hsla(180, 75%, 25%, 0.3)").unwrap();
    if let ColorValue::Hsla(h, s, l, a) = c {
        assert_eq!(h, 180.0);
        assert_eq!(s, 75.0);
        assert_eq!(l, 25.0);
        assert!((a - 0.3).abs() < 0.001);
    }
}

#[test]
fn test_hsl_too_few_parts() {
    assert!(parse_color("hsl(120)").is_none());
    assert!(parse_color("hsl(120, 50%)").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// color.rs — hwb() 和 hwb_to_rgba 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_hwb_basic() {
    let c = parse_color("hwb(0 0% 0%)").unwrap();
    if let ColorValue::Rgba(r, g, b, _a) = c {
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }
}

#[test]
fn test_hwb_with_alpha() {
    let c = parse_color("hwb(120 20% 30% / 0.5)").unwrap();
    if let ColorValue::Rgba(_r, _g, _b, a) = c {
        assert_eq!(a, 128); // 0.5 * 255 ≈ 128
    }
}

#[test]
fn test_hwb_alpha_percentage() {
    let c = parse_color("hwb(240 0% 100% / 50%)");
    assert!(c.is_some());
}

#[test]
fn test_hwb_too_few_parts() {
    assert!(parse_color("hwb(0 0%)").is_none());
    assert!(parse_color("hwb(0)").is_none());
}

#[test]
fn test_hwb_to_rgba_all_sectors() {
    // Sector 0: h=0..60
    let (r, g, b, _a) = hwb_to_rgba(0.0, 0.0, 0.0, 1.0);
    assert_eq!(r, 255);
    assert_eq!(g, 0);
    assert_eq!(b, 0);

    // Sector 1: h=60..120
    let (r, g, b, _a) = hwb_to_rgba(60.0, 0.0, 0.0, 1.0);
    assert_eq!(r, 255);
    assert_eq!(g, 255);
    assert_eq!(b, 0);

    // Sector 2: h=120..180
    let (r, _g, b, _a) = hwb_to_rgba(120.0, 0.0, 0.0, 1.0);
    assert_eq!(r, 0);
    assert_eq!(b, 0);

    // Sector 3: h=180..240
    let (r, _g, _b, _a) = hwb_to_rgba(180.0, 0.0, 0.0, 1.0);
    assert_eq!(r, 0);

    // Sector 4: h=240..300
    let (_r, g, b, _a) = hwb_to_rgba(240.0, 0.0, 0.0, 1.0);
    assert_eq!(g, 0);
    assert_eq!(b, 255);

    // Sector 5: h=300..360
    let (r, g, b, _a) = hwb_to_rgba(300.0, 0.0, 0.0, 1.0);
    assert_eq!(r, 255);
    assert_eq!(g, 0);
    assert_eq!(b, 255);
}

#[test]
fn test_hwb_to_rgba_w_plus_b_over_1() {
    // W+B > 1.0 触发缩放分支
    let (r, g, b, _a) = hwb_to_rgba(0.0, 0.8, 0.8, 1.0);
    assert!(r <= 255);
    assert!(g <= 255);
    assert!(b <= 255);
}

#[test]
fn test_hwb_to_rgba_with_clamped_w_b() {
    // W 和 B 超出 [0,1] 范围被钳制
    let (r, g, b, _a) = hwb_to_rgba(0.0, -0.5, 1.5, 1.0);
    assert!(r <= 255);
    assert!(g <= 255);
    assert!(b <= 255);
}

#[test]
fn test_hwb_to_rgba_hue_wraparound() {
    // h > 360 → h % 360
    let (r1, g1, b1, _) = hwb_to_rgba(360.0, 0.0, 0.0, 1.0);
    let (r2, g2, b2, _) = hwb_to_rgba(0.0, 0.0, 0.0, 1.0);
    assert_eq!(r1, r2);
    assert_eq!(g1, g2);
    assert_eq!(b1, b2);
}

// ═══════════════════════════════════════════════════════════════════════
// color.rs — 命名颜色和关键字
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_named_colors_mixed_case() {
    let c1 = parse_color("BLUE").unwrap();
    let c2 = parse_color("blue").unwrap();
    let c3 = parse_color("Blue").unwrap();
    assert_eq!(c1, c2);
    assert_eq!(c2, c3);
}

#[test]
fn test_named_color_grey() {
    let c = parse_color("grey").unwrap();
    if let ColorValue::Rgba(r, g, b, _a) = c {
        assert_eq!(r, 128);
        assert_eq!(g, 128);
        assert_eq!(b, 128);
    }
}

#[test]
fn test_named_color_fuchsia() {
    let c = parse_color("fuchsia").unwrap();
    if let ColorValue::Rgba(r, g, b, _a) = c {
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 255);
    }
}

#[test]
fn test_currentcolor_keyword() {
    assert_eq!(parse_color("currentcolor"), Some(ColorValue::CurrentColor));
    assert_eq!(parse_color("CURRENTCOLOR"), Some(ColorValue::CurrentColor));
}

#[test]
fn test_transparent_keyword_mixed_case() {
    assert_eq!(parse_color("Transparent"), Some(ColorValue::Transparent));
}

#[test]
fn test_unknown_named_color() {
    assert!(parse_color("notarealcolor").is_none());
    assert!(parse_color("").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_transform.rs — gradient 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_gradient_linear_directions() {
    assert!(parse_gradient("linear-gradient(to top left, red, blue)").is_some());
    assert!(parse_gradient("linear-gradient(to right bottom, red, blue)").is_some());
    assert!(parse_gradient("linear-gradient(to bottom right, red, blue)").is_some());
}

#[test]
fn test_gradient_radial_with_ellipse() {
    assert!(parse_gradient("radial-gradient(ellipse, red, blue)").is_some());
}

#[test]
fn test_gradient_radial_with_closest_side() {
    assert!(parse_gradient("radial-gradient(closest-side, red, blue)").is_some());
}

#[test]
fn test_gradient_radial_with_farthest_side() {
    assert!(parse_gradient("radial-gradient(farthest-side, red, blue)").is_some());
}

#[test]
fn test_gradient_conic_with_at_position() {
    assert!(parse_gradient("conic-gradient(at 50% 50%, red, blue)").is_some());
}

#[test]
fn test_gradient_conic_from_and_at() {
    assert!(parse_gradient("conic-gradient(from 90deg at 0% 100%, red, blue)").is_some());
}

#[test]
fn test_gradient_conic_at_only() {
    assert!(parse_gradient("conic-gradient(at top left, red, blue)").is_some());
}

#[test]
fn test_gradient_invalid_not_a_gradient() {
    assert!(parse_gradient("not-a-gradient()").is_none());
    assert!(parse_gradient("").is_none());
}

#[test]
fn test_gradient_color_with_position() {
    assert!(parse_gradient("linear-gradient(red 10px, blue 20px)").is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_transform.rs — grid_area 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_grid_area_single_value() {
    let result = parse_grid_area("header").unwrap();
    assert_eq!(result.0, "header");
    assert_eq!(result.3, "header");
}

#[test]
fn test_grid_area_two_values() {
    let result = parse_grid_area("1 / 3").unwrap();
    assert_eq!(result.0, "1");
    assert_eq!(result.2, "3");
}

#[test]
fn test_grid_area_three_values() {
    let result = parse_grid_area("1 / 2 / 3").unwrap();
    assert_eq!(result.0, "1");
    assert_eq!(result.1, "2");
    assert_eq!(result.2, "3");
    assert_eq!(result.3, "auto");
}

#[test]
fn test_grid_area_four_values() {
    let result = parse_grid_area("1 / 2 / 3 / 4").unwrap();
    assert_eq!(result, ("1".into(), "2".into(), "3".into(), "4".into()));
}

#[test]
fn test_grid_area_empty() {
    assert!(parse_grid_area("").is_none());
    assert!(parse_grid_area("   ").is_none());
}

#[test]
fn test_grid_area_slash_empty_after() {
    assert!(parse_grid_area("1 /").is_none());
    assert!(parse_grid_area("/ 2").is_none());
}

#[test]
fn test_grid_area_too_many_values() {
    assert!(parse_grid_area("1 / 2 / 3 / 4 / 5").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_transform.rs — text-shadow / box-shadow 补充
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_text_shadow_with_color_no_blur() {
    let s = parse_text_shadow("2px 3px red").unwrap();
    assert!(matches!(s.blur_radius, LengthValue::Px(0.0)));
}

#[test]
fn test_text_shadow_with_blur_and_color() {
    let s = parse_text_shadow("2px 3px 4px blue").unwrap();
    // blur=4px, color=blue
    if let ColorValue::Rgba(r, g, b, _a) = s.color {
        assert_eq!(r, 0);
        assert_eq!(g, 0);
        assert_eq!(b, 255);
    }
}

#[test]
fn test_box_shadow_with_spread() {
    let s = parse_box_shadow("2px 3px 4px 5px red").unwrap();
    assert!(!s.inset);
}

#[test]
fn test_box_shadow_inset_with_all() {
    let s = parse_box_shadow("inset 1px 2px 3px 4px black").unwrap();
    assert!(s.inset);
}

// ═══════════════════════════════════════════════════════════════════════
// parse_extended.rs — parse_contain 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_contain_none() {
    use crate::values::ContainValue;
    assert_eq!(parse_contain("none"), Some(ContainValue::None));
    assert_eq!(parse_contain("strict"), Some(ContainValue::Strict));
    assert_eq!(parse_contain("content"), Some(ContainValue::Content));
    assert_eq!(parse_contain("size"), Some(ContainValue::Size));
    assert_eq!(parse_contain("layout"), Some(ContainValue::Layout));
    assert_eq!(parse_contain("style"), Some(ContainValue::Style));
    assert_eq!(parse_contain("paint"), Some(ContainValue::Paint));
}

#[test]
fn test_contain_multiple_keywords() {
    let c = parse_contain("layout paint");
    assert!(c.is_some());
    let c = parse_contain("size layout style paint");
    assert!(c.is_some());
}

#[test]
fn test_contain_invalid() {
    assert!(parse_contain("invalid").is_none());
    assert!(parse_contain("layout invalid").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_extended.rs — parse_filter 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_filter_none() {
    use crate::values::FilterValue;
    assert_eq!(parse_filter("none"), Some(FilterValue::None));
}

#[test]
fn test_filter_blur() {
    let f = parse_filter("blur(5px)");
    assert!(f.is_some());
}

#[test]
fn test_filter_blur_no_unit() {
    // CSS Filter Effects：blur() 取 `<length>?`，无单位非零值无效（与 Chromium 一致）。
    // 修复前 ZW 宽容地把 `blur(3)` 当 3px（偏离 spec）。
    let f = parse_filter("blur(3)");
    assert!(f.is_none(), "blur(3) 无单位非零应无效");
}

#[test]
fn test_filter_blur_empty_arg_and_bare_zero() {
    use crate::values::FilterValue;
    // blur() 空参 = blur(0)（CSS Filter Effects：`<length>?` 缺省 0）。修复前 ZW 返回 None。
    assert!(matches!(parse_filter("blur()"), Some(FilterValue::Blur(0.0))));
    // 裸 0 是合法 `<length>`（CSS Values unitless-zero）。
    assert!(matches!(parse_filter("blur(0)"), Some(FilterValue::Blur(0.0))));
    // 0px 同。
    assert!(matches!(parse_filter("blur(0px)"), Some(FilterValue::Blur(0.0))));
}

#[test]
fn test_filter_brightness() {
    let f = parse_filter("brightness(1.5)");
    assert!(f.is_some());
}

#[test]
fn test_filter_brightness_percentage() {
    let f = parse_filter("brightness(150%)");
    assert!(f.is_some());
}

#[test]
fn test_filter_contrast() {
    let f = parse_filter("contrast(0.8)");
    assert!(f.is_some());
}

#[test]
fn test_filter_grayscale() {
    let f = parse_filter("grayscale(100%)");
    assert!(f.is_some());
}

#[test]
fn test_filter_hue_rotate_deg() {
    let f = parse_filter("hue-rotate(90deg)");
    assert!(f.is_some());
}

#[test]
fn test_filter_hue_rotate_rad() {
    let f = parse_filter("hue-rotate(1.5708rad)");
    assert!(f.is_some());
}

#[test]
fn test_filter_hue_rotate_turn() {
    let f = parse_filter("hue-rotate(0.25turn)");
    assert!(f.is_some());
}

#[test]
fn test_filter_hue_rotate_plain() {
    let f = parse_filter("hue-rotate(90)");
    assert!(f.is_some());
}

#[test]
fn test_filter_invert() {
    let f = parse_filter("invert(0.5)");
    assert!(f.is_some());
}

#[test]
fn test_filter_opacity() {
    let f = parse_filter("opacity(0.8)");
    assert!(f.is_some());
}

#[test]
fn test_filter_saturate() {
    let f = parse_filter("saturate(2.0)");
    assert!(f.is_some());
}

#[test]
fn test_filter_sepia() {
    let f = parse_filter("sepia(0.7)");
    assert!(f.is_some());
}

#[test]
fn test_filter_drop_shadow_basic() {
    let f = parse_filter("drop-shadow(1 2 3 black)");
    assert!(f.is_some());
}

#[test]
fn test_filter_drop_shadow_no_blur() {
    let f = parse_filter("drop-shadow(1 2 red)");
    assert!(f.is_some());
}

#[test]
fn test_filter_missing_close_paren() {
    assert!(parse_filter("blur(5px").is_none());
}

#[test]
fn test_filter_invalid_name() {
    assert!(parse_filter("unknown(5px)").is_none());
}

#[test]
fn test_filter_no_paren() {
    assert!(parse_filter("blur").is_none());
}

// ── R2306：parse_filter_list — 多函数列表（CSS Filter Effects：none | <filter-function>+）──

#[test]
fn test_filter_list_none_is_empty() {
    let list = parse_filter_list("none").expect("none → Some(空 Vec)");
    assert!(list.is_empty(), "none 应解析为空 filter 列表");
}

#[test]
fn test_filter_list_single() {
    let list = parse_filter_list("blur(5px)").expect("单函数 → Some");
    assert_eq!(list.len(), 1);
}

#[test]
fn test_filter_list_multiple_space() {
    // 顶层空格分割：3 个独立 filter 函数（CSS filter 用空格分隔，非逗号）
    let list = parse_filter_list("blur(5px) brightness(1.5) sepia(0.5)").expect("多函数 → Some");
    assert_eq!(list.len(), 3, "应拆为 3 个 filter 函数");
}

#[test]
fn test_filter_list_drop_shadow_internal_spaces_preserved() {
    // paren-aware：drop-shadow 的参数内空格不应拆分 → 仍是 2 个函数
    let list = parse_filter_list("drop-shadow(2 4 red) blur(3px)").expect("含空格参数 → Some");
    assert_eq!(
        list.len(),
        2,
        "drop-shadow(2 4 red) 内部空格必须保持一体，应为 2 个函数"
    );
}

#[test]
fn test_filter_list_any_invalid_is_none() {
    // 任意单个函数解析失败 → 整列表 None
    assert!(parse_filter_list("blur(5px) bogus brightness(1.5)").is_none());
}

#[test]
fn test_filter_list_empty_is_none() {
    // 空字符串 / 纯空白 → None
    assert!(parse_filter_list("").is_none());
    assert!(parse_filter_list("   ").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_extended.rs — parse_counter_list 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_counter_list_none() {
    let result = parse_counter_list("none").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_counter_list_empty() {
    assert!(parse_counter_list("").is_none());
    assert!(parse_counter_list("   ").is_none());
}

#[test]
fn test_counter_list_with_values() {
    let result = parse_counter_list("section 1 subsection 2").unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_counter_list_name_is_none_rejected() {
    assert!(parse_counter_list("none 5").is_none());
}

#[test]
fn test_counter_list_multiple() {
    let result = parse_counter_list("a 1 b 2 c").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].name, "a");
    assert_eq!(result[0].value, Some(1));
    assert_eq!(result[2].name, "c");
    assert_eq!(result[2].value, None);
}

// ═══════════════════════════════════════════════════════════════════════
// parse_extended.rs — parse_content 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_content_string() {
    use crate::values::ContentValue;
    let c = parse_content("\"hello world\"").unwrap();
    assert_eq!(c, ContentValue::String("hello world".to_string()));
}

#[test]
fn test_content_empty_string() {
    let c = parse_content("\"\"");
    // 空引号返回空字符串内容
    assert!(c.is_some());
}

#[test]
fn test_content_attr() {
    use crate::values::ContentValue;
    let c = parse_content("attr(data-label)").unwrap();
    assert_eq!(c, ContentValue::Attr("data-label".to_string()));
}

#[test]
fn test_content_attr_empty() {
    assert!(parse_content("attr()").is_none());
}

#[test]
fn test_content_counter() {
    use crate::values::ContentValue;
    let c = parse_content("counter(section)").unwrap();
    if let ContentValue::Counter { name, style } = c {
        assert_eq!(name, "section");
        assert!(style.is_none());
    }
}

#[test]
fn test_content_counter_with_style() {
    use crate::values::ContentValue;
    let c = parse_content("counter(section, upper-roman)").unwrap();
    if let ContentValue::Counter { name, style } = c {
        assert_eq!(name, "section");
        assert_eq!(style.unwrap(), "upper-roman");
    }
}

#[test]
fn test_content_counter_empty() {
    assert!(parse_content("counter()").is_none());
}

#[test]
fn test_content_normal_and_none() {
    use crate::values::ContentValue;
    assert_eq!(parse_content("normal"), Some(ContentValue::Normal));
    assert_eq!(parse_content("none"), Some(ContentValue::None));
}

#[test]
fn test_content_single_quotes() {
    let c = parse_content("'hello'");
    assert!(c.is_some());
}

/// R1988：`content: url(...)` 解析为 `ContentValue::Url`（generated content image）。
#[test]
fn test_content_url() {
    use crate::values::ContentValue;
    assert_eq!(
        parse_content("url(icon.png)"),
        Some(ContentValue::Url("icon.png".to_string()))
    );
    // 引号包裹的 url。
    assert_eq!(
        parse_content("url('bullet.svg')"),
        Some(ContentValue::Url("bullet.svg".to_string()))
    );
    assert_eq!(
        parse_content(r#"url("x/y.gif")"#),
        Some(ContentValue::Url("x/y.gif".to_string()))
    );
    // 空 url() → None。
    assert!(parse_content("url()").is_none());
}

#[test]
fn test_content_invalid() {
    assert!(parse_content("something-else").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_extended.rs — parse_quotes 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_quotes_none_and_auto() {
    use crate::values::QuotesValue;
    assert_eq!(parse_quotes("none"), Some(QuotesValue::None));
    assert_eq!(parse_quotes("auto"), Some(QuotesValue::Auto));
}

#[test]
fn test_quotes_single_pair() {
    let q = parse_quotes("\"«\" \"»\"").unwrap();
    if let QuotesValue::Pairs(pairs) = q {
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "«");
        assert_eq!(pairs[0].1, "»");
    }
}

#[test]
fn test_quotes_multiple_pairs() {
    let q = parse_quotes("\"«\" \"»\" \"‹\" \"›\"").unwrap();
    if let QuotesValue::Pairs(pairs) = q {
        assert_eq!(pairs.len(), 2);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// parse_extended.rs — border-image-slice 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_border_image_slice_single() {
    let s = parse_border_image_slice("10").unwrap();
    assert!(s.fill == false);
}

#[test]
fn test_border_image_slice_with_fill() {
    let s = parse_border_image_slice("10 fill").unwrap();
    assert!(s.fill);
}

#[test]
fn test_border_image_slice_percentage() {
    let s = parse_border_image_slice("10% 20% 30% 40%").unwrap();
    assert!(s.fill == false);
}

#[test]
fn test_border_image_slice_negative_rejected() {
    assert!(parse_border_image_slice("-10").is_none());
    assert!(parse_border_image_slice("-5%").is_none());
}

#[test]
fn test_border_image_slice_too_many() {
    assert!(parse_border_image_slice("1 2 3 4 5").is_none());
}

#[test]
fn test_border_image_slice_empty() {
    assert!(parse_border_image_slice("").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_extended.rs — border-image-width 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_border_image_width_auto() {
    let w = parse_border_image_width("auto").unwrap();
    use crate::values::BorderImageWidthComponent;
    assert!(matches!(w.top, BorderImageWidthComponent::Auto));
}

#[test]
fn test_border_image_width_number() {
    let w = parse_border_image_width("3").unwrap();
    use crate::values::BorderImageWidthComponent;
    assert!(matches!(w.top, BorderImageWidthComponent::Number(3.0)));
}

#[test]
fn test_border_image_width_percentage() {
    let w = parse_border_image_width("10%").unwrap();
    use crate::values::BorderImageWidthComponent;
    assert!(matches!(w.top, BorderImageWidthComponent::Percent(10.0)));
}

#[test]
fn test_border_image_width_px() {
    let w = parse_border_image_width("10px").unwrap();
    use crate::values::BorderImageWidthComponent;
    assert!(matches!(w.top, BorderImageWidthComponent::Length(_)));
}

#[test]
fn test_border_image_width_negative_rejected() {
    assert!(parse_border_image_width("-5").is_none());
    assert!(parse_border_image_width("-10%").is_none());
}

#[test]
fn test_border_image_width_empty() {
    assert!(parse_border_image_width("").is_none());
}

#[test]
fn test_border_image_width_too_many() {
    assert!(parse_border_image_width("1 2 3 4 5").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_extended.rs — border-image-repeat 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_border_image_repeat_single() {
    use crate::values::BorderImageRepeatMode;
    let r = parse_border_image_repeat("stretch").unwrap();
    assert!(matches!(r.horizontal, BorderImageRepeatMode::Stretch));
    assert!(matches!(r.vertical, BorderImageRepeatMode::Stretch));
}

#[test]
fn test_border_image_repeat_two_values() {
    use crate::values::BorderImageRepeatMode;
    let r = parse_border_image_repeat("repeat round").unwrap();
    assert!(matches!(r.horizontal, BorderImageRepeatMode::Repeat));
    assert!(matches!(r.vertical, BorderImageRepeatMode::Round));
}

#[test]
fn test_border_image_repeat_space() {
    let r = parse_border_image_repeat("space").unwrap();
    use crate::values::BorderImageRepeatMode;
    assert!(matches!(r.horizontal, BorderImageRepeatMode::Space));
}

#[test]
fn test_border_image_repeat_empty() {
    assert!(parse_border_image_repeat("").is_none());
}

#[test]
fn test_border_image_repeat_invalid() {
    assert!(parse_border_image_repeat("invalid").is_none());
}

#[test]
fn test_border_image_repeat_too_many() {
    assert!(parse_border_image_repeat("stretch repeat round").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_extended.rs — border-image-outset 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_border_image_outset_number() {
    let o = parse_border_image_outset("2").unwrap();
    use crate::values::BorderImageOutsetComponent;
    assert!(matches!(o.top, BorderImageOutsetComponent::Number(2.0)));
}

#[test]
fn test_border_image_outset_px() {
    let o = parse_border_image_outset("10px").unwrap();
    use crate::values::BorderImageOutsetComponent;
    assert!(matches!(o.top, BorderImageOutsetComponent::Length(_)));
}

#[test]
fn test_border_image_outset_four_values() {
    let o = parse_border_image_outset("1 2 3 4").unwrap();
    use crate::values::BorderImageOutsetComponent;
    assert!(matches!(o.top, BorderImageOutsetComponent::Number(1.0)));
}

#[test]
fn test_border_image_outset_negative_rejected() {
    assert!(parse_border_image_outset("-1").is_none());
}

#[test]
fn test_border_image_outset_empty() {
    assert!(parse_border_image_outset("").is_none());
}

#[test]
fn test_border_image_outset_too_many() {
    assert!(parse_border_image_outset("1 2 3 4 5").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_extended.rs — 其他函数边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_column_count_auto_and_number() {
    use crate::values::ColumnCountValue;
    assert_eq!(parse_column_count("auto"), Some(ColumnCountValue::Auto));
    assert_eq!(parse_column_count("3"), Some(ColumnCountValue::Number(3)));
}

#[test]
fn test_column_count_zero() {
    assert!(parse_column_count("0").is_none());
}

#[test]
fn test_column_width_auto_and_px() {
    use crate::values::ColumnWidthValue;
    assert_eq!(parse_column_width("auto"), Some(ColumnWidthValue::Auto));
    let w = parse_column_width("100px");
    assert!(w.is_some());
}

#[test]
fn test_object_fit_values() {
    use crate::values::ObjectFitValue;
    assert_eq!(parse_object_fit("fill"), Some(ObjectFitValue::Fill));
    assert_eq!(parse_object_fit("contain"), Some(ObjectFitValue::Contain));
    assert_eq!(parse_object_fit("cover"), Some(ObjectFitValue::Cover));
    assert_eq!(parse_object_fit("none"), Some(ObjectFitValue::None));
    assert_eq!(parse_object_fit("scale-down"), Some(ObjectFitValue::ScaleDown));
    assert!(parse_object_fit("invalid").is_none());
}

#[test]
fn test_appearance_values() {
    use crate::values::AppearanceValue;
    assert_eq!(parse_appearance("none"), Some(AppearanceValue::None));
    assert_eq!(parse_appearance("auto"), Some(AppearanceValue::Auto));
    assert_eq!(parse_appearance("button"), Some(AppearanceValue::Button));
    assert_eq!(parse_appearance("checkbox"), Some(AppearanceValue::Checkbox));
    assert_eq!(parse_appearance("menulist"), Some(AppearanceValue::Menulist));
    assert!(parse_appearance("invalid").is_none());
}

#[test]
fn test_mix_blend_mode_values() {
    use crate::values::MixBlendModeValue;
    assert_eq!(parse_mix_blend_mode("normal"), Some(MixBlendModeValue::Normal));
    assert_eq!(parse_mix_blend_mode("multiply"), Some(MixBlendModeValue::Multiply));
    assert_eq!(parse_mix_blend_mode("screen"), Some(MixBlendModeValue::Screen));
    assert_eq!(parse_mix_blend_mode("overlay"), Some(MixBlendModeValue::Overlay));
    assert!(parse_mix_blend_mode("invalid").is_none());
}

#[test]
fn test_scrollbar_width_values() {
    use crate::values::ScrollbarWidthValue;
    assert_eq!(parse_scrollbar_width("auto"), Some(ScrollbarWidthValue::Auto));
    assert_eq!(parse_scrollbar_width("thin"), Some(ScrollbarWidthValue::Thin));
    assert_eq!(parse_scrollbar_width("none"), Some(ScrollbarWidthValue::None));
    assert!(parse_scrollbar_width("thick").is_none());
}

#[test]
fn test_scrollbar_gutter_values() {
    use crate::values::ScrollbarGutterValue;
    assert_eq!(parse_scrollbar_gutter("auto"), Some(ScrollbarGutterValue::Auto));
    assert_eq!(parse_scrollbar_gutter("stable"), Some(ScrollbarGutterValue::Stable));
    assert_eq!(
        parse_scrollbar_gutter("stable both-edges"),
        Some(ScrollbarGutterValue::StableBothEdges)
    );
}

#[test]
fn test_text_wrap_values() {
    use crate::values::TextWrapValue;
    assert_eq!(parse_text_wrap("wrap"), Some(TextWrapValue::Wrap));
    assert_eq!(parse_text_wrap("nowrap"), Some(TextWrapValue::Nowrap));
    assert_eq!(parse_text_wrap("balance"), Some(TextWrapValue::Balance));
    assert_eq!(parse_text_wrap("pretty"), Some(TextWrapValue::Pretty));
    assert!(parse_text_wrap("invalid").is_none());
}

#[test]
fn test_hyphens_values() {
    use crate::values::HyphensValue;
    assert_eq!(parse_hyphens("none"), Some(HyphensValue::None));
    assert_eq!(parse_hyphens("manual"), Some(HyphensValue::Manual));
    assert_eq!(parse_hyphens("auto"), Some(HyphensValue::Auto));
    assert!(parse_hyphens("invalid").is_none());
}

#[test]
fn test_line_clamp_none_and_number() {
    use crate::values::LineClampValue;
    assert_eq!(parse_line_clamp("none"), Some(LineClampValue::None));
    assert_eq!(parse_line_clamp("3"), Some(LineClampValue::Count(3)));
}

#[test]
fn test_line_clamp_zero_rejected() {
    assert!(parse_line_clamp("0").is_none());
}

#[test]
fn test_line_clamp_negative_rejected() {
    assert!(parse_line_clamp("-1").is_none());
}

#[test]
fn test_background_image_url() {
    use crate::values::BackgroundImageValue;
    let b = crate::values::parse_background_image("url(image.png)").unwrap();
    assert!(matches!(b, BackgroundImageValue::Url(_)));
}

#[test]
fn test_background_image_gradient() {
    use crate::values::BackgroundImageValue;
    let b = crate::values::parse_background_image("linear-gradient(red, blue)").unwrap();
    assert!(matches!(b, BackgroundImageValue::Gradient(_)));
}

#[test]
fn test_background_image_invalid() {
    assert!(crate::values::parse_background_image("invalid").is_none());
    assert!(crate::values::parse_background_image("").is_none());
}

#[test]
fn test_background_repeat_values() {
    use crate::values::BackgroundRepeatValue;
    assert_eq!(parse_background_repeat("repeat"), Some(BackgroundRepeatValue::Repeat));
    assert_eq!(
        parse_background_repeat("no-repeat"),
        Some(BackgroundRepeatValue::NoRepeat)
    );
    assert_eq!(
        parse_background_repeat("repeat-x"),
        Some(BackgroundRepeatValue::RepeatX)
    );
    assert_eq!(
        parse_background_repeat("repeat-y"),
        Some(BackgroundRepeatValue::RepeatY)
    );
    assert_eq!(parse_background_repeat("space"), Some(BackgroundRepeatValue::Space));
    assert_eq!(parse_background_repeat("round"), Some(BackgroundRepeatValue::Round));
}

#[test]
fn test_background_size_values() {
    use crate::values::BackgroundSizeValue;
    assert_eq!(parse_background_size("cover"), Some(BackgroundSizeValue::Cover));
    assert_eq!(parse_background_size("contain"), Some(BackgroundSizeValue::Contain));
    assert_eq!(parse_background_size("auto"), Some(BackgroundSizeValue::Auto));
    assert_eq!(parse_background_size("100px"), Some(BackgroundSizeValue::Length(100.0)));
    assert_eq!(parse_background_size("50%"), Some(BackgroundSizeValue::Percent(50.0)));
    assert!(parse_background_size("invalid").is_none());
}

#[test]
fn test_background_attachment_values() {
    use crate::values::BackgroundAttachmentValue;
    assert_eq!(
        parse_background_attachment("scroll"),
        Some(BackgroundAttachmentValue::Scroll)
    );
    assert_eq!(
        parse_background_attachment("fixed"),
        Some(BackgroundAttachmentValue::Fixed)
    );
    assert_eq!(
        parse_background_attachment("local"),
        Some(BackgroundAttachmentValue::Local)
    );
}

#[test]
fn test_background_clip_values() {
    use crate::values::BackgroundClipValue;
    assert_eq!(
        parse_background_clip("border-box"),
        Some(BackgroundClipValue::BorderBox)
    );
    assert_eq!(
        parse_background_clip("padding-box"),
        Some(BackgroundClipValue::PaddingBox)
    );
    assert_eq!(
        parse_background_clip("content-box"),
        Some(BackgroundClipValue::ContentBox)
    );
}

#[test]
fn test_background_origin_values() {
    use crate::values::BackgroundOriginValue;
    assert_eq!(
        parse_background_origin("border-box"),
        Some(BackgroundOriginValue::BorderBox)
    );
    assert_eq!(
        parse_background_origin("padding-box"),
        Some(BackgroundOriginValue::PaddingBox)
    );
    assert_eq!(
        parse_background_origin("content-box"),
        Some(BackgroundOriginValue::ContentBox)
    );
}

#[test]
fn test_border_image_source_none() {
    use crate::values::BorderImageSourceValue;
    assert_eq!(parse_border_image_source("none"), Some(BorderImageSourceValue::None));
}

#[test]
fn test_border_image_source_url() {
    use crate::values::BorderImageSourceValue;
    let r = parse_border_image_source("url(border.png)");
    assert!(matches!(r, Some(BorderImageSourceValue::Url(_))));
}

#[test]
fn test_list_style_image_none() {
    use crate::values::ListStyleImageValue;
    assert_eq!(parse_list_style_image("none"), Some(ListStyleImageValue::None));
}

#[test]
fn test_list_style_image_url() {
    use crate::values::ListStyleImageValue;
    let r = parse_list_style_image("url(bullet.png)");
    assert!(matches!(r, Some(ListStyleImageValue::Url(_))));
}

#[test]
fn test_caret_color_values() {
    use crate::values::CaretColorValue;
    assert_eq!(parse_caret_color("auto"), Some(CaretColorValue::Auto));
    let c = parse_caret_color("red");
    assert!(matches!(c, Some(CaretColorValue::Color(_))));
}

#[test]
fn test_will_change_values() {
    use crate::values::WillChangeValue;
    assert_eq!(parse_will_change("auto"), Some(WillChangeValue::Auto));
    assert_eq!(
        parse_will_change("scroll-position"),
        Some(WillChangeValue::ScrollPosition)
    );
    assert_eq!(parse_will_change("contents"), Some(WillChangeValue::Contents));
    let r = parse_will_change("transform");
    assert!(matches!(r, Some(WillChangeValue::Custom(_))));
}

#[test]
fn test_text_overflow_values() {
    use crate::values::TextOverflowValue;
    assert_eq!(parse_text_overflow("clip"), Some(TextOverflowValue::Clip));
    assert_eq!(parse_text_overflow("ellipsis"), Some(TextOverflowValue::Ellipsis));
}

#[test]
fn test_text_overflow_custom_string() {
    use crate::values::TextOverflowValue;
    let r = parse_text_overflow("\"...\"");
    assert!(matches!(r, Some(TextOverflowValue::String(_))));
}

#[test]
fn test_table_layout_values() {
    use crate::values::TableLayoutValue;
    assert_eq!(parse_table_layout("auto"), Some(TableLayoutValue::Auto));
    assert_eq!(parse_table_layout("fixed"), Some(TableLayoutValue::Fixed));
    assert!(parse_table_layout("invalid").is_none());
}

#[test]
fn test_border_collapse_values() {
    use crate::values::BorderCollapseValue;
    assert_eq!(parse_border_collapse("separate"), Some(BorderCollapseValue::Separate));
    assert_eq!(parse_border_collapse("collapse"), Some(BorderCollapseValue::Collapse));
}

// R2311：background 多层 longhand list 解析（`<position>#` / `<repeat-style>#` / `<bg-size>#`）

#[test]
fn test_background_repeat_list_multi_layer() {
    use crate::values::BackgroundRepeatValue;
    // 单层 byte-identical（1 项 Vec）
    assert_eq!(
        parse_background_repeat_list("repeat"),
        Some(vec![BackgroundRepeatValue::Repeat])
    );
    assert_eq!(
        parse_background_repeat_list("no-repeat"),
        Some(vec![BackgroundRepeatValue::NoRepeat])
    );
    // 多层逗号分隔
    assert_eq!(
        parse_background_repeat_list("repeat, no-repeat"),
        Some(vec![BackgroundRepeatValue::Repeat, BackgroundRepeatValue::NoRepeat])
    );
    assert_eq!(
        parse_background_repeat_list("repeat-x, space, round"),
        Some(vec![
            BackgroundRepeatValue::RepeatX,
            BackgroundRepeatValue::Space,
            BackgroundRepeatValue::Round
        ])
    );
    // 任一层失败 → None
    assert_eq!(parse_background_repeat_list("repeat, bogus"), None);
    // 空输入 → None
    assert_eq!(parse_background_repeat_list(""), None);
}

#[test]
fn test_background_size_list_multi_layer() {
    use crate::values::BackgroundSizeValue;
    // 单层
    assert_eq!(
        parse_background_size_list("cover"),
        Some(vec![BackgroundSizeValue::Cover])
    );
    assert_eq!(
        parse_background_size_list("auto"),
        Some(vec![BackgroundSizeValue::Auto])
    );
    assert_eq!(
        parse_background_size_list("50%"),
        Some(vec![BackgroundSizeValue::Percent(50.0)])
    );
    // 多层逗号分隔
    assert_eq!(
        parse_background_size_list("cover, 100px, contain"),
        Some(vec![
            BackgroundSizeValue::Cover,
            BackgroundSizeValue::Length(100.0),
            BackgroundSizeValue::Contain
        ])
    );
    // 任一层失败 → None
    assert_eq!(parse_background_size_list("cover, bogus"), None);
    assert_eq!(parse_background_size_list(""), None);
}

#[test]
fn test_background_position_list_multi_layer() {
    use crate::values::parse_background_position_list;
    // 单层 → 1 项 Vec
    assert_eq!(parse_background_position_list("center").map(|v| v.len()), Some(1));
    // 多层逗号分隔；单层内 "left top"（空格）保持一体非两层
    assert_eq!(
        parse_background_position_list("center, left top").map(|v| v.len()),
        Some(2)
    );
    assert_eq!(
        parse_background_position_list("top, center, bottom").map(|v| v.len()),
        Some(3)
    );
    // 任一层失败 → None
    assert_eq!(parse_background_position_list("center, bogus"), None);
    // 空输入 → None
    assert_eq!(parse_background_position_list(""), None);
}

/// R2313：background-position 的 calc()/min()/max()/clamp() 数学函数解析。
#[test]
fn test_background_position_calc_math_functions() {
    use crate::values::{BackgroundPositionValue, parse_background_position};
    // 单值 calc/min/max/clamp → Calc
    assert!(matches!(
        parse_background_position("calc(50%)"),
        Some(BackgroundPositionValue::Calc(_))
    ));
    assert!(matches!(
        parse_background_position("min(0%, 100%)"),
        Some(BackgroundPositionValue::Calc(_))
    ));
    assert!(matches!(
        parse_background_position("max(0%, 100%)"),
        Some(BackgroundPositionValue::Calc(_))
    ));
    assert!(matches!(
        parse_background_position("clamp(0%, 50%, 100%)"),
        Some(BackgroundPositionValue::Calc(_))
    ));
    // 两值 min/max（paren-aware 拆分，内部空格保持一体）→ TwoValue(Calc, Calc)
    let two = parse_background_position("min(0%, 100%) max(0%, 100%)").expect("两值 min/max");
    match two {
        BackgroundPositionValue::TwoValue(h, v) => {
            assert!(matches!(*h, BackgroundPositionValue::Calc(_)), "水平分量应为 Calc");
            assert!(matches!(*v, BackgroundPositionValue::Calc(_)), "垂直分量应为 Calc");
        }
        other => panic!("两值 min/max 应为 TwoValue(Calc, Calc)，got {other:?}"),
    }
    // 单值回归（非 calc 不受影响）
    assert!(matches!(
        parse_background_position("center"),
        Some(BackgroundPositionValue::Center)
    ));
    assert!(matches!(
        parse_background_position("50%"),
        Some(BackgroundPositionValue::Percent(50.0))
    ));
    assert!(matches!(
        parse_background_position("left top"),
        Some(BackgroundPositionValue::TwoValue(_, _))
    ));
    // 非法 calc → None
    assert!(parse_background_position("calc()").is_none());
}
