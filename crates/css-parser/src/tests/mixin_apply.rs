//! k113：css-mixins-1 `@mixin` / `@apply` 解析期展开钉测。
//!
//! 钉面 = WPT css/css-mixins/mixins 判定页形的解析期等价（展开产物选择器/声明/
//! 改名名；值级语义归 style-system/引擎电池判读）。红线：无 `@apply` 单程逐位
//! 不变、`@mixin` generic At 形保持（CSSOM 面）、顶层 `@apply` 不展开。

use crate::ast::*;
use crate::parser::Parser;

// ── 选择器/规则辅助 ─────────────────────────────────────────────────────────

/// 最小选择器序列化（钉测面：Tag/Class/Id/Where/PseudoElement/组合器）。
fn compound_to_string(c: &CompoundSelector) -> String {
    let mut s = String::new();
    if let Some(t) = &c.type_selector {
        match t {
            TypeSelector::Tag(t) => s.push_str(t),
            TypeSelector::Universal => s.push('*'),
            TypeSelector::Namespaced { .. } => {}
        }
    }
    for sub in &c.subclass_selectors {
        match sub {
            SubclassSelector::Class(name) => {
                s.push('.');
                s.push_str(name);
            }
            SubclassSelector::Id(name) => {
                s.push('#');
                s.push_str(name);
            }
            SubclassSelector::Nesting => s.push('&'),
            SubclassSelector::PseudoElement(PseudoElementSelector::Standard(name)) => {
                s.push_str("::");
                s.push_str(name);
            }
            SubclassSelector::PseudoElement(_) => {}
            SubclassSelector::PseudoClass(PseudoClassSelector::Where(list)) => {
                s.push_str(":where(");
                s.push_str(&list.iter().map(sel_to_string).collect::<Vec<_>>().join(", "));
                s.push(')');
            }
            _ => {}
        }
    }
    s
}

fn sel_to_string(sel: &Selector) -> String {
    let mut s = String::new();
    for (compound, comb) in &sel.complex.parts {
        s.push_str(&compound_to_string(compound));
        match comb {
            None => {}
            Some(Combinator::Descendant) => s.push(' '),
            Some(Combinator::Child) => s.push_str(" > "),
            Some(Combinator::NextSibling) => s.push_str(" + "),
            Some(Combinator::SubsequentSibling) => s.push_str(" ~ "),
        }
    }
    s
}

/// 级联可达样式规则收集（递进条件规则 At(media)/Supports/Layer/Container 块体；
/// `@mixin`/`@apply`/`@result` generic At 原样体不进——那是未展开的原始形）：
/// `(选择器, [(属性, 值)])` 按出现顺序。
fn all_style_rules(css: &str) -> Vec<(String, Vec<(String, String)>)> {
    fn walk(rules: &[Rule], out: &mut Vec<(String, Vec<(String, String)>)>) {
        for r in rules {
            match r {
                Rule::Style(sr) => out.push((
                    sr.selectors.iter().map(sel_to_string).collect::<Vec<_>>().join(", "),
                    sr.declarations
                        .iter()
                        .map(|d| (d.property.clone(), d.value.clone()))
                        .collect(),
                )),
                Rule::At(at) if at.name.eq_ignore_ascii_case("media") => {
                    if let AtRuleBody::Block(inner) = &at.body {
                        walk(inner, out);
                    }
                }
                Rule::Layer(lr) => walk(&lr.rules, out),
                Rule::Supports(sr) => walk(&sr.rules, out),
                Rule::Container(cr) => walk(&cr.rules, out),
                _ => {}
            }
        }
    }
    let ss = Parser::parse_stylesheet(css);
    let mut out = Vec::new();
    walk(&ss.rules, &mut out);
    out
}

/// 首个含指定属性的规则的（选择器, 全声明集）。
fn first_with_prop<'a>(
    rules: &'a [(String, Vec<(String, String)>)],
    prop: &str,
) -> Option<&'a (String, Vec<(String, String)>)> {
    rules.iter().find(|(_, decls)| decls.iter().any(|(p, _)| p == prop))
}

/// 样式表内是否存在指定 (选择器, 属性) 的样式规则。
fn has_rule(rules: &[(String, Vec<(String, String)>)], sel: &str, prop: &str) -> bool {
    rules
        .iter()
        .any(|(s, decls)| s == sel && decls.iter().any(|(p, _)| p == prop))
}

