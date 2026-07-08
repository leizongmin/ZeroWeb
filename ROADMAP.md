# Roadmap

这是一份对外路线图，用来说明 ZeroWeb 大体已经做到哪里、现在在推什么、后面准备补什么。

它不是内部执行手册，也不是交付承诺。项目还在实验阶段，计划会随着实现难度、验证结果和依赖现实情况调整。

> [!IMPORTANT]
> ZeroWeb 目前主要面向学习、研究和工程探索。即使某个阶段显示为“已完成”，也不代表项目已经适合商用或其他生产用途。相关风险仍然需要自行评估。

## 怎么看这份路线图

| 状态 | 含义 |
|------|------|
| `✅ 已完成` | 这一阶段的基础能力已经落进仓库，并有对应测试或验证 |
| `🚧 进行中` | 已经开始推进，但还没有形成稳定可用的整体能力 |
| `⏳ 计划中` | 方向明确，但还没进入完整实现阶段 |
| `❄️ 暂不优先` | 方向存在，但不是当前最优先的事情 |

## 当前重点

现在最值得继续往前推的几件事：

- 当前优先完成 `rendering-compat` / render-compact 验收，把静态 CSS-heavy 页面渲染拉到可验证的 Chromium 参考水平；通过率以 Chromium Oracle 像素对比（`make reftest-oracle`）为诚实度量，同源自渲染 reftest 仅作自一致性参考
- 外部样式表加载、图片子资源与 `ImageCache`、SVG 作为图片资源的栅格化、产品静态页截图门禁（`make product-smoke` / `make product-smoke-legacy`）均已落地；FreeType 字体光栅化（`freetype-raster` 默认开启）落地后 broad 一致率（chr<1%）从 ~36% 跃升至 ~51%（CSS2 65.6% / flexbox 60.0% / tables 64.3%，R1200-R1202 fresh baseline），但 strict 像素级仍受 font-raster 残余噪声约束、处 plateau；残余缺口集中在 vertical writing modes、multicol 碎片化、R109 inline-as-block、baseline-export（taffy 0.7 gate）等结构性方向
- 后续 forward lever 多为多会话架构级：Phase A IFC 统一（解 layout/paint dual-path）、taffy 0.7→0.11 迁移（unblock baseline-export + flex/grid intrinsic，build-breaking、worktree-gated、需单独决策，设计文档见 `docs/goal/rendering-compat/taffy-migration-design.md`）、2em layout dual-path；短期内仍持续修 CSS2 / Flexbox / Grid / Multicol 等可单点修复的渲染缺口
- 用真实静态页面暴露缺口：`apps/browser/assets/welcome.html`、`morning.work` 文章页、`wintertc.org` 首页
- 短期内把 HTML 3.2/4 + CSS1/2 常见静态文档（presentational hints、UA 默认样式、基础表格 / 列表 / 链接颜色）提升为高优先级推进面，配套 `legacy-html` 产品 smoke 与 `make product-smoke-legacy` 回归门禁；不替代 WPT / DC-14 长期目标
- `browser-shell` 已有基础骨架，后续重点是把产品层和 WebView/渲染管线的真实验收打通
- render-compact 验收通过后，再逐步推进完整 JS/DOM API、图形 API、SVG 文档、动画逐帧、真实网站交互兼容性和平台字体差异

## 路线图

