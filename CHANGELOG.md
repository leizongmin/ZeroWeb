# Changelog

本文件记录项目对外可感知的重要变更。

写法参考 Keep a Changelog，不过这里的版本号要结合项目现状来看：仓库还在实验阶段，所有预发布版本都应该按实验性版本理解。

## [Unreleased]

### 新增

- **页面 JavaScript 运行时**：`script-sandbox` 提供 V8（`v8` crate，rusty_v8 更名）/ QuickJS 双后端 feature gate（V8 持久化 Context 复用；QuickJS 全矩阵 parity，136 个失败清零）、Web Worker、ES Modules、WebAssembly JS API 到 `wasm-sandbox` 的自动桥接。
- **P1a DOM/JS Bridge 原生化（R30xx 系列，已主体落地）**：fetch 真实化（GET 端到端 + 二进制响应 body 真实字节）、setTimeout 真实延迟、MutationObserver（characterData / childList addedNodes 回填 / attributeFilter / subtree）与 IntersectionObserver / ResizeObserver 真实回调、getComputedStyle 动态 inline 覆盖 + 计算值序列化、classList 完整 DOMTokenList、HTMLCollection/NodeList item/namedItem、表单控件事件（input/textarea/select、focus/blur/change、Tab 焦点导航）、Selectors L4、DOM 遍历/变异 API（querySelector/closest/dataset/cloneNode/insertAdjacentHTML/prepend/before/after/replaceWith/createDocumentFragment/innerHTML childList emission）、布局几何（offsetWidth/getBoundingClientRect/scroll 尺寸）。
- **P1b V8 原生 DOM 绑定（R3095–R3126 系列）**：native DOM bindings 替换 polyfill 字符串桥——S0 PoC 验证（internal-slot 值传递 + weak-handle GC，native 比 polyfill 快 ~15.6×）、S1 原生只读属性族 + NodeId↔对象身份映射（kill-switch 默认关）、S2 生产接线 + 树写/属性写原生（createElement/appendChild/insertBefore/removeChild/setAttribute）、live Document 共享（`Rc<RefCell<Document>>`，原生写触发重渲染）、S3 查询原生（querySelector/querySelectorAll 全选择器引擎）、S4 EventTarget 原生 + host→page 原生事件派发/对象丰富化/dispatchEvent 冒泡/stopPropagation、节点导航/childNodes/nodeValue/textContent/cloneNode/contains、attributes NamedNodeMap + 完整 Attr 节点、innerHTML/outerHTML getter/setter 原生；RFC §3.2 dom_bindings 五子模块化闭合。
- **模块与 Worker 完整化（R3087–R3094）**：动态 `import()` 外部模块、transitive module 递归 fetch、外部 Worker fetch（`__zw_fetch_script`）、inline DedicatedWorker 真实消息往返、外部 script 源码 fetcher。
- **DOM API 补充**：JS 跨文档导航（R3058）、`Element.checkVisibility`（R3074）、boolean 反射属性 set-false 修复 + `_REFLECTED_BOOL` 扩展（R3039/R3040）。
- **Canvas**：gradient / Pattern 逐像素光栅化（`CanvasStyle::sample_at`，linear/radial/conic，R3079/R3085）。
- **多进程与自动化**：图像解码独立进程 `apps/image-decoder`（D1，隔离编解码器漏洞）、合成器进程 `apps/compositor`（C2，protocol 消息族 + 真实光栅化）、WebDriver 服务 `apps/webdriver`（W3C 协议骨架，wdspec 第一步）。
- **产品版本号**：`crates/product-version` 从构建日期推导版本。
- **渲染兼容性度量**：导入上游真实 WPT reftest（约 9967 个）、`make reftest-oracle` Chromium Oracle 像素一致率（诚实通过率）、`make product-smoke` / `make product-smoke-legacy` 产品回归门禁、`make import-wpt` 测试资产化流程。
- **性能预算体系**：`make bench-gate` / `make bench-capture`（测量 + 门禁比较 + 趋势，perf-gate）。
- **工程**：ci-watchdog 夜间 CI 任务、QuickJS 矩阵纳入 `make test`（v8/quickjs 接口一致性门禁）、QuickJS 后端完整化（Sandbox trait 抽象，aarch64 release 以 QuickJS 替代 V8）。

### 变更

