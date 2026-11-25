//! k118：css-mixins-1 `<dashed-function>` 调用实参文法验证钉测。
//!
//! 钉面 = WPT css/css-mixins/functions/{dashed-function-parsing(49 形),
//! dashed-function-named-arg.tentative(15 形)} 全矩阵的验证器等价
//!（[`dashed_function_value_valid`] 单源）＋样式表声明门行为
//!（consume_declaration None→丢声明/恢复，css-syntax「bad declaration」同
//! 语义）。写面三通道（style-system validate_decl_value pre-arm / shim hub
//! 臂 / 本门）共享本单源，本文件钉 Rust 侧两通道的文法权威。
//!
//! https://drafts.csswg.org/css-mixins-1/#function-rule
//! https://github.com/w3c/csswg-drafts/issues/11749

use crate::parser::Parser;
use crate::values::dashed_function_value_valid;

/// WPT 矩阵逐形（dashed-function-parsing.html + named-arg.tentative.html
/// 64 test_* 形，原样转录）。
const MATRIX: &[(&str, bool)] = &[
    // parsing 合法形（12 基础 + 9 逗号块）。
    ("--func()", true),
    ("--func(auto)", true),
    ("--func(100px)", true),
    ("--func(#fff)", true),
    ("--func(auto, 100px, #fff)", true),
    ("--func(auto ,100px ,#fff)", true),
    ("--func(auto , 100px , #fff)", true),
    ("--func(auto,100px,#fff)", true),
    ("--func(\"thing\")", true),
    ("--func(var(--x))", true),
    ("--func(var(--x, auto))", true),
    ("--func(--bar(), --baz(--fez()))", true),
    ("--func({1}, 2)", true),
    ("--func({ 1}, 2)", true),
    ("--func({1 }, 2)", true),
    ("--func({ 1 }, 2)", true),
    ("--func({1},{2})", true),
    ("--func({1, 2, 3},{4})", true),
    ("--func({,},{4})", true),
    ("--func({{}},{4})", true),
    ("--func({[]},{4})", true),
    // parsing 非法形：declaration-value 保留词元。
    ("--func(!)", false),
    ("--func( !)", false),
    ("--func( ! )", false),
    ("--func(! )", false),
    ("--func(;)", false),
    ("--func(})", false),
    ("--func(])", false),
    // 词法期 bad-url（非法转义）。
    ("--func(url(\\\n))", false),
    // parsing 非法形：空实参。
    ("--func(asdf,)", false),
    ("--func(,)", false),
    ("--func(, )", false),
    ("--func( ,)", false),
    ("--func( , )", false),
    ("--func(a,,b)", false),
    ("--func(a, ,b)", false),
    // parsing 非法形：块非独占。
    ("--func(123 {})", false),
    ("--func(1{})", false),
    ("--func({}1)", false),
    ("--func( {}1)", false),
    ("--func({}1 )", false),
    ("--func({} 1)", false),
    ("--func(1, 2{})", false),
    ("--func(1,2{})", false),
    ("--func(1,2{} )", false),
    ("--func(1, {}2)", false),
    // parsing 非法形：空块体（{}-wrapper 非值）。
    ("--func({})", false),
    ("--func(1, {})", false),
    ("--func(1, { })", false),
    // named-arg 合法形（含 `--` bare dashed-ident 与块内保留形豁免）。
    ("--func(myident)", true),
    ("--func(--myident)", true),
    ("--func(--)", true),
    ("--func(50px --myident:)", true),
    ("--func({--myident:})", true),
    ("--func({ --myident:})", true),
    ("--func({--myident :})", true),
    ("--func({--myident: })", true),
    ("--func(10px, { --myident : })", true),
    // named-arg 非法形：保留形（首 dashed-ident 次 `:`，跨空白）。
    ("--func(--myident:)", false),
    ("--func( --myident:)", false),
    ("--func(--myident :)", false),
    ("--func(--myident: )", false),
    ("--func( --myident : )", false),
    ("--func(10px, --myident : )", false),
];

#[test]
fn test_k118_dashed_fn_grammar_matrix() {
    for (i, (value, expected)) in MATRIX.iter().enumerate() {
        assert_eq!(
            dashed_function_value_valid(value),
            *expected,
            "matrix[{}] {:?} 判定漂移（expected valid={})",
            i,
            value,
            expected
        );
    }
}

