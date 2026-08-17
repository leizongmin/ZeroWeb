# ZeroWeb: 基于 Rust 的跨平台浏览器 — 目标执行契约

**版本**: v1.1
**日期**: 2026-05-30（2026-08-04 状态刷新）
**状态**: Active
**执行模式**: 长期无人值守持续执行（rally run）

> **▶ 恢复推进裁决（2026-08-04 用户决策）**：工作从 rendering-compat 切回父目标，恢复「下一步优先级」P1 DOM/JS Bridge 原生化（P1a 事件循环补全 + fetch/MutationObserver 真实化；P1b V8 原生绑定需独立 RFC）。渲染兼容性降频守成（低频 plateau-guard，深结构等用户点名，见 `rendering-compat.md` 顶部裁决）。本文件基线信息自 2026-05-30「空仓库」刷新至 2026-08-04 实际状态（17 crate + 3 apps 实质实现，~13,192 测试全绿）。

> **说明**
> 本文是 ZeroWeb 的长期目标执行契约，用于定义目标状态和自动推进条件；它不是当前仓库已达到生产可用、可商用或其他生产级别的声明。当前项目仍是实验性项目，默认仅供学习、研究和工程探索使用。

---

## Mission

长期目标是从零构建一个生产可用的、基于 Rust 的跨平台浏览器体系。最终交付三件产物：

1. **可复用的 `webview` 内核库** — 其他 Rust 应用可以 `lib` 方式直接集成，获得完整的网页渲染与交互能力。
2. **跨平台浏览器应用** — 支持 macOS、Linux、Windows 的桌面浏览器，具备多标签页、收藏夹、地址栏、历史记录等完整浏览器体验；Android 和鸿蒙 PC 作为后续平台适配目标，但架构设计必须从第一天预留。
3. **配套基础设施** — 嵌入 API、渲染基础设施、脚本/WASM 沙箱能力与工程化发布产物。

技术路线：基于 permissive 第三方 Rust 模块（html5ever、rusty_v8、wasmtime、wgpu、winit、taffy、hyper/reqwest）自建页面内核，自行实现 DOM、CSSOM、样式系统、布局整合、渲染管线、导航模型与安全边界。**排除所有 MPL 技术线**（servo、stylo、webrender、rust-cssparser、lightningcss、mozjs）。

代码复用：从 OmniTerm 终端项目复制核心渲染/宿主模块到本仓库并重写适配，包括 GPU 渲染器（wgpu）、CPU 软件渲染器、字体渲染栈、图片缓存、场景/Primitive/Backend 分层架构。后续可考虑抽取为共享 crate。

---

## Support Envelope

### 目标平台

| 平台 | 优先级 | 说明 |
|------|--------|------|
| macOS (x86_64 + aarch64) | P0 Day-1 | 桌面首发平台 |
| Linux (x86_64) | P0 Day-1 | 桌面首发平台 |
| Windows (x86_64) | P0 Day-1 | 桌面首发平台 |
| Android (aarch64) | P1 后续 | 移动端适配，架构预留 |
| 鸿蒙 PC | P2 后续 | 高风险平台，架构预留，暂不首发 |

### Web 标准覆盖范围

| 领域 | 核心依赖 | 自建部分 | 首期目标 |
|------|----------|----------|----------|
| HTML 解析 | `html5ever` (Apache-2.0/MIT) | DOM 树、增量更新、mutation observer | WHATWG HTML 合规解析 |
| CSS 解析 | 无（MPL 排除） | **完全自建** CSS parser、选择器引擎、级联、继承、计算值 | 首期：选择器 + 基础属性（盒模型、颜色、字体、定位、flexbox）；逐步扩展到 Grid、动画、媒体查询 |
| 布局 | `taffy` (MIT) — Flexbox/Grid/Block 算法 | 布局整合层（inline formatting、fragmentation、tables、positioning、stacking contexts、scrolling、painting order） | 首期：block + inline + flexbox；逐步扩展 |
| JavaScript | `rusty_v8` (MIT) | DOM bindings、GC 协调、Web API host hooks、event loop | V8 为唯一稳定页面 JS 引擎；QuickJS 仅作为 feature-gated 扩展脚本沙箱 |
| WASM | `wasmtime` (Apache-2.0) / `wasmi` (MIT/Apache-2.0) | 页面 WASM 与 JS/DOM 集成 | 非页面插件沙箱先行；页面 WASM 后续 |
| 渲染 | `wgpu` (Apache-2.0/MIT) | 渲染管线、合成、离屏渲染 | 复用 OmniTerm GPU/CPU 双渲染路径 |
| 窗口/平台 | `winit` (Apache-2.0) | 平台宿主层、事件循环、输入法 | 桌面先行 |
| 网络 | `hyper` / `reqwest` | 导航模型、同源策略、缓存、Cookie | HTTP/HTTPS 基础能力 |

### 非目标（明确排除）

- **不基于 Servo / MPL 技术线构建** — servo、stylo、webrender、rust-cssparser、lightningcss、mozjs 不进入主线依赖
- **首期不做移动端发布** — Android 和鸿蒙 PC 是后续适配，不是首发阻塞
- **首期不做完整 DevTools** — 控制台日志和基础审查先行，完整 DevTools 面板后续
- **首期不做媒体播放** — `<video>`/`<audio>` 播放能力后续，首期只处理布局占位
- **不追求首期双 JS 引擎等价** — V8 是唯一稳定页面引擎，QuickJS 仅用于扩展沙箱
- **首期不做浏览器扩展系统** — WebExtension API 支持后续
- **首期不做 WebRTC** — 实时通信后续
- **首期不做 WebGL/WebGPU** — 3D 图形 API 后续，Canvas 2D 是必须的

### 渐进标准路线（"最新标准"如何分阶段达到）

Web 标准覆盖面极广，"最新标准"不可能在一个里程碑中完成。以下是渐进覆盖策略：

**Tier 1 — 生产必需（Done Criteria 必须覆盖）**：
- HTML：完整解析 + DOM + Shadow DOM（基础）+ iframe + 表单
- CSS：选择器全量 + 盒模型 + 排版（block/inline/flexbox/grid）+ 颜色 + 字体 + 定位 + overflow + transforms + transitions + 自定义属性 + 媒体查询 + 逻辑属性
- JS/DOM：ES2023 基础（V8 自带）+ DOM 操作 + 事件 + Canvas 2D + Fetch + WebSocket + Web Workers + ES Modules + Storage
- 网络：HTTP/HTTPS + 资源加载 + Cookie + 缓存 + 重定向 + CORS
- 安全：同源策略 + CORS + CSP + TLS + Cookie 安全 + 渲染进程沙箱

