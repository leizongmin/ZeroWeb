# ZeroBrowser

ZeroBrowser 是一个基于 Rust 构建的开源跨平台浏览器项目。它同时追求两个交付物：

- 一个可嵌入、可复用的 `webview` 库
- 一个完整的跨平台浏览器应用

项目从一开始就围绕三个约束展开：核心代码保持可控、依赖许可边界保持清晰、开发过程保持可审查。技术路线是基于宽松许可证的 Rust 组件自建页面内核，并在开源协作下持续迭代 DOM、CSS、布局、渲染、导航和安全边界。

ZeroBrowser 也是一次 AI-first 工程实验。目标不是“让 AI 一次性生成大量代码”，而是在明确的架构、测试和验收约束下，让 AI 主导大部分实现工作，而人类主要负责方向、审查和最终验收。

> **状态**  
> 项目仍处于实验性阶段，仅供学习、研究和工程探索使用。核心引擎 crate 和测试体系已经成形，但浏览器 shell、完整的页面 JavaScript 运行时和真实网页兼容性仍在建设中。它还不是一个可日常使用的浏览器。无论是商用还是其他生产用途，都需要你自行评估功能完整性、安全性、兼容性、性能和许可证边界等风险。

## 项目定位

- **以嵌入为先**: 除浏览器应用外，项目还要交付稳定的 Rust `webview` API，方便其他应用直接集成。
- **许可证边界优先**: 主线依赖优先选择 MIT、Apache-2.0、BSD 等宽松许可证，避免核心能力受不合适的 copyleft 依赖约束。
- **Rust 全栈路线**: 页面内核、宿主层和渲染基础设施都尽量在 Rust 生态内完成。
- **AI 辅助、审查驱动**: 欢迎 AI 辅助贡献，但每一笔改动都必须可解释、可验证、可审查。

## 当前范围

当前工作区已经包含一批实质实现：

- 核心引擎：`dom`、`css-parser`、`style-system`、`layout-engine`、`engine-core`、`canvas`
- 基础设施：`render-foundation`、`host-runtime`、`net`、`security`、`storage`、`protocol`
- 对外与产品层：`webview-api`、`browser-shell`
- 应用与测试：`apps/browser`、`apps/webview-demo`、`tests/integration`、`tests/wpt-runner`

当前仍明确处于进行中的部分：

- `browser-shell` 还没有进入完整产品形态
- `script-sandbox` 仍是占位 crate
- 页面级 JavaScript 执行与 Web API 集成尚未完成
- 真实站点兼容性和 WPT 覆盖率还需要大幅推进

## 快速开始

### 前置要求

- Rust `1.85` 或更新版本
- `cargo fmt`
- `cargo clippy`
- Linux 桌面环境下需要安装与 CI 一致的系统依赖：

```bash
sudo apt-get update
sudo apt-get install -y \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libfontconfig1-dev \
  libwayland-dev \
  libx11-dev \
  libxrandr-dev \
  libxi-dev \
  libgl1-mesa-dev
```

### 构建与测试

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### 运行本地入口

```bash
# WebView / rendering demo
cargo run --bin webview-demo

# Browser app entrypoint
cargo run --bin zero-browser
```

`webview-demo` 是当前更适合体验的入口；`zero-browser` 仍是浏览器应用的占位入口。

## 仓库结构

| Path | 用途 |
|------|------|
| `apps/browser` | 浏览器应用入口 |
| `apps/webview-demo` | WebView / 渲染演示程序 |
| `crates/dom` | DOM 树与 HTML 集成 |
| `crates/css-parser` | CSS tokenizer、parser 与值解析 |
| `crates/style-system` | 级联、继承与计算值 |
| `crates/layout-engine` | 布局整合层 |
| `crates/engine-core` | 渲染管线、paint、dirty tracking、compositing |
| `crates/render-foundation` | GPU/CPU 渲染基础设施、字体、图片缓存 |
| `crates/host-runtime` | 窗口、事件循环、平台宿主能力 |
| `crates/net` | HTTP/HTTPS、导航、Cookie |
| `crates/security` | 同源策略、CORS、CSP |
| `crates/storage` | localStorage、sessionStorage、IndexedDB |
| `crates/protocol` | IPC 与多进程消息模型 |
| `crates/webview-api` | 对外暴露的稳定嵌入 API |
| `crates/browser-shell` | 浏览器产品层 UI |
| `crates/script-sandbox` | 页面脚本 / 扩展脚本运行时占位 |
| `crates/wasm-sandbox` | WASM 执行与沙箱能力 |
| `docs` | 规格、研究、路线图和架构文档 |
| `tests` | 集成测试、WPT runner、benchmark 结果 |

更多架构背景见 [docs/architecture.md](/home/lei/work/ZeroBrowser/docs/architecture.md)。

## 文档导航

- [docs/architecture.md](/home/lei/work/ZeroBrowser/docs/architecture.md): 面向贡献者的整体架构与阅读路径
- [docs/specs/zero-browser-spec-rfc.md](/home/lei/work/ZeroBrowser/docs/specs/zero-browser-spec-rfc.md): 主规格与技术 RFC
- [docs/goal/zero-browser/master.md](/home/lei/work/ZeroBrowser/docs/goal/zero-browser/master.md): 当前实现状态与里程碑控制面
- [docs/research/rust-cross-platform-browser-research.md](/home/lei/work/ZeroBrowser/docs/research/rust-cross-platform-browser-research.md): 早期技术路线与许可证调研
- `crates/*/README.md`: 各子系统的细节说明

## 参与贡献

欢迎人工编写和 AI 辅助编写的贡献。请先阅读：

- [CONTRIBUTING.md](/home/lei/work/ZeroBrowser/CONTRIBUTING.md)
- [CODE_OF_CONDUCT.md](/home/lei/work/ZeroBrowser/CODE_OF_CONDUCT.md)
- [SECURITY.md](/home/lei/work/ZeroBrowser/SECURITY.md)

提交贡献前，至少保证：

- 变更目标和边界清晰
- 改动范围尽量小且聚焦
- 对应测试和文档同步更新
- `cargo test` 与 `cargo clippy` 通过

## 许可证

本项目采用 [MIT License](/home/lei/work/ZeroBrowser/LICENSE)。

许可证允许商用和二次集成，但当前项目仍处于实验阶段，默认仅供学习、研究和工程探索使用。任何商用或其他生产用途，都需要你自行评估功能完整性、安全性、兼容性、性能和许可证边界等风险。对于新增第三方依赖，请在提交前确认其许可证与项目策略兼容。
