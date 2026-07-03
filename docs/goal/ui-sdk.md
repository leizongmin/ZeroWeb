# Zero UI SDK 抽取与浏览器迁移 — 长期执行目标（goal contract）

**版本**: v1.0
**日期**: 2026-06-30
**状态**: Active
**执行模式**: 长期无人值守持续执行（`rally run docs/goal/ui-sdk.md`）
**工作分支**: `ui-sdk`
**上游 Spec/RFC**: `docs/specs/ui-sdk-spec-rfc.md`（v1.6.1）
**姊妹目标**: `docs/goal/rendering-compat.md`（页面渲染兼容性，正在活跃推进；本目标不得使其退化）

> **本文档是什么**
> 这是面向 **future executor agent** 的可执行 contract，不是说明文。文体为命令式、执行式。它定义 ZeroBrowser 自绘 UI 能力抽取为浏览器无关、可被外部复用的 Rust UI SDK，并将浏览器迁移为该 SDK 首个完整宿主应用的长期使命、边界、完成标准、测试与质量门禁、文档治理规则和终局输出协议。后续每一轮 `rally run docs/goal/ui-sdk.md` 都以本文件为**唯一稳定入口**。
>
> **与上游 Spec 的分层（权威来源声明）**：本文件是 goal contract 层（使命 / 边界 / 完成标准 / 门禁 / 治理 / 输出协议）。接口签名（spec §5 IF-001~IF-010）、各 crate 模块文件树（spec §8.4.1）、浏览器组件接入映射（§8.4.1A）、完整 UI 能力浏览器接入矩阵（§8.4.1B）、替代方案与详细 RFC 设计（§8）的**权威来源仍是 `docs/specs/ui-sdk-spec-rfc.md`（v1.6.1）**。执行器实现具体接口 / 模块 / 组件时**必须查阅 spec 对应章节**，不得凭本 goal 文档的摘要自行发挥；本 goal 文档的 DC 摘要仅用于验收与门禁判定。
>
> **铁律**：把「实现生产可用的目标能力本身」放在第一位。测试、coverage、验收、文档、归档、evidence 都是用来证明与稳固目标达成的治理手段，**不是**可以替代目标能力本身的完成物。文档框架建好 ≠ 任何核心能力完成。

---

## Mission

把当前集中在 `apps/browser` 的浏览器自绘界面（标签栏、地址栏、菜单、滚动条、页面容器等）抽取为独立的、**浏览器无关**的 Rust 自绘 UI SDK（新增 `ui/` 顶层目录），并采用 Flutter/Compose 风格的 **retained widget tree + 单向数据流 + 局部失效刷新**架构；浏览器迁移为该 SDK 的首个完整宿主应用，最终交付一套可被外部 GUI 程序复用、覆盖桌面与移动端的通用 UI SDK。

**DONE 终局（全 M0–M4，已与用户确认）**：通用 UI SDK 达到外部可复用的 production-ready；浏览器完整迁移为 SDK 宿主且**零功能/输入/渲染/多平台窗口行为退化**；YAML DSL + 完整受控表达式语言可用；完整应用级 UI 能力（animation/gestures/navigation/overlay/collections/commands/forms/assets/platform/restoration/testing/devtools/design-system）的接口与浏览器所需最小子集落地；**至少一个移动端后端可运行（M4 选定 HarmonyOS 为硬指标，同时推进 Android 为第二后端，见 `docs/goal/ui-sdk/decisions/m4-mobile-backend-harmonyos.md` 与 `m4-add-android-backend.md`）**；design-system 首个风格包交付。

**不可违反的关键约束**：

1. 通用 UI SDK 必须与浏览器业务无关。`ui/core`、`ui/render`、`ui/runtime`、`ui/widgets`、`ui/patterns`、`ui/i18n`、`ui/animation`、`ui/gestures`、`ui/navigation`、`ui/overlay`、`ui/collections`、`ui/commands`、`ui/forms`、`ui/assets`、`ui/platform`、`ui/restoration`、`ui/testing`、`ui/devtools`、`ui/design-system`、`ui/dsl` 的 `Cargo.toml` **不得依赖** `zero-browser-shell`、`zero-webview`、`zero-engine`、`zero-net`。
2. 通用 UI crate 必须位于 `ui/` 顶层目录，**不得**放入现有 `crates/`。
3. 浏览器专属 chrome 组件放在 `browser-ui/chrome`（`zero-browser-chrome`），与通用组件分开维护；可依赖 `ui/*`、`zero-browser-shell`、`ui/adapters/webview`。
4. UI SDK 与 WebView 必须共享文本/字体基础层（`foundation/text` / `zero-text-foundation`），避免字体发现、fallback、shaping、glyph cache 重复实现；网页 CSS inline layout / DOM selection **不得**迁入 UI SDK 通用布局。
5. WebView 作为高级自定义组件（`WebViewWidget`）集成，**不得**把网页 DOM 映射为 UI widgets。
6. 主题、i18n、字体/文本、输入法、焦点、无障碍、响应式/自适应布局、移动端关键概念必须在第一版架构中预留。
7. YAML DSL 与表达式语言**不得**执行任意脚本；事件只能绑定到宿主显式注册的 `ActionId`；表达式求值必须无副作用、可缓存、受 `EvalContext` 权限边界限制。
8. 用户可见字符串必须通过 `LocalizedText` / message id 引用，production DSL 不得硬编码可见文案。
9. 浏览器迁移完成后必须保持现有功能、输入、渲染和多平台窗口行为**不退化**；**不得**破坏姊妹目标 `rendering-compat` 的主线（`make test` / `make reftest` / `make product-smoke` 不得退化）。

