# 基于 Rust 从零构建跨平台浏览器的技术调研

日期：2026-05-30  
范围：仅技术调研，不进入 Spec/RFC、路线图或实现  
目标：评估一个“可复用 `webview` 库 + 跨平台浏览器应用”的现实可行技术路线，重点覆盖 OmniTerm 代码复用、Web 内核选择、JS/WASM 引擎边界、平台可达性与生产化风险。

> ⚠️ **约束更新（2026-05-30 第四轮）**：用户最新澄清为“**不能直接依赖 Servo**，并且目标上不希望项目主线建立在 `MPL` 技术线上；但如果 `html5ever`、`V8`、`QuickJS`、`Wasmtime` 等存在现成 Rust 模块，则**允许直接作为第三方依赖引用**，不要求复制到当前仓库维护”。本文以下结论都按这个前提成立。

## 来源阅读说明

| 标签 | 含义 | 本文中的主要来源 |
|---|---|---|
| **第一手事实** | 本地代码、官方文档、官方仓库、官方 API 文档 | OmniTerm 本地 crate、V8、Wasmtime、winit、wgpu、html5ever、QuickJS/rquickjs、Chromium 设计文档、相关开源项目官方仓库 |
| **外部检索** | 官方站点的公开页面、项目博客、官方演讲材料 | Servo WPT 页面、Servo 2025 stats、Servo OpenHarmony/WebDriver 演讲材料、Chromium 设计文档、HarmonyOS WebView 官方 codelab |
| **⚠️ 假设** | 基于已知事实的合理判断，但没有直接文档逐字证明 | OpenHarmony/HarmonyOS PC 宿主层仍需额外平台胶水；双页面 JS 引擎在浏览器级语义下难以首期做到完全等价 |
| **💡 推理** | 基于多源事实得出的工程判断 | “生产可用浏览器”与“自研 parser + JS 引擎拼装”不匹配；trait 边界应放在完整 web core 之上，而不是 DOM/IDL 之下 |
| **作者综合** | 基于多源信息整理出的比较表/模块划分 | 路线对比表、推荐分层、可复用边界划分 |

## Executive Summary

结论先行：

1. 在“**不能直接依赖 Servo**，但允许直接引用 permissive Rust 依赖”的前提下，最现实的路线不再是 `servo` crate 集成，也不是复制 Servo 技术线源码，而是 **基于 permissive 第三方模块的自建 web core 路线**：直接依赖 `html5ever`、`rusty_v8`/`rquickjs`、`wasmtime`、`wgpu`、`winit` 等许可友好的模块，自行补齐 DOM、CSS、布局、渲染和浏览器宿主层。[17][23][24][26][29][39]
2. 这条路线的核心不是“完全从零手写一切”，而是“**只使用与当前许可证策略兼容的外部依赖**”。从当前可验证来源看，`html5ever` 是 Apache-2.0/MIT，`rusty_v8` 是 MIT，`rquickjs` 与 QuickJS 是 MIT，`Wasmtime` 是 Apache-2.0，`wgpu` 是 Apache-2.0/MIT，`winit` 是 Apache-2.0，这些都符合“直接依赖、不复制维护”的依赖模式。[17][23][24][26][28][29][39]
3. OmniTerm 里最有价值的资产，仍然不是“浏览器内核”，而是 **宿主层与渲染基础设施层**：场景/primitive/backend 分层、GPU atlas 与缓存、字体 fallback、软件渲染后备、图片对象缓存、FFI/WASM 边界，这些都适合迁移到浏览器的 chrome/UI/offscreen/testing/embedding 层。[1][2][3][4][5][6]
4. “主内容 JS 引擎通过 trait 在 V8 和 QuickJS 间切换”在这条自建路线下**理论上可做**，因为你自己掌控 DOM/IDL/host bindings；但要做到浏览器级双引擎等价支持，成本仍然很高。更稳妥的研究结论是：trait 抽象可以从第一天设计，但首个可用内核应只选一个默认页面 JS 引擎，另一个放到 feature-gated 次级实现。[25][26][28][29]
5. V8、QuickJS、Wasmtime、wasmi 仍然有价值，但更合理的位置是 **扩展脚本、用户脚本、自动化脚本、非页面插件沙箱**，而不是页面主内容执行引擎本身。[25][26][27][28][29][30][31]
6. 在你刚澄清的工作定义下，“项目主线闭源”与“直接依赖 permissive 第三方 Rust 模块”并不冲突；但 **Servo / Stylo / WebRender / rust-cssparser / Lightning CSS 这类 MPL 技术线应从主线候选集中排除**。当前来源显示 `rust-cssparser`、`servo` 为 MPL-2.0，而 `lightningcss` crate 元数据也标为 MPL-2.0。[35][36][37][40]
7. 平台上，macOS/Linux/Windows/Android 是最强主线；OpenHarmony 已有 Servo 官方开发与演讲证据，但宿主层成熟度仍明显弱于前四个平台，应视为高风险平台扩展而不是首发等权平台。[7][10][16][23][24]
8. 生产可用浏览器的真正门槛不只是解析与执行，而是 **多进程/沙箱、安全隔离、导航模型、网络栈、媒体、存储、可访问性、输入法、测试与兼容性**。Chromium 和 V8 的官方文档都把进程隔离当成基础安全假设，而不是优化项。[27][32][33]

