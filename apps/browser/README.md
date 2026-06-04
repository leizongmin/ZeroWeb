# ZeroBrowser (`zero-browser`)

> ZeroWeb 浏览器应用入口

## 概述

`ZeroBrowser` 是 ZeroWeb 的桌面浏览器应用。基于 `zero-browser-shell`（浏览器 UI 数据模型）和 `zero-webview`（渲染核心）构建，通过 `zero-host-runtime`（窗口管理）在 macOS、Linux、Windows 上提供完整的浏览器体验。

## 主要功能

- **多标签页管理** — 创建、关闭、切换、拖拽排序标签页
- **地址栏** — URL 输入、自动补全（基于历史和书签搜索）
- **导航控制** — 前进、后退、刷新、主页
- **收藏夹** — 书签/文件夹增删改查、收藏栏展示
- **历史记录** — 页面访问记录、搜索、清除
- **下载管理器** — 下载文件、进度显示
- **页面查找** — Ctrl+F 搜索高亮
- **缩放** — Ctrl+/Ctrl- 缩放页面、重置
- **右键上下文菜单** — 5 种场景的默认菜单项
- **设置页面** — 搜索引擎、主页、隐私设置
- **键盘快捷键** — L（地址栏）、T（新标签页）、W（关闭标签页）、R（刷新）等

## 运行

```bash
cargo run --bin zero-browser
```

## 架构

```
zero-browser
├── app.rs          — BrowserApp 主循环（连接 Shell + WebView + HostRuntime）
├── app_render.rs   — GPU 工具栏渲染（标签栏、地址栏、导航按钮）
├── main.rs         — 应用入口
└── pages.rs        — 内置页面（设置页等）
```

## 依赖关系

```
zero-browser
├── zero-browser-shell   — UI 数据模型（Tab/Bookmarks/History/Settings）
├── zero-webview         — WebView 渲染核心
├── zero-host-runtime    — 窗口和事件循环（winit）
└── zero-render-foundation — GPU 渲染基础设施
```
