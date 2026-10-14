# SVG、过滤链与资源污点

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位内联 SVG 空形状仍出现滤镜色块、位移滤镜隐私规则、CSS `filter:url()` 丢内容或重复过滤。内联 SVG 是 DOM 序列化后交给 resvg 的位图路径；HTML 子树过滤则由 Painter 收集隔离图元，pipeline 离屏处理并回填图片资源。这里的 taint 是过滤器像素输入限制，不等同 Canvas 的来源清洁状态。

### User Task Navigation
- SVG 序列化与 CSS transform 注入：`crates/engine/src/paint/painter/mod.rs`；源级修正：`svg_shape_disable.rs`、`svg_filter_taint.rs`。
- 过滤链入口：`crates/engine/src/paint/svg_filter_chain.rs`、`crates/engine/src/paint/painter/effects.rs`；资源回填：`crates/engine/src/pipeline/mod.rs`。
- 常驻断言在上述源级模块的测试及 effects 测试；最终资源消费者见 `tests/wpt-runner/src/reftest.rs`。本轮仅核对代码，没有运行产品或测试。

## Gotchas
- 空形状禁用必须在 resvg 渲染前移除整个元素，包括非自闭合的闭标签，否则零宽 rect 的 feFlood 仍可能产生色块；空 g 不是形状，不能一起删除。 (`crates/engine/src/paint/svg_shape_disable.rs`, `crates/engine/src/paint/painter/mod.rs`)
- 尺寸禁用只识别显式零 width/height/r/rx/ry；path 缺 d、空 d 或 d=none 与 polygon/polyline 缺或空 points 另行禁用，这不是 CSS 几何属性计算的完整实现。 (`crates/engine/src/paint/svg_shape_disable.rs`)
- feDisplacementMap 是否改成恒等 feOffset 只由位移映射 in2 的 taint 决定；主输入 in 为 SourceGraphic 不足以禁用位移，改写时还须保留 in、result 并消耗原闭标签。 (`crates/engine/src/paint/svg_filter_taint.rs`)
- in 与 in2 的缺省独立求值：首原语取 SourceGraphic，后续取隐式前项；不能把缺省 in2 写成显式 in 的副本。当前命名结果后的隐式 taint 标志被清零，不能据模块注释推断所有命名链都完全符合规范。 (`crates/engine/src/paint/svg_filter_taint.rs`)
- feFlood 常规色的输出不继承无关输入 taint，currentColor 及包含它的颜色函数才触发自 taint；feImage 因源级无法判 CORS 被保守视为 tainted，不表示已执行真实网络权限判定。 (`crates/engine/src/paint/svg_filter_taint.rs`)
- 空 filter 引用按恒等步骤跳过；把它交给 resvg 当作透明输出会清掉原图。 (`crates/engine/src/paint/svg_filter_chain.rs`, `crates/engine/src/paint/painter/mod.rs`)
- 隔离收集需要渲染侧 FontLoader，并排除根元素、特定 DropShadow/backdrop-filter 情形及 transform/will-change:transform 后代；自身 transform 不触发后代逃逸门，仍在过滤后作用于占位图。 (`crates/engine/src/paint/painter/mod.rs`, `crates/engine/src/paint/svg_filter_chain.rs`)

## Architecture
- FilterIsolate 保存独立子树图元、真实页面 region、声明序 FilterStep 和图片 key；function 与 url 混合链必须按声明顺序逐步作用于同一离屏结果。 (`crates/engine/src/paint/svg_filter_chain.rs`, `crates/engine/src/pipeline/mod.rs`)
- 隔离根旁路绘制时抑制其自身 FilterPrimitive 发射，过滤交给 pipeline 唯一消费；否则一条链会在子树和结果图上重复应用。 (`crates/engine/src/paint/painter/mod.rs`)
- 隔离 region 在主遍 skip 位点按真实绝对坐标定稿，不能只使用收集期的粗坐标；回填 canvas_images 后宿主必须将同 key RGBA 放入 ImageCache。 (`crates/engine/src/paint/painter/mod.rs`, `crates/engine/src/pipeline/mod.rs`, `tests/wpt-runner/src/reftest.rs`)
- 内联 SVG 的 individual translate、rotate、scale 与 transform 按固定顺序合成到序列化文本；RotateZ 能发射二维 rotate，其他三维项仍有跳过边界，HTML 图元变换不能重复施加。 (`crates/engine/src/paint/painter/mod.rs`)

## Decisions
- 源级 taint 和空形状修正分别受 ZW_SVG_TAINT、ZW_SVG_SHAPE_DISABLE 控制，均仅在值为 0 时关闭；它们弥补 resvg 消费行为，不是替代 DOM/CSS 解析器。 (`crates/engine/src/paint/svg_filter_taint.rs`, `crates/engine/src/paint/svg_shape_disable.rs`)
- 过滤包装 SVG 会补祖先的 color-interpolation-filters 声明，无光源 lighting 原语改成透明 flood；相对 feImage 只在已有文档目录下尝试文件内联，不会自行抓取 HTTP URL。 (`crates/engine/src/paint/svg_filter_chain.rs`)
- filter() 图像函数不能从解析成功推断全部图片路径已应用：effects 的 URL 内层、mask 像素过滤及 border 的深层嵌套组合仍有显式 FIXME。 (`crates/engine/src/paint/painter/effects.rs`, `crates/engine/src/paint/painter/border.rs`)
