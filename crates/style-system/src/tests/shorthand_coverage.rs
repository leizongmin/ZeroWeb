// ═══════════════════════════════════════════════════════════════════
// shorthand 覆盖率测试
//
// 通过完整管线（StyleSystem::compute_styles）测试简写属性展开，
// 覆盖 shorthand::expand_shorthands 中的各分支。
// ═══════════════════════════════════════════════════════════════════

use super::super::*;
use super::helpers::*;

/// 辅助：通过完整管线计算样式
fn compute_style(doc: &zero_dom::Document, element: zero_dom::NodeId, declarations: &[(&str, &str)]) -> ComputedStyle {
    let rules: Vec<zero_css_parser::ast::Rule> =
        vec![zero_css_parser::ast::Rule::Style(zero_css_parser::ast::StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: declarations
                .iter()
                .map(|(p, v)| zero_css_parser::ast::Declaration {
                    property: (*p).to_string(),
                    value: (*v).to_string(),
                    important: false,
                })
                .collect(),
        })];
    let stylesheets = vec![zero_css_parser::Stylesheet { rules }];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(doc, &stylesheets);
    styles.get(&element).cloned().unwrap_or_default()
}

#[test]
/// border-image 简写 - "none" 值
fn test_border_image_shorthand_none() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("border-image", "none")]);
    // border-image: none → 默认值
    assert_eq!(s.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// border-image 简写 - url 和切片值
fn test_border_image_shorthand_with_url() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("border-image", "url(img.png) 30")]);
    assert_eq!(s.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// transition 简写使用 cubic-bezier 时序函数
fn test_transition_shorthand_with_cubic_bezier() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(
        &doc,
        div,
        &[("transition", "all 0.3s cubic-bezier(0.1, 0.7, 1.0, 0.1) 0.1s")],
    );
    assert!(!s.transition_property.is_empty());
}

#[test]
/// animation 简写使用名称和持续时间
fn test_animation_shorthand_with_name_and_duration() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("animation", "slide 2s ease-in-out infinite alternate")]);
    assert!(!s.animation_name.is_empty());
}

#[test]
/// columns 简写使用宽度和数量组合
fn test_columns_shorthand_with_width_and_count() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("columns", "3 200px")]);
    assert_eq!(
        s.column_count,
        crate::property::types::ColumnCountComputedValue::Number(3)
    );
    assert_eq!(
        s.column_width,
        crate::property::types::ColumnWidthComputedValue::Length(LengthValue::Px(200.0))
    );
}

#[test]
/// grid-area 简写测试 - 4 个值
fn test_grid_area_shorthand_four_values() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("grid-area", "1 / 2 / 3 / 4")]);
    assert_eq!(s.grid_row_start, crate::property::GridLineValue::Line(1));
    assert_eq!(s.grid_row_end, crate::property::GridLineValue::Line(3));
    assert_eq!(s.grid_column_start, crate::property::GridLineValue::Line(2));
    assert_eq!(s.grid_column_end, crate::property::GridLineValue::Line(4));
}

#[test]
/// text-decoration 简写使用 underline blue
fn test_text_decoration_shorthand_with_underline() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("text-decoration", "underline blue")]);
    assert!(matches!(
        s.text_decoration_line,
        crate::property::types::TextDecorationLineValue {
            underline: true,
            overline: false,
            line_through: false
        }
    ));
}

#[test]
/// gap 简写测试 - 双值
fn test_gap_shorthand_double_value() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("gap", "10px 20px")]);
    assert_eq!(s.row_gap, LengthValue::Px(10.0));
    assert_eq!(s.column_gap, LengthValue::Px(20.0));
}

