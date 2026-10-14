# Canvas 2D 与页面桥

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

`zero-canvas` 实现状态、路径与软件像素；engine 的 `CanvasRegistry` 和 shim 负责页面对象，painter 收集位图给共享渲染基础设施。此域覆盖 Canvas 2D、OffscreenCanvas、text、drawImage、像素与导出，不声称 WebGL/WebGPU 已实现。

## Gotchas
- Canvas 的 recorded primitives、u8/f32 pixel buffer、页面 ImagePrimitive 是不同阶段；API 调用成功或图元数量增加不能证明最终快照与上屏像素正确。 (`crates/canvas/src/context/types.rs`, `crates/canvas/src/context/context_impl.rs`, `crates/engine/src/paint/painter/mod.rs`)
- CSS 显示尺寸、JS 反射尺寸与 host bitmap 维度不同；排查清空、缩放或巨大画布要同时查 context resize 和 DOM host 钳制。 (`crates/canvas/src/context/context_impl.rs`, `crates/engine/src/js_dom_bridge/canvas.rs`, `crates/engine/src/js_dom_shim/part04.js`)

## Architecture
- Rust OffscreenCanvas 的 transfer 返回 ImageData 并清 bitmap，DOM Canvas 的上屏则由 registry 快照加资源注入；两条通路共用 context，但不共享完整 JS ImageBitmap 生命周期实现。 (`crates/canvas/src/context/offscreen.rs`, `crates/engine/src/js_dom_bridge/canvas.rs`, `crates/engine/src/paint/painter/mod.rs`)

## Child Knowledge Nodes
- [路径、状态、像素与 OffscreenCanvas](context-and-raster.md)：查 CTM、clip、描边、resize、像素与 transfer 行为。
- [DOM Canvas、文字、图像桥与上屏](dom-bridge.md)：查页面对象身份、字体/图像 wire、PNG 导出、污点边界和快照上屏。
