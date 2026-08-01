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
    /// @property 规则（CSS Properties and Values API）。
    Property(PropertyRule),
    /// @counter-style 规则（CSS Counter Styles 3 §3）。driving: R2392。
    CounterStyle(CounterStyleRule),
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
/// 解析 `size` 描述符为像素 `(width, height)` 与 `margin` 描述符为 `(top, right, bottom, left)`
/// 像素（1-4 值简写，同 CSS margin）；命名页 / 页边距盒为后续切片。`size`/`margin` 用于
/// Print 媒体分页的页几何（覆盖默认 A4 + 0 边距）。
#[derive(Debug, Clone)]
pub struct PageRule {
    /// 解析后的页尺寸（px），`None` = `size` 缺失或无效（调用方回退默认 A4）。
    pub size: Option<(f32, f32)>,
    /// 解析后的页边距 `(top, right, bottom, left)` px，`None` = `margin` 缺失或无效（回退 0）。
    /// R2011 P4-followup：垂直边距驱动分页内容区（水平边距待 layout-width-for-print 切片）。
    pub margin: Option<(f32, f32, f32, f32)>,
}

/// CSS `@property` 规则（CSS Properties and Values API Level 1）。
///
/// 格式：`@property --foo { syntax: "<color>"; inherits: false; initial-value: #c0ffee; }`
/// 注册自定义属性 `--foo`，给定语法、是否继承、初始值。注册后，未显式声明的 `var(--foo)`
/// 解析为 `initial-value`（而非 invalid）；`inherits` 控制该值是否像普通自定义属性一样继承。
///
/// 当前仅消费描述符的原始值（`syntax` 不做值校验/类型强制——按 CSS 规范 `syntax` 为 `*`
/// 时 `initial-value` 可缺省，其余情形须有初值；此处宽容存储，由 style-system 在
/// `var()` 解析时用作兜底默认值）。
#[derive(Debug, Clone)]
pub struct PropertyRule {
    /// 自定义属性名（含 `--` 前缀，如 `--foo`）。
    pub name: String,
    /// `syntax` 描述符原始值（如 `<color>`、`<length>`、`*`）。
    pub syntax: String,
    /// `inherits` 描述符（`true`/`false`）。
    pub inherits: bool,
    /// `initial-value` 描述符原始值；`None` = 缺省（仅 `syntax: "*"` 时合法）。
    pub initial_value: Option<String>,
}

/// `@counter-style` 计数系统算法（CSS Counter Styles 3 §3.1.4）。
/// driving: R2392（slice 1 实现 cyclic/fixed/symbolic/alphabetic/numeric）。
#[derive(Debug, Clone, PartialEq)]
pub enum CounterSystem {
    /// `cyclic`：循环遍历 symbols（i-1 % len）。
    Cyclic,
    /// `fixed [N]`：固定序列，N 为首符号值（默认 1）；超出范围走 fallback。
    Fixed(Option<i32>),
    /// `symbolic`：重复 symbol（i 个 × 对应 symbol）。
    Symbolic,
    /// `alphabetic`：位置制（无零位），如 a-z/aa-zz。
    Alphabetic,
    /// `numeric`：位置制（含零位），如 0-9/00-99。
    Numeric,
    /// `additive`：加法表（Roman 式）；slice 1 仅 parse 保留，应用 defer。
    Additive,
    /// `extends <name>`：继承内置/已定义样式；slice 1 仅 parse 保留，应用 defer。
    Extends(String),
}

