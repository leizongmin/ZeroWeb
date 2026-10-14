# 图形基础设施、字体与 Canvas

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

此域连接 engine 的页面绘制结果与 CPU/GPU 场景，覆盖共享渲染基础设施、字体加载/匹配/整形、图像解码/缓存、IPC 图元转换和 Canvas 2D。DOM/CSS 绘制顺序与布局真值在 page-core，进程调度、呈现和故障恢复在 runtime，媒体播放在 media。

### User Task Navigation
| 用户任务 | 源码入口 | 知识入口 | 验证边界 |
|---|---|---|---|
| 字体、字距、变量轴或 CJK 错误 | `FontLoader`、`TextShaper`、`GlyphCache` | render-foundation 的 fonts 索引 | 常驻 hmtx/CJK 测试；系统字体测试可跳过 |
| GPU 与 CPU 像素不一致 | `render_full_scene_gpu`、`scene_supported`、`render_full_scene` | render-foundation GPU/CPU 主题 | 生产 GPU frame 与 headless 结果须分开 |
| 多进程字体、图元字段或截图差异 | `to_render_primitives`、`PaintFonts` | paint-convert | 三个消费端资源同步分别核验 |
| 图片/SVG 有解码却无显示 | `ImageData`、`ImageCache`、`image_resource_key` | render-foundation images | 解码测试不代替页面加载/布局 |
| Canvas JS 正常但空白或导出错误 | `canvas_context_op`、`paint_canvas_element` | Canvas 索引 | pipeline 局部桥与真实页面路径分开 |

功能实跑使用现有 [zeroweb-verify](../../../zeroweb-verify/SKILL.md) 按明确范围选入口；本文仅核对源码与历史，不复制启动、驱动或清理配方，也不声称本轮运行过产品。

## Gotchas
- IPC PaintSnapshot 到 RenderPrimitives 转换不注入字体或图片，CPU/GPU 共享图元不意味着资源 namespace 共享；排查缺字、空白和截图差异需同时查转换与消费者。 (`crates/paint-convert/src/lib.rs`, `crates/paint-convert/src/fonts.rs`, `apps/browser/src/paint_ipc.rs`, `apps/webdriver/src/session.rs`)
- canvas、img 和 video 都经 ImagePrimitive 加缓存像素上屏，但 canvas 用 context ID，URL 图片/视频用解析资源哈希；不能把三者全部改成同一数字键分配规则。 (`crates/engine/src/paint/painter/mod.rs`, `crates/engine/src/paint/painter/text.rs`, `crates/engine/src/paint/helpers.rs`)

## Architecture
- 物理字体、光栅与缓存由 render-foundation 统一提供，paint-convert 管协议映射，Canvas 管路径/bitmap，engine painter 管页面视觉与资源图元；宿主决定资源注册、脏区和最终呈现。 (`crates/render-foundation/src/lib.rs`, `crates/paint-convert/src/lib.rs`, `crates/canvas/src/lib.rs`, `crates/engine/src/paint/painter/mod.rs`)
- 文本须核对 CSS face 列表、hmtx/shape 度量、真实 glyph face/index 与 raster cache，任一环节 fallback 或 namespace 换代都能让布局和像素不同源。 (`crates/layout-engine/src/font_resolution.rs`, `crates/engine/src/text_metrics.rs`, `crates/render-foundation/src/font/loader/shaping.rs`, `crates/render-foundation/src/font/cache.rs`)

## Child Knowledge Nodes
- [共享渲染基础设施](render-foundation/index.md)：字体、GPU/CPU、颜色、图片与 backing store 的公共 API 和消费者。
- [共享图元转换与字体命名空间](paint-convert.md)：IPC 字段映射、下载字体导入和三个产品消费端的差异。
- [Canvas 2D 与页面桥](canvas/index.md)：路径/像素/OffscreenCanvas 与 DOM、text、drawImage、导出和上屏。