/// 顶层规则中 generic At 规则（name 相同者）的数量。
fn count_top_at_rules(css: &str, name: &str) -> usize {
    let ss = Parser::parse_stylesheet(css);
    ss.rules
        .iter()
        .filter(|r| matches!(r, Rule::At(at) if at.name.eq_ignore_ascii_case(name)))
        .count()
}

// ── 定义/注册表 ─────────────────────────────────────────────────────────────

/// mixin-basic：@result 内样式规则展开为父级后代；同名后定义覆盖（last wins）；
/// invalid-name 不注册；missing-result1/2（体级规则/裸声明）不产出。
#[test]
fn test_mixin_basic_expansion_last_wins() {
    let css = "@mixin --m1() { @result { .cls { color: red; } } } \
               @mixin --m1() { @result { .cls { color: green; } } } \
               div { @apply --m1; }";
    let rules = all_style_rules(css);
    // 仅覆盖后的定义产出；`.cls` 以 div 后代落规则。
    assert!(has_rule(&rules, "div .cls", "color"));
    let r = first_with_prop(&rules, "color").expect("expansion rule");
    assert_eq!(r.0, "div .cls");
    assert_eq!(
        r.1.iter().find(|(p, _)| p == "color").map(|(_, v)| v.as_str()),
        Some("green")
    );
}

#[test]
fn test_mixin_invalid_name_not_registered() {
    // invalid-name 定义无效 → 早定义语义（这里无早定义 → apply 未命中）。
    let css = "@mixin invalid-name() { @result { .cls { color: red; } } } \
               div { @apply invalid-name; }";
    assert!(
        !all_style_rules(css)
            .iter()
            .any(|(_, d)| d.iter().any(|(p, _)| p == "color"))
    );
}

#[test]
fn test_mixin_missing_result_no_output() {
    // 体级裸样式规则与裸声明（无 @result）不产出（missing-result1/2）。
    let css = "@mixin --missing-result1() { .cls { color: red; } } \
               @mixin --missing-result2() { color: red; } \
               div { @apply --missing-result1; @apply --missing-result2; }";
    assert!(
        !all_style_rules(css)
            .iter()
            .any(|(_, d)| d.iter().any(|(p, _)| p == "color"))
    );
}

/// mixin-basic --no-parameter-list：裸定义（无参数表）+ result 内容。
#[test]
fn test_mixin_no_parameter_list() {
    let css = "@mixin --no-parameter-list { @result { border: 1px solid green; } } \
               div { @apply --no-parameter-list; }";
    assert!(has_rule(&all_style_rules(css), "div", "border"));
}

/// junk 参数表 → 整条定义不注册（早定义保留——mixin-parameters m1）。
#[test]
fn test_mixin_junk_param_list_keeps_earlier_def() {
    let css = "@mixin --m1() { @result { color: green; } } \
               @mixin --m1(junk-in-parameter-list) { @result { color: red; } } \
               #e1 { @apply --m1; }";
    let rules = all_style_rules(css);
    let r = first_with_prop(&rules, "color").expect("earlier def wins");
    assert_eq!(
        r.1.iter().find(|(p, _)| p == "color").map(|(_, v)| v.as_str()),
        Some("green")
    );
}

/// @layer 包 @mixin 收集 + 括号形 `--m()` 应用（mixin-layers e1/e2）；
/// 跨层同名后源序覆盖（e4 形——层序感知 FIXME k114+，e3 held-F）。
#[test]
fn test_mixin_inside_layer_collected() {
    let css = "@layer one { @mixin --m1() { @result { color: green; } } } \
               #e1 { @apply --m1(); }";
    assert!(has_rule(&all_style_rules(css), "#e1", "color"));

    let css4 = "@layer one { @mixin --m4() { @result { color: red; } } } \
                @layer two { @mixin --m4() { @result { color: green; } } } \
                #e4 { @apply --m4(); }";
    let rules = all_style_rules(css4);
    let r = first_with_prop(&rules, "color").expect("layer two def");
    assert_eq!(
        r.1.iter().find(|(p, _)| p == "color").map(|(_, v)| v.as_str()),
        Some("green")
    );
}

// ── 展开位置/级联形 ─────────────────────────────────────────────────────────