#[test]
fn test_k118_dashed_fn_no_call_always_valid() {
    // 无 dashed-function 调用词元恒真（零漂移口径——含 var() 引用虚警反例）。
    for value in [
        "auto",
        "var(--func)",
        "var(--func-auto, 1px)",
        "calc(1px + 2em)",
        "10px 20px",
        "",
        "   ",
        "--plain-ident",
        "url(https://example.com/a.png)",
    ] {
        assert!(dashed_function_value_valid(value), "无调用值 {:?} 必须恒真", value);
    }
}

#[test]
fn test_k118_dashed_fn_nested_call_recursion() {
    // 嵌套调用递归同文法：合法嵌套过、坏实参嵌套拒（任意深度）。
    assert!(dashed_function_value_valid("--func(--bar(auto), 1px)"));
    assert!(!dashed_function_value_valid("--func(--bar(!))"));
    assert!(!dashed_function_value_valid("--func(--bar(--baz(;)))"));
    // 非 dashed 函数内嵌 dashed 调用同验（透明走行语义）。
    assert!(dashed_function_value_valid("calc(1px + --fn(2px))"));
    assert!(!dashed_function_value_valid("calc(1px + --fn(;))"));
    // 未闭合调用拒。
    assert!(!dashed_function_value_valid("--func(auto"));
    assert!(!dashed_function_value_valid("--func(auto, --bar(1px)"));
}

#[test]
fn test_k118_dashed_fn_stylesheet_declaration_gate() {
    // 样式表门：坏实参调用声明整丢弃，同块后续声明恢复（skip_malformed_
    // declaration 语义——css-syntax「bad declaration」）。
    let ss = Parser::parse_stylesheet("div { top: --func(!); color: red; }");
    let decls = ss
        .rules
        .iter()
        .filter_map(|r| match r {
            crate::ast::Rule::Style(sr) if !sr.apply_expanded => Some(&sr.declarations),
            _ => None,
        })
        .next()
        .unwrap();
    assert!(
        !decls.iter().any(|d| d.property == "top"),
        "坏实参调用声明必须整丢弃，实际 {:?}",
        decls
    );
    assert!(
        decls.iter().any(|d| d.property == "color" && d.value == "red"),
        "同块后续合法声明必须恢复保留，实际 {:?}",
        decls
    );

    // 保留形（#11749）同门。
    let ss = Parser::parse_stylesheet("div { top: --func(--myident:); color: red; }");
    let decls = ss
        .rules
        .iter()
        .filter_map(|r| match r {
            crate::ast::Rule::Style(sr) if !sr.apply_expanded => Some(&sr.declarations),
            _ => None,
        })
        .next()
        .unwrap();
    assert!(!decls.iter().any(|d| d.property == "top"));
    assert!(decls.iter().any(|d| d.property == "color"));

    // 合法调用形保形存储（零重写）。
    let ss = Parser::parse_stylesheet("div { top: --func(auto, 100px); }");
    let decls = ss
        .rules
        .iter()
        .filter_map(|r| match r {
            crate::ast::Rule::Style(sr) if !sr.apply_expanded => Some(&sr.declarations),
            _ => None,
        })
        .next()
        .unwrap();
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].property, "top");
    assert_eq!(decls[0].value, "--func(auto, 100px)");
}

#[test]
fn test_k118_dashed_fn_custom_property_exempt() {
    // custom property 值是任意 token 流（guaranteed-invalid 只在 var() 替换
    // 期判定）——`--x: --func(!)` 不走门（门仅标准属性）。
    let ss = Parser::parse_stylesheet("div { --x: --func(!); color: red; }");
    let decls = ss
        .rules
        .iter()
        .filter_map(|r| match r {
            crate::ast::Rule::Style(sr) if !sr.apply_expanded => Some(&sr.declarations),
            _ => None,
        })
        .next()
        .unwrap();
    assert!(
        decls.iter().any(|d| d.property == "--x"),
        "custom property 声明必须保留（门豁免），实际 {:?}",
        decls
    );
    assert!(decls.iter().any(|d| d.property == "color"));
}
