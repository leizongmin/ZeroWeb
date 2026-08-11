//! 基础选择器解析与匹配。
//!
//! 支持的简单选择器格式：
//! - 标签名：`"div"`, `"span"`
//! - ID 选择器：`"#myid"`
//! - 类选择器：`".myclass"`
//! - 属性选择器：`"[attr]"`, `"[attr=value]"`, `"[attr~=value]"`

use crate::node::ElementData;

/// 简单选择器（仅支持单层选择器，不支持组合器）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleSelector {
    /// 标签名匹配（大小写不敏感）。
    pub tag: Option<String>,
    /// ID 匹配。
    pub id: Option<String>,
    /// 类名匹配列表（支持多个类选择器，如 `.a.b`）。
    pub classes: Vec<String>,
    /// 属性匹配。
    pub attribute: Option<AttributeSelector>,
    /// 伪类列表（`:nth-child(n)` / `:first-child` 等，AND 语义；需 sibling 位置上下文，由
    /// [`SimpleSelector::matches_full`] 配合 [`ElementPosition`] 评估，`matches` 自身不检查）。
    pub pseudos: Vec<PseudoClass>,
}

/// 伪类（需 sibling/树位置上下文，非元素自身属性）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoClass {
    /// `:nth-child(an+b)`——在所有元素兄弟中第 n 个。
    NthChild(Nth),
    /// `:nth-of-type(an+b)`——在同 tag 元素兄弟中第 n 个。
    NthOfType(Nth),
    /// `:nth-last-child(an+b)`——在所有元素兄弟中倒数第 n 个。
    NthLastChild(Nth),
    /// `:nth-last-of-type(an+b)`——在同 tag 元素兄弟中倒数第 n 个。
    NthLastOfType(Nth),
    /// `:first-child`——首个元素兄弟。
    FirstChild,
    /// `:last-child`——末个元素兄弟。
    LastChild,
    /// `:only-child`——唯一元素兄弟。
    OnlyChild,
    /// `:first-of-type`——同 tag 首个。
    FirstOfType,
    /// `:last-of-type`——同 tag 末个。
    LastOfType,
    /// `:only-of-type`——同 tag 唯一。
    OnlyOfType,
    /// `:root`——文档根元素（`<html>`）。
    Root,
    /// `:empty`——无子节点（含文本；spec：无 element/text 子，注释允许，本实现简化为无任何子）。
    Empty,
    /// `:not(simple)`——否定伪类，匹配**不**满足内嵌简单选择器的元素（CSS3 语义：内嵌仅简单选择器，
    /// 无组合器）。内嵌可为含伪类的简单选择器（如 `:not(:first-child)`）。
    Not(SimpleSelector),
    /// `:is(s1, s2, …)` / `:where(…)`——选择器列表，匹配满足**任一**内嵌简单选择器的元素。
    /// `:where` 语义同 `:is`（区别仅在特异性，本引擎无特异性概念，故共用）。
    Is(Vec<SimpleSelector>),
    /// `:has(inner)` / `:has(> inner)`——关系伪类，匹配拥有匹配 `inner` 的后代（默认）或直接子
    /// （`> ` 前缀）的元素。`inner` 为选择器字符串（可为含组合器的链）。需 Document 子树求值——
    /// [`SimpleSelector::matches_full`] 对 `Has` 延后返 `true`，由
    /// `Document::element_matches_selector` 实际评估。
    Has {
        /// 内嵌选择器字符串（可为含组合器的链，如 `.a .b`）。由 Document 子树求值。
        inner: String,
        /// `true` = `:has(> inner)` 直接子作用域；`false` = `:has(inner)` 后代作用域。
        child_scope: bool,
    },
    /// `:checked`——选中态表单元素（`<input type=checkbox|radio>` 带 `checked` 属性，或
    /// `<option>` 带 `selected` 属性）。纯元素属性求值（`matches_full` 内）。
    Checked,
    /// `:disabled`——禁用态表单控件（button/input/select/textarea/option/optgroup 带
    /// `disabled` 属性，或位于带 `disabled` 的 `<fieldset>` 后代——HTML spec §4.10.18 禁用
    /// 传播，首个 `<legend>` 内除外）。祖先链传播由
    /// [`crate::Document::is_effectively_disabled`] 求值（matches_full 延后返 true）。
    Disabled,
    /// `:enabled`——启用态表单控件（表单控件且非 [`PseudoClass::Disabled`]）。
    Enabled,
    /// `:required`——可约束表单控件（input/select/textarea）带 `required` 属性。纯元素属性求值。
    Required,
    /// `:optional`——可约束表单控件无 `required` 属性（`:required` 在可约束元素上的补集）。
    Optional,
    /// `:read-write`——可编辑文本控件（文本可编辑 type 的 input 或 textarea），无 `readonly`/`disabled`。
    /// 纯元素属性求值（注：`contenteditable` 未实现）。
    ReadWrite,
    /// `:read-only`——非 `:read-write`（所有不可编辑元素，含 `<p>`/`<div>` 等非表单元素）。
    ReadOnly,
    /// `:placeholder-shown`——input/textarea 正在显示 placeholder（有 `placeholder` 且当前无值）。
    /// 需读子文本节点（textarea），延后至 `Document::is_placeholder_shown` 复评。
    PlaceholderShown,
    /// `:indeterminate`——`<progress>` 无 `value`，或 `<input type=radio>` 其组内无 checked 成员。
    /// 需子树扫描（radio 组），延后至 `Document::is_indeterminate` 复评。
    Indeterminate,
    /// `:default`——默认表单元素：`<option selected>` / checkbox/radio 带 `checked` / form 内首个 submit 按钮。
    /// 需 form owner 子树扫描，延后至 `Document::is_default_form_element` 复评。
    Default,
}

