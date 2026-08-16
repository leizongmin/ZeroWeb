# Changelog

本文件记录项目对外可感知的重要变更。

写法参考 Keep a Changelog，不过这里的版本号要结合项目现状来看：仓库还在实验阶段，所有预发布版本都应该按实验性版本理解。

## [Unreleased]

### 变更

- **浏览器进程模型固定化**：`zero-browser` 不再支持 `--single-process` / `--multi-process` 或相应环境变量开关；tab 固定使用 `zero-renderer`，页面帧固定经 `zero-compositor`，栅格图像固定由 renderer 启动 `zero-image-decoder`。`ZeroWebView` 仍保持进程内嵌入实现。
- **浏览器二进制瘦身**：发布版 `zero-browser` 不再编译进程内 `tab_worker`、页面脚本 worker 和页面脚本调度，也不再链接 `zero-webview`、V8、QuickJS 或脚本 sandbox；headless 的导航、内联 HTML、脚本执行及截图改由 `zero-renderer` IPC 驱动，browser 只保留宿主、网络代理、绘制快照导入和 chrome 呈现职责。默认 Windows release 从 71.4 MB 降至 18.5 MB；renderer 继续独占 V8 页面脚本执行。

### 新增

- **页面 JavaScript 运行时**：`script-sandbox` 提供 V8（`v8` crate，rusty_v8 更名）/ QuickJS 双后端 feature gate（V8 持久化 Context 复用；QuickJS 全矩阵 parity，136 个失败清零）、Web Worker、ES Modules、WebAssembly JS API 到 `wasm-sandbox` 的自动桥接。
- **P1a DOM/JS Bridge 原生化（R30xx 系列，已主体落地）**：fetch 真实化（GET 端到端 + 二进制响应 body 真实字节）、setTimeout 真实延迟、MutationObserver（characterData / childList addedNodes 回填 / attributeFilter / subtree）与 IntersectionObserver / ResizeObserver 真实回调、getComputedStyle 动态 inline 覆盖 + 计算值序列化、classList 完整 DOMTokenList、HTMLCollection/NodeList item/namedItem、表单控件事件（input/textarea/select、focus/blur/change、Tab 焦点导航）、Selectors L4、DOM 遍历/变异 API（querySelector/closest/dataset/cloneNode/insertAdjacentHTML/prepend/before/after/replaceWith/createDocumentFragment/innerHTML childList emission）、布局几何（offsetWidth/getBoundingClientRect/scroll 尺寸）。
- **P1b V8 原生 DOM 绑定（R3095 起持续落地）**：native DOM bindings 替换 polyfill 字符串桥——S0 PoC 验证（internal-slot 值传递 + weak-handle GC，native 比 polyfill 快 ~15.6×）、S1 原生只读属性族 + NodeId↔对象身份映射（kill-switch 默认关）、S2 生产接线 + 树写/属性写原生（createElement/appendChild/insertBefore/removeChild/setAttribute）、live Document 共享（`Rc<RefCell<Document>>`，原生写触发重渲染）、S3 查询原生（querySelector/querySelectorAll 全选择器引擎）、S4 EventTarget 原生 + host→page 原生事件派发/对象丰富化/dispatchEvent 冒泡/stopPropagation、节点导航/childNodes/nodeValue/textContent/cloneNode/contains、attributes NamedNodeMap + 完整 Attr 节点、innerHTML/outerHTML getter/setter 原生；RFC §3.2 dom_bindings 五子模块化闭合。后续切片：document.referrer 原生（R3176）、MutationRecord.attributeName 有效名（R3175）、SVG/MathML 命名空间在 innerHTML/outerHTML setter 保留（R3181）、fragment 解析 context 元素（R3182）、textContent/innerHTML/outerHTML setter null→empty（R3184）。**S5 customElements/Web Components 里程碑完成（R3262–R3269）**：TBD-1 class 继承验证（R3262）→ S5a native HTMLElement 基类 + class-extends → S5b customElements.upgrade/createElement('my-el') 原生实例 → S5c connected/disconnected 生命周期原生路径 → S5d attributeChangedCallback + HTMLElement Element 接口 → S5e 子树 upgrade（R3269）→ upgrade 时 observed attrs 回调（R3274）。
- **DOM/CSS 选择器一致化（R3277–R3284）**：`:disabled`/`:enabled` disabled-state 传播、`:required`/`:optional`/`:read-only`/`:read-write`、`:placeholder-shown`/`:default`/`:indeterminate`、`:any-link`/`:scope`/`:lang()`/`:dir()`、`:nth-child(an+b of S)`、`:target`、`:valid`/`:invalid`/`:in-range`/`:out-of-range` 约束校验伪类——DOM `querySelector`/CSS matcher 双路径一致化；form_state.rs 子模块抽取（R3280）。
- **文档 API 补充（R3254–R3261）**：window resize 事件派发、matchMedia change 事件、console 桥接 host tracing、TreeWalker 层级方法、document.currentScript、document.scrollingElement、innerText（textContent 近似）、document.designMode；CSSStyleSheet addRule/removeRule legacy 别名（R3276）。
- **模块与 Worker 完整化（R3087–R3094）**：动态 `import()` 外部模块、transitive module 递归 fetch、外部 Worker fetch（`__zw_fetch_script`）、inline DedicatedWorker 真实消息往返、外部 script 源码 fetcher。
- **DOM API 补充**：JS 跨文档导航（R3058）、`Element.checkVisibility`（R3074）、boolean 反射属性 set-false 修复 + `_REFLECTED_BOOL` 扩展（R3039/R3040）。
- **反射属性与序列化 spec 合规（R3185–R3208）**：reflected string attrs null→empty（id/title/lang/accessKey）、enumerated 反射 spec 化（dir/contentEditable/draggable/input.type 等）、toggleAttribute/getAttribute 返回语义、inline style latest-wins（handle 与 parsed 两路径）、cloneNode/outerHTML/CE old-value latest-wins、SVG/外部命名空间属性前缀序列化保留、insertAdjacentHTML position ASCII 大小写不敏感、outerHTML setter Document parent 守卫。
- **Canvas**：gradient / Pattern 逐像素光栅化（`CanvasStyle::sample_at`，linear/radial/conic，R3079/R3085）。
- **多进程与自动化**：图像解码独立进程 `apps/image-decoder`（D1，隔离编解码器漏洞）、WebDriver 服务 `apps/webdriver`（W3C 协议骨架，wdspec 第一步）。
- **合成器进程（C2）RFC v2.1 五切片全部落地**：scroll transform bake、sync_token + Viz present、GPU mailbox fence + mmap 零拷贝、dma-buf fd 导出路径、owned window present surface、Linux landlock + seccomp（network/exec）沙箱、GPU device-lost 模拟 + CPU 回退、compositor crash E2E legacy 回退；Vulkan 真纹理 dma-buf 导出（跳过 read_pixels）仍为后续。
- **官网**：GitHub Pages 双语站点上线（[zeroweb.leizm.com](https://zeroweb.leizm.com)），含项目更新 feed（weekly cron 更新）。
- **字体（字体栈重建切片 R3230-F–R3344-F）**：OpenType features 贯通（font-face feature defaults、CSS feature precedence、shaping 携 feature）、generic layout advance 统一（fontdue/rustybuzz 对齐）、ordered fallback faces 全链路（shaping→IFC→layout→paint）、shaped fallback default-on（R3243-F）、**CSS Fonts 4 two-value `font-size-adjust` 全栈贯通（R3245-F，css-fonts Oracle 净改善 14.34pp）**、font-synthesis 属性（R3248-F）、font-size 绝对关键字 + HTML `<font size>` 映射（R3247-F）、font-variant 族（caps/numeric/east-asian/position + shorthand 展开与 font-stretch，R3250-F–R3253-F）、bidi override mirroring（R3319-F）、@font-face stretch/相对度量（R3341-F–R3344-F）；`unicode-segmentation`（pin =1.12.0）依赖加入。
- **HTML 行为兼容赛道（2026-08-12 启动）**：规范驱动并行开发线——表单兼容性基线 + 共享动作事务核心（form 动作/文本编辑/焦点经共享计划路由 renderer/tab worker/webview）、可取消文本输入事件、form POST 导航、无 JS 保留默认动作、稳定页面节点身份、焦点与 label 激活对齐；规格见 `docs/specs/html-behavior-compatibility-spec-rfc.md`，常驻断言见 `tests/integration/src/html_compat.rs`。
- **wgpu 24→30 升级**（R3275）：一次跨 6 个主版本（原 backlog 目标 29），compositor 真 dma-buf 前置；升级后全 workspace 门禁通过。
- **Canvas**：Path2D svgString 构造器 + SVG path data parser（R3307）、Path2D 复用 + `ctx.fill(path)`（R3306）、`createImageBitmap`（ImageData/HTMLCanvasElement 源 + source-crop 选项 + ImageBitmap drawImage，R3309–R3311）、OffscreenCanvas 主线程 + `transferToImageBitmap`（R3312）、`HTMLCanvasElement.transferControlToOffscreen`（R3313）、resize 重置 bitmap 与绘制状态（R3308）。
- **GPU 渲染**：draw_order 逐图元绘制修复 DC-10 分桶 z 序缺陷（R3277）、clip/blend 双 pass 源层混合（R3278）、窗口模式滤镜/变换离屏 ping-pong（R3279）、分桶路径 clip（R3284）、GPU 阴影模糊离屏画（R3287）、repeating 渐变首色标≠0（R3289）、inset 阴影 GPU + box blur 逐像素对齐 CPU（R3290/R3291）。
- **DOM/CSS 伪类与文档 API**：`:defined`/`:blank`/`:fullscreen`/`:modal`/`:focus`/`:focus-visible`/`:focus-within` DOM/CSS 一致化（R3299–R3302）、元素滚动 RFC S1/S2（ScrollEventParams cursor_x/y IPC，R3298）、DOMRect/DOMRectReadOnly 构造器（R3319）、navigator.serviceWorker 迁移 B-gen shim（R3318）、input.valueAsDate + stepUp/stepDown（R3317）、全局 ImageData 构造器（R3297）。
- **安全与存储修复**：CSP source-expr host 匹配 + mixed-content scheme 大小写（R3342/R3343，真安全绕过修复）、Web Storage auto-inc generator 显式数值 key 修复（R3341）。
- **zero-psl crate**（R3380）：公共后缀列表（PSL）解析与注册域名（eTLD+1）提取，接入 site-isolation；工作区 27→28 member。
- **Canvas WPT 驱动兼容性批量修复（R34xx）**：line-styles 33/33（真 join/cap/miter 几何 + setter 校验）、shadows 50/61（形状 alpha、可见区裁剪、真 join/cap）、compositing（source-out/Clear ops、uncovered-clear、composite enum）、gradient 语义（CTM 变换坐标、零长度、nonfinite、stop 分组插值）、pattern（验证/setTransform/图像错误/零尺寸）、text（fonts.ready settle、align/baseline、setter 校验）、roundRect 角对半径、CSS Color 4 getter 保留 `color()` 形式、clip draw_shadow_path 可见区裁剪。
- **js-dom M4**：native `createProcessingInstruction` + DOMException 身份一致性（polyfill/native 双路径对齐，classList/createElement 校验异常）、testharness-dom WPT 基线（DC-3，testharness local .js inline + baseline truthing）。
- **字体与 net**：WOFF2 webfont 安全解码（R3375-F）；net 修复（R3339 redirect 测试服务器完整请求头读取、R3367 dotdot Windows `to_file_path` 语义）。
- **浏览器打包**：compositor 随 launch/release 构建发布（`make browser` 与发布产物含合成器）、production chrome parity 证据集。
- **js-dom M4（原生/聚 polyfill DOM 对齐，R38–R49）**：MutationObserver 记录语义完整化（childList fragment 展开、NS 记录、no-mutation 守卫、classList parity、批量 id 重命名链、attributeOldValue 预捕获、observe options 校验、CharacterData 编辑 + SetChildText characterData 记录）、HTMLCollection legacy 索引/命名属性语义 + `Element.children` 返回 HTMLCollection、Range/StaticRange/Attr 端点 API、TreeWalker/NodeIterator 遍历 API、document/window slot-tagged 联合派发链、pre-set stopPropagation 标志跳过全部派发监听。
- **Canvas WPT 批量修复（R34xx 续）**：G7 聚类全灭（variationSelectors / ctor.basics / index-from-offset 边角）、`ctx.lang`、WPT 验证证据 662 Pass / 89.28% 覆盖率。
- **net resource loader 重构**：guarded DNS prefetch、异步 HTTP 连接预暖、资源请求/结果桥队列上限、动态模块 fetch 集中调度、HTTP2 优先级与协议遥测、截断流式响应测试覆盖。
- **字体栈续（R3422-F–R3426-F）**：可变轴贯穿光栅 IPC（R3422-F）、author face 布局 advance 对齐（R3424-F）、layout overrides Rc 共享 + 组合 memo（修 R3424-F 默认开启后 layout 10× 回归）、generic CJK contiguous 保留 opt-in（R3426-F）。
- **产品版本号**：`crates/product-version` 从构建日期推导版本。
- **渲染兼容性度量**：导入上游真实 WPT reftest（约 9967 个）、`make reftest-oracle` Chromium Oracle 像素一致率（诚实通过率）、`make product-smoke` / `make product-smoke-legacy` 产品回归门禁、`make import-wpt` 测试资产化流程。
- **性能预算体系**：`make bench-gate` / `make bench-capture`（测量 + 门禁比较 + 趋势，perf-gate）。
- **工程**：ci-watchdog 夜间 CI 任务、QuickJS 矩阵纳入 `make test`（v8/quickjs 接口一致性门禁）、QuickJS 后端完整化（Sandbox trait 抽象，aarch64 release 以 QuickJS 替代 V8）。
- **运行时配置集中化**：新增 `zero-runtime-config` crate（工作区 28→29 member）——`ENVIRONMENT_VARIABLES` 权威清单 + 统一解析函数（`enabled_when_true` / `enabled_by_default` / `enabled_unless_zero` / `positive_usize` 等），业务 crate 不再直接读环境变量。
- **Canvas 色彩管理**：f16 浮点像素存储 + linear 色彩空间（color-type 4/4）、display-p3 色彩管理（wide-gamut 9→12/12）——色彩空间经 GPU 管线贯通。
- **Canvas 滤镜与 GIF**：colorMatrix 滤镜渲染（filters 目录 13/13 全绿）、GIF 首帧解码（drawImage.animated.gif + pattern.animated.gif 全过）。
- **Canvas 路径几何（R34xx + js-dom M8 path-objects）**：arcTo 真切线弧（含无子路径 moveTo、负半径 IndexSizeError）、arc 归一化全圆/幅度语义 + anticlockwise 整圆特例、isPointInPath/fill 的 nonzero 绕组 + fillRule/Path2D 形式、ensuresubpath 语义 + clip 相交/空交集、椭圆负半径校验、曲线/弧段数自适应 + stroke 像素方形覆盖、stroke 零长段剪除 + roundRect 显式闭合 + 环绕 join、worker OffscreenCanvas 接口面——path-objects 用例 62F → 3F。
- **布局与文本测量（ZRG-2026-08-15）**：布局文本宽度改 hmtx 真实测量（替换 estimate 启发式）、paint 字符 advance 按字形实际字体测量（engine/browser 双端）、FreeType measure_advance hinting 取整致英文文本字距错乱修复——跨平台换行点一致（CI watchdog 同步修 shaping 基线跨平台字体 + net async redirect deflake）。
- **多进程与渲染**：renderer JS 状态跨文档加载隔离（lexical state 测试覆盖）、html5test.co 分数在多进程浏览器中正确渲染、compositor scrolling viewport repaint 修复、browser 构建缓存跨启动保留。

### 变更

- 工作区从 20 个 member 扩展到 29 个（新增 `renderer` / `page-runtime` / `product-version` / `psl` / `image-decoder` / `compositor` / `webdriver` / `icon-gen` / `runtime-config`）。
- 测试规模从约 1,000 增长到约 14,281（v8 feature 全量全绿，R3126 记录）。
- 跨平台打包：macOS `.app` bundle + zip、Linux AppImage / .deb、Windows .zip / NSIS 安装器，配套 CI release 工作流（`v*` tag 触发）；release zip 按平台分装、macOS 打包版本号按构建日期推导。
- 渲染兼容性：`freetype-raster` feature 默认开启（FreeType 替代 fontdue 光栅化，broad 一致率显著提升）；WPT reftest 对齐 Chromium Oracle（self-source 约 77%、oracle 真一致约 47.5%）；2026-08-09 字体栈重建 RFC v0.2.3 获批，恢复主动实施。
- 性能：停止页面加载后 CPU 空转、合并渲染进程 pending frames；`render_with_dom_mutations`（JS DOM 变更直接应用到 live doc，跳过 HTML round-trip）+ 增量样式/布局/绘制；`Arc<LayoutBox>` 消除整树深拷贝、cascade 借用去 3 次 String 分配、tokenizer 字节流扫描、stylesheets/query-doc 解析缓存、glyph cache 免 O(n) 驱逐、跨帧 advance 缓存、DOM 遍历合并。
- CI：benchmarks job 授予 `contents: write`（perf 基线/趋势回写 main，修复 403）；Lato-Medium.ttf 纳入 git-tracked fonts 目录（include_bytes 打包修复）。

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
