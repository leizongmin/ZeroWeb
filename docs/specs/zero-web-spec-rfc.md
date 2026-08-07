# Spec: ZeroWeb — 基于 Rust 的跨平台浏览器

**版本**: v1.3
**日期**: 2026-05-30
**作者**: AI Assistant
**状态**: Confirmed

> **说明**
> 本文描述的是 ZeroWeb 的目标状态、约束和设计方向，不构成当前仓库已经适合商用或其他生产用途的声明。当前项目仍是实验性项目，默认仅供学习、研究和工程探索使用；任何生产用途都需要自行评估功能完整性、安全性、兼容性、性能和许可证边界等风险。

---

## 1. 背景与目标

### 1.1 背景

当前浏览器市场被少数几个引擎（Chromium/Blink、Gecko、WebKit）垄断，且这些引擎均为 C++ 编写、历史包袱沉重、难以嵌入独立应用。Rust 生态中虽有 Servo 项目，但其采用 MPL 许可证，与本项目的依赖许可策略冲突；且 Servo 的架构并未将「可嵌入 WebView 库」作为核心设计目标。

与此同时，团队已在 OmniTerm 终端项目中积累了成熟的 GPU/CPU 双路径渲染基础设施（wgpu 合成、字体渲染栈、图片缓存、场景/Primitive/Backend 分层架构），这些资产可直接复用到浏览器渲染管线中，大幅降低从零构建渲染层的成本。

**核心痛点**：
- 现有浏览器引擎（Chromium/Gecko/WebKit）均为 C++ 编写，嵌入成本高且不可控
- Rust 生态中无可用的、采用宽松许可证的浏览器内核
- 需要一个可独立嵌入的 WebView 库，供其他 Rust 应用直接集成
- Servo 受 MPL 许可证限制，不适合作为本项目的主线核心依赖

### 1.2 目标

**长期产品目标**：
- 构建一个生产可用的、基于 Rust 的跨平台浏览器体系，具备自主可控的代码主权
- 交付可复用的 WebView 内核库，使其他 Rust 应用能以 `lib` 方式直接集成网页渲染与交互能力
- 交付跨平台桌面浏览器应用（macOS / Linux / Windows），具备完整的浏览器体验

**用户目标**：
- 开发者：可通过 WebView 库在自有的 Rust 应用中嵌入网页渲染能力
- 终端用户：可在主流桌面平台上使用功能完整的浏览器进行日常网页浏览
- 集成方 / 企业用户：可在遵守项目许可证和第三方 notices 的前提下，将浏览器内核集成到自有产品中

### 1.3 范围边界

**包含（In scope）**：
- 可复用的 `webview` 内核库（WebView API）
- 跨平台桌面浏览器应用（macOS / Linux / Windows）
- HTML 解析 + DOM 树 + Shadow DOM（基础）
- 完全自建 CSS 解析器、选择器引擎、样式系统
- 布局引擎（Block / Inline / Flexbox / Grid）
- 基于 wgpu 的渲染管线（GPU + CPU 双路径）
- JavaScript 集成（V8 引擎 + DOM API）
- 网络栈（HTTP/HTTPS、Cookie、缓存、CORS）
- 多进程架构（浏览器进程 + 渲染进程）
- Canvas 2D API
- Web Workers（Dedicated Worker）
- 客户端存储（localStorage / sessionStorage / IndexedDB）
- 安全模型（同源策略、CSP、沙箱）
- 测试基础设施（单元测试、基准测试、WPT）
- CI/CD 管线

**明确排除（Out of scope）**：
- MPL 许可证技术栈（servo、stylo、webrender、rust-cssparser、lightningcss、mozjs）
- 首期移动端发布（Android 和鸿蒙 PC 为后续适配）
- 首期完整 DevTools（仅控制台日志和基础审查）
- 首期媒体播放（`<video>`/`<audio>` 仅做布局占位）
- 首期 WebGL/WebGPU（3D 图形 API 后续）
- 首期浏览器扩展系统（WebExtension API 后续）
- 首期 WebRTC
- 双 JS 引擎等价（V8 是唯一稳定页面引擎，QuickJS 仅用于扩展沙箱）

---

## 2. 需求类型概览

| 类型 | 适用 | 来源 |
|------|------|------|
| 业务需求 | 是 | 目标文档 Mission 章节 |
| 用户需求 | 是 | 目标文档 Done Criteria + 用户场景分析 |
| 解决方案需求 | 是 | 技术调研文档（Route A 选型） |
| 功能需求 | 是 | 第 3 节详述 |
| 非功能需求 | 是 | 第 4 节详述 |
| 接口需求 | 是 | 第 5 节详述 |
| 过渡需求 | 是 | 从 OmniTerm 复用代码的迁移计划 |

---

## 3. 功能需求

### 3.1 WebView 内核库

#### FR-001: WebView 可嵌入 API
- **描述**: 系统**必须**提供稳定的 Rust API，使其他 Rust 项目能以 `lib` 方式引入 `webview` crate，获得完整的网页渲染与交互能力
- **验收标准**:
  - [ ] `webview` crate 可以被其他 Rust 项目以 `lib` 方式引入
  - [ ] 可以创建 WebView 实例、导航到 URL、注入 JS、接收回调
  - [ ] 提供渲染表面输出接口（可嵌入其他应用的渲染上下文）
  - [ ] 提供脚本桥（Rust ↔ JS 双向调用）
  - [ ] 嵌入示例代码（`apps/webview-demo/`）可编译运行
  - [ ] API 文档完整（`cargo doc` 可生成）
- **优先级**: Must
- **来源**: 目标文档 Done Criteria §1

#### FR-002: URL 加载与页面渲染
- **描述**: WebView**必须**能加载 URL 并渲染 HTML 页面，包含文本、图片和基础 CSS 布局
- **验收标准**:
  - [ ] 可以通过 URL 加载并渲染 HTML 页面
  - [ ] 文本渲染正确（字体查找、glyph 渲染、文本排列、行高）
  - [ ] 图片渲染正确（`<img>` 加载与显示、object-fit）
  - [ ] CSS 基础布局正确（盒模型、颜色、字体、定位、flexbox、grid）
  - [ ] 至少 Top 20 静态网站可以正确加载渲染
- **优先级**: Must
- **来源**: 目标文档 Done Criteria §1

#### FR-003: JavaScript 执行与 DOM 操作
- **描述**: 系统**必须**通过 V8 引擎（rusty_v8）执行页面 JavaScript，并提供完整的 DOM API 绑定
- **验收标准**:
  - [ ] 页面内 JavaScript 可以正确执行
  - [ ] JS 可以操作 DOM（增删改查节点、修改样式、读写属性）
  - [ ] 支持 `document.getElementById`、`querySelector`/`querySelectorAll`、`innerHTML`/`textContent`、`createElement`、`setAttribute` 及 DOM 树操作全量
  - [ ] 事件系统正确触发和传播（addEventListener、冒泡/捕获、自定义事件、焦点事件、输入事件）
  - [ ] HTML spec event loop 集成（microtask、task、requestAnimationFrame、requestIdleCallback）
- **优先级**: Must
- **来源**: 目标文档 Done Criteria §1, M6

### 3.2 HTML 解析与 DOM

#### FR-010: HTML 解析
- **描述**: 系统**必须**基于 html5ever 解析 WHATWG HTML 规范的文档，构建正确的 DOM 树
- **验收标准**:
  - [ ] 可以解析标准 HTML5 文档并生成正确的 DOM 树
  - [ ] 支持完整的 DOM 节点类型（Element、Text、Document、Comment 等）
  - [ ] 解析器能处理错误恢复（malformed HTML）
  - [ ] DOM 修改 API 完整（appendChild、removeChild、insertBefore 等）
  - [ ] Mutation Observer 基础框架就位
  - [ ] 支持 iframe 渲染
  - [ ] 支持 Shadow DOM（基础）
  - [ ] 支持表单元素
