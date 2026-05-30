# Spec: M3 — CSS 解析器 + 样式系统

**版本**: v1.0
**日期**: 2026-05-30
**作者**: AI Assistant
**状态**: Confirmed

---

## 1. 背景与目标

### 1.1 背景

M2 完成了 DOM 树实现，HTML 文档可以解析为 DOM 节点。M3 是 ZeroBrowser 项目中最大的技术创新点：完全自建 CSS 解析器和样式系统。由于所有成熟的 Rust CSS 解析库（rust-cssparser、lightningcss）均采用 MPL 许可证，与项目闭源商业目标冲突，必须从零构建。

CSS 解析器需要处理 CSS 语法的全部复杂性：tokenizer、选择器解析、属性值解析、@规则、嵌套规则等。样式系统需要实现级联算法（specificity、!important、继承、@layer）和计算值生成。

### 1.2 目标

- 构建生产可用的 CSS tokenizer 和 parser
- 支持完整的选择器语法和 CSS 属性解析
- 实现样式系统的级联、继承和计算值
- 与 M2 的 DOM 树集成

### 1.3 范围边界

**包含**:
- CSS tokenizer（完整的 CSS 词法分析）
- CSS parser（样式表 → AST）
- 选择器解析（类型、类、ID、属性、伪类、伪元素、组合器、`:is()`/`:where()`/`:not()`）
- 属性值解析（支持所有 Tier 1 CSS 属性）
- `@media`、`@supports`、`@layer`、`@import` 规则
- 级联算法（specificity、!important、来源排序、@layer）
- 继承和初始值
- 计算值生成
- 自定义属性（`--*`）声明和引用
- 与 DOM 集成的样式计算
- ≥80 个单元测试，覆盖率 ≥ 70%
- ≥4 个 criterion 基准测试

**明确排除**:
- CSS 嵌套语法（CSS Nesting，后续扩展）
- Container Queries（Tier 2）
- `:has()` 选择器（Tier 2）
- Houdini API
- CSS 动画关键帧插值（仅解析，不实现动画引擎）

---

## 2. 需求类型概览

| 类型 | 适用 | 来源 |
|------|------|------|
| 功能需求 | 是 | 第 3 节 |
| 非功能需求 | 是 | 第 4 节 |
| 接口需求 | 是 | 第 5 节 |

---

## 3. 功能需求

### FR-M3-001: CSS Tokenizer
- **描述**: `css-parser` crate **必须**实现完整的 CSS 词法分析器
- **验收标准**:
  - [ ] 支持 CSS 规范定义的所有 token 类型
  - [ ] 正确处理字符串（含转义）、URL、数字（含单位）、标识符
  - [ ] 正确处理注释（`/* */`）
  - [ ] 支持 `@` 关键字、`#` 颜色/ID、各种运算符和分隔符
  - [ ] 错误恢复：遇到非法 token 不 panic
- **优先级**: Must

### FR-M3-002: CSS Parser
- **描述**: `css-parser` crate **必须**实现 CSS 语法解析器，将 token 流转换为 AST
- **验收标准**:
  - [ ] 解析 CSS 样式表为 AST（规则列表）
  - [ ] 解析样式规则（选择器列表 + 声明块）
  - [ ] 解析 @规则（`@media`、`@supports`、`@layer`、`@import`）
  - [ ] 解析声明（属性名 + 属性值）
  - [ ] 错误恢复遵循 CSS 规范（跳过错误规则继续解析）
- **优先级**: Must

### FR-M3-003: 选择器解析
- **描述**: `css-parser` crate **必须**支持完整的选择器语法解析
- **验收标准**:
  - [ ] 类型选择器（`div`）、通配符（`*`）
  - [ ] 类选择器（`.class`）、ID 选择器（`#id`）
  - [ ] 属性选择器（`[attr]`、`[attr=val]`、`[attr~=val]`、`[attr|=val]`、`[attr^=val]`、`[attr$=val]`、`[attr*=val]`）
  - [ ] 伪类（`:hover`、`:active`、`:focus`、`:first-child`、`:last-child`、`:nth-child()`、`:not()`、`:is()`、`:where()`、`:root`、`:empty`）
  - [ ] 伪元素（`::before`、`::after`、`::first-line`、`::first-letter`）
  - [ ] 组合器（后代 ` `、子 `>`、相邻兄弟 `+`、通用兄弟 `~`）
  - [ ] 选择器列表（逗号分隔）
