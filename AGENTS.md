# AGENTS.md

## 编码准则

**持久性**：本准则整个会话期间始终生效，不因多轮对话而退化，不因任务切换而遗忘。仅当用户明确说"跳过准则"或"不用讲究"时暂停；恢复编码任务后自动重新生效。

**权衡**：这些准则倾向于谨慎而非速度。对于简单任务，自行判断即可。

### 1. 先思考，再编码

**不要假设，不要掩盖困惑，主动暴露权衡。**

实现之前：
- 明确陈述假设。如果不确定，提问。
- 如果存在多种理解，全部列出来——不要静默地选择一种。
- 如果存在更简单的方案，说出来。必要时据理力争。
- 如果有不明白的地方，停下来，说明困惑，提问。

### 2. 简单至上

**用最少的代码解决问题，不做推测性开发。**

写新代码前，依次评估：
1. 代码库中已有可复用的实现？→ 直接复用。
2. 标准库 / 平台原生能力可解决？→ 用标准方案。
3. 已安装的依赖能搞定？→ 用现有依赖。
4. 一行代码能完成？→ 写一行。
5. 以上都不满足 → 写最小可工作实现。

- 不实现需求之外的功能。
- 不为只使用一次的代码引入抽象。
- 不添加未要求的"灵活性"或"可配置性"。
- 不为不可能发生的场景编写错误处理。
- 如果你写了 200 行但 50 行就够了，重写它。

**以下不可简化**——即使看起来"多余"也不要删减：
- 信任边界的输入校验（用户输入、外部 API 响应）
- 防止数据丢失的错误处理
- 安全措施（认证、授权、转义、加密）
- 无障碍属性（a11y）

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

修复 bug 时，定位根因而非修补症状：
- 编辑前先追踪所有调用方，确认问题源头。
- 优先在共享路径修复（一个 guard 优于 N 个调用方各加一个 guard）。
- 如果"修复"只是压制了错误表现而非消除错误原因，继续挖。

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

### 6. 路径通用化

**使用环境变量或相对路径，避免硬编码本地绝对路径。**

- 用 `$HOME/path/to` 或 `~/path/to` 代替 `/Users/username/path/to`、`/home/username/path/to`。
- 配置文件和脚本中使用相对路径或可配置的基准路径变量。
- 不要将本机目录结构、用户名、主机名写入代码或配置中。
- 日志和错误消息中若需引用路径，使用相对于项目根目录的路径。

原因：硬编码绝对路径在其他机器/环境中会失效，且可能泄漏用户名、目录结构等敏感信息到公开仓库。（本仓曾因硬编码私有代理和绝对路径触发 SL-008/SL-010 修复，见 commit `58e74ac8`。）

### 准则让步

以下场景可放宽上述准则：
- 用户明确要求快速原型或 spike（简洁优先、精准修改可放宽）
- 紧急热修复且用户确认不需要测试（目标驱动执行可放宽）
- 用户说"跳过准则"或同义表述（全部暂停）

放宽后仍不可跳过：安全措施、输入校验、防止数据丢失的错误处理。放宽结束后自动恢复全部准则。

## 项目概述

ZeroWeb — 用 Rust 构建的跨平台浏览器。两个交付物：
1. 可复用的嵌入式 `ZeroWebView` 库（Rust lib）
2. 桌面 `ZeroBrowser` 浏览器应用（macOS、Linux、Windows；Android 为后续适配目标）

项目自建浏览器核心：DOM、CSSOM、样式系统、布局、渲染管线、导航、安全/运行时边界。外部 Rust crate 用于底层能力（html5ever、v8/rquickjs、wasmtime/wasmi、wgpu+winit、taffy）。

- 语言：Rust（edition 2024，MSRV 1.85）
- 工作区：27 个 workspace member（18 个库 + 6 个应用 + 2 个测试工具 + 1 个开发工具）
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
- **规范驱动注释（第三轮调研建议 #2，2026-08-07）**：实现 web 规范行为（HTML/CSS/DOM/JS API 语义）处，必须添加对应规范链接注释（如 `// https://html.spec.whatwg.org/#xxx`、`// https://drafts.csswg.org/css-xxx/`）；规范算法的未实现步骤标 `// FIXME:`；优化路径标 `// OPTIMIZATION:` 并说明理由。依据：Ladybird 全库 4,750 处 spec 链接注释是 90%+ WPT 的代码层基石（调研报告 §6.3 质量文化注记），规范链接同时是 AI 生成代码时的锚点

## 架构指南

工作区布局（27 个 workspace member，分 5 类）：