- **优先级**: Must
- **来源**: 目标文档 M2, Tier 1 标准

### 3.3 CSS 解析与样式系统

#### FR-020: CSS 解析器
- **描述**: 系统**必须**提供完全自建的 CSS 解析器（tokenizer + parser），不依赖任何 MPL 许可的 CSS 解析库
- **验收标准**:
  - [ ] 完整的 CSS 语法解析（tokenizer + parser），生成正确的 AST
  - [ ] 支持选择器解析：类型、类、ID、属性、伪类、伪元素、组合器、`:is()`/`:where()`/`:not()`
  - [ ] 选择器引擎可以正确匹配 DOM 节点（含复杂选择器）
- **优先级**: Must
- **来源**: 目标文档 M3

#### FR-021: CSS 属性支持
- **描述**: CSS 解析器**必须**支持 Tier 1 阶段的全部 CSS 属性
- **验收标准**:
  - [ ] 支持属性：display、width/height、margin/padding/border、color、background、font、position、overflow、visibility、opacity、z-index、box-sizing、min/max
  - [ ] 支持 Flexbox 全量属性（flex-direction、flex-wrap、justify-content、align-items、align-self、flex-grow/shrink/basis、gap、order）
  - [ ] 支持 Grid 基础属性（grid-template、grid-area、gap）
  - [ ] 支持 transform、transition
  - [ ] 支持自定义属性（`--*`）的声明、引用、回退
  - [ ] 支持媒体查询（`@media`）
  - [ ] 支持逻辑属性
- **优先级**: Must
- **来源**: 目标文档 M3, Tier 1 标准

#### FR-022: 样式系统
- **描述**: 系统**必须**实现完整的样式系统，包括级联、继承、初始值和计算值
- **验收标准**:
  - [ ] 级联规则正确应用（specificity、!important、继承、@layer）
  - [ ] 计算值生成正确
  - [ ] 支持 `@media`、`@supports`、`@layer`、`@import` 规则
  - [ ] 样式系统与 DOM 集成，可以为 DOM 节点计算样式
- **优先级**: Must
- **来源**: 目标文档 M3

### 3.4 布局引擎

#### FR-030: 布局引擎核心
- **描述**: 系统**必须**基于 taffy 扩展构建完整的布局引擎，支持多种布局模式
- **验收标准**:
  - [ ] Block layout（正常流块级布局）
  - [ ] Inline layout（行内格式化上下文、文本换行、white-space 处理）
  - [ ] Flexbox layout（集成 taffy，含 gap、align-self、order）
  - [ ] CSS Grid layout（集成 taffy，含 grid-template、grid-area、gap）
  - [ ] Positioned layout（relative、absolute、fixed、sticky）
  - [ ] Overflow 和 scrolling 布局（scrollable overflow、clip）
  - [ ] 布局输出为盒模型坐标，可供渲染管线消费
  - [ ] 各布局模式与 Chrome 表现一致（对比验证）
- **优先级**: Must
- **来源**: 目标文档 M4

### 3.5 渲染管线

#### FR-040: 页面渲染
- **描述**: 系统**必须**将布局输出连接到渲染管线，实现网页的像素级渲染
- **验收标准**:
  - [ ] 布局盒 → 渲染命令的转换（paint）
  - [ ] 文本渲染（字体查找、glyph 渲染、文本排列、行高、text-overflow）
  - [ ] 矩形/背景/边框渲染（含 border-radius、box-shadow）
  - [ ] 图片渲染（`<img>` 加载与显示、object-fit）
  - [ ] GPU 合成输出到窗口
  - [ ] 增量渲染：脏矩形追踪，DOM 变更只触发局部重排/重绘
  - [ ] GPU 加速合成层（transform、opacity 提升为独立合成层）
  - [ ] 可以加载本地 HTML 文件并渲染出可视化页面
- **优先级**: Must
- **来源**: 目标文档 M5

### 3.6 JavaScript 集成

#### FR-050: Web API
- **描述**: 系统**必须**通过 V8 引擎提供完整的 Web API
- **验收标准**:
  - [ ] `console`、`setTimeout`/`setInterval`/`clearTimeout`/`clearInterval`、`Promise`、`Error`
  - [ ] Fetch API（`fetch()`、`Request`、`Response`、`Headers`）
  - [ ] ES Modules 支持（`<script type="module">`、`import`/`export`、import maps）
  - [ ] `<canvas>` 元素宿主（创建渲染上下文，连接到 canvas crate）
  - [ ] WebSocket 基础通信（`new WebSocket()`）
  - [ ] Web Workers（Dedicated Worker：`new Worker()`、`postMessage`、`onmessage`）
  - [ ] Storage：localStorage / sessionStorage / IndexedDB 基础
  - [ ] CSSOM 基础（JS 读写元素样式、`getComputedStyle()`、操作 stylesheet）
- **优先级**: Must
- **来源**: 目标文档 M6, M9, Tier 1 标准

### 3.7 网络栈

#### FR-060: HTTP/HTTPS 网络能力
- **描述**: 系统**必须**基于 hyper/reqwest 实现 HTTP/HTTPS 网络栈
- **验收标准**:
  - [ ] HTTP/HTTPS 请求正常工作
  - [ ] HTML/CSS/JS/图片资源的加载与缓存
  - [ ] URL 解析和导航模型（前进、后退、hash 变更、pushState/replaceState）
  - [ ] Cookie 管理（含 Secure、HttpOnly、SameSite 属性）
  - [ ] 重定向处理
  - [ ] TLS/HTTPS 证书校验
  - [ ] 可以通过 URL 加载真实网页
- **优先级**: Must
- **来源**: 目标文档 M7

### 3.8 安全模型

#### FR-070: 安全边界
- **描述**: 系统**必须**实施完整的安全模型，包括同源策略、CORS、CSP 和进程沙箱
- **验收标准**:
  - [ ] 同源策略正确阻止跨域不当访问
  - [ ] CORS 预检请求和简单请求正确处理
  - [ ] CSP 基础实施（script-src、style-src、img-src 指令）
  - [ ] Cookie 安全属性（Secure/HttpOnly/SameSite）正确实施
  - [ ] 渲染进程沙箱限制文件系统、网络、进程访问
  - [ ] 链接点击、表单提交的安全策略正确实施
- **优先级**: Must
- **来源**: 目标文档 M7, M8, Tier 1 标准

### 3.9 多进程架构

#### FR-080: 进程隔离
- **描述**: 系统**必须**实现浏览器进程和渲染进程的分离，建立安全边界
- **验收标准**:
  - [ ] 浏览器进程（Browser Process）：窗口管理、标签页管理、网络调度、存储调度
  - [ ] 渲染进程（Renderer Process）：页面渲染、JS 执行、DOM 操作
  - [ ] 进程间通信机制（命令/响应/流式传输）稳定可靠
  - [ ] 渲染进程崩溃恢复（检测崩溃、自动重建、用户通知）
  - [ ] 单个标签页崩溃不影响其他标签页
  - [ ] iframe 基础渲染（嵌入其他页面、sandbox 属性基础支持）
- **优先级**: Must
- **来源**: 目标文档 M8

### 3.10 Canvas 2D

