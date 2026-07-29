//! `:nth-child` / `:nth-last-child` / `:nth-of-type` / `:nth-last-of-type` 的非法 An+B
//! 与非法 `of S` 参数应使整条规则被丢弃（CSS Selectors L4 §17 + CSS Values §7.1.1）。
//!
//! 背景：修复前 An+B 解析为**宽松文本收集**（`parse_nth_expression_str` 用 `unwrap_or(0)`），
//! 对 `1 n`（空格分隔）、`n-1of`（非法 n-ident）、`n + of`（B 部分非法）、`even of`（of 后空列表）、
//! `n of div`（of-type 不支持 of）等非法形式仍产出匹配的 NthPattern → 规则被保留并匹配元素
//! → WPT nth-of-invalid 失败（应全绿却出现 red）。本轮改用 token-based 严格 An+B 校验。

use super::*;

#[test]
fn test_nth_invalid_forms_all_dropped() {
    // 取自 WPT nth-of-invalid.html 的 24 条非法规则（全部应被丢弃 → 整样式表 0 规则）。
    // 注：`even of even` 与（of-type 的）`n of div` 在浏览器中语义各异——前者合法但不匹配
    //（of S=<even> 类型选择器，无 <even> 元素），后者非法（of-type 不支持 of）。
    // 故本断言按「期望丢弃」逐条分组，把唯一合法但空的 `even of even` 单独排除。
    let invalid_only = r#"
        div:nth-child(1 of) { background-color: red; }
        div:nth-last-child(n of) { background-color: red; }
        div:nth-child(even of) { background-color: red; }
        div:nth-child(even .test) { background-color: red; }
        div:nth-last-child(of) { background-color: red; }
        div:nth-child(of ) { background-color: red; }
        div:nth-last-child(of .) { background-color: red; }
        div:nth-child(of .test) { background-color: red; }
        div:nth-last-child(n + of ) { background-color: red; }
        div:nth-child(n - of ) { background-color: red; }
        div:nth-last-child(n + 1of) { background-color: red; }
        div:nth-child(+ of .test) { background-color: red; }
        div:nth-last-child(1 + of .test) { background-color: red; }
        div:nth-child(1 - of .test) { background-color: red; }
        div:nth-last-child(1 n) { background-color: red; }
        div:nth-child("1" of div) { background-color: red; }
        div:nth-last-child(1 "of" div) { background-color: red; }
        div:nth-child(1 of "" div) { background-color: red; }
        div:nth-last-child(n-1of div) { background-color: red; }
        div:nth-of-type(n of div) { background-color: red; }
        div:nth-last-of-type(n of div) { background-color: red; }
    "#;
    let sheet = Parser::parse_stylesheet(invalid_only);
    assert_eq!(
        sheet.rules.len(),
        0,
        "全部非法 nth 形式应使规则被丢弃，实际保留 {} 条",
        sheet.rules.len()
    );
}

#[test]
fn test_nth_even_of_even_valid_unmatched_kept() {
    // `even of even`：An+B=even，of S=<even>（类型选择器）。合法但无 <even> 元素 → 不匹配。
    // 应**保留**规则（合法），与浏览器一致。
    let css = "div:nth-last-child(even of even) { color: red; }";
    let sheet = Parser::parse_stylesheet(css);
    assert_eq!(sheet.rules.len(), 1, "`even of even` 合法，规则应保留");
}

fn first_pseudo(rule: &Rule) -> Option<&PseudoClassSelector> {
    let Rule::Style(style) = rule else {
        return None;
    };
    style.selectors.first().and_then(|s| {
        s.complex.parts.first().and_then(|(compound, _)| {
            compound.subclass_selectors.iter().find_map(|sub| {
                if let SubclassSelector::PseudoClass(pc) = sub {
                    Some(pc)
                } else {
                    None
                }
            })
        })
    })
}

/// 断言 `:nth-child(expr)` 解析为 NthChild{(a,b)}。
fn assert_nth_child(nth: &str, a: i32, b: i32) {
    let sheet = Parser::parse_stylesheet(&format!("{} {{ color: red; }}", nth));
    assert_eq!(sheet.rules.len(), 1, "{nth:?} 应保留 1 条规则");
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::NthChild(p)) => assert_eq!((p.a, p.b), (a, b), "{nth:?}"),
        other => panic!("{nth:?} 应为 NthChild，实际: {:?}", other),
    }
}

