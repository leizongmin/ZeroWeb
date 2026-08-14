# ZeroWeb Compositor (`zero-compositor`)

> 合成器进程（C2）— 接收渲染进程的图元帧，BackingStore 双缓冲管理

## 概述

`ZeroWeb Compositor` (`zero-compositor`) 是 ZeroWeb 多进程架构中的合成器独立进程（C2 骨架），对照 Ladybird 2026-05 合成器独立进程设计（调研报告 §3.3/§3.4）：合成与 backing store 管理从渲染进程移出。渲染进程把页面绘制快照（`CompositorFrame` / `PaintSnapshotParams`）经 IPC 送达本进程，本进程完成光栅化与双缓冲管理，供显示消费方读取。

详见 `docs/goal/archive/compositor-process-rfc-2026-08-07.md`（已实施归档）。

## 主要功能

- **stdio 管道 + bincode IPC** — 与 image-decoder 同款的管道传输与 bincode 序列化（`zero-protocol`），零网络依赖
- **图元帧接收** — 接收 `CompositorFrame`（PaintSnapshotParams 图元快照），按 navigation epoch + frame id 序列接受新帧、丢弃过期帧
- **BackingStore 双缓冲** — `BackingStoreManager` 双缓冲：写 back → swap → 保留 front，供显示消费方读取
- **线程化光栅化** — `RenderingThread` 独立线程执行光栅化（可开关），字体与字形缓存由进程共享
- **GPU 光栅化（C3）** — Linux 默认开启 headless wgpu 上下文在合成器进程内光栅化（对照 Ladybird GPU 隔离），`ZW_COMPOSITOR_GPU=0` 禁用；初始化失败 / GPU 不可用回退 CPU
- **每 surface 独立状态** — 每个页面 surface 独立维护帧序列与双缓冲；窗口 UI surface 与 dma-buf 帧交付支持（RFC 4.3-S5）
- **进程沙箱** — 早期 seccomp + 初始化后 Landlock 约束（Linux）

## 使用示例

`zero-compositor` 由浏览器主进程通过 stdin/stdout 管道 spawn（零协议 IPC），不面向终端用户直接运行：

```bash
# 由 zero-browser 自动启动；手动运行仅用于调试
cargo run --bin zero-compositor
```

启用 / 禁用合成器内 GPU 光栅化：

```bash
ZW_COMPOSITOR_GPU=1 cargo run --bin zero-browser   # 默认开启
ZW_COMPOSITOR_GPU=0 cargo run --bin zero-browser   # 强制 CPU 光栅化
```