#### FR-090: Canvas 2D API
- **描述**: 系统**必须**实现完整的 Canvas 2D 渲染 API
- **验收标准**:
  - [ ] CanvasRenderingContext2D：路径、矩形、圆弧、文本、图像绘制
  - [ ] 变换矩阵、合成模式、渐变/图案
  - [ ] clip、save/restore
  - [ ] getImageData/putImageData
  - [ ] `<canvas>` 元素与 DOM/渲染管线完整集成
  - [ ] Canvas 性能基准：1000 个矩形渲染 < 16ms
- **优先级**: Must
- **来源**: 目标文档 M9

### 3.11 浏览器应用

#### FR-100: 多标签页管理
- **描述**: 浏览器应用**必须**支持多标签页操作
- **验收标准**:
  - [ ] 创建、关闭、切换标签页
  - [ ] 标签页拖拽排序
  - [ ] 单个标签页崩溃不影响其他标签页
- **优先级**: Must
- **来源**: 目标文档 M11

#### FR-101: 导航控制
- **描述**: 浏览器应用**必须**提供完整的导航功能
- **验收标准**:
  - [ ] 地址栏输入 URL 并导航
  - [ ] URL 自动补全
  - [ ] 加载进度指示
  - [ ] 前进、后退、刷新、主页
- **优先级**: Must
- **来源**: 目标文档 M11

#### FR-102: 收藏夹
- **描述**: 浏览器应用**必须**支持收藏夹功能
- **验收标准**:
  - [ ] 添加、删除收藏夹
  - [ ] 文件夹管理
  - [ ] 收藏栏展示
  - [ ] 点击收藏导航
- **优先级**: Must
- **来源**: 目标文档 Done Criteria §2

#### FR-103: 历史记录
- **描述**: 浏览器应用**必须**支持历史记录功能
- **验收标准**:
  - [ ] 记录访问历史
  - [ ] 搜索历史
  - [ ] 清除历史
- **优先级**: Must
- **来源**: 目标文档 Done Criteria §2

#### FR-104: 其他浏览器功能
- **描述**: 浏览器应用**应当**提供完整的桌面浏览器体验
- **验收标准**:
  - [ ] 下载管理器（下载文件、进度显示、打开所在文件夹）
  - [ ] 页面查找（Ctrl+F 搜索高亮）
  - [ ] 缩放（Ctrl+/Ctrl- 缩放页面、重置）
  - [ ] 右键上下文菜单（复制、粘贴、检查元素、图片操作）
  - [ ] 基础设置页面（默认搜索引擎、主页、隐私设置）
  - [ ] 三个桌面平台（macOS/Linux/Windows）上的原生体验
- **优先级**: Should
- **来源**: 目标文档 M11

### 3.12 工程基础设施

#### FR-110: CI/CD 管线
- **描述**: 项目**必须**建立自动化 CI 管线，确保每次提交的质量
- **验收标准**:
  - [ ] GitHub Actions CI 管线就位：编译 + 测试 + clippy + 基准运行
  - [ ] CI 在三平台（macOS/Linux/Windows）上运行
  - [ ] CI 失败时阻止合并
- **优先级**: Must
- **来源**: 目标文档 Done Criteria §6

#### FR-111: 测试基础设施
- **描述**: 项目**必须**建立完善的测试和基准基础设施
- **验收标准**:
  - [ ] `scripts/run-benchmarks.sh` 一键运行所有基准并输出报告
  - [ ] `scripts/check-coverage.sh` 一键测量并输出覆盖率摘要
  - [ ] criterion 基准测试结果持久化到 `tests/benchmarks/results/`（JSON 格式，文件名含日期）
  - [ ] 基准报告包含趋势对比（每轮执行记录对比数据）
- **优先级**: Must
- **来源**: 目标文档 Done Criteria §6

#### FR-112: WPT 测试基础设施
- **描述**: 项目**必须**建立 Web Platform Tests 运行基础设施，追踪 Web 标准合规性
- **验收标准**:
  - [ ] `tests/wpt-runner/` 目录结构就位，可运行 WPT 子集
  - [ ] 按分类追踪通过率：HTML、CSS、JS/DOM、Network、Navigation
  - [ ] 每轮记录通过率变化
  - [ ] 通过率低不阻塞，但"提高通过率"必须是活跃工作项
- **优先级**: Must
- **来源**: 目标文档 Done Criteria §5, M5

#### FR-113: 端到端集成测试
- **描述**: 项目**必须**至少有一个集成测试可以端到端加载真实 URL 并验证渲染输出
- **验收标准**:
  - [ ] 存在 `tests/integration/` 目录下的端到端测试
  - [ ] 测试可以加载真实 URL 并验证渲染输出（截图对比或 DOM 结构验证）
- **优先级**: Must
- **来源**: 目标文档 Done Criteria §5

### 3.13 术语定义

**"中等复杂度页面"**: 指包含约 500-1000 个 DOM 节点、50-100 条 CSS 规则、含文本和图片、使用 flexbox/grid 布局的页面，典型示例为 MDN 文档页面或 Wikipedia 文章页面。

**"正确渲染"**: 指页面布局结构与主流浏览器（Chrome/Firefox）一致，文本可读、链接可点击、图片可见，无明显的布局错误或内容缺失，不要求像素级完全一致。

**"Top 20 静态网站"**: 指 Alexa/Tranco 排名前 20 中的静态内容页面（去除需要登录或动态交互的 SPA 应用），具体清单在实施时确定并记录。

---

## 4. 非功能需求

### NFR-001: 性能 — 首屏渲染
- **描述**: 中等复杂度页面（见 §3.13 术语定义）首屏渲染**必须**在 2 秒以内完成
- **测量**: 端到端基准测试，包含各阶段耗时分解（解析 → 样式 → 布局 → 绘制 → 合成），测试数据持久化到 `tests/benchmarks/results/`
- **优先级**: Must

### NFR-002: 性能 — 增量渲染
- **描述**: DOM 局部变更的重新渲染耗时**必须**低于全量渲染的 20%
- **测量**: 增量渲染基准测试，对比全量重排/重绘耗时
- **优先级**: Must

### NFR-003: 性能 — GPU 加速
- **描述**: GPU 加速合成**必须**正常工作，支持 transform/opacity 提升为独立合成层
- **测量**: GPU 合成层创建和渲染基准测试
- **优先级**: Must

### NFR-004: 性能 — Canvas 2D
- **描述**: Canvas 2D **必须**满足实时渲染性能要求
- **测量**: 1000 个矩形渲染 < 16ms
- **优先级**: Must

### NFR-005: 性能 — 基准回归门禁
- **描述**: 所有关键路径基准 p95 **必须**不超过历史最佳基线的 120%
- **测量**: `cargo bench` 自动对比历史结果
- **优先级**: Must

### NFR-006: 代码质量 — 编译
- **描述**: 所有 crate **必须**编译无错误、无警告
- **测量**: `cargo build` + `cargo clippy` 通过
- **优先级**: Must

### NFR-007: 代码质量 — 测试
- **描述**: 所有现有测试**必须**持续通过，不允许留下红灯
- **测量**: `cargo test` 全通过
- **优先级**: Must

### NFR-008: 代码质量 — 覆盖率
- **描述**: 核心 crate 行覆盖率**必须** ≥ 70%，非核心 crate ≥ 50%
- **测量**: `cargo-llvm-cov` 或 `tarpaulin`，通过 `scripts/check-coverage.sh` 运行
- **优先级**: Must

### NFR-009: 代码质量 — 测试密度
- **描述**: 核心 crate（dom、css-parser、style-system、layout-engine、canvas、security）测试行数**必须** ≥ 实现行数的 30%
- **测量**: 代码行 vs 测试行比例
- **优先级**: Must

