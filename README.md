# ZeroBrowser

ZeroBrowser 是一个基于 Rust 构建的跨平台浏览器项目，目标是在源码、架构和演进节奏上完全自主可控的前提下，交付一套可闭源发布的浏览器体系：底层先提供一个可复用的 `webview` 内核库，供其他应用以 Rust `lib` 方式直接集成；上层再提供一个完整的跨平台浏览器应用，面向 macOS、Linux、Windows、Android，并为鸿蒙 PC 保留后续适配能力。项目目标覆盖现代 Web 能力，包括 HTML、CSS、JavaScript、WASM、多标签页、收藏夹与浏览器级宿主体验。

## 目标

- 构建一个可嵌入、可复用的 Rust `webview` 库
- 构建一个跨平台浏览器应用
- 保持第一方核心代码闭源可控
- 支持现代 Web 标准、JavaScript 与 WASM
- 复用现有高性能 Rust 渲染与宿主基础设施

## 技术选型

项目不采用 Servo 及其他 MPL 技术线作为主线依赖，而是基于许可边界可接受的 Rust 模块和自研核心能力构建页面内核：

- HTML 解析：`html5ever`
- JavaScript 引擎：`rusty_v8` 或 `rquickjs`
- WASM / 插件沙箱：`wasmtime` 或 `wasmi`
- 图形与窗口宿主：`wgpu` + `winit`
- 布局基础能力候选：`taffy`
- 渲染、字体、图片、宿主能力基础：复用和重构 `OmniTerm/terminal/crates` 中已有模块

项目会自行实现和掌控以下核心层：

- DOM
- CSSOM
- 样式系统
- 布局整合
- 渲染管线
- 导航模型
- 安全与运行时边界

## 项目架构

ZeroBrowser 预计按以下层次组织：

- `webview-api`
  面向外部应用的稳定嵌入接口，暴露导航、输入、脚本桥、渲染表面等能力。

- `engine-core`
  页面内核层，负责 HTML 解析、DOM、CSSOM、样式计算、布局、绘制、脚本执行与页面生命周期。

- `host-runtime`
  平台宿主层，负责窗口、事件循环、表面生命周期、输入法、平台集成与系统资源管理。

- `render-foundation`
  渲染基础设施层，承接 GPU 合成、软件后备、字体栈、图片缓存、离屏渲染等能力，并优先复用 OmniTerm 的成熟代码。

- `script-sandbox`
  扩展脚本、用户脚本和自动化脚本运行时，可在不同 JavaScript 引擎实现之间保留抽象边界。

- `wasm-sandbox`
  非页面 Wasm 执行环境，用于插件、扩展能力或受控计算任务。

- `browser-shell`
  浏览器产品层，负责多标签页、收藏夹、地址栏、历史记录、下载、权限 UI 和整体交互体验。

## 最终交付产物

项目最终面向三类产物交付：

- 一个可供其他 Rust 应用直接集成的 `webview` 库
- 一个跨平台浏览器应用
- 一套配套的嵌入 API、渲染基础设施、脚本/WASM 沙箱能力与工程化发布产物

## 当前状态

当前仓库处于调研与架构定义阶段，相关技术研究已沉淀在：

- [docs/research/rust-cross-platform-browser-research.md]($HOME/work/ZeroBrowser/docs/research/rust-cross-platform-browser-research.md)