/// mixin-declarations：结果声明包装 & 展开在父声明后（green 胜 red 的源序形）。
#[test]
fn test_apply_result_decls_after_own() {
    let css = "@mixin --m1() { @result { color: green; } } div { color: red; @apply --m1; }";
    let rules = all_style_rules(css);
    // 两条 div 规则：先 red（自身），后 green（展开）。
    let div_rules: Vec<_> = rules.iter().filter(|(s, _)| s == "div").collect();
    assert_eq!(div_rules.len(), 2, "own + expanded: {:?}", div_rules);
    assert_eq!(
        div_rules[0]
            .1
            .iter()
            .find(|(p, _)| p == "color")
            .map(|(_, v)| v.as_str()),
        Some("red")
    );
    assert_eq!(
        div_rules[1]
            .1
            .iter()
            .find(|(p, _)| p == "color")
            .map(|(_, v)| v.as_str()),
        Some("green")
    );
}

/// apply-nested-declarations：伪元素父级 + :where 特异性保持（展开 spec 0）。
#[test]
fn test_apply_pseudo_element_and_where_specificity() {
    let css = "@mixin --m1() { @result { color: green; } } \
               #e1::after { content: \"AFTER\"; @apply --m1; }";
    let rules = all_style_rules(css);
    assert!(has_rule(&rules, "#e1::after", "color"));
    assert!(has_rule(&rules, "#e1::after", "content"));

    let css2 = "@mixin --m2() { @result { color: red; } } \
                :where(#e2), #u1 { @apply --m2; }";
    let rules2 = all_style_rules(css2);
    // 展开保持父级选择器列表形（特异性按列表内逐选择器计——`:where(#e2)` spec 0，
    // 后绿 spec-0 规则同序胜出；:is() 抬升形会破坏该语义）。
    assert!(
        has_rule(&rules2, ":where(#e2), #u1", "color"),
        ":where spec-0 preserved: {:?}",
        rules2
    );
}

/// apply-within-mixin：嵌套 @apply 后向引用（两程捕获）+ & 链。
#[test]
fn test_apply_within_mixin_forward_reference() {
    let css = "@mixin --m1() { @result { &.a { @apply --m2; } } } \
               .c { @apply --m1; } \
               @mixin --m2() { @result { &.b { color: green; } } }";
    let rules = all_style_rules(css);
    assert!(has_rule(&rules, ".c.a.b", "color"), "chained expansion: {:?}", rules);
}

/// 顶层 @apply 不展开（apply-top-level 红线）：generic At 保持，无样式规则产出。
#[test]
fn test_top_level_apply_not_expanded() {
    let css = ".cls { color: green; } \
               @mixin --m1() { @result { .cls { color: red; } } } \
               @apply --m1;";
    let ss = Parser::parse_stylesheet(css);
    // 顶层 @apply 走 generic At；@mixin 本体 generic At（双程门下仍保持）。
    assert_eq!(count_top_at_rules(css, "apply"), 1);
    assert_eq!(count_top_at_rules(css, "mixin"), 1);
    // .cls 自身规则在，展开面（.cls 第二条 red）不存在。
    assert!(ss.rules.iter().any(|r| matches!(r, Rule::Style(_))));
    let cls_rules = all_style_rules(css).iter().filter(|(s, _)| s == ".cls").count();
    assert_eq!(cls_rules, 1, "only own .cls rule, no expansion output");
}

// ── 局部/参数（卫生改名）────────────────────────────────────────────────────

/// mixin-locals m1：局部改名落元素、体级裸常规声明丢弃、result 引用改名。
#[test]
fn test_locals_renamed_and_bare_decls_dropped() {
    let css = "@mixin --m1() { --my-color: green; font-size: 200px; \
               @result { color: var(--my-color); } } \
               #e1 { color: red; @apply --m1; }";
    let rules = all_style_rules(css);
    // 局部落元素（改名）；font-size 不落。
    let local_rule = rules
        .iter()
        .find(|(_, d)| d.iter().any(|(p, _)| p.starts_with("--zwmx")))
        .expect("renamed local landed");
    assert!(
        local_rule
            .1
            .iter()
            .any(|(p, v)| p == "--zwmx0--my-color" && v == "green")
    );
    assert!(!rules.iter().any(|(_, d)| d.iter().any(|(p, _)| p == "font-size")));
    // result 声明引用改名（展开规则同时承载改名局部与改名引用；own 规则的 red
    // 先于展开规则出现，first_with_prop 会命中 own——按改名局部定位展开规则）。
    assert_eq!(
        local_rule.1.iter().find(|(p, _)| p == "color").map(|(_, v)| v.as_str()),
        Some("var(--zwmx0--my-color)")
    );
}