#[test]
/// R2354: `transition: NONE` 应与 `transition: none` 等价。
/// 关键字大小写不敏感（CSS Syntax §：所有关键字大小写不敏感）。
/// transition: none → transition-property 经 apply 层 filter "none" 后为空列表；
/// 关键回归点：大写 "NONE" 不应作为字面属性名残留（修复前 transition-property = ["NONE"]）。
fn test_shorthand_keywords_case_insensitive_transition_none() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s_lower = compute_style(&doc, div, &[("transition", "none")]);
    let s_upper = compute_style(&doc, div, &[("transition", "NONE")]);
    assert_eq!(s_upper.transition_property, s_lower.transition_property);
    // none → 空列表；不应残留大写 "NONE" 字面值
    assert!(s_upper.transition_property.iter().all(|p| p != "NONE"));
    assert_eq!(s_upper.transition_timing_function, s_lower.transition_timing_function);
}

#[test]
/// R2354: `flex: NONE` / `flex: AUTO` 应与全写关键字等价。
fn test_shorthand_keywords_case_insensitive_flex() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let s = compute_style(&doc, div, &[("flex", "NONE")]);
    assert_eq!(s.flex_grow, 0.0);
    assert_eq!(s.flex_shrink, 0.0);
    assert_eq!(s.flex_basis, crate::property::types::FlexBasisValue::Auto);

    let s = compute_style(&doc, div, &[("flex", "AUTO")]);
    assert_eq!(s.flex_grow, 1.0);
    assert_eq!(s.flex_shrink, 1.0);
    assert_eq!(s.flex_basis, crate::property::types::FlexBasisValue::Auto);
}

#[test]
/// R2354: CSS-wide 关键字 `INHERIT` 在简写层大小写不敏感（border: INHERIT 不应静默失效）。
/// 直接断言 expand_shorthands 对 `border: INHERIT` 产出 12 条声明（4 边 × width/style/color），
/// 而非走 parse_border_shorthand 失败后静默丢弃为空 vec。
fn test_shorthand_keywords_case_insensitive_css_wide() {
    let decls: Vec<(String, String, bool, (u32, u32, u32))> =
        vec![("border".to_string(), "INHERIT".to_string(), false, (0u32, 0u32, 0u32))];
    let result = crate::shorthand::expand_shorthands(&decls);
    assert_eq!(
        result.len(),
        12,
        "border: INHERIT 应识别为 CSS-wide 关键字并展开为 12 条长属性，而非静默丢弃"
    );
    // CSS-wide 路径下，每条声明的值应为原样透传的 "INHERIT"（修复前会落回 parse 默认值
    // "medium"/"none"/"currentcolor"，证明 css-wide 关键字未被识别）。
    assert!(
        result.iter().all(|(_, v, _, _)| v == "INHERIT"),
        "border: INHERIT 展开值应透传关键字原值，实际: {:?}",
        result
    );
}

#[test]
/// R2422: CSS-wide 关键字 `revert`/`revert-layer` 在简写层必须被识别（与 `cascade::is_css_wide_keyword`
/// 对齐，覆盖全 5 个关键字）。修复前 `matches_css_wide_keyword` 仅列 inherit/initial/unset，漏
/// revert/revert-layer——致 `border: revert-layer`/`background: revert` 等跳过 keyword 分支 → 值解析
/// 失败 → 整条声明静默丢弃（expand 返回空 vec）。driving: css-cascade inline-style-background.html
/// `background:revert`、revert-val-005.html。
fn test_shorthand_keywords_revert_and_revert_layer() {
    // border: revert-layer → 12 长属性（4 边 × width/style/color），值透传 "revert-layer"。
    let decls: Vec<(String, String, bool, (u32, u32, u32))> = vec![
        (
            "border".to_string(),
            "revert-layer".to_string(),
            false,
            (0u32, 0u32, 0u32),
        ),
        (
            "background".to_string(),
            "revert".to_string(),
            false,
            (0u32, 0u32, 0u32),
        ),
    ];
    let result = crate::shorthand::expand_shorthands(&decls);
    let border_decls: Vec<_> = result.iter().filter(|(p, _, _, _)| p.starts_with("border-")).collect();
    assert_eq!(
        border_decls.len(),
        12,
        "border: revert-layer 应展开为 12 条长属性，修复前静默丢弃为 0，实际: {border_decls:?}"
    );
    assert!(
        border_decls.iter().all(|(_, v, _, _)| v == "revert-layer"),
        "border: revert-layer 展开值应透传关键字原值，实际: {border_decls:?}"
    );
    // background: revert → 8 长属性（color/image/repeat/position/size/attachment/clip/origin）。
    let bg_decls: Vec<_> = result
        .iter()
        .filter(|(p, _, _, _)| p.starts_with("background-"))
        .collect();
    assert_eq!(
        bg_decls.len(),
        8,
        "background: revert 应展开为 8 条长属性，修复前静默丢弃为 0，实际: {bg_decls:?}"
    );
    assert!(
        bg_decls.iter().all(|(_, v, _, _)| v == "revert"),
        "background: revert 展开值应透传关键字原值，实际: {bg_decls:?}"
    );
}

