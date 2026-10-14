---
name: zeroweb-knowledge
description: ZeroWeb 项目知识。定位和修改 Rust 浏览器、嵌入式 WebView、Android/JNI、DOM/CSS/布局、V8/QuickJS、网络存储安全、字体与 GPU/CPU、媒体、IPC/CDP/WebDriver、构建测试及 Agent 工具时，按领域查约束、历史原因和源码；不表示所有 Web 规范已实现或功能已验收。
verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498
verified_at: 2026-10-14
---

## Module Structure

ZeroWeb 用 Rust 构建浏览器内核，同时交付嵌入式 `ZeroWebView` 与 `ZeroBrowser`。
桌面宿主覆盖 macOS、Linux、Windows，Android 由 Kotlin/Compose 经 JNI 接入共享 Rust。
项目仍处于实验阶段；知识记录定位和修改决策，不把历史 goal 收口当作完整产品成熟。
源码汇总基线为上方 commit；历史导出始于 e523b1c73b96360a6451948f0b748b7b4657750f，
期间共享工作区新增浏览器、脚本、布局与工程提交，受影响条目补读当前源码；
各主题保留实际核验版本。初始化时 AGENTS.md、README.md、docs/engineering 与两个
项目 Skill 的工作区材料已在 d0f281d509de3bf4f9dcd7e8dd01dac3464a663b 提交，
不归为上述旧源码基线；后续局部补充在条目中标明适用 commit。

Rust edition 2024、MSRV 1.85、MIT；workspace 有 32 个 member。
HTML 解析采用 html5ever，布局接入本地 Taffy patch，脚本有 V8/QuickJS 分支，
图元输出使用 GPU/CPU 后端。具体 feature 与消费者配置以当前 Cargo 文件为准。

### Directory Layout

- `apps/` — 浏览器、renderer、compositor、image-decoder、WebDriver、demo 与 Android 宿主。
- `crates/` — 页面内核、脚本、安全与存储、媒体、渲染和嵌入 API。
- `tests/` — 跨 crate 集成、WPT/reftest、Playwright 矩阵与测试资源。
- `scripts/`、`build-support/`、`.github/` — 构建、守卫、打包与交付。
- `docs/`、`website/`、`.agents/skills/` — 技术材料、官网与项目 Agent 工作流。

### Key Entry Points

- `ZeroWebView` in `crates/webview/src/webview.rs` — 嵌入式页面 API。
- `main` in `apps/browser/src/main.rs` — 桌面和无头浏览器入口。
- `runtime` in `apps/renderer/src/runtime.rs` — 独立页面进程运行路径。
- `members` in `Cargo.toml` — 显式工作区和内部依赖声明。

### User Task Navigation

| 用户任务 | 源码入口 | 相关知识 | 验证入口及范围 |
|---|---|---|---|
| 导航、标签、收藏或 Android 宿主 | `apps/browser/src/main.rs`、`apps/android-browser/rust/src/lib.rs` | [产品与宿主](references/products/index.md) | [功能地图](../zeroweb-verify/features/index.md)：按任务选择，未跑 GUI/Android |
| 页面布局、样式或绘制差异 | `crates/engine/src/pipeline/mod.rs`、`crates/layout-engine/src/lib.rs` | [页面内核](references/page-core/index.md) | [网页显示](../zeroweb-verify/features/page-rendering.md)：提供既有验收导航 |
| JS、事件循环或 DOM 身份 | `crates/script-sandbox/src/lib.rs`、`crates/engine/src/dom_bindings/mod.rs` | [脚本平台](references/scripting/index.md) | [网页交互](../zeroweb-verify/features/page-rendering.md)：本次 JS/IPC 未跑 |
| 网络、安全、持久化或 Service Worker | `crates/storage/src/lib.rs`、`crates/net/src/lib.rs` | [网络存储安全](references/web-data/index.md) | [存储流程](../zeroweb-verify/features/storage.md)：仅生产持久化 API 已实跑 |
| 字体、GPU/CPU、Canvas 或媒体 | `crates/render-foundation/src/lib.rs`、`crates/media/src/lib.rs` | [图形字体](references/graphics/index.md)、[媒体](references/media/index.md) | [图形显示](../zeroweb-verify/features/page-rendering.md)、[媒体播放](../zeroweb-verify/features/media.md)：本次未跑 |
| IPC、断连、角色与 feature 分支 | `crates/protocol/src/lib.rs`、`crates/page-runtime/src/lib.rs` | [共享运行契约](references/runtime/index.md) | [嵌入与自动化](../zeroweb-verify/features/embedding-automation.md)：本次未跑 |
| WPT、构建、打包或 Agent 交付 | `Makefile`、`tests/wpt-runner/src/main.rs` | [工程与工具](references/engineering/index.md) | [验证入口](../zeroweb-verify/SKILL.md)：不默认全图运行 |

