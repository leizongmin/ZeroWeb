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

浏览器采用固定多进程模型：`zero-browser` 作为宿主进程 spawn 独立 `zero-renderer`（页面渲染与脚本执行）、`zero-compositor`（合成与呈现）和 `zero-image-decoder`（图像解码）子进程，页面帧经 IPC 导入呈现；默认发布版不链接 WebView、脚本 sandbox 或任何 JS 引擎。headless 模式同样通过 renderer IPC 完成导航、脚本与截图。

```
zero-browser
├── app.rs               — BrowserApp 主循环（连接 Shell + renderer/compositor + HostRuntime）
├── app_input*.rs        — 输入处理（按键、上下文菜单、平台输入适配）
├── app_render*.rs       — GPU chrome 渲染（标签栏、地址栏、导航按钮、菜单等）
├── compositor_client.rs — zero-compositor 子进程发现与连接（ZW_COMPOSITOR_BIN）
├── paint_ipc.rs         — compositor 绘制帧导入与呈现
├── process_backend.rs   — renderer/image-decoder 子进程管理
├── fetch_proxy.rs       — 页面网络请求代理（renderer IPC → 浏览器网络栈）
├── headless.rs          — headless 调试模式（renderer IPC 驱动）
├── tab_manager.rs       — 标签页状态管理
├── tab_snapshot.rs      — 页面快照/元数据（标题、favicon、缩略图）
├── favicon_fetch.rs     — favicon 抓取
├── service_worker_owner.rs — Service Worker 生命周期归属
├── pages.rs             — 内置页面（设置页等）
├── main.rs              — 应用入口
└── lib.rs               — 库入口（供集成测试与 smoke 复用）
```

## 依赖关系

```
zero-browser（默认 feature）
├── zero-browser-shell    — UI 数据模型（Tab/Bookmarks/History/Settings）
├── zero-page-runtime     — 页面运行时契约（页面加载/交互动作）
├── zero-host-runtime     — 窗口和事件循环（winit）
├── zero-render-foundation — GPU 渲染基础设施
├── zero-engine           — 页面管线（经 renderer 子进程使用）
├── zero-protocol         — 多进程 IPC 消息与通道
├── zero-script-sandbox   — 仅 default-features=false（不链接任何 JS 引擎）
├── zero-net / zero-security / zero-storage — 网络代理与安全/存储面
└── zero-webview          — 仅 `test-support` feature 下链接（默认不链接）
```