/// `:nth-*` 的 `an+b` 表达式（a=系数，b=常量；匹配条件：存在 k≥0 使 position = a*k+b）。
/// `odd`=(2,1)、`even`=(2,0)、纯整数 `5`=(0,5)、`n`=(1,0)、`2n+1`=(2,1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nth {
    /// 系数 a（`n` 的倍数）。
    pub a: i32,
    /// 常量 b。
    pub b: i32,
}

impl Nth {
    /// position（1-based）是否匹配 `an+b`：存在非负整数 k 使 position = a*k + b。
    pub fn matches(self, position: i32) -> bool {
        if self.a == 0 {
            position == self.b
        } else {
            let diff = position - self.b;
            diff % self.a == 0 && diff / self.a >= 0
        }
    }
}

/// 元素在兄弟中的位置上下文（伪类评估用，由 `Document` 计算后传入）。
#[derive(Debug, Clone, Copy, Default)]
pub struct ElementPosition {
    /// 在所有元素兄弟中的 1-based 序号。
    pub child_index: usize,
    /// 元素兄弟总数（含自身）。
    pub child_count: usize,
    /// 在同 tag 元素兄弟中的 1-based 序号。
    pub type_index: usize,
    /// 同 tag 元素兄弟总数（含自身）。
    pub type_count: usize,
    /// 是否文档根元素（`<html>`，无元素父）。
    pub is_root: bool,
    /// 是否无子节点（`:empty`）。
    pub is_empty: bool,
}

/// 属性选择器。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSelector {
    /// 属性名。
    pub name: String,
    /// 属性值匹配模式。
    pub matcher: AttributeMatcher,
}

/// 属性值匹配模式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeMatcher {
    /// 仅存在：`[attr]`
    Exists,
    /// 精确匹配：`[attr=value]`
    Exact(String),
    /// 空格分隔列表包含：`[attr~=value]`
    Includes(String),
    /// 前缀匹配：`[attr^=value]`（CSS3，值前缀）
    Prefix(String),
    /// 后缀匹配：`[attr$=value]`（CSS3，值后缀）
    Suffix(String),
    /// 子串匹配：`[attr*=value]`（CSS3，值含子串）
    Substring(String),
    /// 连字符匹配：`[attr|=value]`（CSS3，值相等或以 `value-` 开头，用于 lang/区域）
    DashMatch(String),
}

