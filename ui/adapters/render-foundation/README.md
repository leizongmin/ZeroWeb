# zero-ui-adapter-render-foundation

render-foundation 后端桥接适配器。实现 `zero-ui-render::RenderBackend` trait，把 UI SDK 的 `Scene` 累积为 render-foundation 的 `RenderPrimitives`，经现有 GPU/CPU 管线消费。spec TBD-2 闭环。

## 架构位置

```
ui/render (Scene + RenderBackend trait)
     ↓ paint_scene
ui/adapters/render-foundation
     ↓ RenderPrimitives
render-foundation (GPU/CPU 管线)
```

## 核心类型

| 类型 | 说明 |
|------|------|
| `RenderFoundationBackend` | `RenderBackend` 的完整实现。将 UI Scene 的 fill/stroke/clip/text/external surface 转换为 render-foundation 图元 |

## 图元映射

| UI RenderPrimitive | render-foundation 输出 |
|--------------------|-----------------------|
| `Fill` (零圆角) | `FillPrimitive` |
| `Fill` (圆角) | `RoundedRectPrimitive`（四角半径映射） |
| `Stroke` | `PathStrokePrimitive`（4 角闭合路径；圆角暂忽略） |
| `Clip` | **Stateful clip**（CPU 侧 `Rect::intersection`，不 emit ClipPrimitive） |
| `draw_text` (原始字符串) | shape + raster → `ImagePrimitive`（经共享 `FontdueBackend`） |
| `draw_text_blob` | raster glyph → tinted RGBA → `ImageCache` → `ImagePrimitive` |
| `ExternalSurface` | `add_clip(rect)` + `merge_primitives`（注册 surface primitives） |

### stateful clip 设计

render-foundation 的 `apply_clip` 是**破坏性**（clear-clip-外），但 UI Scene 每 entry 调用 `apply_clip` 具有 stateful 语义。Bridge 通过 CPU 侧 `Rect::intersection(&current_clip)` 裁切待渲染图元，不向 render-foundation 传递 ClipPrimitive。这保证了 UI Scene 的布局/绘制语义在不同后端上的一致。

## 颜色转换

`zero-ui-core::Color(f32 0..1)` → `render-foundation::Color(u8 0..255)`

## 关键工具函数

| 函数 | 说明 |
|------|------|
| `merge_into_frame(source_primitives, source_cache, frame_primitives, frame_cache)` | 把 SDK chrome 的 primitives + ImageCache 合并入浏览器帧（collision-safe rekey + 13 桶 draw_order 偏移） |

## 依赖

- `zero-ui-core` / `zero-ui-render` / `zero-text-foundation` / `zero-render-foundation` / `hashbrown`
- dev-dep（全链集成测试）：`zero-browser-chrome` / `zero-ui-runtime` / `zero-browser-shell`

## 测试

- `cargo test -p zero-ui-adapter-render-foundation` — 24 测
- 覆盖：fill/stroke/clip 映射 / draw_text+draw_text_blob 光栅 / ExternalSurface 合并 / ImageCache merge+rekey / 全链集成测试（model→shell→WidgetHost→Scene→bridge→RenderPrimitives）
- 深度审查：2026-07-03 修复 glyph_cache_key FontId 位打包碰撞（font:32/glyph:16/size:16）