| 阶段 | 主题 | 状态 | 说明 |
|------|------|------|------|
| M1 | 项目骨架与渲染基础设施 | `✅ 已完成` | 工作区、GPU/CPU 渲染基础、`webview-demo` 入口、CI 骨架 |
| M2 | HTML 解析与 DOM | `✅ 已完成` | `html5ever` 集成、DOM 树、基础查询与文档模型 |
| M3 | CSS 解析与样式系统 | `✅ 已完成` | tokenizer、parser、选择器、级联、继承、计算值、简写展开、`@media` 基础 |
| M4 | 布局引擎 | `✅ 已完成` | block / flex / grid 基础整合，几何验证就位 |
| M5 | 渲染管线集成 | `✅ 已完成` | paint、dirty tracking、compositing、渲染链路打通 |
| M6 | JavaScript 运行时与 DOM 绑定基础 | `✅ 已完成` | V8 / QuickJS feature gate、DOM bridge、事件基础、Web Worker、ES Modules、WASM bridge 等基础能力已落地；完整 Web API 兼容性放到 M13 |
| M7 | 网络、安全与导航基础 | `✅ 已完成` | HTTP、URL、导航历史、Cookie、同源策略、CORS、CSP 基础能力 |
| M8 | 协议与多进程基础 | `✅ 已完成` | IPC 消息、协议定义、序列化边界、renderer 入口和进程管理基础已经建立 |
| M9 | Canvas 与存储 | `✅ 已完成` | Canvas 2D、localStorage、sessionStorage、IndexedDB、Cache API、Service Worker registry 基础已在仓库中 |
| M10 | WebView API 与自动化基础 | `✅ 已完成` | 已有可嵌入 API、导航加载、测试和 headless/自动化相关基础，但还会继续演进 |
| M11 | 浏览器产品层 | `🚧 进行中` | `browser-shell`、标签页、地址栏、历史、书签、下载、设置等基础逐步落地；真实窗口/GPU/display 产品验收仍需补齐 |
| M12 | Render compatibility / render-compact | `🚧 进行中` | 当前主线；以 WPT/CSSWG reftest 对齐 Chromium，并以 Chromium Oracle 像素一致率（`make reftest-oracle`）为诚实度量（同源 reftest 存在假通过）；harness 已执行测试页 setup 脚本后再截图，覆盖靠脚本构造内容的用例；外部 CSS、图片资源、SVG 栅格化、产品静态页门禁已落地；FreeType 字体光栅化（`freetype-raster` 默认开启）落地后 broad 一致率（chr<1%）从 ~36% 跃升至 ~51%（R1200-R1202 fresh 10-dir baseline），但 strict 像素级仍受 font-raster 残余噪声约束、处 plateau；残余缺口为 vertical writing modes、multicol 碎片化、R109 inline-as-block、baseline-export（taffy 0.7 gate）等结构性问题，下一批 forward lever（Phase A IFC 统一、taffy 0.7→0.11 迁移、2em layout dual-path）多为多会话架构级 |
| M13 | 完整 JS/DOM API 兼容性 | `⏳ 计划中` | render-compact 验收后推进；目标是从基础 DOM bridge 扩展到更完整的 Web API、事件循环、DOM/CSSOM 操作和真实页面脚本行为 |
| M14 | Canvas / WebGL / WebGPU | `⏳ 计划中` | Canvas 2D 继续补全后，逐步进入 Khronos WebGL CTS 和 GPUWeb WebGPU CTS；不作为 render-compact 的阻塞项 |
| M15 | SVG 文档与内联 SVG DOM 渲染 | `⏳ 计划中` | render-compact 只要求 SVG 作为图片资源栅格化；完整 SVG 文档、内联 SVG DOM、样式和交互放到后续阶段 |
| M16 | CSS 动画逐帧一致性 | `⏳ 计划中` | render-compact 关注静态截图和必要的 CSS 视觉状态；动画/transition 的帧级时间轴、插值和截图一致性后续单独验收 |
| M17 | 真实网站完整交互行为 | `⏳ 计划中` | 在静态渲染稳定后，扩展到登录、表单、路由、滚动、输入、复杂脚本和长会话行为 |
| M18 | 平台字体像素级一致性 | `❄️ 暂不优先` | 不作为近期主线；长期再评估是否追近 Chromium/Safari/Firefox 在不同平台的字体 fallback、hinting、subpixel 和 emoji 细节 |

## 长期兼容性验收标准

长期路线图要对齐行业标准，但不同标准进入主线的时机不同。ZeroWeb 后续按下面这张矩阵逐步纳入验收：

