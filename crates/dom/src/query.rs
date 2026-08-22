//! 基础选择器解析与匹配。
//!
//! 支持的简单选择器格式：
//! - 标签名：`"div"`, `"span"`
//! - ID 选择器：`"#myid"`
//! - 类选择器：`".myclass"`
//! - 属性选择器：`"[attr]"`, `"[attr=value]"`, `"[attr~=value]"`

use crate::node::ElementData;

/// R159：type selector 命名空间限定形态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NsKind {
    /// 缺省（无 ns 限定——任意 ns 的同名 localName，与既有引擎行为一致）。
    #[default]
    Default,
    /// `*|div`——任意 ns。
    AnyNs,
    /// `|div`——仅显式空 ns（namespace 为空串）。
    EmptyNs,
}

/// 简单选择器（仅支持单层选择器，不支持组合器）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleSelector {
    /// 标签名匹配（大小写不敏感）。
    pub tag: Option<String>,
    /// R159：type selector 的命名空间限定（WPT Namespace selector 簇）：
    /// `div` 缺省（HTML 文档默认 ns 语义——本引擎按「任意 ns 中的同名 localName」
    /// 近似，与既有行为一致）、`*|div` AnyNs（任意 ns）、`|div` EmptyNs（仅
    /// namespace 为空串的元素——WPT `#no-namespace |div` expect 仅 createElementNS
    /// ("", div) 产物）。
    pub ns_kind: NsKind,
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
    /// R159：伪元素（`:before`/`:after`/`:first-line`/`:first-letter` 一冒号 legacy
    /// 与 `::before` 等二冒号 modern，及 `::slotted(...)`）——**合法但恒不匹配**
    /// （spec DOM querySelector 不匹配伪元素；WPT 期望 expect: [] 不抛）。
    PseudoElement,
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
    /// `:blank`——值空或纯空白的文本输入控件（CSS UI L4 / Selectors L4 §12）：`<input>` 的 `value`
    /// 属性空/缺省，或 `<textarea>` 的文本内容空/纯空白。与 [`PseudoClass::PlaceholderShown`] 的空值
    /// 检测同源，但**不要求** `placeholder` 属性（无条件空值匹配）。延后至
    /// [`crate::Document::is_blank_element`] 复评（须读 textarea 子文本，matches_full 无 Document 访问）。
    Blank,
    /// `:fullscreen`——处于全屏态的元素（Fullscreen API
    /// https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen）。全屏须 JS
    /// `requestFullscreen()` 激活（运行时状态），静态解析的 DOM 不可知 → 静态永不匹配
    /// （镜像 [`PseudoClass::Visited`] 永不匹配模式）。识别此伪类使复合选择器如 `div:not(:fullscreen)`
    /// 不再被当无效（静默返空），与 CSS matcher（`_ => false`）一致。
    Fullscreen,
    /// `:modal`——以模态方式运行的 top-layer `<dialog>`（HTML §3.4.2，经 JS `showModal()` 激活）。
    /// 模态须运行时 `showModal()` 调用（非 `<dialog open>`，后者经 `show()` 非模态），静态解析的 DOM
    /// 不可知 → 静态永不匹配（镜像 [`PseudoClass::Visited`]）。识别此伪类使复合选择器如
    /// `dialog:not(:modal)` 不再被当无效，与 CSS matcher 一致。
    Modal,
    /// `:focus`——当前获得焦点的元素（CSS Selectors L3 §6.6.2 / DOM §3.3 Focus）。焦点须 JS `.focus()`
    /// 或用户交互激活（运行时状态），静态解析的 DOM 不可知（焦点 NodeId 由 engine shim `_activeElKey`
    /// 追踪，DOM re-parse 不携带）→ 静态永不匹配（镜像 [`PseudoClass::Visited`]/[`PseudoClass::Fullscreen`]）。
    /// 识别此伪类使复合选择器如 `input:not(:focus)` 不再被当无效，与 CSS matcher 一致。
    /// 真保真（`<input autofocus>` 静态匹配）须 engine-shim 共享面改动（run-rules §9），defer。
    Focus,
    /// `:focus-visible`——键盘导航获得焦点的元素（CSS Selectors L4 §14）。焦点启发式（键盘 vs 鼠标）
    /// 须运行时交互信号，静态不可知 → 静态永不匹配（镜像 [`PseudoClass::Focus`]）。
    FocusVisible,
    /// `:focus-within`——自身或后代获得焦点的元素（CSS Selectors L4 §14）。焦点须运行时状态，
    /// 静态不可知 → 静态永不匹配（镜像 [`PseudoClass::Focus`]）。
    FocusWithin,
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
        // 标签名匹配（R157：`*` universal——任意 tag 命中，spec selectors-4 §6.2
        // universal selector；旧版把 "*" 当字面 tag 比对恒 miss，
        // querySelectorAll("*") 全空——WPT Universal selector 簇根源）。
        if let Some(tag) = &self.tag
            && tag != "*"
            && !elem.local_name().eq_ignore_ascii_case(tag)
        {
            return false;
        }
        // R159：`|div` 显式空 ns——元素 namespace 须为空串（WPT `#no-namespace |div`
        // expect 仅 createElementNS("", div) 产物；HTML 解析产物 namespace 是
        // HTMLNS 非空 → 不命中）。`*|div` AnyNs 对 namespace 无约束（localName 已
        // 比对）——无需额外判定。
        if self.ns_kind == NsKind::EmptyNs && !elem.namespace().is_empty() {
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
                    // R124：~= 的词分隔符同 class 域（ASCII whitespace——Unicode 空白
                    // 是字面字符非分隔符，spec attribute selector words）。
                    if !crate::node::split_ascii_whitespace(&attr_val)
                        .iter()
                        .any(|v| v == value)
                    {
                        return false;
                    }
                }
                AttributeMatcher::Prefix(value) => {
                    // R158：空值 ^= 恒不匹配（WPT `[class^=""]` expect 0——
                    // spec 属性和参数均为空串时不匹配；旧版 starts_with("") 恒真）。
                    if value.is_empty() {
                        return false;
                    }
                    let attr_val = match elem.get_attribute(&attr_sel.name) {
                        Some(v) => v,
                        None => return false,
                    };
                    if !attr_val.starts_with(value) {
                        return false;
                    }
                }
                AttributeMatcher::Suffix(value) => {
                    // R158：空值 $= 恒不匹配（同 ^=——WPT `[class$=""]` expect 0）。
                    if value.is_empty() {
                        return false;
                    }
                    let attr_val = match elem.get_attribute(&attr_sel.name) {
                        Some(v) => v,
                        None => return false,
                    };
                    if !attr_val.ends_with(value) {
                        return false;
                    }
                }
                AttributeMatcher::Substring(value) => {
                    // R158：空值 *= 恒不匹配（同 ^=/$=——contains("") 恒真）。
                    if value.is_empty() {
                        return false;
                    }
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
            // R159：伪元素——DOM querySelector **恒不匹配**（spec 伪元素不是元素）。
            PseudoClass::PseudoElement => false,
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
            // `:blank` 须读 textarea 子文本（Document 上下文），延后返 true，由
            // Document::element_matches_selector 经 `is_blank_element` 复评。
            PseudoClass::Blank => true,
            // `:any-link`/`:link`——纯元素属性求值（a/area/link 带 href）。
            PseudoClass::AnyLink => is_any_link(elem),
            // `:visited`——静态永不匹配（隐私安全，防历史探测）。
            PseudoClass::Visited => false,
            // `:fullscreen`/`:modal`——运行时 top-layer 状态（JS requestFullscreen/showModal），
            // 静态解析的 DOM 不可知 → 永不匹配（matches_full=false，非延后——line 1604 早返）。
            PseudoClass::Fullscreen | PseudoClass::Modal => false,
            // `:focus`/`:focus-visible`/`:focus-within`——运行时焦点状态（JS .focus()/用户交互），
            // 静态解析的 DOM 不可知（焦点 NodeId 由 engine shim 追踪，re-parse 不携带）→ 永不匹配
            //（matches_full=false，非延后——line 1604 早返）。真保真须 engine-shim 改动（defer）。
            PseudoClass::Focus | PseudoClass::FocusVisible | PseudoClass::FocusWithin => false,
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
    // R124：ASCII-only trim（内部同源——`str::trim` 的 Unicode 空白集会剥掉单个
    // Unicode 空白字符类名的字符本体，见 trim_ascii_ws 注记）。
    let trimmed = trim_ascii_ws(selector);
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
        parts.push(parse_simple_selector(trim_ascii_ws(seg))?);
        if idx + 1 < tokens.len() {
            combinators.push(*comb);
        }
    }

    if parts.len() == 1 {
        combinators.clear();
    }

    Some(SelectorChain { parts, combinators })
}

