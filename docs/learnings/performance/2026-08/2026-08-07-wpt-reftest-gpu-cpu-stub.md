---
date: 2026-08-07
modules: tests/wpt-runner/src/reftest.rs（render_to_framebuffer_gpu_with_base）, tests/wpt-runner/src/main.rs（effective_jobs）, crates/render-foundation/src/gpu/renderer/mod.rs（GpuRenderer / GPU_CREATE_MUTEX）
---

# reftest 的 --gpu 路径是 CPU 回退 stub（非真 GPU，且曾是 jobs=1 footgun）

**触发**：WPT reftest 耗时调研（杠杆3：「GPU 渲染并行化路径」）

## 问题描述

调研 reftest 提速时，原假设「`--gpu` 走 `GpuRenderer` 光栅、并行化即可拿 10-50× 加速」。
核查代码证伪：reftest 的 GPU 路径**根本不调用 `GpuRenderer`**，是无条件 CPU 回退 stub；
而 `effective_jobs` 还曾对 `--gpu` 强制 `jobs=1`。即 `--gpu` 在 reftest 下做的是**与 CPU
完全相同的软件光栅**，却被串行化，相对默认（min(CPU-1,8) 并行）慢约 5-6×，纯 footgun。

## 根因分析

```rust
// reftest.rs —— stub，无条件回退 CPU
pub fn render_to_framebuffer_gpu_with_base(...) -> FrameBuffer {
    // GPU 渲染路径暂时回退到 CPU（GPU 路径不支持全量图元 + 图片加载）
    render_to_framebuffer_with_base(html, css, config, base_dir)
}
```

- `run_reftest_gpu[_with_base]` → `render_to_framebuffer_gpu_with_base` → CPU `render_to_framebuffer_with_base`。
- `GpuRenderer`（真实 GPU，window 模式 + headless 单测在用）从未接入 reftest framebuffer 路径。
  原因：GPU 路径尚不支持全量 13 种图元 + ImageCache 图片加载。
- `effective_jobs` 旧实现 `if options.use_gpu { 1 } else { default_parallel_jobs() }`：
  对一个**实际跑 CPU 的 stub** 强制单线程，故 `--gpu` ≈ 6× 慢且零收益。
- `--gpu` 不在任何 Makefile / CI / script 的 reftest target 里（仅手动 flag），所以长期未被注意。

`GPU_CREATE_MUTEX`（`gpu/renderer/mod.rs:29`）只**序列化 Instance/Adapter/Device 创建**
（防并发创建 SIGSEGV），不限制创建后按 device 独立渲染——所以真 GPU 路径并行化在技术上可行，
前提是先把 `GpuRenderer` 接进来。

## 解决方案（本轮：诚实小修 + 文档化，非真 GPU 接入）

1. `effective_jobs` 移除 `--gpu` 的 `jobs=1` 强制，改走默认 `default_parallel_jobs()`：
   `--gpu` 不再 6× 慢（虽仍无加速）。
2. 在 `render_to_framebuffer_gpu_with_base` 加醒目 doc，标明 stub 现状 + 真 GPU 接入所需工作
   （全图元 + ImageCache + glyph atlas + 按 `GPU_CREATE_MUTEX` 设计 device 复用/并行度）。
3. `--jobs` 帮助文本去掉「GPU 1」，新增 `--gpu` 帮助行注明「currently a CPU-fallback stub」。

**未做（明确超 scope）**：把 `GpuRenderer` 真正接入 reftest（全图元 + 图片加载 + device 复用）。
这是独立大工程，且本地 WSL2 无 GPU 后端不可实测（依赖 CI 的 Vulkan/lavapipe）。真 GPU 加速
落地前，reftest 的实际瓶颈仍是 CPU 软光栅——见杠杆4。

## 如何复用 / 后续

- 若后续要推进真 GPU reftest：从 `render_to_framebuffer_gpu_with_base` 入手接 `GpuRenderer::new_headless`，
  优先做 device 复用（每 job 一个持久 GpuRenderer，而非每帧新建——当前 stub 即使补齐图元也会因
  每帧重建 device 而被创建开销吃掉收益）。
- 判断某 reftest 是否真用 GPU：`grep GpuRenderer tests/wpt-runner/src/reftest.rs`，应在
  `render_to_framebuffer_gpu_with_base` 内命中；命中前都是 stub。