**Tier 2 — 显著提升兼容性（应尽快覆盖）**：
- CSS：`@layer` + Container Queries + `:has()` + Subgrid + `scroll-snap` + View Transitions
- JS/Web API：IndexedDB + Service Worker（基础）+ IntersectionObserver + Clipboard API + Fullscreen API + Drag & Drop + Web Components（Custom Elements + Shadow DOM）
- HTML：`<template>` + `<slot>` + `<dialog>` + `<details>/<summary>` + `<picture>`
- 存储：IndexedDB + Cache API + OPFS

**Tier 3 — 完整体验（持续扩展）**：
- Canvas 2D 完整 API（Path2D、OffscreenCanvas、ImageBitmap）
- WebGL/WebGPU
- `<video>`/`<audio>` 播放 + Media Source Extensions
- WebRTC
- WebAssembly Component Model
- WebExtension API
- 可访问性（ARIA + 屏幕阅读器集成）
- 完整 DevTools
- Notification API + Push API

---

## Done Criteria

以下条件**全部满足**时，方可输出 `DONE`。任何一项未满足，必须输出 `CONTINUE: <下一步>`。

### 1. WebView 库达到可嵌入级别

- [ ] `webview` crate 可以被其他 Rust 项目以 `lib` 方式引入
- [ ] 可以加载 URL 并渲染 HTML 页面（文本、图片、基础 CSS 布局）
- [ ] 可以执行页面 JavaScript（V8），支持基础 DOM 操作
- [ ] 提供稳定的嵌入 API：导航控制、脚本桥、渲染表面输出、事件回调
- [ ] 多进程架构运行正常（浏览器进程 + 渲染进程分离）
- [ ] 至少 Top 20 静态网站可以正确加载渲染

### 2. 浏览器应用达到日常可用级别

- [ ] 在 macOS / Linux / Windows 上可以编译运行
- [ ] 支持多标签页（创建、关闭、切换）
- [ ] 支持地址栏输入 URL 并导航
- [ ] 支持前进/后退导航
- [ ] 支持收藏夹（添加、删除、管理、点击导航）
- [ ] 支持基础的历史记录
- [ ] 能加载并正确渲染真实网页（非仅本地测试页面）

### 3. Web 标准兼容性（Tier 1 覆盖）

- [ ] HTML：通过 html5ever 解析，DOM 树正确构建，支持 iframe 渲染
- [ ] CSS：自建 parser 支持完整选择器、盒模型、颜色、字体、定位、flexbox、grid、overflow、transforms、transitions、自定义属性、媒体查询、逻辑属性
- [ ] JS/DOM：V8 集成，支持完整 DOM Level 2+ 核心 API（querySelector、addEventListener、innerHTML、DOM 树操作等）
- [ ] Canvas 2D：`<canvas>` 元素可以渲染 2D 图形（路径、矩形、文本、图像绘制、变换、合成）
- [ ] 网络：HTTP/HTTPS 请求、HTML/CSS/JS/资源加载、Fetch API
- [ ] 导航：链接点击、表单提交、同源策略实施
- [ ] WebSocket 基础通信
- [ ] Web Workers（基本的 Dedicated Worker）
- [ ] ES Modules（`<script type="module">` 支持）
- [ ] Storage：localStorage / sessionStorage / IndexedDB 基础
- [ ] 安全：CORS、CSP 基础、同源策略、Cookie 安全属性（Secure/HttpOnly/SameSite）、渲染进程沙箱

### 4. 性能基准体系

- [x] 每个 crate 的关键路径有 criterion 基准测试（HTML 解析、CSS 计算、布局、绘制、Canvas、JS→DOM 桥调用）——79 个基准函数覆盖 16/16 crate（2026-08-08 现状）
- [x] 所有基准测试可通过 `cargo bench` 一键运行，结果持久化到 `tests/benchmarks/results/`——`make bench`（2026-08-08：bench-report.sh 产出 JSON + 人读 txt）
- [x] 中等复杂度页面首屏渲染 < 2 秒（含各阶段耗时分解：解析 → 样式 → 布局 → 绘制 → 合成）——perf-gate Hard Gate（`page/*/total_ms` p95 ≤ 2000ms 绝对）+ 既有集成测试双重断言
- [x] 增量渲染：DOM 局部变更的重新渲染耗时 < 全量渲染的 20%——`tests/integration/src/e2e_rendering.rs` 常驻断言（`make test` 内）
- [ ] GPU 加速合成正常工作——待 GPU/Display 环境验证（历史遗留）
- [x] 回归门禁就位：基准结果 p95 不超过历史最佳基线的 120%——perf-gate 体系（Budget Gate：微基准 ×1.20、页面 ×1.15+40ms、RSS ×1.20+128MB；Hard Gate 绝对预算；公式与流程见 docs/specs/performance-and-resource-budget.md）
- [x] 基准报告可追踪趋势（每轮执行记录对比数据）——`docs/perf/trends/benchmark-trend.csv` + 日快照 + weekly CI 回写与 auto-tighten

### 5. 单元测试与质量

- [ ] 所有 crate 编译无错误、无警告（`cargo build` + `cargo clippy`）
- [ ] 所有现有测试持续绿色（`cargo test` 全通过）
- [ ] **每个 crate 都有完善的单元测试**：覆盖正常路径、边界条件、错误恢复、一致性校验
- [ ] 单元测试数量与代码量比例：核心 crate（dom、css-parser、style-system、layout-engine、canvas、security）测试行数 ≥ 实现行数的 30%
- [ ] 核心模块行覆盖率 ≥ 70%，非核心 crate ≥ 50%
- [ ] 至少一个集成测试可以端到端加载真实 URL 并验证渲染输出
- [ ] WPT 测试基础设施就位并可运行，有按分类的通过率追踪

### 6. 工程化

- [ ] CI 管线可以自动编译、测试、运行 clippy、运行基准测试
- [ ] `scripts/run-benchmarks.sh` 一键运行所有基准并输出报告
- [ ] `scripts/check-coverage.sh` 一键测量并输出覆盖率摘要
- [ ] 代码组织清晰，按计划中的 crate 层次结构划分
- [ ] 所有 crate 有 README 和 API 文档（`cargo doc` 可生成）
- [ ] WebView 嵌入示例代码可编译运行

---

## Current Proven Baseline

截至 2026-08-04，项目已从调研阶段推进到实质实现阶段（详见 [master.md](zero-web/master.md)）：