执行方式：**分阶段交替推进** — 每轮同时 (a) 扩展 SDK 骨架与能力，(b) 把浏览器 chrome 渐进迁移到 SDK，(c) 补测试与 evidence，直到 Done Criteria 全部满足。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 / Spec 锚点 |
|------|----------|------------------|
| 独立目录与分层 | 新增 `ui/` 顶层目录；`ui/{core,render,runtime,widgets,patterns,i18n,animation,gestures,navigation,overlay,collections,commands,forms,assets,platform,restoration,testing,devtools,design-system,dsl,adapters/winit,adapters/webview,examples}`；`browser-ui/chrome` | FR-001 / FR-002 |
| 三棵树 + 单向数据流 | Widget tree（声明）/ Element tree（retained 实例状态）/ Render·Scene tree（布局/绘制/命中/合成/a11y）；事件→Action→AppState→mark needs_layout/paint/semantics→render | FR-003 / FR-004 |
| WebViewWidget | `ui/adapters/webview` 把 `zero-webview` 包装为高级自定义组件；SDK 分配 viewport/clip/scale/theme/输入，WebView 自处理 DOM/CSS/layout/paint，输出 primitives/texture/scene node 由 SDK 合成 | FR-005 / IF-004 |
| 滚动语义边界 | 页面内容尺寸/scroll offset 由 WebView 管理；通用 ScrollBar 几何与视觉由 `ui/widgets` 提供；ScrollBar 拖动转为 `ScrollCommand` | FR-006 |
| 主题系统 | System/Light/Dark/Custom/HighContrast；semantic token；系统主题变化仅触发 paint 失效（字体/间距不变时不触发布局） | FR-007 / IF-003 |
| YAML DSL + 完整表达式语言 | YAML→`WidgetSpec`；表达式四阶段管线 parse/validate/typecheck/eval；字面量/路径/运算符/集合/字符串/样式计算/结构控制/action payload；禁止递归/loop/系统 API/状态写入/未注册函数 | FR-008 / IF-005 / §8.4.7 |
| 首批组件 | `ui/widgets`：Button/IconButton/TextInput/Toolbar/Menu/ContextMenu/Popup/Popover/ListView/Badge/Tooltip/ScrollBar/ProgressIndicator；`ui/patterns`：SearchField/SuggestionList/CommandPalette/DataList/StatusBubble/TabBar/DialogScaffold；`browser-ui/chrome`：BrowserTabStrip/AddressBar/NavigationButtons/SecurityBadge/SiteInfoPanel/BookmarksBar/FindBar/PermissionPrompt/DownloadPanel/DownloadItemView/BrowserMenu/PageLoadIndicator/PageViewportFrame | FR-009 / §8.4.1A |
| 桌面/移动宿主抽象 | 桌面优先；移动端在核心 API 预留 safe area/touch/soft keyboard/density/text scale/back gesture | FR-010 / IF-006 |
| 无障碍/焦点/IME | SemanticsNode、焦点遍历、键盘导航、IME rect | FR-011 |
| 局部失效刷新 | 区分 layout/paint/semantics/composite 失效；hover/pressed/theme 变化不触发布局 | FR-012 |
| 国际化资源 | `ui/i18n` 通用机制（locale/catalog/fallback/参数/plural/text direction/diagnostic）；浏览器文案属 `browser-ui/chrome/i18n` 或 `apps/browser/i18n`；RTL 影响布局方向与可镜像图标 | FR-013 / IF-007 / §8.4.8 |
| 共享文本/字体基础层 | `foundation/text` / `zero-text-foundation`：font 发现/fallback/shaping/bidi/line break/grapheme/glyph cache/glyph atlas/measure；优先复用 workspace 已有 `fontdue/swash/rustybuzz/unicode-bidi` | FR-014 / IF-008 / §8.4.9 |
| 响应式/自适应 + 移动 chrome | WindowMetrics/ViewportClass(Compact/Medium/Expanded)/PlatformClass/InputClass/AdaptiveShell；desktop/tablet/phone shell 共享 `BrowserChromeModel`+`BrowserAction`；禁止 desktop chrome 原样缩小为 mobile | FR-015 / IF-009 / §8.4.4A |
| 完整应用级 UI 能力 | 13 个能力域的接口边界 + 浏览器所需最小子集：animation/gestures/navigation/overlay/collections/commands/forms/assets/platform/restoration/testing/devtools/design-system | FR-016 / IF-010 / §8.4.10 |
| 移动端运行时（M4 终局） | 至少一个移动后端可运行（M4 选定 HarmonyOS 为硬指标 + Android 为第二后端）；PhoneBrowserShell/TabletBrowserShell 可用；移动 gesture/navigation/restoration/platform 适配 skeleton；design-system 首个风格包 | FR-010 / FR-015 / FR-016 / M4 |
| 浏览器迁移 | `apps/browser` 从「绘制所有东西」变为「组装 SDK 组件 + 浏览器状态编排 + Action dispatch」 | FR-009 / §8.2 / §8.4.1A |
| 浏览器接入完整 UI 能力 | 菜单/快捷键/command palette→`ui/commands`；权限/下载/site info→`ui/overlay`+`ui/navigation`；下载/历史/书签/tab overview→`ui/collections`；拖拽/剪贴板/file picker→`ui/platform`；session restore→`ui/restoration` | §8.4.1B |

### 不在范围内（明确排除）

- **首阶段不实现**任意脚本运行时、插件 VM、通用系统自动化能力（DSL 表达式语言是受控计算层，不是脚本运行时）。
- **首阶段不提供**完整可视化设计器/编辑器/热重载工具（`ui/devtools` 只做 inspector/snapshot/preview skeleton）。
- **不重写浏览器内核**页面管线（DOM/CSS/layout/rendering）。
- **不将网页 DOM/CSS 渲染**纳入 UI SDK 通用布局系统；CSS inline formatting / line box / white-space / text-decoration / DOM selection / caret / 网页内部 hit-test / a11y tree 仍由 WebView/engine 负责。
- **不把浏览器专属语义**（URL 安全状态、书签、下载、标签历史）加入通用控件库 `ui/widgets` / `ui/patterns`。
- **不在一个提交中**同时迁移所有浏览器 UI；**不**为「通用」重写浏览器内核页面管线。
- **不修改**与本任务无关的 DOM/CSS/layout 兼容性代码、网络/安全/存储/脚本 sandbox 行为、未经阶段计划覆盖的 WPT/reftest 行为（见 §Support Envelope 代码变更边界）。
- **不破坏**姊妹目标 `rendering-compat`：`make test`、`make reftest`、`make product-smoke` 不得因 UI SDK 迁移而退化。

### 代码变更边界

- **允许修改**：`docs/specs/**`、`docs/goal/ui-sdk*`、`Cargo.toml`、`foundation/**`、`ui/**`、`browser-ui/**`、`apps/browser/**`、`apps/webview-demo/**`、`crates/host-runtime/**`（与 UI runtime 迁移相关部分）、`crates/render-foundation/**`（为 scene 输出与 text foundation 拆分所需最小接口）、`Makefile`/`scripts/**`（如需新增 SDK 测试/coverage 入口）。
- **禁止修改**：与 UI SDK 无关的 DOM/CSS/layout 兼容代码；网络/安全/存储/脚本 sandbox；`tests/wpt-runner/**` 中 WPT/reftest 行为（除非本目标里程碑明确覆盖并先同步 `rendering-compat` 主线）；`docs/goal/rendering-compat*` 姊妹目标的控制面（只读引用，不得改写）。
- **文件大小**：单个 `.rs` 文件 ≤ 2000 行（仓库 AGENTS.md 硬约束）；超出须按职责拆分。

