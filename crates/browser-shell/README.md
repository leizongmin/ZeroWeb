# ZeroBrowser Shell (`zero-browser-shell`)

> 浏览器应用层 — 提供多标签页、收藏夹、地址栏、历史记录等浏览器 UI 功能

## 概述

`ZeroBrowser Shell` (`zero-browser-shell`) 是 `ZeroBrowser` 的应用层 crate，提供 UI-agnostic 的浏览器 shell 数据模型和协调逻辑，可被任何 UI 框架消费。它实现多标签页管理、收藏夹、地址栏、历史记录等核心浏览器 Shell 功能，本身不直接渲染 UI（实际渲染由上层宿主完成）。

## 主要功能

- 多标签页管理 — 创建、切换、关闭标签页
- 收藏夹 — 添加、删除、组织书签
- 地址栏 — URL 输入、自动补全、导航触发
- 历史记录 — 浏览历史存储与检索
- 浏览器 UI 组件 — 整合各交互元素为完整的浏览器外壳

## 使用示例

```rust
use zero_browser_shell::BrowserShell;
use zero_webview::WebViewBuilder;

// 创建浏览器 Shell 实例
let mut shell = BrowserShell::new();

// 创建新标签页并导航，返回标签页 ID
let tab_id = shell.new_tab(Some("https://example.com"));

// 将当前页面添加到收藏夹
shell.add_bookmark();

// 切换标签页
shell.switch_tab(tab_id);
```
