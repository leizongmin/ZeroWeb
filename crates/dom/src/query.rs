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
    /// `:nth-child(an+b of S)`（Selectors L4 §16）：元素须匹配 `of` 列表 S 中任一选择器，且
    /// 在父元素**仅计匹配 S 的元素兄弟**中的位置满足 `an+b`。`of_selectors` 为简单选择器字符串
    /// 列表（`,` 分隔，每个由 `parse_simple_selector` 解析），需兄弟枚举 + S 过滤，由
    /// `Document::matches_nth_child_of` 求值（`matches_full` 延后返 true）。
    NthChildOf(Nth, Vec<String>),
    /// `:nth-last-child(an+b of S)`（Selectors L4）：从末尾仅计匹配 S 的兄弟。同 [`NthChildOf`]
    /// 由 `Document::matches_nth_last_child_of` 求值。
    NthLastChildOf(Nth, Vec<String>),
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
    /// `:any-link` / `:link`——超链接元素（`<a>`/`<area>`/`<link>` 带 `href` 属性，CSS Selectors L4 §18）。
    /// 纯元素属性求值（`matches_full` 内）。`:link` 静态下等价（全当未访问，隐私安全）。
    AnyLink,
    /// `:visited`——已访问超链接。出于隐私安全（CSS Selectors L4 §18：`:visited` 在样式表外
    /// 恒不匹配），本引擎静态永不匹配（`matches_full` 内返 false）。
    Visited,
    /// `:scope`——文档样式表作用域根（`<html>`，等价 `:root`，镜像 style-system 语义）。
    /// 需 Document 知道元素是否文档根，延后至 `Document::is_scope_element` 复评。
    Scope,
    /// `:lang(ranges)`——元素语言匹配（CSS Selectors L4 §14）：自身或最近祖先 `xml:lang`/`lang`
    /// 属性，逗号分隔 BCP 47 语言范围列表 OR 匹配。需祖先链 + BCP 47 匹配，延后至
    /// `Document::matches_lang` 复评。
    Lang(Vec<String>),
    /// `:dir(ltr|rtl)`——元素方向性匹配（CSS Selectors L4 §14）：`dir` 属性沿祖先继承
    /// （ltr/rtl/auto），`auto` 按子树首个强方向字符。需祖先链 + 子树扫描，延后至
    /// `Document::matches_dir` 复评。
    Dir(String),
    /// `:target`——当前文档 URL fragment（百分号解码）指向的唯一元素（CSS Selectors L3 §6.6.2）。
    /// 需读文档 URL，延后至 `Document::is_target_element` 复评。
    Target,
    /// `:valid`——候选校验元素（input/select/textarea，非 disabled/readonly）无约束失败
    /// （HTML §4.10.20）。静态子集：valueMissing + range 越界；patternMismatch/typeMismatch 不
    /// 在静态范围（permissive valid，与 engine shim ValidityState 同哲学）。延后至
    /// `Document::is_valid_element` 复评。
    Valid,
    /// `:invalid`——候选校验元素存在任一静态约束失败（`:valid` 补集）。延后至
    /// `Document::is_invalid_element` 复评。
    Invalid,
    /// `:in-range`——range-applicable input（number/range/date 等）有 value 且落在 [min,max]。
    /// 延后至 `Document::is_in_range_element` 复评。
    InRange,
    /// `:out-of-range`——range-applicable input 有 value 但 <min 或 >max。延后至
    /// `Document::is_out_of_range_element` 复评。
    OutOfRange,
    /// `:defined`——元素已定义（HTML §3.1.3 + CSS Selectors §10）：内置元素或**已升级**的自定义元素
    /// 匹配；**未升级**的自定义元素（标签为合法 custom element 名但尚未 `customElements.define`）
    /// 不匹配。Web Components 高频特征检测（`:not(:defined)` 隐藏未升级组件、`:defined` 触发升级后逻辑）。
    ///
    /// **静态近似（dom crate 无 customElements registry）**：合法 custom element 名（含连字符、
    /// 首字符小写 ASCII 字母、无大写）的元素在 parse 时视为**未升级**（registry 在 engine 层，
    /// dom crate 不可见）→ `:defined` 返 false；其余（原生 HTML 元素、含大写或无连字符的 tag）
    /// → `:defined` 返 true。镜像 R3271 fast-path 的连字符启发式（更精确的 PotentialCustomElementName
    /// 见 [`is_valid_custom_element_name`]）。registry-aware 精确化（engine 注入已注册名集）为 follow-up。
    Defined,
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
            // `:nth-child(an+b of S)` / `:nth-last-child(an+b of S)`——须仅计匹配 S 的兄弟，
            // matches_full 无 Document 访问无法枚举兄弟，延后返 true，由
            // Document::element_matches_selector 经 matches_nth_child_of/_last_child_of 复评。
            PseudoClass::NthChildOf(_, _) | PseudoClass::NthLastChildOf(_, _) => true,
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
            // `:any-link`/`:link`——纯元素属性求值（a/area/link 带 href）。
            PseudoClass::AnyLink => is_any_link(elem),
            // `:visited`——静态永不匹配（隐私安全，防历史探测）。
            PseudoClass::Visited => false,
            // `:defined`——纯元素 tag 名求值（静态近似，无需 Document）：合法 custom element 名
            // → parse 时视为未升级 → 不匹配；其余 tag（原生/含大写/无连字符）→ 已定义 → 匹配。
            PseudoClass::Defined => !is_valid_custom_element_name(elem.local_name()),
            // `:scope`/`:lang()`/`:dir()`/`:target`/`:valid`/`:invalid`/`:in-range`/`:out-of-range`
            // 需 Document 祖先链/根/URL/约束属性上下文，延后返 true，由 Document::element_matches_selector
            // 复评（镜像 :disabled 两阶段模式）。
            PseudoClass::Scope
            | PseudoClass::Lang(_)
            | PseudoClass::Dir(_)
            | PseudoClass::Target
            | PseudoClass::Valid
            | PseudoClass::Invalid
            | PseudoClass::InRange
            | PseudoClass::OutOfRange => true,
            // `:disabled`/`:enabled`——HTML spec `<fieldset disabled>` 向后代传播禁用态
            // 须沿祖先链求值（matches_full 无 Document 访问），故此处延后返 true，
            // 由 Document::element_matches_selector 经 `is_effectively_disabled` 复评
            // （镜像 :has() 的两阶段评估模式）。
            PseudoClass::Disabled | PseudoClass::Enabled => true,
        })
    }
}

