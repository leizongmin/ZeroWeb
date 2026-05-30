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

- JavaScript 运行时和页面级 Web API
- `browser-shell` 的产品层骨架
- 真实站点兼容性、WPT 覆盖和浏览器级加固
- 渲染细节补全，比如 transforms、overflow clip、border-radius、图片渲染

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
| M12 | 兼容性、加固与发布准备 | `⏳ 计划中` | WPT、真实站点验证、性能回归、发布流程、文档收口 |

## 接下来大概率会先做什么

如果按当前仓库状态往下走，顺序大致会是：

1. 补 JavaScript 运行时基础，把页面脚本真正跑起来。
2. 扩更多 DOM API，把静态页面能力往交互页面能力推进。
3. 继续补渲染和布局细节，把 CSS 行为往真实站点靠。
4. 搭出 `browser-shell` 最小可用骨架，让浏览器产品层真正出现。
5. 开始更系统地做兼容性、性能和安全加固。

这不是死板顺序。实际推进时，底层能力和产品层会交替往前推。

## 暂不放在当前优先级里的方向

这些事情不是不做，而是不是现在先做：

- 完整 DevTools
- 媒体播放（`<video>` / `<audio>`）
- WebGL / WebGPU
- WebRTC
- 浏览器扩展系统
- 首期移动端发布

## 关联文档

- [README.md](README.md)
- [docs/goal/zero-web/master.md](docs/goal/zero-web/master.md)
- [docs/specs/zero-web-spec-rfc.md](docs/specs/zero-web-spec-rfc.md)
- [docs/architecture.md](docs/architecture.md)