### 依赖约束

- **原则**：最小化新依赖；优先复用 workspace 已有 crate（`fontdue 0.9` / `swash 0.2` / `rustybuzz 0.20` / `unicode-bidi 0.3` / `taffy 0.7`（本地 patch）/ `winit 0.30` / `serde` / `serde_json` / `thiserror` / `tracing` / `slotmap` / `hashbrown` / `compact_str` / `parking_lot`）。
- **许可证**：如必须引入新 crate，**仅接受** MIT / Apache-2.0 / BSD；执行器自主决策（见 Final Output Contract 的依赖自治条款），但必须在 `master.md` 决策日志 + archive 记录必要性与候选评估。
- **禁止**：引入 GPL / LGPL / AGPL 或其它 copyleft 许可证依赖；引入仅供 DSL 任意脚本能力的依赖。
- **技术约束**（spec §6.4）：Rust edition 2024，MSRV 1.85（与 workspace `Cargo.toml` 一致）；桌面端优先支持 **Windows / macOS / Linux**，移动端首期在类型层预留 **Android / iOS / HarmonyOS** 接入点（M4 至少落地一个可运行后端）；`ui-runtime` / `ui-platform` / `ui-gestures` / `ui-navigation` / `ui-overlay` 的公共 API **不得**向 widgets / patterns / browser-ui 暴露 winit-specific 类型。

### 渐进覆盖策略

上游 Spec 规模庞大（16 FR / 5 里程碑 / 约 24 个新 crate）。按 M0→M4 顺序推进，**每个里程碑都必须可构建、可运行、可测试**，且每个里程碑收口时浏览器非回归门禁必须保持绿色：

- **M0**：文档与架构边界确认（本 goal contract + 目录/依赖边界决策）。不改运行时代码。
- **M1**：UI SDK 核心骨架（广度优先）—— 全部 crate skeleton + 接口边界 + 无窗口单测；浏览器不动。
- **M2**：浏览器首批组件迁移 + `ui/adapters/webview` + text foundation 接入 `ui/render` 与 `zero-webview`；`apps/browser` 局部迁移，逐组件灰度，非回归绿色。
- **M3**：DSL + 完整表达式语言 + 示例应用（counter/form/browser-shell-demo）。
- **M4**：跨平台 runtime adapter + 移动端可运行后端 + design-system 首个风格包 + Phone/Tablet shell。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。每条 DC 必须有自动化证据（测试命令 / coverage 报告路径 / scene snapshot / 可运行产物）落盘到 `docs/goal/ui-sdk/evidence/`，禁止仅凭「看起来对了」或「文档已写」声称通过。

### DC-1: 目录与依赖隔离

- [ ] 新增 `ui/` 顶层目录，包含 spec §FR-002 列出的全部子 crate（core/render/runtime/widgets/patterns/i18n/animation/gestures/navigation/overlay/collections/commands/forms/assets/platform/restoration/testing/devtools/design-system/dsl/adapters/winit/adapters/webview/examples）。
- [ ] 新增 `foundation/text`（`zero-text-foundation`）与 `browser-ui/chrome`（`zero-browser-chrome`）。
- [ ] 上述通用 UI crate 的 `Cargo.toml` **不依赖** `zero-browser-shell`/`zero-webview`/`zero-engine`/`zero-net`；通过 `cargo metadata` 或依赖审查脚本可机械验证，并产出 evidence。
- [ ] `ui/adapters/webview` 可依赖 `zero-webview`；`browser-ui/chrome` 可依赖 `ui/*` + `zero-browser-shell` + `ui/adapters/webview`——这是允许的唯一浏览器耦合点。

### DC-2: 三棵树 + 单向数据流 + retained 模型

- [ ] Widget tree（声明，可由 Rust API 或 YAML DSL 生成）/ Element tree（retained 实例状态、焦点、生命周期、绑定缓存）/ Render·Scene tree（布局/绘制/命中/裁剪/合成/a11y，按失效标记更新）三者边界清晰并落地。
- [ ] 事件 → Action/Message → AppState reducer → mark needs_layout/needs_paint/needs_semantics → render 流程可用且有单测。
- [ ] `WidgetSpec` 改变不丢失稳定 `WidgetId` 的组件状态（光标/选区/焦点）；paint-only 变化（如主题色切换且字体/间距不变）只标记 `needs_paint`，不标记 `needs_layout`——有 invalidation 单测。

### DC-3: WebViewWidget 自定义组件

- [ ] `ui/adapters/webview::WebViewWidget` 接收 viewport/clip/scale/theme/输入，把 `zero-webview` 输出（RenderPrimitives/Texture/ExternalSurface/SceneNode）合成进 UI scene。
- [ ] UI SDK 只计算 WebViewWidget 外部矩形，**不**把网页 DOM 节点映射为 UI widgets；有架构边界测试与 WebViewWidget paint/scroll 集成测试。

### DC-4: 滚动语义与滚动条边界

- [ ] 页面内容尺寸/scroll offset 由 WebView 管理；通用 ScrollBar 几何与视觉由 `ui/widgets` 提供；ScrollBar 拖动发出 `ScrollCommand`，不直接改业务状态；有 hit-test + command 测试。

### DC-5: 主题系统

- [ ] 支持 System/Light/Dark/Custom/HighContrast；组件只消费 semantic token，不硬编码浏览器色值。
- [ ] 系统主题变化时 `ThemeResolver` 生成新 Theme 并发 `ThemeChanged`；字体/间距不变时仅触发 `needs_paint`；自定义 palette 覆盖 semantic token；有 theme resolve + contrast lint + invalidation 测试。

### DC-6: YAML DSL + 完整表达式语言