- 工作区从 20 个 member 扩展到 27 个（新增 `renderer` / `page-runtime` / `product-version` / `image-decoder` / `compositor` / `webdriver` / `icon-gen`）。
- 测试规模从约 1,000 增长到约 14,281（v8 feature 全量全绿，R3126 记录）。
- 跨平台打包：macOS `.app` bundle + zip、Linux AppImage / .deb、Windows .zip / NSIS 安装器，配套 CI release 工作流（`v*` tag 触发）；release zip 按平台分装、macOS 打包版本号按构建日期推导。
- 渲染兼容性：`freetype-raster` feature 默认开启（FreeType 替代 fontdue 光栅化，broad 一致率显著提升）；WPT reftest 对齐 Chromium Oracle（self-source 约 77%、oracle 真一致约 47.5%）；2026-08-09 字体栈重建 RFC v0.2.3 获批，恢复主动实施。
- 性能：停止页面加载后 CPU 空转、合并渲染进程 pending frames；`render_with_dom_mutations`（JS DOM 变更直接应用到 live doc，跳过 HTML round-trip）+ 增量样式/布局/绘制；`Arc<LayoutBox>` 消除整树深拷贝、cascade 借用去 3 次 String 分配、tokenizer 字节流扫描、stylesheets/query-doc 解析缓存、glyph cache 免 O(n) 驱逐、跨帧 advance 缓存、DOM 遍历合并。
- CI：benchmarks job 授予 `contents: write`（perf 基线/趋势回写 main，修复 403）。

### 文档更新

- 重写根 `README.md`，补齐开源协作入口、贡献说明和风险提示。
- 新增面向贡献者的文档：`CONTRIBUTING.md`、`CODE_OF_CONDUCT.md`、`SECURITY.md`、`docs/architecture.md`。
- 在 `.github/` 下新增 issue forms 和 pull request 模板。
- 在关键文档中统一强调：项目仍是实验性代码库，默认不应视为可直接用于生产环境。

## [0.1.0-alpha.0] - 2026-05-30

这是 ZeroWeb 工作区的首个公开预发布版本。

### 新增

- 建立面向浏览器内核的 Cargo workspace，拆出 DOM、CSS 解析、样式计算、布局、渲染、宿主运行时、网络、存储、IPC、WebView API、Canvas 2D 和 WASM 沙箱等模块边界。
- 打通 `render-foundation` 与 `host-runtime` 的基础集成，并提供基于 `wgpu` 的 `apps/webview-demo` 演示程序。
- 新增 `dom` crate，集成 `html5ever` 并提供 DOM 树操作能力。
- 新增 `css-parser` crate，覆盖 tokenizer、parser、selector 和 CSS 值解析。
- 新增 `style-system` crate，覆盖 cascade、inheritance、computed values、selector matching 和 shorthand expansion。
- 新增 `layout-engine` crate，在 `taffy` 之上整合 block / flex / grid 布局能力。
- 新增 `engine` 渲染管线，覆盖 paint、dirty tracking 和 compositing 基础能力。
- 新增 `net` 与 `security` crate，覆盖 URL、导航历史、Cookie、同源检查、CORS 和 CSP 基础能力。
- 新增 `protocol` 与 `storage` crate，覆盖 IPC 消息、localStorage、sessionStorage 和 IndexedDB 基础能力。
- 新增 `canvas` crate，提供 Canvas 2D 图元和绘制能力。
- 新增 `webview` crate，作为可嵌入渲染和生命周期 API 的对外入口。
- 新增基于 `wasmi` 的 `wasm-sandbox` crate。
- 新增跨 crate 集成测试、criterion benchmarks 和 GitHub Actions CI。

### 变更

- 把工作区测试规模扩展到约 1,000 个测试。
- 持续增强 `style-system`，包括 selector matching、shorthand expansion 和长度值处理修正。
- 收紧项目文档，把许可证边界、AI 辅助贡献流程和实验性风险提示说明得更清楚。

### 已知限制

- `browser-shell` 仍以占位实现为主，距离可用浏览器产品还有明显距离。
- `script-sandbox` 仍是占位 crate，页面级 JavaScript 执行尚未完成。
- 真实站点兼容性、WPT 覆盖率和生产级加固仍在推进中。
- 本版本仅面向学习、研究和工程探索。任何商用或其他生产用途都需要自行评估风险。
