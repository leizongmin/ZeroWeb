# AGENTS.md

## 编码准则

### 1. 先思考，再编码

**不要假设，不要掩盖困惑，主动暴露权衡。**

实现之前：
- 明确陈述假设。如果不确定，提问。
- 如果存在多种理解，全部列出来——不要静默地选择一种。
- 如果存在更简单的方案，说出来。必要时据理力争。
- 如果有不明白的地方，停下来，说明困惑，提问。

### 2. 简单至上

**用最少的代码解决问题，不做推测性开发。**

- 不实现需求之外的功能。
- 不为只使用一次的代码引入抽象。
- 不添加未要求的"灵活性"或"可配置性"。
- 不为不可能发生的场景编写错误处理。
- 如果你写了 200 行但 50 行就够了，重写它。

问自己："资深工程师会认为这过度设计了吗？" 如果是，简化。

### 3. 精准修改

**只改必须改的，只清理自己造成的遗留。**

编辑已有代码时：
- 不要顺手"改进"相邻的代码、注释或格式。
- 不要重构没有问题的代码。
- 匹配已有代码风格，即使你会用不同方式写。
- 如果发现无关的死代码，提及它——但不要删除。

当你的修改产生孤立代码时：
- 删除因**你的修改**而变得无用的 import、变量、函数。
- 不要删除之前就存在的死代码，除非被明确要求。

验证标准：每一行修改都应能直接追溯到用户的需求。

### 4. 目标驱动执行

**定义成功标准，循环验证直到通过。**

将任务转化为可验证的目标：
- "添加验证" → "为无效输入编写测试，然后让测试通过"
- "修复 bug" → "编写能复现该 bug 的测试，然后让测试通过"
- "重构 X" → "确保重构前后测试都通过"

多步骤任务，先列出简要计划：
```
1. [步骤] → 验证：[检查方式]
2. [步骤] → 验证：[检查方式]
3. [步骤] → 验证：[检查方式]
```

清晰的成功标准可以让你独立迭代。模糊的标准（"让它能用"）需要反复确认。

## 项目概述

ZeroBrowser — 用 Rust 构建的跨平台浏览器。两个交付物：
1. 可复用的嵌入式 `webview` 库（Rust lib）
2. 完整的跨平台浏览器应用（macOS、Linux、Windows、Android）

项目自建浏览器核心：DOM、CSSOM、样式系统、布局、渲染管线、导航、安全/运行时边界。外部 Rust crate 用于底层能力（html5ever、rusty_v8/rquickjs、wasmtime/wasmi、wgpu+winit、taffy）。

- 语言：Rust（edition 2024，MSRV 1.85）
- 工作区：18 个 crate（16 个库 + 2 个应用）
- 许可证：MIT

## Setup 命令

- 安装依赖：`cargo build`
- 启动开发：`cargo run --bin zero-browser`
- 运行测试：`cargo test`
- 构建：`cargo build`
- Release 构建：`cargo build --release`
- 运行基准测试：`cargo bench`
- 检查覆盖率：`./scripts/check-coverage.sh`
- 运行 clippy：`cargo clippy -- -D warnings`

## 代码风格

- 语言：Rust
- 格式化工具：`rustfmt`（`cargo fmt`）
- 代码检查：`clippy`（`cargo clippy -- -D warnings`，CI 强制）
- CI：GitHub Actions — 在 ubuntu/macos/windows 上运行 cargo check、clippy（deny warnings）、test、build
- 文档注释：公共 API 必须有 `///` 文档注释
- 日志：使用 `tracing` crate，不使用 `println!`

## 架构指南

<!-- TODO: 请补充项目架构描述，包括目录结构、模块职责和关键设计决策 -->

工作区布局（18 个 crate，分 4 层）：

```
apps/
├── browser/          # zero-browser-app — 浏览器应用入口
└── webview-demo/     # zero-webview-demo — WebView 嵌入示例

crates/
├── dom/              # zero-dom — DOM 树（基于 html5ever）
├── css-parser/       # zero-css-parser — CSS 词法分析器 + 解析器
├── style-system/     # zero-style-system — 层叠、继承、计算值
├── layout-engine/    # zero-layout-engine — 基于 Taffy 的布局（Block/Inline/Flex/Grid）
├── engine-core/      # zero-engine-core — 页面内核（协调所有子系统）
├── canvas/           # zero-canvas — Canvas 2D API
├── render-foundation/ # zero-render-foundation — GPU/CPU 渲染、字体栈、图像缓存
├── host-runtime/     # zero-host-runtime — 窗口、事件循环、surface、IME（winit）
├── net/              # zero-net — HTTP/HTTPS 网络栈
├── security/         # zero-security — CORS、CSP、同源策略、沙箱
├── storage/          # zero-storage — localStorage、IndexedDB、Cache API
├── protocol/         # zero-protocol — 多进程 IPC
├── script-sandbox/   # zero-script-sandbox — JS 引擎（V8/QuickJS feature gate）
├── wasm-sandbox/     # zero-wasm-sandbox — WASM 运行时（Wasmtime/wasmi）
├── webview-api/      # zero-webview-api — 稳定的嵌入式 API
└── browser-shell/    # zero-browser-shell — 浏览器 UI（标签页、书签、地址栏）
```

## 测试指引

- 测试框架：Rust 内置（`#[test]`）
- 运行全部测试：`cargo test`
- 运行单个测试：`cargo test -p <crate名> --test <测试名>`
- 运行单个 crate 的测试：`cargo test -p zero-dom`
- 运行并显示输出：`cargo test -- --nocapture`
- 覆盖率报告：`./scripts/check-coverage.sh`

**重要**：所有代码变更提交前必须通过 `cargo test` 和 `cargo clippy -- -D warnings`。

## 安全约束

- 不要提交 .env、credentials.json 等敏感文件
- 不要执行破坏性 git 操作（push --force、reset --hard、clean -f）
- 不要在代码中硬编码 API key、密码或 token
- 修改涉及认证/授权的代码时需格外谨慎