/// 顶层逗号拆分选择器列表（spec `querySelector(All)` 接受逗号分隔列表——
/// `input, button`；旧实现把整串当单选择器解析失败 → 空结果）。忽略 `[]`/`()`
/// 内逗号（属性值 / `:not(a,b)`）。返回 `None` 表示无顶层逗号（单选择器）。
/// R156（js-dom M4）：选择器**有效性**判定（spec `dom-element-matches` /
/// `querySelector` 对非法选择器抛 SyntaxError DOMException——WPT Element-matches
/// invalidSelectors 簇）。与 `Document::matches`（best-effort 返 false）不同，
/// 本函数区分「解析失败」与「无匹配」。词法预检（裸括号/花括号/`<`、`[...]`/`(...)`
/// 配对、`ns|` 命名空间前缀、`::` 伪元素、逗号列表边界、组合器首尾）+ 结构
/// parse 双层：预检拒词法非法形态（parse 对 `]` 等裸符号容错不判非法），
/// parse 拒组合器/伪类结构错误。
pub fn selector_is_valid(selector: &str) -> bool {
    let trimmed = trim_ascii_ws(selector);
    if trimmed.is_empty() {
        return false;
    }
    if !selector_lexically_valid(trimmed) {
        return false;
    }
    match split_top_level_selector_list(trimmed) {
        Some(parts) => parts.iter().all(|p| parse_selector_chain(p).is_some()),
        None => parse_selector_chain(trimmed).is_some(),
    }
}

/// R156：词法层预检——裸 `(`/`)`/`{`/`}`/`<`、未配对 `[`/`]`/`(`/`)`、
/// `ns|` 命名空间前缀（未声明 ns 的选择器在本引擎一律非法——WPT Undeclared
/// namespace 簇）、`::` 伪元素（选择器匹配语义不支持）、空段逗号列表
///（`div,` / `,div`）、顶层组合器开头（`>*`）。
/// R156：`[` 后的属性名段字节（到首个关系符号 `=` 族或 `]` 止，去除首尾空白）。
fn attr_name_segment(bytes: &[u8], open: usize) -> &[u8] {
    let mut j = open + 1;
    let end = bytes.len();
    while j < end {
        match bytes[j] {
            b'=' => break,
            b'~' | b'|' | b'^' | b'$' | b'*' if bytes.get(j + 1) == Some(&b'=') => break,
            b']' => break,
            _ => j += 1,
        }
    }
    let mut seg = &bytes[open + 1..j];
    while let Some((&f, rest)) = seg.split_first() {
        if f == b' ' || f == b'\t' || f == b'\n' || f == b'\r' {
            seg = rest;
        } else {
            break;
        }
    }
    while let Some((&l, rest)) = seg.split_last() {
        if l == b' ' || l == b'\t' || l == b'\n' || l == b'\r' {
            seg = rest;
        } else {
            break;
        }
    }
    seg
}