- **仓库状态**：Cargo workspace，17 个 crate + 3 个应用全部有实质实现；~13,192 测试全绿（74 ignored 为网络型真实网站用例）；行覆盖率 95.46%（函数 96.94%、区域 94.88%）；clippy 零警告；CI 三平台（ubuntu/macos/windows）
- **已实现核心能力**：HTML 解析/DOM（html5ever，含 Shadow DOM 基础、MutationObserver、Range/TreeWalker、FocusManager 等）、完全自建 CSS parser + 样式系统（100+ 属性）、taffy 布局、GPU/CPU 双渲染管线（全 13 种图元）、V8 JS 引擎（DOM polyfill 桥接 + ES Modules + Web Workers + WASM 自动桥接）、多进程架构实际运行（IPC + 独立渲染进程二进制）、真实 WebSocket（tungstenite）、CSP 完整实现 + SecurityContext（HSTS/混合内容）、Top 55+ 真实网站兼容性测试、WPT 1341 用例（23 分类 100% 通过率）、可访问性基础、跨平台打包脚本
- **当前主要缺口**（详见 master.md「下一步优先级」）：① DOM/JS Bridge 为 polyfill 字符串桥接模式——Observer（Mutation/Intersection/Resize）为 stub 不触发回调、fetch() 为 stub 返回空 Response、事件循环为简化版非 spec-compliant（**P1a 修复中 = 当前活跃主线**）；② 渲染兼容性 chromium-Oracle 真一致 ~47.5%（属 rendering-compat 独立目标，深结构等用户点名）
- **渲染兼容性赛道**：已拆分为独立目标 `docs/goal/rendering-compat.md`（WPT reftest 驱动）
- **Canvas 2D 赛道**：独立目标已完成（2026-08-16，DC-1~4 全部满足：WPT 919 文件导入、testharness 全绿、oracle-pass 100%/不一致 0、Mission 中期 80% 达成）——入口文档与运行时面板已归档至 `docs/goal/archive/canvas-2d.md` + `docs/goal/archive/canvas-2d/`

---

## 项目分层架构

```
zero-web/
├── crates/
│   ├── webview/           # 面向外部应用的稳定嵌入接口
│   ├── engine/           # 页面内核：HTML/DOM/CSSOM/样式/布局/绘制/脚本
│   ├── host-runtime/          # 平台宿主：窗口、事件循环、surface、输入法
│   ├── render-foundation/     # 渲染基础设施：GPU/CPU 渲染、字体、图片缓存
│   ├── script-sandbox/        # 扩展/用户脚本引擎（V8/QuickJS feature gate）
│   ├── wasm-sandbox/          # 非页面 WASM 运行时（Wasmtime/wasmi）
│   ├── browser-shell/         # 浏览器应用：多标签页、收藏夹、地址栏、历史
│   ├── css-parser/            # 自建 CSS 解析器
│   ├── style-system/          # 自建样式系统（级联、继承、计算值）
│   ├── layout-engine/         # 布局引擎（基于 taffy 扩展）
│   ├── dom/                   # DOM 树实现
│   ├── net/                   # 网络栈（hyper/reqwest 封装）
│   ├── canvas/                # Canvas 2D 实现（渲染目标、路径、图像数据）
│   ├── security/              # 安全模型：CORS、CSP、同源策略、沙箱
│   ├── storage/               # 存储后端：localStorage、IndexedDB、Cache API
│   └── protocol/              # 多进程 IPC 与协议定义
├── apps/
│   ├── browser/               # 浏览器应用入口
│   └── webview-demo/          # WebView 嵌入示例
├── tests/
│   ├── wpt-runner/            # WPT 测试运行器
│   ├── integration/           # 端到端集成测试
│   └── benchmarks/            # 性能基准测试
└── docs/
    ├── goal/                  # 目标文档（本文件）
    ├── research/              # 技术调研
    └── specs/                 # Spec/RFC 文档
```

---

## Ordered Milestones

里程碑按严格依赖顺序排列。M1-M11 实质已完成并归档（详见 [master.md](zero-web/master.md) 归档记录与「当前仓库事实」）；M12-M14 为规划参考。当前活跃工作面 = **Done Criteria §3 的 JS/DOM 真实化（P1a）**，详见 master.md「下一步优先级」。

### M1: 项目骨架 + 渲染基础设施迁移（✅ 已完成，已归档）

**目标**：建立项目结构，迁移 OmniTerm 渲染基础设施，在桌面平台上显示一个窗口并渲染文本。同步建立测试和基准基础设施。

**交付物**：
- [ ] 完整的 Cargo workspace 结构，所有 crate 骨架就位
- [ ] `render-foundation` crate 从 OmniTerm 迁移并适配（GPU/CPU 双路径、字体栈、图片缓存）
- [ ] `host-runtime` crate 支持 winit 窗口创建和事件循环
- [ ] 可以在 macOS/Linux/Windows 上创建窗口，使用 wgpu 渲染文本（"Hello ZeroWeb"）
- [ ] 所有 crate 编译通过，`cargo clippy` 无警告
- [ ] `render-foundation` 单元测试：覆盖 glyph 渲染、脏区域检测、字体 fallback、GPU vs CPU 结果一致性（≥20 个测试用例）
- [ ] criterion 基准基础设施就位：`tests/benchmarks/` 目录结构、`scripts/run-benchmarks.sh`
- [ ] `render-foundation` 首批基准：glyph 渲染吞吐量、脏区域检测耗时（≥3 个基准）
- [ ] 覆盖率测量脚本就位：`scripts/check-coverage.sh`
- [ ] CI 管线就位（GitHub Actions：编译 + 测试 + clippy + 基准运行）

**验收标准**：
- `cargo build` 在三个桌面平台上成功
- `cargo test` 全通过，render-foundation 覆盖率 ≥ 50%
- `cargo bench` 可运行并输出结果
- 运行 demo 二进制可以看到窗口和渲染文本
- OmniTerm 渲染核心代码已迁移到本仓库

---

### M2: HTML 解析 + DOM 树

**目标**：基于 html5ever 构建完整的 DOM 树，支持文档解析和树操作。

**交付物**：
- [ ] `dom` crate 实现完整的 DOM 节点类型（Element、Text、Document、Comment 等）
- [ ] HTML 解析器集成 html5ever，生成 DOM 树
- [ ] DOM 修改 API（appendChild、removeChild、insertBefore 等）
- [ ] Mutation Observer 基础框架
- [ ] **单元测试**：覆盖所有节点类型、所有树操作、属性读写、HTML 解析错误恢复（≥50 个测试用例，覆盖率 ≥ 70%）
- [ ] **基准测试**：DOM 树构建（10k 节点）、querySelector（1000 元素）、批量 appendChild（≥3 个基准）

