---
date: 2026-08-16
modules: apps/browser（app_platform.rs / process_backend.rs / tab_snapshot.rs）, crates/render-foundation（GPU import blit）
---

# dma-buf 导入偏移硬编码 (0,0) 导致页面覆盖浏览器 chrome

## 问题描述

Linux 上 `make browser`（GPU 渲染 + compositor）启动后，页面内容（webview）从窗口
(0,0) 起绘制，覆盖标签栏和地址栏；Windows/macOS 上布局正常。

## 根因分析

页面显示的路径分两种：

1. **RGBA 位图路径**（Windows/macOS 唯一路径，Linux 回退路径）：
   `compositor_frame_primitives` 按 `x_offset = viewport_x - scroll.x`、
   `y_offset = viewport_y - scroll.y` 把页面位图画到内容区原点，正确。
2. **dma-buf 导入路径**（Linux GPU 专属，`ZW_BROWSER_GPU_DMABUF_IMPORT` 默认开）：
   compositor 把页面光栅结果导出为 dma-buf fd，Browser 经
   `apply_compositor_dmabuf_import` 导入 GPU 纹理后，`set_compositor_import` 的
   `dst_x/dst_y` 在 `process_backend.rs` 提交 pending 时被**硬编码为 (0,0)**
   （16d6a42e 引入，占位值从未接上内容区原点）。

   `draw_compositor_import_pass` 按 `(dst_x, dst_y) + 纹理尺寸` 直接画到窗口，
   import blit 位于 fills 之后、chrome 之上 → 内容区尺寸的页面纹理从窗口左上角
   覆盖整个 chrome。滚动非零时该纹理还是过期未滚动帧，与本地图元平移层叠加成
   「固定首屏 + 滚动内容」双图层（extra 层有 page_scrolled 回退，import 层没有）。

为什么所有自动化都漏掉了：GPU smoke / gui-smoke 经 headless `render_full_scene_gpu_capture`
读回，走 **RGBA 影子**（`allow_gpu_direct_shadow=true` 时 `compositor_frame_primitives`
按正确偏移绘制），真实窗口 present 路径从未被像素断言覆盖。

## 解决方案

- `apply_compositor_dmabuf_import` 改为按 `page_content_rect_for` 计算内容区原点，
  `dst = (content_x - scroll.x, content_y - scroll.y)`，与 RGBA 路径同源；
  `scroll_transform_enabled` 时滚动已烘焙进位图（回读 scroll=0），按原点绘制。
- 滚动非零（transform 关闭）时 `clear_compositor_import`——位图仅覆盖未滚动视口，
  平移会露白，页面内容由 last_render 全文档图元平移路径绘制（与 fills/glyphs、
  extra 层的 page_scrolled 回退一致）。
- 移除 `CompositorDmabufPending` 中失效的 `dst_x/dst_y` 字段。

## 如何避免

- **新 GPU/合成路径必须先做真实窗口像素验证**，不能只依赖 headless capture——
  capture 路径与窗口 present 路径可能走不同数据流（本例 capture 用影子、窗口用 import）。
- dma-buf 导入类路径的偏移/尺寸必须与既有 RGBA 位图路径共用同一几何来源
  （`page_content_rect_for` + 本地滚动状态），并在 render-foundation 层补
  `set_compositor_import` dst 偏移的放置回归测试
  （`parity_compositor_import_respects_dst_offset`）。
