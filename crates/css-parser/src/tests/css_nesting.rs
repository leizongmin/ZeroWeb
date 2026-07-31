//! CSS 嵌套（CSS Nesting Module Level 1）compile 算法驱动测试。
//!
//! driving：wpt css-nesting 的 nesting-basic / nesting-type-selector / implicit-nesting
//! 三大静态 reftest 用例覆盖的语义（显式 `&`、隐式嵌套、前导组合器、`:is(&)`、父列表
//! 交叉积、顶层 `&`→:scope）。本文件验证解析后规则被正确**编译展平**为顶层等价规则。

use crate::Parser;
use crate::ast::*;

/// 把复合选择器序列化为字符串（测试辅助；覆盖嵌套用例所需子集）。
fn compound_to_string(c: &CompoundSelector) -> String {
    let mut s = String::new();
    match &c.type_selector {
        Some(TypeSelector::Tag(t)) => s.push_str(t),
        Some(TypeSelector::Universal) => s.push('*'),
        None => {}
    }
    for sub in &c.subclass_selectors {
        match sub {
            SubclassSelector::Id(id) => {
                s.push('#');
                s.push_str(id);
            }
            SubclassSelector::Class(cls) => {
                s.push('.');
                s.push_str(cls);
            }
            SubclassSelector::Attribute(a) => {
                s.push('[');
                s.push_str(&a.name);
                s.push(']');
            }
            SubclassSelector::PseudoElement(pe) => {
                s.push_str("::");
                let PseudoElementSelector::Standard(n) = pe;
                s.push_str(n);
            }
            SubclassSelector::PseudoClass(pc) => {
                s.push(':');
                s.push_str(&pseudo_class_to_string(pc));
            }
            SubclassSelector::Nesting => s.push('&'),
        }
    }
    s
}

fn pseudo_class_to_string(pc: &PseudoClassSelector) -> String {
    match pc {
        PseudoClassSelector::Simple(n) => n.clone(),
        PseudoClassSelector::Not(l)
        | PseudoClassSelector::Is(l)
        | PseudoClassSelector::Where(l)
        | PseudoClassSelector::Has(l) => {
            let head = match pc {
                PseudoClassSelector::Not(_) => "not",
                PseudoClassSelector::Is(_) => "is",
                PseudoClassSelector::Where(_) => "where",
                _ => "has",
            };
            format!("{}({})", head, selectors_to_string(l))
        }
        PseudoClassSelector::NthChild(_)
        | PseudoClassSelector::NthLastChild(_)
        | PseudoClassSelector::NthChildOf(_, _)
        | PseudoClassSelector::NthLastChildOf(_, _)
        | PseudoClassSelector::NthOfType(_)
        | PseudoClassSelector::NthLastOfType(_) => "nth".to_string(),
        PseudoClassSelector::Lang(l) => format!("lang({})", l.join(",")),
        PseudoClassSelector::Dir(d) => format!("dir({d})"),
    }
}

fn comb_to_string(comb: Option<Combinator>) -> &'static str {
    match comb {
        None => "",
        Some(Combinator::Descendant) => " ",
        Some(Combinator::Child) => " > ",
        Some(Combinator::NextSibling) => " + ",
        Some(Combinator::SubsequentSibling) => " ~ ",
    }
}

/// 把单个复杂选择器序列化为字符串。
fn selector_to_string(sel: &Selector) -> String {
    let mut s = String::new();
    for (compound, comb) in &sel.complex.parts {
        s.push_str(&compound_to_string(compound));
        s.push_str(comb_to_string(*comb));
    }
    s.trim().to_string()
}

/// 把选择器列表序列化为 `, ` 分隔字符串。
fn selectors_to_string(sels: &[Selector]) -> String {
    sels.iter().map(selector_to_string).collect::<Vec<_>>().join(", ")
}