### NFR-010: 文档 — API 文档
- **描述**: 所有公开 API **必须**有 doc comment，所有 crate 有 README 和 API 文档
- **测量**: `cargo doc` 生成完整文档
- **优先级**: Must

### NFR-011: 可移植性 — 跨平台编译
- **描述**: 项目**必须**在 macOS（x86_64 + aarch64）、Linux（x86_64）、Windows（x86_64）上编译运行
- **测量**: 三平台 `cargo build` 成功
- **优先级**: Must

### NFR-012: 可移植性 — 移动端架构预留
- **描述**: 架构设计**必须**从第一天预留 Android（aarch64）和鸿蒙 PC 的适配能力
- **测量**: 架构审查确认平台抽象层设计合理
- **优先级**: Should

### NFR-013: 安全 — 许可证合规
- **描述**: 项目**必须**排除所有 MPL 许可证依赖，确保开源发布与商业集成都具备清晰的许可证边界
- **测量**: `cargo deny` 或等效工具检查许可证
- **优先级**: Must

---

## 5. 接口需求

### IF-001: WebView 嵌入 API
- **类型**: Rust API（lib crate）
- **规范**: `webview` crate 提供稳定的 Rust 公共接口
  - `WebViewBuilder` — 构建器模式创建 WebView 实例
  - `WebView::navigate(url)` — 导航到指定 URL
  - `WebView::evaluate_script(js)` — 注入并执行 JavaScript
  - `WebView::on_callback(handler)` — 注册 JS → Rust 回调
  - `WebView::surface()` — 获取渲染表面输出
  - `WebView::on_event(handler)` — 注册页面事件回调（加载完成、错误等）
- **错误处理**: 通过 `Result<T, WebViewError>` 返回错误，包含错误码和描述

### IF-002: 渲染表面接口
- **类型**: Rust API（渲染抽象）
- **规范**: 提供跨平台渲染表面输出接口，支持嵌入到其他应用窗口
  - 支持输出到 wgpu Surface
  - 支持输出到原始像素缓冲区（CPU 渲染后备）
- **错误处理**: 表面创建失败、尺寸变更通知

### IF-003: 脚本桥接口
- **类型**: Rust ↔ JS 双向调用接口
- **规范**:
  - Rust → JS: `WebView::evaluate_script(js: &str) -> Result<JsValue>`
  - JS → Rust: 通过注册的回调函数 `WebView::on_callback(name, handler)`
  - 支持基本类型传递（字符串、数字、布尔、null）
  - 支持 Promise 异步结果返回
- **错误处理**: JS 执行异常、类型转换失败、超时

### IF-004: 多进程 IPC 协议
- **类型**: 系统内部协议
- **规范**: `protocol` crate 定义浏览器进程与渲染进程间的 IPC 消息格式
  - 命令/响应模式（同步 IPC）
  - 流式传输模式（大块数据传输）
  - 消息序列化/反序列化
- **错误处理**: 进程崩溃检测、消息超时、格式错误

### IF-005: 网络接口
- **类型**: 内部 API（hyper/reqwest 封装）
- **规范**: `net` crate 提供统一的网络请求接口
  - HTTP/HTTPS GET/POST/PUT/DELETE
  - 资源加载（HTML/CSS/JS/图片）
  - WebSocket 连接
- **错误处理**: 网络超时、DNS 解析失败、TLS 错误、HTTP 错误码

### IF-006: 平台宿主接口
- **类型**: Rust API + 平台事件
- **规范**: `host-runtime` crate 基于 winit 提供跨平台窗口和事件循环
  - 窗口创建和管理
  - 输入事件（键盘、鼠标、触摸）
  - 输入法集成
  - 系统事件（窗口大小变更、焦点变化）
- **错误处理**: 窗口创建失败、平台不支持

### IF-007: WASM 沙箱接口
- **类型**: Rust API（feature-gated）
- **规范**: `wasm-sandbox` crate 提供 WASM 运行时沙箱
  - Wasmtime（高性能）或 wasmi（纯 Rust）后端
  - WASI 基础支持
  - 内存隔离和资源限制
- **错误处理**: WASM 编译失败、运行时错误、资源超限

---

## 6. 约束与假设

### 6.1 技术约束

- **C-001**: 所有第三方依赖**必须**采用宽松许可证（Apache-2.0、MIT、BSD 等），**严禁**引入 MPL-2.0 或更严格许可证的依赖
  - 明确排除：servo、stylo、webrender、rust-cssparser、lightningcss、mozjs
- **C-002**: CSS 解析器**必须**完全自建，因为所有成熟的 Rust CSS 解析库（rust-cssparser、lightningcss）均为 MPL 许可
- **C-003**: JavaScript 页面引擎**必须**使用 V8（通过 rusty_v8），QuickJS 仅作为 feature-gated 扩展脚本沙箱
  - **决策记录（B2，2026-08-07）**：单默认引擎是**有意决策**而非权宜——(a) 双引擎浏览器级等价成本极高（DOM/IDL/GC 生命周期/事件循环两套绑定，调研结论见 `docs/research/rust-cross-platform-browser-research.md` §5）；(b) Ladybird 教训：LibJS 自研 6 年+ 仍是最大单点投入，且其 Swift 插曲（2024-08 → 2026-02 放弃）证明「愿景完整度 < 生态成熟度」的选型原则（调研报告 §5.3 L1）。**trait 抽象边界预留（`page_js = v8 | quickjs`）但不承诺双引擎等价**；首个可用内核只稳定支持 V8，QuickJS 仅作扩展/用户脚本沙箱
- **C-004**: 渲染管线**必须**基于 wgpu 构建，支持 GPU 和 CPU 双路径渲染
- **C-005**: 窗口管理**必须**基于 winit，确保跨平台一致性
- **C-006**: 布局算法**必须**基于 taffy 进行扩展，而非从零实现 Flexbox/Grid 基础算法
- **C-007**: 项目**必须**采用 Cargo workspace 组织多 crate 结构
- **C-008**: MSRV（最低支持 Rust 版本）为 Rust 1.85（edition 2024）

### 6.2 平台约束

- **C-010**: 首发平台为 macOS（x86_64 + aarch64）、Linux（x86_64）、Windows（x86_64）
- **C-011**: Android（aarch64）为 P1 后续适配，架构设计必须预留但不阻塞首发
- **C-012**: 鸿蒙 PC 为 P2 后续适配，高风险平台，暂不首发
- **C-013**: 渲染进程沙箱机制因平台而异：Linux 使用 seccomp-bpf；macOS 使用 sandbox-exec；Windows 使用 Job Objects

### 6.3 代码复用约束

- **C-020**: 从 OmniTerm 项目复用以下模块到 `render-foundation` crate：
  - `omniterm-terminal-render`：场景/Primitive/Backend 分层架构
  - `omniterm-terminal-render-wgpu`：GPU glyph atlas、pane 缓存、wgpu 合成
  - `omniterm-terminal-render-soft`：fontdue + swash 字体栈、软件渲染后备
  - `omniterm-terminal-image`：图片对象缓存与 GC 限制
  - `omniterm-terminal-ffi/wasm`：ABI 边界和 WASM 友好封装
- **C-021**: 复用代码必须重写适配浏览器场景，后续可考虑抽取为共享 crate

### 6.4 代码变更边界