/// @counter-style 规则（CSS Counter Styles 3 §3）。
///
/// 格式：`@counter-style <name> { system: cyclic; symbols: "a" "b"; suffix: ") "; }`
/// 解析 `system`/`symbols`/`additive-symbols`/`prefix`/`suffix`/`fallback`/`range` 描述符
/// 为类型化字段；`negative`/`pad`/`speak-as` 描述符 slice 2 仍忽略（应用 defer）。
/// 非法规则（无名 / 无 system / symbols 不足 / additive 无 additive-symbols）返回 None 由上层丢弃。
#[derive(Debug, Clone)]
pub struct CounterStyleRule {
    /// 计数器样式名（`list-style-type` 引用键）。
    pub name: String,
    /// `system` 描述符（缺省 `symbolic`）。
    pub system: CounterSystem,
    /// `symbols` 描述符（已逐个去引号/空白切分）。
    pub symbols: Vec<String>,
    /// `additive-symbols` 描述符（`<integer> && <symbol>` 对，已按 weight 降序排序）。
    /// driving: R2394 slice 2（additive 系统算法所需）。
    pub additive_symbols: Vec<(i32, String)>,
    /// `prefix` 描述符（缺省 `""`）。
    pub prefix: String,
    /// `suffix` 描述符（缺省 `". "`，period + space；`""` 显式置空）。
    pub suffix: String,
    /// `fallback` 描述符（缺省 `"decimal"`）。
    pub fallback: String,
    /// `range` 描述符（`[lower upper]` 对列表，`infinite`→i32::{MIN,MAX}）。
    /// `None` = 缺省（按系统默认 range；slice 2 仅应用此显式 range）。
    /// driving: R2394 slice 2（extends + range 越界 fallback 所需）。
    pub range: Option<Vec<(i32, i32)>>,
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
    /// CSS 嵌套选择器 `&`（CSS Nesting Module Level 1）。
    ///
    /// 作为子类选择器标记存放（而非 `TypeSelector` 变体），因为 `&` 可与类型选择器
    /// 共存于同一复合选择器（如 `div&`、`&.cls`），而 `type_selector` 是单值 `Option`。
    /// `&` 仅出现在**未编译**的嵌套选择器中；解析阶段会被 compile 算法替换为父级
    /// 选择器化合物（见 `parser::compile_style_rule`），编译后的规则不含 `Nesting`。
    /// specificity 贡献 0（与父级合并后特异性自然正确），matcher 兜底按「匹配任意」处理。
    Nesting,
}

/// 属性选择器值的大小写修饰符（CSS Selectors Level 4 §6.3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttrCaseModifier {
    /// 缺省修饰符：按文档语言默认决定大小写敏感性（HTML 不敏感、XML/XHTML 敏感）。
    #[default]
    Default,
    /// `[attr=val s]`：强制大小写敏感，覆盖文档语言默认。
    Sensitive,
    /// `[attr=val i]`：强制 ASCII 大小写不敏感，覆盖文档语言默认。
    Insensitive,
}

/// 属性选择器。
#[derive(Debug, Clone)]
pub struct AttributeSelector {
    /// 属性名。
    pub name: String,
    /// 匹配操作。
    pub matcher: AttributeMatcher,
    /// Selectors Level 4 大小写修饰符（缺省 / `i` / `s`）。
    pub case: AttrCaseModifier,
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
    /// `:nth-child(an+b of S)` 选择器（Selectors L4，`of` 选择器列表过滤计数的兄弟）。
    NthChildOf(NthPattern, Vec<Selector>),
    /// `:nth-last-child(an+b of S)` 选择器（Selectors L4）。
    NthLastChildOf(NthPattern, Vec<Selector>),
    /// `:nth-of-type()` 选择器。
    NthOfType(NthPattern),
    /// `:nth-last-of-type()` 选择器。
    NthLastOfType(NthPattern),
    /// `:lang()` 选择器（CSS Selectors L4 §14：逗号分隔语言范围列表，支持 BCP 47 通配符 `*`）。
    Lang(Vec<String>),
    /// `:dir()` 选择器（参数为 `ltr`/`rtl`，归一化为小写）。
    Dir(String),
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
    /// 通用括号：`( <any-value> )` 非合法 condition/feature 时为 general-enclosed，
    /// 恒求值为 false（CSS Conditional §7）。如 `(@page)`、`()`。
    GeneralEnclosed(String),
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
