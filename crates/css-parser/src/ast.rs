//! CSS AST 数据结构。
//!
//! 定义 CSS 样式表的抽象语法树结构。

/// CSS 样式表 AST。
#[derive(Debug, Clone)]
pub struct Stylesheet {
    /// 样式表中的规则列表。
    pub rules: Vec<Rule>,
}

/// CSS 规则。
#[derive(Debug, Clone)]
pub enum Rule {
    /// 样式规则（选择器 + 声明块）。
    Style(StyleRule),
    /// @规则。
    At(AtRule),
    /// @keyframes 规则。
    Keyframes(KeyframesRule),
    /// @layer 规则。
    Layer(LayerRule),
    /// @import 规则。
    Import(ImportRule),
    /// @supports 规则。
    Supports(SupportsRule),
    /// @container 规则。
    Container(ContainerRule),
    /// @font-face 规则。
    FontFace(FontFaceRule),
    /// @page 规则（CSS Paged Media）。
    Page(PageRule),
}

/// CSS @import 规则。
///
/// 格式：`@import url("path") media-query;` 或 `@import "path" media-query;`
#[derive(Debug, Clone)]
pub struct ImportRule {
    /// 导入的 URL。
    pub url: String,
    /// 媒体查询列表（可选）。
    pub media_queries: Vec<String>,
}

/// CSS 样式规则。
#[derive(Debug, Clone)]
pub struct StyleRule {
    /// 选择器列表。
    pub selectors: Vec<Selector>,
    /// 声明列表。
    pub declarations: Vec<Declaration>,
}

/// CSS @规则。
#[derive(Debug, Clone)]
pub struct AtRule {
    /// @规则名称（如 `media`、`supports`、`layer`、`import`）。
    pub name: String,
    /// 前导部分（如 `@media screen and (max-width: 600px)` 的条件部分）。
    pub prelude: String,
    /// 规则体。
    pub body: AtRuleBody,
}

/// @规则体。
#[derive(Debug, Clone)]
pub enum AtRuleBody {
    /// 包含规则列表的块（如 `@media` 的花括号内容）。
    Block(Vec<Rule>),
    /// 以分号结束的语句（如 `@import`）。
    Statement,
}

/// CSS @font-face 规则。
///
/// 格式：`@font-face { font-family: "X"; src: url("X.woff") format("woff"); }`
/// 仅提取字体族名与 src 中的 URL 列表（权重/样式等描述符当前忽略，
/// 由调用方按 family 注册到 FontLoader）。
#[derive(Debug, Clone)]
pub struct FontFaceRule {
    /// 字体族名（`font-family` 描述符的值，已去引号）。
    pub family: String,
    /// `src` 描述符中解析出的 URL 列表（按出现顺序，已去 url() 包裹与引号）。
    pub sources: Vec<String>,
}

/// CSS @page 规则（Paged Media）。
///
/// 格式：`@page { size: A4; margin: 2cm; }`（prelude 可为命名页 `:first` 等，当前忽略）。
/// 当前仅解析 `size` 描述符并解析为像素 `(width, height)`；`margin` / 命名页 / 页边距盒
/// 为后续切片。`size` 用于 Print 媒体分页的页尺寸（覆盖默认 A4）。
#[derive(Debug, Clone)]
pub struct PageRule {
    /// 解析后的页尺寸（px），`None` = `size` 缺失或无效（调用方回退默认 A4）。
    pub size: Option<(f32, f32)>,
}

/// @keyframes 规则。
#[derive(Debug, Clone)]
pub struct KeyframesRule {
    /// 动画名称。
    pub name: String,
    /// 关键帧列表。
    pub keyframes: Vec<KeyframeBlock>,
}