#[test]
/// R2355: `looks_like_color` 颜色值消歧大小写不敏感（命名色 + rgb()/hsl() 函数名前缀）。
/// `border: 1px SOLID RED` 中 RED 应被识别为颜色（非静默丢色回退 currentcolor）。
fn test_shorthand_color_disambig_case_insensitive_named() {
    let decls: Vec<(String, String, bool, (u32, u32, u32))> = vec![(
        "border".to_string(),
        "1px SOLID RED".to_string(),
        false,
        (0u32, 0u32, 0u32),
    )];
    let result = crate::shorthand::expand_shorthands(&decls);
    // 应产生 border-top-color 声明，值为识别出的颜色 token "RED"
    // （修复前 RED 不被 looks_like_color 识别 → 丢色 → border-top-color = "currentcolor" 默认）。
    let top_color = result
        .iter()
        .find_map(|(p, v, _, _)| if p == "border-top-color" { Some(v.clone()) } else { None })
        .expect("应展开出 border-top-color");
    assert_eq!(
        top_color, "RED",
        "RED 应被识别为颜色 token，实际 border-top-color: {top_color}"
    );
}

#[test]
/// R2355: `RGB(...)` / `HSL(...)` 函数名前缀大小写不敏感。
fn test_shorthand_color_disambig_case_insensitive_function() {
    for upper in ["RGB(255, 0, 0)", "HSL(0, 100%, 50%)"] {
        let value = format!("1px SOLID {upper}");
        let decls: Vec<(String, String, bool, (u32, u32, u32))> =
            vec![("border".to_string(), value, false, (0u32, 0u32, 0u32))];
        let result = crate::shorthand::expand_shorthands(&decls);
        let top_color = result
            .iter()
            .find_map(|(p, v, _, _)| if p == "border-top-color" { Some(v.clone()) } else { None })
            .unwrap_or_else(|| "currentcolor".to_string());
        assert_eq!(
            top_color, upper,
            "{upper} 应被识别为颜色 token（非丢色回退 currentcolor）"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// 伪元素 ::before/::after 计算样式（R487：compute 阶段）
// ═══════════════════════════════════════════════════════════════════

/// 通过 CSS 文本构建样式表并计算，返回指定元素的计算样式。
fn compute_style_from_css(doc: &zero_dom::Document, css: &str, element: zero_dom::NodeId) -> ComputedStyle {
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let stylesheets = vec![stylesheet];
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(doc, &stylesheets);
    styles.get(&element).cloned().unwrap_or_default()
}

#[test]
fn test_before_pseudo_computed_from_content() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let css = r#"div:before { content: "X"; color: red; }"#;
    let s = compute_style_from_css(&doc, css, div);
    let before = s.before_pseudo.expect("div 应有 before 伪元素样式");
    match &before.content {
        crate::property::types::ContentComputedValue::String(t) => {
            assert!(t.contains('X'), "before content 文本: {t}");
        }
        other => panic!("before content 应为 String，实际 {other:?}"),
    }
    // 元素本体不应被伪元素规则污染（content 仍为 Normal）
    assert!(matches!(
        s.content,
        crate::property::types::ContentComputedValue::Normal
    ));
    // 无 ::after 规则 → after_pseudo 为 None
    assert!(s.after_pseudo.is_none(), "无 ::after 规则时 after_pseudo 应为 None");
}

#[test]
fn test_after_pseudo_and_content_none_no_box() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    // content: none 不应生成盒 → after_pseudo 不设置（content 非 String）
    let css = r#"div:after { content: none; }"#;
    let s = compute_style_from_css(&doc, css, div);
    assert!(s.after_pseudo.is_none(), "content:none 不应设置 after_pseudo");
    // before 无规则 → None
    assert!(s.before_pseudo.is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// R2481：background 简写 `<box>` + `/size` bundled 修复（CSS Backgrounds §3.4/§3.10/§3.11）
//
// ① `<box>`：单个 box 同时设 origin+clip；两个 box = origin/clip（改前静默丢）。
// ② `/size`：`/` 分隔 position 与 size（改前 `/size` 整体丢，contain/cover 落 bg_color）。
// ③ url() 内的 `/`（如 url(support/x.png)）不作为 separator（paren-aware depth-0）。
// driving：css-backgrounds background-334/335/336（`/size`）。
// ═══════════════════════════════════════════════════════════════════════

/// 辅助：直接对 `background: <v>` 调 expand_shorthands，取某 longhand 的展开值。
fn bg_longhand(value: &str, prop: &str) -> Option<String> {
    let decls: Vec<(String, String, bool, (u32, u32, u32))> =
        vec![("background".to_string(), value.to_string(), false, (0u32, 0u32, 0u32))];
    let result = crate::shorthand::expand_shorthands(&decls);
    result
        .iter()
        .find(|(p, _, _, _)| p == prop)
        .map(|(_, v, _, _)| v.clone())
}

#[test]
fn test_bg_shorthand_no_box_keeps_defaults() {
    assert_eq!(bg_longhand("red", "background-origin"), Some("padding-box".to_string()));
    assert_eq!(bg_longhand("red", "background-clip"), Some("border-box".to_string()));
}

#[test]
fn test_bg_shorthand_size_keyword_after_slash() {
    // `/ contain` → background-size = contain（改前 /size 整体丢 → size=auto）。
    assert_eq!(
        bg_longhand("url(x.png) / contain", "background-size"),
        Some("contain".to_string())
    );
}

#[test]
fn test_bg_shorthand_size_two_value_with_repeat_after() {
    // background-334 模式：`... / 100% auto no-repeat` → size=100% auto，no-repeat 仍归 repeat。
    assert_eq!(
        bg_longhand(
            "#CCC url(support/g.png) top left / 100% auto no-repeat",
            "background-size"
        ),
        Some("100% auto".to_string())
    );
    assert_eq!(
        bg_longhand(
            "#CCC url(support/g.png) top left / 100% auto no-repeat",
            "background-repeat"
        ),
        Some("no-repeat".to_string())
    );
    assert_eq!(
        bg_longhand(
            "#CCC url(support/g.png) top left / 100% auto no-repeat",
            "background-color"
        ),
        Some("#CCC".to_string())
    );
    assert_eq!(
        bg_longhand(
            "#CCC url(support/g.png) top left / 100% auto no-repeat",
            "background-position"
        ),
        Some("top left".to_string())
    );
}

#[test]
fn test_bg_shorthand_url_internal_slash_not_separator() {
    // url() 内的 `/`（路径分隔）不作为 position/size separator；image 保持完整。
    assert_eq!(
        bg_longhand("url(support/60x60-green.png)", "background-image"),
        Some("url(support/60x60-green.png)".to_string())
    );
    // 无 `/size` → size 默认 auto
    assert_eq!(
        bg_longhand("url(support/60x60-green.png)", "background-size"),
        Some("auto".to_string())
    );
}

#[test]
fn test_bg_shorthand_box_dropped_but_size_parsed() {
    // R2481：`<box>` 累积设 origin/clip 经 A/B 证 net −3（attachment-local false-pass unmasks，
    // host-layer JS-scroll deferred），故**保持 drop**（origin=padding-box、clip=border-box 默认）；
    // 但 `/size` 仍正确解析（box token 不干扰 size 分类）。
    assert_eq!(
        bg_longhand("url(x) content-box / contain", "background-size"),
        Some("contain".to_string())
    );
    // box 被 drop → origin/clip 保持默认（非 content-box）
    assert_eq!(
        bg_longhand("url(x) content-box / contain", "background-origin"),
        Some("padding-box".to_string())
    );
    assert_eq!(
        bg_longhand("url(x) content-box / contain", "background-clip"),
        Some("border-box".to_string())
    );
}

// ═══════════════════════════════════════════════════════════════════════
// R2486：font 简写 `font-size / line-height` 分隔符空格容忍（CSS Fonts §4 font shorthand）
//
// CSS 允许 `font: <size> / <line-height> <family>` 的 `/` 两侧有空白。改前 expand_font
// 仅处理 attached `16px/1.5`（part.contains('/')），spaced `16px / 1.5` 被 split_whitespace
// 拆成单独 `/` token → 落 family_parts 致 family="/ 1.5 serif" 垃圾、line_height 未设。
// ═══════════════════════════════════════════════════════════════════════

/// 辅助：直接对 `font: <v>` 调 expand_shorthands，取某 longhand 的展开值。
fn font_longhand(value: &str, prop: &str) -> Option<String> {
    let decls: Vec<(String, String, bool, (u32, u32, u32))> =
        vec![("font".to_string(), value.to_string(), false, (0u32, 0u32, 0u32))];
    let result = crate::shorthand::expand_shorthands(&decls);
    result
        .iter()
        .find(|(p, _, _, _)| p == prop)
        .map(|(_, v, _, _)| v.clone())
}

#[test]
fn test_font_shorthand_spaced_slash_line_height() {
    // R2486：`font: 16px / 1.5 serif`（spaced /）→ size=16px、line-height=1.5、family=serif。
    // 改前 family="/ 1.5 serif" 垃圾、line-height=normal。
    assert_eq!(font_longhand("16px / 1.5 serif", "font-size"), Some("16px".to_string()));
    assert_eq!(
        font_longhand("16px / 1.5 serif", "line-height"),
        Some("1.5".to_string())
    );
    assert_eq!(
        font_longhand("16px / 1.5 serif", "font-family"),
        Some("serif".to_string())
    );
}

#[test]
fn test_font_shorthand_attached_slash_line_height_regression() {
    // 回归：attached `16px/1.5 serif` 仍正确（spaced-修复不应破坏 attached 路径）。
    assert_eq!(font_longhand("16px/1.5 serif", "font-size"), Some("16px".to_string()));
    assert_eq!(font_longhand("16px/1.5 serif", "line-height"), Some("1.5".to_string()));
    assert_eq!(
        font_longhand("16px/1.5 serif", "font-family"),
        Some("serif".to_string())
    );
}

#[test]
fn test_font_shorthand_spaced_slash_with_style_weight() {
    // `font: italic bold 16px / 1.5 serif`（前置 style + weight + spaced /）。
    assert_eq!(
        font_longhand("italic bold 16px / 1.5 serif", "font-style"),
        Some("italic".to_string())
    );
    assert_eq!(
        font_longhand("italic bold 16px / 1.5 serif", "font-weight"),
        Some("bold".to_string())
    );
    assert_eq!(
        font_longhand("italic bold 16px / 1.5 serif", "font-size"),
        Some("16px".to_string())
    );
    assert_eq!(
        font_longhand("italic bold 16px / 1.5 serif", "line-height"),
        Some("1.5".to_string())
    );
    assert_eq!(
        font_longhand("italic bold 16px / 1.5 serif", "font-family"),
        Some("serif".to_string())
    );
}