/// 选择器组合器（连接两个简单选择器）。与 CSS Selectors L3 §14 对齐——style-system
/// CSS matcher 同支持全部四种（zero_css_parser::ast::Combinator），DOM querySelector
/// 选择器引擎须与之同源（R3285 闭合 `+`/`~` 缺口，延续 R3277-R3284 DOM/CSS 一致化系列）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// 后代选择器（空格）。
    Descendant,
    /// 子选择器（`>`）。
    Child,
    /// 相邻兄弟选择器（`+`）——紧邻的前一个元素兄弟。
    NextSibling,
    /// 通用兄弟选择器（`~`）——任一在先的元素兄弟。
    SubsequentSibling,
}

/// 由简单选择器与组合器构成的选择器链（如 `div > span.foo`）。
#[derive(Debug, Clone)]
pub struct SelectorChain {
    /// 从左到右的简单选择器序列。
    pub parts: Vec<SimpleSelector>,
    /// `combinators[i]` 连接 `parts[i]` 与 `parts[i + 1]`。
    pub combinators: Vec<Combinator>,
}

/// 解析含组合器的选择器链（` ` 后代 / `>` 子 / `+` 相邻兄弟 / `~` 通用兄弟）；单段时退化为简单选择器。
///
/// 与 CSS Selectors L3 §14 四组合器对齐——style-system CSS matcher 同源（R3285）。
/// 忽略 `()`/`[]` 内的组合器边界（如 `:is(a > b)` 内嵌组合器、`[a>b]` 属性值、
/// `:nth-child(2n+1)` 参数内的 `+`、`:not(.a~.b)` 内嵌 `~`）。
pub fn parse_selector_chain(selector: &str) -> Option<SelectorChain> {
    let trimmed = selector.trim();
    if trimmed.is_empty() {
        return None;
    }
    // 扫描出 (子选择器段文本, 指向**下一**段的组合器) 序列。组合器 `>`/`+`/`~` 与空白
    // 均为边界，但忽略 `()`/`[]` 内的出现。多个连续边界（如 `a > b` 中 `>` 两侧空白、
    // 或 `a   b` 多空白）压缩为单一组合器：显式符号（>`/`+`/`~）优先于隐式空白（后代）。
    let tokens = tokenize_combinators(trimmed);
    if tokens.is_empty() {
        return None;
    }

    let mut parts = Vec::new();
    let mut combinators = Vec::new();
    for (idx, (seg, comb)) in tokens.iter().enumerate() {
        parts.push(parse_simple_selector(seg.trim())?);
        if idx + 1 < tokens.len() {
            combinators.push(*comb);
        }
    }

    if parts.len() == 1 {
        combinators.clear();
    }

    Some(SelectorChain { parts, combinators })
}