**验收标准**：
- 可以解析标准 HTML5 文档并生成正确的 DOM 树
- DOM 树操作（增删改查）全部通过测试
- 解析器能处理错误恢复（malformed HTML）
- `cargo bench` 输出 DOM 操作的基线数据

---

### M3: CSS 解析器 + 样式系统

**目标**：自建 CSS parser 和样式系统，支持选择器匹配、级联和计算值。

**交付物**：
- [ ] `css-parser` crate 实现完整的 CSS 语法解析（tokenizer + parser）
- [ ] 支持选择器解析（类型、类、ID、属性、伪类、伪元素、组合器、`:is()`/`:where()`/`:not()`）
- [ ] `style-system` crate 实现级联、继承、初始值、计算值
- [ ] 支持 CSS 属性：display、width/height、margin/padding/border、color、background、font、position、overflow、visibility、opacity、z-index、box-sizing、min/max、flexbox 全量、transform、transition、自定义属性（`--*`）、媒体查询、逻辑属性
- [ ] 样式系统与 DOM 集成，可以为 DOM 节点计算样式
- [ ] 支持 `@media`、`@supports`、`@layer`、`@import` 规则
- [ ] **单元测试**：覆盖所有选择器类型、所有已支持属性的解析与计算值、级联规则全路径、自定义属性、@规则（≥80 个测试用例，覆盖率 ≥ 70%）
- [ ] **基准测试**：CSS 解析吞吐量（100KB CSS）、选择器匹配（1000 元素 vs 100 选择器）、样式计算（1000 元素页面）（≥4 个基准）

**验收标准**：
- CSS parser 可以解析标准 CSS 文本并生成正确的 AST
- 选择器引擎可以正确匹配 DOM 节点（含复杂选择器）
- 级联规则正确应用（specificity、!important、继承、@layer）
- 计算值生成正确
- 自定义属性可以声明、引用、回退
- `cargo bench` 输出 CSS 解析和样式计算的基线数据

---

### M4: 布局引擎

**目标**：基于 taffy 扩展构建完整的布局引擎，支持 Block、Inline、Flexbox、Grid 布局。

**交付物**：
- [ ] `layout-engine` crate 实现布局树构建和布局计算
- [ ] Block layout（正常流块级布局）
- [ ] Inline layout（行内格式化上下文、文本换行、white-space 处理）
- [ ] Flexbox layout（集成 taffy，含 gap、align-self、order）
- [ ] CSS Grid layout（集成 taffy，含 grid-template、grid-area、gap）
- [ ] Positioned layout（relative、absolute、fixed、sticky）
- [ ] Overflow 和 scrolling 布局（scrollable overflow、clip）
- [ ] 布局输出为盒模型坐标，可供渲染管线消费
- [ ] **单元测试**：每种布局模式独立测试集（block/inline/flexbox/grid/positioned/overflow）、边界条件（0 宽高、极端嵌套、超大内容）、与 Chrome 截图对比（≥60 个测试用例，覆盖率 ≥ 70%）
- [ ] **基准测试**：各模式布局耗时（1000 元素）、增量重算 vs 全量重算、极端嵌套布局（≥5 个基准）

**验收标准**：
- 给定 DOM + 计算样式，可以生成正确的布局盒树
- Block/Inline/Flexbox/Grid 布局与浏览器表现一致（对比 Chrome 截图）
- Fixed/Sticky 定位正确工作
- 布局引擎有充足的测试用例覆盖各种布局场景
- `cargo bench` 输出各布局模式的基线数据

---

### M5: 渲染管线集成（首屏渲染）

**目标**：将布局输出连接到渲染管线，实现网页的首次像素级渲染。

**交付物**：
- [ ] 布局盒 → 渲染命令的转换（paint）
- [ ] 文本渲染（字体查找、glyph 渲染、文本排列、行高、text-overflow）
- [ ] 矩形/背景/边框渲染（含 border-radius、box-shadow）
- [ ] 图片渲染（`<img>` 加载与显示、object-fit）
- [ ] GPU 合成输出到窗口
- [ ] 增量渲染：脏矩形追踪，DOM 变更只触发局部重排/重绘
- [ ] GPU 加速合成层（transform、opacity 提升为独立合成层）
- [ ] 可以加载本地 HTML 文件并渲染出可视化页面
- [ ] WPT 测试基础设施搭建（可以运行 WPT 子集并报告通过率）
- [ ] **单元测试**：覆盖 paint 命令生成、脏矩形计算、合成层提升逻辑、文本/矩形/图片渲染正确性（≥30 个测试用例）
- [ ] **基准测试**：绘制命令生成耗时、GPU 合成帧耗时、增量 vs 全量渲染、端到端页面加载各阶段分解（≥4 个基准）
- [ ] **端到端基准**：中等复杂页面首屏渲染（含解析→样式→布局→绘制→合成的阶段耗时分解）

**验收标准**：
- 可以渲染包含文本、链接、图片、颜色、边框、圆角、阴影的基础网页
- 渲染输出与预期视觉效果匹配（截图对比）
- 性能达标：中等复杂页面首屏渲染 < 2 秒
- 增量渲染工作：修改 DOM 节点只触发局部更新
- WPT runner 可以运行 HTML 类测试并输出通过/失败统计
- 端到端基准输出各阶段耗时分解数据

---

### M6: JavaScript 集成（V8）

**目标**：集成 V8 引擎，实现 JS 执行和完整的 DOM API。

**交付物**：
- [ ] V8 引擎通过 `rusty_v8` 集成到 `engine`
- [ ] JS → DOM bindings（`document.getElementById`、`querySelector`/`querySelectorAll`、`innerHTML`/`textContent`、`createElement`、`setAttribute`、DOM 树操作全量）
- [ ] 事件系统（`addEventListener`、`removeEventListener`、事件冒泡/捕获、自定义事件、焦点事件、输入事件）
- [ ] HTML spec event loop 集成（microtask、task、requestAnimationFrame、requestIdleCallback）
- [ ] Web API：`console`、`setTimeout`/`setInterval`/`clearTimeout`/`clearInterval`、`Promise`、`Error`
- [ ] Fetch API 基础实现（`fetch()`、`Request`、`Response`、`Headers`）
- [ ] ES Modules 支持（`<script type="module">`、`import`/`export`、import maps）
- [ ] `<canvas>` 元素宿主（创建渲染上下文，连接到 `canvas` crate）
- [ ] **单元测试**：覆盖每个 DOM binding（读/写/调用）、事件系统全路径（冒泡/捕获/阻止/自定义）、event loop 时序、ES Module 加载（≥60 个测试用例）
- [ ] **基准测试**：Rust↔V8 桥调用开销（空函数、属性读写、DOM 操作）、事件分发延迟、ES Module 加载耗时（≥4 个基准）

