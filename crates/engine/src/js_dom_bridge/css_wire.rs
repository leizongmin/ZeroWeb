//! CSS 选择器 / 样式规则 wire 序列化（R3001 从父文件拆出，控制主文件行数）。
//!
//! 把 `zero_css_parser` AST（Selector / AttributeMatcher / PseudoClass / Declaration）pragmatic
//! 序列化为 CSS 字符串，并从 `<style>` 元素读规则成 `\x1f`/`\x1e` wire 串（供 `__zw_style_rules`
//! → shim CSSStyleSheet.cssRules 读，R2808；`__zw_query_match` 单查询，R2663）。`use super::*` 复用
//! 父模块 `parse_html` / `find_by_selector` / `unique_selector_for_node`（子模块可访祖先私有项）。
//! pub 函数经 `pub use css_wire::*` 重导出，调用点零改动。

use super::*;

/// 从 HTML 快照查询首个匹配元素的**唯一**选择器（供 `__zw_query_match`→querySelector）。
///
/// 用 [`super::unique_selector_for_node`]（`#id`/`tag.class`/`tag` 唯一时返回；歧义时 nth-child 结构
/// 路径回落）——保证返回的 selector 在 dom_html 中**唯一定位**该元素。querySelector 对无 id/class
/// 的歧义元素（`<option>`/`<li>` 等）此前返回 `stable_selector`（如 "option"，多 option 时指向首个），
/// 导致后续 `el.selected`/`el.value` 读错元素；唯一选择器修复之。同一 dom_html 上与旧实现解析到同一元素。
pub fn query_match_selector(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    query_match_selector_doc(&doc, selector)
}

/// 查询 doc 版本（免每次查询重新 parse——见 register_dom_callbacks 查询缓存）。
pub fn query_match_selector_doc(doc: &zero_dom::Document, selector: &str) -> String {
    find_by_selector(doc, selector)
        .and_then(|n| unique_selector_for_node(doc, n))
        .unwrap_or_default()
}

/// CSS AttributeMatcher → 字符串片段（`[name]` 中的 `name` 之后部分）。供 [`css_selector_to_string`]。
fn attr_matcher_str(m: &zero_css_parser::AttributeMatcher) -> String {
    use zero_css_parser::AttributeMatcher as M;
    match m {
        M::Exists => String::new(),
        M::Exact(v) => format!("=\"{}\"", v),
        M::Includes(v) => format!("~=\"{}\"", v),
        M::DashMatch(v) => format!("|=\"{}\"", v),
        M::Prefix(v) => format!("^=\"{}\"", v),
        M::Suffix(v) => format!("$=\"{}\"", v),
        M::Substring(v) => format!("*=\"{}\"", v),
    }
}

/// CSS PseudoClassSelector → 字符串（含前导 `:`）。功能性伪类（:not/:is/:nth-* 带参数）best-effort
/// 简化（参数不重建——`<style>` 检视场景常见简单伪类 :hover/:focus 精确，功能性 rare）。
fn pseudo_class_str(p: &zero_css_parser::PseudoClassSelector) -> String {
    use zero_css_parser::PseudoClassSelector as P;
    match p {
        P::Simple(s) => format!(":{}", s),
        P::Not(_) => ":not(*)".to_string(),
        P::Is(_) => ":is(*)".to_string(),
        P::Where(_) => ":where(*)".to_string(),
        P::Has(_) => ":has(*)".to_string(),
        P::NthChild(_) => ":nth-child(*)".to_string(),
        P::NthLastChild(_) => ":nth-last-child(*)".to_string(),
        P::NthChildOf(_, _) => ":nth-child(*)".to_string(),
        P::NthLastChildOf(_, _) => ":nth-last-child(*)".to_string(),
        P::NthOfType(_) => ":nth-of-type(*)".to_string(),
        P::NthLastOfType(_) => ":nth-last-of-type(*)".to_string(),
        P::Lang(_) => ":lang(*)".to_string(),
        P::Dir(d) => format!(":dir({})", d),
    }
}