**允许修改的路径**（绿色区域，实施过程中可自由变更）：
- `crates/**` — 所有 crate 的源代码
- `apps/**` — 应用入口代码
- `tests/**` — 测试、基准、WPT runner
- `scripts/**` — 工程化脚本
- `docs/specs/**` — Spec/RFC 文档
- `docs/goal/zero-web/master.md` — 运行时控制平面
- `docs/goal/zero-web/archive/**` — 归档区域
- `Cargo.toml`、`Cargo.lock` — 依赖管理
- `.github/workflows/**` — CI 配置

**禁止修改的路径**（除非目标本身变化）：
- `docs/goal/zero-web.md` — 目标执行契约（仅在 Mission 变化时修改）
- `docs/research/` — 技术调研文档（历史记录，只追加不修改）
- `LICENSE` — 项目许可证

### 6.5 已决定的技术决策

| 决策 | 选择 | 理由 | 状态 |
|------|------|------|------|
| 技术路线 | Route A — 自建内核 | MPL 排除 Servo，系统 WebView 不满足自有内核目标 | 已确认 |
| CSS 解析方案 | 完全自建 | 所有成熟 Rust CSS 解析库均为 MPL 许可 | 已确认 |
| JS 页面引擎 | V8（rusty_v8） | 生产级性能，MIT 许可的 Rust 绑定 | 已确认 |
| JS 扩展沙箱 | QuickJS（rquickjs，feature-gated） | 轻量级，适合扩展脚本，不用于页面内容 | 已确认 |
| 布局基础 | taffy 扩展 | MIT 许可，Flexbox/Grid/Block 算法基础 | 已确认 |
| 渲染基础 | OmniTerm 复用 + wgpu | 已有成熟 GPU/CPU 双路径渲染基础设施 | 已确认 |
| 进程模型 | 浏览器进程 + 多渲染进程 | 安全隔离和稳定性 | 已确认 |

### 6.6 假设

- **A-001**: OmniTerm 渲染模块的 API 和架构可以合理适配到浏览器渲染管线 — 状态：待验证
- **A-002**: rusty_v8 的 API 稳定性和性能足以支撑生产级浏览器使用 — 状态：待验证
- **A-003**: html5ever 的解析性能和标准合规性满足浏览器需求 — 状态：待验证（html5ever 是 Servo 项目产物但采用 Apache-2.0/MIT 许可）
- **A-004**: taffy 的 Flexbox/Grid 算法精度满足浏览器级布局需求 — 状态：待验证
- **A-005**: 自建 CSS 解析器可以在合理时间内达到 Tier 1 标准覆盖 — 状态：待验证
- **A-006**: wgpu 在三个桌面平台上的稳定性和性能满足浏览器渲染需求 — 状态：待验证
- **A-007**: V8 的内存和二进制体积对桌面浏览器可接受 — 状态：待验证
- **A-008**: 社区对 html5ever、taffy、wgpu 等库的维护活跃度将持续 — 状态：待验证

---

## 7. 优先级与里程碑建议

| ID | 需求 | 优先级 | 理由 | 里程碑 |
|----|------|--------|------|--------|
| FR-001 | WebView 可嵌入 API | Must | 核心交付物 | M10 |
| FR-002 | URL 加载与页面渲染 | Must | 核心能力 | M5-M7 |
| FR-003 | JavaScript 执行与 DOM 操作 | Must | 核心能力 | M6 |
| FR-010 | HTML 解析 | Must | 渲染基础 | M2 |
| FR-020 | CSS 解析器 | Must | 渲染基础 | M3 |
| FR-021 | CSS 属性支持 | Must | 渲染基础 | M3 |
| FR-022 | 样式系统 | Must | 渲染基础 | M3 |
| FR-030 | 布局引擎核心 | Must | 渲染基础 | M4 |
| FR-040 | 页面渲染 | Must | 视觉输出 | M5 |
| FR-050 | Web API | Must | 交互能力 | M6, M9 |
| FR-060 | HTTP/HTTPS 网络能力 | Must | 真实网页 | M7 |
| FR-070 | 安全边界 | Must | 安全底线 | M7-M8 |
| FR-080 | 进程隔离 | Must | 稳定性 | M8 |
| FR-090 | Canvas 2D API | Must | Web 标准 | M9 |
| FR-100 | 多标签页管理 | Must | 浏览器体验 | M11 |
| FR-101 | 导航控制 | Must | 浏览器体验 | M11 |
| FR-102 | 收藏夹 | Must | 浏览器体验 | M11 |
| FR-103 | 历史记录 | Must | 浏览器体验 | M11 |
| FR-104 | 其他浏览器功能 | Should | 完整体验 | M11 |
| FR-110 | CI/CD 管线 | Must | 工程底线 | M1 |
| FR-111 | 测试基础设施 | Must | 工程底线 | M1 |
| FR-112 | WPT 测试基础设施 | Must | 标准合规 | M5 |
| FR-113 | 端到端集成测试 | Must | 质量验证 | M5+ |
| NFR-001~005 | 性能需求 | Must | 可用性底线 | 全程 |
| NFR-006~009 | 代码质量 | Must | 工程底线 | 全程 |

### 建议里程碑

- **M1（阶段 1）**: 项目骨架 + 渲染基础设施迁移 — 建立 workspace 结构，迁移 OmniTerm 渲染核心，在桌面平台显示窗口并渲染文本。预计 2-3 周。
- **M2（阶段 2）**: HTML 解析 + DOM 树 — 基于 html5ever 构建完整 DOM 树。预计 1-2 周。
- **M3（阶段 3）**: CSS 解析器 + 样式系统 — 完全自建 CSS parser 和样式系统。预计 3-4 周。
- **M4（阶段 4）**: 布局引擎 — 基于 taffy 扩展，支持 Block/Inline/Flexbox/Grid。预计 2-3 周。
- **M5（阶段 5）**: 渲染管线集成（首屏渲染）— 将布局输出连接到渲染管线，首次像素级渲染。预计 2-3 周。
- **M6（阶段 6）**: JavaScript 集成（V8）— 集成 V8 引擎，实现 JS 执行和 DOM API。预计 3-4 周。
- **M7（阶段 7）**: 网络栈 + 导航模型 — 实现真实网页加载能力。预计 2-3 周。
- **M8（阶段 8）**: 多进程架构 + 安全沙箱 — 进程隔离和安全边界。预计 3-4 周。
- **M9（阶段 9）**: Canvas 2D + Web Workers + Storage — 补全 Web 标准核心 API。预计 2-3 周。
- **M10（阶段 10）**: WebView 库 API 稳定化 — 达到可嵌入级别。预计 1-2 周。
- **M11（阶段 11）**: 浏览器应用 — 构建完整桌面浏览器。预计 4-6 周。
- **M12（阶段 12）**: 高级 Web 能力 — 扩展到 Tier 2 标准。预计 4-6 周。
- **M13（阶段 13）**: 性能优化 + 安全加固 — 系统级优化。预计 3-4 周。
- **M14（阶段 14）**: 生产化与平台扩展 — 发布准备和移动端适配。预计 4-6 周。

**预计总工期**: 36-54 周（约 9-14 个月），视团队规模和实际技术难度调整。

---

## 8. 技术设计（RFC）

### 8.1 现状分析（As-Is）

**当前架构**: M1 里程碑执行中。Cargo workspace 已建立，包含 16 个 crate 骨架 + 2 个应用入口，其中 `render-foundation` 和 `host-runtime` 已有实质性实现。wgpu GPU 渲染后端已在 `render-foundation` 中实现，`host-runtime` 提供 `run_with_window()` 用于 GPU surface 创建，Demo 二进制已切换到 wgpu GPU 渲染路径。