## 1. 研究对象与判断标准

用户目标可以拆成两个产品：

- 一个可复用的 `webview` 模块，供其他应用以 `lib` 方式引入。
- 一个跨平台浏览器应用，支持多标签页、收藏夹等桌面浏览器能力。

这里最关键的判断不是“能不能写出一个浏览器 UI”，而是“能不能得到一个足够新的、足够兼容的、足够安全的 web core”。多标签页、收藏夹、历史记录、下载管理都属于浏览器壳层问题；真正决定成败的是页面内核、进程模型和平台宿主整合。基于 Chromium 的官方设计文档，浏览器生产化基线天然包含浏览器进程、渲染进程，以及进一步拆出的 GPU/网络/存储等进程或服务。[32]

我采用以下判断标准：

| 维度 | 含义 |
|---|---|
| 标准兼容上限 | HTML/CSS/JS/WASM/DOM/CSSOM/布局/绘制能力的上限 |
| 首个可复用 `webview` 的实现难度 | 不是 demo，而是可以被别的应用稳定嵌入 |
| 平台可达性 | macOS/Linux/Windows/Android/OpenHarmony 的真实落地难度 |
| 生产化距离 | 安全、隔离、媒体、存储、测试、性能、崩溃恢复、可观测性 |
| OmniTerm 复用率 | 哪些现有代码可直接迁移或重构迁移 |
| 引擎可插拔性 | trait/feature 的切换是否处在现实可维护边界内 |

> **📌 Source Notes (Chapter 1)**
>
> - **第一手事实** [32]: Chromium 官方多进程架构文档证明浏览器生产化问题不止是渲染。
> - **💡 推理**: “多标签页/收藏夹是壳层问题，web core 才是主风险”是基于 Chromium 进程模型与浏览器工程常识得出的判断。
> - **⚠️ 假设**: 本章未依赖你尚未存在的 ZeroWeb 代码结构，因为当前仓库几乎为空。

## 2. OmniTerm 中可复用的资产与不可复用的部分

### 2.1 值得直接迁移或重构迁移的部分

OmniTerm 的价值主要在“渲染宿主层”而不是“浏览器内容内核”。

`omniterm-terminal-render` 已经把 `RenderFrame`、`RenderScene`、`RenderPrimitives` 和 `RenderBackend` 明确分层；这类“场景描述 -> primitive -> backend”的结构很适合迁移成浏览器壳层的 UI 渲染、offscreen 渲染、截图、测试渲染与后备软件路径。[1]

`omniterm-terminal-render-wgpu` 已经具备 GPU glyph atlas、pane 级缓存、surface format 选择、GPU buffer 缓存等能力。这些能力对浏览器壳层仍然有价值，尤其是标签栏、地址栏、书签栏、开发者工具面板、缩略图和嵌入场景下的离屏合成。[2]

`omniterm-terminal-render-soft` 具备 `fontdue + swash` 字体栈、glyph cache、emoji/CJK fallback、软件渲染后备路径。这对浏览器 UI 文本渲染、无 GPU 环境、测试环境和 HarmonyOS/OpenHarmony 这类平台移植的 fallback 路径都非常有价值。[3]

`omniterm-terminal-image` 已经把图像对象、像素缓存、容量限制、anchor、GC 统计做了结构化封装。它不等于浏览器图片解码/缓存系统，但适合作为图片对象缓存与测试资产缓存的起点。[4]

`omniterm-terminal-ffi` 和 `omniterm-terminal-wasm` 表明 OmniTerm 已经有清晰的 ABI 边界和 WASM 友好封装思路，这对未来把 `webview` 暴露给其他宿主语言、插件系统或测试 harness 很有帮助。[5][6]

### 2.2 不应误判为“浏览器内核已完成”的部分

这些 OmniTerm 资产都不能替代浏览器必须具备的：

- DOM/HTML 解析后的树与增量更新模型
- CSSOM、选择器匹配、级联、继承、样式计算
- 浏览器级布局模型（inline formatting、fragmentation、tables、positioning、stacking contexts、scrolling、painting order）
- 页面导航、iframe、same-origin、storage、network、timers、event loop
- 页面内 JS 与 DOM/IDL/GC 的深度耦合

换句话说，OmniTerm 适合成为 **host runtime / rendering foundation**，不适合被误用为 **web content engine**。

