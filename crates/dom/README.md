# ZeroWeb DOM (`zero-dom`)

> 基于 html5ever 的 DOM 树实现，提供完整的节点类型、树操作和增量更新能力。

## 概述

`ZeroWeb DOM` (`zero-dom`) 是 ZeroWeb 渲染管线的第一步 —— 将 HTML 文本解析为结构化的 DOM 树。它基于 html5ever 构建解析器，使用 slotmap 存储节点以实现 O(1) 查找与稳定 ID，同时提供符合 WHATWG DOM 规范的树操作 API、CSS 选择器查询、HTML 序列化以及 MutationObserver 变更追踪。

## 主要功能

- **HTML 解析** — 通过 html5ever 解析完整 HTML5 文档和片段，遵循 WHATWG 错误恢复规范
- **完整节点类型** — Document、Element、Text、Comment、DocumentType、DocumentFragment、ProcessingInstruction
- **树操作 API** — append_child、remove_child、insert_before、replace_child、clone_node，含循环检测
- **属性操作** — get/set/remove/has attribute，自动维护 id 和 class 缓存索引
- **元素查询** — getElementById（O(1)）、getElementsByTagName、getElementsByClassName、querySelector/querySelectorAll（支持标签、ID、类名、属性选择器）
- **HTML 序列化** — 将 DOM 节点序列化为 HTML 字符串，支持 innerHTML / outerHTML
- **MutationObserver** — 追踪 childList、attributes、characterData 变更并通知观察者
- **文本内容** — 递归获取或设置节点的 textContent

## 使用示例

```rust
use zero_dom::{parse_html, Document};

// 解析 HTML 字符串
let doc = parse_html("<!DOCTYPE html><html><body><h1 id='title'>Hello</h1><p class='intro'>World</p></body></html>");

// 通过 ID 查找元素（O(1)）
let h1 = doc.get_element_by_id("title").unwrap();
let text = doc.text_content(h1);
assert_eq!(text.as_deref(), Some("Hello"));

// 通过标签名和类名查找
let ps = doc.get_elements_by_tag_name("p");
assert_eq!(ps.len(), 1);

let intros = doc.get_elements_by_class_name("intro");
assert_eq!(intros.len(), 1);

// 手动构建 DOM 树
let mut doc = Document::new();
let html = doc.create_element("html");
let body = doc.create_element("body");
let p = doc.create_element("p");
let text = doc.create_text_node("Hello, ZeroWeb!");

doc.append_child(doc.root(), html).unwrap();
doc.append_child(html, body).unwrap();
doc.append_child(body, p).unwrap();
doc.append_child(p, text).unwrap();

// 序列化为 HTML
assert_eq!(doc.inner_html(body), "<p>Hello, ZeroWeb!</p>");
```