/// mixin-locals m2/m5：局部互解（序独立）+ @result 后局部可见——均元素级 var 链。
#[test]
fn test_locals_resolve_against_each_other() {
    let css = "@mixin --m2() { --my-color: var(--real-color); --real-color: green; \
               @result { color: var(--my-color); } } \
               #e2 { @apply --m2; }";
    let rules = all_style_rules(css);
    let my = rules
        .iter()
        .find(|(_, d)| d.iter().any(|(p, _)| p == "--zwmx0--my-color"))
        .expect("local 1");
    assert_eq!(
        my.1.iter()
            .find(|(p, _)| p == "--zwmx0--my-color")
            .map(|(_, v)| v.as_str()),
        Some("var(--zwmx0--real-color)")
    );
    assert!(
        rules
            .iter()
            .any(|(_, d)| d.iter().any(|(p, v)| p == "--zwmx0--real-color" && v == "green"))
    );
}

/// mixin-parameters m2/m2b/m3：位置实参绑定 + `{}` 解包。
#[test]
fn test_param_binding_and_brace_unwrap() {
    let css = "@mixin --m3(--real-color) { --my-color: var(--real-color); \
               @result { color: var(--my-color); } } \
               #e3 { @apply --m3({green}); }";
    let rules = all_style_rules(css);
    assert!(
        rules
            .iter()
            .any(|(_, d)| d.iter().any(|(p, v)| p == "--zwmx0--real-color" && v == "green")),
        "brace-unwrapped binding: {:?}",
        rules
    );
}

/// mixin-parameters m4/m14：缺省绑定（裸形与空括号形）。
#[test]
fn test_param_defaults_bound() {
    let css4 = "@mixin --m4(--my-color: green) { @result { color: var(--my-color); } } \
                #e4 { @apply --m4; }";
    let rules = all_style_rules(css4);
    assert!(
        rules
            .iter()
            .any(|(_, d)| d.iter().any(|(p, v)| p == "--zwmx0--my-color" && v == "green"))
    );

    let css14 = "@mixin --m14(--param: green) { @result { color: var(--param); } } \
                 #e14 { @apply --m14(); }";
    assert!(
        all_style_rules(css14)
            .iter()
            .any(|(_, d)| d.iter().any(|(p, v)| p == "--zwmx0--param" && v == "green"))
    );
}

/// mixin-parameters m6b：裸形缺参不绑定（var 回退兑现）；m6 有参时回退不生效。
#[test]
fn test_bare_apply_missing_param_not_bound() {
    let css = "@mixin --m6b(--my-color) { @result { color: var(--my-color, navy); } } \
               #e6b { @apply --m6b; }";
    let rules = all_style_rules(css);
    let r = first_with_prop(&rules, "color").expect("expanded");
    // 不绑定 → 无帧项 → var() 引用保原名（元素级 `--my-color` 缺席 → navy 回退）。
    assert_eq!(
        r.1.iter().find(|(p, _)| p == "color").map(|(_, v)| v.as_str()),
        Some("var(--my-color, navy)")
    );
    assert!(
        !rules
            .iter()
            .any(|(_, d)| d.iter().any(|(p, _)| p.starts_with("--zwmx")))
    );
}

/// mixin-parameters m5/m17/m16：too-many-args / 非法实参 / 非法缺省 → 整条跳过。
#[test]
fn test_invalid_invocations_skip_expansion() {
    // 跳过 = 无展开产出（空壳 own 规则合法物化，断言无声明）。
    let no_decls = |css: &str| all_style_rules(css).iter().all(|(_, d)| d.is_empty());
    let css5 = "@mixin --m5() { @result { color: red; } } #e5 { @apply --m5(too-many-parameters); }";
    assert!(no_decls(css5));
    let css17 = "@mixin --m17(--arg) { @result { color: red; } } #e17 { @apply --m17(!!!); }";
    assert!(no_decls(css17));
    let css16 = "@mixin --m16(--arg: !!) { @result { color: red; } } #e16 { @apply --m16(); }";
    assert!(no_decls(css16));
    // m2c：括号形缺参无缺省 → 跳过（held-F 域——b0 黑 = 引擎现行为同）。
    let css2c = "@mixin --m2c(--my-color) { @result { color: var(--my-color); } } \
                 #e2c { @apply --m2c(); }";
    assert!(no_decls(css2c));
}

