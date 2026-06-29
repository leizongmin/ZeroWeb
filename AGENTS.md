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

### 5. 文件大小控制

**单个源码文件不超过 2000 行。**

为了便于维护和代码审查：
- 单个 `.rs` 文件不要超过 2000 行。
- 如果超过，需要考虑更合理的拆分——按职责拆分为多个模块或子模块。
- 拆分时应遵循高内聚、低耦合原则，每个模块有清晰的单一职责。

## 项目概述

ZeroWeb — 用 Rust 构建的跨平台浏览器。两个交付物：
1. 可复用的嵌入式 `ZeroWebView` 库（Rust lib）
2. 桌面 `ZeroBrowser` 浏览器应用（macOS、Linux、Windows；Android 为后续适配目标）

项目自建浏览器核心：DOM、CSSOM、样式系统、布局、渲染管线、导航、安全/运行时边界。外部 Rust crate 用于底层能力（html5ever、rusty_v8/rquickjs、wasmtime/wasmi、wgpu+winit、taffy）。

- 语言：Rust（edition 2024，MSRV 1.85）
- 工作区：21 个 workspace member（16 个库 + 3 个应用 + 2 个测试工具）
- 许可证：MIT

## Setup 命令

- Linux/macOS 首次构建前：`make setup-rusty-v8`
- Windows 首次构建前：设置 `RUSTY_V8_ARCHIVE` 指向 `rusty_v8` release `.lib`
- 启动浏览器：`cargo run --bin zero-browser`
- 启动 WebView demo：`cargo run --bin webview-demo`
- 启动开发（自动处理 V8 下载）：`make browser`
- 运行测试：`cargo test --workspace`
- 构建：`cargo build --workspace`
- Release 构建：`cargo build --release --workspace`
- 运行 WPT reftest：`make reftest`（release + test-guard；等价于 `cargo run --release --bin zero-wpt-runner -- reftest`）
- 运行基准测试：`./scripts/run-benchmarks.sh`
- 检查覆盖率：`./scripts/check-coverage.sh`
- 运行 clippy：`cargo clippy --workspace --all-targets -- -D warnings`

## 代码风格

- 语言：Rust
- 格式化工具：`rustfmt`（`cargo fmt`）
- 代码检查：`clippy`（`cargo clippy --workspace --all-targets -- -D warnings`，CI 强制）
- CI：GitHub Actions — 在 ubuntu/macos/windows 上运行 cargo check、clippy（deny warnings）、test、build
- 文档注释：公共 API 必须有 `///` 文档注释
- 日志：使用 `tracing` crate，不使用 `println!`

## 架构指南

工作区布局（21 个 workspace member，分 5 类）：

```
apps/
├── browser/          # zero-browser — 桌面浏览器入口
├── renderer/         # zero-renderer — 独立渲染进程入口
└── webview-demo/     # zero-webview-demo — WebView 嵌入示例

crates/
├── dom/              # zero-dom — DOM 树（基于 html5ever）
├── css-parser/       # zero-css-parser — CSS 词法分析器 + 解析器
├── style-system/     # zero-style-system — 层叠、继承、计算值
├── layout-engine/    # zero-layout-engine — 基于 Taffy 的布局（Block/Inline/Flex/Grid）
├── engine/           # zero-engine — 页面内核（协调所有子系统）
├── canvas/           # zero-canvas — Canvas 2D API
├── render-foundation/ # zero-render-foundation — GPU/CPU 渲染、字体栈、图像缓存
├── host-runtime/     # zero-host-runtime — 窗口、事件循环、surface、IME（winit）
├── net/              # zero-net — HTTP/HTTPS 网络栈
├── security/         # zero-security — CORS、CSP、同源策略、沙箱
├── storage/          # zero-storage — localStorage、IndexedDB、Cache API
├── protocol/         # zero-protocol — 多进程 IPC
├── script-sandbox/   # zero-script-sandbox — 扩展/用户脚本运行时（V8/QuickJS feature gate）
├── wasm-sandbox/     # zero-wasm-sandbox — WASM 运行时（Wasmtime/wasmi）
├── webview/          # zero-webview — 稳定的嵌入式 API
└── browser-shell/    # zero-browser-shell — 浏览器 UI（标签页、书签、地址栏）

tests/
├── integration/      # zero-integration-tests — 跨 crate 集成测试
└── wpt-runner/       # zero-wpt-runner — WPT / reftest / 兼容性工具
```

关键职责与设计边界：

- `zero-webview` 是稳定嵌入边界。`zero-browser` 也应像外部宿主一样优先通过它接入页面能力，不要随意绕过到更底层 crate。
- `zero-protocol` + `apps/renderer` 定义多进程边界。涉及导航、输入、存储、网络代理时，先确认消息契约，再同步修改浏览器主进程和渲染进程两端。
- `zero-engine` 负责把 DOM / CSS / 样式 / 布局 / 绘制串成页面管线；`render-foundation` 负责真正的 GPU/CPU 图元输出，二者不要混写职责。
- `script-sandbox` 和 `wasm-sandbox` 是隔离执行层。改动脚本或 WASM 集成时，优先保持 feature gate、宿主桥接和错误边界清晰。
- `tests/integration` 覆盖跨 crate 管线，`tests/wpt-runner` 覆盖规范兼容性和 reftest。行为变化优先补这两层里最贴近的测试。

从请求到像素的大致链路：

1. `net` 获取资源并维护导航上下文。
2. `dom` 解析 HTML，`css-parser` 解析样式规则。
3. `style-system` 计算层叠、继承和最终样式。
4. `layout-engine` 生成布局树与几何结果。
5. `engine` 产出绘制命令、合成层和脚本桥接。
6. `render-foundation` 输出 GPU/CPU 图元，`host-runtime` 负责窗口与 surface。
7. `webview` 把整条链路封装成稳定 API，供 `zero-browser` 或外部应用调用。

## 测试指引

- 测试框架：Rust 内置（`#[test]`）
- 运行全部测试：`cargo test --workspace`
- 运行单个测试：`cargo test -p <crate名> --test <测试名>`
- 运行单个 crate 的测试：`cargo test -p zero-dom`
- 运行并显示输出：`cargo test -- --nocapture`
- 覆盖率报告：`./scripts/check-coverage.sh`

**重要**：所有代码变更提交前必须执行 `cargo fmt`，并通过 `cargo test --workspace` 和 `cargo clippy --workspace --all-targets -- -D warnings`。

## 安全约束

- 不要提交 .env、credentials.json 等敏感文件
- 不要执行破坏性 git 操作（push --force、reset --hard、clean -f）
- 不要在代码中硬编码 API key、密码或 token
- 修改涉及认证/授权的代码时需格外谨慎
