# zero-ui-render

通用 UI SDK 的渲染/场景抽象层。定义自己的 `Scene` / `RenderNode` / `PaintCtx` / `ClipStack` / `hit_test` 模型，**不直接依赖 render-foundation GPU 后端**。

## 架构位置

```
ui/render ←── ui/runtime（WidgetHost paint 输出到此层）
     │
     ├── zero-text-foundation（TextBlob 文本单元）
     └── ui/adapters/render-foundation 桥接（RenderBackend impl）
```

ui/render 通过 `RenderBackend` trait 解耦具体后端；当前生产后端是 `ui/adapters/render-foundation` → render-foundation。

## 核心类型

| 类型 | 职责 |
|------|------|
| `Scene` | 绘制命令列表 + clip state；`Scene::translated` 局部→绝对坐标 |
| `SceneRecorder` | Builder 模式收集绘制命令（`draw_text_blob` / `fill_rect` 等） |
| `RenderNode` | 单个绘制节点（clip + 姿态 + primitive） |
| `RenderPrimitive` | `Fill`, `Stroke`, `TextBlob`, `Image`, `ExternalSurface` 等图元 |
| `ClipStack` | 流式裁剪栈（None→ZERO 不可见整支裁断）；sticky-zero 传播 |
| `PaintCtx` | 绘制上下文（clip + tokens + recorder） |
| `RenderBackend` trait | `paint_scene` 消费 Scene，将其派发到具体后端 |
| `Layer` | 合成层数据 |
| `hit_test` | 后序 topmost-wins 命中检测 |

## 设计约束

1. 不依赖 `render-foundation`（TBD-2：通过 `RenderBackend` trait + adapter 模式解耦）
2. 不依赖浏览器业务 crate（DC-1）
3. clip 语义：`Stateful Clip`——clip 入栈后消失（非破坏性），但实际由 bridge 后端 CPU 侧 `Rect::intersection` 实现

## 与 render-foundation 的边界

```
ui/render (Scene + RenderBackend trait)
     ↓ paint_scene
ui/adapters/render-foundation (RenderFoundationBackend impl RenderBackend)
     ↓ RenderPrimitives
render-foundation (GPU/CPU 管线)
```

## 测试

- `cargo test -p zero-ui-render` — 14 测
- 覆盖：Scene translate / ClipStack sticky-zero / hit_test 坐标空间 / paint_scene 派发

## 深度审查

2026-07-03 全 crate 审查，0 behavior bug。3 处安全加固（clip 语义 docstring + 回归测 + hit_test 注释修正）。
