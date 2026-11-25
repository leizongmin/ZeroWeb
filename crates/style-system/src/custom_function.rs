//! CSS 自定义函数（css-mixins-1 `@function` / `<dashed-function>`，k109 刀）。
//!
//! 三件：注册表收集（§2.1 顶层规则）、`--name(...)` 调用形扫描替换（§3）、
//! 函数体求值（§3.1 假设子元素注册语义 ＋ §2.3.1 shadowing）。函数内
//! random 键按 css-values-5 §9.4.1 ua-ident 捕获——键名拼接
//! `{fname}-{internal}` 后复用既有 [`substitute_random_values`]（零改
//! random.rs）。
//!
//! <https://drafts.csswg.org/css-mixins-1/>
//! <https://drafts.csswg.org/css-values-5/#random-evaluating>

use std::collections::HashMap;

use zero_css_parser::Stylesheet;
use zero_css_parser::ast::{FunctionRule, Rule};
use zero_css_parser::parser::split_top_level_commas;
use zero_css_parser::values::{RandomEvalContext, substitute_random_values};

use crate::computed::resolve_env_and_var;

/// 求值深度上限（循环防护：自引用/互引用函数链 guaranteed-invalid，
/// css-mixins-1 §3 invalid at substitution——调用形保留原文交下游拒）。
const MAX_EVAL_DEPTH: usize = 32;

/// 扫描样式表顶层 `@function` 规则，构建「函数名 → 规则」映射。
///
/// 同名后者胜（css-mixins-1 §4 执行模型——cascade layer 维度本层未建模，
/// FIXME 随 layer 刀补齐）。仅顶层有效（@property 同型——全局注册无作用域，
/// 不递归进 @media 等条件块）。
pub(crate) fn collect_custom_functions<'a>(
    stylesheets: impl IntoIterator<Item = &'a Stylesheet>,
) -> HashMap<String, FunctionRule> {
    let mut map: HashMap<String, FunctionRule> = HashMap::new();
    for ss in stylesheets {
        for rule in &ss.rules {
            if let Rule::Function(fr) = rule {
                map.insert(fr.name.clone(), fr.clone());
            }
        }
    }
    map
}

/// 值文本是否含 `<dashed-function>` 调用形（`--ident(`，ident 边界感知）。
///
/// 站前 any() 预滤门（4.75 attr 站 has_attr_word 同型）；假阳（url 载荷、
/// 字符串字面量内形似体）由站内逐点查注册表天然兜住。
pub(crate) fn has_dashed_function_call(value: &str) -> bool {
    let chars: Vec<(usize, char)> = value.char_indices().collect();
    for k in 0..chars.len() {
        let (i, c) = chars[k];
        if c != '-' || !value[i..].starts_with("--") {
            continue;
        }
        // ident 边界：`a--f(` 是单 ident `a--f` 的函数形，非双杠名调用。
        if i > 0 && is_ident_char(chars[k - 1].1) {
            continue;
        }
        let mut j = k + 2;
        while j < chars.len() && is_ident_char(chars[j].1) {
            j += 1;
        }
        if j > k + 2 && j < chars.len() && chars[j].1 == '(' {
            return true;
        }
    }
    false
}

