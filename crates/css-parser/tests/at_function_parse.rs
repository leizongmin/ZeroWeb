//! 刀 k109 @function 解析钉（css-mixins-1 §2.1；wptfyi-css 2026-11-25）。
//!
//! 根因修复的常驻锚（行为面＝`@function` 规则是否物化进 AST——语料
//! random-in-custom-function.tentative.html 15 定义全丢同形）：
//! ① 基本形物化（prelude 双杠 Function token ＋ result 描述符）
//! ② 参数列表（类型＋default；尾逗号丢弃）
//! ③ `returns <css-type>` 类型收集
//! ④ 无效 prelude（非双杠名）不吞后继合法规则（@property 臂同型）
//! ⑤ 参数重名 → 规则 invalid 整体丢弃＋后邻存活（§2.1.1）
//! ⑥ unknown descriptor invalid-ignored 不废规则（§2.1.2）
//! ⑦ result 重复声明后者胜序保留（§4 执行模型）
use zero_css_parser::FunctionRule;
use zero_css_parser::Parser;
use zero_css_parser::ast::Rule;

fn function_rules(css: &str) -> Vec<FunctionRule> {
    Parser::parse_stylesheet(css)
        .rules
        .into_iter()
        .filter_map(|r| match r {
            Rule::Function(f) => Some(f),
            _ => None,
        })
        .collect()
}

/// 钉①：基本形物化。
#[test]
fn basic_function_materializes() {
    let fs = function_rules("@function --f() { result: random(1, 1e6); }");
    assert_eq!(fs.len(), 1, "@function must materialize; got {:?}", fs.len());
    assert_eq!(fs[0].name, "--f");
    assert!(fs[0].params.is_empty());
    assert!(fs[0].returns.is_none());
    assert_eq!(fs[0].decls.len(), 1);
    assert_eq!(fs[0].decls[0].name, "result");
    assert_eq!(fs[0].decls[0].value, "random(1, 1000000.0)");
}

/// 钉②：参数列表（类型＋default；嵌套函数括号深度感知切分）。
#[test]
fn params_with_types_and_defaults() {
    let fs = function_rules(
        "@function --f-default-arguments(--x <number>: random(--foo property-index-scoped, 1, 1e6), --y) { result: var(--x); }",
    );
    assert_eq!(fs.len(), 1);
    assert_eq!(fs[0].params.len(), 2, "both params must parse");
    assert_eq!(fs[0].params[0].name, "--x");
    assert_eq!(fs[0].params[0].ty.as_deref(), Some("<number>"));
    assert_eq!(
        fs[0].params[0].default.as_deref(),
        Some("random(--foo property-index-scoped, 1, 1000000.0)"),
        "default value keeps nested-function commas intact"
    );
    assert_eq!(fs[0].params[1].name, "--y");
    assert!(fs[0].params[1].ty.is_none());
    assert!(fs[0].params[1].default.is_none());
}

/// 钉②b：尾逗号形（§3 Using final trailing comma 不传参）。
#[test]
fn trailing_comma_drops_empty_final_arg_param() {
    let fs = function_rules("@function --f(--x <number>,) { result: var(--x); }");
    assert_eq!(fs.len(), 1);
    assert_eq!(fs[0].params.len(), 1, "trailing comma must not add a param");
}

/// 钉③：returns 类型收集（含乘子复合形）。
#[test]
fn returns_type_collected() {
    let fs = function_rules("@function --g() returns <number> { result: 1; }");
    assert_eq!(fs.len(), 1);
    assert_eq!(fs[0].returns.as_deref(), Some("<number>"));

    let plus = function_rules("@function --g2() returns <number>+ { result: 1 2; }");
    assert_eq!(plus.len(), 1);
    assert_eq!(plus[0].returns.as_deref(), Some("<number>+"));
}

/// 钉④：无效 prelude（非双杠名）只丢自身，后继合法规则存活。
#[test]
fn invalid_prelude_does_not_swallow_followers() {
    let sheet = Parser::parse_stylesheet(
        "@function foo() { result: 1; }\n@function --ok() { result: 2; }\ndiv { width: 3px; }",
    );
    let fns: Vec<_> = sheet
        .rules
        .iter()
        .filter_map(|r| match r {
            Rule::Function(f) => Some(f.name.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(fns, vec!["--ok"], "non-dashed name must be dropped, follower kept");
    assert_eq!(sheet.rules.len(), 2, "style rule must survive");
}

/// 钉⑤：参数重名 → 规则 invalid（§2.1.1），后邻存活。
#[test]
fn duplicate_param_names_invalidate_rule() {
    let sheet = Parser::parse_stylesheet("@function --dup(--x, --x) { result: 1; }\n@function --ok() { result: 2; }");
    let fns: Vec<_> = sheet
        .rules
        .iter()
        .filter_map(|r| match r {
            Rule::Function(f) => Some(f.name.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(fns, vec!["--ok"], "duplicate param name must invalidate the rule");
}

/// 钉⑥：unknown descriptor invalid-ignored 不废规则（§2.1.2）。
#[test]
fn unknown_descriptor_ignored() {
    let fs = function_rules("@function --f() { unknown-desc: 1; result: 2; }");
    assert_eq!(fs.len(), 1, "unknown descriptor must not invalidate the rule");
    assert_eq!(fs[0].decls.len(), 1);
    assert_eq!(fs[0].decls[0].name, "result");
}

/// 钉⑦：result 重复声明后者胜序保留（§4 执行模型）。
#[test]
fn repeated_result_keeps_order() {
    let fs = function_rules("@function --mypi() { result: 3; result: 3.14; }");
    assert_eq!(fs.len(), 1);
    assert_eq!(fs[0].decls.len(), 2, "both result declarations kept in order");
    assert_eq!(fs[0].decls[0].value, "3");
    assert_eq!(fs[0].decls[1].value, "3.14");
}

/// 钉⑧：locals custom properties 声明物化（名大小写敏感保留）。
#[test]
fn local_declarations_materialize() {
    let fs = function_rules("@function --locals() { --x: random(1, 1e6); --y: 2; result: var(--x) var(--y); }");
    assert_eq!(fs.len(), 1);
    assert_eq!(fs[0].decls.len(), 3);
    assert_eq!(fs[0].decls[0].name, "--x");
    assert_eq!(fs[0].decls[0].value, "random(1, 1000000.0)");
    assert_eq!(fs[0].decls[2].name, "result");
    assert_eq!(fs[0].decls[2].value, "var(--x) var(--y)");
}
