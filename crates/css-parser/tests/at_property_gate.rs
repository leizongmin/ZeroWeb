//! 刀 ATP 门判定钉（css-properties-values-api-1；wptfyi-css 2026-11-19）。
//!
//! 根因修复的常驻锚（行为面＝`@property` 规则是否物化进 AST——语料
//! 'Rule not applied' 断言同形）：
//! ① 乘子位于尖括号外（`<color>#`/`<length>+`/`foo+`；未知名 `<foo>#` 拒）
//! ② 无效 `@property` 不吞后继合法规则（malformed 恢复改 @font-face 同型，
//!    R3757 注记同族）
//! ③ ident 分量 initial 全等门（语料 `gandalf`/`grey` 拒钉）
//! ④ transform-function/transform-list font-relative 单位拒（§initial-value-
//!    descriptor computationally independent）
//! ⑤ universal 形 var() 前置拒（corpus `var(--x)` invalid 钉）
//! ⑥ serialize_css_ident C0/DEL hex 转义（corpus `--tab\9 tab` 钉）+
//!    `@property` canonical cssText
use zero_css_parser::Parser;
use zero_css_parser::PropertyRule;
use zero_css_parser::ast::Rule;
use zero_css_parser::parser::syntax_string_content;
use zero_css_parser::serialize::{serialize_css_ident, serialize_property_rule_css_text};

fn property_rules(css: &str) -> Vec<PropertyRule> {
    Parser::parse_stylesheet(css)
        .rules
        .into_iter()
        .filter_map(|r| match r {
            Rule::Property(p) => Some(p),
            _ => None,
        })
        .collect()
}

/// 钉①：乘子在尖括号外的 typed 语法合法（`<color>#` 逗号列表 / `<length>+`
/// 空白列表）；未知名带乘子仍拒（严格文法）。
#[test]
fn multiplier_outside_angle_brackets() {
    let ok = property_rules("@property --ok { syntax: \"<color>#\"; inherits: false; initial-value: red, blue; }");
    assert_eq!(ok.len(), 1, "`<color>#` must materialize; got {ok:?}");
    assert_eq!(ok[0].name, "--ok");

    let plus = property_rules("@property --p { syntax: \"<length>+\"; inherits: false; initial-value: 0px 10px; }");
    assert_eq!(plus.len(), 1, "`<length>+` must materialize; got {plus:?}");

    let unknown = property_rules("@property --u { syntax: \"<foo>#\"; inherits: false; initial-value: red; }");
    assert!(unknown.is_empty(), "unknown type with multiplier must be dropped");
}

/// 钉①b：乘子可附 ident 字面量分量（`foo+`）；裸 CSS 全局关键字仍拒。
#[test]
fn ident_component_takes_multiplier() {
    let ok = property_rules("@property --i { syntax: \"foo+ | <color>#\"; inherits: false; initial-value: red; }");
    assert_eq!(ok.len(), 1, "`foo+` ident component must materialize; got {ok:?}");

    let kw = property_rules("@property --k { syntax: \"initial\"; inherits: false; initial-value: initial; }");
    assert!(kw.is_empty(), "css-wide keyword component must be dropped");
}