/// 扫描选择器，按组合器边界（`>`/`+`/`~`/空白，忽略 `()`/`[]` 内）切分为
/// `(段文本, 指向下一段的组合器)` 序列。末段的组合器无意义（填占位 `Descendant`，调用方不读）。
/// 多个连续边界压缩为显式符号优先（`>`/`+`/`~` 优先于空白后代）。
fn tokenize_combinators(s: &str) -> Vec<(&str, Combinator)> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut depth = 0i32;
    // 先收集所有顶层（depth==0）边界位置及类型。`pending_comb`：上一边界后待定的组合器，
    // 显式符号（>`/`+`/`~`）覆盖隐式空白（后代）；连续空白不重复计。
    // 边界 = (边界后的下一段起始字节 idx, 组合器)。
    let mut boundaries: Vec<(usize, Combinator)> = Vec::new();
    let mut i = 0usize;
    // 跟踪「自上一非空白字符以来是否已有待定组合器」，用于空白边界仅在词际触发。
    let mut last_was_segment_char = false; // 上一字节是否为段内普通字符（非边界、非边界后空白）
    let mut pending_explicit: Option<Combinator> = None;

    while i < len {
        match bytes[i] {
            b'(' | b'[' => {
                depth += 1;
                last_was_segment_char = true;
            }
            b')' | b']' => {
                if depth > 0 {
                    depth -= 1;
                }
                last_was_segment_char = true;
            }
            b'>' | b'+' | b'~' if depth == 0 => {
                let comb = match bytes[i] {
                    b'>' => Combinator::Child,
                    b'+' => Combinator::NextSibling,
                    b'~' => Combinator::SubsequentSibling,
                    _ => unreachable!(),
                };
                // 若紧前的边界是「空白触发的后代」且其间无段字符（符号紧随空白），
                // 显式符号覆盖该后代边界（如 `h1 + p` 中 `+` 前的空白误记为后代 → 改为相邻兄弟）。
                let mut upgraded = false;
                if !last_was_segment_char
                    && let Some(last) = boundaries.last_mut()
                    && last.1 == Combinator::Descendant
                {
                    last.1 = comb;
                    upgraded = true;
                }
                if !upgraded {
                    pending_explicit = Some(comb);
                } else {
                    pending_explicit = None;
                }
                last_was_segment_char = false;
            }
            b' ' | b'\t' | b'\n' | b'\r' if depth == 0 => {
                // 空白边界：仅在「上一字节是段内字符」**且**「无待定显式符号」时构成后代边界。
                // 若已有 pending_explicit（如 `h1 +` 中的 `+`），符号本身才是边界，空白仅被吸收。
                if last_was_segment_char && pending_explicit.is_none() {
                    boundaries.push((i, Combinator::Descendant));
                    last_was_segment_char = false;
                }
                // 否则跳过连续空白（含显式符号前后的空白，pending_explicit 保留待覆盖）。
            }
            _ if depth == 0 => {
                // 段内普通字符：若此前有 pending 显式符号（符号分隔了两段），在此记录边界
                //（位置 = 当前 i，组合器 = pending 显式符号），随后开启新段。
                if pending_explicit.is_some() {
                    boundaries.push((i, pending_explicit.unwrap()));
                    pending_explicit = None;
                }
                last_was_segment_char = true;
            }
            _ => {
                // `()`/`[]` 内部字符：不计边界，但维持段字符状态以便括号闭合后正确处理。
                last_was_segment_char = true;
            }
        }
        i += 1;
    }
    // 丢弃尾部任何 pending（选择器不应以组合器结尾；若如此，下一段为空，调用方 parse 返回 None）。

    // 按 boundaries 切分字符串为段序列，每段附「指向下一段的组合器」。
    let mut out = Vec::new();
    let mut seg_start = 0usize;
    for (b_idx, comb) in &boundaries {
        out.push((&s[seg_start..*b_idx], *comb));
        seg_start = *b_idx;
        // 跳过当前段与下一段之间的边界字符（空白/符号及其后空白），将 seg_start 推到下一段首个非边界字符。
        while seg_start < len {
            match bytes[seg_start] {
                b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'+' | b'~' => seg_start += 1,
                _ => break,
            }
        }
    }
    out.push((&s[seg_start..], Combinator::Descendant));
    out
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