/// 收集样式表中所有样式规则的 `(选择器文本, 属性集合)`，按出现顺序。
fn style_rules(css: &str) -> Vec<(String, Vec<String>)> {
    let ss = Parser::parse_stylesheet(css);
    ss.rules
        .iter()
        .filter_map(|r| {
            if let Rule::Style(sr) = r {
                Some((
                    selectors_to_string(&sr.selectors),
                    sr.declarations.iter().map(|d| d.property.clone()).collect(),
                ))
            } else {
                None
            }
        })
        .collect()
}

/// 提取首个匹配 `prop` 的规则的选择器文本（断言唯一/存在由调用方负责）。
fn first_rule_with_prop<'a>(rules: &'a [(String, Vec<String>)], prop: &str) -> Option<&'a str> {
    rules
        .iter()
        .find(|(_, props)| props.iter().any(|p| p == prop))
        .map(|(sel, _)| sel.as_str())
}

#[test]
/// `& > div`、`& .child`：显式 `&` + 组合器，替换为父级化合物。
fn test_nesting_explicit_amp_combinator() {
    let rules = style_rules(".test-1 { & > div { background: green; } }");
    assert_eq!(first_rule_with_prop(&rules, "background"), Some(".test-1 > div"));

    let rules = style_rules(".test-3 { & .child { background: green; } }");
    assert_eq!(first_rule_with_prop(&rules, "background"), Some(".test-3 .child"));
}

#[test]
/// `.parent &`：`&` 在末位（amid），父级化合物拼入 `&` 位置，保留前导部分与组合器。
fn test_nesting_amp_at_end() {
    // 父级 span > b（多化合物），嵌套 `.test-4 section &`
    let rules = style_rules("span > b { .test-4 section & { background: green; } }");
    assert_eq!(
        first_rule_with_prop(&rules, "background"),
        Some(".test-4 section span > b")
    );
}

#[test]
/// `&.cls`：`&` 与子类共存，合并到父级末化合物。
fn test_nesting_amp_with_subclass() {
    let rules = style_rules(".test-6 { &.test { background: green; } }");
    assert_eq!(first_rule_with_prop(&rules, "background"), Some(".test-6.test"));
}

#[test]
/// `div&`：类型选择器与 `&` 共存，合并（类型取嵌套，子类取父级）。
fn test_nesting_type_with_amp() {
    let rules = style_rules("div.test-14 { div& { background: green; } }");
    assert_eq!(first_rule_with_prop(&rules, "background"), Some("div.test-14"));
}

#[test]
/// 裸 `&`：替换为整个父级（含多化合物父级 span > b）。
fn test_nesting_bare_amp() {
    let rules = style_rules(".test-8 { & { background: green; } }");
    assert_eq!(first_rule_with_prop(&rules, "background"), Some(".test-8"));
}

#[test]
/// 隐式嵌套（无 `&`）：`.child` → `父级 后代 .child`。
fn test_nesting_implicit_descendant() {
    let rules = style_rules(".test-2 { .test-2-child { background: green; } }");
    assert_eq!(
        first_rule_with_prop(&rules, "background"),
        Some(".test-2 .test-2-child")
    );

    // :root { div {...} } —— 裸类型选择器隐式后代（nesting-type-selector 用例）
    let rules = style_rules(":root { div { background: green; } }");
    assert_eq!(first_rule_with_prop(&rules, "background"), Some(":root div"));
}

#[test]
/// 前导组合器隐式嵌套：`> div` → 注入 `&` → `父级 > div`；`+ .bar` → `父级 + .bar`。
fn test_nesting_leading_combinator_implicit() {
    let rules = style_rules(".test-1 { > div { background: green; } }");
    assert_eq!(first_rule_with_prop(&rules, "background"), Some(".test-1 > div"));

    let rules = style_rules(".test-7 { + .sibling { background: green; } }");
    assert_eq!(first_rule_with_prop(&rules, "background"), Some(".test-7 + .sibling"));
}