- **优先级**: Must

### FR-M3-004: CSS 属性值解析
- **描述**: `css-parser` crate **必须**支持 Tier 1 CSS 属性的值解析
- **验收标准**:
  - [ ] 长度/百分比（px、em、rem、%、vh、vw、vmin、vmax）
  - [ ] 颜色（命名颜色、#hex、rgb/rgba、hsl/hsla）
  - [ ] 数字和整数
  - [ ] 字体属性值（font-family、font-size、font-weight、font-style）
  - [ ] display 值（block、inline、flex、grid、inline-block、none）
  - [ ] flexbox 属性值（flex-direction、flex-wrap、justify-content、align-items 等）
  - [ ] grid 属性值（grid-template、grid-area 等）
  - [ ] position 值（static、relative、absolute、fixed、sticky）
  - [ ] transform 函数
  - [ ] transition 值
  - [ ] 自定义属性值（`var(--name)`、`var(--name, fallback)`）
- **优先级**: Must

### FR-M3-005: 样式系统 — 级联
- **描述**: `style-system` crate **必须**实现 CSS 级联算法
- **验收标准**:
  - [ ] Specificity 计算（ID > 类/属性/伪类 > 类型/伪元素）
  - [ ] `!important` 声明优先于普通声明
  - [ ] 来源排序（user agent < user < author）
  - [ ] `@layer` 层级排序
  - [ ] 出现顺序（相同 specificity 时后声明优先）
- **优先级**: Must

### FR-M3-006: 样式系统 — 继承与计算值
- **描述**: `style-system` crate **必须**实现属性继承和计算值生成
- **验收标准**:
  - [ ] 继承属性正确传递（font、color、line-height 等）
  - [ ] 初始值正确设置（每个属性有明确的初始值）
  - [ ] 计算值生成（相对值→绝对值转换）
  - [ ] `inherit`、`initial`、`unset`、`revert` 关键字
- **优先级**: Must

### FR-M3-007: 自定义属性
- **描述**: 支持 CSS 自定义属性（CSS Variables）
- **验收标准**:
  - [ ] `--name: value` 声明
  - [ ] `var(--name)` 引用
  - [ ] `var(--name, fallback)` 回退值
  - [ ] 自定义属性继承
- **优先级**: Must

### FR-M3-008: DOM 集成
- **描述**: 样式系统**必须**与 DOM 树集成
- **验收标准**:
  - [ ] 给定 DOM 树 + CSS 样式表，可以为每个元素计算样式
  - [ ] 选择器匹配正确（使用 M2 的 DOM 树遍历）
- **优先级**: Must

---

## 4. 非功能需求

### NFR-M3-001: 性能 — CSS 解析吞吐量
- **描述**: CSS 解析 100KB CSS 文件**应当**在合理时间内完成
- **测量**: criterion 基准测试
- **优先级**: Should

### NFR-M3-002: 性能 — 选择器匹配
- **描述**: 1000 元素 vs 100 选择器匹配**应当**在合理时间内完成
- **测量**: criterion 基准测试
- **优先级**: Should

### NFR-M3-003: 代码质量
- **描述**: `cargo build` + `cargo clippy` 零警告，`cargo test` 全通过
- **优先级**: Must

### NFR-M3-004: 测试覆盖率
- **描述**: css-parser 和 style-system 覆盖率 ≥ 70%
- **优先级**: Must

---

## 5. 技术设计（RFC）

### 5.1 架构

```
CSS 文本 → Tokenizer → Token 流 → Parser → CSS AST
                                                    ↓
DOM 树 + CSS AST → StyleSystem → 选择器匹配 → 级联 → 计算样式
```

### 5.2 核心数据结构

