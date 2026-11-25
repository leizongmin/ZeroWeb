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

/// D3 caret 重锚域专用 fresh 解析（wptfyi-editing d63，2026-11-20）：跳过粘性
/// memo 直接 find + canonical 重算，形态与 [`query_match_selector_doc`] 逐值等价。
///
/// 粘性层（[`sticky_canonical_for_selector`]，d57）服务 wrapper 身份稳定
///（input-events d57–d59 增益域），但 shim D3 世界同步的 caret 重映射臂
///（`_zwCeMapStaleContainer`）以「query 落点 = 当前视图活子树」为前提——
/// 粘性命中返回同家族缓存壳时，重映射落 childNodes 投影滞后的陈旧壳、
/// join walk 的 `bc === el` 上行永不命中（join-white-space 套件 −5,192 单页复现
/// 定谳：zz-probe-d63-jws `scParent` 家族读数，reresolve 映射节点父壳带 setup 代
/// sel vs fresh 域匿名壳）。重映射域必须投影新鲜：shim 经 `__zw_query_match_fresh`
/// 消费本函数；陈旧谓词 `_zwCeCaretAnchorStale` 与页面可见查询仍走粘性通道
/// 零变化（谓词 fresh 会把活粘性壳误判陈旧，ie 全套 −911 二分实证）。
pub fn query_match_selector_doc_fresh(doc: &zero_dom::Document, selector: &str) -> String {
    find_by_selector(doc, selector)
        .map(|n| unique_selector_for_node(doc, n).unwrap_or_default())
        .unwrap_or_default()
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
        .map(|n| sticky_canonical_for_selector(doc, ctx, selector, n))
        .unwrap_or_default()
}

/// d57 刀 C-view（input-events goal，2026-11-19）：`__zw_query_match`（querySelector
/// 返回点）canonical 粘性层——同查询串跨 mutation/materialize 稳定返回**首个物化**的
/// canonical 形。
///
/// 病灶：canonical 形态随文档内容漂移（innerHTML 积 journal 后首键 materialize 使
/// `tag` 最小形失去唯一性 → 重算翻成 `html:nth-child(1) > …` 结构路径）→ shim
/// `_wrapSelector` 按串建 proxy，同节点两次查询返回不同 wrapper → 页面 `===` 身份
/// 断言翻面（corpus harness churn 分支每测重挂、监听器孤儿化——evidence
/// 2026-11-19-d56r §5 root root）。
///
/// 键设计（d57b 勘误）：按**原始查询串**粘性，不按 (html_ptr, NodeId)——快照
/// mutation drain 重建 Arc/重派 NodeId，对象键跨 drain 必失；查询串键跨 drain 存续，
/// 陈旧条目由命中时**自验证**淘汰（memo 经 `find_by_selector` 仍解析回本次找到的
/// 节点才可信，否则重算刷新）。自验证保证服务串恒解析到服务节点（resolution 不变量
/// ——canonical 需唯一性只是该不变量的实现捷径，不作为独立不变式维持；`div` 首匹配
/// 仍是 wrapper 时歧义无害）。等价真浏览器「同一节点同一 JS 包装」的键稳定面。
/// spec：DOM wrapper identity per node（同一 Node 返回同一 JS 对象）。
fn sticky_canonical_for_selector(
    doc: &zero_dom::Document,
    ctx: Option<UniqSelCtx>,
    selector: &str,
    found: zero_dom::NodeId,
) -> String {
    if let Some(memo) = STICKY_QUERY.with(|m| m.borrow().get(selector).cloned()) {
        let hit = match ctx {
            Some(c) => find_by_selector_cached(doc, c, &memo),
            None => find_by_selector(doc, &memo),
        };
        if hit == Some(found) {
            return memo;
        }
    }
    let canonical = unique_selector_for_node_ctx(doc, ctx, found).unwrap_or_default();
    if !canonical.is_empty() {
        STICKY_QUERY.with(|m| {
            let mut map = m.borrow_mut();
            // 封顶整体清空（与 UNIQ_MEMO_CAP 同模式；粘性丢失仅退回逐代重算语义，
            // 自验证使陈旧条目无害，放大上限只损内存不损语义）。
            if map.len() >= STICKY_QUERY_CAP {
                map.clear();
            }
            map.insert(selector.to_string(), canonical.clone());
        });
    }
    canonical
}

/// 粘性 canonical 封顶（见 [`sticky_canonical_for_selector`]）。
const STICKY_QUERY_CAP: usize = 65536;

/// d57 刀 C-view 配套：导航换页清粘性表（`load_html` 文档换代块与 renderer/browser
/// `url_changed` 分支调用）——同查询串的 memo 不得跨文档存活：自验证只保证 memo 串
/// 解析回本次节点（resolution 正确），不保证与新页全新计算同 canonical 形（旧页
/// path-form 条目在新页验证通过但换 wrapper 家族键，下游家族敏感面——selection
/// modify walk 跨非可编辑 span 跳位——行为翻转；wptfyi-editing d57 selce 套件
/// modify-around-non-editable-span 4 键 P2F 实证）。页面内 mutation drain 不走此
/// 钩子，粘性保持（d57a census 翻面消除依赖此点）。
pub fn reset_sticky_query() {
    STICKY_QUERY.with(|m| m.borrow_mut().clear());
}

