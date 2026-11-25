//! k133：`::part(<ident-list>)` 解析钉测（CSS Shadow Parts §3.2.2，
//! https://drafts.csswg.org/css-shadow-parts/#part）。
//!
//! 钉面 = 解析期等价（Part 变体名单/大小写归一/缀后门）；匹配与级联语义归
//! style-system shadow_part_segment 钉 + 引擎电池判读。判据页：css/css-shadow
//! pseudo-elements-after-part ＋ grouping-with-checked/disabled。

use crate::ast::*;
use crate::parser::Parser;

/// sheet 内级联可达样式规则的最右 compound 子类向量（克隆——`::part` 面钉测
/// 只关心规则存活性与选择器子类形态）。
fn style_rule_selectors(css: &str) -> Vec<Vec<SubclassSelector>> {
    let ss = Parser::parse_stylesheet(css);
    let mut out = Vec::new();
    for r in &ss.rules {
        if let Rule::Style(sr) = r {
            for sel in &sr.selectors {
                if let Some(last) = sel.complex.parts.last() {
                    out.push(last.0.subclass_selectors.clone());
                }
            }
        }
    }
    out
}

#[test]
fn kc5r133_part_parse_basics() {
    // 单名解析：`::part(mypart)` Function token 主形态 → Part 变体名单 [mypart]。
    let subs = style_rule_selectors("::part(mypart) { color: red }");
    assert_eq!(subs.len(), 1, "::part(mypart) 规则应存活");
    assert!(
        matches!(&subs[0][0], SubclassSelector::PseudoElement(PseudoElementSelector::Part { names }) if names == &vec!["mypart".to_string()]),
        "::part(mypart) 应解析为 Part{{[\"mypart\"]}}，得 {:?}",
        subs[0]
    );

    // 多名空白分隔（css-shadow-parts：ident-list 空格分词，任一命中即匹配）＋
    // 名与括号间空白形态＋ASCII 大小写归一（匹配面 part 属性 token 小写比对）。
    let subs = style_rule_selectors("::part( a  b ) { color: red }");
    assert!(
        matches!(&subs[0][0], SubclassSelector::PseudoElement(PseudoElementSelector::Part { names }) if names == &vec!["a".to_string(), "b".to_string()]),
        "::part( a  b ) 应归一名单 [\"a\",\"b\"]，得 {:?}",
        subs[0]
    );
    let subs = style_rule_selectors("::part(MyPart) { color: red }");
    assert!(
        matches!(&subs[0][0], SubclassSelector::PseudoElement(PseudoElementSelector::Part { names }) if names == &vec!["mypart".to_string()]),
        "part 名 ASCII 大小写不敏感归一小写，得 {:?}",
        subs[0]
    );

    // 前置成分合法（type selector 限定）。
    let subs = style_rule_selectors("div::part(x) { color: red }");
    assert_eq!(subs.len(), 1, "div::part(x) 规则应存活");
    assert!(matches!(
        &subs[0][0],
        SubclassSelector::PseudoElement(PseudoElementSelector::Part { .. })
    ));
}

#[test]
fn kc5r133_part_parse_invalid_forms() {
    // 裸 `::part`（无参）非法（css-shadow-parts §3.2.2：参数须非空 ident list）。
    assert!(style_rule_selectors("::part { color: red }").is_empty());
    // 空名单非法。
    assert!(style_rule_selectors("::part() { color: red }").is_empty());
    assert!(style_rule_selectors("::part(  ) { color: red }").is_empty());
    // 非法实参 token（数字/字符串/不闭合）。
    assert!(style_rule_selectors("::part(3) { color: red }").is_empty());
    assert!(style_rule_selectors("::part(\"a\") { color: red }").is_empty());
    assert!(style_rule_selectors("::part(a { color: red }").is_empty());
    // Part 后缀 compound 成分非法（CSS Shadow Parts §3.2.2：::part(x).foo 非法）。
    assert!(style_rule_selectors("::part(x).foo { color: red }").is_empty());
    assert!(style_rule_selectors("::part(x)#i { color: red }").is_empty());
    assert!(style_rule_selectors("::part(x)[a] { color: red }").is_empty());
    // 二次 `::part` 缀后非法（双伪元素形态无 spec 语义）。
    assert!(style_rule_selectors("::part(x)::part(y) { color: red }").is_empty());
    // `::slotted(s)` 缀后 `::part` 非法（slotted 缀后仅 tree-abiding 白名单，part 不在表）。
    assert!(style_rule_selectors("::slotted(s)::part(y) { color: red }").is_empty());
    // k133 裁剪面：`::highlight(name)` Function 缀后暂拒（is_known_functional 白名单
    // 无 highlight——FIXME(k134+)：函数伪元素缀后扩面）。
    assert!(style_rule_selectors("::part(x)::highlight(n) { color: red }").is_empty());
}

#[test]
fn kc5r133_part_parse_suffix_forms() {
    // 缀后伪元素（白名单内）：Part + Standard 两子类（剥尾框架消费面——
    // selector_pseudo_element 取 last=Standard，剥尾主体含 Part）。
    let subs = style_rule_selectors("::part(x)::before { color: red }");
    assert_eq!(subs.len(), 1, "::part(x)::before 规则应存活");
    assert_eq!(subs[0].len(), 2, "应有两子类（Part + Standard(before)）");
    assert!(matches!(
        &subs[0][0],
        SubclassSelector::PseudoElement(PseudoElementSelector::Part { .. })
    ));
    assert!(
        matches!(&subs[0][1], SubclassSelector::PseudoElement(PseudoElementSelector::Standard(n)) if n == "before")
    );

    // 缀后伪类语法合法（行为面由匹配语义裁决——checked 常规匹配生效、:hover
    // 交互态无模拟不生效；css-shadow-parts §3.2.2 允许伪类缀后）。
    let subs = style_rule_selectors("::part(x):checked { color: red }");
    assert_eq!(subs.len(), 1, "::part(x):checked 规则应存活");
    assert!(matches!(&subs[0][1], SubclassSelector::PseudoClass(PseudoClassSelector::Simple(n)) if n == "checked"));
}