/// 钉②：无效 `@property`（syntax 缺引号）只丢自身，后继合法规则存活。
#[test]
fn invalid_property_rule_does_not_swallow_followers() {
    let sheet = Parser::parse_stylesheet(
        "@property --bad { syntax: <color>; inherits: false; initial-value: red; }\n\
         @property --good { syntax: \"<color>\"; inherits: false; initial-value: red; }\n\
         div { color: red; }",
    );
    let props: Vec<_> = sheet
        .rules
        .iter()
        .filter_map(|r| match r {
            Rule::Property(p) => Some(p.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(props, vec!["--good"], "only the valid @property survives");
    assert!(
        sheet.rules.iter().any(|r| matches!(r, Rule::Style(_))),
        "style rule after invalid @property must survive"
    );
}

/// 钉③：ident 分量 initial 须为全等单 ident token（`gandalf` 过、`wizard` 拒、
/// 多 token 拒——语料 gandalf/grey 面）。
#[test]
fn ident_component_initial_exact_match() {
    let ok = property_rules("@property --g { syntax: \"gandalf\"; inherits: false; initial-value: gandalf; }");
    assert_eq!(ok.len(), 1, "equal ident initial must materialize; got {ok:?}");

    let mismatch = property_rules("@property --m { syntax: \"gandalf\"; inherits: false; initial-value: wizard; }");
    assert!(mismatch.is_empty(), "differing ident initial must be dropped");
}

/// 钉④：transform 族 initial 禁 font-relative 单位（内层函数参数同拒）。
#[test]
fn transform_initial_rejects_font_relative() {
    for (syntax, iv) in [
        ("<transform-function>", "translateX(1em)"),
        ("<transform-list>", "translateY(1lh) scale(2)"),
    ] {
        let css = format!("@property --t {{ syntax: \"{syntax}\"; inherits: false; initial-value: {iv}; }}");
        assert!(
            property_rules(&css).is_empty(),
            "{syntax} must reject font-relative `{iv}`"
        );
    }
    let ok = property_rules(
        "@property --t { syntax: \"<transform-function>\"; inherits: false; initial-value: translateX(10px); }",
    );
    assert_eq!(ok.len(), 1, "px transform initial must materialize");
}

/// 钉⑤：universal 形 var() initial 拒、普通值过、缺省合法。
#[test]
fn universal_initial_var_rejected() {
    let var_rule = property_rules("@property --v { syntax: \"*\"; inherits: false; initial-value: var(--x); }");
    assert!(var_rule.is_empty(), "var() initial must invalidate the rule");

    let ok = property_rules("@property --v { syntax: \"*\"; inherits: false; initial-value: 10px; }");
    assert_eq!(ok.len(), 1);

    let bare = property_rules("@property --b { syntax: \"*\"; inherits: true; }");
    assert_eq!(bare.len(), 1, "universal without initial must materialize");
}

/// 钉⑥：C0/DEL hex 转义（corpus cssText 钉 `--tab\9 tab`）与普通字符逐字转义。
#[test]
fn serialize_css_ident_control_escapes() {
    assert_eq!(serialize_css_ident("--tab\ttab"), "--tab\\9 tab");
    assert_eq!(serialize_css_ident("\u{7f}x"), "\\7f x");
    assert_eq!(serialize_css_ident("a:b"), "a\\:b");
    assert_eq!(serialize_css_ident("--plain_1"), "--plain_1");
}

/// 钉⑥b：`@property` canonical cssText——描述符固定序、syntax 内容剥引号保形、
/// 缺 initial-value 段省略。
#[test]
fn property_rule_css_text_shape() {
    let rule = PropertyRule {
        name: "--x".to_string(),
        syntax: "\" <color># \"".to_string(),
        inherits: false,
        initial_value: Some("red".to_string()),
    };
    assert_eq!(
        serialize_property_rule_css_text(&rule),
        "@property --x { syntax: \" <color># \"; inherits: false; initial-value: red; }"
    );
    let bare = PropertyRule {
        name: "--b".to_string(),
        syntax: "\"*\"".to_string(),
        inherits: true,
        initial_value: None,
    };
    assert_eq!(
        serialize_property_rule_css_text(&bare),
        "@property --b { syntax: \"*\"; inherits: true; }"
    );
}

/// 钉⑦：函数形 initial（calc/rgb）整值保形——逗号在函数括号内不构成
/// 顶层逗号（<length-percentage> calc 混合形、<color> rgb() 形语料锚定）。
#[test]
fn function_form_initial_materializes_whole() {
    let calc = property_rules(
        "@property --c { syntax: \"<length-percentage>\"; inherits: false; initial-value: calc(10% + 10px); }",
    );
    assert_eq!(calc.len(), 1, "calc mixed LP initial must materialize; got {calc:?}");
    assert_eq!(
        calc[0].initial_value.as_deref(),
        Some("calc(10% + 10px)"),
        "initial-value must keep whole calc text"
    );

    let rgb = property_rules("@property --r { syntax: \"<color>\"; inherits: false; initial-value: rgb(1, 2, 3); }");
    assert_eq!(rgb.len(), 1, "rgb() color initial must materialize; got {rgb:?}");
    assert_eq!(rgb[0].initial_value.as_deref(), Some("rgb(1, 2, 3)"));

    let rgb_nospace =
        property_rules("@property --r2 { syntax: \"<color>\"; inherits: false; initial-value: rgb(1,2,3); }");
    assert_eq!(rgb_nospace.len(), 1, "rgb(1,2,3) must materialize; got {rgb_nospace:?}");

    // 函数形按括号平衡许可（修复前 RParen 落单拒整值）：`<length>` calc(10px)
    //（probe6 zzb2 锚定）与 typed 臂逐型函数面。
    let len_calc =
        property_rules("@property --lc { syntax: \"<length>\"; inherits: false; initial-value: calc(10px); }");
    assert_eq!(
        len_calc.len(),
        1,
        "<length> calc(10px) must materialize; got {len_calc:?}"
    );

    // 顶层逗号仍拒（非 `#` 语法）：`<length>` initial `1px, 2px` 非法。
    let top_comma = property_rules("@property --t { syntax: \"<length>\"; inherits: false; initial-value: 1px, 2px; }");
    assert!(
        top_comma.is_empty(),
        "top-level comma with non-list syntax must be dropped"
    );
}

/// 钉⑧：prelude 期失败（名非 `--` 前缀）＝extent 整体消耗——body 不被按
/// qualified rule 二次解析泄漏成 style 规则（语料 'Invalid property name does
/// not parse' 断言 cssRules 无幽灵条目同形；修复前 rules.length==1）。
#[test]
fn invalid_name_prelude_consumes_extent() {
    let sheet = Parser::parse_stylesheet("@property foo { syntax: \"<color>\"; inherits: false; initial-value: red; }");
    assert!(
        sheet.rules.is_empty(),
        "invalid-name rule must vanish whole; got {:?}",
        sheet.rules
    );

    // 语句形式残余同样不泄漏。
    let stmt = Parser::parse_stylesheet("@property foo; div { color: red; }");
    assert_eq!(
        stmt.rules.len(),
        1,
        "only the following style rule survives; got {:?}",
        stmt.rules
    );

    // 缺名整体块形（`@property { … }`）也不泄漏。
    let noname = Parser::parse_stylesheet("@property { syntax: \"<color>\"; }\ndiv { color: red; }");
    assert_eq!(noname.rules.len(), 1, "block extent consumed; got {:?}", noname.rules);
}

/// syntax 声明值 string 抽取钉（引号原文 whitespace 保形；缺引号 None）。
#[test]
fn syntax_string_content_extraction() {
    assert_eq!(syntax_string_content("\" <color># \"").as_deref(), Some(" <color># "));
    assert_eq!(syntax_string_content("<color>"), None, "unquoted is not a string");
}