/// mixin-parameters m9：非法 type() 参数不绑定但 mixin 仍展开（does not fail entire mixin）。
#[test]
fn test_illegal_type_param_not_bound_mixin_expands() {
    let css = "@mixin --m9(--my-length type(length)) { @result { font-size: 10px; \
               --some-length: var(--my-length); } } \
               #e9 { @apply --m9(1.0em); }";
    let rules = all_style_rules(css);
    let r = first_with_prop(&rules, "font-size").expect("mixin expanded");
    assert_eq!(
        r.1.iter().find(|(p, _)| p == "font-size").map(|(_, v)| v.as_str()),
        Some("10px")
    );
    // 不绑定 → var() 引用保原名（元素级缺席 → IACVT）；参数不落元素。
    assert!(r.1.iter().any(|(p, v)| p == "--some-length" && v == "var(--my-length)"));
    assert!(
        !rules
            .iter()
            .any(|(_, d)| d.iter().any(|(p, _)| p.starts_with("--zwmx")))
    );
}

/// mixin-parameters m10：外层参数在内层 mixin 可见（帧链）。
#[test]
fn test_outer_param_visible_in_inner_mixin() {
    let css = "@mixin --m10inner(--inner-color) { @result { color: var(--outer-color); } } \
               @mixin --m10outer(--outer-color) { @result { @apply --m10inner(red); } } \
               #e10 { @apply --m10outer(green); }";
    let rules = all_style_rules(css);
    // 内层 result 的 color 引用落到外层参数的改名名（元素级链可解）。
    let r = first_with_prop(&rules, "color").expect("inner result");
    assert_eq!(
        r.1.iter().find(|(p, _)| p == "color").map(|(_, v)| v.as_str()),
        Some("var(--zwmx0--outer-color)")
    );
}

/// mixin-parameters m12：函数体不经帧链改名——引用保原名（元素级缺席 → black）。
#[test]
fn test_function_body_refs_not_renamed() {
    let css = "@function --f() { result: var(--my-color); } \
               @mixin --m12(--my-color) { @result { color: --f(); } } \
               #e12 { @apply --m12(red); }";
    let rules = all_style_rules(css);
    // 展开发生（参数落元素改名名），--f() 原样未改名。
    assert!(
        rules
            .iter()
            .any(|(_, d)| d.iter().any(|(p, _)| p == "--zwmx0--my-color"))
    );
    let r = first_with_prop(&rules, "color").expect("result");
    assert_eq!(
        r.1.iter().find(|(p, _)| p == "color").map(|(_, v)| v.as_str()),
        Some("--f()")
    );
}

/// mixin-parameters m13：实参调用点求值——嵌套 @apply 实参按外层帧改名。
#[test]
fn test_args_resolved_at_call_site() {
    let css = "@mixin --m13-override-outer(--inner, --outer) { @result { color: var(--inner); } } \
               @mixin --m13-set-inner(--inner) { @result { @apply --m13-override-outer(var(--inner), red); } } \
               @mixin --m13(--outer) { @result { @apply --m13-set-inner(var(--outer)); } } \
               #e13 { @apply --m13(green); }";
    let rules = all_style_rules(css);
    // 最内层绑定：--inner := var(--zwmx?--inner)（set-inner 帧）——链上调用点改名。
    let inner_binding = rules.iter().find(|(_, d)| {
        d.iter()
            .any(|(p, v)| p.starts_with("--zwmx") && p.ends_with("--inner") && v.starts_with("var(--zwmx"))
    });
    assert!(inner_binding.is_some(), "call-site renamed chain: {:?}", rules);
}

/// mixin-cycle：a→b→c→(b) 栈跳过——三条 --last-processed（a/b/c）产出且无第
/// 四条（不发散；值级 'c' 兑现归元素级 cascade 判读）。
#[test]
fn test_cycle_skip_in_progress() {
    let css = "@mixin --a() { @result { --last-processed: a; @apply --b; } } \
               @mixin --b() { @result { --last-processed: b; @apply --c; } } \
               @mixin --c() { @result { --last-processed: c; @apply --b; } } \
               #target { @apply --a; }";
    let rules = all_style_rules(css);
    let n = rules
        .iter()
        .filter(|(_, d)| d.iter().any(|(p, _)| p == "--last-processed"))
        .count();
    assert_eq!(n, 3, "a/b/c each once, re-entrant b skipped: {:?}", rules);
}

// ── @contents ───────────────────────────────────────────────────────────────