**代码规模**: 3,616 行 Rust 源代码（32 个 `.rs` 文件），69 个单元测试，5 个 criterion 基准测试，零 clippy 警告。

**已有资产**:

| 资产 | 状态 | 说明 |
|------|------|------|
| 技术调研文档 | ✅ 完成 | 四轮迭代，技术路线已确认 |
| Cargo workspace | ✅ 完成 | 16 crate + 2 apps，全部编译通过 |
| CI 管线 | ✅ 完成 | GitHub Actions 三平台（ubuntu/macos/windows） |
| `render-foundation` | ✅ GPU+CPU | 2,532 行源码，53 个测试，5 个基准；CPU 渲染数据模型 + wgpu GPU 后端已实现 |
| `host-runtime` | ✅ 完成 | 329 行源码，3 个测试；winit 窗口+事件循环+GPU surface 创建（`run_with_window()`） |
| Demo 二进制 | ✅ GPU 版 | 800×600 wgpu GPU 渲染 + CPU 后备 PPM 输出 + winit 窗口展示 |
| 覆盖率测量脚本 | ✅ 就位 | `scripts/check-coverage.sh` |
| 基准运行脚本 | ⚠️ 路径有误 | `scripts/run-benchmarks.sh` 引用不存在的 Cargo.toml |

**render-foundation 已实现模块**:

| 模块 | 内容 | 行数 | 测试数 |
|------|------|------|--------|
| `color` | Color (RGBA)、hex 解析、sRGB→linear、premultiplied alpha | 180 | 5 |
| `geometry` | Point、Size、Rect、DamageTracker（脏矩形合并） | 294 | 9 |
| `primitive` | FillPrimitive、GlyphPrimitive、RenderPrimitives | 152 | 4 |
| `surface` | SurfaceDescriptor、FrameBuffer（CPU RGBA 像素缓冲区） | 180 | 8 |
| `font/loader` | FontLoader（fontdue 集成）、字体加载和 glyph 光栅化 | 143 | 6 |
| `font/cache` | GlyphCache、LRU 淘汰策略 | 201 | 6 |
| `gpu/atlas` | GpuAtlas — wgpu 纹理图集，glyph 上传、纹理绑定 | 374 | — |
| `gpu/pipeline` | GpuPipeline — wgpu 渲染管线（着色器、bind group、render pipeline） | 213 | — |
| `gpu/renderer` | GpuRenderer — wgpu surface 管理、场景渲染、GPU glyph 合成 | 659 | — |

**尚未实现的关键部分**:
- `render-foundation`: swash 字体整形、图片加载/缓存、实际渲染管线（布局盒 → 渲染命令转换）
- `host-runtime`: 输入法支持、键盘事件转换
- 所有其他 crate: 仅有骨架占位代码

**OmniTerm 可复用资产清单**（待迁移）:

| OmniTerm 模块 | 功能 | 迁移目标 |
|---------------|------|----------|
| `omniterm-terminal-render` | 场景/Primitive/Backend 分层架构 | `render-foundation` |
| `omniterm-terminal-render-wgpu` | GPU glyph atlas、pane 缓存、wgpu 合成 | `render-foundation` |
| `omniterm-terminal-render-soft` | fontdue + swash 字体栈、软件渲染后备 | `render-foundation` |
| `omniterm-terminal-image` | 图片对象缓存与 GC 限制 | `render-foundation` |
| `omniterm-terminal-ffi` / `-wasm` | ABI 边界和 WASM 友好封装 | `wasm-sandbox` |

**剩余技术差距**:
1. CSS 解析器需要完全自建（MPL 排除所有现有方案）— M3 的核心创新点
2. 浏览器级样式/布局/渲染/脚本集成是最大的技术空白
3. 多进程架构和安全沙箱需要平台特定的系统级编程
4. V8 与 Rust 的桥接（rusty_v8）在生产环境中的可靠性未经验证
5. `run-benchmarks.sh` 脚本路径有误，需要修复

### 8.2 目标状态（Target State）

**整体架构图**:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Browser Application                         │
│                    (apps/browser — browser-shell crate)             │
│   ┌──────────┬──────────┬──────────┬──────────┬──────────────────┐  │
│   │ Tab Mgr  │ Nav Bar  │ Bookmark │ History  │ Settings/Menu    │  │
│   └──────────┴──────────┴──────────┴──────────┴──────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│                      WebView API (webview crate)                │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐               │
│   │  Navigate    │ │ Script Bridge│ │  Surface     │               │
│   │  Control     │ │  (Rust↔JS)   │ │  Output      │               │
│   └──────────────┘ └──────────────┘ └──────────────┘               │
├─────────────────────────────────────────────────────────────────────┤
│                      Engine Core (engine crate)                │
│   ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌─────────┐  │
│   │   DOM   │ │ CSS Parse│ │  Style   │ │  Layout  │ │  Paint  │  │
│   │ (html5- │ │ (自建)   │ │ System   │ │ (taffy+  │ │ + Compo-│  │
│   │  ever)  │ │          │ │ (自建)   │ │  自建)   │ │  site)  │  │
│   └─────────┘ └──────────┘ └──────────┘ └──────────┘ └─────────┘  │
│   ┌──────────────────┐ ┌───────────────────────────────────────┐   │
│   │  JS Engine (V8)  │ │         Navigation Model              │   │
│   │  + DOM Bindings  │ │  (URL, History, Same-Origin)          │   │
│   └──────────────────┘ └───────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│                     Supporting Crates                               │
│   ┌──────────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐     │
│   │    Canvas    │ │   Net    │ │ Security │ │   Storage    │     │
│   │    (2D)      │ │(hyper/   │ │ (CORS/   │ │ (localStorage│     │
│   │              │ │ reqwest) │ │ CSP/SOP) │ │ /IndexedDB)  │     │
│   └──────────────┘ └──────────┘ └──────────┘ └──────────────┘     │
│   ┌──────────────┐ ┌──────────────────────────────────────────┐    │
│   │  Protocol    │ │  Script Sandbox (QuickJS feature gate)   │    │
│   │  (IPC)       │ │  WASM Sandbox (Wasmtime/wasmi)          │    │
│   └──────────────┘ └──────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                     Infrastructure                                  │
│   ┌──────────────────┐ ┌──────────────────────────────────────┐    │
│   │  Host Runtime    │ │  Render Foundation                    │    │
│   │  (winit + IME)   │ │  (wgpu GPU + CPU fallback + Fonts)   │    │
│   └──────────────────┘ └──────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

**多进程架构**:

```
┌─────────────────────────────────────────┐
│           Browser Process               │
│  ┌────────────┐  ┌──────────────────┐   │
│  │ Window Mgr │  │  Network Mgr     │   │
│  │ (winit)    │  │  (hyper/reqwest) │   │
│  └────────────┘  └──────────────────┘   │
│  ┌────────────┐  ┌──────────────────┐   │
│  │ Tab Mgr    │  │  Storage Mgr     │   │
│  └────────────┘  └──────────────────┘   │
│  ┌────────────────────────────────────┐  │
│  │     IPC (protocol crate)           │  │
│  └───────┬───────────────┬────────────┘  │
└──────────┼───────────────┼──────────────┘
           │               │
    ┌──────┴──────┐  ┌─────┴───────┐
    │  Renderer   │  │  Renderer   │
    │  Process #1 │  │  Process #2 │
    │ ┌─────────┐ │  │ ┌─────────┐ │
    │ │  V8/JS  │ │  │ │  V8/JS  │ │
    │ │ DOM     │ │  │ │ DOM     │ │
    │ │ Layout  │ │  │ │ Layout  │ │
    │ │ Paint   │ │  │ │ Paint   │ │
    │ └─────────┘ │  │ └─────────┘ │
    └─────────────┘  └─────────────┘
```

