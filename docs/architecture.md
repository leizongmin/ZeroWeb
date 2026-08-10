# ZeroWeb 架构总览

第一次进这个仓库，通常会先问三件事：

1. 这个仓库现在到底实现到了哪里
2. 各个 crate 分别负责什么
3. 应该从哪里开始阅读和修改

更正式的需求和约束在 [docs/specs/zero-web-spec-rfc.md](specs/zero-web-spec-rfc.md)，当前实现进度看 [docs/goal/zero-web/master.md](goal/zero-web/master.md)，渲染兼容性的执行进展看 [docs/goal/rendering-compat.md](goal/rendering-compat.md)。

> **注意**
> 这份文档说的是当前结构和目标方向，不代表项目已经到了可日常使用、商用或其他生产可用的程度。真要这么用，风险得自己评估。

## 这个仓库现在想收敛到哪里

- 构建一个可嵌入的 Rust `ZeroWebView` 库
- 构建一个完整浏览器应用
- 在开源协作下保持核心代码与演进节奏可控
- 让主线依赖尽量保持宽松许可证边界
- 用自动化测试、基准和文档约束 AI-first 开发流程

## 工作区分层

整个工作区共 27 个 workspace member：18 个库 crate、6 个应用入口（`apps/`）、2 个测试工具（`tests/`）和 1 个开发工具（`tools/icon-gen`，不随发布产物分发）。下文按「应用与进程入口 / 产品层和 API 层 / 引擎层 / 基础设施层 / 测试基础设施」分组列出。

### 应用与进程入口

| Path | 作用 |
|------|------|
| `apps/browser` | 桌面浏览器应用入口，基于 `browser-shell` + `webview` + `host-runtime`，提供窗口模式与 `--headless` / remote debugging（WebSocket）入口 |
| `apps/renderer` | 独立渲染进程入口（`zero-renderer`），负责多进程 IPC 下的页面渲染与脚本执行，通过 stdin/stdout 管道与浏览器主进程通信 |
| `apps/image-decoder` | 图像解码进程（`zero-image-decoder`，D1）：PNG/JPEG/WebP 在独立进程解码（隔离编解码器漏洞），由渲染进程内 webview 经管道 spawn；env `ZW_IMAGE_DECODER_PROCESS=1` 启用，未启用/SVG/降级路径回退进程内解码 |
| `apps/compositor` | 合成器进程（`zero-compositor`，C2）：RFC v2.1 五切片全部落地——scroll transform bake、sync_token + Viz present、GPU mailbox fence + mmap 零拷贝、dma-buf fd 导出、owned window present surface、Linux landlock/seccomp 沙箱、GPU device-lost 模拟 + CPU 回退、crash E2E legacy 回退；Vulkan 真纹理 dma-buf 导出仍为后续 |
| `apps/webdriver` | WebDriver 服务（`zero-webdriver`）：W3C 协议骨架（wdspec 第一步） |
| `apps/webview-demo` | 最小演示程序，用于串起宿主窗口和渲染基础设施（wgpu/CPU 渲染静态文本） |

### 产品层和 API 层

| Crate | 作用 |
|-------|------|
| `crates/webview` | 对外暴露稳定嵌入 API，屏蔽底层渲染细节；承载导航加载、脚本执行、Service Worker、Web Worker、WASM 自动桥接与 `SecurityContext` 安全检查等页面级能力 |
| `crates/browser-shell` | 浏览器产品层 UI-agnostic 数据模型：标签页（含导航历史、拖拽排序）、书签 / 文件夹、历史记录、下载、设置、地址栏自动补全、右键上下文菜单。它不直接画 UI，由 `apps/browser` 渲染 |

### 引擎层

| Crate | 作用 |
|-------|------|
| `crates/dom` | DOM 树、html5ever 集成、节点和文档模型、查询 / 序列化、事件、Shadow DOM、Range / TreeWalker、`FocusManager` |
| `crates/css-parser` | CSS tokenizer、parser、选择器和值解析、@规则（`@media` / `@container` / `@supports` / `@layer`）、`:has()`、calc/min/max/clamp、数学函数与命名颜色 |
| `crates/style-system` | 级联、继承、计算值、DOM 样式匹配、简写展开、`@media` / `@container` 评估、Transform / Transition / Animation、逻辑属性、`var()` 解析 |
| `crates/layout-engine` | 布局整合层（基于 taffy，Block / Flex / Grid / Position），把样式转换成布局树和几何输出，含 inline formatting（text-align / indent / float 排除 / tab-size / word-break 等） |
| `crates/engine` | 渲染主管线，负责串起解析、样式、布局、paint、dirty tracking、compositing；包含 DOM Bridge（V8 polyfill）、资源预加载、CSS 动画 / 过渡运行时 |
| `crates/canvas` | Canvas 2D 绘制能力（路径、变换、drawImage、shadow、Path2D、合成等） |