| 覆盖面 | 行业标准 / 基准 | 对应阶段 | ZeroWeb 使用方式 |
|--------|------------------|----------|------------------|
| Web 平台一致性 | WPT: Web Platform Tests | M12-M17 | M12 先聚焦 reftest、CSS 子集和真实静态页；M13 以后扩大到 DOM、HTML、Fetch、URL、Storage、Service Worker、Web API 等 testharness 覆盖；M17 纳入更多交互类用例 |
| CSS 渲染 | CSSWG / WPT CSS tests | M12、M16 | M12 覆盖 CSS 2.1、Flexbox、Grid、Position、Display、Box、Float、Table、Multicol、Text、Fonts、Writing Modes、Text Decoration 等静态渲染；M16 扩展到 Animations、Transitions 和逐帧一致性 |
| JavaScript 语言 | Test262 | M13 | 用于验证 ECMAScript 和 ECMA-402 语言/Intl 行为，和 DOM/Web API 测试分开统计 |
| HTML 解析 | html5lib tests + WPT HTML parsing tests | M13 | 持续验证 tokenizer、tree construction、quirks/no-quirks、fragment parsing 与主流浏览器行为一致 |
| DOM / Web API | WPT testharness tests | M13-M17 | 覆盖 DOM、HTML、CSSOM、Events、URL、Encoding、Fetch、Streams、Storage、Clipboard、Fullscreen、Custom Elements、Shadow DOM 等 API |
| 浏览器自动化与交互 | WebDriver Classic / WebDriver BiDi wdspec | M17 | 用于验证导航、窗口、输入、点击、键盘、滚动、脚本执行和跨进程自动化行为 |
| Canvas 2D | WPT Canvas tests | M14 | 在现有 Canvas 2D 基础上补齐路径、文本、图像、像素、合成、变换等兼容性 |
| WebGL | Khronos WebGL Conformance Tests | M14 | WebGL 进入主线后作为主要一致性门禁 |
| WebGPU | GPUWeb WebGPU CTS | M14 | WebGPU 进入主线后作为主要一致性门禁 |
| WebAssembly | WebAssembly spec tests | M13-M14 | 验证 WASM 模块、实例化、导入导出、trap、数值语义和 JS/WASM 边界 |
| 可访问性 | ARIA-AT + WPT accessibility-related tests | M17 | 用于长期验证 ARIA、键盘交互、无障碍树和辅助技术互操作；不作为 render-compact 阻塞项 |
| 性能基准 | BrowserBench: Speedometer、JetStream、MotionMark | M17-M18 | 正确性稳定后再纳入趋势跟踪；不把性能分数当作早期功能完成标准 |
| 浏览器覆盖口径 | MDN Browser Compatibility Data / Baseline | M13-M18 | 作为功能覆盖矩阵和对外兼容性说明口径，不替代 WPT/CTS 的实际测试 |
| 行业互操作优先级 | Interop project / wpt.fyi | M12-M18 | 用来选择高价值兼容性领域、跟踪 WPT 结果和定位与主流浏览器差距 |

不会把 Acid2、Acid3 这类历史测试作为主要路线图目标。它们可以作为趣味性 smoke，但不能替代 WPT、Test262、CTS 和真实网站验收。

## 接下来大概率会先做什么

如果按当前仓库状态往下走，顺序大致会是：

1. 完成 render-compact 验收：WPT/CSSWG reftest、静态产品页、真实静态文章页、图片密集首页都要能和 Chromium 做稳定截图对比。
2. 收敛 ZeroBrowser glyph 后处理与 layout/paint/glyph 度量一致性（外部 stylesheet、图片子资源/`ImageCache`、SVG 作为图片资源的栅格化均已贯通；`freetype-raster` 默认开启已把 broad 一致率（chr<1%）拉到 ~51%，静态页当前差异主要来自行盒与字体度量的 strict 像素级残余噪声，而非资源加载）。
3. 统一 inline formatting、layout IFC 和 paint IFC 的权威结果，解决文本串联、重叠、标题误拆行和正文压缩。
4. 搭出 `browser-shell` 最小可用骨架，让浏览器产品层真正出现。
5. render-compact 验收后，把 Test262、WPT testharness、WebDriver wdspec、WebGL CTS、WebGPU CTS、WebAssembly spec tests 等行业测试按阶段接入。
6. 再逐步推进完整 JS/DOM API、Canvas/WebGL/WebGPU、SVG 文档、CSS 动画逐帧和真实网站完整交互行为。

这不是死板顺序。实际推进时，底层能力和产品层会交替往前推。

## 暂不放在当前优先级里的方向

这些事情不是不做，而是不是现在先做：

- 完整 DevTools
- 媒体播放（`<video>` / `<audio>`）
- WebRTC
- 浏览器扩展系统
- 首期移动端发布
- 完全复制 Chromium/Safari/Firefox 的平台字体像素差异

## 关联文档

- [README.md](README.md)
- [docs/goal/zero-web/master.md](docs/goal/zero-web/master.md)
- [docs/goal/rendering-compat.md](docs/goal/rendering-compat.md)
- [docs/goal/rendering-compat/master.md](docs/goal/rendering-compat/master.md)
- [docs/specs/zero-web-spec-rfc.md](docs/specs/zero-web-spec-rfc.md)
- [docs/architecture.md](docs/architecture.md)