**验收标准**：
- 页面内 JavaScript 可以正确执行
- JS 可以操作 DOM（增删改查节点、修改样式、读写属性）
- 事件系统可以正确触发和传播
- Fetch API 可以发起网络请求并获取响应
- ES Modules 可以正确加载和执行
- 可以运行交互式网页（表单验证、动态内容、AJAX）
- Rust↔JS 桥调用基线数据已记录

---

### M7: 网络栈 + 导航模型

**目标**：集成网络栈，实现 URL 导航、资源加载和安全基础。

**交付物**：
- [ ] `net` crate 基于 hyper/reqwest 实现 HTTP/HTTPS 请求
- [ ] HTML/CSS/JS/图片资源的加载与缓存
- [ ] URL 解析和导航模型（前进、后退、hash 变更、pushState/replaceState）
- [ ] `security` crate 实现同源策略、CORS 基础
- [ ] Cookie 管理（含 Secure、HttpOnly、SameSite 属性）
- [ ] 重定向处理
- [ ] TLS/HTTPS 证书校验
- [ ] WebSocket 协议基础支持（`new WebSocket()`）
- [ ] **单元测试**：覆盖 HTTP 请求/响应、URL 解析、导航历史、同源策略判断、CORS 预检/简单请求、Cookie 属性、重定向链、WebSocket 连接生命周期（≥50 个测试用例）
- [ ] **基准测试**：HTTP 请求延迟、资源加载吞吐量、URL 解析吞吐量、Cookie 处理耗时（≥4 个基准）

**验收标准**：
- 可以通过 URL 加载真实网页
- HTML 中的外部资源（CSS、JS、图片）可以正确加载
- 链接点击可以导航到新页面
- 同源策略阻止跨域不当访问
- CORS 预检请求和简单请求正确处理
- WebSocket 可以建立连接并收发消息
- 网络栈关键操作基线数据已记录

---

### M8: 多进程架构 + 安全沙箱

**目标**：实现浏览器进程和渲染进程的分离，建立基础安全边界。

**交付物**：
- [ ] `protocol` crate 定义多进程 IPC 协议
- [ ] 浏览器进程（Browser Process）：窗口管理、标签页管理、网络调度、存储调度
- [ ] 渲染进程（Renderer Process）：页面渲染、JS 执行、DOM 操作
- [ ] 进程间通信机制（命令/响应/流式传输）
- [ ] 渲染进程崩溃恢复（检测崩溃、自动重建、用户通知）
- [ ] 渲染进程沙箱：限制文件系统、网络、进程访问（Linux: seccomp-bpf；macOS: sandbox-exec；Windows: Job Objects）
- [ ] CSP (Content-Security-Policy) 基础实施（script-src、style-src、img-src 指令）
- [ ] iframe 基础渲染（嵌入其他页面、sandbox 属性基础支持）
- [ ] **单元测试**：覆盖 IPC 消息序列化/反序列化、进程启动/关闭、崩溃检测/恢复、沙箱越权拒绝、CSP 策略判断、iframe sandbox 属性解析（≥40 个测试用例）
- [ ] **基准测试**：IPC 往返延迟、大消息传输吞吐量、进程启动耗时（≥3 个基准）

**验收标准**：
- 浏览器进程和渲染进程分离运行
- 单个标签页崩溃不影响其他标签页
- IPC 通信稳定可靠
- 渲染进程无法越权访问文件系统
- CSP 阻止内联脚本执行（当策略配置时）
- iframe 可以嵌入并渲染另一个页面
- IPC 通信基线数据已记录

---

### M9: Canvas 2D + Web Workers + Storage

**目标**：实现 Canvas 2D 渲染、Web Workers 多线程和客户端存储。

**交付物**：
- [ ] `canvas` crate 实现 Canvas 2D API（CanvasRenderingContext2D：路径、矩形、圆弧、文本、图像绘制、变换矩阵、合成模式、渐变/图案、clip、save/restore、getImageData/putImageData）
- [ ] `<canvas>` 元素与 DOM/渲染管线完整集成
- [ ] Web Workers 基础支持（Dedicated Worker：`new Worker()`、`postMessage`、`onmessage`）
- [ ] `storage` crate 实现 localStorage / sessionStorage（完整的 Storage API）
- [ ] IndexedDB 基础实现（打开数据库、创建 object store、基础 CRUD 事务）
- [ ] CSSOM 基础（JS 读写元素样式、`getComputedStyle()`、操作 stylesheet）
- [ ] **单元测试**：Canvas 2D 每类 API 独立测试集（路径/矩形/文本/图像/变换/合成/像素操作）、Web Workers 生命周期与消息传递、localStorage/sessionStorage CRUD 与边界、IndexedDB 事务完整性（≥60 个测试用例，canvas crate 覆盖率 ≥ 70%）
- [ ] **基准测试**：1000 矩形绘制、文本渲染、getImageData/putImageData、Worker postMessage 吞吐量、localStorage 批量读写、IndexedDB 事务吞吐量（≥6 个基准）

**验收标准**：
- Canvas 2D 可以绘制路径、文本、图像，支持变换和合成
- Web Worker 可以在后台线程执行 JS 并与主线程通信
- localStorage/sessionStorage 可以持久化存储键值对
- IndexedDB 可以执行基础事务
- Canvas 性能基准：1000 个矩形渲染 < 16ms
- 所有新增 crate 基线数据已记录

---

### M10: WebView 库 API 稳定化

**目标**：`webview` crate 达到可嵌入级别，其他应用可以集成使用。浏览器应用将基于此 API 构建。

**交付物**：
- [ ] 稳定的 Rust API：创建 WebView、导航、注入 JS、回调
- [ ] 渲染表面输出（可嵌入其他应用的渲染上下文）
- [ ] 脚本桥（Rust ↔ JS 双向调用）
- [ ] 嵌入示例代码（`apps/webview-demo/`）
- [ ] API 文档和使用指南
- [ ] 确认 browser-shell 将作为 webview 的消费者构建

**验收标准**：
- 可以创建独立的 Rust 项目，添加 webview 依赖后即可嵌入网页渲染
- 嵌入 API 文档完整（`cargo doc` 生成）
- 至少一个嵌入示例可以编译运行
- webview-demo 可以加载 URL 并渲染交互式页面

