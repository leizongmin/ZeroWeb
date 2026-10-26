# 构建、测试与本地运行

环境准备先看 [Linux 和 macOS 开发环境](linux-macos.md) 或 [Windows 开发环境](windows.md)。提交前需要跑哪些检查，以 [命令与质量门禁](../engineering/quality-gates.md) 为准。

## 前置要求

- Rust `1.85` 或更新版本，包含 `rustfmt` 和 `clippy`
- 各平台系统依赖见上面两份开发环境文档

## 常用命令

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
> 跑测试和 WPT reftest 请用 `make test` / `make reftest`，不要裸跑 `cargo test` 或 `cargo run --bin zero-wpt-runner -- reftest`。入口会先在不设内存阈值的阶段编译，随后由 `scripts/test-guard.rs` 只包裹运行阶段；运行时超过内存或时长阈值即杀掉整棵进程树（`make test` 显式传单进程 4 GB、全树 8 GB、900 s；`make reftest` 用 test-guard 默认的单进程 6 GB、全树 16 GB、1800 s），避免内存型 bug（如 CSS parser 未闭合括号死循环）或长时间挂起触发系统级 OOM 连累整台机器。阈值可在命令行覆盖，例如大目录 reftest 需放宽超时：`cargo build --release --bin zero-wpt-runner && ./target/test-guard --time-limit 7200 -- ./target/release/zero-wpt-runner reftest`。详见 [OOM 防护](../rally/oom-guard.md)。

涉及渲染 / 布局变更时，建议额外跑 `make product-smoke`：它把产品静态页 `apps/browser/assets/welcome.html` 渲染后与 Chromium Oracle 像素截图对比（默认 diff 超过 20% 即失败，可用 `make product-smoke MAX_DIFF=22` 调阈值），用来捕获 `make test` / `make reftest` 覆盖不到的产品可见回归。

`freetype-raster` feature（默认开启）在非 Ahem 字体路径上用 FreeType 替代 fontdue 光栅化，是 broad 一致率显著提升的关键（R1094 实测全 corpus oracle +232 零回归）。需纯 Rust 构建时：`cargo build --no-default-features -p zero-render-foundation`。

## 运行本地入口

```bash
# rendering pipeline demo (render-foundation + host-runtime)
cargo run --bin webview-demo

# Browser app entrypoint
cargo run --bin zero-browser

# Headless mode: WebSocket remote debugging protocol (default port 9222)
cargo run --bin zero-browser -- --headless --remote-debugging-port=9222
```

想先验证最短渲染链路，可以先跑 `webview-demo`；想直接看浏览器壳、多进程和 GPU 路径，就跑 `make browser`。需要 CPU + scale 1.0 的 WPT 对齐模式时使用 `make browser-cpu`。Windows 对应入口见 [Windows 开发环境](windows.md)。

打包成各平台安装包见 [打包与发布](../releases/packaging.md)。
