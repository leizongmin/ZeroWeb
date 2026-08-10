# GitHub 仓库元数据建议

这份文档整理了 ZeroWeb 在 GitHub 仓库设置页和首个 Release 页面里可以直接拿去用的文案。

## 官方网站

`https://zeroweb.leizm.com`

将此地址填写到 GitHub 仓库 **About → Website**，并用于发布产物的 Homepage 字段。

## 仓库一句话介绍

### 推荐英文版

`An experimental Rust browser engine and embeddable WebView project, built in the open for learning, research, and AI-first engineering exploration.`

### 备选短版

`An experimental Rust browser engine and embeddable WebView project for learning, research, and AI-first engineering exploration.`

### 中文参考

`一个实验性的 Rust 浏览器内核与可嵌入 WebView 项目，面向学习、研究和 AI-first 工程探索。`

## GitHub Topics

GitHub Topics 最好别堆太多，10 到 12 个就够了。优先保留最能说明项目性质和技术方向的标签。

### 推荐 Topics

- `rust`
- `browser-engine`
- `webview`
- `browser`
- `rendering`
- `layout-engine`
- `dom`
- `css`
- `wgpu`
- `wasm`
- `cross-platform`
- `experimental`

### 可选替换项

如果你想更强调 AI 协作属性，可以把一两个通用标签换成：

- `ai-assisted`
- `ai-first`

Topics 太多反而会把“Rust 浏览器内核 / WebView / 实验项目”这几个核心信号冲淡。

## 首个版本 Tag

### 推荐

`v0.1.0-alpha.0`

### 为什么先用这个

- 当前 workspace 版本已经是 `0.1.0`
- 仓库还没有历史 tag
- 项目明确处于实验阶段
- `alpha.0` 比较准确地表达了“第一次公开预发布”，不容易让人误会成稳定版本

## 首个 Release 标题

### 推荐标题

`v0.1.0-alpha.0 — First public pre-release`

### 中文说明版本

`v0.1.0-alpha.0 — 首个公开预发布版本`

## 首个 Release 摘要

### 英文短摘要

`ZeroWeb v0.1.0-alpha.0 is the first public pre-release of an experimental Rust browser-engine workspace with early DOM, CSS, layout, rendering, networking, storage, WebView, Canvas, and WASM foundations. This release is intended for learning and research, not for production use.`

### 中文短摘要

`ZeroWeb v0.1.0-alpha.0 是首个公开预发布版本，当前已经具备 DOM、CSS、布局、渲染、网络、存储、WebView、Canvas 和 WASM 等基础模块。该版本主要面向学习和研究，不面向生产用途。`

更完整的首个发布说明草稿在 [v0.1.0-alpha.0.md](v0.1.0-alpha.0.md)。

## 后续版本建议

如果后续继续公开预发布，可以先按这个节奏往下走：

- `v0.1.0-alpha.1`
- `v0.1.0-beta.0`
- `v0.1.0-rc.0`
- `v0.1.0`

如果在 `0.1.0` 之前 API 或架构还会明显变动，也完全可以继续挂更多 alpha tag，不用急着进 beta。