## Gotchas

- 工作区依赖声明为 engine、page-runtime、script-sandbox、webview 关闭默认 feature，应用启用的引擎必须沿消费者 Cargo 声明追踪，库自己的 default 不能证明产品配置。 (`Cargo.toml`)
- `make build` 把其余 workspace 与 zero-browser 分开构建，联合构建的 feature 并集不能代替单包发布配置检查；该隔离已在历史改动中落地。 (`Makefile`, `git:627afe21d0e6b240854f6cbf33fd516e9ee24e0f`)
- Cargo 包版本与产品显示版本不同：显示版本优先使用 ZERO_BUILD_VERSION，其次 SOURCE_DATE_EPOCH 的 UTC 日期，再本地日期；显式非法值直接失败，不静默改成当前日期。 (`build-support/product_version.rs`, `crates/product-version/src/lib.rs`)

## Architecture

- ZeroWebView 是嵌入 API 边界，engine 串联页面管线，render-foundation 执行 GPU/CPU 图元输出；修改页面算法与物理输出须先确定职责所在。 (`docs/architecture.md`, `crates/webview/src/lib.rs`, `crates/engine/src/lib.rs`, `crates/render-foundation/src/lib.rs`)
- 发布浏览器的页面执行与嵌入式进程内路径不同：页面进程经 protocol 交互，page-runtime 用共享契约连接不同宿主；只测进程内行为不能覆盖 IPC 路径。 (`docs/architecture.md`, `crates/page-runtime/src/lib.rs`, `apps/renderer/src/runtime.rs`)
- 本地 taffy-local 由 patch.crates-io 接入，不属于显式 32 个 workspace member；布局依赖变动需要检查 patch 消费关系。 (`Cargo.toml`, `crates/taffy-local/Cargo.toml`)

## Conventions

- 本项目知识与验证入口分别为 zeroweb-knowledge 和 zeroweb-verify；存在入口只触发按任务阅读，不自动授权提交前维护，也不证明功能通过。 (`AGENTS.md`)

## Child Knowledge Nodes

- [知识索引](references/index.md) — 先按领域定位，再读直接主题；共享 API 和跨域分支只有一个维护节点。

## Generation Metadata

- 生成日期：2026-10-14；源码汇总基线：`1217776cc9cf179b93aca8b1557f0dab44b8e498`；历史导出基线：`e523b1c73b96360a6451948f0b748b7b4657750f`。
- 覆盖：全仓 8 个领域，124 份知识文档、86 个叶子，逻辑深度 5；共 1223 条有来源知识。
- 初始化范围依据：4058 个 Git 可见文件、4174647 行；忽略的依赖、构建产物和捕获资料不参与源码扫描。关键实现、消费者、测试、配置与领域 Git 历史按节点核对，未逐行穷尽全仓。
- 未覆盖：全部 Web 规范符合性、所有平台/feature 的运行行为和主入口宿主发现；知识核验不代表产品功能验收。
- 初始化核验：来源路径与结构 PASS；独立语义核验 PASS。结果包含来源、覆盖、共享消费者、分支差异及任务导航检查；该历史回执不覆盖后续增量。
- 功能实跑：zeroweb-verify 的存储生产 API 历史独立 7/7 通过，保留原运行版本；GUI、JS/IPC、Chrome parity、OPFS、Service Worker、媒体和 Android 等保持 NOT_RUN。
- 局部维护：2026-10-14 按 HEAD `d0f281d509de3bf4f9dcd7e8dd01dac3464a663b` 核对 R5117 calc 实现与两个回归断言，补充 R5118 历史缺口边界，共新增6条；Touch 已有知识不重复，验证地图补定向入口。本次未刷新整篇源码基线，未实跑产品。
- 增量核验：来源路径、树结构、243个修改文档本地链接及逐分类/文档计数自查 PASS，导航清单与现有124个节点一致；新增条目由作者对照源码与历史来源核对，未另做独立语义复验。
- 工作区变更：初始化资产已在 `d0f281d509de3bf4f9dcd7e8dd01dac3464a663b` 提交，本次增量尚未提交；运行报告与历史材料留在本地被忽略的任务目录。
- 导航清单：[node-catalog.json](node-catalog.json) 只列文档，不注册子 Skill。