#[test]
/// `:is(&)`：`&` 在函数参数内，替换为父级整个复杂选择器。
fn test_nesting_amp_inside_is() {
    let rules = style_rules(".test-4 { :is(&) { background: green; } }");
    assert_eq!(first_rule_with_prop(&rules, "background"), Some(":is(.test-4)"));

    // :is(.test-5, &.does-not-exist) —— 列表中 `&` 仅替换含 & 的项
    let rules = style_rules(".test-5 { :is(.test-5, &.x) { background: green; } }");
    assert_eq!(
        first_rule_with_prop(&rules, "background"),
        Some(":is(.test-5, .test-5.x)")
    );
}

#[test]
/// 父级是选择器列表：交叉积（per spec 应 :is()，交叉积匹配等价）。
/// 编译为单条规则的选择器列表 `.a + .c, .b + .c`（CSS 等价表示）。
fn test_nesting_parent_list_cross_product() {
    let rules = style_rules(".a, .b { & + .c { background: green; } }");
    let combined = rules
        .iter()
        .find(|(_, p)| p.iter().any(|x| x == "background"))
        .map(|(s, _)| s.as_str())
        .expect("应有 background 规则");
    assert!(combined.contains(".a + .c"));
    assert!(combined.contains(".b + .c"));
}

#[test]
/// 顶层 `&`：等价 `:scope`（文档样式表中 ≡ :root）。
fn test_nesting_top_level_amp_is_scope() {
    let rules = style_rules("& .test-12 { background: green; }");
    assert_eq!(first_rule_with_prop(&rules, "background"), Some(":scope .test-12"));
}

#[test]
/// 自身声明与嵌套规则共存：自身声明保留，嵌套规则独立展开。
fn test_nesting_own_decls_plus_nested() {
    let rules = style_rules(".a { color: red; .b { color: blue; } }");
    // 两条样式规则：.a { color:red } 与 .a .b { color:blue }
    assert!(rules.iter().any(|(s, p)| s == ".a" && p.contains(&"color".to_string())));
    assert!(
        rules
            .iter()
            .any(|(s, p)| s == ".a .b" && p.contains(&"color".to_string()))
    );
}

#[test]
/// 多层嵌套：编译递归线程父级（祖父 → 父 → 子）。
fn test_nesting_deep() {
    let rules = style_rules(".a { .b { .c { color: red; } } }");
    assert_eq!(first_rule_with_prop(&rules, "color"), Some(".a .b .c"));
}

// kill-switch（ZW_CSS_NESTING=0）的零回归回退由 reftest A/B（默认 vs =0）验证，
// 不在此加单测——`cargo test` 多线程并行下设全局 env 会与其他嵌套用例竞态致 flaky。

// ── 嵌套 @规则重父化（R2262）──────────────────────────────────────────────
// `.a { @media/q { <body> } }` → body 相对 .a 重父化后包裹回 @规则。

/// 从样式表提取所有 @media（Rule::At name=media）块体内的样式规则 `(选择器, 属性集)`。
fn media_body_style_rules(css: &str) -> Vec<(String, Vec<String>)> {
    let ss = Parser::parse_stylesheet(css);
    let mut out = Vec::new();
    for r in &ss.rules {
        if let Rule::At(at) = r
            && at.name.eq_ignore_ascii_case("media")
            && let crate::ast::AtRuleBody::Block(inner) = &at.body
        {
            for ir in inner {
                if let Rule::Style(sr) = ir {
                    out.push((
                        selectors_to_string(&sr.selectors),
                        sr.declarations.iter().map(|d| d.property.clone()).collect(),
                    ));
                }
            }
        }
    }
    out
}

#[test]
/// 嵌套 @media 含显式 `& {...}` 规则：body 相对父级重父化。
fn test_nesting_media_explicit_amp() {
    let rules = media_body_style_rules(".a { @media (min-width: 10px) { & { color: green; } } }");
    assert_eq!(first_rule_with_prop(&rules, "color"), Some(".a"));
}