> **📌 Source Notes (Chapter 2)**
>
> - **第一手事实** [1][2][3][4][5][6]: 上述判断来自对 OmniTerm 本地 crate 代码与接口的直接阅读。
> - **💡 推理**: “可复用到浏览器壳层，但不能替代 web core”是基于这些 crate 的职责边界与浏览器所需子系统对比得出。
> - **⚠️ 假设**: 迁移成本未做代码级 PoC 测量，因此这里只判断“边界适配性”，不判断具体工时。

## 3. 为什么“从零拼装一个现代浏览器内核”与目标不匹配

### 3.1 parser + JS 引擎 + WASM 引擎远远不够

`html5ever` 确实是 Servo 系里的 HTML 解析器，能够按 WHATWG HTML 规范解析/序列化，并明确以生产浏览器所需 hook 为目标，例如 `document.write` 一类能力。[17]  
`rust-cssparser` 是 CSS Syntax Level 3 的 Rust 实现，但它明确说明自己 **不做** 最后一步，即不负责把 component values 完整变成“你想支持的属性、选择器与更高层语义”。[18]  
`lightningcss` 是优秀的 CSS parser/transformer/minifier，但它的定位是“解析、变换、降级、压缩”，不是完整浏览器样式系统。[19]  
`taffy` 目前只实现了 CSS 规范中的 Flexbox、Grid 和 Block 布局算法；这对应用 UI 很强，但距离完整浏览器布局模型仍然很远。[20]

这意味着：即便你把 `html5ever + rust-cssparser/lightningcss + taffy + V8/QuickJS + Wasmtime/wasmi` 全接起来，你得到的也不是“现代浏览器内核”，而只是若干底层积木。

### 3.2 真正缺的是 browser-grade style/layout/render/script integration

即便在排除 `MPL` 技术线之后，问题本质并没有变化：真正稀缺的不是 parser 或单个引擎，而是 browser-grade 的 style/layout/render/script integration。  
这意味着：在你当前可接受的依赖边界下，`html5ever` 可以提供 HTML 解析，[17] `taffy` 可以提供部分布局算法，[20] `wgpu`/`winit` 可以提供图形和宿主地基，[23][24] `rusty_v8` 或 `rquickjs` 可以提供 JS 执行，[26][29] 但 DOM、CSSOM、样式系统、完整布局、导航与安全模型仍需你自己构建。

因此，“闭源主线 + permissive 依赖”是可以成立的，但它把项目主线从“整合现成 browser-grade Rust 内核”转成了“以 permissive 积木自建 browser-grade 内核”。

### 3.3 生产化还需要安全与进程模型

Chromium 的官方多进程架构文档把浏览器进程、渲染进程以及更多服务拆分视为健壮性和安全性的基础。[32]  
Chromium 的 Site Isolation 文档进一步给出真实代价：Chrome 67 在桌面全站点隔离下大约有 10% 到 13% 的额外内存开销；Chrome 77 在 Android 上对登录站点隔离时大约有 3% 到 5% 开销。[33]  
V8 官方关于 untrusted code mitigations 的文档也直接建议：若执行不可信 JS/WASM，应考虑将其放入独立进程；某些极端计算工作负载下，mitigation 的性能代价可高达 15%。[27]

这几个来源合起来说明：浏览器的“高标准兼容 + 高性能 + 生产安全”不是单进程 toy engine 能够自然演化出来的。

> **📌 Source Notes (Chapter 3)**
>
> - **第一手事实** [17][18][19][20][21][22][27][32][33]: HTML/CSS/parser/layout/render/security/process 的能力边界来自官方仓库与官方文档。
> - **💡 推理**: “积木齐了不等于浏览器内核齐了”是基于这些组件职责边界得出的工程判断。
> - **⚠️ 假设**: 本章没有直接测量 ZeroWeb 的资源预算，因此“与目标不匹配”是从系统复杂度而非团队规模估算得出。

## 4. 候选路线比较

下表是本文的核心结论之一。  
**作者综合**：这张表是基于下列来源做的工程比较，不是任何原始来源的原话。[7][8][9][10][12][13][17][18][19][20][21][22][25][26][28][29][30][31][32][33][34][35][36][37][38][39]

| 路线 | 标准兼容上限 | 首个可复用 `webview` 距离 | 主内容 JS 可切换性 | 平台可达性 | 许可证匹配度 | 生产化前景 | 结论 |
|---|---|---:|---|---|---|---|
| **A. 基于 permissive 依赖的自建 web core** | 中 | 远 | 中到高 | 高 | 高 | 低到中 | **当前主线** |
| **B. Servo / MPL 技术线整合** | 高 | 中到远 | 低 | 高（OHOS 例外） | 低 | 中高 | **与你的许可证目标冲突，应排除** |
| **C. 系统 WebView 壳层** | 高（借宿主内核） | 近 | 低 | 高 | 中到高 | 高 | **可做对照/验证，但不满足“自有内核”目标** |