/// @layer 规则。
///
/// CSS 级联层，用于控制样式的级联顺序。
/// 格式：`@layer <name> { <rules> }` 或 `@layer <name>;`（声明-only）。
#[derive(Debug, Clone)]
pub struct LayerRule {
    /// 层名称（可能为空字符串表示匿名层）。
    pub name: String,
    /// 层内的规则列表。
    pub rules: Vec<Rule>,
}

/// 单个关键帧块（如 `0% { ... }` 或 `from { ... }`）。
#[derive(Debug, Clone)]
pub struct KeyframeBlock {
    /// 关键帧选择器列表（如 `0%`、`50%`、`100%`、`from`、`to`）。
    pub selectors: Vec<KeyframeSelector>,
    /// 声明列表。
    pub declarations: Vec<Declaration>,
}

/// 关键帧选择器。
#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeSelector {
    /// 百分比（0.0 - 100.0）。
    Percentage(f64),
    /// from（等同 0%）。
    From,
    /// to（等同 100%）。
    To,
}

/// CSS 声明（属性名 + 值）。
#[derive(Debug, Clone)]
pub struct Declaration {
    /// 属性名。
    pub property: String,
    /// 属性值（原始字符串）。
    pub value: String,
    /// 是否为 `!important` 声明。
    pub important: bool,
}

// ── 选择器 ────────────────────────────────────────────────────────────

/// CSS 选择器。
#[derive(Debug, Clone)]
pub struct Selector {
    /// 选择器列表中的单个复杂选择器。
    pub complex: ComplexSelector,
}

/// 复杂选择器（含组合器的选择器链）。
#[derive(Debug, Clone)]
pub struct ComplexSelector {
    /// 复合选择器链：从右到左排列（最右边是最终匹配的目标）。
    /// 每个元素是 (复合选择器, 组合器)，组合器表示与左边元素的关系。
    pub parts: Vec<(CompoundSelector, Option<Combinator>)>,
}

/// 组合器。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// 后代（空格）。
    Descendant,
    /// 子元素（`>`）。
    Child,
    /// 相邻兄弟（`+`）。
    NextSibling,
    /// 通用兄弟（`~`）。
    SubsequentSibling,
}

/// 复合选择器（无组合器的选择器组合）。
#[derive(Debug, Clone)]
pub struct CompoundSelector {
    /// 类型选择器（标签名或 `*`）。
    pub type_selector: Option<TypeSelector>,
    /// 子类选择器列表。
    pub subclass_selectors: Vec<SubclassSelector>,
}

/// 类型选择器。
#[derive(Debug, Clone)]
pub enum TypeSelector {
    /// 标签名（如 `div`、`span`）。
    Tag(String),
    /// 通配符（`*`）。
    Universal,
}

/// 子类选择器。
#[derive(Debug, Clone)]
pub enum SubclassSelector {
    /// ID 选择器（`#id`）。
    Id(String),
    /// 类选择器（`.class`）。
    Class(String),
    /// 属性选择器。
    Attribute(AttributeSelector),
    /// 伪类选择器。
    PseudoClass(PseudoClassSelector),
    /// 伪元素选择器。
    PseudoElement(PseudoElementSelector),
}

/// 属性选择器。
#[derive(Debug, Clone)]
pub struct AttributeSelector {
    /// 属性名。
    pub name: String,
    /// 匹配操作。
    pub matcher: AttributeMatcher,
}

/// 属性匹配操作。
#[derive(Debug, Clone)]
pub enum AttributeMatcher {
    /// 属性存在（`[attr]`）。
    Exists,
    /// 精确匹配（`[attr=val]`）。
    Exact(String),
    /// 空格分隔列表包含（`[attr~=val]`）。
    Includes(String),
    /// 破折号匹配（`[attr|=val]`）。
    DashMatch(String),
    /// 前缀匹配（`[attr^=val]`）。
    Prefix(String),
    /// 后缀匹配（`[attr$=val]`）。
    Suffix(String),
    /// 子串匹配（`[attr*=val]`）。
    Substring(String),
}