- [ ] `ui/dsl` 把 YAML 解析为 `WidgetSpec`（含 component/props/bindings/actions/children/control directives）。
- [ ] 表达式四阶段管线 parse/validate/typecheck/eval 全部落地，覆盖字面量/路径/算术·比较·布尔·空值合并·条件/集合 map·filter·any·all·count/字符串 concat·contains·starts_with·ends_with·format/样式 token·clamp·min·max/`if`·`for_each`·`visible_when`·`enabled_when`/action payload。
- [ ] sandbox negative tests 全绿：拒绝递归/无限循环/文件·网络·进程·时钟·随机数访问/未注册函数/状态写入，返回 `DslError::ForbiddenCapability`/`UnknownFunction`。
- [ ] 求值有最大 AST depth / 最大节点数 / 最大 collection iteration 数上限，防恶意 DSL 消耗 CPU。

### DC-7: 首批组件（通用 + 组合模式 + 浏览器领域）

- [ ] `ui/widgets` 提供 spec §FR-009 列出的全部首批基础控件；`ui/patterns` 提供全部首批组合模式；`browser-ui/chrome` 提供全部浏览器领域组件。
- [ ] 浏览器领域组件由通用 widgets/patterns 组合绘制，输出进入统一 UI SDK scene，**不**绕过 `ui/render`；有 component test + scene snapshot。

### DC-8: 无障碍 / 焦点 / IME

- [ ] SemanticsNode 模型可用；可聚焦组件参与焦点遍历（Tab 按声明顺序或显式 traversal policy）；TextInput 提供当前光标位置对应屏幕 IME rect；有 focus traversal + IME rect 测试。

### DC-9: 局部失效刷新

- [ ] hover 变化只触发 `needs_paint`；文本变长导致测量变化才触发 `needs_layout`+`needs_paint`；有 layout/paint invalidation 测试。

### DC-10: 国际化资源与 message id

- [ ] `ui/i18n` 提供 locale/catalog/fallback/参数替换/plural/text direction/diagnostic；浏览器文案在 `browser-ui/chrome/i18n` 或 `apps/browser/i18n`，不在 `ui/widgets`/`ui/patterns`。
- [ ] DSL 通过 `i18n:` message id 引用文案；缺失 key 走 fallback 并产生 diagnostic；带参数与 plural 规则可解析；RTL locale 影响文本方向/默认对齐/可镜像图标；locale 切换触发 layout+paint+semantics 失效；有 catalog resolve/fallback/plural/RTL snapshot 测试。

### DC-11: 共享文本/字体基础层

- [ ] `ui/render` 与 `zero-webview` 通过 `zero-text-foundation` 得到一致的 font fallback chain，并共享 glyph cache（同字体同字号同 glyph 可复用 atlas entry）。
- [ ] WebView 的 CSS inline layout / line box / selection 仍由 WebView/engine 处理，只在 shape/measure/raster 阶段调用基础层；有 shared fallback + glyph cache reuse + architecture boundary 测试。

### DC-12: 响应式/自适应 + 移动 chrome

- [ ] WindowMetrics/ViewportClass/PlatformClass/InputClass 可用；`Adaptive` 按 metrics 选择 desktop/tablet/phone shell；三者共享 `BrowserChromeModel`+`BrowserAction`。
- [ ] mobile safe area 与软键盘影响布局（address/search overlay 不被 keyboard rect 遮挡，bottom toolbar 避开 safe area）；DSL responsive branch 跨断点切换时保留稳定 `WidgetId` 的输入状态；有 adaptive shell selection + safe-area/keyboard + responsive branch 测试。

### DC-13: 完整应用级 UI 能力（13 域接口 + 浏览器最小子集）

- [ ] animation/gestures/navigation/overlay/collections/commands/forms/assets/platform/restoration/testing/devtools/design-system 的接口与模块边界落地；浏览器所需最小子集（§8.4.1B 矩阵）接入：菜单/快捷键/command palette 同 command、权限提示 desktop popover + mobile sheet、下载/历史/书签/tab overview 虚拟化、拖拽/剪贴板/file picker 走 platform service、route/scroll/input selection 可恢复；有对应模块 skeleton 测试与浏览器接入集成测试。

### DC-14: 浏览器迁移完成 + 零退化（硬门禁）

- [ ] `apps/browser` 启动后标签、地址栏、导航按钮、书签栏、菜单、滚动、WebView 渲染均保持可用（`cargo run --bin zero-browser` + product smoke）。
- [ ] `make test` 全绿；`make product-smoke`（welcome.html vs chromium Oracle，diff ≤ 当前阈值，可 `MAX_DIFF` 调整）不退化；`make reftest` 不退化（不得拖累 `rendering-compat` 主线）。
- [ ] `apps/browser` 不再直接拥有 toolbar/tab/address/menu 几何与绘制，只负责状态编排、进程/导航接入、Action dispatch、应用生命周期。
- [ ] counter/form/browser-shell-demo 示例可构建运行；counter 示例不依赖 `zero-browser-shell`/`zero-webview`。

### DC-15: 移动端运行时（M4 终局，已纳入 DONE）

- [ ] 至少一个移动后端可运行（**M4 硬指标：HarmonyOS**；**第二后端：Android**，尽量在 M4 内达标但不阻塞 DONE）。决策记录：`docs/goal/ui-sdk/decisions/m4-mobile-backend-harmonyos.md` 与 `m4-add-android-backend.md`。
- [ ] PhoneBrowserShell / TabletBrowserShell 可用并与 DesktopBrowserShell 共享 `BrowserChromeModel`+`BrowserAction`。
- [ ] touch/pan/pinch/fling gesture、soft keyboard、safe area、text scale、平台 back gesture 的最小适配 skeleton 落地；`ui/testing`/`ui/devtools` 支持 responsive preview / layout bounds / semantics snapshot。
- [ ] design-system 首个风格包（至少 Zero default）交付。

### DC-16: 测试与质量不可退让

- [ ] 所有现有测试持续全绿（`make test` 零失败），不引入新的 `#[ignore]`（除既有真实网站兼容性测试等已记录例外）。
- [ ] 所有新增 SDK 能力 / 行为变化 / 兼容性扩展 / 回归修复必须同步补单元测试 + 必要的集成/e2e/scene snapshot 测试；**不允许只改代码不加测试**。
- [ ] `cargo build --workspace` 零错误；`cargo clippy --workspace --all-targets -- -D warnings` 零警告；`cargo fmt` 无变更。
- [ ] 不能带着已知失败测试继续推进；不能把红灯留给下一轮；flaky / 历史遗留失败 / 环境脚本问题 / 测试基础设施缺陷必须修到稳定可重复。

### DC-17: Coverage 量化（长期主线）