---

### M11: 浏览器应用

**目标**：构建完整的桌面浏览器应用（基于 webview）。

**交付物**：
- [ ] 多标签页管理（创建、关闭、切换、拖拽排序）
- [ ] 地址栏（URL 输入、自动补全、加载进度指示）
- [ ] 导航控制（前进、后退、刷新、主页）
- [ ] 收藏夹（添加、删除、文件夹管理、收藏栏展示）
- [ ] 历史记录（记录访问、搜索、清除）
- [ ] 下载管理器（下载文件、进度显示、打开所在文件夹）
- [ ] 页面查找（Ctrl+F 搜索高亮）
- [ ] 缩放（Ctrl+/Ctrl- 缩放页面、重置）
- [ ] 右键上下文菜单（复制、粘贴、检查元素、图片操作）
- [ ] 基础设置页面（默认搜索引擎、主页、隐私设置）
- [ ] 三个桌面平台（macOS/Linux/Windows）上的原生体验

**验收标准**：
- 浏览器可以正常启动并加载首页
- 可以打开多个标签页并流畅切换
- 可以访问真实网站（如 GitHub、Wikipedia、MDN）
- 收藏夹和历史记录功能完整可用
- 下载管理器可以下载文件
- 页面查找和缩放功能正常
- 应用在三个桌面平台上运行稳定

---

### M12: 高级 Web 能力

**目标**：扩展 Web 标准支持，提升兼容性至 Tier 2 水平。

**交付物**：
- [ ] WASM 支持（Wasmtime 集成，页面 WASM 与 JS 互操作）
- [ ] 更多 DOM API（MutationObserver、IntersectionObserver、ResizeObserver、Clipboard API、Fullscreen API、Drag & Drop）
- [ ] `script-sandbox` crate 支持 QuickJS feature gate
- [ ] Service Worker 基础（注册、fetch 事件拦截、缓存管理）
- [ ] Cache API
- [ ] 持续 WPT 通过率追踪和提升

**验收标准**：
- 页面 WASM 可以加载执行并与 JS 互操作
- Observer API 可以正确触发回调
- Service Worker 可以拦截 fetch 请求
- 扩展脚本沙箱可以通过 feature gate 启用 QuickJS
- WPT 通过率有可追踪的提升趋势

---

### M13: 性能优化 + 安全加固

**目标**：系统性能优化和安全模型深化。

**交付物**：
- [ ] 渲染管线优化（GPU 批处理、纹理 atlas 优化、减少 draw call）
- [ ] 布局增量计算（dirty bit 标记、只重算受影响子树）
- [ ] JS 执行优化（V8 快照预热、减少 Rust↔JS 桥开销）
- [ ] 资源预加载（speculative parsing、`<link rel="preload">`）
- [ ] 站点隔离（跨站 iframe 在独立渲染进程中）
- [ ] CSP 完整实现（所有主要指令、report-only 模式）
- [ ] Mixed Content 阻止（HTTPS 页面阻止 HTTP 子资源）
- [ ] HSTS 支持
- [ ] 权限模型基础（摄像头/麦克风/定位/通知的权限请求 UI）

**验收标准**：
- Speedometer 或等效基准有基线数据且不退化
- 增量布局性能显著优于全量重算
- 站点隔离下跨站 iframe 无法访问父页面 DOM
- CSP 可以有效阻止违规资源加载
- 权限提示正常弹出并可记住用户选择

---

### M14: 生产化与平台扩展

**目标**：发布准备、可访问性、移动平台适配。

**交付物**：
- [ ] 可访问性基础（ARIA 属性传递、键盘导航、焦点管理、高对比度模式）
- [ ] Android 平台适配（APK 打包、触摸输入、虚拟键盘、移动端 UI）
- [ ] 鸿蒙 PC 架构预留和初步适配（平台胶水层、输入法适配）
- [ ] 发布打包（macOS .app/.dmg、Linux .AppImage/.deb、Windows .exe 安装包）
- [ ] 自动更新机制基础
- [ ] 持续 WPT 通过率提升
- [ ] 真实网站兼容性矩阵（Top 100 网站逐个验证记录）

**验收标准**：
- 键盘可以完整导航浏览器界面
- Android 版本可以加载并渲染网页
- 各平台有可安装的发布包
- Top 100 网站兼容性矩阵有明确记录
- WPT 通过率持续提升

---

## Testing & Quality Gates

### 测试层次

| 层次 | 覆盖范围 | 工具 | 要求 |
|------|----------|------|------|
| **单元测试** | 每个 crate 的核心逻辑 | `cargo test` | **强制要求**：每个 crate 必须有单元测试模块；新功能必须同步添加测试；测试必须覆盖正常路径、边界条件和错误恢复 |
| **集成测试** | 跨 crate 交互 | `tests/integration/` | 关键路径（HTML→DOM→CSS→Layout→Render→Pixel）必须有集成测试 |
| **性能基准** | 关键操作耗时 | `criterion` | **强制要求**：从 M1 起每个 crate 的关键路径必须有基准；每轮追踪不退化；结果持久化到 `tests/benchmarks/results/` |
| **WPT 测试** | Web 标准合规性 | `tests/wpt-runner/` | 从 M5 开始建立，持续扩展覆盖 |
| **兼容性测试** | 真实网站加载 | 手动 + 自动截图对比 | 从 M7 开始维护 Top N 目标网站清单 |
| **端到端测试** | 完整浏览器使用流程 | 自动化 UI 测试 | 从 M11 开始 |

### 单元测试标准（强制）

**原则：测试与代码同步编写，不允许出现无测试的功能代码。**

每个 crate 的单元测试必须包含以下类别：

| 测试类别 | 说明 | 示例 |
|----------|------|------|
| **正常路径** | 标准输入的正确行为 | 解析有效 HTML 生成正确 DOM 树 |
| **边界条件** | 空输入、极大输入、极端值 | 解析空文档、0x0 布局、空字符串 CSS |
| **错误恢复** | 非法输入的容错行为 | 解析 malformed HTML 不 panic |
| **一致性校验** | 输出与预期参照一致 | 布局结果与 Chromium 截图对比 |
| **状态转换** | 修改后再验证 | DOM 增删后查询结果正确 |

**各 crate 最低测试要求**：