/// 伪类选择器。
#[derive(Debug, Clone)]
pub enum PseudoClassSelector {
    /// 简单伪类（无参数，如 `:hover`、`:focus`）。
    Simple(String),
    /// `:not()` 选择器。
    Not(Vec<Selector>),
    /// `:is()` 选择器。
    Is(Vec<Selector>),
    /// `:where()` 选择器。
    Where(Vec<Selector>),
    /// `:has()` 选择器。
    Has(Vec<Selector>),
    /// `:nth-child()` 选择器。
    NthChild(NthPattern),
    /// `:nth-last-child()` 选择器。
    NthLastChild(NthPattern),
    /// `:nth-of-type()` 选择器。
    NthOfType(NthPattern),
    /// `:nth-last-of-type()` 选择器。
    NthLastOfType(NthPattern),
    /// `:lang()` 选择器。
    Lang(String),
}

/// nth 函数的模式（如 `2n+1`、`odd`、`even`、`3`）。
#[derive(Debug, Clone)]
pub struct NthPattern {
    /// 系数 `a`（在 `an+b` 中）。
    pub a: i32,
    /// 偏移 `b`（在 `an+b` 中）。
    pub b: i32,
}

/// 伪元素选择器。
#[derive(Debug, Clone)]
pub enum PseudoElementSelector {
    /// 标准伪元素（如 `::before`、`::after`）。
    Standard(String),
}

// ── @supports ─────────────────────────────────────────────────────────

/// @supports 规则。
///
/// 格式：`@supports (<条件>) { <规则> }`
#[derive(Debug, Clone)]
pub struct SupportsRule {
    /// 条件。
    pub condition: SupportsCondition,
    /// 条件为真时应用的规则列表。
    pub rules: Vec<Rule>,
}

/// @supports 条件。
#[derive(Debug, Clone, PartialEq)]
pub enum SupportsCondition {
    /// 属性值测试：`(property: value)`。
    Property(String, String),
    /// 选择器测试：`selector(<selector>)`。
    Selector(String),
    /// 逻辑与：`<cond1> and <cond2>`。
    And(Vec<SupportsCondition>),
    /// 逻辑或：`<cond1> or <cond2>`。
    Or(Vec<SupportsCondition>),
    /// 逻辑非：`not <cond>`。
    Not(Box<SupportsCondition>),
}

// ── @container ──────────────────────────────────────────────────────

/// @container 规则。
///
/// 格式：`@container <name>? (<条件>) { <规则> }`
#[derive(Debug, Clone)]
pub struct ContainerRule {
    /// 容器名称（可选）。
    pub name: Option<String>,
    /// 容器查询条件。
    pub condition: ContainerCondition,
    /// 条件为真时应用的规则列表。
    pub rules: Vec<Rule>,
}

/// @container 条件。
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerCondition {
    /// 基于尺寸的查询：`size(<条件>)` 或直接 `(<条件>)`。
    Size(ContainerSizeCondition),
    /// 基于 inline-size 的查询：`inline-size(<条件>)`。
    InlineSize(ContainerSizeCondition),
}

/// 容器尺寸条件。
///
/// 支持格式如 `(min-width: 400px)`、`(width > 300px)`、`(max-width: 800px)`。
/// 也支持范围语法如 `(200px <= width <= 500px)`。
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerSizeCondition {
    /// 查询的特征名（如 `min-width`、`width`、`max-width`、`inline-size`、`block-size`）。
    pub feature: String,
    /// 比较值。
    pub value: String,
    /// 比较运算符（如 `>`、`>=`、`<`、`<=`）。为 None 时表示冒号语法（min-width: 400px）。
    pub operator: Option<String>,
    /// 范围查询的下界（如 `200px <= width <= 500px` 中的 `200px`）。
    pub range_min: Option<String>,
    /// 范围查询的上界（如 `200px <= width <= 500px` 中的 `500px`）。
    pub range_max: Option<String>,
}