/// `<dashed-function>` 替换（css-mixins-1 §3）：扫描值文本中已注册
/// `--name(...)` 调用形，实参先于调用属性语境替换（§3 arbitrary
/// substitution 先行——嵌套函数递归＋random 按位求值），参数按位匹配
/// （尾逗号/纯空白实参移除，§3）后求值函数体，产物原位替换。
///
/// guaranteed-invalid（未注册名/求值失败/深度超限）：该调用形保留原文——
/// 下游 apply 对函数形语法拒（IACVT 等价；基线本就把函数形留作字面值，
/// 语义同向）。字符串字面量/引号域内的形似体本层按文本扫描处理，
/// FIXME 随 tokenizer 级替换刀收严。
pub(crate) fn substitute_dashed_functions(
    value: &str,
    consuming_prop: &str,
    registry: &HashMap<String, FunctionRule>,
    customs: &HashMap<String, String>,
    element_seed: Option<u64>,
    font_base_px: Option<f64>,
    depth: usize,
) -> String {
    if depth >= MAX_EVAL_DEPTH {
        return value.to_string();
    }
    let chars: Vec<(usize, char)> = value.char_indices().collect();
    let mut out = String::with_capacity(value.len());
    let mut last = 0usize; // 未产段起点（字节偏移）
    let mut k = 0usize; // chars 游标
    while k < chars.len() {
        let (i, c) = chars[k];
        if c == '-' && value[i..].starts_with("--") && (i == 0 || !is_ident_char(chars[k - 1].1)) {
            let mut j = k + 2;
            while j < chars.len() && is_ident_char(chars[j].1) {
                j += 1;
            }
            if j > k + 2 && j < chars.len() && chars[j].1 == '(' {
                let name = &value[i..chars[j].0];
                if let Some(rule) = registry.get(name) {
                    if let Some(cm) = matching_close_paren(&chars, j) {
                        let inner = &value[chars[j].0 + 1..chars[cm].0];
                        // §3：实参先行替换——嵌套 dashed-function 递归（深度
                        // 护栏）＋random 调用属性语境求值（写面 fold 已烤键
                        // 原样保留，omitted 形本站补展开；键序数＝消费属性
                        // 文本序）。
                        let ctx = RandomEvalContext {
                            element_seed,
                            pct_typed: false,
                            font_base_px,
                        };
                        let args: Vec<String> = split_top_level_commas(inner)
                            .into_iter()
                            .map(|a| a.trim().to_string())
                            .filter(|a| !a.is_empty()) // §3 纯空白实参移除
                            .map(|a| {
                                let nested = substitute_dashed_functions(
                                    &a,
                                    consuming_prop,
                                    registry,
                                    customs,
                                    element_seed,
                                    font_base_px,
                                    depth + 1,
                                );
                                substitute_random_values(&nested, consuming_prop, &ctx)
                                    .map(std::borrow::Cow::into_owned)
                                    .unwrap_or(nested)
                            })
                            .collect();
                        out.push_str(&value[last..i]);
                        match evaluate_custom_function(rule, &args, customs, element_seed, font_base_px, depth + 1) {
                            Some(product) => {
                                // 产物可含 body 级嵌套调用（untyped result
                                // 引用他函数）——深度护栏内再扫。
                                let rescanned = substitute_dashed_functions(
                                    &product,
                                    consuming_prop,
                                    registry,
                                    customs,
                                    element_seed,
                                    font_base_px,
                                    depth + 1,
                                );
                                out.push_str(&rescanned);
                            }
                            None => out.push_str(&value[i..chars[cm].0 + 1]), // 调用形保留原文
                        }
                        last = chars[cm].0 + 1;
                        k = cm + 1;
                        continue;
                    }
                }
            }
        }
        k += 1;
    }
    out.push_str(&value[last..]);
    out
}