```
┌─ css-parser crate ─────────────────────────────────┐
│  Tokenizer (char stream → Token stream)             │
│  Parser (Token stream → Stylesheet AST)             │
│  AST: Stylesheet, Rule, AtRule, SelectorList,       │
│       Declaration, ComponentValue                   │
│  Selector: SimpleSelector, CompoundSelector,        │
│            ComplexSelector, SelectorList             │
│  Values: Length, Percentage, Color, String, etc.    │
└─────────────────────────────────────────────────────┘
┌─ style-system crate ───────────────────────────────┐
│  CascadeStore (规则存储和索引)                       │
│  Specificity (A, B, C) 三元组                       │
│  Cascade (级联算法)                                  │
│ ComputedStyle (计算后的样式值)                       │
│  PropertyRegistry (属性初始值和继承定义)             │
└─────────────────────────────────────────────────────┘
```

### 5.3 Tokenizer 设计

CSS Tokenizer 基于 CSS Syntax Module Level 3 规范实现。关键 token 类型：

- `Ident(String)` — 标识符
- `AtKeyword(String)` — @关键字
- `Hash(String)` — #值
- `String(String)` — 字符串字面量
- `Url(String)` — URL
- `Number(f64)` — 数字
- `Percentage(f64)` — 百分比
- `Dimension(f64, String)` — 带单位数字
- `Function(String)` — 函数调用
- `Colon`, `Semicolon`, `Comma`, `LBrace`, `RBrace` 等 — 分隔符
- `Comment` — 注释（可选保留）
- `Whitespace` — 空白（可选保留）
- `Error(String)` — 错误 token

### 5.4 AST 设计

```rust
/// CSS 样式表 AST
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

/// CSS 规则
pub enum Rule {
    /// 样式规则（选择器 + 声明块）
    Style(StyleRule),
    /// @规则
    At(AtRule),
}

/// 样式规则
pub struct StyleRule {
    pub selectors: SelectorList,
    pub declarations: Vec<Declaration>,
}

/// 声明
pub struct Declaration {
    pub property: String,
    pub value: Vec<ComponentValue>,
    pub important: bool,
}

/// @规则
pub struct AtRule {
    pub name: String,
    pub prelude: Vec<ComponentValue>,
    pub body: AtRuleBody,
}

/// 组件值
pub enum ComponentValue {
    Token(Token),
    Function(FunctionValue),
    SimpleBlock(BlockValue),
}
```

### 5.5 选择器设计

```rust
/// 选择器列表
pub struct SelectorList(pub Vec<ComplexSelector>);

/// 复杂选择器（含组合器）
pub struct ComplexSelector {
    pub compound: CompoundSelector,
    pub combinator: Option<(Combinator, Box<ComplexSelector>)>,
}

/// 复合选择器（无组合器）
pub struct CompoundSelector {
    pub type_selector: Option<TypeSelector>,
    pub subclass_selectors: Vec<SubclassSelector>,
}

/// 组合器
pub enum Combinator {
    Descendant,    // 空格
    Child,         // >
    NextSibling,   // +
    SubsequentSibling, // ~
}

/// 子类选择器
pub enum SubclassSelector {
    Id(String),
    Class(String),
    Attribute(AttributeSelector),
    PseudoClass(PseudoClassSelector),
    PseudoElement(PseudoElementSelector),
}
```

### 5.6 实施计划

1. 实现 CSS Tokenizer（tokenizer 模块）
2. 实现 CSS Parser 核心（parser 模块）
3. 实现选择器解析（selector 模块）
4. 实现属性值解析（values 模块）
5. 实现 StyleSystem 级联和继承
6. 实现 DOM 集成和样式计算
7. 编写单元测试（≥80 个）
8. 编写基准测试（≥4 个）

### 5.7 测试策略

- **Tokenizer 测试**: 各种 token 类型的边界条件
- **Parser 测试**: 完整 CSS 文件的解析和 AST 验证
- **选择器测试**: 每种选择器类型的解析和匹配
- **级联测试**: specificity 竞争、!important、继承、@layer
- **基准测试**: 解析吞吐量、选择器匹配、样式计算

---

## 6. TBD 清单

| ID | 项目 | 优先级 | 缺失信息 | 后续步骤 |
|----|------|--------|----------|----------|
| TBD-M3-1 | CSS 值类型完整性（所有 CSS 函数的解析范围） | 重要 | 首期聚焦 Tier 1 属性 | 实现时确定 |
| TBD-M3-2 | `@supports` 条件解析深度 | 可选 | 首期支持基础条件 | 实现时确定 |

---

## 7. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-05-30 | 初始版本 — M3 里程碑 Spec + RFC |