### 4.1 路线 A：基于 permissive 依赖的自建 web core

在当前约束下，这条路线是唯一与许可证策略直接兼容的主线。

路线 A 的正确理解是：

- 不直接依赖 `servo`、`stylo`、`webrender`、`mozjs`、`rust-cssparser`、`lightningcss` 等 `MPL` 技术线；
- 直接依赖 `html5ever`、`rusty_v8`/`rquickjs`、`wasmtime`、`wgpu`、`winit`、`taffy` 等 permissive 模块；
- 自己实现 DOM、CSSOM、样式系统、布局整合、渲染管线、导航模型与安全边界；
- 对外发布的 `webview` 和浏览器应用只暴露你自己的 crate/API。

这条路最大的优点是许可证边界清晰、项目主线可控。最大的缺点是：你放弃了 Rust 世界里最成熟的浏览器技术线，只能用 permissive 积木自己补齐中间所有高复杂度层。

### 4.2 路线 B：Servo / MPL 技术线整合

从纯技术角度，这条路的标准兼容上限最高，因为 Servo Book 的项目结构已经展示了 browser-grade 技术母体：`html5ever`、`rust-cssparser`、`mozjs`、`stylo`、`webrender`。[9]  
但在你的当前目标下，这条路的关键问题已经不是技术，而是许可证：`servo`、`rust-cssparser`、`stylo`、`webrender` 都属于 `MPL` 技术线。[21][22][36][37]

因此，路线 B 现在不是“不够现实”，而是**直接与你的许可证目标冲突**，应从主线候选中排除。

### 4.3 路线 C：系统 WebView 壳层

HarmonyOS 官方 WebView codelab 已经展示了宿主平台原生 WebView 的加载 URL、本地 HTML、JS bridge、前进后退等能力。[34]  
这说明“浏览器壳层”在多数平台上都可以很快做出来。

但这条路不满足你要构建“自有内核 + 可控内核抽象”的目标，所以这里只能把它视为：

- 产品验证基线
- 平台对照组
- 某些高风险平台的 fallback

> **📌 Source Notes (Chapter 4)**
>
> - **第一手事实** [7][8][10][11][13][17][18][19][20][21][22][34][35][36][37][38][39]: 各路线的技术边界来自官方文档与官方仓库。
> - **外部检索** [32][33]: Chromium 设计文档用于衡量“生产浏览器”的系统复杂度基线。
> - **💡 推理**: “基于 permissive 依赖的自建 web core 是当前主线”来自上述事实的综合，而不是任何单一来源的原话。
> - **⚠️ 假设**: 路线 C 作为 fallback 的可行性高，但它不满足原始产品目标，因此不作为主推荐。

## 5. JS / WASM trait 抽象应该放在哪里

### 5.1 主内容 JS 引擎能否抽象成 `trait JsEngine`

在你当前接受的自建路线下，主内容 JS 引擎**理论上可以**抽象成 `trait JsEngine`，因为 DOM/IDL/host bindings 都由你自己定义。  
但这并不意味着“双引擎浏览器级等价支持”是低成本的。无论是 V8 还是 QuickJS，真正困难的都不是 `eval()` 或基本对象暴露，而是：

- DOM 对象如何映射到各自 VM 的对象模型
- GC/生命周期如何与 Rust 侧节点树协同
- Promise、microtask、timers、event loop、exception、Web API host hooks 如何保证一致语义

因此，更现实的做法是：

- 第一版先选一个默认页面 JS 引擎
- 把抽象边界预留好
- 第二个引擎只做 feature-gated 实验实现，不承诺首期完全等价

### 5.2 V8 和 QuickJS 各自适合什么位置

V8 官方文档显示其 embedding 是强 C++/VM 语义导向的；上下文、handle、template、安全 token 都是浏览器级脚本宿主需要深入处理的机制。[25]  
`rusty_v8` 提供了 Rust binding，但其构建要求本身就说明了接入复杂度：从源码构建需要 Python 3、`curl`、Linux 上的 `glib-2.0` 与 `libclang 19+`，且 32-bit Windows 不支持；Android 也需要专门 target 构建。[26]

QuickJS 官方文档则强调它“小、易嵌入、ES2023 支持度高、启动快”，但也明确 **不支持 ECMA402**。[28]  
`rquickjs` 文档进一步说明 QuickJS runtime 默认被 mutex 保护，同一 runtime 不适合多线程同时执行，`parallel` 支持也还是 experimental。[29]

因此：

- **V8** 更像“高性能、复杂、安全与构建成本高的重型嵌入引擎”。
- **QuickJS** 更像“轻量、易嵌入、非常适合用户脚本/扩展脚本/自动化脚本”的轻型引擎。

### 5.3 WASM 也不该和主页面语义简单拆开