/// 求值单个函数体（css-mixins-1 §3.1）：参数按位注入（缺位兑 default，
/// 无 default guaranteed-invalid）→ locals 声明序后者胜 → result（后者胜）
/// 求值。untyped result＝作用域 var 代入后原文返回（random as-is 交外层
/// 属性 4.7 站按消费属性键求值）；typed result＝函数会话内完成 random
/// 求值（键 `{fname}-result`，css-values-5 §9.4.1 ua-ident 捕获）。
fn evaluate_custom_function(
    rule: &FunctionRule,
    args: &[String],
    customs: &HashMap<String, String>,
    element_seed: Option<u64>,
    font_base_px: Option<f64>,
    depth: usize,
) -> Option<String> {
    if depth >= MAX_EVAL_DEPTH {
        return None;
    }
    let fname = &rule.name;
    // 作用域基座：call-site customs（§2.3 shadowing：locals > params >
    // call-site）。参数逐个物化时已物化参数并入基座（参数间引用最小单趟，
    // 任意序非循环解析 FIXME 随 spec 轮补）。
    let mut scope: HashMap<String, String> = customs.clone();
    for (idx, param) in rule.params.iter().enumerate() {
        let raw = match args.get(idx) {
            Some(a) if !a.is_empty() => a.clone(),
            _ => param.default.clone()?, // 缺参无 default → guaranteed-invalid
        };
        // typed 参数＝假设元素按参数类型注册（§3.1）→ 会话内求值；untyped
        // 参数原样。
        let value = if param.ty.is_some() {
            eval_typed_value(
                &raw,
                &scope,
                &format!("{fname}-{}", param.name),
                element_seed,
                font_base_px,
            )?
        } else {
            raw
        };
        scope.insert(param.name.clone(), value);
    }
    // locals：声明序后者胜（§4）。与 typed 参数同名者继承参数注册语义
    //（值覆写、类型仍在——§2.3.1 shadowing；③⑦ 断言定谳：有无 typed 参数
    // 决定 locals 注册语义）→ 会话内求值；其余 unregistered 原文。
    for decl in &rule.decls {
        if !decl.name.starts_with("--") {
            continue;
        }
        let typed = rule.params.iter().any(|p| p.name == decl.name && p.ty.is_some());
        let value = if typed {
            eval_typed_value(
                &decl.value,
                &scope,
                &format!("{fname}-{}", decl.name),
                element_seed,
                font_base_px,
            )?
        } else {
            decl.value.clone()
        };
        scope.insert(decl.name.clone(), value);
    }
    // result：声明序后者胜（§4 执行模型）。
    let result = rule.decls.iter().rev().find(|d| d.name == "result")?;
    let text = resolve_env_and_var(&result.value, &scope);
    if rule.returns.is_some() {
        eval_typed_value(&text, &scope, &format!("{fname}-result"), element_seed, font_base_px)
    } else {
        Some(text)
    }
}

/// typed 值会话内求值（css-mixins-1 §3.1 假设子元素注册语义）：作用域
/// var 代入后 random 替换——键 ua-ident 捕获（css-values-5 §9.4.1 type
/// that causes evaluation），键名＝`{fname}-{internal}`（参数/局部名带双杠
/// → 三连杠拼形；result 无杠名 → 单杠拼形）。None＝guaranteed-invalid
///（不可解 random——调用形保留原文交下游拒）。
fn eval_typed_value(
    text: &str,
    scope: &HashMap<String, String>,
    key_property: &str,
    element_seed: Option<u64>,
    font_base_px: Option<f64>,
) -> Option<String> {
    let resolved = resolve_env_and_var(text, scope);
    let ctx = RandomEvalContext {
        element_seed,
        pct_typed: false,
        font_base_px,
    };
    substitute_random_values(&resolved, key_property, &ctx).map(std::borrow::Cow::into_owned)
}

/// CSS ident 字符判据（ASCII 域＋非 ASCII 域收宽——k109 页面全 ASCII，
/// 非 ASCII ident 收宽为最小保守形）。
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || !c.is_ascii()
}

