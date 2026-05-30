# zero-style-system

> CSS 样式系统 — 实现级联、继承、选择器匹配和计算值生成，为 DOM 元素计算出完整的计算样式。

## 概述

zero-style-system 是 ZeroWeb 渲染管线中的样式计算模块，位于 CSS 解析器（zero-css-parser）和布局引擎（zero-layout-engine）之间。它接收解析后的 CSS 样式表和 DOM 文档，通过选择器匹配、级联优先级排序、属性继承和相对单位转换四个阶段，为每个 DOM 元素生成完整的 `ComputedStyle`。

## 主要功能

- **选择器匹配** — 支持标签、ID、类、属性、伪类（`:first-child`、`:last-child`、`:root`、`:empty`、`:nth-child()`、`:not()`、`:is()`、`:where()`）以及后代、子、相邻兄弟、通用兄弟组合器
- **级联算法** — 按 `!important`、来源（UA / User / Author）、`@layer`、选择器特异性、源码顺序五个维度决定胜出声明
- **属性继承** — 处理 `inherit`、`initial`、`unset`、`revert`、`revert-layer` 全局关键字，以及可继承属性的隐式继承
- **计算值生成** — 将 em、rem、vh、vw、vmin、vmax、ch 等相对单位转换为绝对像素值，支持 `var()` 自定义属性引用和回退值
- **`ComputedStyle` 结构体** — 覆盖盒模型、边框、颜色/背景、字体、文本、Flexbox、定位、Overflow 共 50+ 个 CSS 属性的 typed 字段
- **`PropertyRegistry`** — 提供属性初始值查询、继承性判断、已知属性枚举

## 使用示例

```rust
use zero_style_system::StyleSystem;
use zero_dom::Document;
use zero_css_parser::Stylesheet;

// 创建 DOM 文档和样式系统
let mut doc = Document::new();
let root = doc.root();
let div = doc.create_element("div");
doc.set_attribute(div, "id", "app");
doc.append_child(root, div).unwrap();

let mut sys = StyleSystem::new();
sys.set_viewport(1920.0, 1080.0);

// 传入解析后的样式表，计算所有元素的计算样式
let stylesheets: Vec<Stylesheet> = vec![];
let styles = sys.compute_styles(&doc, &stylesheets);

// 获取某个元素的计算样式
if let Some(style) = styles.get(&div) {
    println!("display: {:?}", style.display);
    println!("color: {:?}", style.color);
}
```