/// R156：`[...]` 内 `=`/`~=`/`|=`/`^=`/`$=`/`*=` 之后的未引用值不得含空白。
fn attr_values_wellformed(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'[' {
            // 找匹配的 ]
            let mut j = i + 1;
            let mut depth = 1;
            while j < b.len() && depth > 0 {
                if b[j] == b'[' {
                    depth += 1;
                } else if b[j] == b']' {
                    depth -= 1;
                }
                j += 1;
            }
            let seg = &b[i + 1..j.saturating_sub(1)];
            // 段内找首个关系符号
            let mut k = 0usize;
            while k < seg.len() {
                let c = seg[k];
                if c == b'=' || (matches!(c, b'~' | b'|' | b'^' | b'$' | b'*') && seg.get(k + 1) == Some(&b'=')) {
                    // 值起点（跳过符号）
                    let vs = if c == b'=' { k + 1 } else { k + 2 };
                    let mut m = vs;
                    while m < seg.len() && (seg[m] == b' ' || seg[m] == b'\t') {
                        m += 1;
                    }
                    if m >= seg.len() {
                        return false; // `[a=]` 空值
                    }
                    // R157：**substring 族运算符**（~= ^= $= *= |=）的 unquoted 值
                    // 内部空白不拒绝——WPT Selectors-API 用例（`[class*= banana ]`，
                    // expect 命中）对 op/value 两端空白宽容，匹配端 trim 后语义等价。
                    // 精确 `=`（Exact）保持严格（`[class= space unquoted ]` 在
                    // invalidSelectors 列表——unquoted 精确值含空白非法）。
                    if c == b'=' && seg[m] != b'"' && seg[m] != b'\'' {
                        let mut e = m;
                        while e < seg.len() && seg[e] != b']' {
                            if seg[e] == b' ' || seg[e] == b'\t' {
                                return false;
                            }
                            e += 1;
                        }
                    }
                    break;
                }
                k += 1;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    true
}

fn selector_lexically_valid(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut bracket = 0i32;
    let mut paren = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        let idx = i;
        // R157：CSS 转义对（`\x`——结构字符的转义形态 `\[`, `\]`, `\ `, `\.`）
        // 整对跳过，不参与结构判定（`.test\.foo\[5\]bar` 的 `\[` 不是 attr 括号；
        // hex 长转义 `\e9 ` 的空白终结符不构成词边界误判——按 `\`+1char 保守近似，
        // hex 尾随空白仅理论歧义，WPT 形态全覆盖）。
        if b == b'\\' {
            i += 2;
            continue;
        }
        i += 1;
        // R157：`[...]` 属性上下文内的 `|`（`|=` 运算符）与 `.`/`#`（unquoted 值
        // 字符，如 `.example.`）是合法形态——ns 前缀/class ident 校验只对元素
        // 选择器段生效（bracket > 0 跳过这三类；`:` 伪类段保持全域校验——
        // `[a=b]:not(.x)` 的 `::` 判定不受 attr 影响）。
        let in_attr = bracket > 0;
        match b {
            b'[' => {
                bracket += 1;
                // R156：属性名段校验（`[*=test]` 裸 `*=`、`[*|*=test]` 的 `*|*` 局部
                // 名 `*` 全非法）。名段（`[` 到首个关系符号）形态：`name` / `*|name` /
                // `ns|name`（ns 本引擎无声明，仅 `*|` 与空前缀合法——由 `|` 分支判）。
                // 局部 name 不得为 `*` 且须 ident 首字符（字母/_/-/转义/非 ASCII）。
                let name = attr_name_segment(bytes, idx);
                if name.is_empty() {
                    return false;
                }
                let nb = name;
                let local = match nb.iter().position(|&c| c == b'|') {
                    Some(pos) => &nb[pos + 1..],
                    None => nb,
                };
                let first = local.first().copied().unwrap_or(0);
                if local.is_empty()
                    || first == b'*'
                    || !(first == b'-'
                        || first == b'_'
                        || first == b'\\'
                        || first.is_ascii_alphabetic()
                        || first >= 0x80)
                {
                    return false;
                }
            }
            b']' => {
                bracket -= 1;
                if bracket < 0 {
                    return false;
                }
            }
            b'(' => paren += 1,
            b')' => {
                paren -= 1;
                if paren < 0 {
                    return false;
                }
            }
            // `{`/`}`/`<` 在选择器语法中无位置（块级语法 / legacy `>` 裸形态）
            b'{' | b'}' | b'<' => return false,
            // R156：`%` 非法组合器（WPT Invalid combinator `div % address`）。
            b'%' => return false,
            b'>' | b'+' | b'~' => {
                // R156：连续显式组合器（`div ++ address` / `div ~~ address`——两符号
                // 间无段字符）非法。跳过其间空白看前一字节是否同类符号。
                let mut j = idx;
                while j > 0 {
                    match bytes[j - 1] {
                        b' ' | b'\t' | b'\n' | b'\r' => j -= 1,
                        b'>' | b'+' | b'~' => return false,
                        _ => break,
                    }
                }
            }
            // R156：`.`/`#` 后必须跟 ident 首字符（字母/下划线/转义/非 ASCII——
            // `.5cm` 数字开头、`..test`/`.foo..quux` 连点、`.bar.` 尾点、裸 `.`/
            // `#` 全非法；括号内（`:not(.5cm)` 等）同规）。`-` 开头 ident 允许。
            b'.' | b'#' if !in_attr => {
                let nxt = bytes.get(idx + 1).copied().unwrap_or(0);
                // R158：U+000B（VT）按 R124 语义算**字面类名字符**（ASCII 空白分词
                // 域不含 VT；`querySelector('.'+VT)` 查询合法返 null 不抛）。
                let ident_start = nxt == b'-'
                    || nxt == b'_'
                    || nxt == b'\\'
                    || nxt == 0x0B
                    || nxt.is_ascii_alphabetic()
                    || nxt >= 0x80;
                if !ident_start {
                    return false;
                }
            }
            // `ns|type` 前缀 / `::` 伪元素（idx>0 防 `||` 误判——本引擎无 column 组合器）
            b'|' if !in_attr => {
                // R172：`|*`/`|div` 段首裸形态（idx==0）= 显式空 ns 前缀，合法
                //（`||` 仍是非法 column 组合器）。
                if idx > 0 && bytes[idx - 1] == b'|' {
                    return false;
                }
                // `|=` / `~=` 等运算符形态由 attr 分支处理；此分支只管元素段的
                // ns 前缀（`ns|div`）。
                // R156：ns 前缀选择器（`ns|div` / `^|div` / `$|div`）——本引擎未实现
                // namespace 声明（@namespace / Parses NS 变量），任何「| 前有内容」的
                // 形态一律非法（`*|div` 的任意 ns 与 `|div` 的显式空 ns 除外——
                // 前缀位是 `*` 或段首）。WPT Undeclared namespace / Invalid namespace 簇。
                // R172（js-dom M4）：`|` 的**显式空前缀**（`|div`/`|*`——`|` 是段内
                // 首字符）在**任意段位置**合法（spec selectors-4：默认 ns 声明缺省时
                // `|name` = 显式无 ns）——`#id |div` 的后代段同样合法。段首判定
                // 放宽：`|` 前回溯（跳空白/组合器符号）到段边界即放行；「| 前有
                // **非空白内容**」（`ns|div`）仍非法。
                let p = if idx == 0 { b' ' } else { bytes[idx - 1] };
                // R172：`|` 前是空白/组合器符号（`#id |div` 的后代段首）→ 显式空
                // ns 前缀，放行（回溯确认 | 与段边界之间**只有空白/组合器**——
                // `ns|div` 的 | 前是 ident 字符，回溯立即停在非空白处 j<idx 且
                // bytes[j..idx] 非全空白/符号 → 仍拒）。
                let is_segment_start = p == b' '
                    || p == b'\t'
                    || p == b'\n'
                    || p == b'\r'
                    || p == b'>'
                    || p == b'+'
                    || p == b'~'
                    || p == b',';
                if p != b'*' && !is_segment_start {
                    return false;
                }
            }
            // `::` 伪元素（idx+1 是第二冒号）——选择器匹配语义不支持
            // R159：`::` 伪元素不再词法拒绝——WPT 期望 `#x::before` 等**合法但
            // 匹配零元素**（DOM querySelector 不匹配伪元素——spec selectors-
            // matching 伪元素永不命中）。match 侧 PseudoElement 匹配恒 false；
            // `::` 后必须跟合法伪元素名（空名在 parse 层拒）。
            b':' => {}
            _ => {}
        }
    }
    // R157：**尾部未闭合 `[` 宽容**（WPT validSelectors 的 `#a [align="center"`
    // expect 命中——Selectors-API 层浏览器自动补 `]`；只有 `]` 多余（负 bracket，
    // 已在循环内拒）或 `(` 不配对仍非法。`[` 尾余 = 段尾截断，parse 端 find(']')
    // 返 None → 整链 None……须同步在 parse 端补 `]`（见 parse_simple_selector 的
    // R157 补尾），此处只做词法放行。
    // R159：**伪类参数内**的尾部未闭合 `(` 宽容（`::slotted(foo`——WPT 期望合法
    // 零匹配；`(` 后有 `:name(` 形态即伪参上下文）。顶层裸 `(` / `)` 仍非法
    //（WPT invalidSelectors）。parse 端 find(')') miss 时 args 取到串尾。
    if bracket < 0 || paren < 0 {
        return false;
    }
    if paren > 0 {
        // `:name(` 伪参上下文判定：最后一个 `:` 之后存在 `(`。无 `:`（顶层裸括号
        // 形态）→ 不合法（WPT invalidSelectors 的 `(`）。
        let paren_ctx_ok = s.rfind(':').map(|c| s[c..].contains('(')).unwrap_or(false);
        if !paren_ctx_ok {
            return false;
        }
    }
    // R156：`[attr= unquoted value ]` 未引用属性值含内部空白非法（spec 属性值
    // 未引用形态不允许空白；引用形态 `"a b"` 合法）。逐 `[...]` 段扫：`=` 后
    //（无引号包裹）遇空白即非法。
    if !attr_values_wellformed(s) {
        return false;
    }
    // 顶层逗号列表：空段非法（`div,` / `,a` / `a,,b`）。
    if let Some(parts) = split_top_level_selector_list(s) {
        for p in parts {
            if trim_ascii_ws(p).is_empty() {
                return false;
            }
        }
    }
    // 顶层以组合器符号开头（`>*` / `+a` / `~a`）——相对选择器形态，matches 不接受。
    let first = s.bytes().next().unwrap();
    if matches!(first, b'>' | b'+' | b'~' | b',') {
        return false;
    }
    true
}

pub(crate) fn split_top_level_selector_list(s: &str) -> Option<Vec<&str>> {
    let mut out: Vec<&str> = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut found = false;
    let bytes = s.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        let b = bytes[idx];
        // R158：转义对整对跳过（`\,` 是字面逗号——WPT escapes `#\.\,\:\!` 的
        // 逐字符转义 id；旧版在此切列表 → 残段以 `\` 开头 parse None → 整链误判非法）。
        if b == b'\\' {
            idx += 2;
            continue;
        }
        match b {
            b'(' | b'[' => depth += 1,
            b')' | b']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            b',' if depth == 0 => {
                out.push(s[start..idx].trim());
                start = idx + 1;
                found = true;
            }
            _ => {}
        }
        idx += 1;
    }
    if !found {
        return None;
    }
    out.push(s[start..].trim());
    Some(out)
}