- [ ] 新增 `ui/*` 与 `foundation/text` crate 的 line coverage **≥ 85%**（首版 skeleton 阶段允许先建立基线，但必须持续向 85% 推进并在 master.md 记录曲线）。
- [ ] 全仓 coverage 以当前基线（line 95.46% / function 96.94% / region 94.88%，见 `rendering-compat` 基线）为**非回归 floor**，不得因 UI SDK 迁移显著下降。
- [ ] 统一口径统一走 `scripts/check-coverage.sh`（或等价统一脚本）；coverage 报告路径写入 master.md Latest Evidence。
- [ ] **禁止**通过缩小统计范围、把模块排除出口径、只测 happy path 来伪造达标。
- [ ] 若当前缺少 coverage 测量手段 / 统一脚本 / 报告链路 / 某模块暂无法纳入口径——这是**要继续推进的 active milestone**（补齐 coverage 测量能力本身），**不是** BLOCK 理由。

### DC-18: 证据持久化与文档自洽

- [ ] 每轮 evidence 结构化落盘到 `docs/goal/ui-sdk/evidence/`：测试命令、coverage 报告路径、关键验收结果、当前已验证通过的能力矩阵、未解决缺口。
- [ ] `master.md` 内部各 section 保持自洽：active milestone / done criteria 进度 / coverage matrix / Latest Evidence 不得互相矛盾；若出现「仍有未完成 milestone 但 evidence 声称 all done criteria met」类冲突，必须**先修正文档与状态判断**，再继续推进。

### DONE 前置铁律（防止假 DONE）

- 即使测试全绿、coverage 达标、文档齐全，**也不自动等于**任务完成；只有目标能力本身（通用 UI SDK + 浏览器迁移 + DSL + 移动运行时）达到 production-ready 并被自动化证据**广泛证明**时，才允许 `DONE`。
- 即使 `master.md` section 全部写完、archive 已建立、milestones 已列出，**也不自动等于**完成任何核心目标；若缺少与目标能力直接对应的真实代码、测试和验收证据，必须继续推进。
- 若缺少 `master.md`、缺少必需 section、archive 为空且无任何有效里程碑、没有测试证据、没有实际代码/测试推进——表示任务尚未开始或远未完成，**绝不能**输出 `DONE`。

---

## Current Proven Baseline

截至 2026-06-30（本 goal 创建时点；执行器首轮必须复核并更新到 master.md）：

### 已有可复用基础（SDK 将**扩展**而非重写）

| 领域 | 状态 | 详情 |
|------|------|------|
| 浏览器自绘 UI | ✅ 存在但耦合 | `apps/browser/src/{app.rs,app_render.rs,app_input.rs,page_scroll.rs,colors.rs,app_platform.rs,layout.rs}` 集中手写布局/绘制/输入；待抽取 |
| 渲染基础 | ✅ 可复用 | `crates/render-foundation`（CPU/GPU 图元 + 字体栈 + 图像缓存）；M7 后支持全 13 种 RenderPrimitives |
| 窗口/事件/IME | ✅ 可复用 | `crates/host-runtime`（winit 0.30：窗口、事件循环、surface、IME） |
| WebView 稳定边界 | ✅ 可复用 | `crates/webview`（嵌入式 API） |
| 浏览器业务状态 | ✅ 可复用 | `crates/browser-shell`（标签、书签、历史、设置、下载，UI-agnostic） |
| 字体/排版依赖 | ✅ workspace 已声明 | `fontdue 0.9` / `swash 0.2` / `rustybuzz 0.20` / `unicode-bidi 0.3` —— `zero-text-foundation` 优先复用 |
| 布局依赖 | ✅ workspace 已声明 | `taffy 0.7`（本地 patch `crates/taffy-local`）；响应式布局可评估复用 |
| 序列化/错误/日志 | ✅ workspace 已声明 | `serde` / `serde_json` / `thiserror` / `anyhow` / `tracing` |
| 测试基础设施 | ✅ 成熟 | `make test`（release + `scripts/test-guard.rs` 包裹）、`make reftest`、`make product-smoke`（welcome.html vs chromium Oracle，diff>20% 退出 2）、`scripts/check-coverage.sh`、`docs/rally/oom-guard.md` |
| Coverage 基线 | ✅ 已量化 | line 95.46% / function 96.94% / region 94.88%（`rendering-compat` 基线，作为本目标非回归 floor） |

### 尚不存在（本目标要新建）

- `ui/` 顶层目录与全部 `zero-ui-*` crate —— **均不存在**。
- `foundation/text` / `zero-text-foundation` —— 不存在。
- `browser-ui/chrome` / `zero-browser-chrome` —— 不存在。
- Widget/Element/Render 三棵树模型、retained + 单向数据流运行时、Theme/i18n/DSL/表达式语言 —— 均不存在。
- `docs/goal/ui-sdk/master.md`、`docs/goal/ui-sdk/archive/`、`docs/goal/ui-sdk/evidence/` —— **均不存在**（首轮必须创建）。

### 已知关键缺口

| 缺口 | 影响范围 | 严重性 |
|------|----------|--------|
| UI 绘制/几何/输入耦合在 `apps/browser` | 复用性、迁移可行性 | P0 |
| 无独立 Widget/组件模型与三棵树 | 全部 UI 能力 | P0 |
| 无主题/i18n/焦点/IME/a11y SDK 级抽象 | 长期可维护、移动端 | P0 |
| 无共享 text foundation | UI 与 WebView 字体能力重复 | P1 |
| 无 DSL / 表达式语言 | 外部声明式复用 | P1 |
| 无移动端抽象与运行时 | 移动端 DONE | P1（M4） |

### 测试基线（首轮复核）

- 现有测试全绿（`make test`）；reftest / product-smoke 由 `rendering-compat` 主线维护，本目标不得使其退化。
- UI SDK 新增 crate 暂无 coverage 数据（M1 建立基线后写入 master.md）。

---

## Single Active Milestone

**当前活跃里程碑：M1 — UI SDK 核心骨架（广度优先，spec §M1 原文）**。

### M1 目标

建立通用 UI SDK 的全部 crate skeleton、核心类型、接口边界与无窗口单测，使架构在编译期与测试期成立，**且不触碰浏览器运行时行为**（浏览器非回归门禁保持绿色）。采用广度优先：先把 spec §FR-002 列出的 crate 都立起来（含接口 + 最小测试），再在后续里程碑里逐个填实与迁移。

### M1 范围