/// CSS Selector AST → CSS 字符串（pragmatic 序列化：tag/*/id/class/attr/pseudo + 组合器）。
/// 供 `__zw_style_rules`（CSSStyleSheet.cssRules 读，R2808）。
fn css_selector_to_string(sel: &zero_css_parser::Selector) -> String {
    use zero_css_parser::{Combinator, SubclassSelector, TypeSelector};
    let parts = &sel.complex.parts;
    let n = parts.len();
    if n == 0 {
        return String::new();
    }
    let compound = |c: &zero_css_parser::CompoundSelector| -> String {
        let mut s = match &c.type_selector {
            Some(TypeSelector::Tag(t)) => t.clone(),
            Some(TypeSelector::Universal) => "*".to_string(),
            None => String::new(),
        };
        for sub in &c.subclass_selectors {
            match sub {
                SubclassSelector::Id(id) => s.push_str(&format!("#{}", id)),
                SubclassSelector::Class(cls) => s.push_str(&format!(".{}", cls)),
                SubclassSelector::Attribute(a) => {
                    s.push('[');
                    s.push_str(&a.name);
                    s.push_str(&attr_matcher_str(&a.matcher));
                    s.push(']');
                }
                SubclassSelector::PseudoClass(p) => s.push_str(&pseudo_class_str(p)),
                SubclassSelector::PseudoElement(pe) => match pe {
                    zero_css_parser::PseudoElementSelector::Standard(name) => {
                        s.push_str("::");
                        s.push_str(name);
                    }
                },
                SubclassSelector::Nesting => s.push('&'),
            }
        }
        s
    };
    let comb = |c: &Combinator| -> &str {
        match c {
            Combinator::Descendant => " ",
            Combinator::Child => " > ",
            Combinator::NextSibling => " + ",
            Combinator::SubsequentSibling => " ~ ",
        }
    };
    // parts 为 CSS 左→右序（parts[0]=最左，parts[n-1]=subject/目标）。parts[i].1 为 parts[i] 与其右元素
    //（parts[i+1]）间的组合器（末元素 None）。正向遍历，parts[i] 前的组合器取自 parts[i-1].1。
    let mut out = compound(&parts[0].0);
    for i in 1..n {
        match &parts[i - 1].1 {
            Some(c) => out.push_str(comb(c)),
            None => out.push(' '),
        }
        out.push_str(&compound(&parts[i].0));
    }
    out
}

/// CSS Declaration 列表 → `prop: value; prop2: value2`（!important 追加）。供 [`style_rules_wire`]。
fn css_declarations_to_string(decls: &[zero_css_parser::Declaration]) -> String {
    decls
        .iter()
        .map(|d| {
            if d.important {
                format!("{}: {} !important", d.property, d.value)
            } else {
                format!("{}: {}", d.property, d.value)
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// `<style>` 元素解析的规则 → `\x1f`（规则间）/`\x1e`（selectorText/cssText 间）wire 串
///（供 `__zw_style_rules` → shim CSSStyleSheet.cssRules 读，R2808）。仅 StyleRule；@-rule defer。
/// cssText 格式：`selectorText { decls }`（selectorList 逗号空格连）。
pub fn style_rules_wire(html: &str, selector: &str) -> String {
    let doc = parse_html(html);
    let Some(node) = find_by_selector(&doc, selector) else {
        return String::new();
    };
    let text = doc.text_content(node).unwrap_or_default();
    style_rules_text(&text)
}

/// [`style_rules_wire`] 的纯文本版（js-dom M4 R113）——handle-based `<style>`（createElement
/// 后 append）的规则源是 mutation 历史里的 style 文本（无 selector 可查快照），解析同款。
pub fn style_rules_text(text: &str) -> String {
    let ss = zero_css_parser::Parser::parse_stylesheet(text);
    let mut entries: Vec<String> = Vec::new();
    for rule in &ss.rules {
        if let zero_css_parser::Rule::Style(sr) = rule {
            let selector_text = sr
                .selectors
                .iter()
                .map(css_selector_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            let css_text = format!(
                "{} {{ {} }}",
                selector_text,
                css_declarations_to_string(&sr.declarations)
            );
            entries.push(format!("{}\x1e{}", selector_text, css_text));
        }
        // @-rule / keyframes / @media defer（cssRules 读仅 StyleRule）
    }
    entries.join("\x1f")
}