```
apps/
├── browser/          # zero-browser — 桌面浏览器入口
├── renderer/         # zero-renderer — 独立渲染进程入口
├── image-decoder/    # zero-image-decoder — 图像解码独立进程（D1）
├── compositor/       # zero-compositor — 合成器进程（C2）
├── webdriver/        # zero-webdriver — WebDriver 服务（W3C 协议）
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
├── product-version/  # zero-product-version — 产品版本号（从构建日期推导）
├── security/         # zero-security — CORS、CSP、同源策略、沙箱
├── storage/          # zero-storage — localStorage、IndexedDB、Cache API
├── protocol/         # zero-protocol — 多进程 IPC
├── script-sandbox/   # zero-script-sandbox — 扩展/用户脚本运行时（V8/QuickJS feature gate）
├── wasm-sandbox/     # zero-wasm-sandbox — WASM 运行时（Wasmtime/wasmi）
├── page-runtime/     # zero-page-runtime — 页面运行时统一契约（WPT / TabWorker / renderer）
├── webview/          # zero-webview — 稳定的嵌入式 API
└── browser-shell/    # zero-browser-shell — 浏览器 UI（标签页、书签、地址栏）

tests/
├── integration/      # zero-integration-tests — 跨 crate 集成测试
└── wpt-runner/       # zero-wpt-runner — WPT / reftest / 兼容性工具

tools/
└── icon-gen/         # zero-icon-gen — 图标资产生成（不随发布产物分发）
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

- **测试资产化（调研 P1/P4 成文，2026-08-07）**：渲染兼容性修复（CSS/布局/绘制/Web API 语义）必须附带对应 WPT/reftest 用例——优先执行 `make import-wpt TEST=<上游用例> REF=<参照页> NOTE="Rxxxx 修复"` 导入 `tests/wpt-runner` 常驻断言集并记入 `imported-tests.txt` 账本；无法导入上游用例时，至少补一个等价的本地 reftest/单测。依据：Ladybird CodePolicy「每修复/新特性必带测试 + 通过的新 WPT 测试导入常驻 CI」是其 WPT 通过数单向增长的机制（调研报告 §4.1、§5.2 P1/P4）。导入须连同修复同一提交，杜绝回头路。

## 安全约束

- 不要提交 .env、credentials.json 等敏感文件
- 不要执行破坏性 git 操作（push --force、reset --hard、clean -f）
- 不要在代码中硬编码 API key、密码或 token
- 修改涉及认证/授权的代码时需格外谨慎

### 提交前安全门禁

执行 `git commit` 前，必须先调用 `lei-pre-commit-guard` skill 对暂存区进行安全扫描：
- 裁决为 **PASS** 时允许提交
- 裁决为 **BLOCK** 时，输出发现报告并等待用户确认或修复后重新扫描
- 用户明确要求跳过时可豁免

### 提交前质量门禁

文档与 GitHub 元数据豁免：当且仅当全部待提交文件都位于 `docs/**`、`.github/**`，或文件扩展名为 `.md` 时，可跳过 `cargo fmt`、`cargo clippy`、构建、测试、reftest、基准和覆盖率。豁免项目代码检查时，仍必须：
- 执行 `git diff --check`
- 调用 `lei-pre-commit-guard` 并获得 **PASS**
- 按变更类型执行相关 Markdown 链接、YAML、CODEOWNERS 或其他配置语法检查

只要待提交内容包含任一不符合上述范围的文件，就不适用豁免。执行 `git commit` 前，必须先在本地跑通 `cargo fmt` 和 `cargo clippy`，禁止跳过：
- `cargo fmt --all -- --check` 必须无 diff（有 diff 先 `cargo fmt --all` 修复再提交）
- `cargo clippy --workspace --all-targets -- -D warnings` 必须无 warning/error（CI 用 `-D warnings` 强制，本地须同等严格）
- 若默认 feature（v8）因环境（如缺 rusty_v8 预编译库）无法本地编译，至少在能编译的 feature 下跑 clippy（如 `--no-default-features --features quickjs`），并在提交说明中注明覆盖范围
- 原因：CI 用 `-D warnings`，本地不跑会让 clippy warning（如 `type_complexity`）在 CI 变 error 才暴露，浪费 CI 往返。本仓曾因本地未跑 clippy 导致 `register_callback` 的 `Box<dyn Fn>` 触发 `type_complexity`，全平台 CI 失败一轮

## 无人值守运行安全

在无人值守场景（rally 循环、CI、长时间自动执行）下跑测试或构建命令时，**必须**用「内存上限 + 墙钟超时」包裹器包裹，禁止裸跑原始测试/构建命令（如 `cargo test`、`npm test`、`pytest`、`go test`、`make` 等）。

原因：单个内存型 bug（如无限循环 realloc、解析器未闭合括号死循环）或死循环会吃光内存触发系统级 OOM，连累 rally 父进程 / tmux session / 整个无人值守流程被内核整体回收，自动执行被彻底打断。包裹器只杀失控的测试进程树，不影响父进程。

包裹器由项目按语言和环境自选，例如：进程树内存监控器（test-guard 类）、`ulimit -v`（Linux）、`timeout` 命令，或把 rally 父进程跑在 `systemd-run --scope -p MemoryMax=` 限内存单元内（Linux，推荐作为兜底）。把项目选定的包裹入口（如 `make test`）和阈值记录在 `docs/rally/run-rules.md` 或 Makefile 中，并要求一律走该入口，禁止绕过。

> 本仓库已选定入口与阈值，见 [`docs/rally/run-rules.md`](docs/rally/run-rules.md)（`make test` / `make reftest`，经 `scripts/test-guard.rs` 包裹）与 [`docs/rally/oom-guard.md`](docs/rally/oom-guard.md)。

## 经验沉淀

在日常排查问题、修复 bug 以及开发新功能的过程中，如果发现了可积累的技术经验（如踩坑根因、平台差异、性能优化手段、可复用代码模式等），应主动将经验总结并保存到 `docs/learnings/` 目录下，供后续查阅参考。

### 触发场景

满足以下任一条件时，应主动沉淀经验：
- 经过深入排查才定位的 bug 根因及修复方案
- 平台/环境相关的坑点
- 可复用的代码模式或最佳实践
- 性能优化的有效手段
- 工具链/构建系统的使用技巧

### 不触发场景

以下情况不需要沉淀：
- 简单的拼写错误或语法修复
- 仅服务于当前一次性任务的临时方案
- 属于 agent 行为规则的内容（应走 `lei-self-evolution` 流程，写入 `AGENTS.md` / `TOOLS.md` / `MEMORY.md`）

### 产出位置

经验文件统一放在 `docs/learnings/` 目录下，按类型分目录：
- `docs/learnings/bugs/` — 踩坑记录（根因 + 修复 + 如何避免）
- `docs/learnings/patterns/` — 可复用代码模式、最佳实践
- `docs/learnings/platform/` — 平台相关经验
- `docs/learnings/performance/` — 性能优化经验

每条经验一个 `.md` 文件，文件名简洁描述主题（如 `wsl-clipboard-empty.md`）。文件内容应包含：日期、相关模块、问题描述、根因分析、解决方案。

## 自进化（Self-Evolution）

当发现**可复用、长期有效、可执行**的经验（agent 行为规则、协作方式、工作流、工具坑点、用户偏好等），应通过自进化流程将其固化为长期规则，而非只在本次对话内解决。完整流程见 `lei-self-evolution` skill，以下为本仓必须遵守的边界。

### 触发条件（满足任一）

- 用户明确表达"以后都这样做""记住""下次应该""你应该"等长期偏好或行为纠正
- 经过反复试错、失败重试才摸清的可复用工作流或工具用法
- 命令 / 工具 / 集成的稳定坑点（参数、执行顺序、平台差异、环境前提）
- 发现既有规则或记忆已过时，需要更新

### 质量闸门

必须**同时**满足：具体、可执行、长期有效、非重复、低风险（不含密钥 / 隐私 / 临时日志）。证据不足或只服务当前一次性任务时，**直接放弃**，按正常对话回复用户，不输出任何自进化相关内容。

### 硬性边界（不可违反）

- **审批前不得直接修改目标文件**：`AGENTS.md` / `TOOLS.md` / `MEMORY.md` / 新建 `SKILL.md` 只有在用户明确批准后才能改动。
- **审批前先静默建草案**：在 `evolution-drafts/pending/` 下创建草案，该过程不向用户展示。
- **进化请求必须是纯文本**：按 `lei-self-evolution` 规定格式输出请求后**立即停止本轮执行**，等待用户下一轮明确批准或拒绝；禁止替用户确认。
- **低置信度放弃**：拿不准时宁可不进化，按正常对话回复。

### 目标文件选择

- 行为规则 / 安全边界 / 协作方式 / 全局任务顺序 → `AGENTS.md`
- 工具固定坑点 / 命令参数 / 平台差异 → `TOOLS.md`
- 用户稳定偏好 / skill 使用偏好 / 长期事实 → `MEMORY.md`
- 可独立复用的多步骤操作手册 → 新建 `SKILL.md`

> 纯技术经验（踩坑根因、平台差异、性能手段、可复用代码模式）放 `docs/learnings/`，**不**进自进化；自进化只沉淀约束 agent 行为的规则与用户偏好。