/// contents-rule：apply 块优先（m1/m3）、无块用回退（m8）、空块压制回退（m9）、
/// 空回退无产出（m6）。
#[test]
fn test_contents_precedence_arms() {
    // 块覆盖回退。
    let css3 = "@mixin --m3() { @result { &.a { @contents { color: blue; } } } } \
                .b { @apply --m3 { color: green; } }";
    let rules = all_style_rules(css3);
    let r = first_with_prop(&rules, "color").expect("contents");
    assert_eq!(r.0, ".b.a");
    assert_eq!(
        r.1.iter().find(|(p, _)| p == "color").map(|(_, v)| v.as_str()),
        Some("green")
    );

    // 无块 → 回退。
    let css8 = "@mixin --m8() { @result { @contents { color: green; } } } #e8 { @apply --m8; }";
    assert!(has_rule(&all_style_rules(css8), "#e8", "color"));

    // 空块压制回退。
    let css9 = "@mixin --m9() { @result { @contents { color: red; } } } \
                #e9 { color: green; @apply --m9 {}; }";
    let rules9 = all_style_rules(css9);
    assert_eq!(
        rules9
            .iter()
            .filter(|(_, d)| d.iter().any(|(p, _)| p == "color"))
            .count(),
        1,
        "fallback suppressed by empty block: {:?}",
        rules9
    );

    // 空回退块：@contents 后继声明仍在（m6）。
    let css6 = "@mixin --m6() { @result { @contents { } color: green; } } #e6 { @apply --m6; }";
    assert!(has_rule(&all_style_rules(css6), "#e6", "color"));
}

// ── 条件 ────────────────────────────────────────────────────────────────────

/// mixin-conditionals m1/m3：条件包 @result 解包；条件包局部改名（外壳保留+递归）。
#[test]
fn test_conditionals_wrapping_result_and_locals() {
    let css = "@mixin --m1() { @result { color: red; } \
               @media (width < 1000px) { @result { color: green; } } } \
               #e1 { @apply --m1; }";
    let ss = Parser::parse_stylesheet(css);
    // 展开后存在 @media 块，体内样式规则相对父级（bare decl → & → #e1）。
    let media = ss
        .rules
        .iter()
        .find_map(|r| match r {
            Rule::At(at) if at.name == "media" => Some(at),
            _ => None,
        })
        .expect("conditional preserved");
    let AtRuleBody::Block(inner) = &media.body else {
        panic!("block")
    };
    assert!(inner.iter().any(|r| matches!(r, Rule::Style(sr)
        if sel_to_string(&sr.selectors[0]) == "#e1"
            && sr.declarations.iter().any(|d| d.property == "color" && d.value == "green"))));

    let css3 = "@mixin --m3() { --some-color: red; \
               @media (width < 3000px) { --some-color: green; } \
               @result { color: var(--some-color); } } \
               #e3 { @apply --m3; }";
    let rules = all_style_rules(css3);
    // 局部裸定义与条件内定义同名 → 同一改名名（后者 media 内）。
    let renamed: Vec<_> = rules
        .iter()
        .flat_map(|(_, d)| d.iter())
        .filter(|(p, _)| p.starts_with("--zwmx") && p.ends_with("--some-color"))
        .collect();
    assert_eq!(
        renamed.len(),
        2,
        "bare + conditional locals share renamed name: {:?}",
        renamed
    );
}

// ── 逐字节红线 ──────────────────────────────────────────────────────────────

/// 无 @apply 单程：@mixin generic At 形保持（CSSOM 面）；含 @apply 双程下
/// @mixin generic At 形同样保持。
#[test]
fn test_mixin_generic_at_shape_preserved() {
    let single = "@mixin --m { @result { color: green; } }";
    assert_eq!(count_top_at_rules(single, "mixin"), 1);
    assert!(
        Parser::parse_stylesheet(single)
            .rules
            .iter()
            .all(|r| !matches!(r, Rule::Style(_)))
    );

    let two_pass = "@mixin --m { @result { color: green; } } .a { color: @apply-unused; }";
    assert_eq!(count_top_at_rules(two_pass, "mixin"), 1);
}

/// AST parity：@mixin 规则的 prelude/body 形不变（generic At name=mixin,
/// prelude=定义头原串）。
#[test]
fn test_mixin_at_rule_prelude_shape() {
    let css = "@mixin --m1() { @result { color: green; } } .a { @apply --m1; }";
    let ss = Parser::parse_stylesheet(css);
    let at = ss
        .rules
        .iter()
        .find_map(|r| match r {
            Rule::At(at) if at.name == "mixin" => Some(at),
            _ => None,
        })
        .expect("generic At preserved");
    assert_eq!(at.prelude, "--m1()");
    assert!(matches!(&at.body, AtRuleBody::Block(_)));
}