**关键变更**:
1. 从零构建 Cargo workspace，包含 15+ crate
2. 从 OmniTerm 迁移渲染基础设施到 `render-foundation`
3. 自建 CSS 解析器和样式系统（核心创新点）
4. 基于 taffy 扩展布局引擎
5. 集成 V8（rusty_v8）实现 JavaScript 运行时
6. 实现多进程架构（浏览器进程 + 多渲染进程）
7. 构建完整的安全模型

### 8.3 影响分析

| 影响项 | 影响级别 | 描述 |
|--------|----------|------|
| OmniTerm 项目 | 中 | 代码复用需要提取和重写，可能影响 OmniTerm 维护 |
| Rust 生态系统 | 低 | 使用成熟 crate（html5ever、wgpu、taffy、rusty_v8），无生态影响 |
| 目标平台系统 API | 高 | 沙箱机制需要平台特定的系统调用（seccomp/sandbox-exec/Job Objects） |
| V8 二进制分发 | 中 | V8 二进制约 30-50MB，影响最终产物大小 |
| CI/CD 基础设施 | 中 | 需要三平台 CI、基准测试持久化、覆盖率追踪 |

### 8.4 详细设计

#### 8.4.1 模块设计

**Crate 依赖关系**（自底向上）:

```
webview ──→ engine ──→ dom
              │              ├──→ css-parser
              │              ├──→ style-system ──→ css-parser
              │              ├──→ layout-engine ──→ taffy
              │              ├──→ canvas ──→ render-foundation
              │              ├──→ net ──→ hyper, reqwest
              │              ├──→ security
              │              └──→ storage
              ├──→ host-runtime ──→ winit
              ├──→ render-foundation ──→ wgpu, fontdue, swash
              ├──→ protocol
              ├──→ script-sandbox ──→ rusty_v8 / rquickjs (feature gate)
              └──→ wasm-sandbox ──→ wasmtime / wasmi

browser-shell ──→ webview
apps/browser ──→ browser-shell
apps/webview-demo ──→ webview
```

#### 8.4.2 数据模型

**DOM 树核心数据结构**（伪代码）:

```
struct Document {
    node_map: SlotMap<NodeId, Node>,
    root: NodeId,
    quirks_mode: QuirksMode,
}

enum Node {
    Element(ElementData),
    Text(TextData),
    Document(DocumentData),
    Comment(CommentData),
    DocumentType(DocumentTypeData),
    DocumentFragment(DocumentFragmentData),
}

struct ElementData {
    tag: QualName,
    attributes: Vec<Attribute>,
    children: Vec<NodeId>,
    parent: Option<NodeId>,
    style: Option<ComputedStyles>,
    layout_box: Option<LayoutBoxId>,
}
```

**渲染管线数据流**:

```
HTML Text → html5ever → DOM Tree
                          ↓
CSS Text → css-parser → CSS AST → style-system → Computed Styles
                                                     ↓
                              DOM + Computed Styles → layout-engine → Layout Box Tree
                                                                          ↓
                                                     Layout Boxes → Paint → Display List
                                                                          ↓
                                                    Display List → Compositor → GPU/CPU Render → Pixels
```

#### 8.4.3 API 设计

**WebView 嵌入 API**（Rust 接口签名伪代码）:

```
// 创建 WebView
let webview = WebViewBuilder::new()
    .with_size(800, 600)
    .with_surface(wgpu_surface)
    .build()?;

// 导航
webview.navigate("https://example.com")?;

// 执行 JS
let result: JsValue = webview.evaluate_script("document.title")?;

// 注册回调
webview.on_callback("onPageLoaded", |ctx| {
    let url: String = ctx.get_arg(0)?;
    println!("Page loaded: {}", url);
    Ok(())
});

// 事件循环
webview.run()?;
```

**多进程 IPC 消息协议**（伪代码）:

```
enum IpcMessage {
    // Browser → Renderer
    CreateView { url: Url, view_id: ViewId },
    Navigate { view_id: ViewId, url: Url },
    DestroyView { view_id: ViewId },

    // Renderer → Browser
    ViewReady { view_id: ViewId, title: String },
    ViewPainted { view_id: ViewId, display_list: DisplayList },
    ViewError { view_id: ViewId, error: RenderError },

    // Network 代理
    FetchRequest { request_id: RequestId, url: Url, headers: Headers },
    FetchResponse { request_id: RequestId, response: Response },
}
```

#### 8.4.4 主要流程

**页面加载与渲染流程**:

```
1. 用户输入 URL 或点击链接
2. Browser Process 解析 URL，分配/复用 Renderer Process
3. 通过 IPC 发送 CreateView/Navigate 命令
4. Renderer Process 执行：
   a. Net 模块发起 HTTP 请求获取 HTML
   b. html5ever 解析 HTML → 构建 DOM 树
   c. 识别外部资源（CSS/JS/图片），并行加载
   d. css-parser 解析 CSS → style-system 计算样式
   e. layout-engine 基于样式计算布局
   f. Paint 模块生成 Display List
   g. Compositor 合成并输出像素
5. 通过 IPC 将渲染结果发送回 Browser Process
6. Browser Process 将像素输出到窗口 Surface
7. 如有 JS 执行，V8 在 Renderer Process 内运行
8. JS 触发 DOM 变更 → 增量样式/布局/渲染
```

### 8.5 安全考量

**访问控制**:
- 渲染进程运行在沙箱中，限制文件系统、网络和进程访问
- 浏览器进程负责权限管理和资源调度
- 站点隔离：跨站 iframe 在独立渲染进程中

**数据保护**:
- Cookie 安全属性强制执行（Secure、HttpOnly、SameSite）
- 同源策略阻止跨域数据访问
- CSP 阻止未授权的资源加载和脚本执行
- TLS 证书校验确保传输安全

**潜在风险与缓解**:

| 风险 | 可能性 | 影响 | 缓解策略 |
|------|--------|------|----------|
| V8 漏洞被利用 | 中 | 高 | 进程沙箱 + 站点隔离 + 及时更新 V8 |
| CSS 解析器 DoS | 中 | 中 | 输入大小限制 + 解析超时 |
| IPC 消息伪造 | 低 | 高 | 进程间身份验证 + 消息签名 |
| 渲染进程资源耗尽 | 中 | 中 | 内存/CPU 限制 + 崩溃恢复 |
| 第三方 crate 漏洞 | 中 | 中 | 定期依赖审计 + `cargo audit` |

### 8.6 备选方案

#### 方案比较表

| 维度 | 方案 A: 自建内核（选定） | 方案 B: Servo/MPL 集成 | 方案 C: 系统 WebView 壳 |
|------|--------------------------|------------------------|------------------------|
| 实现复杂度 | 🔴 极高 | 🟡 中 | 🟢 低 |
| 许可证合规 | 🟢 完全合规 | 🔴 MPL 冲突 | 🟢 依赖系统组件 |
| 自主可控 | 🟢 完全可控 | 🟡 受 MPL 约束 | 🔴 依赖系统实现 |
| WebView 可嵌入 | 🟢 原生设计 | 🟡 需要适配 | 🟡 受限于系统 API |
| 技术风险 | 🔴 高（自建 CSS 等） | 🟡 中 | 🟢 低 |
| Web 标准覆盖 | 🟡 渐进覆盖 | 🟢 覆盖面广 | 🟢 取决于系统 |
| 跨平台一致性 | 🟢 一致 | 🟢 一致 | 🔴 平台差异大 |
| **推荐度** | ⭐⭐⭐ | ⭐⭐ | ⭐ |

