# ZeroWeb Layout Engine (`zero-layout-engine`)

> 基于 Taffy 的布局引擎，支持 Block、Flexbox 和 Grid 布局，将计算样式转换为精确的几何位置

## 概述

`ZeroWeb Layout Engine` (`zero-layout-engine`) 是 ZeroWeb 渲染管线中的布局阶段。它接收 DOM 树和计算样式（ComputedStyle），通过 Taffy 布局算法计算每个元素的位置和大小，输出一棵 `LayoutBox` 树供后续渲染使用。整个流程分为三步：将 `ComputedStyle` 转换为 `taffy::Style`、从 DOM 构建 Taffy 布局树、执行布局计算并提取结果。

## 主要功能

- **多布局模式**：支持 Block、Inline（行内格式化上下文、文本布局与换行）、Flexbox、Grid 布局算法，并含 Table（表格布局与边框合并）、Multicol（多列碎片化与平衡）、margin collapse、float 定位等 CSS 布局特性
- **样式转换层**：完整的 `ComputedStyle` 到 `taffy::Style` 映射，覆盖 display、position、size、margin、padding、border、overflow、flex 属性、对齐方式等
- **布局盒树**：输出结构化的 `LayoutBox` 树，包含位置、尺寸、内容区域偏移、边框、内边距、外边距等完整几何信息
- **定位支持**：支持 static、relative、absolute、fixed 定位模式
- **溢出处理**：支持 visible、hidden、clip、scroll 四种溢出裁剪方式
- **Flex 布局**：支持 flex-direction、flex-wrap、flex-grow、flex-shrink、flex-basis、gap、justify-content、align-items 等属性
- **DOM 树构建**：自动跳过文本节点、注释节点和 `display: none` 的元素，递归构建布局树

## 使用示例

```rust
use std::collections::HashMap;
use zero_layout_engine::LayoutEngine;
use zero_dom::Document;
use zero_style_system::ComputedStyle;
use zero_css_parser::values::{DisplayValue, LengthValue, FlexDirectionValue};

// 创建 DOM
let mut doc = Document::new();
let root = doc.root();
let html = doc.create_element("html");
doc.append_child(root, html).unwrap();
let container = doc.create_element("div");
doc.append_child(html, container).unwrap();
let item1 = doc.create_element("span");
doc.append_child(container, item1).unwrap();
let item2 = doc.create_element("span");
doc.append_child(container, item2).unwrap();

// 设置计算样式
let mut styles = HashMap::new();
let mut container_style = ComputedStyle::default();
container_style.display = DisplayValue::Flex;
container_style.flex_direction = FlexDirectionValue::Row;
container_style.width = LengthValue::Px(300.0);
container_style.height = LengthValue::Px(100.0);
styles.insert(container, container_style);

for id in [item1, item2] {
    let mut item_style = ComputedStyle::default();
    item_style.width = LengthValue::Px(100.0);
    item_style.height = LengthValue::Px(50.0);
    styles.insert(id, item_style);
}

// 执行布局
let mut engine = LayoutEngine::new(800.0, 600.0); // 视口 800x600
let result = engine.compute(&doc, &styles);

// 访问布局结果
println!("视口: {}x{}", result.viewport_width, result.viewport_height);
println!("根节点: {}x{}", result.root.width, result.root.height);
for child in &result.root.children {
    println!("子节点: x={}, y={}, {}x{}", child.x, child.y, child.width, child.height);
}
```
