# ZeroWeb

<img src="./banner.svg" alt="ZeroWeb" width="100%">

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-experimental-brown.svg)](#当前状态)
[![CI](https://img.shields.io/badge/ci-github--actions-black.svg)](.github/workflows/ci.yml)
[![Website](https://img.shields.io/badge/website-zeroweb.leizm.com-155eef.svg)](https://zeroweb.leizm.com)

ZeroWeb 是一个用 Rust 写的实验性跨平台浏览器项目。这个仓库同时在做两件事：

官网：[zeroweb.leizm.com](https://zeroweb.leizm.com)

- 一个可嵌入、可复用的 `ZeroWebView` 库
- 一个完整的 `ZeroBrowser` 浏览器应用

项目主线会尽量把核心代码和依赖边界握在自己手里，所以页面内核主要基于宽松许可证的 Rust 组件慢慢搭起来。DOM、CSS、布局、渲染、导航和安全边界，都在这个仓库里一层层补。

这也是个 AI-first 的工程实验。我们想看看，在架构边界、测试和验收都写清楚的前提下，AI 到底能把这种复杂系统推进到什么程度。

> [!IMPORTANT]
> 这个仓库还在实验阶段，主要用来学习、研究和做工程探索。核心 crate 和测试已经有不少东西，但浏览器 shell、完整的页面 JavaScript 运行时、真实站点兼容性都还在路上。它现在不是一个日常可用的浏览器。商用或其他生产用途，请自己评估功能、安全、兼容性、性能和许可证边界风险。

**快速导航**

- [路线图](ROADMAP.md)
- [官网](https://zeroweb.leizm.com)
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
| `ZeroWebView` | 已有稳定嵌入 API、可运行 demo，以及跨 crate 和产品层 smoke 测试；Service Worker、WASM 桥接与 `SecurityContext` 安全检查等页面级能力已接入其中 |
| 浏览器应用 | `zero-browser`、`browser-shell` 和 `zero-renderer` 已打通桌面入口、多进程链路、headless 调试与跨平台打包（Linux / macOS / Windows）+ CI 发布工作流；合成器进程（C2）RFC v2.1 五切片全部落地（scroll transform bake、sync_token + Viz present、GPU mailbox fence + mmap 零拷贝、dma-buf fd 导出、owned window present surface、Linux landlock/seccomp 沙箱、GPU device-lost 模拟 + CPU 回退、crash E2E），Vulkan 真纹理 dma-buf 导出仍为后续；整体仍处于实验阶段 |
| 页面 JavaScript（当前主线） | `script-sandbox` 已提供 V8/QuickJS feature gate（含 V8 持久化 Context 复用）、Web Worker、ES Modules、WebAssembly JS API 到 `wasm-sandbox` 的自动桥接。**P1a DOM/JS Bridge 原生化已主体落地**：fetch 真实化（GET 端到端 + 二进制响应 body 真实字节）、setTimeout 真实延迟、MutationObserver（characterData 变化 / childList addedNodes 回填 / attributeFilter / subtree）与 IntersectionObserver/ResizeObserver 已真实触发回调；表单控件事件、Selectors L4、DOM 遍历/变异 API（含 innerHTML/outerHTML childList emission）、布局几何、`getComputedStyle` 动态 inline 覆盖 + 计算值序列化、classList 完整 DOMTokenList、HTMLCollection/NodeList item/namedItem。**P1b V8 原生 DOM 绑定为当前活跃主线**（2026-08-09 RFC 获批后 R3095 起持续落地）：S0 PoC 验证 → S1 原生只读属性族 + NodeId↔对象映射（native 比 polyfill 快 ~15.6×）→ S2 生产接线（kill-switch 默认关）+ 树写/属性写原生 → live Document 共享（原生写触发重渲染）→ S3 查询原生 → S4 EventTarget 原生 + 事件派发/冒泡/stopPropagation 原生化，dom_bindings 五子模块化闭合；命名空间/序列化 spec 合规（R3181–R3208）；**S5 customElements/Web Components 里程碑完成（R3262–R3269：native HTMLElement 基类 + class 继承、customElements.upgrade/createElement 原生实例、connected/disconnected 生命周期、attributeChangedCallback、子树 upgrade）**；DOM/CSS 表单状态选择器族一致化（R3277–R3284）。完整 Web API 与站点兼容性仍在推进 |
| 渲染兼容性（恢复主动实施） | 以 WPT/CSSWG reftest 对齐 Chromium 为验收标准，Chromium Oracle 像素一致率（`make reftest-oracle`）为诚实度量（同源 reftest 存在假通过，仅作自一致性参考）。自源 reftest 约 77%、Chromium Oracle 真一致约 47.5%、strict 像素级处低位 plateau。自主 clean-lever 轻量修复面已 definitively 穷尽（11 vein 审计全 exhaust）；2026-08-04 起本方向降频守成、主线切回 zero-web DOM/JS Bridge 原生化，**2026-08-09 字体栈重建 RFC v0.2.3 获批后恢复主动实施，多切片已落地**——OpenType features 贯通（R3230-F–R3233-F）、generic advance 统一（R3234-F–R3235-F）、ordered fallback faces（R3236-F–R3241-F）、shaped fallback default-on（R3243-F）、**two-value `font-size-adjust` 全栈贯通（R3245-F，css-fonts Oracle 净改善 14.34pp）**、font-synthesis 与 font-size 绝对关键字（R3247-F–R3248-F）、font-variant 族与 bidi 处理（R3250-F–R3256-F、R3319-F）、@font-face stretch/相对度量（R3341-F–R3344-F）；Phase A IFC / R1043 vertical-mode / R2174 border-box 仍等用户点名。残余缺口集中在 vertical writing modes（部分切片已落地，整体仍待推进）、multicol 碎片化、R109 inline-as-block 等结构性方向，根因是 layout↔paint IFC 度量不一致（Phase-A spread）。详见 [路线图](ROADMAP.md) 与 [docs/goal/rendering-compat.md](docs/goal/rendering-compat.md) |
| HTML 行为兼容（新赛道） | 2026-08-12 启动：以表单场景为起点的规范驱动并行开发线（源码深潜 + 官方规范交叉验证），范围为基础 HTML 元素的解析、DOM/IDL、交互状态、事件与默认动作；已建立表单兼容性基线 + 共享动作事务核心（form 动作/文本编辑/焦点经共享计划路由，可取消文本输入事件、form POST 导航、无 JS 保留默认动作、稳定页面节点身份），配套 [`html-behavior-compatibility-spec-rfc.md`](docs/specs/html-behavior-compatibility-spec-rfc.md) 规格与 `tests/integration/src/html_compat.rs` 常驻断言 |
| 安全与可访问性 | CSP 完整实现、HSTS 预加载、混合内容阻止 / 升级、权限模型与站点隔离已落地并统一接入 `SecurityContext`（R3342/R3343 修复 CSP source-expr 前缀匹配与 mixed-content 大小写绕过）；可访问性基础（`FocusManager` Tab 导航 + ARIA）已起步 |
| 项目定位 | 适合学习、研究、工程探索，不适合直接当成生产浏览器 |

各模块现状见上方表格，未完成的工作见 [路线图](ROADMAP.md)。

## 快速开始

### 1. 前置要求

- Rust `1.85` 或更新版本，包含 `rustfmt` 和 `clippy`
- Linux 和 macOS 请按 [Linux 和 macOS 开发环境](docs/development/linux-macos.md) 配置。
- Windows 开发环境（MSVC、LLVM/libclang、rusty_v8）请按 [Windows 开发环境](docs/development/windows.md) 配置。

### 2. 构建与测试

```bash
make build                                 # 准备 rusty_v8 并构建整个 workspace
make test                                  # 完整测试门禁（经 test-guard 包裹）
make fetch-wpt-data                        # 首次跑 reftest 前先拉取上游 WPT 测试数据（~2 万文件，独立 repo；reftest target 会自动触发）
make reftest                               # = WPT reftest（release 构建，经 test-guard 包裹）
make reftest-oracle                        # ZeroWeb 渲染 vs Chromium Oracle 像素一致率（诚实通过率度量）
make browser                               # 启动浏览器（GPU 模式）
make browser-cpu                           # CPU + scale 1.0 的 WPT 对齐模式
make product-smoke                         # 产品静态页（welcome.html）vs Chromium Oracle 像素回归门禁
make product-smoke-legacy                  # HTML 3.2/4 + CSS1/2 静态页（42 个 fixture）vs Chromium Oracle 趋势门禁
cargo clippy --workspace --all-targets -- -D warnings
```

> [!NOTE]
> 跑测试和 WPT reftest 请用 `make test` / `make reftest`，不要裸跑 `cargo test` 或 `cargo run --bin zero-wpt-runner -- reftest`。`make reftest` 使用 release 构建（约 4× 快于 debug）。这两个 target 由 `scripts/test-guard.rs` 包裹，在单进程 RSS 超过 6 GB、全树内存超过 16 GB 或总时长超过 1800 s 时杀掉整棵进程树，避免内存型 bug（如 CSS parser 未闭合括号死循环）或长时间挂起触发系统级 OOM 连累整台机器。阈值可在命令行覆盖，例如大目录 reftest 需放宽超时：`./target/test-guard --time-limit 7200 -- cargo run --release --bin zero-wpt-runner -- reftest`。

> 涉及渲染 / 布局变更时，建议额外跑 `make product-smoke`：它把产品静态页 `apps/browser/assets/welcome.html` 渲染后与 Chromium Oracle 像素截图对比（默认 diff 超过 20% 即失败，可用 `make product-smoke MAX_DIFF=22` 调阈值），用来捕获 `make test` / `make reftest` 覆盖不到的产品可见回归。

`freetype-raster` feature（默认开启）在非 Ahem 字体路径上用 FreeType 替代 fontdue 光栅化，是 broad 一致率显著提升的关键（R1094 实测全 corpus oracle +232 零回归）。需纯 Rust 构建时：`cargo build --no-default-features -p zero-render-foundation`。

### 3. 运行本地入口

```bash
# rendering pipeline demo (render-foundation + host-runtime)
cargo run --bin webview-demo

# Browser app entrypoint
cargo run --bin zero-browser

# Headless mode: WebSocket remote debugging protocol (default port 9222)
cargo run --bin zero-browser -- --headless --remote-debugging-port=9222
```

想先验证最短渲染链路，可以先跑 `webview-demo`；想直接看浏览器壳、多进程和 GPU 路径，就跑 `make browser`。需要 CPU + scale 1.0 的 WPT 对齐模式时使用 `make browser-cpu`。Windows 对应入口见 [Windows 开发环境](docs/development/windows.md)。

### 4. 打包为可分发产物

需要把 `zero-browser` 打成各平台安装包时，可以用仓库里的打包脚本（产物输出到 `target/packages/`）：

```bash
./scripts/package-linux.sh                                              # Linux：.AppImage / .deb（--appimage|--deb|--all）
./scripts/package-macos.sh                                              # macOS：ZeroBrowser.app + .zip（需在 macOS 上运行）
powershell -ExecutionPolicy Bypass -File scripts/package-windows.ps1    # Windows：.zip（-Installer 生成 NSIS 安装器）
```

macOS 下载产物要免除 Gatekeeper 手工放行，必须使用 Apple Developer ID 签名并完成公证。release/weekly workflow 支持仓库 Secrets：`MACOS_CERTIFICATE`（base64 编码的 `.p12`）、`MACOS_CERTIFICATE_PASSWORD`、`MACOS_KEYCHAIN_PASSWORD`、`APPLE_ID`、`APPLE_TEAM_ID`、`APPLE_APP_PASSWORD`。未配置时仍会生成 ad-hoc 签名的 `.app` zip，但首次运行仍可能被 macOS 拦截。

推送 `v*` tag 时，`.github/workflows/release.yml` 会在 Linux、macOS、Windows 上自动构建并附带产物。项目仍在实验阶段，这些产物仅供本地测试与体验，不代表正式发布。

## 仓库结构

整个工作区共 28 个 workspace member：19 个库 crate、6 个应用入口（`apps/`）、2 个测试工具（`tests/`）和 1 个开发工具（`tools/icon-gen`，不随发布产物分发）。下文按「应用与进程入口 / 核心引擎 / 基础设施 / 产品层与测试」分组列出。

### 应用与进程入口

| Path | 用途 |
|------|------|
| `apps/browser` | 桌面浏览器入口，支持窗口模式和 `--headless` / remote debugging |
| `apps/renderer` | 独立渲染进程入口，负责多进程 IPC 下的页面渲染与脚本执行 |
| `apps/image-decoder` | 图像解码独立进程（PNG/JPEG/WebP，隔离编解码器漏洞），未启用时回退进程内解码 |
| `apps/compositor` | 合成器进程：protocol 消息族 + 真实光栅化（C2） |
| `apps/webdriver` | WebDriver 服务（W3C 协议骨架，wdspec 第一步） |
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
| `crates/product-version` | 产品版本号（从构建日期推导） |
| `crates/psl` | 公共后缀列表（PSL）解析与注册域名（eTLD+1）提取（接入 site-isolation） |
| `crates/wasm-sandbox` | WASM 执行与沙箱能力 |
| `crates/script-sandbox` | 页面 JavaScript 与扩展 / 用户脚本运行时（V8 / QuickJS feature gate） |
| `crates/page-runtime` | WPT / TabWorker / zero-renderer 三条页面路径共享的页面加载与运行时契约（运行时统一） |

### 产品层与测试

| Path | 用途 |
|------|------|
| `crates/webview` | 对外暴露的稳定嵌入 API |
| `crates/browser-shell` | 浏览器产品层数据模型（标签页、书签、历史、设置，UI-agnostic） |
| `docs` | 规格、研究、路线图和架构文档 |
| `tests/integration` | 跨 crate 集成测试 |
| `tests/wpt-runner` | WPT / reftest / 兼容性基础设施 |
| `tests/benchmarks` | benchmark 结果产物 |
| `tools/icon-gen` | 图标资产生成工具（`zero-icon-gen`）：从源 SVG 产出 Linux / Windows / macOS 三端图标（PNG / ICO / iconset / 运行时窗口 RGBA），不随发布产物分发 |

想先了解整体分层，可以看 [docs/architecture.md](docs/architecture.md)。

## 文档导航

| 文档 | 说明 |
|------|------|
| [ROADMAP.md](ROADMAP.md) | 对外路线图，说明已经做完什么、正在推什么、接下来补什么 |
| [CHANGELOG.md](CHANGELOG.md) | 对外发布层面的变更记录 |
| [docs/architecture.md](docs/architecture.md) | 面向贡献者的整体架构与阅读路径 |
| [docs/development/linux-macos.md](docs/development/linux-macos.md) | Linux 和 macOS 开发环境配置 |
| [docs/development/windows.md](docs/development/windows.md) | Windows 开发环境配置 |
| [docs/governance/contribution-responsibility.md](docs/governance/contribution-responsibility.md) | 贡献责任、风险等级、责任域和 owner 路由 |
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
- [docs/governance/contribution-responsibility.md](docs/governance/contribution-responsibility.md)
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- [SECURITY.md](SECURITY.md)

提 PR 前，至少把这几件事做了：

- 变更目标和边界清晰
- 风险等级和责任域已说明；合入前有人类责任维护者接管
- 改动范围尽量小且聚焦
- 对应测试和文档同步更新
- `make test` 与 `cargo clippy` 通过

## 许可证

本项目采用 [MIT License](LICENSE)。

MIT 许可证允许商用和二次集成，但这不等于这个仓库已经适合直接上线。它目前仍是实验项目，默认还是面向学习、研究和工程探索。真要拿去商用或放进生产环境，风险得你自己评估。新增第三方依赖前，也请先确认许可证是否和项目策略兼容。