**图例**: 🟢 优秀 | 🟡 一般 | 🔴 较差

#### 详细对比

| 方案 | 描述 | 优势 | 劣势 | 决策 |
|------|------|------|------|------|
| 方案 A | 基于 permissive Rust 模块自建页面内核 | 许可证合规、完全自主可控、可嵌入设计、跨平台一致 | 实现复杂度极高、技术风险高、CSS 解析器需从零构建 | ✅ 选定 |
| 方案 B | 基于 Servo 集成，复用 Stylo/WebRender | 实现复杂度低、Web 标准覆盖广、社区支持 | MPL 许可证冲突、不满足项目许可证策略、自主可控受限 | ❌ 排除 |
| 方案 C | 使用系统 WebView（WebView2/WebKitGTK/WKWebView） | 实现复杂度最低、即时可用 | 无法实现「自有内核」目标、跨平台不一致、嵌入受限 | ❌ 排除 |

**最终选择**: 方案 A — 自建内核

**理由**:
1. MPL 许可证与项目的主线许可证策略根本冲突，排除方案 B
2. 系统 WebView 壳无法满足「可复用 WebView 内核库」的核心目标，排除方案 C
3. OmniTerm 渲染基础设施可复用，降低了渲染管线的实现成本
4. Rust 生态中已有关键基础模块（html5ever、taffy、wgpu、rusty_v8），自建的缺口集中在 CSS 和集成层

### 8.7 实施计划

**按里程碑严格顺序执行**:

1. **M1**: 建立 Cargo workspace，迁移 OmniTerm 渲染核心，搭建 CI
2. **M2**: 集成 html5ever，实现 DOM 树和树操作
3. **M3**: 自建 CSS 解析器和样式系统（核心创新点，可能需要多次迭代）
4. **M4**: 基于 taffy 扩展布局引擎，实现 Block/Inline/Flexbox/Grid
5. **M5**: 连接布局 → 渲染管线，实现首屏渲染，建立 WPT 基础设施
6. **M6**: 集成 V8，实现 JS 执行和 DOM API 绑定
7. **M7**: 实现网络栈，开始加载真实网页
8. **M8**: 实现多进程架构和安全沙箱
9. **M9**: 补全 Canvas 2D、Web Workers、Storage
10. **M10**: WebView API 稳定化，提供嵌入示例
11. **M11**: 构建完整浏览器应用
12. **M12-M14**: 高级能力、性能优化、生产化

**风险缓解**:
- M3（CSS 解析器）是最大风险点 — 考虑渐进实现，先支持 Tier 1 必要属性
- M6（V8 集成）需要充分验证 rusty_v8 的可靠性 — 早期进行 PoC
- M8（多进程架构）需要平台特定代码 — 每个平台单独测试和验证

### 8.8 测试策略

- **单元测试**: 每个 crate 的核心逻辑，覆盖正常路径、边界条件、错误恢复。强制要求测试与代码同步编写。
- **集成测试**: 跨 crate 交互，关键路径（HTML → DOM → CSS → Layout → Render → Pixel）必须有集成测试。
- **性能基准**: 从 M1 起每个 crate 的关键路径必须有 criterion 基准，结果持久化到 `tests/benchmarks/results/`。
- **WPT 测试**: 从 M5 开始建立 WPT 运行基础设施，按分类追踪通过率。
- **兼容性测试**: 从 M7 开始维护 Top N 真实网站兼容性矩阵。
- **端到端测试**: 从 M11 开始自动化 UI 测试。

### 8.9 回滚计划

由于项目从零开始，没有需要回滚的历史系统。回滚策略主要针对开发过程：

1. **Crate 级回滚**: 每个 crate 独立版本管理，可通过 `Cargo.toml` 降级
2. **里程碑级回滚**: 每个里程碑完成后打 git tag，可回退到任意里程碑
3. **渲染后备**: GPU 渲染失败时自动降级到 CPU 软件渲染
4. **进程恢复**: 渲染进程崩溃后自动重建，不影响其他标签页

---

## 9. TBD 清单

| ID | 项目 | 优先级 | 缺失信息 | 后续步骤 |
|----|------|--------|----------|----------|
| TBD-1 | MSRV（最低支持 Rust 版本）策略 | ~~已解决~~ | 已确定 MSRV 为 Rust 1.85 | ✅ 已在 Cargo.toml 中配置 `rust-version = "1.85"` |
| TBD-2 | OmniTerm 代码复用许可证 | 重要 | 需确认 OmniTerm 的许可证与本项目兼容；假设同一团队/组织可复用 | 启动 M1 前确认许可证或获取授权 |
| TBD-3 | V8 二进制分发策略 | 重要 | V8 二进制体积大（30-50MB），分发方式未定 | 评估 rusty_v8 的分发机制，确定是否需要自建分发 |
| TBD-4 | CSS 解析器性能目标 | 重要 | 自建 CSS 解析器的性能基线未知 | M3 开始时建立解析性能基线，与现有方案对比 |
| TBD-5 | 多进程进程间通信机制选型 | 重要 | 未确定使用 Unix socket、named pipe 还是共享内存 | M8 开始时评估各方案性能和跨平台兼容性 |
| TBD-6 | Android NDK 适配范围 | 可选 | Android 适配的具体 API level 和 NDK 版本 | M14 前确定 Android 适配范围 |
| TBD-7 | 鸿蒙 PC SDK 可用性 | 可选 | 鸿蒙 PC 的 Rust 工具链和 SDK 状态未知 | 持续跟踪鸿蒙 PC 开发者工具链进展 |
| TBD-8 | 第三方 crate 审计策略 | 重要 | 未确定依赖安全审计的频率和工具 | 引入 `cargo audit` 和 `cargo deny` 到 CI |
| TBD-9 | 浏览器 UI 框架选型 | 重要 | 浏览器应用 UI（标签页、地址栏等）的渲染方案未定 | 评估使用 wgpu 自绘 vs egui/iced 等方案 |
| TBD-10 | 国际化（i18n）策略 | 可选 | 浏览器应用是否需要多语言支持 | M11 前确定 i18n 范围和方案 |

---

## 10. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.4 | 2026-08-07 | C-003 补充决策记录（B2）：单默认 JS 引擎为有意决策，引用 Ladybird Swift 教训与双引擎等价成本；trait 抽象预留但不承诺等价 |
| v1.3 | 2026-05-30 | 更新 §8.1 As-Is 分析：wgpu GPU 渲染后端已在 render-foundation gpu 模块实现（atlas.rs、pipeline.rs、renderer.rs）；host-runtime 新增 run_with_window() 用于 GPU surface 创建；Demo 切换到 wgpu GPU 渲染路径；代码规模增至 3,616 行 / 32 文件 / 69 测试 / 零 clippy 警告；从「尚未实现」列表移除 wgpu GPU 后端和 GPU surface 创建 |
| v1.2 | 2026-05-30 | 更新 §8.1 As-Is 分析以反映 M1 代码进展（2,112 行源码、55 测试、5 基准）；补充 render-foundation 和 host-runtime 实现细节；标注 run-benchmarks.sh 路径问题 |
| v1.1 | 2026-05-30 | 状态更新为 Confirmed；解决 TBD-1（MSRV = Rust 1.85）；更新 C-008 约束 |
| v1.0 | 2026-05-30 | 初始版本 — 基于目标文档 `docs/goal/zero-web.md` v1.0 和技术调研文档 `docs/research/rust-cross-platform-browser-research.md` 创建完整的 Spec + RFC |