Wasmtime 是面向宿主嵌入的 standalone runtime，强调 WebAssembly、WASI 和 Component Model。[30]  
Wasmi 则是规格贴合度高、宿主可控、解释器路径清晰的 Wasm VM。[31]

这两者都很适合作为：

- 插件运行时
- 扩展沙箱
- 非页面计算任务
- 内部自动化或测试组件

但“页面里的 WebAssembly”并不只是“跑一个 `.wasm` 文件”这么简单。它要和 JS 对象模型、异常、Promise、事件循环、同源策略、Web API host bindings 一起工作。V8 的 mitigations 文档甚至把 JS 和 WebAssembly 作为同一类不可信代码处理对象讨论。[27]

### 5.4 推荐的 trait/feature 边界

现实可维护的抽象边界应是：

- `page_js = v8 | quickjs`：页面主内容 JS，引擎抽象预留，但首期只稳定支持一个
- `script_sandbox = v8 | quickjs`：扩展、用户脚本、自动化脚本
- `wasm_sandbox = wasmtime | wasmi`：非页面插件/工具运行时

而不是：

- `page_js_engine = v8 | quickjs`
- `page_wasm_engine = wasmtime | wasmi`

前者现实，后者在“首个生产级 webview”目标下不现实。

> **📌 Source Notes (Chapter 5)**
>
> - **第一手事实** [25][26][27][28][29][30][31]: JS/WASM 引擎边界来自官方文档与官方仓库。
> - **💡 推理**: “页面引擎可抽象，但双引擎首期难以等价”是基于 V8/QuickJS 的嵌入复杂度与宿主绑定成本得出的结论。
> - **⚠️ 假设**: 未来若出现新的 Rust browser-grade web core，这个边界可重评；但在 2026-05-30 可验证资料下，这仍是最稳妥结论。

## 6. 自主可控源码的现实定义

### 6.1 “自主可控”不等于“所有代码都必须从零写”

如果“自主可控”的定义是：

- 第一方核心逻辑、产品 API、发布节奏和架构决策由你掌控；
- 运行时允许依赖 permissive 的第三方 Rust 模块；
- 构建过程不依赖不可审计的第三方二进制黑盒；
- 对外只暴露你自己的 API 和发布物；

那么这和“闭源主代码 + 第三方 permissive 依赖”是兼容的。

如果“自主可控”的定义是：

- 所有核心代码都必须由本项目原创；
- 不接受 MPL 技术线进入产品主线；
- 主内容引擎必须自由替换 V8/QuickJS；

那么目标会从“构建生产浏览器”退化成“长期浏览器内核研发项目”，交付周期和风险都会显著上升。

### 6.2 许可证边界

基于当前可验证资料：

- `servo/html5ever` 是 Apache-2.0 / MIT 双许可。[17]
- `rusty_v8` 是 MIT；其 README 也强调可从源码完整构建，不必依赖外部预编译 blob。[26]
- QuickJS 是 MIT。[28]
- Wasmtime 是 Apache-2.0。[39]
- wasmi 是 MIT / Apache-2.0。[31]
- `wgpu` 是 Apache-2.0 / MIT。[41]
- `winit` 是 Apache-2.0。[24]
- `lightningcss` crate 元数据是 MPL-2.0。[40]
- `rust-cssparser` 是 MPL-2.0。[36]
- `servo` 主仓库是 MPL-2.0。[37]

Mozilla 官方 FAQ 明确说明 MPL 是 **file-level copyleft**，允许与其他许可证代码共同组成 Larger Work，但这也正说明：如果你的目标是避免把产品主线建立在 `MPL` 技术线上，那么这些依赖应从主线排除。[35]

因此：

- 如果你说的“闭源主线”是“第一方代码闭源、第三方依赖保留各自许可与 notices”，那么 `html5ever`、`rusty_v8`、`rquickjs`、`Wasmtime`、`wgpu`、`winit` 仍可直接依赖。
- 如果你说的“整个项目最终统一成单一闭源许可证”是指**连第三方依赖都不想保留原许可证义务**，那就连 permissive 依赖也无法完全满足；这通常不现实。

这不是技术障碍，而是源码治理与法务边界。本文不是法律意见；真正发布前应做一次许可证审计，并确认你接受的是“闭源主代码 + 第三方 notices”而不是“整个组合物单一许可证”。

### 6.3 供应链控制建议

在“闭源主线 + 允许 permissive 依赖”前提下，更合理的供应链策略是：

- 禁止运行时关键依赖直接从公网 HEAD 拉取；
- 对 `rusty_v8` 这类可能使用预编译归档的依赖，优先切换为源码构建或内部制品库镜像；官方文档给出了源码构建路径。[26]
- 在 `Cargo.lock`、内部 crate 镜像或私有 registry 层固定版本，不依赖未锁定的远程 HEAD；
- 为所有第三方依赖维护许可证清单、版本清单、源码来源和安全更新策略；
- 产品主线禁止引入 `MPL` 技术线依赖。

