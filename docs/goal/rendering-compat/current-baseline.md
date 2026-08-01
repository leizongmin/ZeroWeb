# 当前能力/缺口基线

> **说明**：本文档是从 `rendering-compat.md` 主文档移出的详细能力/缺口大表。主文档中只保留简要引用，详情见此处。

截至 2026-07-16，项目渲染兼容性现状（活跃状态以本节和 `docs/goal/rendering-compat/master.md` 顶部裁决包为准）：

## 已有能力

| 领域 | 状态 | 详情 |
|------|------|------|
| 渲染管线（Parse → Layout → Paint） | ✅ 全链路贯通 | HTML → CSS → Style → Layout → Paint 生成完整 `RenderPrimitives` |
| CSS 属性解析 | ✅ 100+ 属性 | box model、flexbox、grid、border、background、transform、animation、transition 等 |
| Flexbox/Grid 布局 | ✅ 基于 taffy | 所有子属性均已接入 |
| Block/Inline 布局 | ✅ 基础可用 | Block via taffy, Inline via 自建 InlineFormattingContext |
| Table 布局 | ✅ 已实现 | 表格网格构建、auto table layout、colspan、border-spacing、匿名表格盒 |
| Multi-column 布局 | ✅ 已实现 | column-count/column-width、column-gap、shortest-column-first 分布策略 |
| Quirks mode | ✅ 已实现（实质） | CSS parser（mode-gated）+ style system（3 quirks 预烘焙）两层活跃；layout-engine 无独立 quirks 层（由 style-system 预烘焙覆盖，架构合理）。wpt-data quirks 用例全过（R248 实证） |
| 文字排版 | ✅ 已集成 | rustybuzz OpenType shaping + unicode-bidi BiDi 算法 + CJK line-breaking |
| Paint 系统 | ✅ 13 种图元 | 填充、圆角矩形、路径、线段、渐变、阴影、图片、文字、滤镜、混合模式、变换、裁剪 |
| CPU 软件渲染 | ✅ **已实现（M7）** | fills/rounded/glyphs + gradient/shadow/image/stroke/path_fill/path_stroke/filter/blend（cpu/ 下各模块，附单测） |
| GPU 渲染 | ✅ **已实现（M7）** | wgpu+WGSL `draw_*_pass` + `collect_*_vertices` + mesh 管线（rounded/gradient/image/shadow/stroke/path/filter），非 CPU passthrough，附 `test_gpu_full_scene_*` |
| 浏览器图元消费 | ✅ **已实现（M7）** | `append_webview_primitives()`（`app_render_primitives.rs:17`，字段迭代 ~31-487）遍历全 13 字段无丢弃 |
| Margin 折叠 | ✅ 已实现（taffy 0.7 CollapsibleMarginSet；R323 实测 6 探针 case + 5 reftest 全过） | 块级元素间距与主流浏览器一致 |
| BFC（margin 隔离部分） | ✅ 已实现（overflow:hidden/flex/grid 等 BFC 的子元素 margin 不与父折叠；R323 实测） | margin 折叠隔离正确；浮动包含（float containment）部分未单独验证 |
| 滚动容器 | ⚠️ 简化处理 | 无真正滚动容器，浏览器层手动偏移 |

## 已知关键缺口