| Crate | 最低测试数量 | 关键测试场景 |
|-------|-------------|-------------|
| `dom` | 覆盖所有节点类型 + 所有树操作 + 增量更新 | 构建/遍历/修改/序列化完整 DOM 树 |
| `css-parser` | 覆盖所有选择器类型 + 所有已支持属性 + @规则 | 复杂选择器匹配、级联、自定义属性 |
| `style-system` | 覆盖级联全路径 + 继承 + 计算值 | specificity 竞争、!important 覆盖、初始值 |
| `layout-engine` | 覆盖所有布局模式 + 定位 + overflow | block/inline/flexbox/grid/sticky 各有独立测试集 |
| `render-foundation` | 覆盖 GPU/CPU 双路径 + 字体 fallback + 脏追踪 | GPU 渲染 vs CPU 渲染结果一致性 |
| `canvas` | 覆盖所有 2D API 分组 | 路径绘制、文本渲染、变换、合成、像素操作 |
| `net` | 覆盖 HTTP/HTTPS/CORS/Cookie/WebSocket | 请求/响应、重定向、跨域、Cookie 属性 |
| `security` | 覆盖同源策略/CORS/CSP/沙箱 | 跨域拒绝、CSP 阻止、沙箱隔离 |
| `storage` | 覆盖 localStorage/sessionStorage/IndexedDB | CRUD、容量限制、持久化 |
| `protocol` | 覆盖 IPC 消息序列化/反序列化/传输 | 命令/响应往返、崩溃恢复 |

### 性能基准标准（强制）

**原则：每个 crate 从创建起就建立关键路径的基准，为后续优化提供量化依据。**

**基准组织结构**：

```
tests/benchmarks/
├── benches/
│   ├── dom_bench.rs          # DOM 树构建、遍历、修改
│   ├── css_parser_bench.rs   # CSS 解析、选择器匹配
│   ├── style_bench.rs        # 样式计算、级联
│   ├── layout_bench.rs       # 各模式布局耗时
│   ├── render_bench.rs       # 绘制、合成
│   ├── canvas_bench.rs       # Canvas 2D 绘制操作
│   ├── net_bench.rs          # HTTP 请求、资源加载
│   └── e2e_bench.rs          # 端到端页面加载
├── results/                   # 持久化基准结果（JSON）
└── scripts/
    └── run-benchmarks.sh      # 一键运行所有基准并输出报告
```

**各 crate 最低基准要求**：

| Crate | 关键基准 | 首次建立里程碑 |
|-------|----------|---------------|
| `render-foundation` | glyph 渲染吞吐量、脏区域检测耗时、GPU draw call 数量 | M1 |
| `dom` | DOM 树构建（10k 节点）、查询（querySelector）、修改（批量 appendChild） | M2 |
| `css-parser` | CSS 解析吞吐量（100KB CSS）、选择器匹配（1000 元素 vs 100 选择器） | M3 |
| `style-system` | 样式计算耗时（1000 元素页面级联）、自定义属性解析 | M3 |
| `layout-engine` | Block/Inline/Flexbox/Grid 各模式（1000 元素）、增量重算 vs 全量重算 | M4 |
| `render-foundation` (paint) | 绘制命令生成耗时、GPU 合成帧耗时 | M5 |
| `engine` (JS) | Rust↔V8 桥调用开销、DOM 操作吞吐量、事件分发延迟 | M6 |
| `net` | HTTP 请求延迟、资源加载吞吐量 | M7 |
| `canvas` | 1000 矩形绘制、文本渲染、getImageData 耗时 | M9 |
| 端到端 | 完整页面加载耗时（各阶段分解） | M5+ |

**基准治理规则**：
- 每个基准测试必须有明确的度量名称、单位和目标阈值
- 基准结果以 JSON 格式持久化到 `tests/benchmarks/results/`，文件名含日期
- 每轮执行后对比最新结果与历史最佳，如 p95 退化超过 20% 必须修复后才可继续
- `scripts/run-benchmarks.sh` 一键运行并输出可读报告（含趋势对比）
- CI 中运行基准（非阻塞但记录），开发者本地运行回归门禁

### 质量门禁

**每轮执行必须满足以下条件才能继续前进**：

1. **编译门禁**：`cargo build` 成功，`cargo clippy` 无新增警告
2. **测试门禁**：`cargo test` 全通过，不允许留下红灯
3. **覆盖门禁**：新代码必须有对应测试；不允许无测试的代码变更；核心 crate 覆盖率不退化
4. **基准门禁**：`cargo bench` 可运行，关键基准 p95 不超过历史最佳基线的 120%
5. **文档门禁**：公开 API 必须有 doc comment

### 覆盖率策略

- **目标**：核心 crate（dom、css-parser、style-system、layout-engine、canvas、security）行覆盖率 ≥ 70%；其他 crate ≥ 50%
- **测量**：`cargo-llvm-cov` 或 `tarpaulin`，通过 `scripts/check-coverage.sh` 一键运行
- **报告**：每轮记录覆盖率数据到 `docs/goal/zero-web/master.md`
- **持续扩展**：覆盖率提升是主线任务的一部分，不是附加工作
- **不伪装合规**：不允许通过缩小测量范围来"达标"

### WPT 通过率追踪

从 M5 开始建立 WPT 运行基础设施：
- 按分类追踪通过率：HTML、CSS、JS/DOM、Network、Navigation
- 每轮记录通过率变化
- 通过率低不 BLOCK，但"提高通过率"必须是活跃工作项

---

## Latest Evidence

**当前状态**（2026-08-04，详见 [master.md](zero-web/master.md)）：

| 项 | 状态 |
|----|------|
| 仓库代码 | 17 个 crate + 3 个应用，全部有实质实现 |
| 编译状态 | ✅ `cargo build --workspace` 通过 |
| 测试状态 | ✅ ~13,192 测试全绿（74 ignored 网络型真实网站用例） |
| 覆盖率 | ✅ 95.46% line / 96.94% function / 94.88% region |
| WPT 通过率 | ✅ 1341 用例（23 分类，100% 通过率，按分类追踪） |
| 性能基线 | ✅ 79 criterion 基准 + 页面级首屏基准 + 峰值 RSS；**perf-gate 门禁体系就位**（Hard Gate 绝对预算 + Budget Gate 基线相对 + 趋势追踪，2026-08-08，见 docs/specs/performance-and-resource-budget.md）；首屏 < 2s 验证；增量渲染图元 < 全量 20% |
| CI | ✅ GitHub Actions 三平台（编译 + clippy + test + build + 发布打包） |
| 文档 | ✅ 目标文档 + master.md 控制面板 + 归档 + 各 crate README |

**当前活跃主线**（2026-08-04 恢复推进）：P1a DOM/JS Bridge 原生化——事件循环补全 + fetch/MutationObserver 真实化（见 master.md「下一步优先级」）。