#[test]
/// 嵌套 @media 含裸声明（隐式 `& { ... }`）：声明归属父级。
fn test_nesting_media_bare_declaration() {
    let rules = media_body_style_rules(".a { @media (min-width: 10px) { color: green; } }");
    assert_eq!(first_rule_with_prop(&rules, "color"), Some(".a"));
}

#[test]
/// 嵌套 @media body 内含嵌套规则：相对父级去糖（`.a .b`）。
fn test_nesting_media_nested_rule() {
    let rules = media_body_style_rules(".a { @media (min-width: 10px) { .b { color: green; } } }");
    assert_eq!(first_rule_with_prop(&rules, "color"), Some(".a .b"));
}

#[test]
/// 嵌套 @supports：条件保留，body 相对父级重父化（包裹回 Rule::Supports）。
fn test_nesting_supports_reparent() {
    let ss = Parser::parse_stylesheet(".a { @supports (display: grid) { & { color: green; } } }");
    // 顶层规则应为 Rule::Supports，其 body 含 `.a { color:green }`。
    let supports = ss
        .rules
        .iter()
        .find_map(|r| if let Rule::Supports(s) = r { Some(s) } else { None })
        .expect("应有 @supports 规则");
    let body: Vec<(String, Vec<String>)> = supports
        .rules
        .iter()
        .filter_map(|r| {
            if let Rule::Style(sr) = r {
                Some((
                    selectors_to_string(&sr.selectors),
                    sr.declarations.iter().map(|d| d.property.clone()).collect(),
                ))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(first_rule_with_prop(&body, "color"), Some(".a"));
}

#[test]
/// 嵌套 @layer：层名保留，body 相对父级重父化（包裹回 Rule::Layer）。
fn test_nesting_layer_reparent() {
    let ss = Parser::parse_stylesheet(".a { @layer base { & { color: green; } } }");
    let layer = ss
        .rules
        .iter()
        .find_map(|r| if let Rule::Layer(l) = r { Some(l) } else { None })
        .expect("应有 @layer 规则");
    assert_eq!(layer.name, "base");
    let body: Vec<(String, Vec<String>)> = layer
        .rules
        .iter()
        .filter_map(|r| {
            if let Rule::Style(sr) = r {
                Some((
                    selectors_to_string(&sr.selectors),
                    sr.declarations.iter().map(|d| d.property.clone()).collect(),
                ))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(first_rule_with_prop(&body, "color"), Some(".a"));
}

#[test]
/// forgiving-skip 含 `&` 的参数保留 &（显式嵌套）：`:is(.a, !&)` 在 `.parent` 内应编译为
/// `:is(.a, .parent)`（直接匹配 .a），而非 `.parent :is(.a)`（隐式后代，parent 不存在则永不匹配）。
/// driving: nest-containing-forgiving（`:is(.test-N, !&)` / `:is(.test-N, :unknown(div,&))`）。
fn test_nesting_forgiving_skip_preserves_amp() {
    // `!&`：invalid 选择器含 &，forgiving-skip 后注入 & 标记 → 显式嵌套 → 直接匹配 .a。
    let rules = style_rules(".parent { :is(.a, !&) { color: green; } }");
    // 应有 :is(.a, .parent) 规则（显式，无 .parent 前缀）。
    assert!(
        rules.iter().any(|(s, _)| s.contains(":is(.a, .parent)")),
        "!& 应注入 & 保显式嵌套，实际: {:?}",
        rules
    );

    // `:unknown(div,&)`：未知函数参数内 &，整块消耗后扫描到 & → 注入 & 标记。
    let rules = style_rules(".parent { :is(.b, :unknown(div,&)) { color: green; } }");
    assert!(
        rules.iter().any(|(s, _)| s.contains(":is(.b, .parent)")),
        ":unknown(div,&) 应注入 &，实际: {:?}",
        rules
    );
}
