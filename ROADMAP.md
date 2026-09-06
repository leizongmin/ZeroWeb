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

- **页面 JavaScript / P1b V8 原生 DOM 绑定（已收官，2026-08-31）**: P1a DOM/JS Bridge 原生化主体落地 + P1b V8 原生 DOM 绑定 **M1–M5 全达成**（R383/R384：V8 与 QuickJS 双引擎 `native_dom` default-on land，kill-switch 已删除）+ 多进程 worker native bindings；js-dom 专项目标 DC-1~8 全达成并归档（详见 [docs/goal/archive/js-dom/master.md](docs/goal/archive/js-dom/master.md)）。完整 Web API 兼容性仍持续推进
- **Render compact（恢复主动实施）**: WPT/CSSWG reftest 对齐 Chromium Oracle（`make reftest-oracle` 为诚实度量），上游 WPT reftest corpus **14496/16814 = 86.2%**（R4095 轮口径）；inline SVG paint 默认放开（R3991，user 点名）后 svg transform/origin/stylesheet 级联、run-in 并入、SVG intrinsic/flex §9.9/table section/rtl abspos/contain:size + contain:inline-size/nbsp strut/backface-visibility/overflow:clip 单轴/stretch 关键字/SVG2 intrinsic sizing bbox 等系列持续收口（R3936–R4095）；Phase A IFC / R1043 vertical-mode / R2174 border-box 仍等用户点名。详见 [docs/goal/rendering-compat/master.md](docs/goal/rendering-compat/master.md)
- **Legacy HTML 与表单**: UA 默认样式与表单控件已落地（R2156/R2162 两切片 default-on），`make product-smoke-legacy`（42 个 legacy fixture vs chrome-127 oracle）作为趋势回归门禁
- **媒体播放（三 goal 已完成，2026-09-05 收口归档）**: `crates/media`（zero-media）已落地——webm/Matroska demux、VP9 纯 Rust 解码、AV1（`decode-av1` feature）、H.264 mp4（`decode-h264` feature，AAC 音频链 + 伴生轨 + precise-seek 随切片 2 落地，分发前须法务复核）、音频解码（mp3/ogg-vorbis/opus/webm 音轨 + AAC/wav）、`VideoPlayer` 播放驱动、renderer 播放泵事件循环节拍、混音总线与 Web Audio 最小面。三 goal：[media-playback](docs/goal/archive/media-playback/master.md)（解码选型 RFC 路线 C + 帧上屏/连续播放/多格式，DC-1~5 ✅）、[media-audio](docs/goal/archive/media-audio/master.md)（cpal 输出 + A/V 同步 + AudioContext 最小面 webaudio 50 用例 1418P/0F，DC-1~5 ✅）、[media-elements](docs/goal/archive/media-elements/master.md)（HTMLMediaElement 语义面 WPT 640P/0F/13PF = 98.01%，DC-1~4 ✅）。完整 `<video>`/`<audio>` 播放体验仍归 M14/后续
- **HTML 行为兼容（新赛道，2026-08-12 启动）**: 规范驱动的并行开发线——基础 HTML 元素解析、DOM/IDL、交互状态、事件与默认动作（不含 CSS 样式/布局精度/控件外观）；已建立表单兼容性基线 + 共享动作事务核心（form 动作/文本编辑/焦点共享计划、可取消文本输入事件、form POST 导航、稳定页面节点身份），见 [docs/specs/html-behavior-compatibility-spec-rfc.md](docs/specs/html-behavior-compatibility-spec-rfc.md) 与 [docs/research/research-html-compat-parallel-track-2026-08-12.md](docs/research/research-html-compat-parallel-track-2026-08-12.md)
- **HTML 编辑与键盘（keyboard/editing 三 goal 活跃）**: contenteditable/execCommand/Selection 编辑管线（WPT selection 45P→1824P = 70.2%、execCommand event 179P/1F、CE 键入/删除/Enter 换行落 DOM——editing M2 三切片完成）+ 键盘分发基线与页面滚动映射（keyboard 三 goal M1 切片推进中，2026-09-07）
- **M12/M14 剩余面拆分（2026-09-07 用户决策，新立 6 子 goal）**: [storage-opfs](docs/goal/storage-opfs.md)（OPFS 真实化）、[page-wasm](docs/goal/page-wasm.md)（页面 WASM 深化）、[android-browser](docs/goal/android-browser.md)（Android 可用化）、[webdriver](docs/goal/webdriver.md)（W3C 协议补齐）、[web-components](docs/goal/web-components.md)（Web Components 缺口补齐）、[event-loop-spec](docs/goal/event-loop-spec.md)（事件循环 spec 形态），均与 rendering-compat 声明 run-rules §9 碰撞边界
- **Browser shell 产品化**: 打通 `browser-shell` 与 WebView/渲染管线的真实验收

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
| M8 | 协议与多进程基础 | `✅ 已完成` | IPC 消息、协议定义、序列化边界、renderer 入口和进程管理基础已经建立；合成器进程（C2）RFC v2.1 五切片落地（dma-buf/mailbox fence 零拷贝、landlock/seccomp 沙箱、GPU device-lost CPU 回退） |
| M9 | Canvas 与存储 | `✅ 已完成` | Canvas 2D、localStorage、sessionStorage、IndexedDB、Cache API、Service Worker registry 基础已在仓库中；**IndexedDB 原生 Rust 路由落地（工厂 schema/事务 wire/object store/index/cursor/持久化，storage↔engine 双端接线）**，**[storage-indexeddb goal](docs/goal/archive/storage-indexeddb.md) 已完成（2026-08-19：WPT imported 168/210 文件 80.00%、1073/1073 Pass，含跨 renderer 连接/事务与持久化）**；Canvas WPT 兼容性批量修复持续（line-styles/shadows/compositing/gradient/pattern/text 等 R34xx 系列），**canvas-2d goal 已完成（R57 终态：Chromium Oracle 不一致归零 41/41 100%、Mission 中期 80% 达成、DC-1~4 全部满足）**；**表单验证（form-validation）M1-M3 完成（提交阻断全链路，WPT 919/0 全灭）**；**媒体解码管线起步（2026-09-01 起，[media-playback](docs/goal/archive/media-playback/master.md) / [media-audio](docs/goal/archive/media-audio/master.md) / [media-elements](docs/goal/archive/media-elements/master.md) 三 goal，2026-09-05 全部完成收口归档）**：`crates/media`（zero-media）落地——webm demux + VP9 解码 + YUV→RGBA、AV1（feature `decode-av1`）、H.264 mp4（feature `decode-h264`，D-RFC-3 获批，AAC 音频链/伴生轨/precise-seek 随切片 2）、音频解码（mp3/ogg-vorbis/opus/webm 音轨 + AAC/wav）、播放驱动与混音、Web Audio 最小面（webaudio 50 用例 1418P/0F）；media-elements 语义面 WPT 640P/0F/13PF = 98.01%。完整 `<video>`/`<audio>` 播放体验仍归 M14/后续 |
| M10 | WebView API 与自动化基础 | `✅ 已完成` | 已有可嵌入 API、导航加载、测试和 headless/自动化相关基础，但还会继续演进 |
| M11 | 浏览器产品层 | `🚧 进行中` | `browser-shell`、标签页、地址栏、历史、书签、下载、设置等基础逐步落地；真实窗口/GPU/display 产品验收仍需补齐 |
| M12 | Render compatibility / render-compact | `🚧 进行中` | 2026-08-04 起降频守成、2026-08-09 字体栈重建获批后恢复主动实施；WPT/CSSWG reftest 对齐 Chromium Oracle；**inline SVG paint 默认放开（R3991，user 点名）+ svg transform/origin/stylesheet 级联 + run-in 并入 + SVG intrinsic/flex §9.9/table section/rtl abspos/contain:size + contain:inline-size/nbsp strut/backface-visibility/stretch 关键字/SVG2 intrinsic sizing 系列（R3936–R4095，corpus 14496/16814 = 86.2%）**，详见「当前重点」 |
| M13 | 完整 JS/DOM API 兼容性 | `🚧 进行中` | **js-dom 专项目标已收官归档（2026-08-31，R391，DC-1~8 全达成，[docs/goal/archive/js-dom.md](docs/goal/archive/js-dom.md)）**：P1b V8 原生 DOM 绑定 **M1-M5 达成**（R383/R384：V8 `native_dom` default-on land + kill-switch 删除；QuickJS 同款 default-on，M7）+ 多进程 worker native bindings（R386）+ `make test` 18510P/0F 全绿（R389）；**M3 框架端到端闭环（R97-R99 lit + R100 Vue 3 + R339 DC-2 QuickJS parity）**，M4 大收口（R100-R384）：events/nodes 大簇（R104-R186）、Range 全序复刻 + live-range 迁移（R208-R265，mutations 八套件全 100%）、Range 深水区（R266-R289）、selector 工厂域 identity 归一（R290-R311）、L2 查询归并 + 执行路径测绘（R319-R331）、MutationObserver 全族清零（R332-R336）、M4 深收口（R344-R368：sweep 55493P、CE registry per-realm 三片）、域导入 + pending-apply RFC + pa2 代际令牌（R370-R384）；完整 Web API 兼容性仍在此阶段扩展（parse-position 域等剩余深项转入后续专项） |
| M14 | Canvas / WebGL / WebGPU | `⏳ 计划中` | Canvas 2D 继续补全后，逐步进入 Khronos WebGL CTS 和 GPUWeb WebGPU CTS；媒体解码管线（`crates/media`）已起步（webm/VP9/AV1/音频解码 + 播放驱动，见「当前重点」）；不作为 render-compact 的阻塞项 |
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