impl SimpleSelector {
    /// 检查元素是否匹配此选择器的**自身部分**（tag/id/class/attr）。
    ///
    /// **不检查伪类**——`:nth-child`/`:first-child` 等需 sibling 位置上下文（`ElementPosition`），
    /// 由 [`Self::matches_full`] 配合 `Document` 计算的位置评估。仅当无伪类时 `matches` 与
    /// `matches_full` 等价；有伪类时调用方须用 `matches_full`（经 `Document::element_matches_selector`）。
    pub fn matches(&self, elem: &ElementData) -> bool {
        // 标签名匹配
        if let Some(tag) = &self.tag
            && !elem.local_name().eq_ignore_ascii_case(tag)
        {
            return false;
        }

        // ID 匹配
        if let Some(id) = &self.id
            && elem.id.as_deref() != Some(id.as_str())
        {
            return false;
        }

        // 类名匹配（所有指定的类名都必须存在）
        for class in &self.classes {
            if !elem.class_list.iter().any(|c| c == class) {
                return false;
            }
        }

        // 属性匹配
        if let Some(attr_sel) = &self.attribute {
            match &attr_sel.matcher {
                AttributeMatcher::Exists => {
                    if !elem.has_attribute(&attr_sel.name) {
                        return false;
                    }
                }
                AttributeMatcher::Exact(value) => {
                    if elem.get_attribute(&attr_sel.name).as_deref() != Some(value.as_str()) {
                        return false;
                    }
                }
                AttributeMatcher::Includes(value) => {
                    let attr_val = match elem.get_attribute(&attr_sel.name) {
                        Some(v) => v,
                        None => return false,
                    };
                    if !attr_val.split_whitespace().any(|v| v == value) {
                        return false;
                    }
                }
                AttributeMatcher::Prefix(value) => {
                    let attr_val = match elem.get_attribute(&attr_sel.name) {
                        Some(v) => v,
                        None => return false,
                    };
                    if !attr_val.starts_with(value) {
                        return false;
                    }
                }
                AttributeMatcher::Suffix(value) => {
                    let attr_val = match elem.get_attribute(&attr_sel.name) {
                        Some(v) => v,
                        None => return false,
                    };
                    if !attr_val.ends_with(value) {
                        return false;
                    }
                }
                AttributeMatcher::Substring(value) => {
                    let attr_val = match elem.get_attribute(&attr_sel.name) {
                        Some(v) => v,
                        None => return false,
                    };
                    if !attr_val.contains(value) {
                        return false;
                    }
                }
                AttributeMatcher::DashMatch(value) => {
                    let attr_val = match elem.get_attribute(&attr_sel.name) {
                        Some(v) => v,
                        None => return false,
                    };
                    // `[attr|=val]`：值等于 val，或以 `val-` 开头（如 `en` 匹配 `en-US`）。
                    if !(attr_val == *value || attr_val.starts_with(&format!("{value}-"))) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// 检查元素是否匹配此选择器（**含伪类**），需 `Document` 计算的 [`ElementPosition`]。
    /// 无伪类时退化为 [`Self::matches`]。
    pub fn matches_full(&self, elem: &ElementData, pos: ElementPosition) -> bool {
        if !self.matches(elem) {
            return false;
        }
        // 多伪类 AND 语义（如 `li:first-child:last-child`）。
        self.pseudos.iter().all(|p| match p {
            PseudoClass::NthChild(nth) => nth.matches(pos.child_index as i32),
            PseudoClass::NthOfType(nth) => nth.matches(pos.type_index as i32),
            // 倒序位置：nth-last-child 从末尾数（child_count - child_index + 1）。
            PseudoClass::NthLastChild(nth) => nth.matches((pos.child_count - pos.child_index + 1) as i32),
            PseudoClass::NthLastOfType(nth) => nth.matches((pos.type_count - pos.type_index + 1) as i32),
            PseudoClass::FirstChild => pos.child_index == 1,
            PseudoClass::LastChild => pos.child_index == pos.child_count,
            PseudoClass::OnlyChild => pos.child_count == 1,
            PseudoClass::FirstOfType => pos.type_index == 1,
            PseudoClass::LastOfType => pos.type_index == pos.type_count,
            PseudoClass::OnlyOfType => pos.type_count == 1,
            PseudoClass::Root => pos.is_root,
            PseudoClass::Empty => pos.is_empty,
            // :not(inner)——否定（内嵌经 matches_full 递归评估，可含伪类）。
            PseudoClass::Not(inner) => !inner.matches_full(elem, pos),
            // :is/:where(list)——任一内嵌匹配则真。
            PseudoClass::Is(list) => list.iter().any(|inner| inner.matches_full(elem, pos)),
            // :has(inner)——需 Document 子树求值，matches_full 无 Document 访问，延后返 true。
            // 由 Document::element_matches_selector 在 matches_full 后额外评估。
            PseudoClass::Has { .. } => true,
            // 表单状态伪类——纯元素 tag+属性求值（无 Document/position 依赖）。
            PseudoClass::Checked => is_checked(elem),
            PseudoClass::Required => is_required(elem),
            PseudoClass::Optional => is_optional(elem),
            // `:read-write`/`:read-only` 须含 disabled 态判定（spec：禁用控件只读）——
            // `<fieldset disabled>` 传播禁用须祖先链求值（matches_full 无 Document 访问），
            // 故延后返 true，由 Document::element_matches_selector 经
            // `is_effectively_read_write` 复评（镜像 :has()/:disabled 两阶段模式）。
            PseudoClass::ReadWrite | PseudoClass::ReadOnly => true,
            // `:placeholder-shown`/`:indeterminate`/`:default` 须 Document 子树/属性上下文，
            // 延后返 true，由 Document::element_matches_selector 复评。
            PseudoClass::PlaceholderShown | PseudoClass::Indeterminate | PseudoClass::Default => true,
            // `:disabled`/`:enabled`——HTML spec `<fieldset disabled>` 向后代传播禁用态
            // 须沿祖先链求值（matches_full 无 Document 访问），故此处延后返 true，
            // 由 Document::element_matches_selector 经 `is_effectively_disabled` 复评
            // （镜像 :has() 的两阶段评估模式）。
            PseudoClass::Disabled | PseudoClass::Enabled => true,
        })
    }
}

/// 选择器组合器（连接两个简单选择器）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// 后代选择器（空格）。
    Descendant,
    /// 子选择器（`>`）。
    Child,
}

/// 由简单选择器与组合器构成的选择器链（如 `div > span.foo`）。
#[derive(Debug, Clone)]
pub struct SelectorChain {
    /// 从左到右的简单选择器序列。
    pub parts: Vec<SimpleSelector>,
    /// `combinators[i]` 连接 `parts[i]` 与 `parts[i + 1]`。
    pub combinators: Vec<Combinator>,
}

/// 按 `sep` 切分字符串，但忽略 `()`/`[]` 内的出现（如 `:is(a > b)` 内的 `>`、`[a=b]`）。
fn split_outside_brackets(s: &str, sep: u8) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, &b) in s.as_bytes().iter().enumerate() {
        match b {
            b'(' | b'[' => depth += 1,
            b')' | b']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            c if c == sep && depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

/// 按空白切分（后代组合器边界），忽略 `()`/`[]` 内的空白（如 `:is(.a, .b)` 逗号后空格）。
/// 跳过空段。
fn split_ws_outside_brackets(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        match bytes[i] {
            b'(' | b'[' => depth += 1,
            b')' | b']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            b' ' | b'\t' | b'\n' | b'\r' if depth == 0 => {
                if i > start {
                    parts.push(&s[start..i]);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        parts.push(&s[start..]);
    }
    parts
}

/// 解析含后代/子选择器的选择器链；单段时退化为简单选择器。
pub fn parse_selector_chain(selector: &str) -> Option<SelectorChain> {
    let trimmed = selector.trim();
    if trimmed.is_empty() {
        return None;
    }
    // `>` 切分子选择器段——但须忽略 `()`/`[]` 内的 `>`（如 `:is(a > b)` 内嵌组合器、`[a>b]` 属性值）。
    let segments: Vec<&str> = split_outside_brackets(trimmed, b'>')
        .into_iter()
        .map(str::trim)
        .collect();
    let mut parts = Vec::new();
    let mut combinators = Vec::new();

    for (seg_idx, segment) in segments.iter().enumerate() {
        // 空白切分后代组合器——须忽略 `()`/`[]` 内的空白（如 `:is(.a, .b)` 逗号后空格）。
        let subs: Vec<&str> = split_ws_outside_brackets(segment);
        if subs.is_empty() {
            return None;
        }
        for (sub_idx, sub) in subs.iter().enumerate() {
            parts.push(parse_simple_selector(sub)?);
            let is_last_in_segment = sub_idx + 1 == subs.len();
            let is_last_segment = seg_idx + 1 == segments.len();
            if !is_last_in_segment {
                combinators.push(Combinator::Descendant);
            } else if !is_last_segment {
                combinators.push(Combinator::Child);
            }
        }
    }

    if parts.len() == 1 {
        combinators.clear();
    }

    Some(SelectorChain { parts, combinators })
}

/// 解析 `:nth-child(an+b)` 的 `an+b` 参数 → `(a, b)`。
///
/// 支持：`odd`→(2,1)、`even`→(2,0)、纯整数 `5`→(0,5)、`n`→(1,0)、`2n`→(2,0)、
/// `2n+1`→(2,1)、`-n+3`→(-1,3)、`n+2`→(1,2)。无法解析 → `None`。
pub fn parse_nth(arg: &str) -> Option<Nth> {
    let s = arg.trim();
    match s {
        "odd" => return Some(Nth { a: 2, b: 1 }),
        "even" => return Some(Nth { a: 2, b: 0 }),
        _ => {}
    }
    if let Some(n_pos) = s.find('n') {
        // 形如 [a]n[+/-b]
        let left = s[..n_pos].trim();
        let a: i32 = match left {
            "" | "+" => 1,
            "-" => -1,
            other => other.parse().ok()?,
        };
        let right = s[n_pos + 1..].trim();
        let b: i32 = if right.is_empty() {
            0
        } else {
            // right 形如 "+1"/"-3"/"1"——i32::parse 不接受前导 '+'，规范化处理。
            let r = right.replacen('+', "", 1);
            r.parse().ok()?
        };
        Some(Nth { a, b })
    } else {
        // 纯整数 b
        Some(Nth {
            a: 0,
            b: s.parse().ok()?,
        })
    }
}

// 解析伪类名（+可选括号参数）→ `PseudoClass`。`name` 为 `:` 之后、`(` 或下一个分隔符之前的部分。
// `args` 为括号内原始字符串（`nth-*` 用），无括号时为 `None`。
// 注：`:disabled`/`:enabled` 的元素级判定（含 `<fieldset disabled>` / `<select disabled>` /
// `<optgroup disabled>` 祖先传播）由 `Document::is_effectively_disabled` 负责（需祖先链
// 上下文），`matches_full` 对二者延后返 true，由 `Document::element_matches_selector` 复评。

/// `:checked`——checkbox/radio（带 `checked` 属性）或 option（带 `selected` 属性）。
/// `type` 属性值按 HTML ASCII 大小写不敏感比较。
fn is_checked(elem: &ElementData) -> bool {
    match elem.local_name() {
        "input" => {
            elem.get_attribute("type")
                .is_some_and(|t| t.eq_ignore_ascii_case("checkbox") || t.eq_ignore_ascii_case("radio"))
                && elem.has_attribute("checked")
        }
        "option" => elem.has_attribute("selected"),
        _ => false,
    }
}

/// 可设 `required` 的元素（HTML spec `:required`/`:optional` 仅限可约束表单控件）。
fn is_requireable_tag(tag: &str) -> bool {
    matches!(tag, "input" | "select" | "textarea")
}

/// `:required`——可约束元素带 `required` 属性。
fn is_required(elem: &ElementData) -> bool {
    is_requireable_tag(elem.local_name()) && elem.has_attribute("required")
}

/// `:optional`——可约束元素无 `required` 属性。
fn is_optional(elem: &ElementData) -> bool {
    is_requireable_tag(elem.local_name()) && !elem.has_attribute("required")
}

// `:read-write`/`:read-only` 含 disabled 态判定（含 `<fieldset disabled>` 祖先传播），
// 须 Document 上下文，由 `Document::is_effectively_read_write` 负责。

/// 属性选择器运算符（CSS3 属性选择器全部 6 种）。
#[derive(Clone, Copy)]
enum AttrOp {
    /// `[attr~=value]`
    Includes,
    /// `[attr=value]`
    Exact,
    /// `[attr^=value]`
    Prefix,
    /// `[attr$=value]`
    Suffix,
    /// `[attr*=value]`
    Substring,
    /// `[attr|=value]`
    DashMatch,
}

/// 解析属性选择器运算符。`content` 为 `[` 与 `]` 之间的内容（如 `href^="https"`）。
/// 返回 `(运算符, 属性名, 值原始串)`；仅存在（`[attr]`，无运算符）返回 `None`。
///
/// 两字符运算符（`~=` `^=` `$=` `*=` `|=`）先于单字符 `=` 检测——属性名不含这些字符，
/// 且 CSS 语法恒为 `name op value`，故 `find` 命中的首个运算符即真运算符（值内相同字符
/// 序列位于运算符之后，不会被先命中）。值内引号由 [`strip_attr_quotes`] 处理。
fn parse_attr_operator(content: &str) -> Option<(AttrOp, &str, &str)> {
    for (token, op) in [
        ("~=", AttrOp::Includes),
        ("^=", AttrOp::Prefix),
        ("$=", AttrOp::Suffix),
        ("*=", AttrOp::Substring),
        ("|=", AttrOp::DashMatch),
    ] {
        if let Some(pos) = content.find(token) {
            return Some((op, &content[..pos], &content[pos + token.len()..]));
        }
    }
    content
        .find('=')
        .map(|pos| (AttrOp::Exact, &content[..pos], &content[pos + 1..]))
}

/// 去除属性值两端一对匹配引号（`"` 或 `'`）。无引号或不成对则原样返回。
/// 让 `[attr="value"]` / `[attr^='https']` 的带引号形式与裸 `[attr=value]` 等价。
fn strip_attr_quotes(s: &str) -> String {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 && (bytes[0] == b'"' || bytes[0] == b'\'') && bytes[0] == bytes[s.len() - 1] {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn parse_pseudo(name: &str, args: Option<&str>) -> Option<PseudoClass> {
    match name {
        "nth-child" => Some(PseudoClass::NthChild(parse_nth(args?)?)),
        "nth-of-type" => Some(PseudoClass::NthOfType(parse_nth(args?)?)),
        "nth-last-child" => Some(PseudoClass::NthLastChild(parse_nth(args?)?)),
        "nth-last-of-type" => Some(PseudoClass::NthLastOfType(parse_nth(args?)?)),
        "first-child" => Some(PseudoClass::FirstChild),
        "last-child" => Some(PseudoClass::LastChild),
        "only-child" => Some(PseudoClass::OnlyChild),
        "first-of-type" => Some(PseudoClass::FirstOfType),
        "last-of-type" => Some(PseudoClass::LastOfType),
        "only-of-type" => Some(PseudoClass::OnlyOfType),
        "root" => Some(PseudoClass::Root),
        "empty" => Some(PseudoClass::Empty),
        // :not(simple)——CSS3 否定伪类，内嵌经 parse_simple_selector（可含伪类，如 :not(:first-child)）。
        "not" => Some(PseudoClass::Not(parse_simple_selector(args?)?)),
        // :is()/:where()——选择器列表（按 `,` 拆为多个 SimpleSelector，任一匹配）。
        // `:where` 语义同 `:is`（区别仅特异性，本引擎无特异性概念）。
        "is" | "where" => {
            let a = args?;
            let list: Vec<SimpleSelector> = a.split(',').filter_map(|s| parse_simple_selector(s.trim())).collect();
            if list.is_empty() {
                None
            } else {
                Some(PseudoClass::Is(list))
            }
        }
        // :has(inner) / :has(> inner)——关系伪类。`> ` 前缀 = 直接子作用域，否则后代作用域。
        // inner 为选择器字符串（含组合器），由 Document 子树求值（paren-aware 切分已就绪）。
        "has" => {
            let a = args?;
            let t = a.trim();
            let (child_scope, inner) = if let Some(rest) = t.strip_prefix('>') {
                (true, rest.trim().to_string())
            } else {
                (false, t.to_string())
            };
            if inner.is_empty() {
                None
            } else {
                Some(PseudoClass::Has { inner, child_scope })
            }
        }
        // 表单状态伪类（无参，纯元素属性求值）。与 style-system CSS 匹配同源。
        "checked" => Some(PseudoClass::Checked),
        "disabled" => Some(PseudoClass::Disabled),
        "enabled" => Some(PseudoClass::Enabled),
        "required" => Some(PseudoClass::Required),
        "optional" => Some(PseudoClass::Optional),
        "read-write" => Some(PseudoClass::ReadWrite),
        "read-only" => Some(PseudoClass::ReadOnly),
        "placeholder-shown" => Some(PseudoClass::PlaceholderShown),
        "indeterminate" => Some(PseudoClass::Indeterminate),
        "default" => Some(PseudoClass::Default),
        _ => None, // 未识别伪类（:hover/:focus 等）→ 视为不匹配该 compound（保守）
    }
}
///
/// 支持格式：
/// - `"div"` — 标签名
/// - `"#myid"` — ID
/// - `".myclass"` — 类名
/// - `"[attr]"` — 属性存在
/// - `"[attr=value]"` — 属性精确匹配
/// - `"[attr~=value]"` — 属性空格分隔匹配
/// - `":nth-child(2)"` / `":first-child"` 等 — 伪类（多伪类 AND）
/// - `"div#id.class[attr=val]:first-child"` — 组合
pub fn parse_simple_selector(selector: &str) -> Option<SimpleSelector> {
    let s = selector.trim();
    if s.is_empty() {
        return None;
    }

    let mut result = SimpleSelector {
        tag: None,
        id: None,
        classes: Vec::new(),
        attribute: None,
        pseudos: Vec::new(),
    };

    let mut rest = s;

    // 解析标签名（开头的连续非特殊字符）
    if let Some(pos) = rest.find(['#', '.', '[', ':']) {
        if pos > 0 {
            result.tag = Some(rest[..pos].to_string());
        }
        rest = &rest[pos..];
    } else {
        result.tag = Some(rest.to_string());
        return Some(result);
    }

    // 解析后续的选择器部分
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('#') {
            // ID 选择器
            let end = r.find(['.', '[', ':']).unwrap_or(r.len());
            if end == 0 {
                return None; // 空的 ID 选择器
            }
            result.id = Some(r[..end].to_string());
            rest = &r[end..];
        } else if let Some(r) = rest.strip_prefix('.') {
            // 类选择器
            let end = r.find(['#', '.', '[', ':']).unwrap_or(r.len());
            if end == 0 {
                return None; // 空的类选择器
            }
            result.classes.push(r[..end].to_string());
            rest = &r[end..];
        } else if let Some(r) = rest.strip_prefix(':') {
            // 伪类：名字直到 `(` 或下一个分隔符；`:nth-child(...)` 含括号参数。
            let (name, args, next_rest): (&str, Option<&str>, &str) = match r.find('(') {
                Some(open) => {
                    // `)` 相对 r[open..] 的偏移 → 换算到 r 的绝对位置。
                    let close = r[open..].find(')')?;
                    let arg_end = open + close;
                    let name = &r[..open];
                    let args = &r[open + 1..arg_end];
                    (name, Some(args), &r[arg_end + 1..])
                }
                None => {
                    let end = r.find(['#', '.', '[', ':']).unwrap_or(r.len());
                    (&r[..end], None, &r[end..])
                }
            };
            if name.is_empty() {
                return None; // 空伪类名
            }
            result.pseudos.push(parse_pseudo(name, args)?);
            rest = next_rest;
        } else {
            let r = rest.strip_prefix('[')?;
            // 属性选择器
            let end_bracket = r.find(']')?;
            let attr_content = &r[..end_bracket];

            // 属性运算符检测：两字符运算符（~= ^= $= *= |=）须先于单字符 `=` 检测，
            // 否则 `[attr^=v]` 的 `=` 会先命中单字符分支。值去引号（`[a="v"]`→`v`）。
            // 返回 (运算符, name, value)——运算符为 None 表示 `[attr]` 仅存在。
            let attr_sel = if let Some((op, name, value)) = parse_attr_operator(attr_content) {
                let name = name.trim().to_string();
                let value = strip_attr_quotes(value.trim());
                let matcher = match op {
                    AttrOp::Includes => AttributeMatcher::Includes(value),
                    AttrOp::Exact => AttributeMatcher::Exact(value),
                    AttrOp::Prefix => AttributeMatcher::Prefix(value),
                    AttrOp::Suffix => AttributeMatcher::Suffix(value),
                    AttrOp::Substring => AttributeMatcher::Substring(value),
                    AttrOp::DashMatch => AttributeMatcher::DashMatch(value),
                };
                AttributeSelector { name, matcher }
            } else {
                AttributeSelector {
                    name: attr_content.trim().to_string(),
                    matcher: AttributeMatcher::Exists,
                }
            };

            result.attribute = Some(attr_sel);
            rest = &r[end_bracket + 1..];
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_selector() {
        let sel = parse_simple_selector("div").unwrap();
        assert_eq!(sel.tag.as_deref(), Some("div"));
        assert!(sel.id.is_none());
        assert!(sel.classes.is_empty());
    }

    #[test]
    fn test_parse_id_selector() {
        let sel = parse_simple_selector("#myid").unwrap();
        assert!(sel.tag.is_none());
        assert_eq!(sel.id.as_deref(), Some("myid"));
    }

    #[test]
    fn test_parse_class_selector() {
        let sel = parse_simple_selector(".myclass").unwrap();
        assert!(sel.tag.is_none());
        assert_eq!(sel.classes, vec!["myclass"]);
    }

    #[test]
    fn test_parse_attribute_selector_exists() {
        let sel = parse_simple_selector("[data-test]").unwrap();
        assert!(sel.attribute.is_some());
        let attr = sel.attribute.unwrap();
        assert_eq!(attr.name, "data-test");
        assert!(matches!(attr.matcher, AttributeMatcher::Exists));
    }

    #[test]
    fn test_parse_attribute_selector_exact() {
        let sel = parse_simple_selector("[type=text]").unwrap();
        let attr = sel.attribute.unwrap();
        assert_eq!(attr.name, "type");
        assert!(matches!(attr.matcher, AttributeMatcher::Exact(v) if v == "text"));
    }

    #[test]
    fn test_parse_combined_selector() {
        let sel = parse_simple_selector("div#myid.myclass").unwrap();
        assert_eq!(sel.tag.as_deref(), Some("div"));
        assert_eq!(sel.id.as_deref(), Some("myid"));
        assert_eq!(sel.classes, vec!["myclass"]);
    }

    #[test]
    fn test_parse_multiple_class_selector() {
        let sel = parse_simple_selector(".foo.bar").unwrap();
        assert!(sel.tag.is_none());
        assert_eq!(sel.classes, vec!["foo", "bar"]);
    }

    #[test]
    fn test_parse_tag_with_multiple_classes() {
        let sel = parse_simple_selector("div.a.b").unwrap();
        assert_eq!(sel.tag.as_deref(), Some("div"));
        assert_eq!(sel.classes, vec!["a", "b"]);
    }

    #[test]
    fn test_parse_empty_selector() {
        assert!(parse_simple_selector("").is_none());
        assert!(parse_simple_selector("  ").is_none());
    }

    #[test]
    fn test_parse_nth_expr() {
        assert_eq!(parse_nth("odd"), Some(Nth { a: 2, b: 1 }));
        assert_eq!(parse_nth("even"), Some(Nth { a: 2, b: 0 }));
        assert_eq!(parse_nth("5"), Some(Nth { a: 0, b: 5 }));
        assert_eq!(parse_nth("n"), Some(Nth { a: 1, b: 0 }));
        assert_eq!(parse_nth("2n"), Some(Nth { a: 2, b: 0 }));
        assert_eq!(parse_nth("2n+1"), Some(Nth { a: 2, b: 1 }));
        assert_eq!(parse_nth("-n+3"), Some(Nth { a: -1, b: 3 }));
        assert_eq!(parse_nth("n+2"), Some(Nth { a: 1, b: 2 }));
        assert!(parse_nth("abc").is_none());
        // matches：position = a*k + b（k≥0）。
        assert!(Nth { a: 0, b: 3 }.matches(3) && !Nth { a: 0, b: 3 }.matches(2));
        assert!(Nth { a: 2, b: 1 }.matches(1) && Nth { a: 2, b: 1 }.matches(3) && !Nth { a: 2, b: 1 }.matches(2));
        assert!(Nth { a: -1, b: 3 }.matches(1) && Nth { a: -1, b: 3 }.matches(3) && !Nth { a: -1, b: 3 }.matches(4));
    }

    #[test]
    fn test_parse_pseudo_selectors() {
        let sel = parse_simple_selector("li:nth-child(2)").unwrap();
        assert_eq!(sel.tag.as_deref(), Some("li"));
        assert_eq!(sel.pseudos, vec![PseudoClass::NthChild(Nth { a: 0, b: 2 })]);

        let sel = parse_simple_selector(":first-child").unwrap();
        assert_eq!(sel.pseudos, vec![PseudoClass::FirstChild]);

        let sel = parse_simple_selector("tr:nth-of-type(odd)").unwrap();
        assert_eq!(sel.pseudos, vec![PseudoClass::NthOfType(Nth { a: 2, b: 1 })]);

        // 多伪类 AND + 与 tag/attr 组合。
        let sel = parse_simple_selector("li.x:first-child:last-child").unwrap();
        assert_eq!(sel.tag.as_deref(), Some("li"));
        assert_eq!(sel.classes, vec!["x"]);
        assert_eq!(sel.pseudos, vec![PseudoClass::FirstChild, PseudoClass::LastChild]);

        // 未识别伪类（:hover）→ 解析失败（保守，避免静默误匹配）。
        assert!(parse_simple_selector("a:hover").is_none());

        // 空伪类名 → None。
        assert!(parse_simple_selector("div:").is_none());
    }

    #[test]
    fn test_parse_not_pseudo() {
        // :not(.skip) → Not(SimpleSelector{classes:[skip]})。
        let sel = parse_simple_selector("div:not(.skip)").unwrap();
        assert_eq!(sel.tag.as_deref(), Some("div"));
        assert_eq!(sel.pseudos.len(), 1);
        match &sel.pseudos[0] {
            PseudoClass::Not(inner) => {
                assert!(inner.tag.is_none());
                assert_eq!(inner.classes, vec!["skip"]);
            }
            other => panic!("expected Not, got {other:?}"),
        }
        // :not(:first-child)——内嵌含伪类。
        let sel = parse_simple_selector("li:not(:first-child)").unwrap();
        match &sel.pseudos[0] {
            PseudoClass::Not(inner) => assert_eq!(inner.pseudos, vec![PseudoClass::FirstChild]),
            other => panic!("expected Not, got {other:?}"),
        }
        // 空 :not() → None。
        assert!(parse_simple_selector("div:not()").is_none());
    }

    #[test]
    fn test_parse_selector_chain_descendant() {
        let chain = parse_selector_chain("div .child").unwrap();
        assert_eq!(chain.parts.len(), 2);
        assert_eq!(chain.combinators, vec![Combinator::Descendant]);
    }

    #[test]
    fn test_parse_selector_chain_child() {
        let chain = parse_selector_chain("div > span").unwrap();
        assert_eq!(chain.parts.len(), 2);
        assert_eq!(chain.combinators, vec![Combinator::Child]);
    }
}