/// 括号深度扫描取匹配闭括号（chars 游标制）。引号域内括号本层不豁免，
/// FIXME 与调用形扫描同刀收严。
fn matching_close_paren(chars: &[(usize, char)], open: usize) -> Option<usize> {
    let mut d = 0i32;
    for (m, (_, ch)) in chars.iter().enumerate().skip(open) {
        match ch {
            '(' => d += 1,
            ')' => {
                d -= 1;
                if d == 0 {
                    return Some(m);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::Parser;

    const SEED: Option<u64> = Some(7);

    fn registry(css: &str) -> HashMap<String, FunctionRule> {
        collect_custom_functions(std::iter::once(&Parser::parse_stylesheet(css)))
    }

    /// 钉：untyped result＝原文替换（random as-is 留外层属性求值）。
    #[test]
    fn untyped_result_keeps_raw_random() {
        let reg = registry("@function --f() { result: random(--foo property-index-scoped, 1, 1e6); }");
        let out = substitute_dashed_functions("--f()", "--number", &reg, &HashMap::new(), SEED, None, 0);
        assert!(
            out.to_ascii_lowercase().contains("random("),
            "raw random preserved; got {out}"
        );
    }

    /// 钉：untyped 两 locals＝var 作用域代入后原文两份（外层属性序数 1/2
    /// 键分异值——④ 断言机制）。
    #[test]
    fn untyped_two_locals_resolve_scope() {
        let reg = registry(
            "@function --locals() { --x: random(--foo property-index-scoped, 1, 1e6); \
             --y: random(--foo property-index-scoped, 1, 1e6); result: var(--x) var(--y); }",
        );
        let out = substitute_dashed_functions("--locals()", "--numbers", &reg, &HashMap::new(), SEED, None, 0);
        let n = out.to_ascii_lowercase().matches("random(").count();
        assert_eq!(n, 2, "both locals substituted as raw text; got {out}");
        assert!(
            !out.contains("var("),
            "function-local vars resolved in-station; got {out}"
        );
    }

    /// 钉：typed result＝会话内数值化（键 `{fname}-result`）——同名同键
    /// 恒同值（确定性），异名异键异值（② 断言机制）。
    #[test]
    fn typed_result_numeric_and_fname_scoped() {
        let css = "@function --f-number() returns <number> { result: random(--foo property-index-scoped, 1, 1e6); } \
                   @function --g-number() returns <number> { result: random(--foo property-index-scoped, 1, 1e6); }";
        let reg = registry(css);
        let f1 = substitute_dashed_functions("--f-number()", "--number", &reg, &HashMap::new(), SEED, None, 0);
        let f2 = substitute_dashed_functions("--f-number()", "--number", &reg, &HashMap::new(), SEED, None, 0);
        let g = substitute_dashed_functions("--g-number()", "--number", &reg, &HashMap::new(), SEED, None, 0);
        let v1: f64 = f1.trim().parse().expect("typed result is numeric");
        assert!((1.0..=1e6).contains(&v1), "in range; got {f1}");
        assert_eq!(f1, f2, "same fname same key deterministic");
        assert_ne!(f1, g, "distinct fnames capture distinct keys");
    }

    /// 钉：typed default 参数＝会话内求值（键 `{fname}-参数名` 三连杠形）——
    /// 异函数异值（⑥ 断言机制）。
    #[test]
    fn typed_default_argument_evaluates_per_fname() {
        let css = "@function --f-default-arguments(--x <number>: random(--foo property-index-scoped, 1, 1e6)) { result: var(--x); } \
                   @function --g-default-arguments(--x <number>: random(--foo property-index-scoped, 1, 1e6)) { result: var(--x); }";
        let reg = registry(css);
        let f = substitute_dashed_functions(
            "--f-default-arguments()",
            "--number",
            &reg,
            &HashMap::new(),
            SEED,
            None,
            0,
        );
        let g = substitute_dashed_functions(
            "--g-default-arguments()",
            "--number",
            &reg,
            &HashMap::new(),
            SEED,
            None,
            0,
        );
        f.trim().parse::<f64>().expect("default-arg result numeric");
        assert_ne!(f, g, "default random keyed by fname");
    }

    /// 钉：实参 random＝调用属性语境先替换（烤键同键同值）——异函数同值
    ///（⑧ 断言机制）；嵌套 dashed-function 实参递归。
    #[test]
    fn argument_random_at_call_site_equal_across_fnames() {
        let css = "@function --f-arguments(--x <number>) { result: var(--x); } \
                   @function --g-arguments(--x <number>) { result: var(--x); } \
                   @function --h-inner() returns <number> { result: 42; }";
        let reg = registry(css);
        let mk = |name: &str| {
            let call = format!("{name}(random(--foo property-index-scoped, 1, 1e6))");
            substitute_dashed_functions(&call, "--number", &reg, &HashMap::new(), SEED, None, 0)
        };
        let f = mk("--f-arguments");
        let g = mk("--g-arguments");
        let vf: f64 = f.trim().parse().expect("call-site random resolved to number");
        assert!((1.0..=1e6).contains(&vf), "in range; got {f}");
        assert_eq!(f, g, "same call-site key equal across fnames");
        let nested = substitute_dashed_functions(
            "--f-arguments(--h-inner())",
            "--number",
            &reg,
            &HashMap::new(),
            SEED,
            None,
            0,
        );
        assert_eq!(nested.trim(), "42", "nested dashed call substituted in argument");
    }

    /// 钉：local 覆写 typed 参数＝仍按参数类型会话内求值（键 fname-参数名）——
    /// 数值化且异函数异值（⑦ 断言机制）。
    #[test]
    fn local_shadowing_typed_param_stays_registered() {
        let css = "@function --f-locals-arguments(--x <number>) { --x: random(--foo property-index-scoped, 1, 1e6); result: var(--x); } \
                   @function --g-locals-arguments(--x <number>) { --x: random(--foo property-index-scoped, 1, 1e6); result: var(--x); }";
        let reg = registry(css);
        let f = substitute_dashed_functions(
            "--f-locals-arguments(0)",
            "--number",
            &reg,
            &HashMap::new(),
            SEED,
            None,
            0,
        );
        let g = substitute_dashed_functions(
            "--g-locals-arguments(0)",
            "--number",
            &reg,
            &HashMap::new(),
            SEED,
            None,
            0,
        );
        let vf: f64 = f
            .trim()
            .parse()
            .expect("shadowed local evaluated in-session, not raw arg");
        assert!((1.0..=1e6).contains(&vf), "local value in range; got {f}");
        assert_ne!(f, g, "shadowed local keyed by fname");
    }

    /// 钉：typed result var×2＝每引用新鲜序数键（-1/-2）——两位置异值
    ///（⑤ 断言机制，`<number>+` 双位）。
    #[test]
    fn typed_result_fresh_index_per_reference() {
        let reg = registry(
            "@function --loc-reg() returns <number>+ { --x: random(--foo property-index-scoped, 1, 1e6); \
             result: var(--x) var(--x); }",
        );
        let out = substitute_dashed_functions("--loc-reg()", "--numbers", &reg, &HashMap::new(), SEED, None, 0);
        let parts: Vec<f64> = out.split_whitespace().filter_map(|s| s.parse().ok()).collect();
        assert_eq!(parts.len(), 2, "two typed positions; got {out}");
        assert_ne!(parts[0], parts[1], "fresh index per reference");
    }

    /// 钉：未注册名＝调用形保留原文（guaranteed-invalid 同向基线）。
    #[test]
    fn unregistered_name_kept_verbatim() {
        let reg = registry("@function --f() { result: 1; }");
        let out = substitute_dashed_functions(
            "var(--x, --nope()) --nope(1)",
            "--number",
            &reg,
            &HashMap::new(),
            SEED,
            None,
            0,
        );
        assert_eq!(out, "var(--x, --nope()) --nope(1)");
    }

    /// 钉：缺参无 default＝guaranteed-invalid 原文保留；纯空白/尾逗号实参
    /// 移除不误判（§3）。
    #[test]
    fn missing_arg_without_default_kept_and_blank_args_dropped() {
        let reg = registry("@function --f-arguments(--x <number>) { result: 1; }");
        let missing = substitute_dashed_functions("--f-arguments()", "--number", &reg, &HashMap::new(), SEED, None, 0);
        assert_eq!(missing, "--f-arguments()", "no default → invocation kept verbatim");
        let blank = substitute_dashed_functions(
            "--f-arguments( , 42 )",
            "--number",
            &reg,
            &HashMap::new(),
            SEED,
            None,
            0,
        );
        assert_eq!(blank.trim(), "1", "blank arg removed → 42 binds to first param");
    }

    /// 钉：untyped 参数缺省（无类型无 default）＝guaranteed-invalid。
    #[test]
    fn untyped_param_missing_is_guaranteed_invalid() {
        let reg = registry("@function --f(--x) { result: var(--x); }");
        let out = substitute_dashed_functions("--f()", "--number", &reg, &HashMap::new(), SEED, None, 0);
        assert_eq!(out, "--f()", "missing untyped arg without default → kept verbatim");
    }

    /// 钉：循环防护（自引用函数）＝深度护栏内原文保留不挂死。
    #[test]
    fn cyclic_call_depth_guarded() {
        let reg = registry("@function --a() { result: --a(); }");
        let out = substitute_dashed_functions("--a()", "--number", &reg, &HashMap::new(), SEED, None, 0);
        assert!(!out.contains("--a(--a(--a"), "depth guard terminates; got {out}");
    }
}