#[test]
fn test_nth_valid_forms_preserved() {
    // 回归：常见合法 An+B 形式全部正确解析（修复后不得误丢）。
    assert_nth_child(":nth-child(odd)", 2, 1);
    assert_nth_child(":nth-child(even)", 2, 0);
    assert_nth_child(":nth-child(3)", 0, 3);
    assert_nth_child(":nth-child(+3)", 0, 3);
    assert_nth_child(":nth-child(-3)", 0, -3);
    assert_nth_child(":nth-child(0)", 0, 0);
    assert_nth_child(":nth-child(n)", 1, 0);
    assert_nth_child(":nth-child(-n)", -1, 0);
    assert_nth_child(":nth-child(+n)", 1, 0);
    assert_nth_child(":nth-child(2n)", 2, 0);
    assert_nth_child(":nth-child(-2n)", -2, 0);
    assert_nth_child(":nth-child(+2n)", 2, 0);
    assert_nth_child(":nth-child(2n+1)", 2, 1);
    assert_nth_child(":nth-child(2n-1)", 2, -1);
    assert_nth_child(":nth-child(-n+3)", -1, 3);
    assert_nth_child(":nth-child(n+2)", 1, 2);
    assert_nth_child(":nth-child(2n + 1)", 2, 1);
    assert_nth_child(":nth-child(2n +1)", 2, 1);
    assert_nth_child(":nth-child(2n+ 1)", 2, 1);
    assert_nth_child(":nth-child( 2n+1 )", 2, 1);
    assert_nth_child(":nth-child(0n+5)", 0, 5);
    // 粘合形式（tokenizer 把 `n-<int>` 粘进 dimension 单位 / ident）：
    assert_nth_child(":nth-child(2n-1)", 2, -1);
    assert_nth_child(":nth-child(-2n-1)", -2, -1);
    assert_nth_child(":nth-child(-n-1)", -1, -1);
    assert_nth_child(":nth-child(-n+3)", -1, 3);
    assert_nth_child(":nth-child(n-0)", 1, 0);
}

#[test]
fn test_nth_of_type_rejects_of() {
    // `:nth-of-type(n of div)` / `:nth-last-of-type(n of div)`：of-type 系不支持 of S → 非法。
    for css in [
        "div:nth-of-type(n of div) { color: red; }",
        "div:nth-last-of-type(n of div) { color: red; }",
    ] {
        let sheet = Parser::parse_stylesheet(css);
        assert_eq!(sheet.rules.len(), 0, "{css:?} of-type 不支持 of，应丢弃");
    }
    // 回归：of-type 合法形式仍保留。
    let sheet = Parser::parse_stylesheet("div:nth-of-type(2n+1) { color: red; }");
    assert_eq!(sheet.rules.len(), 1);
    match first_pseudo(&sheet.rules[0]) {
        Some(PseudoClassSelector::NthOfType(p)) => assert_eq!((p.a, p.b), (2, 1)),
        other => panic!("应为 NthOfType，实际: {:?}", other),
    }
}

#[test]
fn test_nth_of_empty_selector_list_invalid() {
    // `of` 后紧跟 `)`（空选择器列表）→ 非法（of 必须带至少一个选择器）。
    for css in [
        "div:nth-child(2n of) { color: red; }",
        "div:nth-last-child(2n of ) { color: red; }",
        "div:nth-child(2n of, .x) { color: red; }",
    ] {
        let sheet = Parser::parse_stylesheet(css);
        assert_eq!(sheet.rules.len(), 0, "{css:?} of 后空选择器列表应丢弃");
    }
    // 回归：of 带有效选择器仍保留。
    let sheet = Parser::parse_stylesheet("div:nth-child(2n of .x) { color: red; }");
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nth_space_minus_form() {
    // `2n - 2`（空格 + 减号）：孤立 `-` 被 tokenizer 作 Ident("-")，须正确解析为 a=2,b=-2。
    // driving: WPT nth-last-child-of-nesting（嵌套 of S 内含 nth-last-child）。
    assert_nth_child(":nth-child(2n - 2)", 2, -2);
    assert_nth_child(":nth-child(2n - 0)", 2, 0);
    assert_nth_child(":nth-child(2n + 3)", 2, 3);
}
