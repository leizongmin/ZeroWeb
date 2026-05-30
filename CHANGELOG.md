# Changelog

本文件记录项目对外可感知的重要变更。

写法参考 Keep a Changelog，不过这里的版本号要结合项目现状来看：仓库还在实验阶段，所有预发布版本都应该按实验性版本理解。

## [Unreleased]

### 文档更新

- 重写根 `README.md`，补齐开源协作入口、贡献说明和风险提示。
- 新增面向贡献者的文档：`CONTRIBUTING.md`、`CODE_OF_CONDUCT.md`、`SECURITY.md`、`docs/architecture.md`。
- 在 `.github/` 下新增 issue forms 和 pull request 模板。
- 在关键文档中统一强调：项目仍是实验性代码库，默认不应视为可直接用于生产环境。

## [0.1.0-alpha.0] - 2026-05-30

这是 ZeroBrowser 工作区的首个公开预发布版本。

### 新增

- 建立面向浏览器内核的 Cargo workspace，拆出 DOM、CSS 解析、样式计算、布局、渲染、宿主运行时、网络、存储、IPC、WebView API、Canvas 2D 和 WASM 沙箱等模块边界。
- 打通 `render-foundation` 与 `host-runtime` 的基础集成，并提供基于 `wgpu` 的 `apps/webview-demo` 演示程序。
- 新增 `dom` crate，集成 `html5ever` 并提供 DOM 树操作能力。
- 新增 `css-parser` crate，覆盖 tokenizer、parser、selector 和 CSS 值解析。
- 新增 `style-system` crate，覆盖 cascade、inheritance、computed values、selector matching 和 shorthand expansion。
- 新增 `layout-engine` crate，在 `taffy` 之上整合 block / flex / grid 布局能力。
- 新增 `engine-core` 渲染管线，覆盖 paint、dirty tracking 和 compositing 基础能力。
- 新增 `net` 与 `security` crate，覆盖 URL、导航历史、Cookie、同源检查、CORS 和 CSP 基础能力。
- 新增 `protocol` 与 `storage` crate，覆盖 IPC 消息、localStorage、sessionStorage 和 IndexedDB 基础能力。
- 新增 `canvas` crate，提供 Canvas 2D 图元和绘制能力。
- 新增 `webview-api` crate，作为可嵌入渲染和生命周期 API 的对外入口。
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
