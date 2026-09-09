# ZeroWeb Paint Convert (`zero-paint-convert`)

> IPC 图元快照（`PaintSnapshotParams`）→ 渲染图元（`RenderPrimitives`）公共转换层

## 概述

`ZeroWeb Paint Convert` (`zero-paint-convert`) 收敛多进程架构中「renderer 图元快照 → 渲染图元」映射的唯一实现。多进程链路上，renderer 发布 `zero-protocol` 的 `PaintSnapshotParams`（图元快照），消费方（compositor 合成器进程、browser 主进程、webdriver 截图通道）各自把它转换成 `zero-render-foundation` 的 `RenderPrimitives` 用于光栅化——2026-09-09 前该映射由 compositor `convert.rs` 与 browser `paint_ipc.rs` 双份维护，两笔渲染修复被迫同批改两处，故抽出本公共 crate（webdriver-screenshot goal M1）。

映射语义以 compositor 在用版本为基准（主链路真值）；调用方在其上组合各自的副作用（browser 写 `TabSnapshot` 缓存、compositor 进程内光栅化）。

## 主要功能

- **统一转换入口** — `to_render_primitives(params)` 按值消费 `PaintSnapshotParams`（字段逐一 move，避免 clone），输出 `RenderPrimitives`
- **全图元类型覆盖** — fill / rounded rect / gradient（含 CSS Color 4 多色彩空间插值与色相法）/ shadow / image / stroke / path fill / path stroke / clip / transform / filter / blend mode / glyph 共 13 类绘制图元逐一映射
- **字体变轴** — `IpcFontVariation` 校验（非法整体丢弃）→ `OpenTypeVariation`，glyph 按 variation 索引关联
- **字形文本源** — glyph text run 去重 intern（`run_id` 冲突文本剔除），`IpcGlyphSource` 还原为 `GlyphSource`
- **非绘制元数据** — 文本控件 caret 边界（`text_control_boundaries`）随转换透传，供 browser 输入法交互使用（CPU/GPU 光栅化不消费）
- **IPC 面缺口显式标注** — shadow 裁剪窗口 / punch-out 区域暂无 IPC 传递（恒 `None`），以注释锚定来源轮次

## 使用示例

```rust
use zero_paint_convert::to_render_primitives;
use zero_protocol::PaintSnapshotParams;

// renderer 经 IPC 收到图元快照后，转换为渲染图元供光栅化
let params: PaintSnapshotParams = /* bincode 反序列化 */;
let primitives = to_render_primitives(params);
// primitives.draw_order 中的 DrawOp 按绘制顺序引用各类图元
```

调用方按需决定传入克隆或移动：compositor 光栅化前有 scroll transform 变换，browser 写 `TabSnapshot` 缓存，webdriver 截图消费 viewport 克隆。

## 相关文档

- 协议消息与快照结构：`crates/protocol/src/paint_snapshot.rs`
- 渲染图元定义：`crates/render-foundation/src/primitive.rs`
- 落地记录：`docs/goal/archive/webdriver-screenshot/master.md`
