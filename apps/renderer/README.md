# ZeroWeb Renderer (`zero-renderer`)

> 渲染进程 — 独立进程处理页面渲染与 JS 执行，经 IPC 向浏览器传递绘制快照

## 概述

`ZeroWeb Renderer` (`zero-renderer`) 是 ZeroWeb 多进程架构中的独立渲染进程（bin），由浏览器主进程通过 stdin/stdout 管道 spawn，Windows 下以 GUI 子系统运行（不弹控制台）。renderer 内部持有 `zero-webview` 的 `WebView` 作为统一页面运行时（B3：渲染 / 字体 / 脚本 / hit-test 全经 WebView，与浏览器内 tabworker 同一页面运行时），处理导航、分阶段页面加载、脚本执行、表单交互，并把绘制帧经 IPC 发布给浏览器（默认发布到 compositor 进程；`ZW_COMPOSITOR_PROCESS=0` 时回退 legacy 直发路径）。同 crate 也暴露 `zero_renderer` lib，供 in-process 测试复用运行时 wiring。

## 主要功能

- **分阶段页面加载** — `AsyncPageLoad` 异步 tick 模型（16ms 帧预算），主文档经浏览器 IPC 代理抓取（GET/POST），支持 `LoadComplete` / 错误页生成（`error://`）
- **JS 执行 worker** — `RendererJsWorker` 独立线程执行页面脚本与 ES module（V8/QuickJS feature gate），含脚本预取（并发 4）、fetch 注入、`DOMContentLoaded` / `load` 生命周期与资源事件派发
- **帧发布** — `CompositorPublishThread` 异步发布线程保序发送页面帧（含图片像素去重：browser 端 ImageCache 已存则不重传），`FrameTransaction` 在输入消息边界内合并渲染与发布
- **表单交互** — 焦点管理（focus/blur/change-on-blur）、IME 合成（preedit/commit）、Tab 焦点导航、radio/checkbox/label 激活、表单提交（Enter / click）、`javascript:` 链接与 hash 路由
- **事件路由** — hit-test（元素 / 链接 / 图片）、键盘 / 鼠标 / 滚动事件派发、CSS 过渡与动画事件（transitionstart/run/end、animationstart/end/iteration）、observer tick
- **字体管线** — 系统字体加载、@font-face live 加载（`ZW_LIVE_FONTFACE` 可禁用）、`FontFace.load()`、文本度量注入、行度量 map（`ZW_PERFONT_LINEHEIGHT`）
- **平台集成** — macOS App 支持（dispatch / AppKit）、Windows GUI 子系统、Linux 沙箱（Landlock/seccomp）

## 使用示例

`zero-renderer` 由浏览器主进程自动 spawn（`--type=renderer` 风格参数），不面向终端用户直接运行：

```bash
# 手动启动仅用于调试（正常由 zero-browser 经 stdin/stdout 管道 spawn）
cargo run --bin zero-renderer

# 帧发布模式选择：默认 compositor；=0 回退 legacy 直发
ZW_COMPOSITOR_PROCESS=1 cargo run --bin zero-browser
```