逐分类条目总数：

| 分类 | 条目 |
|---|---:|
| Gotchas | 675 |
| Architecture | 343 |
| Conventions | 11 |
| Branching Behavior | 17 |
| Decisions | 50 |
| Patterns | 82 |
| Testing Strategy | 20 |
| Security | 2 |
| Consumer Analysis | 18 |
| Dependencies | 5 |

逐文档条目数（结构导航、API 清单和示例不计入）：

| 文档 | 条目 | 分类计数 |
|---|---:|---|
| SKILL.md | 7 | Gotchas 3；Architecture 3；Conventions 1 |
| [references/engineering/agent-tools/index.md](references/engineering/agent-tools/index.md) | 4 | Gotchas 2；Architecture 2 |
| [references/engineering/agent-tools/parity-and-regression.md](references/engineering/agent-tools/parity-and-regression.md) | 10 | Gotchas 6；Architecture 3；Branching Behavior 1 |
| [references/engineering/agent-tools/rally-and-learnings.md](references/engineering/agent-tools/rally-and-learnings.md) | 11 | Gotchas 5；Architecture 3；Decisions 2；Conventions 1 |
| [references/engineering/agent-tools/site-optimizer.md](references/engineering/agent-tools/site-optimizer.md) | 12 | Gotchas 6；Architecture 3；Decisions 2；Conventions 1 |
| [references/engineering/build-and-release/guards-and-ci.md](references/engineering/build-and-release/guards-and-ci.md) | 12 | Gotchas 7；Architecture 4；Branching Behavior 1 |
| [references/engineering/build-and-release/index.md](references/engineering/build-and-release/index.md) | 4 | Gotchas 2；Architecture 2 |
| [references/engineering/build-and-release/packaging.md](references/engineering/build-and-release/packaging.md) | 11 | Gotchas 5；Architecture 4；Patterns 2 |
| [references/engineering/build-and-release/toolchains.md](references/engineering/build-and-release/toolchains.md) | 10 | Gotchas 6；Architecture 4 |
| [references/engineering/governance.md](references/engineering/governance.md) | 12 | Gotchas 5；Architecture 3；Decisions 2；Conventions 2 |
| [references/engineering/index.md](references/engineering/index.md) | 4 | Gotchas 2；Architecture 2 |
| [references/engineering/testing/index.md](references/engineering/testing/index.md) | 3 | Gotchas 2；Architecture 1 |
| [references/engineering/testing/integration-and-product.md](references/engineering/testing/integration-and-product.md) | 11 | Gotchas 6；Architecture 3；Conventions 2 |
| [references/engineering/testing/performance-and-oracles.md](references/engineering/testing/performance-and-oracles.md) | 12 | Gotchas 7；Architecture 3；Patterns 2 |
| [references/engineering/testing/wpt-and-reftest.md](references/engineering/testing/wpt-and-reftest.md) | 14 | Gotchas 8；Architecture 4；Conventions 2 |
| [references/engineering/website.md](references/engineering/website.md) | 12 | Gotchas 6；Architecture 4；Patterns 2 |
| [references/graphics/canvas/context-and-raster.md](references/graphics/canvas/context-and-raster.md) | 14 | Gotchas 7；Architecture 4；Patterns 2；Testing Strategy 1 |
| [references/graphics/canvas/dom-bridge.md](references/graphics/canvas/dom-bridge.md) | 15 | Gotchas 6；Architecture 5；Security 2；Testing Strategy 2 |
| [references/graphics/canvas/index.md](references/graphics/canvas/index.md) | 3 | Gotchas 2；Architecture 1 |
| [references/graphics/index.md](references/graphics/index.md) | 4 | Gotchas 2；Architecture 2 |
| [references/graphics/paint-convert.md](references/graphics/paint-convert.md) | 12 | Gotchas 5；Architecture 3；Decisions 1；Consumer Analysis 3 |
| [references/graphics/render-foundation/cpu-and-backing.md](references/graphics/render-foundation/cpu-and-backing.md) | 14 | Gotchas 6；Architecture 5；Patterns 3 |
| [references/graphics/render-foundation/fonts/index.md](references/graphics/render-foundation/fonts/index.md) | 3 | Gotchas 1；Architecture 2 |
| [references/graphics/render-foundation/fonts/loading-and-matching.md](references/graphics/render-foundation/fonts/loading-and-matching.md) | 13 | Gotchas 6；Architecture 4；Patterns 2；Dependencies 1 |
| [references/graphics/render-foundation/fonts/shaping-and-raster.md](references/graphics/render-foundation/fonts/shaping-and-raster.md) | 17 | Gotchas 6；Architecture 6；Decisions 1；Patterns 2；Testing Strategy 2 |
| [references/graphics/render-foundation/gpu.md](references/graphics/render-foundation/gpu.md) | 14 | Gotchas 6；Architecture 5；Patterns 2；Testing Strategy 1 |
| [references/graphics/render-foundation/images.md](references/graphics/render-foundation/images.md) | 13 | Gotchas 7；Architecture 3；Patterns 2；Testing Strategy 1 |
| [references/graphics/render-foundation/index.md](references/graphics/render-foundation/index.md) | 9 | Gotchas 2；Architecture 2；Consumer Analysis 5 |
| [references/index.md](references/index.md) | 0 | 仅导航 |
| [references/media/audio.md](references/media/audio.md) | 13 | Gotchas 8；Architecture 3；Decisions 2 |
| [references/media/index.md](references/media/index.md) | 5 | Gotchas 3；Architecture 2 |
| [references/media/mse-and-streaming.md](references/media/mse-and-streaming.md) | 13 | Gotchas 8；Architecture 4；Decisions 1 |
| [references/media/playback.md](references/media/playback.md) | 12 | Gotchas 7；Architecture 4；Decisions 1 |
| [references/media/video-decode.md](references/media/video-decode.md) | 11 | Gotchas 7；Architecture 2；Dependencies 2 |
| [references/page-core/document/forms-and-focus.md](references/page-core/document/forms-and-focus.md) | 13 | Gotchas 11；Architecture 2 |
| [references/page-core/document/index.md](references/page-core/document/index.md) | 2 | Gotchas 1；Architecture 1 |
| [references/page-core/document/mutations-and-shadow.md](references/page-core/document/mutations-and-shadow.md) | 13 | Gotchas 11；Architecture 2 |
| [references/page-core/document/tree-and-parsing.md](references/page-core/document/tree-and-parsing.md) | 13 | Gotchas 9；Architecture 3；Decisions 1 |
| [references/page-core/index.md](references/page-core/index.md) | 6 | Gotchas 3；Architecture 3 |
| [references/page-core/layout/boxes-and-positioning.md](references/page-core/layout/boxes-and-positioning.md) | 19 | Gotchas 10；Architecture 3；Decisions 3；Patterns 3 |
| [references/page-core/layout/flex-grid-and-taffy.md](references/page-core/layout/flex-grid-and-taffy.md) | 12 | Gotchas 6；Architecture 3；Decisions 2；Dependencies 1 |
| [references/page-core/layout/fragmentation.md](references/page-core/layout/fragmentation.md) | 14 | Gotchas 8；Architecture 4；Decisions 2 |
| [references/page-core/layout/index.md](references/page-core/layout/index.md) | 3 | Gotchas 1；Architecture 2 |
| [references/page-core/layout/inline/collection-and-line-breaking.md](references/page-core/layout/inline/collection-and-line-breaking.md) | 17 | Gotchas 11；Architecture 3；Patterns 3 |
| [references/page-core/layout/inline/finalization-and-reuse.md](references/page-core/layout/inline/finalization-and-reuse.md) | 12 | Gotchas 7；Architecture 3；Patterns 2 |
| [references/page-core/layout/inline/index.md](references/page-core/layout/inline/index.md) | 2 | Architecture 2 |
| [references/page-core/layout/inline/metrics-and-writing-modes.md](references/page-core/layout/inline/metrics-and-writing-modes.md) | 15 | Gotchas 9；Architecture 4；Decisions 2 |
| [references/page-core/layout/invalidation-and-budget.md](references/page-core/layout/invalidation-and-budget.md) | 11 | Gotchas 6；Architecture 3；Patterns 2 |
| [references/page-core/layout/tables.md](references/page-core/layout/tables.md) | 13 | Gotchas 8；Architecture 3；Patterns 2 |
| [references/page-core/paint/animation-and-hit-testing.md](references/page-core/paint/animation-and-hit-testing.md) | 15 | Gotchas 8；Architecture 5；Patterns 2 |
| [references/page-core/paint/index.md](references/page-core/paint/index.md) | 6 | Gotchas 3；Architecture 3 |
| [references/page-core/paint/painter.md](references/page-core/paint/painter.md) | 18 | Gotchas 9；Architecture 5；Decisions 2；Patterns 2 |
| [references/page-core/paint/svg-and-filters.md](references/page-core/paint/svg-and-filters.md) | 14 | Gotchas 7；Architecture 4；Decisions 3 |
| [references/page-core/pipeline.md](references/page-core/pipeline.md) | 12 | Gotchas 6；Architecture 3；Decisions 1；Patterns 2 |
| [references/page-core/styles/cascade-and-matching.md](references/page-core/styles/cascade-and-matching.md) | 16 | Gotchas 12；Architecture 3；Patterns 1 |
| [references/page-core/styles/index.md](references/page-core/styles/index.md) | 2 | Gotchas 1；Architecture 1 |
| [references/page-core/styles/syntax-and-selectors.md](references/page-core/styles/syntax-and-selectors.md) | 10 | Gotchas 7；Architecture 2；Decisions 1 |
| [references/page-core/styles/values-and-properties.md](references/page-core/styles/values-and-properties.md) | 13 | Gotchas 10；Architecture 2；Decisions 1 |
| [references/products/android/chrome-and-state.md](references/products/android/chrome-and-state.md) | 12 | Gotchas 9；Architecture 1；Patterns 1；Conventions 1 |
| [references/products/android/index.md](references/products/android/index.md) | 2 | Gotchas 1；Architecture 1 |
| [references/products/android/page-input-and-surface.md](references/products/android/page-input-and-surface.md) | 13 | Gotchas 10；Architecture 1；Testing Strategy 2 |
| [references/products/android/roles-and-jni.md](references/products/android/roles-and-jni.md) | 14 | Gotchas 9；Architecture 2；Patterns 3 |
| [references/products/automation/cdp-and-bidi.md](references/products/automation/cdp-and-bidi.md) | 23 | Gotchas 16；Architecture 2；Decisions 2；Testing Strategy 3 |
| [references/products/automation/index.md](references/products/automation/index.md) | 3 | Gotchas 2；Architecture 1 |
| [references/products/automation/ui-driver-and-devtools.md](references/products/automation/ui-driver-and-devtools.md) | 17 | Gotchas 12；Architecture 2；Dependencies 1；Testing Strategy 2 |
| [references/products/automation/webdriver.md](references/products/automation/webdriver.md) | 16 | Gotchas 11；Architecture 3；Testing Strategy 2 |
| [references/products/browser-state/index.md](references/products/browser-state/index.md) | 3 | Gotchas 2；Architecture 1 |
| [references/products/browser-state/profile-and-tabs.md](references/products/browser-state/profile-and-tabs.md) | 20 | Gotchas 8；Architecture 5；Decisions 3；Patterns 4 |
| [references/products/browser-state/user-services.md](references/products/browser-state/user-services.md) | 19 | Gotchas 9；Architecture 3；Decisions 4；Patterns 3 |
| [references/products/desktop/chrome-and-window.md](references/products/desktop/chrome-and-window.md) | 24 | Gotchas 13；Architecture 4；Decisions 3；Patterns 2；Testing Strategy 2 |
| [references/products/desktop/index.md](references/products/desktop/index.md) | 3 | Gotchas 2；Architecture 1 |
| [references/products/desktop/navigation-and-input.md](references/products/desktop/navigation-and-input.md) | 24 | Gotchas 15；Architecture 6；Decisions 2；Conventions 1 |
| [references/products/embedded-webview.md](references/products/embedded-webview.md) | 16 | Gotchas 12；Architecture 1；Decisions 1；Testing Strategy 2 |
| [references/products/host-runtime.md](references/products/host-runtime.md) | 16 | Gotchas 8；Architecture 3；Decisions 3；Consumer Analysis 2 |
| [references/products/index.md](references/products/index.md) | 3 | Gotchas 2；Architecture 1 |
| [references/runtime/compositor/fallback-and-recovery.md](references/runtime/compositor/fallback-and-recovery.md) | 12 | Gotchas 4；Architecture 4；Patterns 4 |
| [references/runtime/compositor/index.md](references/runtime/compositor/index.md) | 4 | Gotchas 2；Architecture 2 |
| [references/runtime/compositor/surfaces-and-present.md](references/runtime/compositor/surfaces-and-present.md) | 11 | Gotchas 4；Architecture 4；Patterns 3 |
| [references/runtime/conditions/index.md](references/runtime/conditions/index.md) | 4 | Gotchas 2；Architecture 2 |
| [references/runtime/conditions/platform-and-product-modes.md](references/runtime/conditions/platform-and-product-modes.md) | 11 | Gotchas 4；Architecture 2；Branching Behavior 5 |
| [references/runtime/conditions/runtime-environment.md](references/runtime/conditions/runtime-environment.md) | 9 | Gotchas 2；Patterns 2；Branching Behavior 5 |
| [references/runtime/conditions/script-features.md](references/runtime/conditions/script-features.md) | 8 | Gotchas 2；Decisions 1；Branching Behavior 5 |
| [references/runtime/index.md](references/runtime/index.md) | 3 | Gotchas 2；Architecture 1 |
| [references/runtime/page-runtime/hosts-and-loading.md](references/runtime/page-runtime/hosts-and-loading.md) | 11 | Gotchas 4；Architecture 4；Patterns 3 |
| [references/runtime/page-runtime/index.md](references/runtime/page-runtime/index.md) | 7 | Gotchas 2；Architecture 1；Consumer Analysis 4 |
| [references/runtime/page-runtime/interaction-contract.md](references/runtime/page-runtime/interaction-contract.md) | 11 | Gotchas 4；Architecture 4；Patterns 3 |
| [references/runtime/protocol/frame-delivery.md](references/runtime/protocol/frame-delivery.md) | 10 | Gotchas 4；Architecture 4；Patterns 2 |
| [references/runtime/protocol/index.md](references/runtime/protocol/index.md) | 7 | Gotchas 2；Architecture 1；Consumer Analysis 4 |
| [references/runtime/protocol/messages-and-lifecycle.md](references/runtime/protocol/messages-and-lifecycle.md) | 11 | Gotchas 4；Architecture 3；Patterns 4 |
| [references/runtime/renderer/index.md](references/runtime/renderer/index.md) | 4 | Gotchas 2；Architecture 2 |
| [references/runtime/renderer/navigation-and-loop.md](references/runtime/renderer/navigation-and-loop.md) | 10 | Gotchas 4；Architecture 4；Patterns 2 |
| [references/runtime/renderer/publication-and-isolation.md](references/runtime/renderer/publication-and-isolation.md) | 10 | Gotchas 4；Architecture 3；Patterns 3 |
| [references/scripting/dom-binding/api-surface.md](references/scripting/dom-binding/api-surface.md) | 8 | Gotchas 5；Architecture 3 |
| [references/scripting/dom-binding/identity/handles-and-generations.md](references/scripting/dom-binding/identity/handles-and-generations.md) | 7 | Gotchas 6；Architecture 1 |
| [references/scripting/dom-binding/identity/index.md](references/scripting/dom-binding/identity/index.md) | 2 | Gotchas 2 |
| [references/scripting/dom-binding/identity/snapshot-and-live-dom.md](references/scripting/dom-binding/identity/snapshot-and-live-dom.md) | 9 | Gotchas 7；Architecture 2 |
| [references/scripting/dom-binding/index.md](references/scripting/dom-binding/index.md) | 2 | Architecture 2 |
| [references/scripting/event-loop/events-and-default-actions.md](references/scripting/event-loop/events-and-default-actions.md) | 12 | Gotchas 9；Architecture 3 |
| [references/scripting/event-loop/index.md](references/scripting/event-loop/index.md) | 2 | Architecture 2 |
| [references/scripting/event-loop/tasks-and-checkpoints.md](references/scripting/event-loop/tasks-and-checkpoints.md) | 10 | Gotchas 8；Architecture 2 |
| [references/scripting/index.md](references/scripting/index.md) | 2 | Architecture 2 |
| [references/scripting/modules.md](references/scripting/modules.md) | 11 | Gotchas 8；Architecture 3 |
| [references/scripting/runtimes/engines-and-sandbox.md](references/scripting/runtimes/engines-and-sandbox.md) | 8 | Gotchas 5；Architecture 3 |
| [references/scripting/runtimes/index.md](references/scripting/runtimes/index.md) | 2 | Architecture 2 |
| [references/scripting/runtimes/threading-and-disposal.md](references/scripting/runtimes/threading-and-disposal.md) | 9 | Gotchas 6；Architecture 3 |
| [references/scripting/wasm.md](references/scripting/wasm.md) | 10 | Gotchas 8；Architecture 2 |
| [references/scripting/web-platform-apis.md](references/scripting/web-platform-apis.md) | 18 | Gotchas 15；Architecture 3 |
| [references/scripting/workers.md](references/scripting/workers.md) | 9 | Gotchas 8；Architecture 1 |
| [references/web-data/index.md](references/web-data/index.md) | 3 | Gotchas 2；Architecture 1 |
| [references/web-data/network/cache-cookie-and-privacy.md](references/web-data/network/cache-cookie-and-privacy.md) | 11 | Gotchas 5；Architecture 4；Patterns 2 |
| [references/web-data/network/fetch-and-navigation.md](references/web-data/network/fetch-and-navigation.md) | 10 | Gotchas 5；Architecture 5 |
| [references/web-data/network/index.md](references/web-data/network/index.md) | 3 | Architecture 3 |
| [references/web-data/network/websocket-and-resource-policy.md](references/web-data/network/websocket-and-resource-policy.md) | 10 | Gotchas 5；Architecture 5 |
| [references/web-data/security/index.md](references/web-data/security/index.md) | 2 | Gotchas 1；Architecture 1 |
| [references/web-data/security/isolation-and-permissions.md](references/web-data/security/isolation-and-permissions.md) | 10 | Gotchas 5；Architecture 5 |
| [references/web-data/security/origin-and-site.md](references/web-data/security/origin-and-site.md) | 9 | Gotchas 5；Architecture 4 |
| [references/web-data/security/resource-gates.md](references/web-data/security/resource-gates.md) | 11 | Gotchas 5；Architecture 4；Decisions 1；Patterns 1 |
| [references/web-data/service-workers/fetch-and-clients.md](references/web-data/service-workers/fetch-and-clients.md) | 11 | Gotchas 6；Architecture 4；Patterns 1 |
| [references/web-data/service-workers/index.md](references/web-data/service-workers/index.md) | 3 | Gotchas 2；Architecture 1 |
| [references/web-data/service-workers/registration-and-lifecycle.md](references/web-data/service-workers/registration-and-lifecycle.md) | 11 | Gotchas 7；Architecture 4 |
| [references/web-data/storage/cache-api-and-opfs.md](references/web-data/storage/cache-api-and-opfs.md) | 11 | Gotchas 6；Architecture 5 |
| [references/web-data/storage/index.md](references/web-data/storage/index.md) | 2 | Architecture 2 |
| [references/web-data/storage/indexeddb.md](references/web-data/storage/indexeddb.md) | 10 | Gotchas 5；Architecture 5 |
| [references/web-data/storage/web-storage-and-persistence.md](references/web-data/storage/web-storage-and-persistence.md) | 9 | Gotchas 5；Architecture 3；Patterns 1 |