| 缺口 | 影响范围 | 严重性 | 当前状态 |
|------|----------|--------|----------|
| **渲染器图元覆盖** | **所有视觉输出** | ✅ **已实现（M7）** | CPU（cpu/gradient.rs+shadow.rs+image+stroke.rs+effects.rs filter/blend）+ GPU（gpu/renderer/mod.rs draw_gradient/image/rounded_rect_pass + collect_shadow/stroke/path_fill/path_stroke/color_filter/blur_filter + mesh）均已实现全 13 种图元，附单测（test_gpu_full_scene_gradient/shadow/stroke 等）。**注**：granular DC-8/9 各项的 framebuffer 像素断言 rigor 仍待逐项复核。原「CPU 仅 3 种 / GPU 仅 2 种 / 全部无法渲染」描述已过时（pre-M7） |
| **浏览器图元消费** | **所有视觉输出** | ✅ **已实现（M7）** | `append_webview_primitives()`（`app_render_primitives.rs:17`）遍历 `RenderPrimitives` 全 13 字段（迭代 ~31-487：fills/glyphs/shadows/rounded_rects/gradients/images/strokes/path_fills/path_strokes/clips/transforms/filters/blend_modes）无静默丢弃。原「仅 fills+glyphs，11 种丢弃」描述已过时（pre-M7） |
| **Inline formatting 所有权分裂** | **静态页面基础排版** | **P1-严重** | inline/inline-block 在 taffy 中映射为 block，同时 IFC 又通过 `text_content()` 收集 inline 子树文本；父容器和子 inline 盒可能重复或错位绘制文本。`welcome.html` 中 `ZeroBrowser`、card/link/shortcut 文本串联是该类缺口的产品可见症状 |
| **Layout/Paint IFC 双路径** | **文本布局与 glyph 输出一致性** | **P1-严重** | layout 阶段和 paint 阶段不是同一份 IFC 结果；paint 二次运行 IFC 时 style map、float exclusion、container width 可能不同，导致 box 背景位置与 glyph 位置不一致 |
| ~~**外部样式表加载缺失**~~ | **真实静态网页 CSS** | ✅ **已贯通（R213）** | ✅ **已修复**：`fetch_url`（webview.rs:515）各成功路径现均 `load_html(&html, Some(&external_css))`（非 None；调用点 :561/586/600/611/645）；`prepare_page_subresources`（:263）→ `resolve_external_css`（:363）经 extract_stylesheet_hrefs 提取 `<link rel="stylesheet">` + base URL 解析 + 逐个 HTTP 抓取 + 合并注入级联，抓取/解码失败记 `tracing::warn!`（:386）不阻断（宽松降级）；R213 测试 test_fetch_url_loads_external_stylesheet + ..._missing_does_not_break 覆盖。~~原（已过时）~~： `WebView::fetch_url()` 三条成功路径都会调用 `load_html(&html, None)`；`RenderPipeline::collect_stylesheets()` 只收调用方传入 CSS 和文档内 `<style>`，不抓取 `<link rel="stylesheet">`。morning.work 文章页依赖外链 CSS，当前会静默退化为仅内联样式 |
| ~~**图片子资源/ImageCache 未贯通**~~ | **Logo/图片密集静态页面** | ✅ **已贯通（R318 实测）** | `<img>` paint 生成 `ImagePrimitive`；`WebView::fetch_image_subresources`（webview.rs:402）在 `fetch_url`（:515）各导航路径抓取 + 解码 `<img src>`（PNG/JPEG 魔数 + SVG via resvg/tiny-skia），写入 `image_cache`；`app_platform.rs` render_cpu/render_gpu/render_frame 三处传 `Some(&mut webview.image_cache())`（非 None）。**R318 实测**：WinterTC 首页 header logo + 13 个参与方 SVG/PNG logo（alibaba/bytedance/cloudflare/deno/fastly/igalia/netlify/nodejs/shopify/suborbital/vercel/azion/matrix）全部正确渲染（非占位 glyph），产品 smoke diff=13.70%（残余为 system-ui 字体度量/line-height，非图片缺口）。原「传 None / Logo 缺失」描述已过时 |
| **浏览器层 glyph 重排** | **ZeroBrowser 产品渲染路径** | **P1-严重** | ZeroBrowser 在消费 WebView 图元前会按 baseline 对 glyph 做后处理重排；该逻辑可能破坏 engine 已经计算好的 fragment x 坐标，尤其影响 grid/flex 中同一 baseline 的不同卡片文本 |
| ~~**真实静态页面 smoke 缺失**~~ | **验收有效性** | ✅ **已建立产品 smoke 证据链** | `apps/browser/assets/welcome.html`、morning.work、WinterTC、legacy HTML、窄屏等静态页面已进入 `docs/goal/rendering-compat/evidence/product-static/` 证据链，覆盖 Chromium oracle 截图、diff-summary 和多轮根因分析；后续是扩展 viewport / 结构检查 / 回归门禁，不再表述为"没有真实静态页面 smoke" |
| ~~**Margin 折叠**~~ | CSS 2.1 布局正确性 | ✅ **已实现（R323 实测）** | taffy 0.7 `CollapsibleMarginSet` 内置；R323 探针 6 case（相邻/父子/border 阻断/负 margin/祖父嵌套/BFC 子不折叠）全过 + margin reftest 5/5 全绿。原「完全未实现」描述过时 |
| ~~**BFC**~~ | 布局隔离 | ✅ **已实现** | `establishes_bfc`（全条件：overflow/float/abspos/flow-root/flex/grid/table/multicol）接线生产；margin 隔离 R323 实测 6 探针全过；`use_bfc_float_containment` 落地 float containment。原「无 BFC 概念，overflow: hidden 不隔离浮动、不阻止 margin 折叠」描述过时 |
| ~~**替换元素**~~ | 图片/媒体渲染 | ✅ **已实现** | `<img>` 固有尺寸（HTML 属性 + SVG data URI + 解码固有尺寸三来源）已实现；R325 修 CSS §10 两侧显式尺寸不强制固有宽高比（旧 `<img style="width:200px;height:50px">` 被拉成 200×200）；`compute_object_fit_rect` 全 5 值；R318 图片数据端到端贯通。原「无固有尺寸计算，图片无法正确显示」描述过时 |
| **滚动容器** | 页面滚动 | P1-严重 | overflow: scroll/auto 无真正滚动，长页面无法正确浏览 |
| ~~Float 布局~~ | CSS 2.1 核心功能 | ✅ **核心已实现（R895 / DC-11）** | float:left/right 定位、clear、BFC float containment、inline exclusion 已实测；残余按具体 CSS2 float 边缘 case 追踪，不再表述为"仅有 inline context exclusion / clear 不完整" |
| ~~Position: fixed~~ | 视口定位 | ✅ **R324 已修复** | `adjust_fixed_to_viewport` 改为扣除累积祖先偏移（旧「加上」仅 parent_offset=0 时正确）；fixed 在有偏移 positioned 祖先内也视口相对，与 R98 absolute-viewport 约定一致。新单测 + 8 旧单测更新 + 全量 reftest 零回归 |
| Position: sticky | 滚动吸附 | P2-中等 | 需 host layer 动态调整，未完整实现 |
| ~~text-shadow~~ | 文字效果 | ✅ **已实现（DC-12）** | paint 阶段已生成并渲染 text-shadow 图元；后续只按具体 reftest 回归处理 |
| ~~多背景图层~~ | 视觉丰富度 | ✅ **已实现** | effects.rs:134 全图层 `.rev()` 叠加（原「仅第一个」已过时） |
| ~~重复渐变~~ | 视觉丰富度 | ✅ **已实现** | cpu/gradient.rs:28 `if gradient.repeating`（原「未实现」已过时） |
| ~~clip-path~~ | CSS 裁剪 | ✅ 已实现（M9） | painter/effects_indicators.rs + helpers.rs 全形状裁剪（原「仅生成指示器」已过时） |
| ~~backdrop-filter~~ | 模糊背景 | ✅ 已实现（M9，R894 实测） | painter/effects.rs；blur 效果正确限定元素盒内（原「完全未实现」已过时） |
| ~~CSS mask~~ | 遮罩效果 | ✅ 已实现（M9） | painter/effects.rs 渐变蒙版裁剪 + alpha 衰减（原「完全未实现」已过时） |
| 3D transform | 3D 效果 | P3-低 | 仅 2D 支持，3D 函数忽略 |
| ~~真实 WPT reftest~~ | 验证有效性 | ✅ **已实现（R484）** | R484 完成 10/10 目录全量去子集化——从上游 WPT（`https://github.com/web-platform-tests/wpt`）`MANIFEST.json` 自动提取并导入 **~9967 个真实 reftest**（css2/CSS2/flexbox/grid/position/float/tables/multicol/text/writing-modes/fonts/text-decor 全覆盖），并建 DC-14 chromium Oracle 一致率基线 3608/9967=36.2%（chr<1%）。原 685 个 inline reftest 现仅作 smoke（不计入通过率）。原「685 inline 未用真实 WPT」描述已过时（pre-R484） |

## 测试基线

- 总测试数：~13,014，全绿（`make test`：13014 passed / 0 failed / 74 ignored，截至 R2369）
- Coverage：95.46% line, 96.94% function, 94.88% region
- Inline reftest：685 个，100% 通过（⚠️ 手写简单场景，容差过宽松，**不计入本目标通过率统计**。本目标的通过率必须基于上游真实 WPT reftest）
- **关键事实**：当前 WPT runner 是 smoke test，不证明渲染正确性。本目标的核心挑战是从"不崩溃"升级到"渲染正确"。（历史：M7 前渲染器仅支持 3/13 种图元；**M7 后 CPU/GPU 均已支持全 13 种图元**——见 Current Proven Baseline 表。当前 reftest 通过率受字体度量 / 布局结构性 plateau 限制，非图元覆盖限制。）
