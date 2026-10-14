# HTML 到绘制结果的管线与资源抽取

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

排查资源已下载但不显示、静态首帧与动画不同、DOM 重建后缓存失配时读本文。`RenderPipeline` 串联文档、样式、布局和 Painter；资源抽取只产生原始 URL 与描述符，网络获取、策略检查和解码由宿主协调。

### User Task Navigation
- 外链、脚本、图片和字体发现：`crates/engine/src/pipeline/extract.rs`；宿主获取见 [页面加载契约](../runtime/page-runtime/hosts-and-loading.md)，脚本执行见 [页面事件循环](../scripting/event-loop/tasks-and-checkpoints.md)。
- 字体解析与度量注入：`RenderPipeline::set_font_resolver` / `set_font_metric_map`；见 [字体匹配](../graphics/render-foundation/fonts/loading-and-matching.md)。
- 管线回归入口：`crates/engine/src/tests/pipeline.rs`；真实像素消费者为 `tests/wpt-runner/src/reftest.rs`。验证资产见 [项目验证入口](../../../zeroweb-verify/SKILL.md)，初始化资产已在 d0f281d509de3bf4f9dcd7e8dd01dac3464a663b 提交，本文仅核对源码，未实跑。

## Gotchas
- `render_html` 与 `render_html_animated` 不是只差时间参数：静态路径有文本级 vars 预处理、伪元素文本注入、`content:url()` 尺寸修正和 filter isolate 回填；动画路径未同样调用这些步骤，不能据静态入口通过推断动画入口等价。 (`crates/engine/src/pipeline/mod.rs`)
- `incremental_render` 的面积门控只标记是否全重绘，随后仍调用完整 `render_html`；`incremental_paint` 也重新计算全量样式和布局，只在 Painter 阶段限定脏矩形。 (`crates/engine/src/pipeline/mod.rs`)
- 完整 HTML 重建会清空 persistent handle 到 NodeId 映射并使 CSS 解析缓存失效；保留旧 handle 不能跨 fresh DOM 当作仍有效的节点身份。 (`crates/engine/src/pipeline/mod.rs`)
- `extract_page_scripts_indexed` 在 JS 类型和 template inert 过滤前计数，返回序号对应全部 script 元素而非可执行脚本列表；CSP 与 currentScript 消费者必须使用同一口径。 (`crates/engine/src/pipeline/extract.rs`)
- 图片有效源目前按 src、srcset 首 URL、picture 首个 source srcset 逐级回退，不评估 source 的 media/type，也不按 DPR 选择候选；不能把发现了 URL 当作响应式选源已完成。 (`crates/engine/src/pipeline/extract.rs`)
- `extract_import_urls` 只投影顶层 Import URL，不消费其媒体条件；条件嵌套的 font-face 则经 `flatten_conditional_rules` 按 media 上下文和 supports 单源判断，两类抽取不能互相替代。 (`crates/engine/src/pipeline/extract.rs`)

## Architecture
- 样式表向量先放首个 color-scheme meta 的合成规则，再放外部 CSS 参数，最后按文档序放 style 块，使 author CSS 覆盖 meta 提示；style 文本去 CDATA 后，仅 XML 内容模式额外解码字符实体。 (`crates/engine/src/pipeline/mod.rs`)
- 静态动画首帧先注册 keyframes，再在克隆时钟上以 t=0 求值并丢弃事件；真实时钟不被首帧污染，宿主推进仍由 `tick_animation_clock` 负责。 (`crates/engine/src/pipeline/mod.rs`)
- filter isolate 图元先由 Painter 收集，pipeline 离屏生成 straight-alpha 像素、逐步过滤后以同 key 放入 `canvas_images`；宿主必须把这些 RGBA 插入 ImageCache，占位 ImagePrimitive 本身不携带像素。 (`crates/engine/src/pipeline/mod.rs`, `tests/wpt-runner/src/reftest.rs`)

## Decisions
- 外链 CSS 解析返回原始 href，WebView 按文档基准解析并先做 CSP、混合内容检查；单个获取失败不阻断整页，engine 不在资源抽取函数内发起网络请求。 (`crates/engine/src/pipeline/extract.rs`, `crates/webview/src/webview.rs`)

## Patterns
- ResourcePreloader 按原始 URL 去重，仅高优先级条目覆盖旧项；待加载列表取 Pending 状态按优先级排序，同优先级来自 HashMap，不保证文档序，fetchpriority=low 不能降级 Style。 (`crates/engine/src/preload.rs`)
- 文本、布局和绘制要共用宿主字体输入：reftest 在 render 前设置 resolver、line metric map 和测量上下文，renderer 也向 WebView 注入 line metric map；只提供字形 loader 不足以保证布局与绘制一致。 (`tests/wpt-runner/src/reftest.rs`, `apps/renderer/src/runtime.rs`)