> **📌 Source Notes (Chapter 6)**
>
> - **第一手事实** [17][24][26][28][31][35][36][37][39][40][41]: 许可证、源码构建与依赖边界来自官方仓库和官方 FAQ。
> - **💡 推理**: “闭源主线 = 第一方代码闭源 + 第三方 permissive 依赖保留 notices”是基于这些许可证事实得出的工程定义。
> - **⚠️ 假设**: 具体法务可接受边界取决于项目商业模式；本文只做技术与源码治理层面的可行性判断。

## 7. 平台可达性与推荐分层

### 7.1 平台判断

`wgpu` 官方 README 说明它原生跑在 Vulkan、Metal、D3D12 和 OpenGL 上，并在 wasm 上支持 WebGL2/WebGPU；支持矩阵中 Windows、Linux/Android、macOS/iOS 都是明确的一等平台。[23]  
`winit` 官方 docs.rs 页面也明确列出了 Windows、Linux、macOS、Android、wasm 等平台与依赖关系。[24]

这说明：如果你把宿主层建立在 `winit + wgpu` 一类通用 Rust 基础设施之上，macOS/Linux/Windows/Android 是有坚实地基的。

OpenHarmony 则不同。Servo 的嵌入文档明确提到 Android/OpenHarmony 交叉编译环境变量。[7] Servo 官方 README 也把 OpenHarmony 列入当前开发平台。[10] 此外，Servo 官方关于 OpenHarmony 的演讲材料显示：OpenHarmony 上已有 WebRender/统一渲染实验、移动端 WebDriver 迁移，以及在 OHOS 上跑 WPT/收集结果的工作。[16]

但我没有在 `winit`/`wgpu` 的官方主文档里看到 OpenHarmony 被列为一等官方平台。[23][24]  
**⚠️ 假设**：这意味着 OpenHarmony/HarmonyOS PC 的浏览器宿主层大概率仍需额外平台胶水或 Servo 专门端口，而不能简单当成“换个 Linux target 就行”。

### 7.2 对浏览器项目的推荐分层

**作者综合**：基于本文所有来源，推荐把最终系统理解为下列层次，而不是单体浏览器。

| 层 | 建议角色 | 备注 |
|---|---|---|
| `webview` | 稳定嵌入接口 | 对外暴露 `WebViewHandle`、导航、输入、脚本桥、纹理/表面输出 |
| `engine` | 默认内核适配器 | 首个现实主线，基于 permissive 依赖自建页面内核 |
| `host-runtime` | 平台窗口、事件循环、surface 生命周期 | 可吸收 OmniTerm 宿主经验 |
| `render-foundation` | 字体、图片、GPU atlas、软件后备、离屏渲染 | 可直接重构迁移 OmniTerm 代码 |
| `browser-shell` | 多标签页、收藏夹、历史、下载、权限 UI | 浏览器应用层 |
| `script-sandbox-*` | 扩展/用户脚本引擎 | V8/QuickJS feature 切换放这里 |
| `wasm-sandbox-*` | 非页面 Wasm 运行时 | Wasmtime/wasmi feature 切换放这里 |

### 7.3 与 OmniTerm 的衔接方式

直接可迁移或强参考迁移：

- `render` 的 scene/backend 分层 [1]
- `render-wgpu` 的 atlas/cache/compositing 机制 [2]
- `render-soft` 的字体 fallback 与软件后备 [3]
- `image` 的图像对象缓存与 GC 限制 [4]
- `ffi` / `wasm` 的宿主边界暴露方式 [5][6]

不建议直接迁移为“页面内核”的：

- terminal cell 模型
- terminal snapshot 到 render frame 的生成逻辑
- 终端输入协议与 parser

### 7.4 关于“生产可用”的现实表述

基于已验证资料，更准确的表述应该是：

- **可以研究并构建一个以 permissive 第三方模块为基础、自建页面内核、并以 OmniTerm 为宿主/渲染基础设施来源的跨平台浏览器体系。**
- **不能把“完全原创从零 + 主内容 JS/WASM 任意切换 + 最新标准 + 一流性能 + 生产可用”视为同一阶段内天然兼容的目标。**

这不是否定目标，而是把目标拆成符合当前 Rust 浏览器生态现实的技术边界。

> **📌 Source Notes (Chapter 7)**
>
> - **第一手事实** [1][2][3][4][5][6][7][10][16][23][24]: 平台与分层判断来自本地代码和官方文档。
> - **外部检索** [34]: HarmonyOS 官方 WebView 能力可作为平台对照，但不是自有内核证据。
> - **⚠️ 假设**: OpenHarmony/HarmonyOS PC 的宿主层仍需专门胶水，是基于官方主文档中缺少一等平台声明所作出的保守判断。
> - **💡 推理**: “OmniTerm 适合 host/render foundation，而页面内核应建立在 permissive 依赖之上自建”是全文最重要的工程综合结论。