1. **`ui/core`**：基础类型与协议（geometry / event / widget / element / action / binding / theme / focus / semantics / invalidation / layout(WindowMetrics·ViewportClass·adaptive branch)）。
2. **`foundation/text`（`zero-text-foundation`）skeleton**：`FontProvider` / `TextShaper` / `TextMeasurer` / `GlyphCache` 接口 + 最小测试；优先复用 workspace 已有 fontdue/swash/rustybuzz/unicode-bidi；最终落点（`foundation/text` vs 先从 `crates/render-foundation/text` 过渡）在 M1 内确认并记录。
3. **`ui/render`**：scene/render node、paint context、合成输入；接入 text foundation；与 `render-foundation` 依赖方向在 M1 确认（TBD-2）。
4. **`ui/runtime`**：app lifecycle、runtime loop 抽象、ThemeChanged、IME、input routing；winit 类型不得泄漏到 widgets。
5. **`ui/widgets`**：迁移最小 ScrollBar（从 `apps/browser/src/page_scroll.rs` 抽取通用部分）/ Button / TextInput skeleton。
6. **`ui/patterns`**：SearchField / SuggestionList / TabBar 等通用组合模式 skeleton。
7. **`ui/i18n`**：locale / catalog loader / message id / fallback / 参数 / RTL direction skeleton + locale catalog 示例。
8. **13 个应用级能力域 skeleton**：`ui/{animation,gestures,navigation,overlay,collections,commands,forms,assets,platform,restoration,testing,devtools,design-system}` 的接口边界 + 最小单测（animation 需 fake clock；testing 需 scene/semantics snapshot 工具）。
9. **`ui/dsl` skeleton**：YAML→`WidgetSpec` 最小骨架（完整表达式语言在 M3）。
10. **`ui/adapters/{winit,webview}` skeleton** + **`browser-ui/chrome` skeleton**（仅占位与依赖边界，组件迁移在 M2）。
11. workspace `Cargo.toml` 加入新 members；CI 通过 `cargo build --workspace` + `cargo clippy -- -D warnings`。

### M1 完成标准

- [ ] 全部上述 crate 存在、可编译、有无窗口单测；通用 crate 不依赖浏览器业务 crate（DC-1 机械验证 evidence 落盘）。
- [ ] `make test` 全绿；`make product-smoke` 不退化。
- [ ] 三棵树 + 单向数据流 + 局部失效有最小可用单测（DC-2 / DC-9 的 skeleton 级证据）。
- [ ] text foundation 接口可用并接入 `ui/render`（DC-11 skeleton 级）。

### M1 不做

- 不迁移浏览器 chrome 组件（M2）；不实现完整 DSL 表达式语言（M3）；不实现移动端运行时（M4）；不重写 render-foundation 后端。

### ⚠️ 治理 bootstrap 与首里程碑衔接（强制）

执行器进入 M1 时，**必须先完成文档治理 bootstrap**（见 §Document Control 第一轮 checklist：创建 `master.md` + `archive/` + `evidence/`、复核仓库事实、确认 done criteria、确认测试基线）。**完成 bootstrap 后，在同一轮内必须继续启动 M1 的第一个真实 crate**（`ui/core` 最小类型 + 单测，加入 workspace 并 `cargo test -p zero-ui-core` 通过），**不得**把「文档框架/master.md 已建好」当成 milestone 完成、收口依据或停机点。

---

## Ordered Next Milestones

> 里程碑顺序为建议推进路径；执行器在 done criteria 未满足前可自主重排子任务，但每跨入一个里程碑都必须先在 master.md 记录决策与依赖。

### M0 — 文档与架构边界确认（本 goal contract 即产出）

- 输出本 goal doc + 目录/依赖边界决策；不改运行时代码。**状态**：本文件即 M0 交付；执行器首轮复核并落地 `master.md`。

### M1 — UI SDK 核心骨架（广度优先，**当前活跃**）

- 见 §Single Active Milestone。依赖：无（可与浏览器并存）。

### M2 — 浏览器首批组件迁移 + WebView adapter + text foundation 接入

- 抽取滚动条/按钮/文本输入/菜单/popup/toolbar 到 `ui/widgets`；SearchField/SuggestionList/TabBar 等到 `ui/patterns`；新增 `browser-ui/chrome` 首批领域组件（按 spec §8.4.1A 映射）。
- `ui/render` 与 `ui/widgets::TextInput` 接入共享 text foundation；`ui/adapters/webview` 明确 WebView 只在 shape/measure/raster 阶段调用 text foundation。
- 定义 `BrowserChromeModel` + `BrowserAction` + desktop/tablet/phone shell 共享合约；browser menu/shortcut/context menu 接 `ui/commands`；PermissionPrompt/DownloadPanel/SiteInfoPanel 接 `ui/overlay`；Downloads/Bookmarks/History/TabOverview 接 `ui/collections`。
- **逐组件灰度迁移**（shim / feature-flag），任意提交点浏览器可运行；`apps/browser` 改为消费 `browser-ui/chrome`。依赖：M1。

### M3 — DSL + 完整表达式语言 + 示例应用

- `ui/dsl`：YAML→`WidgetSpec`；表达式 parse/validate/typecheck/eval + sandbox negative tests；条件渲染/列表渲染/属性绑定/样式绑定/action payload；DSL `i18n` message id 引用；responsive branch；command/route/overlay/asset/animation/gesture 引用。
- 示例：counter / form / browser-shell-demo。依赖：M1（M2 可并行部分）。

### M4 — 跨平台 runtime adapter + 移动端可运行后端 + design-system（终局）

- 完善 runtime adapter；touch/软键盘/safe area/text scale/platform back gesture；PhoneBrowserShell/TabletBrowserShell 可运行；**至少一个移动后端可运行（M4 硬指标 HarmonyOS + 第二后端 Android）**；gesture/navigation/restoration/platform 移动适配 skeleton；`ui/testing`/`ui/devtools` responsive preview / layout bounds / semantics snapshot；design-system 首个风格包。依赖：M2/M3。

---

## Testing & Quality Gates

### 测试层次

| 层次 | 内容 | 运行频率 |
|------|------|----------|
| 单元测试 | 每个 `ui/*` / `foundation/text` / `browser-ui/chrome` crate 的 `#[test]`（无窗口） | 每次修改后 |
| 组件/集成测试 | Widget mount/update、invalidation、theme、i18n、text foundation、browser-ui/chrome props/action、scene snapshot | 每个 milestone 验证 |
| DSL/表达式测试 | parser golden / typecheck / eval（确定性 EvalContext）/ sandbox negative | M3 每次修改后 |
| 浏览器 smoke | `cargo run --bin zero-browser` + `make product-smoke`（welcome.html vs chromium Oracle） | 涉及浏览器迁移时 |
| 全量回归 | `make test` + `make reftest`（确保不拖累 `rendering-compat`） | 每轮执行结束 |