thread_local! {
    /// 原始查询串 → 首个物化 canonical。页面作用域（导航经 [`reset_sticky_query`]
    /// 清空）；页面内 mutation drain 跨代存续，陈旧条目由命中自验证淘汰（memo 不再
    /// 解析回本次找到的节点即重算）。
    static STICKY_QUERY: std::cell::RefCell<HashMap<String, String>> =
        std::cell::RefCell::new(HashMap::new());
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
pub(crate) fn css_selector_to_string(sel: &zero_css_parser::Selector) -> String {
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
    // k117（css-mixins-1 §cssom）：顶层 `@mixin` canonical 预扫——parse 输出
    // generic `Rule::At` 不保 raw 体形（k113 保留形），canonical 从 raw tokens
    // 单源重建；generic At 臂遇 `@mixin` 按文档序逐个取（计数失配时不发射）。
    let mixin_canons = super::mixin_css::scan_top_level_mixins(text);
    let mut mixin_idx = 0usize;
    let mut entries: Vec<String> = Vec::new();
    for rule in &ss.rules {
        if let zero_css_parser::Rule::Style(sr) = rule {
            // k117：`@apply` 展开产物跳过（css-mixins-1 §cssom——求值域保留，
            // 枚举面单条形：cssRules 只发条目形规则，`ss.cssRules.length==1`
            // mixin-cssom 形）。标记见 [`zero_css_parser::StyleRule::apply_expanded`]。
            if sr.apply_expanded {
                continue;
            }
            let selector_text = sr
                .selectors
                .iter()
                .map(css_selector_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            // k117：`@apply` 原文条目（declaration 载体 property="@apply"）在块
            // 内 → cssText 走多行形（mixin-cssom 期望 template literal 逐字节：
            // `#foo {\n  color: red;\n  @apply --m2 { color: green; }\n}`）；
            // 无 @apply 条目维持单行形（html/ 对端域 styleExists 单行钉不破）。
            // 块形条目无尾分号、语句形带尾分号。
            // https://drafts.csswg.org/css-mixins-1/#cssom
            let has_apply = sr.declarations.iter().any(|d| d.property == "@apply");
            let css_text = if has_apply {
                let mut lines: Vec<String> = Vec::new();
                for d in &sr.declarations {
                    if d.property == "@apply" {
                        if d.value.contains('{') {
                            lines.push(format!("  @apply {}", d.value));
                        } else {
                            lines.push(format!("  @apply {};", d.value));
                        }
                    } else if d.important {
                        lines.push(format!("  {}: {} !important;", d.property, d.value));
                    } else {
                        lines.push(format!("  {}: {};", d.property, d.value));
                    }
                }
                format!("{} {{\n{}\n}}", selector_text, lines.join("\n"))
            } else {
                format!(
                    "{} {{ {} }}",
                    selector_text,
                    css_declarations_to_string(&sr.declarations)
                )
            };
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
        } else if let zero_css_parser::Rule::FontFeatureValues(fvr) = rule {
            // @font-feature-values wire 条目（css-fonts-4 §font-feature-values-cssom；
            // wptfyi-css knife-fv1：map-live/map_iteration cssRules 读面）。
            // selectorText 字段 = `@font-feature-values` 标记（真实选择器不可能以
            // `@` 起头，shim 侧据此分派 CSSFontFeatureValuesRule 物化——host
            // `__zw_parse_ffv_rule` 单源解析/串化，与 insertRule 写面单实现双通道）。
            // cssText = canonical 全文（含族名列表）。
            let css_text = zero_css_parser::serialize::serialize_font_feature_values_css_text(fvr);
            entries.push(format!("@font-feature-values\x1e{}", css_text));
        } else if let zero_css_parser::Rule::Property(pr) = rule {
            // @property wire 条目（css-properties-values-api-1 §at-property-rule +
            // cssom-1 CSSPropertyRule；wptfyi-css 刀 ATP：at-property/-cssom cssRules
            // 读面）——selectorText 字段 = `@property` 标记（真实选择器不可能以 `@`
            // 起头，shim 侧据此分派 CSSPropertyRule 物化——host `__zw_parse_property_rule`
            // 单源解析/串化，与 insertRule 写面单实现双通道，fv1 同型）。cssText =
            // canonical 全文（描述符固定序）。
            // https://drafts.css-houdini.org/css-properties-values-api-1/
            let css_text = zero_css_parser::serialize::serialize_property_rule_css_text(pr);
            entries.push(format!("@property\x1e{}", css_text));
        } else if let zero_css_parser::Rule::Keyframes(kfr) = rule {
            // @keyframes wire 条目（css-animations-1 §at-ruledef-keyframes +
            // cssom-1 CSSKeyframesRule；wptfyi-css 刀 KF：keyframes-name-valid
            // 39F cssRules 读面——此前 keyframes 显式 defer，规则不入 wire 即
            // sheet.cssRules 恒缺位，valid 页全页空 Rao / invalid 页 vacuous 过）。
            // selectorText 字段 = `@keyframes` 标记（真实选择器不可能以 `@` 起头，
            // shim 侧据此分派 CSSKeyframesRule 物化——host `__zw_parse_keyframes_rule`
            // 单源解析/串化，与 insertRule 写面单实现双通道，property-rule 同型）。
            // cssText = canonical 全文（`@keyframes <name> { <sel> { decls } }`）。
            // https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes
            let css_text = zero_css_parser::serialize::serialize_keyframes_css_text(kfr);
            entries.push(format!("@keyframes\x1e{}", css_text));
        } else if let zero_css_parser::Rule::FontFace(ffr) = rule {
            // @font-face wire 条目（css-fonts-4 §font-face-rule + cssom-1
            // CSSFontFaceRule；wptfyi-css 刀 FONT-1：at-font-face-descriptors
            // 88F wholesale cssRules 读面——此前 @font-face 随 @-rule defer，
            // sheet.cssRules[0] undefined → `.style` 全页崩）。selectorText 字段
            // = `@font-face` 标记（真实选择器不可能以 `@` 起头，shim 侧据此
            // 物化 CSSFontFaceRule——descriptor declaration face 文法核在 shim
            // JS 侧单点，无宿主 parse 回调）。cssText 合成 canonical（family
            // 按 cssom-1 ident/串形回序列化；src 折叠字段已去 url() 包裹，
            // 回拼 url("…")；weight/style/stretch 等其余描述符折叠值非原文，
            // 不发射——CSSOM descriptor 面以 shim 存储为准）。
            // https://drafts.csswg.org/css-fonts-4/#font-face-rule
            let fam_ident = serialize_css_ident(&ffr.family);
            let fam_css = if fam_ident == ffr.family {
                fam_ident
            } else {
                format!("\"{}\"", ffr.family.replace('\\', "\\\\").replace('"', "\\\""))
            };
            let mut css_text = format!("@font-face {{ font-family: {fam_css};");
            // FONT-2（wptfyi-css 2026-11-25）：weight/style/stretch/
            // font-feature-settings 描述符发射——CSSFontFaceRule.style 读回门
            //（random-in-descriptors fontWeight "300"＋sibling-function-descriptors
            // fontWeight/fontFeatureSettings；此前折叠值不发射＝shim store 空）。
            // 值形 canonical：weight u16、style normal/italic/oblique [deg]、
            // stretch %（parse_font_stretch 存百分数值）、features `"tag" v` 列表。
            if let Some(w) = ffr.weight {
                css_text.push_str(&format!(" font-weight: {w};"));
            }
            if let Some(st) = &ffr.style {
                let text = match st {
                    zero_css_parser::values::types::FontStyleValue::Normal => "normal".to_string(),
                    zero_css_parser::values::types::FontStyleValue::Italic => "italic".to_string(),
                    zero_css_parser::values::types::FontStyleValue::Oblique(angle) => match angle {
                        Some(a) => format!("oblique {}deg", *a as i64),
                        None => "oblique".to_string(),
                    },
                };
                css_text.push_str(&format!(" font-style: {text};"));
            }
            if let Some(s) = ffr.stretch {
                // 百分数值 canonical（30.0 → "30%"；87.5 保 "87.5%"）。
                let mut text = format!("{s}");
                if text.ends_with(".0") {
                    text.truncate(text.len() - 2);
                }
                css_text.push_str(&format!(" font-stretch: {text}%;"));
            }
            if let zero_css_parser::values::FontFeatureSettingsValue::Features(items) = &ffr.feature_settings {
                let rendered: Vec<String> = items
                    .iter()
                    .map(|item| {
                        let tag = String::from_utf8_lossy(&item.tag).to_string();
                        format!("\"{tag}\" {}", item.value)
                    })
                    .collect();
                if !rendered.is_empty() {
                    css_text.push_str(&format!(" font-feature-settings: {};", rendered.join(", ")));
                }
            }
            if !ffr.sources.is_empty() {
                css_text.push_str(" src:");
                for (i, s) in ffr.sources.iter().enumerate() {
                    if i > 0 {
                        css_text.push(',');
                    }
                    if s.starts_with("local(") {
                        // css-fonts-4 §src-desc local 源形原样回拼——sources 以
                        // `local(` 前缀判别（at_rules 提取侧约定，裸串=url 载荷）。
                        css_text.push_str(&format!(" {s}"));
                    } else {
                        css_text.push_str(&format!(" url(\"{}\")", s.replace('\\', "\\\\").replace('"', "\\\"")));
                    }
                }
                css_text.push(';');
            }
            css_text.push_str(" }");
            entries.push(format!("@font-face\x1e{}", css_text));
        } else if let zero_css_parser::Rule::Page(pr) = rule {
            // @page wire 条目（cssom-1 CSSPageRule cssRules 读面；wptfyi-css
            // 刀 FONT-2：sibling-function-descriptors @page ×2 style.margin
            // 读回门——此前 Rule::Page 随 @-rule defer 恒不入 cssRules）。
            // selectorText 字段 = `@page` 标记（真实选择器不可能以 `@` 起头）。
            // cssText canonical 全文（serialize_page_rule_css_text：缺省描述符
            // 省略＋margin 四元简写回拼——invalid 描述符解析期已弃行）。
            // https://drafts.csswg.org/css-page-3/#at-page-rule
            let css_text = zero_css_parser::serialize::serialize_page_rule_css_text(pr);
            entries.push(format!("@page\x1e{}", css_text));
        } else if let zero_css_parser::Rule::CounterStyle(csr) = rule {
            // @counter-style wire 条目（cssom-1 CSSCounterStyleRule cssRules
            // 读面；刀 FONT-2：sibling-function-descriptors @counter-style ×2
            // 七接口读回门——此前随 @-rule defer）。selectorText 字段 =
            // `@counter-style` 标记。cssText canonical 全文（非初始值描述符
            // 发射——invalid 弃行解析期已达成）。
            // https://drafts.csswg.org/css-counter-styles-3/#the-counter-style-rule
            let css_text = zero_css_parser::serialize::serialize_counter_style_css_text(csr);
            entries.push(format!("@counter-style\x1e{}", css_text));
        } else if let zero_css_parser::Rule::FontPaletteValues(fpv) = rule {
            // @font-palette-values wire 条目（css-fonts-4 §font-palette-values
            // + cssom-1 CSSFontPaletteValuesRule cssRules 读面；刀 FONT-2：
            // random-in-descriptors basePalette ×2＋sibling-function-descriptors
            // overrideColors 读回门——此前 generic at-rule 臂块体声明全丢）。
            // selectorText 字段 = `@font-palette-values` 标记。cssText canonical
            // 全文（首写胜描述符——invalid 弃行解析期已达成）。
            // https://drafts.csswg.org/css-fonts-4/#font-palette-values-rule
            let css_text = zero_css_parser::serialize::serialize_font_palette_values_css_text(fpv);
            entries.push(format!("@font-palette-values\x1e{}", css_text));
        } else if let zero_css_parser::Rule::At(at) = rule {
            // @media wire 条目（cssom-1 CSSMediaRule cssRules 读面；wptfyi-css
            // 刀 ζ：random-in-media parseable 键——insertRule("@media q{}") 后
            // cssRules 恒缺位（@-rule defer）→ query_is_css_parseable 双门挂）。
            // prelude 原文保形（generic at-rule 消费不校验 prelude，random 函数形
            // token 嵌套深度跟踪已备）＝mediaText 逐字读回≠"not all"；cascade
            // 求值面不变（parse_media_query 拒函数形值 → 规则不应用，parseable
            // ≠ known——语料 unknown 键持 Pass 红线）。其余 @-rule（supports/
            // layer/import/page…）维持 defer。
            // https://drafts.csswg.org/cssom-1/#cssmediarule
            if at.name.eq_ignore_ascii_case("media") {
                let body_css = nested_style_rules_css_text_from_body(&at.body);
                let css_text = if body_css.is_empty() {
                    format!("@media {} {{ }}", at.prelude)
                } else {
                    format!("@media {} {{ {} }}", at.prelude, body_css)
                };
                entries.push(format!("@media\x1e{}", css_text));
            } else if at.name.eq_ignore_ascii_case("mixin") {
                // @mixin wire 条目（css-mixins-1 §cssom CSSMixinRule cssRules 读
                // 面；wptfyi-css k117：mixin-cssom 8F＋mixin-parsing 17F 靶面）。
                // selectorText 字段 = `@mixin` 标记（真实选择器不可能以 `@` 起头，
                // shim 侧据此分派 CSSMixinRule 物化——host `__zw_parse_mixin_rule`
                // 单源解析/串化，与 insertRule 写面单实现双通道，function-rule
                // 同型）。cssText = canonical 全文（预扫 raw token 重建——generic
                // At 路径 AST 不保体形；文档序计数取，失配不发射）。
                // https://drafts.csswg.org/css-mixins-1/#cssom
                if let Some(css_text) = mixin_canons.get(mixin_idx) {
                    entries.push(format!("@mixin\x1e{}", css_text));
                }
                mixin_idx += 1;
            }
        } else if let zero_css_parser::Rule::Container(cr) = rule {
            // @container wire 条目（css-conditional-5 §container-queries
            // CSSContainerRule cssRules 读面；wptfyi-css 刀 ζ：
            // random-in-container-query ×2＋random-item-in-container-query ×1——
            // 规则此前随 @-rule defer 恒不入 cssRules（探针 D6 锚：纯
            // `(width: 30px)` 也 rules=0＝表面化共性门非 random 特异）→
            // test_cq_condition_valid rules.length==2 门挂）。条件按 AST 存储
            // 形直排（colon/op/range 三形 round-trip；style()/or 复合形廉价
            // 子切片保形——求值面不变：函数形值 px 解析失败恒 false＝unknown，
            // style() 域含 random 维持不匹配红线）。嵌套 style 规则体同 @media。
            // https://drafts.csswg.org/css-conditional-5/#container-queries
            let name_part = cr
                .name
                .as_ref()
                .map(|n| format!("{} ", serialize_css_ident(n)))
                .unwrap_or_default();
            let mut cond_texts = vec![container_condition_css_text(&cr.condition)];
            for extra in &cr.extra_conditions {
                cond_texts.push(container_condition_css_text(extra));
            }
            let body_css = nested_style_rules_css_text(&cr.rules);
            let cond_join = cond_texts.join(", ");
            let css_text = if body_css.is_empty() {
                format!("@container {}{} {{ }}", name_part, cond_join)
            } else {
                format!("@container {}{} {{ {} }}", name_part, cond_join, body_css)
            };
            entries.push(format!("@container\x1e{}", css_text));
        }
        // 其余 @-rule defer（cssRules 读 = StyleRule + position-try +
        // font-feature-values + property + keyframes + font-face + media +
        // container）
    }
    entries.join("\x1f")
}

/// @media 块体的嵌套 style 规则文本（AtRuleBody::Block → 规则列表委托
/// [`nested_style_rules_css_text`]；Statement 体（畸形）→ 空串）。
fn nested_style_rules_css_text_from_body(body: &zero_css_parser::AtRuleBody) -> String {
    match body {
        zero_css_parser::AtRuleBody::Block(rules) => nested_style_rules_css_text(rules),
        zero_css_parser::AtRuleBody::Statement => String::new(),
    }
}

/// 嵌套 style 规则序列化（@media/@container cssText 读面）——StyleRule 用与
/// 顶层同款 selector/declarations 序列化拼 ` { … }`（多规则空白连接）；非 style
/// 嵌套规则维持 defer（与顶层一致）。
fn nested_style_rules_css_text(rules: &[zero_css_parser::Rule]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for rule in rules {
        if let zero_css_parser::Rule::Style(sr) = rule {
            let selector_text = sr
                .selectors
                .iter()
                .map(css_selector_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!(
                "{} {{ {} }}",
                selector_text,
                css_declarations_to_string(&sr.declarations)
            ));
        }
    }
    parts.join(" ")
}

/// ContainerCondition → 条件文本（CSSContainerRule containerQuery 读面）。
/// Size 裸形括号回卷（括号是条件语法，storage 丢括号——钉测实证）；InlineSize
/// 函数形直排；Style 按 `style(...)`／`not style(...)` 形（property/value 廉价
/// 子切片直排）。
fn container_condition_css_text(cond: &zero_css_parser::ContainerCondition) -> String {
    match cond {
        zero_css_parser::ContainerCondition::Size(sc) => {
            format!("({})", container_size_condition_css_text(sc))
        }
        zero_css_parser::ContainerCondition::InlineSize(sc) => {
            format!("inline-size({})", container_size_condition_css_text(sc))
        }
        zero_css_parser::ContainerCondition::Style {
            property,
            value,
            negated,
        } => {
            let inner = match value {
                Some(v) => format!("{}: {}", property, v),
                None => property.clone(),
            };
            if *negated {
                format!("not style({})", inner)
            } else {
                format!("style({})", inner)
            }
        }
        // `not (...)` 尺寸条件否定（css-contain-3）：内层条件文本（Size 形自带
        // 括号）外卷 `(not …)`——与括号包裹解析形恒等回卷（round-trip 钉测）。
        zero_css_parser::ContainerCondition::Not(inner) => {
            format!("(not {})", container_condition_css_text(inner))
        }
    }
}

/// ContainerSizeCondition → 条件文本：range 形 `{min} <= {feature} <= {max}`；
/// op 形 `{feature} {op} {value}`；colon 形 `{feature}: {value}`（storage 形
/// 直排＝random/not garbage-parse 形恒等回卷——钉测三形实证）。
fn container_size_condition_css_text(sc: &zero_css_parser::ContainerSizeCondition) -> String {
    if let (Some(min), Some(max)) = (&sc.range_min, &sc.range_max) {
        return format!("{} <= {} <= {}", min, sc.feature, max);
    }
    match &sc.operator {
        Some(op) => format!("{} {} {}", sc.feature, op, sc.value),
        None => format!("{}: {}", sc.feature, sc.value),
    }
}

/// CSSOM serialize an identifier（cssom-1 §serialize-an-identifier）——标识符
/// 重新转义。tokenizer 解码过的 ident（`--\{` → `--{`）回序列化时按字符合法性
/// 重新转义：合法 ident 字符直排，其余单字符 `\X` 转义（语料 `--{` → `--\{`）。
/// NULL → U+FFFD（css-syntax-3 §input-preprocessing 同源口径）。
pub(crate) fn serialize_css_ident(s: &str) -> String {
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

    #[test]
    /// @font-face 入 wire（css-fonts-4 §font-face-rule + cssom-1 CSSFontFaceRule）：
    /// 此前 @font-face 随 @-rule defer → `sheet.cssRules` 读面全缺位（wptfyi-css
    /// 刀 FONT-1：at-font-face-descriptors 88F wholesale `.style` 崩形）。
    /// selectorText 字段 = `@font-face` 标记，shim 侧据以物化 CSSFontFaceRule。
    fn wire_font_face_rule_entry() {
        let w = style_rules_text("@font-face { font-family: F; src: url(f.woff); }");
        assert_eq!(w, "@font-face\x1e@font-face { font-family: F; src: url(\"f.woff\"); }");
    }

    #[test]
    /// local() 源 face 不丢弃（css-fonts-4 §src-desc `local(<family-name>)`
    /// 合法源形）——consume_font_face_rule 此前只认 url() 载荷，local-only
    /// face 整条丢弃（FONT-1b：at-font-face-descriptors 静态 `<style>` 即
    /// local-only src，88F wholesale 不翻的根因）。sources 以 `local(…)`
    /// 形入列（`local(` 前缀判别约定），wire cssText 回拼；载荷字符串经
    /// 声明 tokenizer 规范为 canonical 双引号形。
    fn wire_font_face_local_source_entry() {
        let w = style_rules_text("@font-face { font-family: Test; src: local('Courier New'), local('Courier'); }");
        assert_eq!(
            w,
            "@font-face\x1e@font-face { font-family: Test; src: local(\"Courier New\"), local(\"Courier\"); }"
        );
    }
}

// ── 刀 ζ 钉测（kc5r106：MQ/CQ 条件 random/random-item 合法 unknown 形
// CSSOM 表面化 wire 读面）─────────────────────────────────────────────
#[cfg(test)]
mod cqmq_wire_tests {
    use super::style_rules_text;

    #[test]
    /// @media 入 wire（cssom-1 CSSMediaRule）：prelude 原文保形＋嵌套 style
    /// 规则体直排。此前 @media 随 @-rule defer → `sheet.cssRules` 读面恒缺位。
    fn wire_media_rule_entry() {
        let w = style_rules_text("@media (width > 30px) { h1 { color: red; } }");
        assert_eq!(w, "@media\x1e@media (width > 30px) { h1 { color: red; } }");
    }

    #[test]
    /// @media 空体：`{ }` 空块形（与 @position-try 空声明块同 canonical）。
    fn wire_media_empty_body() {
        let w = style_rules_text("@media screen { }");
        assert_eq!(w, "@media\x1e@media screen { }");
    }

    #[test]
    /// @container 入 wire（css-conditional-5 §container-queries）：name ＋
    /// colon 形条件 round-trip＋嵌套体（custom property 声明保形）。
    fn wire_container_rule_entry() {
        let w = style_rules_text("@container name (width: 30px) { main { --match: true; } }");
        assert_eq!(
            w,
            "@container\x1e@container name (width: 30px) { main { --match: true; } }"
        );
    }

    #[test]
    /// @container random 条件 round-trip（刀 ζ F2P 关键形）：函数形值位
    /// String 保形（parse_size_condition 三臂全 String），序列化直排恒等。
    /// 求值面不变（px 解析失败恒 false＝unknown）。
    fn wire_container_random_condition_roundtrip() {
        let w = style_rules_text("@container name (width: random(--foo, 10px, 30px)) {}");
        assert_eq!(
            w,
            "@container\x1e@container name (width: random(--foo, 10px, 30px)) { }"
        );
    }

    #[test]
    /// @container `or` 复合形（test_cq_condition_valid 第二规则形）：解析臂
    /// `or` 关键字入 extra_conditions（OR 语义同逗号形）；两段 garbage-parse
    /// String 保形＋括号回卷恒等（首段双括号形、次段 not 嵌套形原样）；
    /// `or` → `,` 规范化（AST 无 or/comma 区分，canonical OR 序列化）——
    /// 规则级存活（rules.length==2 门）不依赖条件语义。
    fn wire_container_compound_or_condition_roundtrip() {
        let src = "@container name ((width: random(--foo, 10px, 30px))) or (not ((width: random(--foo, 10px, 30px)))) { main { --match: true; } }";
        let w = style_rules_text(src);
        assert_eq!(
            w,
            "@container\x1e@container name ((width: random(--foo, 10px, 30px))), (not ((width: random(--foo, 10px, 30px)))) { main { --match: true; } }"
        );
    }

    #[test]
    /// Style 与 @media 混排：wire 条目源序保持，`\x1f` 分隔、`\x1e` 分字段。
    fn wire_media_mixed_with_style_rules() {
        let w = style_rules_text("h1 { color: red; } @media screen { p { color: blue; } }");
        assert_eq!(
            w,
            "h1\x1eh1 { color: red; }\x1f@media\x1e@media screen { p { color: blue; } }"
        );
    }
}

// ── FONT-2 钉测（kc5r107：@font/@page/@counter-style 描述符 CSSOM 保形族
// wire 读面）──────────────────────────────────────────────────────────────
#[cfg(test)]
mod font2_wire_tests {
    use super::style_rules_text;

    #[test]
    /// @page 入 wire（cssom-1 CSSPageRule）：margin 四元简写回拼（全等 → 单值）。
    /// 此前 Rule::Page 随 @-rule defer → cssRules 读面恒缺位。
    fn wire_page_rule_entry() {
        let css = "@page { margin: 100px; margin: calc(0px * sibling-index()); }";
        assert_eq!(style_rules_text(css), "@page\x1e@page { margin: 100px; }");
    }

    #[test]
    /// @counter-style 入 wire（cssom-1 CSSCounterStyleRule）：invalid 描述符
    /// 弃行（首写胜）＋七接口 canonical 回拼——sibling-function-descriptors
    /// 断言值逐字（system "fixed 1"/negative "--pass"/prefix/suffix "--pass"/
    /// range "1 infinite"/pad "1 --pass"/symbols "--pass"）。
    fn wire_counter_style_rule_entry() {
        let css = "@counter-style --foo { system: fixed 1; system: fixed calc(max(sibling-index(), 2)); negative: --pass; negative: linear-gradient(red calc(20px * sibling-index()), pink); prefix: --pass; suffix: --pass; range: 1 infinite; range: calc(max(sibling-index(), 2)) infinite; pad: 1 --pass; pad: calc(max(sibling-index(), 2)) --fail; symbols: --pass; }";
        assert_eq!(
            style_rules_text(css),
            "@counter-style\x1e@counter-style --foo { system: fixed 1; symbols: --pass; prefix: --pass; suffix: --pass; range: 1 infinite; negative: --pass; pad: 1 --pass; }"
        );
    }

    #[test]
    /// @font-palette-values 入 wire（css-fonts-4 CSSFontPaletteValuesRule）：
    /// 首写胜（base-palette: 3 保留、random(10, 30) 弃行）＋override-colors
    /// 保形——random-in-descriptors basePalette "3" 读回门。
    fn wire_palette_rule_entry() {
        let css =
            "@font-palette-values --palette { font-family: my-font; base-palette: 3; base-palette: random(10, 30); }";
        assert_eq!(
            style_rules_text(css),
            "@font-palette-values\x1e@font-palette-values --palette { font-family: my-font; base-palette: 3; }"
        );
    }

    #[test]
    /// CSSFontFaceRule.style 读回面：weight/style/stretch/font-feature-settings
    /// 描述符发射（此前折叠值不发射＝shim store 空）——random-in-descriptors
    /// fontWeight "300"＋sibling-function-descriptors fontFeatureSettings
    /// `"vert" 2` 读回门。
    fn wire_font_face_descriptor_readback() {
        let css = "@font-face { font-family: my-font; font-weight: 300; font-weight: random(500, 900); font-feature-settings: \"vert\" 2; font-feature-settings: \"vert\" calc(max(sibling-index(), 1)); }";
        assert_eq!(
            style_rules_text(css),
            "@font-face\x1e@font-face { font-family: my-font; font-weight: 300; font-feature-settings: \"vert\" 2; }"
        );
    }

    #[test]
    /// 无 src 的 @font-face 保留（css-fonts-4：CSSOM 枚举面只要求 font-family
    /// 有效——此前 sources.is_empty() 整条弃＝rules[1] undefined 根因）。
    fn wire_font_face_no_src_retained() {
        let css = "@font-face { font-family: my-font; font-stretch: 30%; font-stretch: random(--foo element-scoped, 0%, 10%); }";
        assert_eq!(
            style_rules_text(css),
            "@font-face\x1e@font-face { font-family: my-font; font-stretch: 30%; }"
        );
    }
}

#[cfg(test)]
mod k117_wire_tests {
    use super::style_rules_text;

    #[test]
    /// @mixin 入 wire（css-mixins-1 §cssom；wptfyi-css k117）：selectorText 字段
    /// = `@mixin` 标记，cssText = canonical 多行形（@result 平级行＋嵌套块单行
    /// ＋空参剥 ()）——mixin-cssom 'serialization of @mixin' 形。
    fn wire_mixin_rule_entry() {
        let css = "@mixin --m1() { @result { color: green; & { --foo: bar; } } }";
        assert_eq!(
            style_rules_text(css),
            "@mixin\x1e@mixin --m1 {\n  @result {\n  color: green;\n  & { --foo: bar; }\n}\n}"
        );
    }

    #[test]
    /// 含 @apply 条目的 StyleRule cssText 多行分形（css-mixins-1 §cssom）：
    /// 语句形带尾分号、块形无尾分号——mixin-cssom 'serialization of rule with
    /// @apply' 形；无 @apply 条目维持单行形（html/ 对端域 styleExists 钉）。
    fn wire_style_rule_apply_multiline_form() {
        let w = style_rules_text("#foo { @apply --m1; }");
        assert_eq!(w, "#foo\x1e#foo {\n  @apply --m1;\n}");
        let w2 = style_rules_text("#foo { color: red; @apply --m2 { color: green; } }");
        assert_eq!(w2, "#foo\x1e#foo {\n  color: red;\n  @apply --m2 { color: green; }\n}");
        // 无 @apply 条目单行形不变。
        let w3 = style_rules_text("h1 { color: purple; }");
        assert_eq!(w3, "h1\x1eh1 { color: purple; }");
    }

    #[test]
    /// @apply 空体 contents 块 canonical `{ }` 单空格（mixin-cssom 'empty
    /// contents block' 形）＋参数 canonical 剥壳（type() → `<length>`）。
    fn wire_mixin_params_and_empty_contents() {
        let w = style_rules_text(
            "@mixin --m3(--arg type(<length>): 1em, --other-arg) { @result { margin-left: var(--arg); } }",
        );
        assert_eq!(
            w,
            "@mixin\x1e@mixin --m3(--arg <length>: 1em, --other-arg) {\n  @result {\n  margin-left: var(--arg);\n}\n}"
        );
        let w2 = style_rules_text("@mixin --m2() { @result { @contents } }");
        assert_eq!(w2, "@mixin\x1e@mixin --m2 {\n  @result {\n  @contents;\n}\n}");
        let w3 = style_rules_text("#foo { @apply --m5 { } }");
        assert_eq!(w3, "#foo\x1e#foo {\n  @apply --m5 { }\n}");
    }
}

// 刀 ζ face 指纹（kc5r106）：#[used] 防 DCE 剥 bin 指纹（constants 不物化课）——
// gen bin 探针判代锚。
#[used]
static KC5R106_CQMQ_FACE: [u8; 22] = *b"ZW-KC5R106-CQMQ-FACE!!";

// 刀 FONT-2 face 指纹（kc5r107）：同上判代锚——@font/@page/@counter-style
// 描述符 CSSOM 保形族。
#[used]
static KC5R107_FONT2_FACE: [u8; 22] = *b"ZW-KC5R107-FONT2-FACE!";

// 刀 k117 face 指纹（kc5r117）：@mixin CSSOM 物化族判代锚——wire @mixin 臂 +
// @apply 多行分形 + shim CSSMixinRule/CSSResultRule 物化 + replaceSync 深度
// 感知切分。#[used] 防 DCE 剥 bin 指纹（constants 不物化课）。
#[used]
static KC5R117_MX_CSSOM_FACE: [u8; 24] = *b"ZW-KC5R117-MX-CSSOM-FACE";

// 刀 k118 face 指纹（kc5r118）：`<dashed-function>` 调用实参文法写面族判代
// 锚——dashed_function_value_valid 单源 + consume_declaration 声明门 +
// style-system validate_decl_value pre-arm + shim hub 臂三通道接线。
// #[used] 防 DCE 剥 bin 指纹（constants 不物化课）。
#[used]
static KC5R118_DFN_VALID_FACE: [u8; 25] = *b"ZW-KC5R118-DFN-VALID-FACE";

// 刀 k119 face 指纹（kc5r119）：dashed-function 求值端会话作用域重构族判代
// 锚——调用方作用域穿线＋绑定期实参解析/块解包＋无效调用 Invalid（IACVT）
// ＋CSS 宽关键字局部/缺省语义＋SCOPE_IACVT 遮蔽标记截臂。
// #[used] 防 DCE 剥 bin 指纹（constants 不物化课）。
#[used]
static KC5R119_FN_EVAL_SCOPE_FACE: [u8; 24] = *b"ZW-KC5R119-FN-EVAL-SCOPE";

// 刀 k120 face 指纹（kc5r120）：dashed-function typed 求值域族判代锚——
// 类型强制层（<length> calc 化简 px canonical＋<angle> 归一 deg＋<time>
// 归一 s＋<number>/<color> 校验透传）＋typed 实参失败≡IACVT（缺省/遮蔽
// 标记）＋typed 局部失败 IACVT 遮蔽＋块产物顶层块门。
#[used]
static KC5R120_TYPED_EVAL_FACE: [u8; 26] = *b"ZW-KC5R120-TYPED-EVAL-FACE";

// 刀 k121 face 指纹（kc5r121）：dashed-function 元素级 CSS 宽关键字落地族
// 判代锚——4.755 站产物整值关键字消费元素落地（initial/revert=absent 兑
// 回退＋inherit/unset=父计算值代入）＋revert-layer/revert-rule 字面持
// （回溯架构 FIXME）。
#[used]
static KC5R121_ELEM_KW_FACE: [u8; 23] = *b"ZW-KC5R121-ELEM-KW-FACE";

// 刀 k122 face 指纹（kc5r122）：dashed-function style() 冒号等值类型感知
// 比较族判代锚——合并型别表（@function 参数型别∪@property 注册语法，
// params 优先）穿线会话站＋IfEvalCtx type_lookup 双侧 canonical 化比较
// （typed_text_eq——解析失败回退裸文本零 P2F）＋元素站 type_lookup=None。
#[used]
static KC5R122_TYPED_QUERY_FACE: [u8; 27] = *b"ZW-KC5R122-TYPED-QUERY-FACE";

// 刀 k123 face 指纹（kc5r123）：inherit() 记法父栈帧替换族判代锚——
// 会话内 inherit(--name[, fallback]) 取调用方已解析作用域（局部不遮蔽
// 父帧读取）＋SCOPE_IACVT 遮蔽/缺席＝guaranteed-invalid（fallback 臂/
// 无 fallback IACVT）＋result 站 Err → 调用 invalid＋非 -- 首参字面持。
#[used]
static KC5R123_INHERIT_FRAME_FACE: [u8; 29] = *b"ZW-KC5R123-INHERIT-FRAME-FACE";

// 刀 k124 face 指纹（kc5r124）：@function 体条件规则组族判代锚——
// FunctionRule body 条目树（Decl/Conditional）＋求值端条件三臂展平
//（@supports 静态支持判定/@media 视口语境/@container 容器链快照——
// 条件满足组按源序原地展开，§4 last-wins 语义保持）。
#[used]
static KC5R124_FN_COND_FACE: [u8; 23] = *b"ZW-KC5R124-FN-COND-FACE";

// 刀 k125 face 指纹（kc5r125）：标准属性值域 `<dashed-function>` 替换族
// 判代锚——级联 drop-if-invalid 门按替换函数同 var() 口径放行＋简写形
// 并入 pending 哨兵域（4.6 复燃幂等 → 4.755 站代入 → 4.85 二趟展开落长
// 属性）；未注册名 guaranteed-invalid → IACVT 初始值。
#[used]
static KC5R125_STDLONG_FACE: [u8; 23] = *b"ZW-KC5R125-STDLONG-FACE";

// 刀 k126 face 指纹（kc5r126）：函数求值端 attr() 替换＋taint 闭环族判代
// 锚——函数体三域（参数缺省/局部声明/result）attr() 文本级替换
//（css-values-5 §attr-notations 替换序 var→attr）＋产物 url 污染随
// DashOutcome::Value 通道逐层合并上传，顶层消费站对取 URL 属性裁决
// IACVT（§attr-security，4.75 站同口径）。
#[used]
static KC5R126_FN_ATTR_FACE: [u8; 22] = *b"ZW-KC5R126-FNATTR-FACE";

// 刀 k127 face 指纹（kc5r127）：var()/attr() 不动点迭代替换族判代锚——
// 函数体局部声明/result 域 var↔attr 交替链迭代至不动点（css-mixins-1
// §locally-substitute-a-var「var() in attribute value」判据族）＋attr()
// Syntax 型别属性值含替换词延迟文法裁决（css-values-5 §8.7.1 step 6 先
// 替后查）＋交替不收敛按环处置 IACVT（SUBSTITUTE_FIXPOINT_LIMIT 上限）。
#[used]
static KC5R127_FNATTR_FIXPT_FACE: [u8; 21] = *b"ZW-KC5R127-FIXPT-FACE";

// k128 face 指纹（function-layer 刀）：@function 级联层 rank（css-parser
// FunctionRule.layer 携 collect_layer＋匿名层哨兵占位兑现 k117 FIXME）＋
// collect_custom_functions 递归 @layer 块＋rank 裁决（style-system
// custom_function.rs——css-mixins-1 §2.1.1 stronger cascade layer wins）。
#[used]
static KC5R128_FN_LAYER_FACE: [u8; 23] = *b"ZW-KC5R128-FNLAYER-FACE";