## 8. 结论

这项调研的最终结论是：

- **首选路线应当是基于 permissive 第三方模块的自建 web core 路线。**
- **OmniTerm 应被视为宿主层和渲染基础设施仓库，而不是浏览器内核雏形。**
- **页面主内容 JS 引擎可以从一开始预留 `V8/QuickJS` trait 抽象，但首个可用内核应只稳定支持一个默认引擎；`Wasmtime/wasmi` 更适合作为插件或扩展沙箱。**
- **如果“闭源主线”的含义是“第一方代码闭源、第三方 permissive 依赖保留 notices”，则这条路线可行；如果含义是“整个组合物不保留任何第三方许可证义务”，则连 permissive 依赖也无法完全满足。**
- **OpenHarmony/HarmonyOS PC 可以纳入目标平台，但必须被标记为高风险平台适配项。**
- **如果坚持“主内容 JS 引擎可热切换”作为硬前提，技术上并非绝对不可能，但会显著拉长首个生产级 `webview` 的落地时间。**

基于现有公开资料，到 2026-05-30 为止，Rust 世界里最接近你当前目标的现实策略不是“整合 Servo 技术线”，而是“**以 `html5ever`、`rusty_v8`/`rquickjs`、`wasmtime`、`wgpu`、`winit`、`taffy` 等 permissive 模块为基础，自建页面内核，再围绕它构建强宿主、强嵌入、强平台适配的浏览器体系**”。

> **📌 Source Notes (Chapter 8)**
>
> - **第一手事实** [7][8][10][12][13][17][18][19][20][21][22][23][24][25][26][27][28][29][30][31][32][33][35][36][37][39]: 全文结论建立在这些官方与本地来源之上。
> - **💡 推理**: 本章是全文综合判断，不是任何单一来源的直接结论。
> - **⚠️ 假设**: 若未来 Servo 的嵌入 API、WPT 覆盖率、OpenHarmony 宿主链路或新的 Rust web core 生态发生显著变化，本结论需要重新验证。

## 参考来源

