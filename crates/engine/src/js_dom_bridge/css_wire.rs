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
    query_match_selector_doc_ctx(doc, None, selector)
}

/// S1c memo 变体（worker 热查询回调专用，见 [`UniqSelCtx`]）——定位与物化两级
/// memo 命中与重算逐值等价。
pub fn query_match_selector_doc_cached(doc: &zero_dom::Document, ctx: UniqSelCtx, selector: &str) -> String {
    query_match_selector_doc_ctx(doc, Some(ctx), selector)
}

fn query_match_selector_doc_ctx(doc: &zero_dom::Document, ctx: Option<UniqSelCtx>, selector: &str) -> String {
    let found = match ctx {
        Some(c) => find_by_selector_cached(doc, c, selector),
        None => find_by_selector(doc, selector),
    };
    found
        .and_then(|n| unique_selector_for_node_ctx(doc, ctx, n))
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
        // 45 号 M1：`:host` 族 best-effort 序列化（参数不重建，同上功能伪类口径）。
        P::Host(_) => ":host".to_string(),
        P::HostContext(_) => ":host-context(*)".to_string(),
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
            // R3757：序列化为本地名（wire 快照不含命名空间映射；本地名足够用于
            // 快照比对场景）。`*|`（ns=None 空 name）按 "*"。
            Some(TypeSelector::Namespaced { name, .. }) if !name.is_empty() => name.clone(),
            Some(TypeSelector::Namespaced { .. }) => "*".to_string(),
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
                    // R4262：函数伪元素序列化保真（`::scroll-button(block-end)`）。
                    zero_css_parser::PseudoElementSelector::Functional { name, arg } => {
                        s.push_str("::");
                        s.push_str(name);
                        s.push('(');
                        s.push_str(arg);
                        s.push(')');
                    }
                    // 45 号 M2：`::slotted(<compound>)` best-effort 序列化（compound
                    // 按类型/子类选择器直排，保真口径同上）。
                    zero_css_parser::PseudoElementSelector::Slotted(arg) => {
                        s.push_str("::slotted(");
                        if let Some(ts) = &arg.type_selector {
                            match ts {
                                zero_css_parser::TypeSelector::Universal => s.push('*'),
                                zero_css_parser::TypeSelector::Tag(t) => s.push_str(t),
                                zero_css_parser::TypeSelector::Namespaced { .. } => s.push('*'),
                            }
                        }
                        for sub in &arg.subclass_selectors {
                            match sub {
                                zero_css_parser::SubclassSelector::Id(id) => {
                                    s.push('#');
                                    s.push_str(id);
                                }
                                zero_css_parser::SubclassSelector::Class(c) => {
                                    s.push('.');
                                    s.push_str(c);
                                }
                                _ => {}
                            }
                        }
                        s.push(')');
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

/// CSS Declaration 列表 → `prop: value; prop2: value2;`（!important 追加）。供 [`style_rules_wire`]。
/// s163（html-semantics M2）：每条声明带**尾随分号**（CSSOM serialize a declaration
/// block——每条声明序列化以 `;` 结尾、空格连写，real UA `rule.cssText` 实测
/// `h1 { color: purple; }` 形）。WPT interactions-of-styling-and-scripting
/// styleExists 逐字节比对（`"h1 { color: purple; }" == rule.cssText`）钉该形态；
/// 旧无分号形态使 styleExists 恒 false。下游 `_parseDeclarations` 按 `;` 切分，
/// 尾空段无 `:` 跳过——读取面零波及。
/// https://drafts.csswg.org/cssom-1/#serialize-a-declaration-block
fn css_declarations_to_string(decls: &[zero_css_parser::Declaration]) -> String {
    decls
        .iter()
        .map(|d| {
            if d.important {
                format!("{}: {} !important;", d.property, d.value)
            } else {
                format!("{}: {};", d.property, d.value)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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
        } else if let zero_css_parser::Rule::PositionTry(ptr) = rule {
            // @position-try wire 条目（wptfyi-css at-position-try-cssom）：selectorText
            // 字段 = `@position-try <name>`（真实选择器不可能以 `@` 起头，shim 侧据此
            // 分派 CSSPositionTryRule 物化）；cssText 按 CSSOM 规则序列化形
            //（空声明块 `{ }` 单空格，Chrome 同形）。声明白名单裁剪归 shim 描述符面。
            // https://drafts.csswg.org/css-anchor-position-1/#interfaces
            let name = serialize_css_ident(&ptr.name);
            let decls = css_declarations_to_string(&ptr.declarations);
            let css_text = if decls.is_empty() {
                format!("@position-try {} {{ }}", name)
            } else {
                format!("@position-try {} {{ {} }}", name, decls)
            };
            entries.push(format!("@position-try {}\x1e{}", name, css_text));
        }
        // 其余 @-rule / keyframes / @media defer（cssRules 读仅 StyleRule + position-try）
    }
    entries.join("\x1f")
}

/// CSSOM serialize an identifier（cssom-1 §serialize-an-identifier）——标识符
/// 重新转义。tokenizer 解码过的 ident（`--\{` → `--{`）回序列化时按字符合法性
/// 重新转义：合法 ident 字符直排，其余单字符 `\X` 转义（语料 `--{` → `--\{`）。
/// NULL → U+FFFD（css-syntax-3 §input-preprocessing 同源口径）。
fn serialize_css_ident(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\0' => out.push('\u{FFFD}'),
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => out.push(c),
            c if (c as u32) >= 0xA0 && c != '\u{7F}' => out.push(c),
            _ => {
                out.push('\\');
                out.push(c);
            }
        }
    }
    out
}

#[cfg(test)]
mod position_try_wire_tests {
    use super::style_rules_text;

    #[test]
    /// wire 条目格式：selectorText 字段 = `@position-try <name>`，cssText 声明带尾分号。
    fn wire_position_try_entry_format() {
        let w = style_rules_text("@position-try --pf { top: 1px; }");
        assert_eq!(w, "@position-try --pf\x1e@position-try --pf { top: 1px; }");
    }

    #[test]
    /// 空声明块：cssText 序列化为单空格空块 `{ }`（CSSOM 规则序列化形，Chrome 同形）。
    fn wire_position_try_empty_block() {
        let w = style_rules_text("@position-try --empty { }");
        assert_eq!(w, "@position-try --empty\x1e@position-try --empty { }");
    }

    #[test]
    /// 转义块名重转义：tokenizer 解码 `--\{` → `--{`，wire 序列化按
    /// serialize an identifier 重新转义回 `--\{`。
    fn wire_position_try_escaped_name_reescaped() {
        let w = style_rules_text("@position-try --\\{ { inset: 0; }");
        assert_eq!(w, "@position-try --\\{\x1e@position-try --\\{ { inset: 0; }");
    }

    #[test]
    /// 与 Style 规则混排：wire 条目顺序保持源序，`\x1f` 分隔、`\x1e` 分字段。
    fn wire_position_try_mixed_with_style_rules() {
        let w = style_rules_text("h1 { color: red; } @position-try --p { top: 1px; }");
        assert_eq!(
            w,
            "h1\x1eh1 { color: red; }\x1f@position-try --p\x1e@position-try --p { top: 1px; }"
        );
    }
}
