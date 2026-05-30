# ZeroBrowser 运行时控制平面

**最后更新**: 2026-05-30
**执行状态**: 12/16 crate 已实现，637 个测试全绿

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | ✅ Cargo workspace + 16 crate（12 个已实现） |
| 编译状态 | ✅ `cargo build --workspace` 通过 |
| 测试状态 | ✅ `cargo test --workspace` 637 个测试全绿 |
| Clippy | ✅ 零警告（全 workspace） |

### 已实现 crate（12 个）

| Crate | 测试 | 说明 |
|-------|------|------|
| dom | 84 | DOM 树、html5ever 集成、查询 API |
| css-parser | 138 | Tokenizer、Parser、选择器、值解析 |
| style-system | 101 | 级联、继承、计算值、DOM 集成 |
| layout-engine | 61 | taffy 集成（Block/Flex/Grid/Position） |
| engine-core | 39 | 渲染管线、paint、dirty tracking、compositing |
| render-foundation | 53 | GPU/CPU 渲染、字体栈、图片缓存 |
| host-runtime | 3 | winit 窗口、事件循环 |
| net | 30 | HTTP client、URL、导航历史、Cookie |
| security | 22 | 同源策略、CORS、CSP |
| protocol | 24 | IPC 消息、bincode 序列化 |
| storage | 35 | localStorage、sessionStorage、IndexedDB |
| canvas | 29 | Canvas 2D API、路径、变换 |
| webview-api | 15 | WebView 嵌入 API、Builder |

### 占位 crate（4 个）

| Crate | 说明 |
|-------|------|
| script-sandbox | JS 引擎（V8/QuickJS feature gate）— 需要二进制 |
| wasm-sandbox | WASM 运行时（Wasmtime/wasmi）— 可实现 wasmi |
| browser-shell | 浏览器 UI — 需要 UI 框架选型 |
| render-foundation | 部分模块仍有 TODO（GPU renderer） |

---

## 里程碑完成情况

| 里程碑 | 状态 |
|--------|------|
| M1 项目骨架 + 渲染基础设施 | ✅ |
| M2 HTML 解析 + DOM 树 | ✅ |
| M3 CSS 解析器 + 样式系统 | ✅ |
| M4 布局引擎 | ✅ |
| M5 渲染管线集成 | ✅ |
| M6 JavaScript 集成 (V8) | ⏸ 需要 rusty_v8 |
| M7 网络栈 + 导航模型 | ✅ |
| M8 多进程架构 (IPC) | ✅ (protocol crate) |
| M9 Canvas + Storage | ✅ |

---

## 归档记录

- **M1** ✅ → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)
- **M2** ✅ → [archive/m2-dom.md](archive/m2-dom.md)
- **M3** ✅ → [archive/m3-css-parser-style-system.md](archive/m3-css-parser-style-system.md)
- **M4** ✅ → [archive/m4-layout-engine.md](archive/m4-layout-engine.md)
- **M5** ✅ → [archive/m5-rendering-pipeline.md](archive/m5-rendering-pipeline.md)
- **M7** ✅ → [archive/m7-network-security.md](archive/m7-network-security.md)
