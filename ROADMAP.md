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

- 当前优先完成 `rendering-compat` / render-compact 验收，把静态 CSS-heavy 页面渲染拉到可验证的主流浏览器参考水平
- 外部 CSS、图片子资源、SVG/PNG Logo、layout/paint/glyph 一致性和产品静态页截图门禁
- `browser-shell` 的产品层骨架
- render-compact 验收通过后，再逐步推进完整交互 Web 能力、图形 API、SVG 文档、动画逐帧和真实网站交互兼容性

## 路线图

| 阶段 | 主题 | 状态 | 说明 |
|------|------|------|------|
| M1 | 项目骨架与渲染基础设施 | `✅ 已完成` | 工作区、GPU/CPU 渲染基础、`webview-demo` 入口、CI 骨架 |
| M2 | HTML 解析与 DOM | `✅ 已完成` | `html5ever` 集成、DOM 树、基础查询与文档模型 |
| M3 | CSS 解析与样式系统 | `✅ 已完成` | tokenizer、parser、选择器、级联、继承、计算值、简写展开、`@media` 基础 |
| M4 | 布局引擎 | `✅ 已完成` | block / flex / grid 基础整合，几何验证就位 |
| M5 | 渲染管线集成 | `✅ 已完成` | paint、dirty tracking、compositing、渲染链路打通 |
| M6 | JavaScript 运行时与 DOM 绑定 | `🚧 进行中` | 这是眼下最大的缺口，决定页面交互和真实兼容性上限 |
| M7 | 网络、安全与导航基础 | `✅ 已完成` | HTTP、URL、导航历史、Cookie、同源策略、CORS、CSP 基础能力 |
| M8 | 协议与多进程基础 | `✅ 已完成` | IPC 消息、协议定义、序列化边界已经建立 |
| M9 | Canvas 与存储 | `✅ 已完成` | Canvas 2D、localStorage、sessionStorage、IndexedDB 基础已在仓库中 |
| M10 | WebView API | `✅ 已完成` | 已有可嵌入 API 和对应测试，但还会继续演进 |
| M11 | 浏览器产品层 | `⏳ 计划中` | `browser-shell`、标签页、地址栏、历史、权限 UI 等仍需补齐 |
| M12 | Render compatibility / render-compact | `🚧 进行中` | 先把 CSS reftest、静态页面截图对比、外部 CSS、图片资源、layout/paint/glyph 一致性补到可验收 |
| M13 | 完整 JS/DOM API 兼容性 | `⏳ 计划中` | render-compact 验收后推进；目标是从基础 DOM bridge 扩展到更完整的 Web API、事件循环、DOM/CSSOM 操作和真实页面脚本行为 |
| M14 | Canvas / WebGL / WebGPU | `⏳ 计划中` | Canvas 2D 继续补全后，逐步进入 WebGL/WebGPU；不作为 render-compact 的阻塞项 |
| M15 | SVG 文档与内联 SVG DOM 渲染 | `⏳ 计划中` | render-compact 只要求 SVG 作为图片资源栅格化；完整 SVG 文档、内联 SVG DOM、样式和交互放到后续阶段 |
| M16 | CSS 动画逐帧一致性 | `⏳ 计划中` | render-compact 关注静态截图和必要的 CSS 视觉状态；动画/transition 的帧级时间轴、插值和截图一致性后续单独验收 |
| M17 | 真实网站完整交互行为 | `⏳ 计划中` | 在静态渲染稳定后，扩展到登录、表单、路由、滚动、输入、复杂脚本和长会话行为 |
| M18 | 平台字体像素级一致性 | `❄️ 暂不优先` | 不作为近期主线；长期再评估是否追近 Chromium/Safari/Firefox 在不同平台的字体 fallback、hinting、subpixel 和 emoji 细节 |

## 接下来大概率会先做什么

如果按当前仓库状态往下走，顺序大致会是：

1. 完成 render-compact 验收：WPT reftest、静态产品页、真实静态文章页、图片密集首页都要能和 Chromium 做稳定截图对比。
2. 补齐外部 stylesheet、图片子资源/ImageCache、SVG 作为图片资源的栅格化，以及 ZeroBrowser glyph 后处理收敛。
3. 统一 inline formatting、layout IFC 和 paint IFC 的权威结果，解决文本串联、重叠、标题误拆行和正文压缩。
4. 搭出 `browser-shell` 最小可用骨架，让浏览器产品层真正出现。
5. render-compact 验收后，再逐步推进完整 JS/DOM API、Canvas/WebGL/WebGPU、SVG 文档、CSS 动画逐帧和真实网站完整交互行为。

这不是死板顺序。实际推进时，底层能力和产品层会交替往前推。

## 暂不放在当前优先级里的方向

这些事情不是不做，而是不是现在先做：

- 完整 DevTools
- 媒体播放（`<video>` / `<audio>`）
- WebGL / WebGPU（render-compact 之后进入后续阶段）
- WebRTC
- 浏览器扩展系统
- 首期移动端发布
- 完全复制 Chromium/Safari/Firefox 的平台字体像素差异

## 关联文档

- [README.md](README.md)
- [docs/goal/zero-web/master.md](docs/goal/zero-web/master.md)
- [docs/specs/zero-web-spec-rfc.md](docs/specs/zero-web-spec-rfc.md)
- [docs/architecture.md](docs/architecture.md)