/// 扫描选择器，按组合器边界（`>`/`+`/`~`/空白，忽略 `()`/`[]` 内）切分为
/// `(段文本, 指向下一段的组合器)` 序列。末段的组合器无意义（填占位 `Descendant`，调用方不读）。
/// 多个连续边界压缩为显式符号优先（`>`/`+`/`~` 优先于空白后代）。
/// R156：符号串起点——从符号字节向前跳过紧邻空白（` +` 的起点是空白字节）。
/// 切分左段时用（左段 = 段首..符号串起点，不含空白与符号）。
fn seg_char_run_start(bytes: &[u8], sym_idx: usize) -> usize {
    let mut j = sym_idx;
    while j > 0 {
        match bytes[j - 1] {
            b' ' | b'\t' | b'\n' | b'\r' => j -= 1,
            _ => break,
        }
    }
    j
}

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
    // R156（js-dom M4）：待定显式符号的**起始字节 idx**（符号可能带前导空白——
    // `#a1 +div` 的符号串是 " +"，起点是空白）。边界推送用符号起点切分，
    // 保证左段不含符号本体（旧版用段首 idx 切分，`#a1+div` 的左段切出 "#a1+"——
    // id 标识吞 `+`，无空格组合器全形态 miss，WPT Element-matches sibling 簇根源）。
    let mut pending_explicit_idx: usize = 0;

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
                    pending_explicit_idx = seg_char_run_start(bytes, i);
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
                    boundaries.push((pending_explicit_idx, pending_explicit.unwrap()));
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
    // R156：边界 idx = 符号串（含前导空白）起点；段起点 = 从边界向后跳过符号/空白，
    // 用 max 保证单调（连续符号 `a > > b` 等病态形态下边界序可追上段起点——
    // 旧版 slice [seg_start..b_idx] 可 start>end panic；现空段以 "" 占位 →
    // parse_simple_selector 返 None → 整链拒绝（非法选择器语义，不 panic）。
    let mut out = Vec::new();
    let mut seg_start = 0usize;
    for (b_idx, comb) in &boundaries {
        if *b_idx >= len {
            break;
        }
        let end = (*b_idx).max(seg_start);
        out.push((trim_ascii_ws(&s[seg_start..end]), *comb));
        let mut j = *b_idx;
        while j < len {
            match bytes[j] {
                b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'+' | b'~' => j += 1,
                _ => break,
            }
        }
        seg_start = seg_start.max(j);
    }
    out.push((trim_ascii_ws(&s[seg_start..]), Combinator::Descendant));
    out
}

/// 解析 `:nth-child(an+b)` 的 `an+b` 参数 → `(a, b)`。
///
/// 支持：`odd`→(2,1)、`even`→(2,0)、纯整数 `5`→(0,5)、`n`→(1,0)、`2n`→(2,0)、
/// `2n+1`→(2,1)、`-n+3`→(-1,3)、`n+2`→(1,2)。无法解析 → `None`。
pub fn parse_nth(arg: &str) -> Option<Nth> {
    // R162：nth 公式内空白全剥（spec CSS microgrammar 允许 an+b 记号间任意
    // ASCII whitespace——WPT `2n \t\r\n+ \t\r\n4` 形态；trim 只去两端，
    // `+` 与数之间的内部空白致 parse fail → None → 整选择器非法）。
    let s: String = arg
        .chars()
        .filter(|c| !matches!(c, ' ' | '\t' | '\r' | '\n' | '\u{c}'))
        .collect();
    let s = s.as_str();
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

/// R157（js-dom M4）：CSS 字符串/标识符**规范转义反解**（CSS Syntax §4.3.2
/// "consume an escaped code point"）：`\` + 1–6 个十六进制数字 + 可选单个空白
/// = 码点（`\e9`→`é`、`\0000e9 `→`é`）；`\` + 非十六进制字符 = 字面字符
///（`\.`→`.`）。无效十六进制 → U+FFFD。供属性值反解（WPT Element-matches /
/// ParentNode-querySelector 的 `[data-attr-value="\e9"]` 簇——旧版无反解，
/// 转义值按原串比对全 miss）。
pub(crate) fn unescape_css_string(s: &str) -> String {
    if !s.contains('\\') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        // \ + 十六进制序列（最多 6 位）
        let mut hex = String::new();
        while hex.len() < 6 {
            match chars.peek() {
                Some(&c) if c.is_ascii_hexdigit() => {
                    hex.push(c);
                    chars.next();
                }
                _ => break,
            }
        }
        if !hex.is_empty() {
            let cp = u32::from_str_radix(&hex, 16).unwrap_or(0xFFFD);
            out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
            // 可选单个消费空白（码点转义终结符——`\e9 x` 的空格被吃掉，`\e9  x` 留一个）
            if matches!(
                chars.peek(),
                Some(' ') | Some('\t') | Some('\n') | Some('\r') | Some('\u{c}')
            ) {
                chars.next();
            }
        } else if let Some(&c) = chars.peek() {
            // \ + 非十六进制字符 = 字面字符（\n 不换行——CSS 转义的 n 是字面 'n'）
            out.push(c);
            chars.next();
        }
        // 行尾裸 \ 由 CSS 字符串层处理（选择器值内不出现），此处忽略
    }
    out
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
        // R159：伪元素族——合法解析 + 恒不匹配（matches 侧 false；WPT
        // `#pseudo-element:before` expect [] 不抛）。`::slotted(foo`（未闭合括号）
        // `::slotted(foo`（未闭合括号）同为合法形态——args 原样吞（匹配恒 false
        // 无需解析）。
        "before" | "after" | "first-line" | "first-letter" | "slotted" | "selection" | "marker" | "placeholder"
        | "backdrop" => Some(PseudoClass::PseudoElement),
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
        // `:blank`——值空或纯空白的文本输入控件（CSS UI L4 / Selectors L4 §12）。无参，延后至
        // Document::is_blank_element 复评（须读 textarea 子文本）。
        "blank" => Some(PseudoClass::Blank),
        // `:fullscreen`/`:modal`——运行时 top-layer 状态（JS requestFullscreen/showModal 激活），
        // 静态解析不可知 → 识别为合法伪类但静态永不匹配（matches_full 内 false，镜像 :visited）。
        // 识别目的：使 `dialog:not(:modal)` 等复合选择器不被当无效（静默返空），与 CSS matcher 一致。
        "fullscreen" => Some(PseudoClass::Fullscreen),
        "modal" => Some(PseudoClass::Modal),
        // `:focus`/`:focus-visible`/`:focus-within`——运行时焦点状态（JS .focus() / 用户交互激活），
        // 静态解析不可知 → 识别为合法伪类但静态永不匹配（matches_full 内 false，镜像 :visited/:fullscreen）。
        // 识别目的：使 `input:not(:focus)` 等复合选择器不被当无效（静默返空），与 CSS matcher 一致。
        "focus" => Some(PseudoClass::Focus),
        "focus-visible" => Some(PseudoClass::FocusVisible),
        "focus-within" => Some(PseudoClass::FocusWithin),
        _ => None, // 未识别伪类（:hover/:focus 等）→ 视为不匹配该 compound（保守）
    }
}
/// R3254-L7：扫描到第一个**未转义**分隔符的位置（反斜杠后的字符跳过——CSS 转义序列，与生成端 `stable_selector_for_node` 成对）。
fn find_unescaped_delim(s: &str, delims: &[char]) -> Option<usize> {
    let mut chars = s.char_indices();
    while let Some((index, ch)) = chars.next() {
        if ch == '\\' {
            chars.next(); // 跳过被转义字符
        } else if delims.contains(&ch) {
            return Some(index);
        }
    }
    None
}

