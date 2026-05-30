# zero-browser-shell

> 浏览器应用层 — 提供多标签页、收藏夹、地址栏、历史记录等浏览器 UI 功能

## 概述

`zero-browser-shell` 是 ZeroBrowser 的应用层 crate，负责构建浏览器的用户界面交互。它基于 `zero-webview-api` 提供的稳定嵌入式 API 和 `zero-host-runtime` 提供的窗口与事件循环能力，实现多标签页管理、收藏夹、地址栏、历史记录等核心浏览器 Shell 功能。

## 主要功能

- 多标签页管理 — 创建、切换、关闭标签页
- 收藏夹 — 添加、删除、组织书签
- 地址栏 — URL 输入、自动补全、导航触发
- 历史记录 — 浏览历史存储与检索
- 浏览器 UI 组件 — 整合各交互元素为完整的浏览器外壳

## 使用示例

```rust
use zero_browser_shell::BrowserShell;
use zero_webview_api::WebViewBuilder;

// 创建浏览器 Shell 实例
let mut shell = BrowserShell::new();

// 创建新标签页并导航
shell.new_tab("https://example.com");

// 添加到收藏夹
shell.add_bookmark("https://example.com", "Example Site");

// 切换标签页
shell.switch_tab(0);
```
