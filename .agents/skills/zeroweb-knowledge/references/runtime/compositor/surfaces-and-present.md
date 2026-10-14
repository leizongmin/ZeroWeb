# surface、frame、滚动与最终呈现

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次未运行窗口呈现。 -->

## Module Structure

`run_role` 管理页面 surface、UI surface 和窗口元数据；页面帧通过 back buffer 光栅后 swap，browser 经 compositor_client 拉取页面或合成 present。像素算法及字体转换归 graphics。

## Gotchas
- 已存在 surface 只接受更高 epoch 或同 epoch 更高 frame_id；旧帧返回当前序号回执但不重光栅，不能把收到 CompositorFrameResult 当作新帧提交证明。 (`apps/compositor/src/lib.rs`)
- paint 视口是 CSS 像素，backing 是物理像素，滚动传入值还要除 device scale 转成 CSS 变换；重复缩放会使页面与窗口坐标分叉。 (`apps/compositor/src/lib.rs`, `apps/compositor/src/rasterize.rs`, `apps/compositor/src/scroll_transform.rs`)
- 滚动不能只平移首屏 front buffer：它不包含新露出的文档区域，compositor 保留 paint 并从图元坐标重绘可见区。 (`apps/compositor/src/scroll_transform.rs`, `apps/compositor/src/lib.rs`)
- `present_authoritative` 当前按是否登记 window_surface 设置；这个协议位不是独立的 OS 窗口扫描输出证据，不应仅凭它宣称 compositor 已完成 native present。 (`apps/compositor/src/lib.rs`, `crates/protocol/src/message.rs`)

## Architecture
- 每 surface 独立保存 epoch/frame、backing、scroll、paint、image cache 和 PaintFonts，字体/glyph/GPU renderer 则有进程共享状态；切换 surface 必须切换字体 namespace。 (`apps/compositor/src/lib.rs`, `apps/compositor/src/font_resources.rs`)
- 导航 epoch 改变清 surface 图片与字体，并清共享 GPU image texture cache；共享 cache 清理可能引起其他 surface 重传开销，但不能复用上一文档局部字体 ID。 (`apps/compositor/src/lib.rs`)
- scroll transform 把矩形、glyph、path、clip、gradient 坐标向负滚动量平移，并把 dirty rect 重置为整个 viewport；文档坐标的局部脏区不能直接复用到滚动后视口。 (`apps/compositor/src/scroll_transform.rs`)
- Get 帧时若 scroll 与上次 rasterized scroll 不同，采用保存 paint 全量 CPU 重绘再 swap；回读同时携带已烘焙偏移，browser 用它计算尚在飞行的本地滚动差值。 (`apps/compositor/src/lib.rs`, `apps/browser/src/compositor_client.rs`)

## Patterns
- 部分脏区路径仅在 scroll transform 关闭且 DisplayList 非全视口时启用，先 copy_front_to_back，再修改脏区；不能拿未初始化 back buffer 做局部重绘。 (`apps/compositor/src/lib.rs`, `apps/compositor/src/gpu_raster.rs`)
- present 按白底、页面 opaque、UI src-over 合成，页面与 UI 任一没有有效像素时返回空帧；UI 半透明覆盖语义不同于页面 backing 的不透明 blit。 (`apps/compositor/src/present.rs`, `apps/compositor/src/lib.rs`)
- browser 的页面 completed cache 按 surface/epoch/frame 筛选，而 present cache 按 page surface 存最新结果；两条回读通路不能共用同一个“帧新鲜”判断。 (`apps/browser/src/compositor_client.rs`)
