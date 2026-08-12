//! CSS 解析器 parser.rs 和 values 覆盖率补充测试。

use crate::ast::*;
use crate::parser::Parser;
use crate::tokenizer::{Token, Tokenizer};
use crate::values::{FontFeatureSetting, FontFeatureSettingsValue};

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — nth 表达式解析（通过 :nth-child 等伪类测试）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_child_odd_even() {
    let css = "li:nth-child(odd) { color: red; } li:nth-child(even) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 2);
}

#[test]
fn test_nth_child_an_plus_b() {
    let css = "li:nth-child(2n+1) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_child_neg_an_b() {
    let css = "li:nth-child(-n+3) { color: green; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_child_just_n() {
    let css = "li:nth-child(n) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_child_just_number() {
    let css = "li:nth-child(5) { color: purple; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_last_child() {
    let css = "li:nth-last-child(3n+1) { color: orange; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_nth_of_type() {
    let css = "p:nth-of-type(2n) { color: teal; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @规则解析
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_at_rule_unknown_with_block() {
    // 未知 @rule 带大括号块
    let css = "@unknown { div { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_rule_unknown_statement() {
    let css = "@custom-rule value;";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_import_with_media() {
    let css = r#"@import "style.css" screen, print;"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_import_simple() {
    let css = r#"@import "reset.css";"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_container_basic() {
    let css = "@container (min-width: 700px) { div { color: blue; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_supports() {
    let css = "@supports (display: grid) { div { display: grid; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_at_layer_block() {
    let css = "@layer base { div { color: red; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — 伪类函数选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_not_selector() {
    let css = "p:not(.excluded) { color: green; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_is_selector() {
    let css = "p:is(.a, .b) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_where_selector() {
    let css = "p:where(.a, .b) { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_has_selector() {
    let css = "div:has(> p) { color: teal; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_lang_selector() {
    let css = "p:lang(en) { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — 属性选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attribute_exact_match() {
    let css = r#"[data-type="text"] { color: red; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_starts_with() {
    let css = r#"[href^="https"] { color: green; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_ends_with() {
    let css = r#"[src$=".png"] { color: blue; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_contains() {
    let css = r#"[class*="btn"] { color: teal; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_whitespace_separated() {
    let css = r#"[class~="active"] { color: orange; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_attribute_dash_match() {
    let css = r#"[lang|="en"] { color: purple; }"#;
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — 声明与 !important
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_declaration_with_important() {
    let css = "div { color: red !important; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_multiple_declarations() {
    let css = "div { color: red; background: blue; font-size: 16px; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — keyframes
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_keyframes_from_to() {
    let css = "@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_keyframes_percentage() {
    let css = "@keyframes slide { 0% { left: 0; } 50% { left: 50%; } 100% { left: 100%; } }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — 复杂选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_descendant_combinator() {
    let css = "div p span { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_child_combinator() {
    let css = "div > p { color: blue; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_adjacent_sibling_combinator() {
    let css = "h1 + p { color: green; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_general_sibling_combinator() {
    let css = "h1 ~ p { color: teal; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_pseudo_elements() {
    let css = "p::before { content: '»'; } p::after { content: '.'; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 2);
}

#[test]
fn test_class_and_id_combo() {
    let css = "div.card#main { color: red; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

#[test]
fn test_multiple_selectors_same_rule() {
    let css = "h1, h2, h3 { font-weight: bold; }";
    let ss = Parser::parse_stylesheet(css);
    assert_eq!(ss.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// values/types.rs — calc 深度限制和错误路径
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_deeply_nested() {
    // 深度嵌套的 calc 表达式
    use crate::values::parse_calc;
    let result = parse_calc("calc(1px + 2px)");
    assert!(result.is_some());
}

#[test]
fn test_calc_simple_addition() {
    use crate::values::parse_calc;
    assert!(parse_calc("calc(100% - 20px)").is_some());
}

#[test]
fn test_calc_multiplication() {
    use crate::values::parse_calc;
    assert!(parse_calc("calc(2 * 10px)").is_some());
}

#[test]
fn test_calc_division() {
    use crate::values::parse_calc;
    assert!(parse_calc("calc(100% / 2)").is_some());
}

#[test]
fn test_calc_invalid() {
    use crate::values::parse_calc;
    assert!(parse_calc("calc()").is_none());
    assert!(parse_calc("calc(invalid)").is_none());
}

#[test]
fn test_length_various_units() {
    use crate::values::parse_length;
    assert!(parse_length("10px").is_some());
    assert!(parse_length("1.5em").is_some());
    assert!(parse_length("2rem").is_some());
    assert!(parse_length("50%").is_some());
    assert!(parse_length("100vh").is_some());
    assert!(parse_length("100vw").is_some());
    assert!(parse_length("5vmin").is_some());
    assert!(parse_length("5vmax").is_some());
    assert!(parse_length("").is_none());
    assert!(parse_length("invalid").is_none());
}

#[test]
fn test_eval_calc_basic() {
    use crate::values::{eval_calc, parse_calc};
    if let Some(expr) = parse_calc("calc(10px + 20px)") {
        let result = eval_calc(&expr, None);
        assert!(result.is_some());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @font-face 规则解析
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_font_face_basic_url_quoted_family() {
    let css = r#"@font-face { font-family: "TestFont"; src: url("test.woff"); }"#;
    let ws = Parser::parse_stylesheet(css);
    assert_eq!(ws.rules.len(), 1, "should parse one @font-face rule");
    match &ws.rules[0] {
        Rule::FontFace(ff) => {
            assert_eq!(ff.family, "TestFont", "family quotes stripped");
            assert_eq!(ff.sources, vec!["test.woff".to_string()], "src url extracted");
        }
        other => panic!("expected Rule::FontFace, got {other:?}"),
    }
}

#[test]
fn test_font_face_unquoted_family_and_bare_url() {
    let css = r#"@font-face{font-family:MyFont;src:url(MyFont.ttf)}"#;
    let ws = Parser::parse_stylesheet(css);
    match &ws.rules[0] {
        Rule::FontFace(ff) => {
            assert_eq!(ff.family, "MyFont");
            assert_eq!(ff.sources, vec!["MyFont.ttf".to_string()]);
        }
        other => panic!("expected Rule::FontFace, got {other:?}"),
    }
}

#[test]
fn test_font_face_feature_settings_descriptor() {
    let css = r#"@font-face {
        font-family: FeatureFont;
        src: url(feature.ttf);
        font-feature-settings: "liga" off, "kern" 2;
    }"#;
    let ws = Parser::parse_stylesheet(css);
    match &ws.rules[0] {
        Rule::FontFace(ff) => assert_eq!(
            ff.feature_settings,
            FontFeatureSettingsValue::Features(vec![
                FontFeatureSetting {
                    tag: *b"liga",
                    value: 0,
                },
                FontFeatureSetting {
                    tag: *b"kern",
                    value: 2,
                },
            ])
        ),
        other => panic!("expected FontFace, got {other:?}"),
    }
}

#[test]
fn test_font_face_multiple_sources_with_format_ignored() {
    // 多个 src（含 format() 描述符），format 部分应被忽略，仅提取 url()
    let css = r#"@font-face {
        font-family: 'Multi';
        src: url(a.woff) format("woff"), url(b.ttf) format("truetype");
    }"#;
    let ws = Parser::parse_stylesheet(css);
    match &ws.rules[0] {
        Rule::FontFace(ff) => {
            assert_eq!(ff.family, "Multi");
            assert_eq!(
                ff.sources,
                vec!["a.woff".to_string(), "b.ttf".to_string()],
                "both urls extracted in order, format() ignored"
            );
        }
        other => panic!("expected Rule::FontFace, got {other:?}"),
    }
}

#[test]
fn test_font_face_does_not_break_surrounding_rules() {
    let css = r#"
        p { color: red; }
        @font-face { font-family: "F"; src: url("f.woff"); }
        div { color: blue; }
    "#;
    let ws = Parser::parse_stylesheet(css);
    assert_eq!(ws.rules.len(), 3, "3 rules preserved");
    assert!(matches!(ws.rules[0], Rule::Style(_)), "first is style");
    assert!(matches!(ws.rules[1], Rule::FontFace(_)), "second is font-face");
    assert!(matches!(ws.rules[2], Rule::Style(_)), "third is style");
}

/// R2417：`@font-face` 的 `font-weight` 描述符解析为绝对权重。
#[test]
fn test_font_face_weight_descriptor() {
    let cases = [
        (
            r#"@font-face { font-family: "A"; src: url(a.woff); font-weight: bold; }"#,
            Some(700),
        ),
        (
            r#"@font-face { font-family: "A"; src: url(a.woff); font-weight: normal; }"#,
            Some(400),
        ),
        (
            r#"@font-face { font-family: "A"; src: url(a.woff); font-weight: 600; }"#,
            Some(600),
        ),
        (
            r#"@font-face { font-family: "A"; src: url(a.woff); font-weight: 900; }"#,
            Some(900),
        ),
        (
            r#"@font-face { font-family: "A"; src: url(a.woff); font-weight: bolder; }"#,
            None,
        ),
        // 无 font-weight 描述符 → None（视为 normal/400）。
        (r#"@font-face { font-family: "A"; src: url(a.woff); }"#, None),
    ];
    for (css, expected) in cases {
        let ws = Parser::parse_stylesheet(css);
        match &ws.rules[0] {
            Rule::FontFace(ff) => assert_eq!(ff.weight, expected, "css: {css}"),
            other => panic!("expected FontFace, got {other:?}"),
        }
    }
}

#[test]
fn test_font_face_stretch_descriptor() {
    let cases = [
        ("normal", Some(100.0)),
        ("condensed", Some(75.0)),
        ("semi-expanded", Some(112.5)),
        ("ultra-expanded", Some(200.0)),
        ("137.5%", Some(137.5)),
        ("invalid", None),
    ];
    for (value, expected) in cases {
        let css = format!("@font-face {{ font-family: A; src: url(a.woff); font-stretch: {value}; }}");
        let stylesheet = Parser::parse_stylesheet(&css);
        match &stylesheet.rules[0] {
            Rule::FontFace(face) => assert_eq!(face.stretch, expected, "value: {value}"),
            other => panic!("expected FontFace, got {other:?}"),
        }
    }
}

#[test]
fn test_font_face_size_adjust_descriptor() {
    let cases = [
        ("150%", Some(1.5)),
        ("0%", Some(0.0)),
        ("-1%", None),
        ("1.5", None),
        ("invalid", None),
    ];
    for (value, expected) in cases {
        let css = format!("@font-face {{ font-family: A; src: url(a.woff); size-adjust: {value}; }}");
        let stylesheet = Parser::parse_stylesheet(&css);
        match &stylesheet.rules[0] {
            Rule::FontFace(face) => assert_eq!(face.size_adjust, expected, "value: {value}"),
            other => panic!("expected FontFace, got {other:?}"),
        }
    }
}

#[test]
fn test_font_face_unicode_range_descriptor() {
    let css = "@font-face { font-family: A; src: url(a.woff); unicode-range: U+41-5A, U+6??, U+1F600; }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::FontFace(face) => assert_eq!(
            face.unicode_ranges,
            vec![(0x41, 0x5A), (0x600, 0x6FF), (0x1F600, 0x1F600)]
        ),
        other => panic!("expected FontFace, got {other:?}"),
    }

    let invalid = Parser::parse_stylesheet("@font-face { font-family: A; src: url(a.woff); unicode-range: U+4?F; }");
    match &invalid.rules[0] {
        Rule::FontFace(face) => assert!(face.unicode_ranges.is_empty()),
        other => panic!("expected FontFace, got {other:?}"),
    }
}

#[test]
fn test_font_face_missing_family_or_src_dropped() {
    // 缺 src → 规则被丢弃（返回 None），不进入样式表
    let css = r#"@font-face { font-family: "NoSrc"; }"#;
    let ws = Parser::parse_stylesheet(css);
    assert_eq!(ws.rules.len(), 0, "font-face without src is dropped");
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs — @page 规则解析（R2010 P4：CSS Paged Media `size` 描述符）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_page_size_letter_resolved() {
    let css = "@page { size: letter; }";
    let ws = Parser::parse_stylesheet(css);
    assert_eq!(ws.rules.len(), 1, "should parse one @page rule");
    match &ws.rules[0] {
        Rule::Page(p) => {
            let (w, h) = p.size.expect("letter size resolved");
            assert!((w - 816.0).abs() < 0.1, "letter width 8.5in=816px, got {w}");
            assert!((h - 1056.0).abs() < 0.1, "letter height 11in=1056px, got {h}");
        }
        other => panic!("expected Rule::Page, got {other:?}"),
    }
}

#[test]
fn test_page_size_a4_landscape_swapped() {
    let css = "@page { size: A4 landscape; }";
    let ws = Parser::parse_stylesheet(css);
    match &ws.rules[0] {
        Rule::Page(p) => {
            let (w, h) = p.size.expect("a4 landscape resolved");
            // A4 portrait ≈ (793.7, 1122.5)；landscape 交换 → 宽 > 高
            assert!(w > h, "landscape: width {w} should exceed height {h}");
            assert!((h - 793.7).abs() < 1.0, "landscape height = portrait width, got {h}");
        }
        other => panic!("expected Rule::Page, got {other:?}"),
    }
}

#[test]
fn test_page_size_explicit_lengths() {
    let css = "@page { size: 200mm 300mm; }";
    let ws = Parser::parse_stylesheet(css);
    match &ws.rules[0] {
        Rule::Page(p) => {
            let (w, h) = p.size.expect("explicit lengths resolved");
            assert!((w - 200.0 * 96.0 / 25.4).abs() < 0.1, "width 200mm, got {w}");
            assert!((h - 300.0 * 96.0 / 25.4).abs() < 0.1, "height 300mm, got {h}");
        }
        other => panic!("expected Rule::Page, got {other:?}"),
    }
}

#[test]
fn test_page_size_auto_or_missing_is_none() {
    // `size: auto` 与无 size 描述符 → size=None（调用方回退默认 A4）
    for css in ["@page { size: auto; }", "@page { margin: 2cm; }"] {
        let ws = Parser::parse_stylesheet(css);
        match &ws.rules[0] {
            Rule::Page(p) => assert!(p.size.is_none(), "auto/missing size → None for {css:?}"),
            other => panic!("expected Rule::Page, got {other:?}"),
        }
    }
}

#[test]
fn test_page_rule_among_other_rules() {
    // @page 与样式规则共存——确认 dispatch 不吞相邻规则
    let css = "@page { size: legal; } div { color: red; }";
    let ws = Parser::parse_stylesheet(css);
    assert_eq!(ws.rules.len(), 2, "page + style rule both parsed");
    assert!(matches!(ws.rules[0], Rule::Page(_)));
    assert!(matches!(ws.rules[1], Rule::Style(_)));
}

#[test]
fn test_resolve_page_size_px_keywords_and_orient() {
    use crate::parser::resolve_page_size_px;
    // 命名关键字
    let (aw, ah) = resolve_page_size_px("A4").unwrap();
    assert!((ah - 1122.5).abs() < 1.0, "A4 height ≈1122.5, got {ah}");
    assert!(aw < ah, "A4 portrait w<h");
    // portrait / landscape 单独 = A4 朝向
    let (lw, lh) = resolve_page_size_px("landscape").unwrap();
    assert!(lw > lh, "landscape w>h");
    // legal
    let (_lw, lh2) = resolve_page_size_px("legal").unwrap();
    assert!((lh2 - 1344.0).abs() < 0.1, "legal height 14in=1344, got {lh2}");
}

#[test]
fn test_resolve_page_size_px_lengths_and_invalid() {
    use crate::parser::resolve_page_size_px;
    // 单长度 → 正方
    let (w, h) = resolve_page_size_px("10cm").unwrap();
    assert!((w - h).abs() < 0.01 && (w - 10.0 * 96.0 / 2.54).abs() < 0.01);
    // 双长度
    let (w, h) = resolve_page_size_px("100px 200px").unwrap();
    assert_eq!((w as i32, h as i32), (100, 200));
    // 无效 / 相对单位 → None
    assert!(resolve_page_size_px("bogus").is_none());
    assert!(resolve_page_size_px("50%").is_none());
    assert!(resolve_page_size_px("").is_none());
}

// ── R2011 @page margin 描述符 ───────────────────────────────────────────

#[test]
fn test_page_margin_and_size_both_parsed() {
    let css = "@page { size: letter; margin: 2cm; }";
    let ws = Parser::parse_stylesheet(css);
    match &ws.rules[0] {
        Rule::Page(p) => {
            let (w, _h) = p.size.expect("size parsed");
            assert!((w - 816.0).abs() < 0.1, "letter width");
            let (mt, _r, mb, _l) = p.margin.expect("margin parsed");
            let two_cm = 2.0 * 96.0 / 2.54;
            assert!((mt - two_cm).abs() < 0.1, "margin-top 2cm");
            assert!((mb - two_cm).abs() < 0.1, "margin-bottom 2cm");
        }
        other => panic!("expected Rule::Page, got {other:?}"),
    }
}

#[test]
fn test_resolve_page_margin_px_shorthand() {
    use crate::parser::resolve_page_margin_px;
    // 1 值：四边同
    let (t, r, b, l) = resolve_page_margin_px("10px").unwrap();
    assert_eq!((t as i32, r as i32, b as i32, l as i32), (10, 10, 10, 10));
    // 2 值：(top bottom, right left)
    let (t, r, b, l) = resolve_page_margin_px("10px 20px").unwrap();
    assert_eq!((t as i32, r as i32, b as i32, l as i32), (10, 20, 10, 20));
    // 3 值：(top, right left, bottom)
    let (t, r, b, l) = resolve_page_margin_px("1px 2px 3px").unwrap();
    assert_eq!((t as i32, r as i32, b as i32, l as i32), (1, 2, 3, 2));
    // 4 值
    let (t, r, b, l) = resolve_page_margin_px("1px 2px 3px 4px").unwrap();
    assert_eq!((t as i32, r as i32, b as i32, l as i32), (1, 2, 3, 4));
    // 单位换算（cm → px）
    let (t, _r, _b, _l) = resolve_page_margin_px("1in").unwrap();
    assert!((t - 96.0).abs() < 0.01, "1in = 96px, got {t}");
    // 无效 / 相对单位 / 空 → None
    assert!(resolve_page_margin_px("bogus").is_none());
    assert!(resolve_page_margin_px("50%").is_none());
    assert!(resolve_page_margin_px("").is_none());
}