/// R3254-L7：去掉 CSS 转义前缀（`\x` → `x`）——与生成端 `escape_css_ident` 成对。
/// js-dom M4 R124：选择器串首尾空白裁剪——仅 **ASCII whitespace**（CSS 语法域空白；
/// Rust `str::trim` 是 Unicode 空白集，会把 `.\u{000B}` / `.\u{00A0}` 这类「单个
/// Unicode 空白字符类名」选择器的类名字符本身裁掉——WPT
/// getElementsByClassName-whitespace 19F 簇根因之一）。
pub fn trim_ascii_ws(s: &str) -> &str {
    s.trim_matches(|c: char| matches!(c, ' ' | '\t' | '\n' | '\u{000C}' | '\r'))
}

pub(crate) fn unescape_css_ident(s: &str) -> String {
    // R157：与字符串值同源的规范反解（十六进制码点转义）——`#t\\e9` 命中 id "té"
    //（spec CSS Syntax escaped code point；旧逐字符版把 `\\e9` 反解成 "e9"）。
    // wire 侧 escape_css_ident 只转义非 [a-zA-Z0-9_-] 字符（均非 ASCII 十六进制
    // 数字），hex 升级对既有 wire 对（`\.`→`.`）向后兼容。
    unescape_css_string(s)
}

