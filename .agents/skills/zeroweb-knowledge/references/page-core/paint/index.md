# 页面绘制、合成与命中几何

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 LayoutBox 与 computed style 到页面图元的转换、SVG 位图/过滤隔离、动画求值与 DOM 几何。页面绘制顺序在 engine，物理字体与 GPU/CPU 光栅化在 [图形基础设施](../../graphics/index.md)，跨进程帧呈现在 [运行时](../../runtime/index.md)。关键词：Painter、draw_order、FilterIsolate、AnimationClock、HitTestCache、RectBridge。

### User Task Navigation
- 内容盖字、负 z-index、背景或控件异常：先读 painter，再查 `crates/engine/src/paint/painter/mod.rs` 和 `text.rs`。
- SVG 空形状、过滤链丢像素：读 SVG 主题，沿 `svg_filter_chain.rs` 到 `pipeline/mod.rs` 的资源回填。
- 点击穿透、动画与 rect 不一致：读动画与命中主题，核对 `hit_test.rs` 以及宿主刷新快照的时机。
- 验证源码入口为 `crates/engine/src/paint/tests/css_remaining.rs`、动画/命中模块测试和 `tests/wpt-runner/src/reftest.rs`；本轮未实跑，源码断言不代表生产 GPU 一致性。

## Gotchas
- PrimitiveCounts 不包含全部后处理窗口，判断空子树还须检查 filter/blend 与 backdrop 输出；错误发射全场景 transform 会移动或清掉已绘祖先背景。 (`crates/engine/src/paint/painter/mod.rs`)
- promote_compositing_layers 是提升元数据和排序近似，不是 Painter 的 stacking scope 等价实现，也不证明宿主已创建 GPU 合成层。 (`crates/engine/src/composite/mod.rs`, `crates/engine/src/paint/painter/mod.rs`)
- 静态 render_html 会处理 filter isolate 并回填图片，而其他渲染入口未必走同一接线；只测单独 Painter 或动画入口不能证明静态整页效果路径。 (`crates/engine/src/pipeline/mod.rs`)

## Architecture
- 混合图元的 draw_order 保留跨类型语义，不能按 text/image/fill 各向量重新归类绘制；后处理新增图元窗口也以原顺序为边界。 (`crates/engine/src/paint/painter/mod.rs`, `crates/engine/src/paint/mod.rs`)
- 布局几何、inline 上报矩形和最终绘制位置并非同一字段；命中与 rect 必须消费正确上报值，但不能修改递归坐标累积锚。 (`crates/engine/src/hit_test.rs`, `crates/engine/src/rect_bridge.rs`)
- AnimationClock 输出计算样式增量，事件派发与实际刷新在宿主；首帧的 t=0 克隆求值不推进真实时钟，不能当作动画已经开始播放。 (`crates/engine/src/animation.rs`, `crates/engine/src/pipeline/mod.rs`)

## Child Knowledge Nodes
- [绘制顺序、文本、控件与视觉效果](painter.md) — 层叠上下文、背景传播、文本染色、裁剪和控件装饰。
- [SVG、过滤链与资源污点](svg-and-filters.md) — SVG 源级修正、过滤输入隐私限制和离屏图片回填。
- [动画、过渡、命中与 DOM 几何](animation-and-hit-testing.md) — 插值、事件消费、布局快照、指针穿透和元素矩形。
