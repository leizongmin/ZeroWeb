# ZeroWeb WebView Demo (`zero-webview-demo`)

> wgpu「Hello ZeroWeb」文本渲染演示 — 渲染管线 M1 里程碑示例

## 概述

`zero-webview-demo`（bin 名 `webview-demo`）是 ZeroWeb 渲染管线的 M1 里程碑演示：创建桌面窗口，使用 wgpu GPU 渲染静态文本「Hello ZeroWeb!」。它演示 `zero-host-runtime` 的窗口管理（`HostRuntime` / `WindowConfig`）与 `zero-render-foundation` 的 GPU/CPU 渲染器集成。本 demo 不依赖 `zero-webview` 的 WebViewBuilder / WebViewEvent 嵌入 API，是渲染栈（而非 WebView 嵌入）的最小示例。

## 主要功能

- **GPU 渲染** — 经 `GpuRenderer` 使用 wgpu 渲染「Hello ZeroWeb!」文本（32px 深灰字、白色背景、水平居中）
- **CPU 后备** — GPU 不可用时自动降级为 softbuffer 软件渲染；同时将渲染结果输出到 `demo_output.ppm`
- **渲染模式切换** — `--renderer auto|gpu|cpu` 命令行参数（或 `ZEROWEB_RENDERER` 环境变量），默认 `auto`（优先 GPU，失败降级 CPU）
- **字体管线** — 加载系统字体并光栅化 glyph；未找到系统字体时回退到内置 5x7 点阵字体

## 使用示例

```bash
# 运行 demo（默认 auto：优先 GPU，失败降级 CPU）
cargo run --bin webview-demo

# 强制 GPU / CPU 渲染
cargo run --bin webview-demo -- --renderer gpu
cargo run --bin webview-demo -- --renderer cpu
```

## 源码导航

`src/main.rs`（单文件）：

```
main                        — 入口：解析渲染模式、CPU 后备 PPM 输出、创建 HostRuntime 窗口并进入事件循环
parse_render_mode_from_args — 解析 --renderer 参数（支持 --renderer=value 与 --renderer value 两种写法）
DemoState                   — 应用状态：GPU 渲染器 / CPU surface / 字体与 glyph 缓存
build_scene                 — 构建静态场景（背景填充 + 居中文本 glyph 序列）
render_gpu / render_cpu     — GPU（wgpu）与 CPU（softbuffer）双后端渲染
load_system_font            — 加载系统字体（Linux/macOS/Windows 候选路径）
get_font5x7 / render_text_fallback — 5x7 点阵后备字体
logical_size_from_window    — 窗口逻辑尺寸与缩放因子换算
```
