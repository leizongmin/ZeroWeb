# ZeroWeb CSS Parser (`zero-css-parser`)

> 自建 CSS 解析器 — tokenizer + parser，支持完整选择器和属性解析，不依赖任何 MPL 许可的 CSS 解析库。

## 概述

`ZeroWeb CSS Parser` (`zero-css-parser`) 是 ZeroWeb 渲染管线的第一阶段，负责将 CSS 文本转换为结构化的 AST。它包含一个基于 CSS Syntax Module Level 3 规范的词法分析器（Tokenizer）和一个递归下降语法解析器（Parser），能解析样式规则、@规则、完整的选择器语法以及常见的 CSS 属性值类型。该 crate 为下游的 `zero-style-system` 提供解析后的结构化数据。

## 主要功能

- **词法分析（Tokenizer）** — 将 CSS 字符流转换为 token 流，支持标识符、数字、百分比、带单位数值、字符串、URL、颜色匹配运算符、注释等全部 CSS token 类型
- **语法解析（Parser）** — 将 token 流转换为 AST，支持样式规则（选择器 + 声明块）和 @规则（`@media`、`@import`、`@supports` 等）
- **完整选择器解析** — 类型选择器、通配符、ID、类、属性选择器（7 种匹配模式）、伪类（`:not()`、`:is()`、`:where()`、`:nth-child()`、`:lang()` 等）、伪元素（`::before`、`::after`）、组合器（后代、子元素、相邻兄弟、通用兄弟）
- **选择器特异性计算** — 按 CSS 规范计算 (A, B, C) 三元组，正确处理 `:is()`/`:not()` 取最大值、`:where()` 为零的规则
- **属性值类型化** — 解析颜色（命名、十六进制、rgb/hsl）、长度（px/em/rem/vh/vw 等）、display、position、overflow、flex 布局、字体、`var()` 引用等常见 CSS 属性值

## 使用示例

```rust
use zero_css_parser::Parser;
use zero_css_parser::specificity;

// 解析完整的 CSS 样式表
let stylesheet = Parser::parse_stylesheet(r#"
    body {
        color: #333;
        font-size: 16px;
    }

    .container > .item:hover {
        display: flex;
        background-color: rgba(0, 0, 0, 0.5);
    }

    @media screen and (max-width: 600px) {
        .container {
            flex-direction: column;
        }
    }
"#);

// 遍历解析结果
for rule in &stylesheet.rules {
    println!("{:?}", rule);
}

// 计算选择器特异性
// .container > .item:hover → (0, 3, 0)
let sel = &stylesheet.rules[1];
if let zero_css_parser::Rule::Style(style_rule) = sel {
    let spec = specificity(&style_rule.selectors[0]);
    println!("specificity: ({}, {}, {})", spec.0, spec.1, spec.2);
}
```
