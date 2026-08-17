# ZeroWeb Image Decoder (`zero-image-decoder`)

> 图像解码进程（D1）— 独立进程解码 PNG/JPEG/WebP，隔离编解码器漏洞

## 概述

`ZeroWeb Image Decoder` (`zero-image-decoder`) 是 ZeroWeb 的独立图像解码进程（对照 Ladybird ImageDecoder 进程）。由渲染进程内 webview 的 `ImageDecoderProxy` 经 stdin/stdout 管道 spawn（`--type=image-decoder --instance-id=N`）。动机：编解码器处理不可信输入（畸形图片），独立进程把解码器漏洞限制在子进程内，崩溃或利用不波及渲染主进程。

注意：SVG 解码依赖资源加载（字体等），保持在渲染进程内完成；本进程仅处理 PNG/JPEG/WebP（mime 显式分派，见 webview 侧）。

## 主要功能

- **独立进程隔离** — 解码器漏洞被限制在子进程地址空间，对照 Ladybird ImageDecoder 进程设计
- **PNG / JPEG / WebP 解码** — mime 显式分派，复用 `zero-render-foundation` 的图像解码器，与进程内路径同一实现，保证结果一致
- **零协议 IPC** — 与 renderer 同款：bincode 序列化 `IpcMessage` 经 stdin/stdout 管道
- **请求-响应循环** — 接收 `ImageDecodeRequest`（request_id + mime + 字节）→ 解码 → 返回 `ImageDecodeResult`（request_id + RGBA 像素 + 尺寸，或错误）
- **通道断开退出** — 检测到管道关闭即退出，生命周期随宿主

## 使用示例

`zero-image-decoder` 由渲染进程自动 spawn，不面向终端用户直接运行：

```bash
# 手动启动仅用于调试（正常由 zero-renderer 经 stdin/stdout 管道 spawn）
cargo run --bin zero-image-decoder -- --type=image-decoder --instance-id=1
```

## 部署要求

由 `zero-renderer` 固定使用；解码器不可用时资源加载失败，不会回退到 renderer 进程内解码。
webview 按以下顺序定位本二进制（与 zero-renderer / zero-compositor 的发现模式一致）：

1. 环境变量 `ZW_IMAGE_DECODER_BIN`
2. macOS 主应用中的 `ZeroBrowser Helper (Image Decoder).app`
3. `current_exe`（即 zero-renderer）所在目录
4. 测试二进制目录上溯（`target/debug/deps/` → `target/debug/`）
5. `PATH` 兜底

macOS 发布包将解码器封装为独立 Helper App；Linux 与 Windows 发布产物仍将
`zero-image-decoder` 与 `zero-renderer` 放在同一目录。各平台打包脚本已内置；
缺失时资源加载失败，不会回退进程内解码（fail-closed）。

## 相关文档

- D1 目标：`docs/goal/` 图像解码独立进程切片
- 图像解码实现：`zero-render-foundation` 的 `image_cache` 模块