---

## Document Control / Archive Policy

### 文档控制平面

本项目采用**两层文档控制平面**：

#### 入口文档（稳定、不频繁修改）

- **路径**：`docs/goal/zero-web.md`（本文件）
- **职责**：定义长期 Mission、Done Criteria、执行协议、文档治理规则
- **修改条件**：仅在目标本身发生实质性变化时修改（如新增平台、调整技术路线、修改完成标准）
- **禁止行为**：每轮执行不重写本文件；日常进度、证据、活跃里程碑更新写入 master.md

#### 运行时控制平面（持续演进）

- **路径**：`docs/goal/zero-web/master.md`
- **职责**：当前真实状态的唯一控制面板，包含：
  - 当前活跃里程碑及其完成状态
  - 仍然有效的目标边界和完成标准
  - 测试基线和验证证据
  - 当前能力矩阵和已验证项
  - 下一步计划
  - 未解决问题列表
- **治理规则**：
  - master.md 是持续演进的增量控制面板，不是一次性交付物
  - 不允许无限增长 — 过时内容必须重写、压缩或迁移到 archive
  - 各章节之间必须自洽（活跃里程碑、Done Criteria、覆盖率矩阵、Latest Evidence 不能互相矛盾）
  - 如果出现矛盾（如"里程碑未完成但证据声称全部满足"），执行代理必须先纠正文档和状态评估再继续

#### 归档区域（历史记录）

- **路径**：`docs/goal/zero-web/archive/`
- **职责**：存储已完成里程碑的详细过程、关键决策、验证结果、commit hash 和历史证据
- **性质**：archive 是历史记录区，不是当前状态的来源

### 首轮进入检查清单（Must-Complete-First-Round）

执行代理在首次进入时**必须**完成以下操作，这些不是可选的，也不是可以推迟的工作：

- [ ] 探索当前仓库事实（代码状态、已有文档、依赖配置）
- [ ] 定义/确认 Done Criteria（与本文件一致或提出合理调整）
- [ ] 创建 `docs/goal/zero-web/master.md`，包含完整的当前状态评估和首个活跃里程碑计划
- [ ] 创建 `docs/goal/zero-web/archive/` 目录
- [ ] 确认测试基线（当前为空，明确记录"无测试"）
- [ ] 选择第一个活跃里程碑（M1）并开始执行

**关键要求**：完成 master.md 和 archive 初始化后，执行代理**必须**在同一轮内继续启动第一个真正的里程碑（M1），直接推进核心目标能力。**不允许**把"文档框架已建立"当作里程碑完成或收工理由。

### 文档治理原则

1. master.md 各章节必须自洽 — 活跃里程碑、Done Criteria、覆盖率矩阵、Latest Evidence 不能互相矛盾
2. 如果发现矛盾，执行代理必须先纠正文档再继续
3. master.md 不允许无限增长 — 过时内容必须压缩或归档
4. archive 是只追加的 — 不修改已归档内容
5. 所有验证证据必须以结构化形式持久化（测试命令、覆盖率报告路径、验收结果）

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| 目标能力已达到生产可用质量，且被广泛自动化证据证明 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进，还有未完成的工作 | `CONTINUE: <下一步>` | 这是默认输出 |
| 遇到真正的外部阻塞（依赖不可用、平台不支持、安全漏洞无法绕过） | `BLOCK: <原因>` | 罕见使用 |
| verify 发现未满足条件但进展仍可推进 | `CONTINUE: <下一步>` | 返回执行，不是 DONE |

### DONE 允许条件

**同时满足以下所有条件时才允许输出 DONE**：

1. ✅ 所有 Done Criteria 中的检查项均已满足
2. ✅ 目标能力本身已达到生产可用质量（不只是文档完整）
3. ✅ 有真实代码、测试和验收证据直接对应目标能力
4. ✅ `cargo build` + `cargo test` + `cargo clippy` 全通过
5. ✅ 有自动化证据支撑（测试报告、覆盖率数据、性能基线）
6. ✅ master.md 内部自洽，archive 已建立，进度已归档

### 禁止输出 DONE 的情况

即使以下情况中部分条件看起来"还行"，也**不允许**输出 DONE：

- ❌ master.md 缺失、必填章节缺失、archive 为空且无有效里程碑
- ❌ 无测试证据，或测试存在红色（失败）项
- ❌ 无实际代码/测试进度（仅有文档和计划）
- ❌ 覆盖率无法证明（无测量脚本、无报告管线、无量化数据）
- ❌ master.md 各章节矛盾（如"里程碑未完成但证据声称全部满足"）
- ❌ 所有 master.md 章节都填了、archive 建了、计划列了，但没有真实代码、测试和验收证据直接对应目标能力
- ❌ 测试全绿、覆盖率达标、文档完整，但目标能力本身未达到生产可用质量

### BLOCK 策略

如果用户要求禁用 BLOCK，则遵循以下规则：

- "未完成、证据不足、暂时无法验证覆盖率、文档状态不一致" 都是**继续推进的信号**，不是 BLOCK 的理由
- 即使遇到困难，如果仍有可能推进，输出 `CONTINUE: <下一步>`
- 只有在真正无法继续（外部依赖不可用、平台根本性不支持）时才输出 `BLOCK`

---

## Execution Protocol

### 自主执行原则

执行代理必须：

1. **自主探索**当前仓库状态，识别能力缺口
2. **自主分解**里程碑为可执行的子任务
3. **自主实现**代码，不等待用户逐步指令
4. **自主添加测试**，新功能必须有对应测试
5. **自主验证**，运行测试、检查编译、追踪覆盖率
6. **自主归档**，完成的里程碑记录到 archive
7. **持续推动**，直到 Done Criteria 全部满足

### 代码提交规则

- 有阶段性进展时及时提交代码并推送到远端
- 及时拉取远端更新并 rebase
- 提交信息使用英文，文档和注释使用中文

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。遇到 flaky test、遗留失败、环境脚本问题、测试基础设施缺陷时，当作当前任务的一部分修复，直到稳定可重复
2. **技术决策**：在 master.md 中记录关键决策及其理由
3. **依赖问题**：优先自行解决；只有真正无法解决时才 BLOCK
4. **范围变更**：如果发现目标需要调整，在 master.md 中记录并说明理由，但不修改本文件（除非 Mission 本身变化）

### 当 verify 发现缺口时

- 默认输出 `CONTINUE: <下一步>` 并返回执行
- 不输出 DONE 或大段解释
- 如果仍有可能推进，就不结束