/// 解析 `:nth-child(...)` / `:nth-last-child(...)` 参数——支持 Selectors L4 `an+b of S` 语法。
///
/// `arg` 形如 `even`、`2n+1`、`even of .item`、`-n+3 of .a, .b`。`of` 关键字（大小写不敏感，
/// 两侧须有空格——CSS tokenization：`of` 为独立 ident，前后空白分隔）切分 `an+b` 与 `of S`
/// 选择器列表。无 `of` → 纯 [`PseudoClass::NthChild`]/[`PseudoClass::NthLastChild`]；有 `of`
/// → [`PseudoClass::NthChildOf`]/[`PseudoClass::NthLastChildOf`]（`of_selectors` 为 `,` 分隔
/// 的简单选择器字符串列表，延后至 `Document` 求值）。`last` 控制变体方向。
fn parse_nth_or_nth_of(arg: &str, last: bool) -> Option<PseudoClass> {
    // ` of ` 分隔符大小写不敏感查找（CSS tokenization：`of` 为独立 ident，前后须有空格）。
    let lower = arg.to_ascii_lowercase();
    let of_idx = lower.find(" of ");
    let Some(of_idx) = of_idx else {
        // 无 `of` → 纯 nth-*。
        let nth = parse_nth(arg)?;
        return Some(if last {
            PseudoClass::NthLastChild(nth)
        } else {
            PseudoClass::NthChild(nth)
        });
    };
    let nth_part = &arg[..of_idx];
    let of_part = &arg[of_idx + 4..]; // 跳过 " of "
    let nth = parse_nth(nth_part)?;
    // `of S`：逗号分隔简单选择器列表（与 `:is()` 同 parse_simple_selector）。
    let of_selectors: Vec<String> = of_part
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if of_selectors.is_empty() {
        // `of ` 后无选择器（如 `even of`）→ 非法，视为不匹配。
        return None;
    }
    Some(if last {
        PseudoClass::NthLastChildOf(nth, of_selectors)
    } else {
        PseudoClass::NthChildOf(nth, of_selectors)
    })
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

/// `:any-link` / `:link`（CSS Selectors L4 §18）——超链接元素：`<a>`/`<area>`/`<link>` 带 `href`
/// 属性。`:link` 静态下等价（全当未访问，隐私安全）。纯元素 tag+属性求值。
fn is_any_link(elem: &ElementData) -> bool {
    matches!(elem.local_name(), "a" | "area" | "link") && elem.has_attribute("href")
}

/// 判定 tag 名是否为**合法 custom element 名**（HTML spec PotentialCustomElementName，
/// https://html.spec.whatwg.org/multipage/custom-elements.html#prod-potentialcustomelementname）。
///
/// 用于 `:defined` 静态近似：合法 CE 名（首字符小写 ASCII 字母、含 ASCII 连字符、仅小写字母/数字/`-`/`.`）
/// 在 dom crate parse 时视为未升级 custom element；非合法名（原生 HTML 元素、含大写或无连字符）视为已定义。
///
/// 镜像 engine `dom_bindings/custom_elements.rs` R3271 fast-path 的连字符启发式，并补精确字符集校验
/// （排除含大写的 tag——HTML 解析器已小写化，故含大写者非 HTML 元素，按 SVG/MathML/未知处理为 defined）。
/// spec 保留名（`annotation-xml`/`color-profile`/`font-face` 等无连字符或不符合本字符集）自然落 false。
///
/// `pub`：style-system matcher 的 `:defined` 求值复用（R3299 DOM/CSS 同源一致性）。
pub fn is_valid_custom_element_name(tag: &str) -> bool {
    // PCEN_Char 集合（ASCII 子集）：小写字母 a-z、数字 0-9、连字符 `-`、句点 `.`。
    // 不含大写（HTML tag 已小写化；含大写 → 非本集 → 非合法 CE 名）。
    fn is_pcen_char(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '-' | '.')
    }
    let mut chars = tag.chars();
    // 首字符须为小写 ASCII 字母（spec: CustomElementProductionStartChar = lower-case ASCII letter）。
    let Some(first) = chars.next() else {
        return false; // 空串非合法名。
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    // 须含至少一个连字符（CE 名核心要求；`div`/`svg`/`a` 等无连字符 → 非合法 CE 名）。
    // 其余字符须属 PCEN_Char 集。
    let rest = chars.as_str();
    rest.contains('-') && rest.chars().all(is_pcen_char)
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
        "nth-child" => parse_nth_or_nth_of(args?, false),
        "nth-of-type" => Some(PseudoClass::NthOfType(parse_nth(args?)?)),
        "nth-last-child" => parse_nth_or_nth_of(args?, true),
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
        // 超链接伪类（CSS Selectors L4 §18）：`:any-link`/`:link` 静态等价（全当未访问，隐私安全），
        // `:visited` 永不匹配（样式表外恒 false，防历史探测）。
        "any-link" | "link" => Some(PseudoClass::AnyLink),
        "visited" => Some(PseudoClass::Visited),
        // `:scope`——文档样式表等价 `:root`，延后至 Document 复评。
        "scope" => Some(PseudoClass::Scope),
        // `:target`——当前文档 URL fragment 指向的唯一元素，延后至 Document 复评。
        "target" => Some(PseudoClass::Target),
        // 约束校验伪类（HTML §4.10.20，无参）：候选校验元素的约束状态，延后至 Document 复评。
        "valid" => Some(PseudoClass::Valid),
        "invalid" => Some(PseudoClass::Invalid),
        "in-range" => Some(PseudoClass::InRange),
        "out-of-range" => Some(PseudoClass::OutOfRange),
        // `:lang(ranges)`——逗号分隔 BCP 47 语言范围列表（如 `en, fr`/`*-CA`），延后至 Document 复评。
        "lang" => {
            let a = args?;
            let list: Vec<String> = a
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if list.is_empty() {
                None
            } else {
                Some(PseudoClass::Lang(list))
            }
        }
        // `:dir(ltr|rtl)`——方向性，参数归一化小写；非 ltr/rtl 延后求值时自然不匹配。
        "dir" => Some(PseudoClass::Dir(args.unwrap_or("").trim().to_ascii_lowercase())),
        // `:defined`——HTML §3.1.3 元素已定义（内置或已升级 custom element）。无参，纯元素 tag 名求值
        //（matches_full 内经 [`is_valid_custom_element_name`] 静态近似：合法 CE 名 → 未升级 → 不匹配）。
        "defined" => Some(PseudoClass::Defined),
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

    #[test]
    fn test_parse_selector_chain_next_sibling() {
        // 相邻兄弟组合器 `+`（CSS Selectors L3 §14.3）。
        let chain = parse_selector_chain("h1 + p").unwrap();
        assert_eq!(chain.parts.len(), 2);
        assert_eq!(chain.combinators, vec![Combinator::NextSibling]);
    }

    #[test]
    fn test_parse_selector_chain_subsequent_sibling() {
        // 通用兄弟组合器 `~`（CSS Selectors L3 §14.4）。
        let chain = parse_selector_chain("h1 ~ p").unwrap();
        assert_eq!(chain.parts.len(), 2);
        assert_eq!(chain.combinators, vec![Combinator::SubsequentSibling]);
    }

    #[test]
    fn test_parse_selector_chain_mixed_combinators() {
        // 混合四种组合器：`div > h1 + p ~ span`。
        let chain = parse_selector_chain("div > h1 + p ~ span").unwrap();
        assert_eq!(chain.parts.len(), 4);
        assert_eq!(
            chain.combinators,
            vec![
                Combinator::Child,
                Combinator::NextSibling,
                Combinator::SubsequentSibling,
            ]
        );
    }

    #[test]
    fn test_parse_selector_chain_sibling_no_spaces() {
        // 无空格 `h1+p` 与 `h1~p` 须同样解析。
        let chain = parse_selector_chain("h1+p").unwrap();
        assert_eq!(chain.combinators, vec![Combinator::NextSibling]);
        let chain = parse_selector_chain("h1~p").unwrap();
        assert_eq!(chain.combinators, vec![Combinator::SubsequentSibling]);
    }

    #[test]
    fn test_parse_selector_chain_plus_inside_pseudo_ignored() {
        // `:nth-child(2n+1)` 内的 `+` 非组合器边界——须忽略 `()` 内出现。
        let chain = parse_selector_chain("li:nth-child(2n+1) + p").unwrap();
        assert_eq!(chain.parts.len(), 2);
        assert_eq!(chain.combinators, vec![Combinator::NextSibling]);
    }

    #[test]
    fn test_parse_selector_chain_tilde_inside_is_ignored() {
        // `:is(.a ~ .b)` 内嵌的 `~` 非顶层组合器——须忽略 `()` 内出现。
        let chain = parse_selector_chain("div :is(.a ~ .b)").unwrap();
        assert_eq!(chain.parts.len(), 2);
        assert_eq!(chain.combinators, vec![Combinator::Descendant]);
    }

    #[test]
    fn test_tokenize_combinators_basic() {
        let toks = tokenize_combinators("h1 + p");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].0.trim(), "h1");
        assert_eq!(toks[0].1, Combinator::NextSibling);
        assert_eq!(toks[1].0.trim(), "p");
    }

    #[test]
    fn test_tokenize_combinators_descendant_collapses_whitespace() {
        // 多空白压缩为单一后代组合器。
        let toks = tokenize_combinators("a    b");
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].1, Combinator::Descendant);
    }

    /// R3299：`is_valid_custom_element_name` 字符集校验（HTML spec PotentialCustomElementName）。
    #[test]
    fn test_is_valid_custom_element_name_r3299() {
        // 合法 CE 名（首字符小写字母 + 含连字符 + 仅 PCEN_Char）。
        assert!(is_valid_custom_element_name("my-widget"));
        assert!(is_valid_custom_element_name("x-foo-bar"));
        assert!(is_valid_custom_element_name("a-")); // 单字母 + 尾连字符（spec 允许）。
        assert!(is_valid_custom_element_name("a.b-c")); // 含句点（PCEN_Char 允许 `.`）。
        assert!(is_valid_custom_element_name("my-widget2")); // 含数字。
        // 非法：无连字符（原生 HTML 元素名）。
        assert!(!is_valid_custom_element_name("div"));
        assert!(!is_valid_custom_element_name("svg"));
        assert!(!is_valid_custom_element_name("a"));
        // 非法：首字符非小写 ASCII 字母。
        assert!(!is_valid_custom_element_name("-foo"));
        assert!(!is_valid_custom_element_name("1-foo"));
        assert!(!is_valid_custom_element_name("A-foo"));
        // 非法：含大写（HTML tag 已小写化；含大写 → 非 HTML 元素，按 defined 处理）。
        assert!(!is_valid_custom_element_name("my-Widget"));
        // 非法：含非 PCEN_Char（下划线、冒号等）。
        assert!(!is_valid_custom_element_name("my_widget"));
        assert!(!is_valid_custom_element_name("my:widget"));
        // 非法：空串。
        assert!(!is_valid_custom_element_name(""));
    }

    /// R3299：`:defined` parse_pseudo 识别（不再落 `_ => None` 致整选择器无效）。
    #[test]
    fn test_parse_pseudo_defined_r3299() {
        let sel = parse_simple_selector(":defined").expect(":defined 应解析为合法伪类");
        assert_eq!(sel.pseudos.len(), 1);
        assert!(matches!(sel.pseudos[0], PseudoClass::Defined));
        // 复合选择器含 :defined 应整体有效（此前 :defined 落 None 致 parse_simple_selector 返 None）。
        let comp = parse_simple_selector("my-widget:defined").expect("my-widget:defined 应解析成功");
        assert_eq!(comp.pseudos.len(), 1);
        assert_eq!(comp.tag.as_deref(), Some("my-widget"));
    }
}
