# 窗口与事件宿主

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 `zero-host-runtime` 的窗口配置、winit 事件转换和事件循环。定位窗口关闭、DPI、鼠标坐标、IME 或后台轮询时读取。它不创建 GPU renderer，也不负责页面导航或 Android isolated Service。

真实业务消费者只有 `apps/browser/src/main.rs` 与 `apps/webview-demo/src/main.rs`，因此按普通领域组织，不声称三个业务领域共享。`crates/webview` 的 Cargo 依赖声明不是调用证据。

### Key Entry Points
- `window::HostRuntime` / `WindowConfig`：窗口及事件循环入口。
- `event::AppEvent`：物理坐标、逻辑键名、IME 与后台 Poll 的宿主协议。
- `app_icon::window_icon`：内嵌 RGBA 图标转换。
- 窗口呈现深入 [图形领域](../graphics/index.md)，真实窗口验收见 [验证入口](../../../zeroweb-verify/SKILL.md)；本文仅静态核对，未实跑窗口。

## API Surface

`WindowConfig::new(title)` 提供 `with_size`、`with_resizable`、`with_decorations`、`with_fullscreen`、`with_maximized` 和 `with_unified_titlebar`。`HostRuntime::new(config)` 后消费自身运行以下任一入口：

- `run(FnMut(AppEvent)) -> HostResult<()>`：基本事件模式，不提供窗口引用。
- `run_with_window(FnMut(AppEvent, Option<Arc<Window>>)) -> HostResult<()>`：消费者自行创建 Surface。
- `run_with_window_polling(Duration, Arc<AtomicBool>, callback) -> HostResult<()>`：额外派发后台 Poll，不自动重绘。

## Usage Examples

桌面入口在创建 HostRuntime 前按平台设置真实窗口配置。以下是 `browser_window_config` 中的连续摘录；16ms 轮询接线见下方消费者分析。来源：`apps/browser/src/main.rs`。

```rust
let mut config = WindowConfig::new("ZeroBrowser")
    .with_size(1024, 768)
    .with_resizable(true)
    .with_maximized(true);
```

Demo 先创建窗口配置和运行时，随后用 run_with_window 回调初始化 Surface。来源：`apps/webview-demo/src/main.rs`。

```rust
let config = WindowConfig::new("ZeroWeb Demo — wgpu GPU").with_size(800, 600);
let runtime = HostRuntime::new(config);
```

## Gotchas
- 真实基本模式的 CloseRequested 直接退出 event loop，不派发给业务回调；GPU 模式先回调再退出，基本模式测试 helper 的 close 分发不能证明生产模式能执行退出清理。 (`crates/host-runtime/src/window.rs`, `crates/host-runtime/src/window_tests.rs`)
- 窗口在 `resumed` 创建，回调类型仍允许 `None`；Surface 初始化必须以实际窗口引用为前提，不能仅凭事件类型假定窗口已经可用。 (`crates/host-runtime/src/window.rs`, `apps/webview-demo/src/main.rs`)
- WindowConfig 的尺寸用 LogicalSize 创建窗口，Resized、鼠标、触摸和 PanGesture 则携带物理像素；消费者必须分别处理窗口尺寸与页面逻辑视口，不能把同名 width 当同一单位。 (`crates/host-runtime/src/window.rs`, `crates/host-runtime/src/event.rs`, `apps/webview-demo/src/main.rs`)
- MouseInput、MouseWheel 和 PanGesture 使用 PointerTracker 最近坐标，Touch 也更新该坐标；未收到任何指针事件时坐标为零，事件并不是重新向系统查询当前位置。 (`crates/host-runtime/src/window.rs`)
- KeyboardInput 的 `key` 是 logical_key 的字符串，不是物理 `KeyA`；Named 用 Debug 名称，Unidentified 保留该名称而 text 可另有内容，输入处理不能只读 key 丢掉远程键盘文字。 (`crates/host-runtime/src/event.rs`)
- `HostError::WindowCreationFailed` 的存在不代表创建失败被结构化返回：当前 resumed 使用 `expect("Failed to create window")`，窗口创建失败会 panic；EventLoop 构建与运行失败才映射 HostError。 (`crates/host-runtime/src/lib.rs`, `crates/host-runtime/src/window.rs`)
- `window_new.rs` 不在 lib 的公开模块声明树内，不能用于说明现行 API；README 对基本模式关闭回调的示例也须以 window.rs 的真实分发行为为准。 (`crates/host-runtime/src/lib.rs`, `crates/host-runtime/src/window.rs`, `crates/host-runtime/README.md`)
- lib 内旧测试以 Destroyed 代替不可构造的 ScaleFactorChanged 并声称缩放未处理，但当前 BasicApp/GpuApp 都已分发该事件；该替代测试只覆盖被忽略事件，不覆盖 DPI 变化。 (`crates/host-runtime/src/lib.rs`, `crates/host-runtime/src/window.rs`)

## Architecture
- Poll 在 `about_to_wait` 中消费后台结果；active 为 false 时 ControlFlow::Wait，为 true 时回调后重新读取标记并设 WaitUntil，Poll 与 RedrawRequested 是分离的业务通道。 (`crates/host-runtime/src/window.rs`, `crates/host-runtime/src/event.rs`)
- HostRuntime 保存窗口配置，不保存 GPU/CPU 渲染对象；Arc<Window> 的交付允许宿主独立持有 Surface 生命周期，Demo 的 auto 模式在 GPU 初始化失败时回退 CPU。 (`crates/host-runtime/src/window.rs`, `apps/webview-demo/src/main.rs`)
- Linux 仅识别 WINIT_UNIX_BACKEND 的 x11/wayland，忽略大小写与首尾空白；其他值不强制后端，由 winit 自动选择，不是任意后端名透传。 (`crates/host-runtime/src/window.rs`)

## Decisions
- macOS 一体化标题栏采用透明标题栏、隐藏标题文字和 fullsize content view，刻意不用隐藏整个标题栏的选项，以保留系统红黄绿按钮。 (`crates/host-runtime/src/window.rs`)
- 桌面 Wayland 禁用 CSD 是为避开失焦时 subsurface commit 导致 compositor 断开；Windows 同样无装饰但为自绘 caption buttons，不能把相同配置推断为相同平台原因。 (`apps/browser/src/main.rs`)
- 窗口图标嵌入 256×256 RGBA，解码失败返回 None 不阻止启动；macOS 图标来源是应用 bundle 的 icns，而不是窗口图标路径。 (`crates/host-runtime/src/app_icon.rs`, `crates/host-runtime/src/window.rs`)

## Consumer Analysis
- 桌面浏览器使用 `run_with_window_polling` 的 16ms Poll 消费 IPC、信号与 smoke/driver 状态，并在显式 process::exit 前保存用户数据及清理子进程；不能把 event loop 退出当 renderer 自动清理。 (`apps/browser/src/main.rs`)
- Demo 使用 `run_with_window`，首次重绘拿窗口创建 Surface，Resized 与 ScaleFactorChanged 更新物理/逻辑尺寸并按 needs_redraw 请求绘制；代码注释中的“持续刷新”不是 HostRuntime 自动请求每帧的保证。 (`apps/webview-demo/src/main.rs`)
