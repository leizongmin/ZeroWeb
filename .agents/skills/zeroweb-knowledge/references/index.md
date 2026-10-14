# ZeroWeb 知识索引

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

从下面 8 个领域选择当前任务，再沿领域索引读取直接主题。领域按功能组织，
同一功能可以跨多个 crate；共享协议、图形基础和跨平台配置在单一节点维护，
其他领域用链接引用。源码路径相对项目根，知识导航相对当前文档。

浏览器操作和产品验收参见 [验证入口](../../zeroweb-verify/SKILL.md)、
[功能地图](../../zeroweb-verify/features/index.md)。本知识库记录代码事实与修改约束；
这些导航不表示 GUI、JS/IPC、GPU 或 Android 已在本次初始化中运行通过。
本次新建知识、验证及工程规则资产属于未提交工作区，不归为上方源码基线。

## Child Knowledge Nodes

- [产品入口、浏览器状态与平台宿主](products/index.md) — 桌面导航、标签、输入、Android/JNI、嵌入式 WebView、CDP/WebDriver 和用户 profile。
- [页面内核：文档、样式、布局与绘制](page-core/index.md) — DOM 树、表单、CSS 层叠、行内与块布局、失效、SVG、动画和命中测试。
- [JavaScript、DOM 绑定与脚本沙箱](scripting/index.md) — V8/QuickJS、DOM 身份和快照、事件循环、module、Worker、通用 Web API 和 WASM。
- [网络、持久存储与安全边界](web-data/index.md) — 资源请求、缓存、Cookie、WebSocket、IndexedDB、Cache API/OPFS、origin 隔离、资源门禁和 Service Worker。
- [图形基础设施、字体与 Canvas](graphics/index.md) — GPU/CPU 图元、字体加载和塑形、图片、绘制转换及 Canvas 页面桥。
- [媒体解码、播放与 Web Audio](media/index.md) — 视频与音频解码、播放时间、音频输出、MSE 和流式输入的职责及 feature 限制。
- [多进程运行、共享契约与跨域配置](runtime/index.md) — IPC、页面与合成器生命周期、断连恢复、共享资源协议、feature 和平台分支。
- [测试、构建、官网与 Agent 工程工具](engineering/index.md) — WPT/reftest、生产验收、资源守卫、构建打包、官网及 Agent 交付流程。