### 基础设施层

| Crate | 作用 |
|-------|------|
| `crates/render-foundation` | GPU/CPU 渲染、字体栈（fontdue + `freetype-raster` feature default-on：非 Ahem 路径用 FreeType 光栅化，提升与 Chromium 字体度量一致性；R1094 实测全 corpus oracle +232 零回归，是 broad 一致率显著提升的关键）、图片缓存 + GC、裁剪 / scissor、图元基础设施 |
| `crates/host-runtime` | 平台窗口、事件循环、surface 生命周期、输入事件（鼠标 / 键盘 / 触摸 / IME） |
| `crates/net` | HTTP/HTTPS、URL、导航历史、Cookie、WebSocket（tungstenite）、HTTP 响应缓存 |
| `crates/security` | 同源策略、CORS、CSP（含 `script-src-attr` / `unsafe-eval` / `wasm-unsafe-eval` / `strict-dynamic` 等完整指令）、HSTS 预加载、混合内容阻止 / 升级、权限模型、站点隔离、COOP/COEP，统一收敛到 `SecurityContext` 门面 |
| `crates/storage` | localStorage、sessionStorage、IndexedDB（KeyRange / Index / Cursor / Transaction）、Cache API、Service Worker 注册表 |
| `crates/protocol` | IPC 消息、bincode 序列化、`PipeTransport` 帧协议、`SharedMemoryChannel`、`RendererHandle` / `ProcessManager`（多渲染进程管理与崩溃检测） |
| `crates/product-version` | 产品版本号（从构建日期推导，随 `zero-product-version` 分发） |
| `crates/wasm-sandbox` | 受控 WASM 执行环境（wasmi），host function 导入、fuel / 执行限制、错误传播 |
| `crates/script-sandbox` | 页面脚本 / 扩展脚本运行时：V8（`v8` crate，原 rusty_v8）/ QuickJS feature gate，含 Isolate / Context 管理、持久化 Context 复用、Dedicated Worker、ES Modules、错误处理与超时 |
| `crates/page-runtime` | WPT / TabWorker / `zero-renderer` 三条页面路径共享的页面运行时契约（`PageLoadHost` / `AsyncFetchHost` / `BlockingFetchHost` 等），让 in-process（webview）和 IPC（renderer）两种宿主复用同一套分阶段页面加载逻辑 |

> 另有 `crates/taffy-local`，它是 taffy 0.12 的本地 `[patch.crates-io]` 补丁（仓库已从 vendored 0.7.7 升级到 0.12.1，本地补丁仍补充 `cached_baselines()` 等访问器），不是普通业务 crate。

### 测试基础设施

| Path | 作用 |
|------|------|
| `tests/integration` | 跨 crate 集成测试（DOM Bridge polyfill、多进程架构、安全管线、真实网站兼容性、产品层 smoke 等） |
| `tests/wpt-runner` | Web Platform Tests / reftest / 兼容性基础设施（按分类通过率报告、Chromium Oracle 像素对比）；reftest harness 会执行测试页 setup 脚本（DOM 变更、`requestAnimationFrame` / `takeScreenshot` / `getBoundingClientRect` 等）后再截图对比，覆盖靠脚本构造内容的用例；`product-smoke --struct-check` 另提供结构性回归门（兄弟盒重叠检测 + `--expect-class` 元素计数 + `--expect-lines` 行数断言），与像素 diff 门互补 |
| `tests/benchmarks` | benchmark 结果产物（不是 workspace member） |

> 另有 `tools/icon-gen`（`zero-icon-gen`），它是开发工具而非业务 crate：从源 SVG 生成 Linux / Windows / macOS 三端图标资产（PNG / ICO / macOS iconset / 运行时窗口 RGBA），不随发布产物分发。

## 从请求到像素的大致链路

现在主线上的数据流，大致可以这么看：

1. `net` 负责 URL、导航和资源获取（含 WebSocket 与响应缓存）。
2. `dom` 基于 `html5ever` 把 HTML 解析为 DOM 树。
3. `css-parser` 解析样式规则。
4. `style-system` 把选择器和规则匹配到 DOM 节点，生成计算样式。
5. `layout-engine` 把计算样式转换为布局树和几何信息。
6. `engine` 把布局结果转换为绘制命令、合成层与脚本桥接。
7. `render-foundation` 把图元输出到 GPU/CPU 渲染后端。
8. `host-runtime` 管理窗口和 surface，把帧显示到平台宿主。
9. `webview` 把这条链路包装成嵌入式 API，供 `apps/browser` 或第三方应用调用。
10. 多进程形态下，`apps/renderer` 作为独立渲染进程承担步骤 2–7，通过 `protocol` IPC 与浏览器主进程交互；`page-runtime` 让这条加载链路在「进程内」和「IPC」两种宿主下走同一套契约。