/// 解析单个简单选择器。
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
    let s = trim_ascii_ws(selector);
    if s.is_empty() {
        return None;
    }

    let mut result = SimpleSelector {
        tag: None,
        ns_kind: NsKind::Default,
        id: None,
        classes: Vec::new(),
        attribute: None,
        pseudos: Vec::new(),
    };

    let mut rest = s;

    // 解析标签名（开头的连续非特殊字符）
    // R59：`*|name`（任意 ns）/ `|name`（显式空 ns）前缀剥离 + NsKind 记账
    //（WPT Namespace selector 簇；`ns|name` 有名前缀已在词法层拒）。
    fn strip_ns(rest: &str) -> (&str, NsKind) {
        if let Some(r) = rest.strip_prefix("*|") {
            (r, NsKind::AnyNs)
        } else if let Some(r) = rest.strip_prefix('|') {
            (r, NsKind::EmptyNs)
        } else {
            (rest, NsKind::Default)
        }
    }
    if let Some(pos) = rest.find(['#', '.', '[', ':']) {
        if pos > 0 {
            let (raw, kind) = strip_ns(&rest[..pos]);
            result.ns_kind = kind;
            if !raw.is_empty() {
                result.tag = Some(raw.to_string());
            }
        } else {
            // 段首即特殊字符——ns 前缀可能在 `*|` 后（`*|div` 的 find 命中 0 因 `*` 非
            // 特殊字符——实际此分支只进 `|div` 形态：`|` 非特殊字符集，段首特殊字符
            // 只有 #.[: —— `|div` 走 else 分支。此处保空 tag + Default。
        }
        rest = &rest[pos..];
    } else {
        let (raw, kind) = strip_ns(rest);
        result.ns_kind = kind;
        if !raw.is_empty() {
            result.tag = Some(raw.to_string());
        }
        return Some(result);
    }

    // 解析后续的选择器部分
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('#') {
            // ID 选择器（R3254-L7：段边界跳过转义序列，段内容 unescape）
            let end = find_unescaped_delim(r, &['.', '[', ':']).unwrap_or(r.len());
            if end == 0 {
                return None; // 空的 ID 选择器
            }
            result.id = Some(unescape_css_ident(&r[..end]));
            rest = &r[end..];
        } else if let Some(r) = rest.strip_prefix('.') {
            // 类选择器（R3254-L7：同上）
            let end = find_unescaped_delim(r, &['#', '.', '[', ':']).unwrap_or(r.len());
            if end == 0 {
                return None; // 空的类选择器
            }
            result.classes.push(unescape_css_ident(&r[..end]));
            rest = &r[end..];
        } else if let Some(r) = rest.strip_prefix(':') {
            // 伪类：名字直到 `(` 或下一个分隔符；`:nth-child(...)` 含括号参数。
            // R159：`::` 双冒号伪元素语法（`::before`/`::slotted(foo)`——WPT 期望
            // 合法但零匹配）——剥前导第二冒号后按伪元素名解析。
            let r = r.strip_prefix(':').unwrap_or(r);
            let (name, args, next_rest): (&str, Option<&str>, &str) = match r.find('(') {
                Some(open) => {
                    // `)` 相对 r[open..] 的偏移 → 换算到 r 的绝对位置。
                    // R159：未闭合 `)` 宽容——args 取到串尾（`::slotted(foo` WPT
                    // 期望合法零匹配；伪元素匹配恒 false 无需完整 args）。
                    match r[open..].find(')') {
                        Some(close) => {
                            let arg_end = open + close;
                            let name = &r[..open];
                            let args = &r[open + 1..arg_end];
                            (name, Some(args), &r[arg_end + 1..])
                        }
                        None => (&r[..open], Some(&r[open + 1..]), ""),
                    }
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
            // R157：尾部未闭合 `[` 宽容（WPT validSelectors `#a [align="center"`
            // expect 命中——Selectors-API 浏览器自动补 `]`；词法层已放行尾余）。
            let end_bracket = match r.find(']') {
                Some(pos) => pos,
                // R157：截断段（前导 `[` 已 strip，`r` 内无 `]`）宽容到串尾——
                // Selectors-API 浏览器自动补 `]` 语义。
                None => r.len(),
            };
            let attr_content = &r[..end_bracket];

            // 属性运算符检测：两字符运算符（~= ^= $= *= |=）须先于单字符 `=` 检测，
            // 否则 `[attr^=v]` 的 `=` 会先命中单字符分支。值去引号（`[a="v"]`→`v`）。
            // 返回 (运算符, name, value)——运算符为 None 表示 `[attr]` 仅存在。
            let attr_sel = if let Some((op, name, value)) = parse_attr_operator(attr_content) {
                // R157：`*|` any-ns 前缀剥离（WPT `[TiTlE]` case-insensitive / `[*|TiTlE]`
                // 形态——本引擎属性无 ns 域，`*|name` 等价 name；ns|name 前缀已在
                // 词法层拒）。
                let name = name.trim().strip_prefix("*|").unwrap_or(name.trim());
                let name = unescape_css_string(name);
                // R157：值反解 CSS 转义（先去引号——转义在引号内层）
                let value = unescape_css_string(&strip_attr_quotes(value.trim()));
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
                    name: unescape_css_string(attr_content.trim().strip_prefix("*|").unwrap_or(attr_content.trim())),
                    matcher: AttributeMatcher::Exists,
                }
            };

            result.attribute = Some(attr_sel);
            // R157：截断段（end_bracket = r.len()）rest 归空；正常路径跳过 `]`。
            rest = if end_bracket < r.len() {
                &r[end_bracket + 1..]
            } else {
                ""
            };
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    // R174（js-dom M4）：no-ns 标记属性还原协议——`apply_empty_ns_markers` 把带
    // `data-zw-empty-ns` 的元素 ns 置空并剔标记属性，使 `|div`（NsKind::EmptyNs）
    // 在 shim 序列化 → re-parse 往返后仍可命中（WPT ParentNode ns 簇的
    // createElementNS("", "div") 产物）。HTML 解析产物 ns 恒 HTMLNS → 不标记
    // 的元素不受影响（零回归护栏）。
    #[test]
    fn zz_r174_empty_ns_marker_roundtrip() {
        let mut doc =
            crate::parse_html("<body><div id=\"plain\"></div><div id=\"nons\" data-zw-empty-ns=\"\"></div></body>");
        doc.apply_empty_ns_markers();
        let root = doc.root();
        // 标记元素：ns 置空 + 标记属性剔除
        let hits = doc.query_selector_all(root, "|div");
        assert_eq!(hits.len(), 1, "only the marked element should match |div");
        let got = doc.get(hits[0]).unwrap();
        let crate::NodeKind::Element(e) = &got.kind else {
            panic!("not an element");
        };
        assert_eq!(e.id.as_deref(), Some("nons"));
        assert!(
            !e.attributes.iter().any(|a| a.name.local.as_ref() == "data-zw-empty-ns"),
            "marker attribute must be stripped"
        );
        // 未标记元素：ns 保持 HTMLNS（`|div` 不命中，普通 `div` 命中）
        let plain = doc.query_selector_all(root, "div");
        assert_eq!(plain.len(), 2, "plain tag selector matches both");
    }

    // R172（js-dom M4）：ns 组合形态的 validity 回归——`|div`/`|*`（显式空 ns）
    // 与 `*|div`（任意 ns）在**任意段位置**合法（含 `#id |div` 后代段）；`ns|div`
    // 有名前缀仍非法（无 @namespace 声明表）。WPT ParentNode-querySelector-All
    // Namespace selector 簇（旧词法层把 `#id |div` 的段首 `|` 误判 ns 前缀拒）。
    #[test]
    fn zz_r172_ns_forms_validity() {
        for s in [
            "#no-namespace |*",
            "#no-namespace |div",
            "#any-namespace *|div",
            "|*",
            "|div",
            "*|div",
            "div |p",
            "#x > |div",
        ] {
            assert!(selector_is_valid(s), "should be valid: {s}");
            assert!(parse_selector_chain(s).is_some(), "should parse: {s}");
        }
        for bad in ["ns|div", "a b|c", "||div"] {
            assert!(!selector_is_valid(bad), "should be invalid: {bad}");
        }
    }

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

    /// R3254-L7：转义 round-trip——`id="a.b"` 生成 `#a\.b`，解析回 `a.b` 且不被
    /// `.` 截断成 id=a + class=b。
    #[test]
    fn test_parse_escaped_id_and_class() {
        let sel = parse_simple_selector(r#"#a\.b"#).unwrap();
        assert_eq!(sel.id.as_deref(), Some("a.b"));
        assert!(sel.classes.is_empty());

        let sel = parse_simple_selector(r#"div.x\:y"#).unwrap();
        assert_eq!(sel.tag.as_deref(), Some("div"));
        assert_eq!(sel.classes, vec!["x:y"]);

        let sel = parse_simple_selector(r#"#a\ b"#).unwrap();
        assert_eq!(sel.id.as_deref(), Some("a b"));

        // 转义不破坏普通选择器。
        let sel = parse_simple_selector("#plain").unwrap();
        assert_eq!(sel.id.as_deref(), Some("plain"));
        assert_eq!(unescape_css_ident("plain"), "plain");
        assert_eq!(unescape_css_ident(r#"a\.b"#), "a.b");
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

    /// R3300：`:blank` parse_pseudo 识别（不再落 `_ => None` 致整选择器无效）。
    #[test]
    fn test_parse_pseudo_blank_r3300() {
        let sel = parse_simple_selector(":blank").expect(":blank 应解析为合法伪类");
        assert_eq!(sel.pseudos.len(), 1);
        assert!(matches!(sel.pseudos[0], PseudoClass::Blank));
        // 复合 `input:blank` 应整体有效（此前 :blank 落 None 致返 None）。
        let comp = parse_simple_selector("input:blank").expect("input:blank 应解析成功");
        assert_eq!(comp.pseudos.len(), 1);
        assert_eq!(comp.tag.as_deref(), Some("input"));
    }

    /// R3301：`:fullscreen`/`:modal` parse_pseudo 识别（不再落 `_ => None` 致复合选择器无效）。
    #[test]
    fn test_parse_pseudo_fullscreen_modal_r3301() {
        let fs = parse_simple_selector(":fullscreen").expect(":fullscreen 应解析为合法伪类");
        assert_eq!(fs.pseudos.len(), 1);
        assert!(matches!(fs.pseudos[0], PseudoClass::Fullscreen));
        let modal = parse_simple_selector(":modal").expect(":modal 应解析为合法伪类");
        assert_eq!(modal.pseudos.len(), 1);
        assert!(matches!(modal.pseudos[0], PseudoClass::Modal));
        // 复合 `dialog:not(:modal)` 应整体有效（此前 :modal 落 None 致整 :not() 解析失败）。
        // :not 内嵌经 parse_simple_selector——:modal 识别后 :not(:modal) 应解析为合法 Not。
        let not_modal = parse_simple_selector(":not(:modal)").expect(":not(:modal) 应解析成功");
        assert_eq!(not_modal.pseudos.len(), 1);
        assert!(matches!(&not_modal.pseudos[0], PseudoClass::Not(_)));
    }

    /// R3302：`:focus`/`:focus-visible`/`:focus-within` parse_pseudo 识别（不再落 `_ => None` 致复合选择器无效）。
    #[test]
    fn test_parse_pseudo_focus_family_r3302() {
        for (sel_str, expect) in [
            (":focus", "focus"),
            (":focus-visible", "focus-visible"),
            (":focus-within", "focus-within"),
        ] {
            let sel = parse_simple_selector(sel_str).unwrap_or_else(|| panic!("{sel_str} 应解析为合法伪类"));
            assert_eq!(sel.pseudos.len(), 1, "{sel_str} 应含 1 伪类");
            let ok = match expect {
                "focus" => matches!(sel.pseudos[0], PseudoClass::Focus),
                "focus-visible" => matches!(sel.pseudos[0], PseudoClass::FocusVisible),
                "focus-within" => matches!(sel.pseudos[0], PseudoClass::FocusWithin),
                _ => false,
            };
            assert!(ok, "{sel_str} 应解析为 {expect} 变体");
        }
        // 复合 `input:not(:focus)` 应整体有效（此前 :focus 落 None 致整 :not() 解析失败）。
        let not_focus = parse_simple_selector("input:not(:focus)").expect("input:not(:focus) 应解析成功");
        assert_eq!(not_focus.tag.as_deref(), Some("input"));
        assert_eq!(not_focus.pseudos.len(), 1);
        assert!(matches!(&not_focus.pseudos[0], PseudoClass::Not(_)));
    }
    #[test]
    fn test_ascii_whitespace_class_tokenization_r124() {
        let mut doc = Document::new();
        let span = doc.create_element("span");
        doc.set_attribute(span, "class", "\u{00A0}");
        let elem = doc.get(span).unwrap();
        if let crate::NodeKind::Element(e) = &elem.kind {
            assert_eq!(
                e.class_list,
                vec!["\u{00A0}".to_string()],
                "U+00A0 是字面类名字符（非分隔符）"
            );
        }
        doc.set_attribute(span, "class", "a\u{2003}b c");
        let elem2 = doc.get(span).unwrap();
        if let crate::NodeKind::Element(e) = &elem2.kind {
            // U+2003（EM SPACE）在类名内部是字面字符 → 与 'a' 连成一个 token 'a\u{2003}b'；
            // ASCII space 分隔出 'c'。
            assert_eq!(e.class_list.len(), 2);
            assert_eq!(e.class_list[0], "a\u{2003}b");
            assert_eq!(e.class_list[1], "c");
        }
        // 选择器 trim：'.<U+000B>' 的类名字符不被剥（Rust str::trim 会剥成 '.'）。
        assert_eq!(
            parse_simple_selector(".\u{000B}").unwrap().classes,
            vec!["\u{000B}".to_string()]
        );
        assert_eq!(trim_ascii_ws("  .a  "), ".a");
        assert_eq!(trim_ascii_ws("\u{00A0}.a"), "\u{00A0}.a", "U+00A0 非裁剪集");
    }
}

// js-dom M4 R124：class 域 ASCII whitespace 语义（spec html-infrastructure「ascii
// whitespace」——分隔符仅 space/\t/\n/\f/\r，U+00A0/U+2000 系/U+3000 等是字面类名字符）。
// WPT dom/nodes/getElementsByClassName-whitespace-class-names.html 19F 簇驱动：
// `<span class="&#x00A0;">` 的 class 是合法单字符类名，gEBCN/NBSP) 须命中该 span。

#[cfg(test)]
mod zz_r156_tests {
    use crate::parser::parse_html;

    // R156（js-dom M4）：无空格组合器回归（`#a1+div` 旧把左段切成 "#a1+" —— id 标识
    // 吞符号，全形态 miss；WPT Element-matches sibling 簇 + no-refNodes 大簇根源）。
    #[test]
    fn zz_r156_invalid_forms_rejected() {
        let invalid = [
            "",
            "[",
            "]",
            "(",
            ")",
            "{",
            "}",
            "<",
            ">",
            "#",
            "div,",
            ".",
            ".5cm",
            "..test",
            ".foo..quux",
            ".bar.",
            "div % address, p",
            "div ++ address, p",
            "div ~~ address, p",
            "[*=test]",
            "[*|*=test]",
            "[class= space unquoted ]",
            "div:example",
            ":example",
            "div:linkexample",
            "div::example",
            "::example",
            ":::before",
            ":: before",
            "ns|div",
            ":not(ns|div)",
            "^|div",
            "$|div",
            ">*",
        ];
        for s in invalid {
            assert!(!crate::query::selector_is_valid(s), "should be INVALID: {s:?}");
        }
        let valid = ["div", "#a1+div", ".x.y", "[a=b]", "* ", "a , b", "div:not(.x)"];
        for s in valid {
            assert!(crate::query::selector_is_valid(s), "should be VALID: {s:?}");
        }
    }

    // R157（js-dom M4）：属性值 CSS 转义反解回归（`[data-x="t\\e9"]`——`\\e9` 是
    // 十六进制码点转义 = "é"；旧版无反解按原串比对全 miss，WPT Element-matches /
    // ParentNode-querySelector 的 escaped value 簇根源）。运算符形态（~=/|=/^=/$=/*=
    // /quoted/unquoted）同步覆盖。
    #[test]
    fn zz_r157_attr_escaped_values() {
        let html = "<body><div id=\"d1\" title=\"a b c\" lang=\"en-US\" data-x=\"té\" data-y=\"é x\"></div><a id=\"a1\" href=\"https://example.com/x?y=z\"></a></body>";
        let doc = crate::parse_html(html);
        let root = doc.root();
        // 转义值（引号内）：\e9 → é
        assert_eq!(doc.query_selector_all(root, "[data-x=\"t\\e9\"]").len(), 1);
        // 码点转义 + 空白终结符（\0000e9 后第一个空格被吃掉；"é x" 需双空格）
        assert_eq!(doc.query_selector_all(root, "[data-y=\"\\0000e9  x\"]").len(), 1);
        // \ + 非十六进制 = 字面字符（\e9 之外的形态：\\2e = '.'）
        assert_eq!(
            doc.query_selector_all(root, "[data-x=\"t\\e9\"],[href*=example]").len(),
            2
        );
        // 运算符族形态回归
        assert_eq!(doc.query_selector_all(root, "[title~=b]").len(), 1);
        assert_eq!(doc.query_selector_all(root, "[title~=\"b\"]").len(), 1);
        assert_eq!(doc.query_selector_all(root, "[lang|=en]").len(), 1);
        assert_eq!(doc.query_selector_all(root, "[href*=example]").len(), 1);
        assert_eq!(doc.query_selector_all(root, "[href^=\"https:\"]").len(), 1);
        assert_eq!(doc.query_selector_all(root, "[href$=z]").len(), 1);
        assert_eq!(doc.query_selector_all(root, "[data-x=\"té\"]").len(), 1);
    }

    // R158：转义 id 形态不误杀——`#,\,\:\!`（逐字符转义）合法。
    #[test]
    fn zz_r158_escaped_id_valid() {
        // R158：转义 id 形态不误杀（engine r118 / WPT escapes 族——`#\.comma`、
        // `#\30 nextIsWhiteSpace`（hex+空白终结符）、逐字符转义混合形态）。
        for s in [
            "#\\.comma",
            "#\\.\\,\\:\\!",
            "#\\30 nextIsWhiteSpace",
            "#\\000030 spaceMoreThan6Hex",
            "#\\61 BMPRegular",
            "#spac\\65\r\ns",
            "#hel\\6C o",
        ] {
            assert!(crate::query::selector_is_valid(s), "valid: {s:?}");
        }
    }

    // R158：invalid 形态在词法层的再验证——`[` 裸括号（自动补 `]` 后空名）须拒。
    #[test]
    fn zz_r158_bare_bracket_rejected() {
        for s in ["[", "]", "(", ")"] {
            assert!(!crate::query::selector_is_valid(s), "invalid: {s:?}");
        }
        assert!(crate::query::selector_is_valid("#a [align=\"center\""));
    }

    // R160：`:empty` spec 语义（注释/PI 子节点不影响空判定；文本/元素子节点非空）
    // + `:target` fragment 判定（parse_html_element_json_with_url 的 Rust 侧
    // 基础——doc.set_url 后 is_target_element 命中 id 元素）。
    #[test]
    fn zz_r160_empty_semantics() {
        let html = "<body><div id=\"pe\"><p id=\"p1\"></p><p id=\"p2\"><!-- c --></p><p id=\"p3\"> </p><p id=\"p4\">T</p><span id=\"s1\"></span></div></body>";
        let doc = crate::parse_html(html);
        let root = doc.root();
        let ids: Vec<String> = doc
            .query_selector_all(root, "#pe :empty")
            .into_iter()
            .filter_map(|id| doc.get_attribute(id, "id"))
            .collect();
        assert_eq!(
            ids,
            vec!["p1".to_string(), "p2".to_string(), "s1".to_string()],
            ":empty = no children or comments only"
        );
        // :target 无 URL → 无命中
        assert_eq!(doc.query_selector_all(root, ":target").len(), 0);
    }

    // R159：伪元素（合法但零匹配）+ ns type selector（`*|div` 任意 / `|div` 空 ns）回归。
    #[test]
    fn zz_r159_pseudo_element_and_ns() {
        let html = "<body><div id=\"a\" class=\"x\"><p id=\"p1\">t</p></div></body>";
        let doc = crate::parse_html(html);
        let root = doc.root();
        // 伪元素：合法解析 + 恒零匹配（一/二冒号 + slotted 未闭合括号）
        assert!(crate::query::selector_is_valid("#a::before"));
        assert!(crate::query::selector_is_valid("#a:before"));
        assert!(crate::query::selector_is_valid("::slotted(foo"));
        assert_eq!(doc.query_selector_all(root, "#a::before").len(), 0);
        assert_eq!(doc.query_selector_all(root, "#a:before").len(), 0);
        assert_eq!(doc.query_selector_all(root, "::slotted(foo").len(), 0);
        // `*|div` 任意 ns：HTML 元素命中（HTMLNS 非空亦任意）
        assert_eq!(doc.query_selector_all(root, "*|div").len(), 1);
        assert_eq!(doc.query_selector_all(root, "*|p").len(), 1);
        // `|div` 显式空 ns：HTML 解析产物 namespace 是 HTMLNS → 不命中
        assert_eq!(doc.query_selector_all(root, "|div").len(), 0);
        // `ns|div` 有名前缀仍非法（无 @namespace 声明表）
        assert!(!crate::query::selector_is_valid("ns|div"));
    }

    // R157：`*` universal 回归——`<body>` 片段 querySelectorAll("*") 全命中。
    #[test]
    fn zz_r157_universal_star() {
        let html = "<body><div id=\"universal\"><p id=\"p1\">x</p></div></body>";
        let doc = crate::parse_html(html);
        let root = doc.root();
        // `<body>` 片段解析补全 html/body 等（html5ever 容错）——计数用包含式断言
        let star_hits = doc.query_selector_all(root, "*").len();
        assert!(star_hits >= 4, "universal star hits {} >= 4", star_hits);
        assert_eq!(doc.query_selector_all(root, "div *").len(), 1, "descendant star");
        assert_eq!(doc.query_selector_all(root, "#universal *").len(), 1, "scoped star");
    }

    #[test]
    fn zz_r157_unclosed_and_star_pipe() {
        let html = "<body><div id=\"a\" title=\"t\"><div id=\"a1\" align=\"center\"></div></div></body>";
        let doc = crate::parse_html(html);
        let root = doc.root();
        assert!(crate::query::selector_is_valid("#a [align=\"center\""));
        assert_eq!(
            doc.query_selector_all(root, "#a [align=\"center\"").len(),
            1,
            "unclosed auto-close"
        );
        assert_eq!(doc.query_selector_all(root, "[*|TiTlE]").len(), 1, "star-pipe");
        assert_eq!(doc.query_selector_all(root, "[TiTlE]").len(), 1, "case-insensitive");
    }

    #[test]
    fn zz_r157_lexical_false_positives() {
        // R157：词法校验误杀回归——`|=` 运算符（attr 内 `|` 非命名空间前缀）与
        // attr 值内的 `.`（`.example.` 是合法 unquoted 值）。
        let valid = [
            "[lang|=\"fr\"]",
            "#a[lang|=en]",
            "#attr-contains a[href*=\".example.\"]",
            "[href*=example]",
            "[class~=foo]",
            "[lang|=en-US]",
        ];
        for s in valid {
            assert!(crate::query::selector_is_valid(s), "should be VALID: {s:?}");
        }
    }

    #[test]
    fn zz_r156_fuzz_no_panic() {
        // R156：符号切分不 panic 回归（旧 pending 符号起点可早于段起点 → slice 越界）。
        let sels = [
            "a+b",
            "a+ b",
            "a +b",
            "a + b",
            "a~b",
            "a>b",
            "a > b",
            "a> b",
            "a >b",
            ".x+.y",
            "#i+p",
            "*+*",
            "a+b>c",
            "a>b~c+d",
            ":not(a)+b",
            ":not(a+invalid)>b",
            "[x=a b]+c",
            "[x=+]+c",
            "a,, +b",
            "+a",
            "a+",
            "a >",
            " ~ ",
            "a~ ~b",
            "a+ +b",
            "a > > b",
            "p:not(#a1)+div",
            "div:not(.x)~p",
        ];
        let doc = parse_html(
            "<body><div id=\"adjacent\"><div id=\"a1\" class=\"x\"></div><div id=\"a2\" class=\"x\"></div><p id=\"p3\"></p></div></body>",
        );
        let root = doc.root();
        for s in sels {
            let _ = doc.query_selector_all(root, s);
            let _ = doc.query_selector(root, s);
        }
    }

    #[test]
    fn zz_r156_sibling_combinators_no_space() {
        let html = "<body><div id=\"adjacent\"><div id=\"a1\" class=\"x\"></div><div id=\"a2\" class=\"x\"></div><p id=\"p3\"></p></div></body>";
        let doc = crate::parse_html(html);
        let root = doc.root();
        assert_eq!(doc.query_selector_all(root, "#a1+div").len(), 1, "v1 id+tag nospace");
        assert_eq!(doc.query_selector_all(root, "#a2+p").len(), 1, "v1b cross-tag");
        assert_eq!(doc.query_selector_all(root, ".x+p").len(), 1, "v2 class+tag nospace");
        assert_eq!(doc.query_selector_all(root, "div+p").len(), 1, "v3 tag+tag nospace");
        assert_eq!(doc.query_selector_all(root, "#a1~p").len(), 1, "v4 tilde id");
        assert_eq!(doc.query_selector_all(root, "div~p").len(), 1, "v5 tilde tag");
        assert_eq!(
            doc.query_selector_all(root, "#a1 + div").len(),
            1,
            "v6 spaced still works"
        );
        assert_eq!(
            doc.query_selector_all(root, "#adjacent>div").len(),
            2,
            "v7 child nospace"
        );
        assert_eq!(
            doc.query_selector_all(root, "#a1+div, #nope").len(),
            1,
            "v8 selector list"
        );
        assert_eq!(
            doc.query_selector_all(root, "#a1:not(#nope)+div").len(),
            1,
            "v9 pseudo then combinator"
        );
    }
}
