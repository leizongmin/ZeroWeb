# ZeroWeb

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-experimental-brown.svg)](#当前状态)
[![CI](https://img.shields.io/badge/ci-github--actions-black.svg)](.github/workflows/ci.yml)

ZeroWeb 是一个用 Rust 写的实验性跨平台浏览器项目。这个仓库同时在做两件事：

- 一个可嵌入、可复用的 `ZeroWebView` 库
- 一个完整的 `ZeroBrowser` 浏览器应用

项目主线会尽量把核心代码和依赖边界握在自己手里，所以页面内核主要基于宽松许可证的 Rust 组件慢慢搭起来。DOM、CSS、布局、渲染、导航和安全边界，都在这个仓库里一层层补。

这也是个 AI-first 的工程实验。我们想看看，在架构边界、测试和验收都写清楚的前提下，AI 到底能把这种复杂系统推进到什么程度。

> [!IMPORTANT]
> 这个仓库还在实验阶段，主要用来学习、研究和做工程探索。核心 crate 和测试已经有不少东西，但浏览器 shell、完整的页面 JavaScript 运行时、真实站点兼容性都还在路上。它现在不是一个日常可用的浏览器。商用或其他生产用途，请自己评估功能、安全、兼容性、性能和许可证边界风险。

**快速导航**

- [路线图](ROADMAP.md)
- [当前状态](#当前状态)
- [快速开始](#快速开始)
- [仓库结构](#仓库结构)
- [文档导航](#文档导航)
- [参与贡献](#参与贡献)
- [许可证](#许可证)

## 项目定位

- **以嵌入为先**: 除浏览器应用外，项目还要交付稳定的 Rust `ZeroWebView` API，方便其他应用直接集成。
- **许可证边界优先**: 主线依赖优先选择 MIT、Apache-2.0、BSD 等宽松许可证，避免核心能力受不合适的 copyleft 依赖约束。
- **Rust 全栈路线**: 页面内核、宿主层和渲染基础设施都尽量在 Rust 生态内完成。
- **AI 可以写，但要能审**: 欢迎 AI 辅助贡献，但提交上来的改动必须能讲清楚、测清楚、看清楚。

## 当前状态

| 方向 | 现状 |
|------|------|
| `ZeroWebView` | 已有稳定嵌入 API、可运行 demo，以及跨 crate 和产品层 smoke 测试 |
| 浏览器应用 | `zero-browser`、`browser-shell` 和 `zero-renderer` 已打通桌面入口、多进程链路与 headless 调试；整体仍处于实验阶段 |
| 页面 JavaScript | `script-sandbox` 已提供 V8/QuickJS feature gate 与脚本运行时基础能力；完整 Web API 和站点兼容性仍在推进 |
| 真实站点兼容性 | 已有 WPT runner、reftest 和真实站点 smoke，但离生产级兼容性还有明显距离 |
| 项目定位 | 适合学习、研究、工程探索，不适合直接当成生产浏览器 |

现在已经有一批能跑起来的模块：

- 核心引擎：`dom`、`css-parser`、`style-system`、`layout-engine`、`engine`、`canvas`
- 基础设施：`render-foundation`、`host-runtime`、`net`、`security`、`storage`、`protocol`
- 运行时与隔离：`script-sandbox`、`wasm-sandbox`
- 对外与产品层：`webview`、`browser-shell`
- 应用与测试：`apps/browser`、`apps/renderer`、`apps/webview-demo`、`tests/integration`、`tests/wpt-runner`

还没做完的地方也很明确：

- `browser-shell` 还没有进入完整产品形态
- 页面级 JavaScript 和 Web API 兼容性仍在持续补齐
- 浏览器产品层、无头调试和多进程链路还需要更多稳定性验证
- 真实站点兼容性和 WPT 覆盖率还需要大幅推进

## 快速开始

### 1. 前置要求

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

### 2. 构建与测试

```bash
cargo build --workspace
make test                                  # = cargo test --workspace（经 test-guard 包裹）
make reftest                               # = WPT reftest（同样经 test-guard 包裹）
cargo clippy --workspace --all-targets -- -D warnings
```

> [!NOTE]
> 跑测试和 WPT reftest 请用 `make test` / `make reftest`，不要裸跑 `cargo test` 或 `cargo run --bin zero-wpt-runner -- reftest`。这两个 target 由 `scripts/test-guard.rs` 包裹，在单进程 RSS 或全树内存超限时杀掉整棵进程树，避免内存型 bug（如 CSS parser 未闭合括号死循环）触发系统级 OOM 连累整台机器。

在 Linux 和 macOS 上，构建前需先下载 `rusty_v8` 预构建产物：`make setup-rusty-v8`（缓存到 `${XDG_CACHE_HOME:-$HOME/.cache}/zero-web/rusty_v8`）。推荐用 `make build` 或 `make browser`，会自动执行该步骤。Windows 需在本地环境里设置 `RUSTY_V8_ARCHIVE` 为 release `.lib` 的 URL。

### 3. 运行本地入口

```bash
# rendering pipeline demo (render-foundation + host-runtime)
cargo run --bin webview-demo

# Browser app entrypoint
cargo run --bin zero-browser

# Headless mode: WebSocket remote debugging protocol (default port 9222)
cargo run --bin zero-browser -- --headless --remote-debugging-port=9222
```

想先验证最短渲染链路，可以先跑 `webview-demo`；想直接看浏览器壳、多进程和 headless 能力，就跑 `zero-browser`。`zero-browser` 还支持 `--renderer=<mode>`（切换渲染后端）、`--scale=<factor>`（HiDPI）等参数，`--help` 可看完整列表。在 Linux 和 macOS 上，`make build` / `make browser` 会先自动处理 `rusty_v8` 下载。

## 仓库结构

### 应用与进程入口

| Path | 用途 |
|------|------|
| `apps/browser` | 桌面浏览器入口，支持窗口模式和 `--headless` / remote debugging |
| `apps/renderer` | 独立渲染进程入口，负责多进程 IPC 下的页面渲染与脚本执行 |
| `apps/webview-demo` | 最小渲染管线演示程序（wgpu/CPU 渲染静态文本，演示 render-foundation 与 host-runtime 集成） |

### 核心引擎

| Path | 用途 |
|------|------|
| `crates/dom` | DOM 树与 HTML 集成 |
| `crates/css-parser` | CSS tokenizer、parser 与值解析 |
| `crates/style-system` | 级联、继承与计算值 |
| `crates/layout-engine` | 布局整合层 |
| `crates/engine` | 渲染管线、paint、dirty tracking、compositing |
| `crates/canvas` | Canvas 2D 能力 |

### 基础设施

| Path | 用途 |
|------|------|
| `crates/render-foundation` | GPU/CPU 渲染基础设施、字体、图片缓存 |
| `crates/host-runtime` | 窗口、事件循环、平台宿主能力 |
| `crates/net` | HTTP/HTTPS、导航、Cookie |
| `crates/security` | 同源策略、CORS、CSP |
| `crates/storage` | localStorage、sessionStorage、IndexedDB、Cache API |
| `crates/protocol` | IPC 与多进程消息模型 |
| `crates/wasm-sandbox` | WASM 执行与沙箱能力 |
| `crates/script-sandbox` | 页面 JavaScript 与扩展 / 用户脚本运行时（V8 / QuickJS feature gate） |

### 产品层与测试

| Path | 用途 |
|------|------|
| `crates/webview` | 对外暴露的稳定嵌入 API |
| `crates/browser-shell` | 浏览器产品层数据模型（标签页、书签、历史、设置，UI-agnostic） |
| `docs` | 规格、研究、路线图和架构文档 |
| `tests/integration` | 跨 crate 集成测试 |
| `tests/wpt-runner` | WPT / reftest / 兼容性基础设施 |
| `tests/benchmarks` | benchmark 结果产物 |

想先了解整体分层，可以看 [docs/architecture.md](docs/architecture.md)。

## 文档导航

| 文档 | 说明 |
|------|------|
| [ROADMAP.md](ROADMAP.md) | 对外路线图，说明已经做完什么、正在推什么、接下来补什么 |
| [CHANGELOG.md](CHANGELOG.md) | 对外发布层面的变更记录 |
| [docs/architecture.md](docs/architecture.md) | 面向贡献者的整体架构与阅读路径 |
| [docs/releases/github-metadata.md](docs/releases/github-metadata.md) | GitHub 仓库介绍、Topics、tag 和 release 标题建议 |
| [docs/releases/v0.1.0-alpha.0.md](docs/releases/v0.1.0-alpha.0.md) | 首个预发布版本的 release 文案草稿 |
| [docs/specs/zero-web-spec-rfc.md](docs/specs/zero-web-spec-rfc.md) | 主规格与技术 RFC |
| [docs/goal/zero-web/master.md](docs/goal/zero-web/master.md) | 当前实现状态与里程碑控制面 |
| [docs/goal/rendering-compat.md](docs/goal/rendering-compat.md) | 渲染兼容性（reftest / WPT 兼容性）执行控制面与进展记录 |
| [docs/research/rust-cross-platform-browser-research.md](docs/research/rust-cross-platform-browser-research.md) | 早期技术路线与许可证调研 |
| `crates/*/README.md` | 各子系统的细节说明 |

## 参与贡献

想提改动的话，先看这几份文档：

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- [SECURITY.md](SECURITY.md)

提 PR 前，至少把这几件事做了：

- 变更目标和边界清晰
- 改动范围尽量小且聚焦
- 对应测试和文档同步更新
- `cargo test` 与 `cargo clippy` 通过

## 许可证

本项目采用 [MIT License](LICENSE)。

MIT 许可证允许商用和二次集成，但这不等于这个仓库已经适合直接上线。它目前仍是实验项目，默认还是面向学习、研究和工程探索。真要拿去商用或放进生产环境，风险得你自己评估。新增第三方依赖前，也请先确认许可证是否和项目策略兼容。