| # | 类型 | 来源 | 关键用途 |
|---|---|---|---|
| [1] | 第一手事实 | [omniterm-terminal-render/src/lib.rs]($HOME/work/OmniTerm/terminal/crates/omniterm-terminal-render/src/lib.rs:41) | 场景/primitive/backend 分层 |
| [2] | 第一手事实 | [omniterm-terminal-render-wgpu/src/lib.rs]($HOME/work/OmniTerm/terminal/crates/omniterm-terminal-render-wgpu/src/lib.rs:23) | GPU atlas、pane cache、合成 |
| [3] | 第一手事实 | [omniterm-terminal-render-soft/src/lib.rs]($HOME/work/OmniTerm/terminal/crates/omniterm-terminal-render-soft/src/lib.rs:27) | 字体 fallback、软件渲染、glyph cache |
| [4] | 第一手事实 | [omniterm-terminal-image/src/lib.rs]($HOME/work/OmniTerm/terminal/crates/omniterm-terminal-image/src/lib.rs:10) | 图片对象缓存与 GC |
| [5] | 第一手事实 | [omniterm-terminal-ffi/src/lib.rs]($HOME/work/OmniTerm/terminal/crates/omniterm-terminal-ffi/src/lib.rs:1) | ABI 边界 |
| [6] | 第一手事实 | [omniterm-terminal-wasm/src/lib.rs]($HOME/work/OmniTerm/terminal/crates/omniterm-terminal-wasm/src/lib.rs:9) | WASM 友好封装 |
| [7] | 第一手事实 | [Servo Book: Embedding Overview](https://book.servo.org/embedding/overview.html) | Servo 嵌入目标、Android/OpenHarmony 交叉编译 |
| [8] | 第一手事实 | [docs.rs: `servo` crate](https://docs.rs/servo/latest/servo/) | `Servo` 与 `WebView` API、依赖结构 |
| [9] | 第一手事实 | [Servo Book: Project Structure](https://book.servo.org/design-documentation/project-structure.html) | html5ever/cssparser/mozjs/stylo/webrender 组成 |
| [10] | 第一手事实 | [servo/servo README](https://github.com/servo/servo) | 当前开发平台：macOS/Linux/Windows/OpenHarmony/Android |
| [11] | 第一手事实 | [Servo Book: Getting servoshell](https://book.servo.org/trying/getting-servoshell.html) | servoshell 只是测试浏览器，不是 fully fledged browser |
| [12] | 第一手事实 | [doc.servo.org: `script::dom`](https://doc.servo.org/script/dom/index.html) | DOM reflector 由 SpiderMonkey `JSObject` 持有 |
| [13] | 第一手事实 | [servo/mozjs README](https://github.com/servo/mozjs) | Servo 的 SpiderMonkey Rust bindings |
| [14] | 外部检索 | [Servo WPT Pass Rates](https://servo.org/wpt/) | Servo 官方 WPT 仪表盘 |
| [15] | 外部检索 | [Servo 2025 Stats](https://blogs.igalia.com/mrego/servo-2025-stats/) | 2025 WPT 与社区活跃度变化 |
| [16] | 外部检索 | [Driving Innovation with Servo and OpenHarmony (PDF)](https://servo.org/files/2025-09-13-driving-innovation-with-servo-and-openharmony-unified-rendering-and-webdriver.pdf) | OpenHarmony 渲染/WebDriver 现状 |
| [17] | 第一手事实 | [servo/html5ever README](https://github.com/servo/html5ever) | HTML5 parser、browser hooks |
| [18] | 第一手事实 | [servo/rust-cssparser README](https://github.com/servo/rust-cssparser) | CSS Syntax Level 3，非完整样式系统 |
| [19] | 第一手事实 | [docs.rs: `lightningcss`](https://docs.rs/lightningcss/latest/lightningcss/) | CSS parser/transformer/minifier 定位 |
| [20] | 第一手事实 | [docs.rs: `taffy`](https://docs.rs/taffy/latest/taffy/) | Flexbox/Grid/Block 布局能力边界 |
| [21] | 第一手事实 | [servo/stylo README](https://github.com/servo/stylo) | browser-grade CSS style engine |
| [22] | 第一手事实 | [servo/webrender README](https://github.com/servo/webrender) | GPU-based web renderer |
| [23] | 第一手事实 | [gfx-rs/wgpu README](https://github.com/gfx-rs/wgpu) | 图形后端与支持平台 |
| [24] | 第一手事实 | [docs.rs: `winit`](https://docs.rs/winit/latest/winit/) | 窗口/事件循环平台支持 |
| [25] | 第一手事实 | [V8: Getting started with embedding](https://v8.dev/docs/embed) | V8 embed 复杂度、contexts/security model |
| [26] | 第一手事实 | [denoland/rusty_v8 README](https://github.com/denoland/rusty_v8) | Rust binding、构建成本、Android target、Windows 限制 |
| [27] | 第一手事实 | [V8: Untrusted code mitigations](https://v8.dev/docs/untrusted-code-mitigations) | JS/Wasm 进程隔离与性能代价 |
| [28] | 第一手事实 | [QuickJS 官方文档](https://bellard.org/quickjs/quickjs.html) | 轻量、易嵌入、ES2023、ECMA402 缺失 |
| [29] | 第一手事实 | [docs.rs: `rquickjs`](https://docs.rs/rquickjs/latest/rquickjs/) | QuickJS Rust 封装、线程/特性边界 |
| [30] | 第一手事实 | [Wasmtime 文档](https://docs.wasmtime.dev/) | standalone WebAssembly/WASI/Component Model runtime |
| [31] | 第一手事实 | [docs.rs: `wasmi`](https://docs.rs/wasmi/latest/wasmi/) | Wasm 解释器/宿主控制边界 |
| [32] | 外部检索 | [Chromium Multi-process Architecture](https://www.chromium.org/developers/design-documents/multi-process-architecture/) | 生产浏览器系统复杂度基线 |
| [33] | 外部检索 | [Chromium Site Isolation](https://www.chromium.org/Home/chromium-security/site-isolation) | 隔离模型与内存代价 |
| [34] | 外部检索 | [HarmonyOS Using WebView Codelab](https://developer.huawei.com/consumer/en/codelab/HarmonyOS-WebView/) | 宿主平台现成 WebView 能力对照 |
| [35] | 第一手事实 | [Mozilla MPL 2.0 FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/) | MPL file-level copyleft、Larger Work 边界 |
| [36] | 第一手事实 | [servo/rust-cssparser](https://github.com/servo/rust-cssparser) | CSS syntax crate、MPL-2.0 |
| [37] | 第一手事实 | [servo/servo](https://github.com/servo/servo) | Servo 仓库 MPL-2.0、平台、LTS 发布 |
| [38] | 第一手事实 | [QuickJS official site](https://bellard.org/quickjs/) | QuickJS 可嵌入性、MIT、能力边界 |
| [39] | 第一手事实 | [bytecodealliance/wasmtime](https://github.com/bytecodealliance/wasmtime) | Wasmtime 许可证与运行时定位 |
| [40] | 第一手事实 | [parcel-bundler/lightningcss `Cargo.toml`](https://github.com/parcel-bundler/lightningcss) | `lightningcss` 许可证为 MPL-2.0 |
| [41] | 第一手事实 | [gfx-rs/wgpu LICENSE files](https://github.com/gfx-rs/wgpu) | `wgpu` 为 Apache-2.0 / MIT 双许可 |
| [42] | 第一手事实 | [DioxusLabs/taffy LICENSE](https://github.com/DioxusLabs/taffy) | `taffy` 为 MIT 许可 |
