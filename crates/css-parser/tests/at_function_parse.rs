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

/// k124：平面声明视图（body 条目树取 Decl 项——无条件组的函数与刀前
/// decls 平面恒等；条件组项是求值域语义，测试断言面只涉平面声明）。
fn decls_of(f: &zero_css_parser::ast::FunctionRule) -> Vec<&zero_css_parser::ast::FunctionDecl> {
    f.body
        .iter()
        .filter_map(|i| match i {
            zero_css_parser::ast::FunctionBodyItem::Decl(d) => Some(d),
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
    assert_eq!(decls_of(&fs[0]).len(), 1);
    assert_eq!(decls_of(&fs[0])[0].name, "result");
    assert_eq!(decls_of(&fs[0])[0].value, "random(1, 1000000.0)");
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
    assert_eq!(decls_of(&fs[0]).len(), 1);
    assert_eq!(decls_of(&fs[0])[0].name, "result");
}

/// 钉⑦：result 重复声明后者胜序保留（§4 执行模型）。
#[test]
fn repeated_result_keeps_order() {
    let fs = function_rules("@function --mypi() { result: 3; result: 3.14; }");
    assert_eq!(fs.len(), 1);
    assert_eq!(decls_of(&fs[0]).len(), 2, "both result declarations kept in order");
    assert_eq!(decls_of(&fs[0])[0].value, "3");
    assert_eq!(decls_of(&fs[0])[1].value, "3.14");
}

/// 钉⑧：locals custom properties 声明物化（名大小写敏感保留）。
#[test]
fn local_declarations_materialize() {
    let fs = function_rules("@function --locals() { --x: random(1, 1e6); --y: 2; result: var(--x) var(--y); }");
    assert_eq!(fs.len(), 1);
    assert_eq!(decls_of(&fs[0]).len(), 3);
    assert_eq!(decls_of(&fs[0])[0].name, "--x");
    assert_eq!(decls_of(&fs[0])[0].value, "random(1, 1000000.0)");
    assert_eq!(decls_of(&fs[0])[2].name, "result");
    assert_eq!(decls_of(&fs[0])[2].value, "var(--x) var(--y)");
}

/// 钉⑨/⑩：WPT at-function-parsing 全页镜像（k114 刀验收入锚；页形
/// `replaceSync(prelude + " {}")` → cssRules.length 1/0）。
const WPT_VALID_PRELUDES: &[&str] = &[
    "@function --foo()",
    "@function --foo( )",
    "@function --foo(--x)",
    "@function --foo( --x )",
    "@function --foo(--x auto)",
    "@function --foo(--x <angle>)",
    "@function --foo(--x <color>)",
    "@function --foo(--x <custom-ident>)",
    "@function --foo(--x <image>)",
    "@function --foo(--x <integer>)",
    "@function --foo(--x <length>)",
    "@function --foo(--x <length-percentage>)",
    "@function --foo(--x <number>)",
    "@function --foo(--x <percentage>)",
    "@function --foo(--x <resolution>)",
    "@function --foo(--x <string>)",
    "@function --foo(--x <time>)",
    "@function --foo(--x <url>)",
    "@function --foo(--x <transform-function>)",
    "@function --foo(--x <transform-list>)",
    "@function --foo(--x type(auto))",
    "@function --foo(--x type(<length>))",
    "@function --foo(--x type(<length> | auto))",
    "@function --foo(--x type(none | auto))",
    "@function --foo(--x type(*))",
    "@function --foo(--x, --y)",
    "@function --foo(--x, --y, --z)",
    "@function --foo(--x <length>, --y, --z)",
    "@function --foo(--x, --y <number>, --z <angle>)",
    "@function --foo(--x : 10px)",
    "@function --foo(--x type(*): 10px)",
    "@function --foo(--x <length>: 10px)",
    "@function --foo(--x <length>: 10px, --y)",
    "@function --foo(--x, --y <length>: 10px)",
    "@function --foo(--x type(<length> | auto): auto)",
    "@function --foo(--x type(<length> | auto) : auto)",
    "@function --foo(--x:1px, --y, --z:2px)",
    "@function --foo(--x <length>#)",
    "@function --foo(--x <length>+)",
    "@function --foo(--x type(<length>+))",
    "@function --foo(--x <transform-function>#)",
    "@function --foo(--x <transform-function>+)",
    "@function --foo(--x) returns type(*)",
    "@function --foo(--x) returns <length>",
    "@function --foo(--x) returns <length>+",
    "@function --foo(--x) returns type(<length>)",
    "@function --foo(--x) returns type(<length> | auto)",
    "@function --foo(--x) returns type(foo | bar)",
];

const WPT_INVALID_PRELUDES: &[&str] = &[
    "@function --foo ()",
    "@function --foo (--x)",
    "@function --foo(--x: 10px !important)",
    "@function --foo(--x <length>: 10deg)",
    "@function --foo(--x <angle>: 10px)",
    "@function --foo(--x <color>: 10px)",
    "@function --foo(--x type(<color>+): red 5)",
    "@function --foo(--x type(auto | none): thing)",
    "@function --foo(--x <transform-list>#)",
    "@function --foo(--x <transform-list>+)",
    "@function --foo(--x *)",
    "@function --foo(--x !)",
    "@function --foo(--x 50px)",
    "@function --foo(--x <length> | auto)",
    "@function --foo(--x none | auto)",
    "@function --foo(--x <dino>)",
    "@function --foo(!)",
    "@function --foo(--x: !)",
    "@function --foo(--x type(!))",
    "@function --foo(,)",
    "@function --foo(,,,)",
    "@function --foo(--x, ;)",
    "@function --foo(;)",
    "@function --foo(])",
    "@function --foo(, --x])",
    "@function --foo(--x) !",
    "@function --foo(--x) ! <length>",
    "@function --foo(--x) returns <length>!",
    "@function --foo(--x) length",
    "@function --foo(--x) returns",
    "@function --foo(--x) returns ",
    "@function --foo(--x) returns *",
    "@function --foo(--x) returns <transform-list>#",
    "@function --foo(--x) returns <transform-list>+",
    "@function --foo(--x) returns auto | none",
    "@function --foo(--x): <length>",
    "@function --foo(--x): length",
    "@function --foo(--x) returneth <length>",
];

/// 钉⑨：valid prelude 全表 → 恰 1 条 Function 规则物化。
#[test]
fn wpt_valid_prelude_table_materializes() {
    for prelude in WPT_VALID_PRELUDES {
        let fs = function_rules(&format!("{prelude} {{}}"));
        assert_eq!(fs.len(), 1, "must be valid: {prelude:?}");
    }
}

/// 钉⑩：invalid prelude 全表 → 0 条 Function 规则（整规则丢弃）。
#[test]
fn wpt_invalid_prelude_table_drops_rule() {
    for prelude in WPT_INVALID_PRELUDES {
        let fs = function_rules(&format!("{prelude} {{}}"));
        assert_eq!(fs.len(), 0, "must be invalid: {prelude:?}");
    }
}

/// 钉⑪：k115 canonical 序列化形（css-syntax-3 #serialize-an-identifier
/// escape 重编码——AST ty/returns/default 存 CSSOM 序列化形：常规形逐字节
/// 恒同（k114 账面零漂移），转义痕迹 `\9 ` TAB/`I\ dent` 空格 ident 形在
/// token 消费期保留；name/decl 名保持 decoded（接口面键），escape 形分离
/// 由 `escape_css_ident` 单源）。
#[test]
fn cssom_canonical_text_shapes() {
    // 常规形恒同。
    let fs = function_rules("@function --foo(--x <length>: 10px) returns <number>+ {}");
    assert_eq!(fs[0].params[0].ty.as_deref(), Some("<length>"));
    assert_eq!(fs[0].params[0].default.as_deref(), Some("10px"));
    assert_eq!(fs[0].returns.as_deref(), Some("<number>+"));
    // type() 复合形 token 重铸保形（剥壳在引擎回调域）。
    let fs = function_rules("@function --foo(--x type(<length> | auto)) {}");
    assert_eq!(fs[0].params[0].ty.as_deref(), Some("type(<length> | auto)"));
    // TAB ident：decoded 接口形与 escape 重编码分离。
    let fs = function_rules("@function --a-\\9 -b(--x) {}");
    assert_eq!(fs[0].name, "--a-\t-b");
    assert_eq!(zero_css_parser::escape_css_ident(&fs[0].name), "--a-\\9 -b");
    // 空格 ident type 分量：decoded `I dent` 不可逆——收集期重编码 `I\ dent`。
    let fs = function_rules("@function --foo(--x I\\ dent) {}");
    assert_eq!(fs[0].params[0].ty.as_deref(), Some("I\\ dent"));
    // decl 名 decoded（接口面 getPropertyValue 键）＋escape 形分离。
    let fs = function_rules("@function --foo() { --l-\\9 -x: 1px; }");
    assert_eq!(decls_of(&fs[0])[0].name, "--l-\t-x");
    assert_eq!(
        zero_css_parser::escape_css_ident(&decls_of(&fs[0])[0].name),
        "--l-\\9 -x"
    );
}