## 随后计划

1. **M12/M14 拆分子 goal 推进**（storage-opfs / page-wasm / android-browser / webdriver / web-components / event-loop-spec 六线并行，keyboard/editing 三 goal 继续切片）
2. **媒体线延续**（媒体三 goal 2026-09-05 已完成收口归档——H.264 分发前法务复核、Mixer N→1 桌面可选切片、切片 3 stss 索引加速评估等余项挂账；存储/Service Worker 两 goal 亦已于 2026-09-06 收口归档）
3. **render-compact** 深结构（Phase A IFC metric coherence、multicol 碎片化等仍等用户点名授权）
4. **browser-shell** 最小可用产品形态
5. 逐步接入 Test262、WPT testharness、WebDriver wdspec、WebGL/WebGPU CTS 等行业测试
6. 逐步推进完整 JS/DOM API、图形 API、SVG 文档、CSS 动画和真实网站交互兼容性

底层能力和产品层会交替推进。

## 暂不放在当前优先级里的方向

这些事情不是不做，而是不是现在先做：

- 完整 DevTools
- 媒体播放完整体验（媒体三 goal 已完成收口归档，见「当前重点」；产品级完整播放体验仍不在近期主线）
- WebRTC
- 浏览器扩展系统
- 首期移动端发布（Android M0 bootstrap 已落地：Kotlin chrome + Rust JNI 桥，decoder/compositor 复用共享 role 循环；renderer Android transport adapter 与完整移动端发布仍不在近期主线）
- 完全复制 Chromium/Safari/Firefox 的平台字体像素差异

## 关联文档

- [README.md](README.md)
- [docs/goal/zero-web/master.md](docs/goal/zero-web/master.md)
- [docs/goal/rendering-compat.md](docs/goal/rendering-compat.md)
- [docs/goal/rendering-compat/master.md](docs/goal/rendering-compat/master.md)
- [docs/specs/zero-web-spec-rfc.md](docs/specs/zero-web-spec-rfc.md)
- [docs/architecture.md](docs/architecture.md)