### 质量门禁

| 门禁 | 标准 | 不通过时的处理 |
|------|------|----------------|
| 编译 | `cargo build --workspace` 零错误 | 立即修复 |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` 零警告 | 立即修复 |
| 格式化 | `cargo fmt` 无变更 | 提交前格式化 |
| 现有测试 | `make test` 零失败 | 立即修复，**不允许带着红灯继续** |
| Reftest/product-smoke | `make reftest` / `make product-smoke` 不退化（不拖累 `rendering-compat`） | 涉及渲染/布局变更时必跑；退化则立即回滚或修复 |
| 新增代码测试覆盖 | 每个新增 SDK 能力 / 行为变化 / 回归修复必须有对应测试 | **不允许只改代码不加测试** |
| 文件大小 | 单 `.rs` ≤ 2000 行 | 超出按职责拆分 |
| 依赖许可 | 仅 MIT/Apache-2.0/BSD；最小化；论证必要性 | 违反则换方案 |

### Coverage 要求（长期主线）

- 新增 `ui/*` + `foundation/text` crate line coverage ≥ 85%（M1 建立基线后持续向目标推进，曲线写入 master.md）。
- 全仓 coverage 以 line 95.46% / function 96.94% / region 94.88% 为非回归 floor。
- 统一口径走 `scripts/check-coverage.sh`（或等价统一脚本）；报告路径写入 master.md Latest Evidence。
- **禁止**缩小统计范围 / 把模块排除出口径 / 只测 happy path 来伪造达标。
- 若缺少 coverage 测量手段 / 统一脚本 / 报告链路 / 某模块暂无法纳入口径——视为要继续推进的 active milestone（补齐 coverage 测量能力），**不是**终止条件。

### 证据持久化

每轮执行结束后，以下证据必须持久化到 `docs/goal/ui-sdk/evidence/`：

```
evidence/
├── test-<timestamp>.txt               # make test 摘要
├── coverage-<timestamp>.txt           # coverage 摘要（ui/* + 全仓 floor）
├── product-smoke-<timestamp>.txt      # welcome.html vs chromium diff（涉及浏览器迁移时）
├── dep-isolation-<timestamp>.txt      # 通用 crate 不依赖浏览器业务的机械验证
├── capability-matrix-<timestamp>.md   # 当前已验证通过的能力矩阵 + 未解决缺口
└── <milestone>-<topic>-<timestamp>.{md,txt,png}  # 关键验收 / scene snapshot / 根因分析
```

---

## Latest Evidence

**初始状态（2026-06-30，本 goal 创建时点）**：

- 任务**尚未开始实际推进**：`ui/` / `foundation/text` / `browser-ui/chrome` 均不存在；无任何 SDK crate、无三棵树实现、无 DSL。
- `docs/goal/ui-sdk/master.md`、`archive/`、`evidence/` **均不存在**。
- 无任何与本目标直接对应的测试证据、coverage 数据（针对 ui/*）或验收结果。
- 仓库现状：浏览器自绘 UI 耦合在 `apps/browser`；`make test` / `make reftest` / `make product-smoke` 由 `rendering-compat` 主线维护且当前可用（作为本目标非回归参照）。

> 因此当前**严禁**输出 `DONE`。首轮执行的正确输出是 `CONTINUE: <bootstrap + M1 ui/core 第一步>`。

执行器从 M1 开始执行；每轮把真实 evidence 追加到本 section 的运行态副本（即 `master.md` 的 Latest Evidence），入口文档的「Latest Evidence」只保留初始事实与状态判定原则，不持续 append 运行态。

---

## Document Control / Archive Policy

本目标采用**两层文档控制平面**，路径固定，后续所有 `rally run` session 都以下列路径为准：

### 入口文档（稳定，不频繁修改）

- **路径**：`docs/goal/ui-sdk.md`（本文件）。
- **职责**：定义长期 Mission、Support Envelope、Done Criteria、Testing & Quality Gates、执行协议和文档治理规则。
- **修改条件**：仅在 contract 本身发生实质变化时修改（如调整 DONE 终局范围、里程碑边界、技术路线）。**禁止**在每轮执行里重写它。
- **日常进展、evidence、active milestone 更新一律写入 `master.md`**，不得回写入口文档，也不得把入口替换成 `master.md` 或其它形态。

### 运行时控制平面（持续演进）

- **路径**：`docs/goal/ui-sdk/master.md`。
- **职责**：当前真实状态的唯一控制面板，保存仍有效的目标边界、done criteria 进度、active milestone、测试基线、coverage 曲线、验证证据、下一步计划、依赖决策日志。
- **演进原则**：`master.md` 是持续演进的增量控制面，**不是**一次性交付物。创建第一版只表示治理框架建立，**不表示**核心目标已被覆盖，更**不表示**任何核心能力已完成。它不能无限 append——过时内容必须重写、压缩或迁移到 archive。
- **自洽要求**：active milestone / done criteria 进度 / coverage matrix / Latest Evidence 不得互相矛盾；冲突时先修正文档与状态判断再推进。

### 归档区（历史记录）

- **路径**：`docs/goal/ui-sdk/archive/`。
- **职责**：保存已完成 milestone 的详细过程、关键决策、验证结果、commit hash 和历史证据。archive 是历史记录区，**不是**当前状态来源。
- 证据区：`docs/goal/ui-sdk/evidence/`（见 §Testing & Quality Gates）。

### 第一轮必须先完成的 checklist（强制，非可选，不得延后）

执行器第一次进入本目标时，**必须**按序完成：

1. **复核仓库事实**：确认 `ui/` / `foundation/text` / `browser-ui/chrome` 是否已存在、workspace members 现状、`apps/browser/src` 相关文件、text/font 依赖现状；与本文 Current Proven Baseline 比对，差异写入 master.md。
2. **确认 done criteria**：通读本文 Done Criteria，确认无歧义；有异议先在 master.md 记录并按 §Final Output Contract 处理。
3. **创建 `docs/goal/ui-sdk/master.md`**：包含 Active Milestone（M1）、Done Criteria 进度表（初始全 `[ ]`）、Current Proven Baseline（复核后）、Testing & Quality Gates、coverage 基线（待 M1 建立）、Latest Evidence（初始：尚未开始）、依赖决策日志、下一步。
4. **创建 `docs/goal/ui-sdk/archive/` 与 `docs/goal/ui-sdk/evidence/`** 目录（空目录加 `.gitkeep` 或首轮 evidence）。
5. **确认测试基线**：跑一次 `make test`（必要时 `make product-smoke`）确认当前绿色基线并记录。
6. **选定第一个 active milestone**：M1（广度骨架），并按下方衔接条款立即进入 `ui/core` 第一步。

### Bootstrap 后的衔接（强制）

完成第一版 `master.md` + archive bootstrap 后，执行器**在同一轮内**必须继续启动第一个真实 milestone（M1 的 `ui/core` 最小类型 + 单测，加入 workspace 并通过 `cargo test -p zero-ui-core`），直接推进核心目标能力本身。**不得**把「文档框架已建好」当成 milestone 完成或收口依据。

### 自主性原则（长期无人值守）

在 done criteria 未满足前，执行器必须**自主**：探索当前仓库现状、识别能力缺口、拆解 milestones、实现代码、补测试、验证、归档进展、持续推进——而不是等待用户逐步下达下一条指令。遇到需要多轮推进的架构任务，不要把「需要多会话/长期推进」当成需要用户决策的阻塞；应更新 `master.md` 并以 `CONTINUE: <下一步>` 传给后续轮次。

---

## Final Output Contract

### 输出协议（每轮最后一行必须是下列三者之一）

- **`DONE`**：仅当 Done Criteria（DC-1~DC-18）**全部满足**、且目标能力本身（通用 UI SDK + 浏览器迁移 + DSL + 移动运行时）达到 production-ready 并被自动化证据广泛证明时。即使测试全绿/coverage 达标/文档齐全，也不自动等于 DONE。
- **`CONTINUE: <下一步>`**：默认输出。只要仍可继续推进（包括 verify session 发现缺口但仍可修复、coverage 暂不可验证、某模块尚未实现、文档状态不一致需修正等），就必须输出 `CONTINUE: <具体下一步>` 并转回执行，**不得**输出解释性段落或泛泛重述。
- **`BLOCK: <原因>`**：仅限真正无法继续的情形。**本目标默认禁用 BLOCK**：未完成、证据不足、coverage 暂时无法验证、依赖未定、文档状态不一致、缺少测量手段——**都属于继续推进信号，不属于 BLOCK**。执行器应把这些视为当前工作内容（如「补齐 coverage 测量能力」「评估并选定 serde_yaml/手写 parser」「确认 text foundation 落点」），而非终止条件。

### verify 发现缺口后的处理

verify session 发现缺口时，必须**转回执行**而不是收口：输出 `CONTINUE: <要修的缺口与下一步>`，更新 master.md，继续推进。严禁在任务尚未开始或远未完成时错误输出 `DONE`。

### 仓库协作纪律（来自 docs/rally/run-rules.md）

- 文档/注释默认中文；执行日志与反馈（含警告/报错）用英文。
- 有阶段性进展时及时在 `ui-sdk` 分支提交并推送远端，并及时拉取远端更新并 rebase。
- 跑测试/WPT reftest 必须用 `make test` / `make reftest`（release + `scripts/test-guard.rs` 包裹），**禁止裸跑** `cargo test` 或 `cargo run --bin zero-wpt-runner -- reftest`（OOM 保护，见 `docs/rally/oom-guard.md`）。
- 涉及渲染/布局变更时额外跑 `make product-smoke`（welcome.html vs chromium Oracle 回归门禁，见 `docs/rally/run-rules.md`；属姊妹目标 `rendering-compat` 的 DC-13，非本目标 DC-13）。
- 单 `.rs` 文件 ≤ 2000 行。
- 取得重大进展或遇到真正卡点（需用户决策、长时间阻塞、无法继续推进）时，通过飞书 CLI 以应用机器人身份通知本人（命令见 run-rules.md），仅为告知，不阻塞或改变后续工作流。

### 依赖自治条款（执行器自主决策，已与用户确认）

执行器自主决定下列依赖选择，硬约束为「仅 MIT/Apache-2.0/BSD、最小化新依赖、优先复用 workspace 已有 crate、论证必要性、决策与候选评估写入 master.md + archive」：YAML 解析（serde_yaml vs 手写）、表达式 parser（parser combinator crate vs 手写）、i18n（ICU4X/Fluent vs 手写 plural/RTL）、text shaping/font（复用 swash/rustybuzz/fontdb/unicode-bidi vs 新增）、布局（复用 taffy vs 自研）。**依赖不确定性不是 BLOCK**。

### 执行技能路由（来自 spec §6.7）

| 范围 / 触发条件 | Skill | 模式 | 原因 |
|---|---|---|---|
| 需求/RFC 发生实质变更（超出本 contract） | `lei-spec-rfc`（`C:\Users\leizo\work\skills\spec-rfc\SKILL.md`） | required | 本目标的上游 Spec/RFC 即由其产出；变更须回写 spec 并同步本 goal |
| 浏览器迁移可视验收、截图、产品 smoke | `lei-product-acceptance` | preferred | 浏览器迁移后需可视验收（DC-14 零退化门禁） |
| HarmonyOS 移动后端（若 M4 选定 HarmonyOS） | `lei-harmonyos6-dev` | preferred | ArkTS / ArkUI / `.ets` / Ability / `@ohos` kit 适配（API 12-22 兼容） |

说明：`required` = 实施阶段必须经该 skill 处理这类任务，环境中缺失则停止并报告；`preferred` = 优先建议经该 skill 处理，不可用可回退通用执行器但须在执行报告说明。其余无专用 skill 需求的任务由通用执行器承担。



### 终局输出验证清单（宣布 DONE 前必须逐项确认）

- [ ] DC-1~DC-18 全部满足，且每条有 evidence 落盘。
- [ ] 通用 UI SDK 可被外部程序复用（counter 示例不依赖浏览器 crate）。
- [ ] 浏览器完整迁移、零退化（`make test` / `make product-smoke` / `make reftest` 不退化）。
- [ ] DSL + 完整表达式语言 + sandbox 全绿。
- [ ] 至少一个移动后端可运行；design-system 首个风格包交付。
- [ ] coverage：ui/* + text-foundation ≥ 85%，全仓不低于 floor。
- [ ] `master.md` 各 section 自洽，无矛盾；archive 已归档全部已完成 milestone。