这条链路已经能在测试、demo 和浏览器应用里跑起来，并且有大量单元 / 集成测试与 WPT 用例兜底；但离「真实网页 + 完整 JavaScript + 完整浏览器 UI」的成熟度仍有距离。当前主线是 P1b V8 原生 DOM 绑定（P1a DOM/JS Bridge 原生化已主体落地），渲染兼容性（reftest 对齐 Chromium）自 2026-08-09 字体栈重建获批后恢复主动实施。

## 现在做到哪了

粗略说，仓库现在分成三档：

- **核心内核已有实质实现**: dom、css-parser、style-system、layout-engine、engine、render-foundation、host-runtime、net、security、storage、protocol、canvas、wasm-sandbox、script-sandbox、page-runtime、product-version、webview 都有可运行代码和对应测试。
- **产品层骨架已成，持续打磨**: `apps/browser`（桌面入口 + headless / remote debugging）、`browser-shell`（标签页 / 书签 / 历史 / 下载 / 设置 / 上下文菜单等数据模型）、`apps/renderer`（多进程渲染进程入口）、`apps/image-decoder`（D1 图像解码进程）、`apps/compositor`（C2 合成器进程）、`apps/webdriver`（WebDriver 服务）已打通，但产品形态、稳定性和真实站点兼容性仍在推进。
- **当前主线**: P1b V8 原生 DOM 绑定（2026-08-09 RFC 获批，R3095 起持续落地：S0 PoC 验证 native ~15.6×、S1 原生只读属性族 + NodeId 映射、S2 生产接线 + 树写/属性写原生、live Document 共享、S3 查询原生、S4 EventTarget 与事件派发/冒泡/stopPropagation 原生化，续以命名空间/序列化 spec 合规 R3181–R3208；P1a DOM/JS Bridge 原生化已主体落地）；渲染兼容性（WPT/CSSWG reftest 对齐 Chromium Oracle）2026-08-04 起降频守成、2026-08-09 字体栈重建 RFC v0.2.3 获批后恢复主动实施（首片 gated shaped paint R3209 已 default-off 落地）——Chromium Oracle 真一致约 47.5%、self-source 约 77%、strict 处低位 plateau，自主 clean-lever 轻量修复面已 11 vein 审计穷尽；残余缺口为 vertical writing modes（部分切片已落地，整体仍 user-gated）、multicol 碎片化、R109 inline-as-block 等结构性问题，根因是 layout↔paint IFC 度量不一致（Phase-A spread），Phase A IFC / R1043 / R2174 等深方向仍需用户点名授权。完整 Web API 与真实网站交互兼容性是后续阶段。详见 [路线图](../ROADMAP.md)。

所以今天的 ZeroWeb 是一个内核已成形、产品层在打磨的浏览器工作区，但还不是一个做完的浏览器产品。

## 写代码时最该记住的约束

编码准则详见 [CLAUDE.md](../CLAUDE.md)。架构层面的关键约束：

- **许可证边界**: 核心路径优先 MIT、Apache-2.0、BSD 等宽松许可证依赖
- **架构边界**: `webview` 是稳定嵌入边界；`engine` 负责管线串联（不混入渲染图元）；`render-foundation` 负责 GPU/CPU 图元输出（不混入布局/样式逻辑）；`protocol` + `apps/renderer` 定义多进程契约
- **测试入口**: 使用 `make test` / `make reftest`（经 `scripts/test-guard.rs` OOM 包裹），不要裸跑 `cargo test`；渲染变更额外跑 `make product-smoke` / `make product-smoke-legacy`
- **诚实度量**: 渲染兼容性以 `make reftest-oracle`（Chromium Oracle 像素对比）为诚实通过率，同源 reftest 存在假通过仅作自一致性参考

## 建议的阅读顺序

第一次进仓库，按这个顺序读，通常最省时间：

1. [README.md](../README.md)
2. 本文档
3. [docs/specs/zero-web-spec-rfc.md](specs/zero-web-spec-rfc.md)
4. [docs/goal/zero-web/master.md](goal/zero-web/master.md)
5. 目标 crate 的 `README.md`
6. 对应 crate 的 `src/lib.rs` 和测试文件

## 比较好的切入点

如果你想开始动手，可以先看这几类事情：

- 扩展现有单元测试和集成测试
- 补充 WPT runner 的覆盖面、修渲染 / 布局兼容性缺口（见 `docs/goal/rendering-compat.md`）
- 打磨 `browser-shell` 产品层与 `apps/browser` 的真实窗口 / GPU 验收
- 推进 `webview` 与真实导航链路、`page-runtime` 契约的衔接
- 修补样式、布局和渲染的兼容性缺口
