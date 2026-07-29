# Done Criteria 详细进度

> **说明**：本文档是从 `rendering-compat.md` 主文档移出的 DC-1~14 完整进度详情。主文档中只保留一行状态摘要，详情见此处。

## DC-1: WPT Reftest 基础设施就位

- [ ] 能够从上游 WPT 仓库（`https://github.com/web-platform-tests/wpt`）fetch 并解析 reftest test list（**扩展**现有 `manifest.rs`，不重写）
- [ ] 解析上游 WPT MANIFEST.json 中每个 reftest 的 `fuzzy()` 元数据（maxDiff、maxPixel），并传递给像素对比引擎
- [ ] 能够用 CPU 软件渲染器对 ZeroWeb 渲染输出截图（**复用**现有 `render_scene_to_framebuffer`）
- [ ] 能够用 GPU 渲染器对 ZeroWeb 渲染输出截图
- [ ] **自动化 headless Chromium 截图**：通过 Puppeteer/Playwright 脚本自动在 headless Chromium 中渲染 reftest HTML 并截图，作为参考基线（零手动操作）
- [ ] **Viewport 对齐**：ZeroWeb 截图和 Chromium 截图在相同 viewport 尺寸下捕获（默认 800×600，可配置）
- [ ] **JS 执行支持**：Reftest harness 在截图前通过 `script-sandbox` V8 runtime 执行页面 JavaScript
- [ ] **分类容差机制**：支持按 reftest 分类设置不同像素容差阈值：
  - 布局类（不含文字渲染）：严格容差（max_diff_ratio ≤ 0.1%, max_channel_diff ≤ 2）
  - 文字类：宽松容差（max_diff_ratio ≤ 0.5%, max_channel_diff ≤ 5）
  - 优先使用 WPT fuzzy 注解的 per-test 容差，无注解时使用分类默认值
  - **容差锁定**：以上数值为硬性上限，不允许通过「实测校准」等理由放宽容差。如果文字类 reftest 因字体渲染差异导致大面积失败，应在 master.md 中记录具体原因，通过修复渲染来降低失败率，而非放宽容差
  - **禁止**设置过宽松的默认容差来掩盖真实渲染差距
- [ ] **范围外 reftest 过滤**：导入时自动过滤或标记范围外 reftest（SVG、Canvas、WebGL），维护 skip list 文件（如 `tests/wpt-runner/reftest-skip-list.txt`）。**Skip list 约束**：仅允许跳过明确不在范围内的 reftest（SVG、Canvas、WebGL、动画帧级验证等）。**不允许**跳过范围内但已知的困难 case 或预期会失败的 case。Skip list 中每一项必须有注释说明跳过原因和对应的范围外分类
- [ ] 通过率报告按 WPT 目录分类输出（文本 + JSON 格式）
- [ ] Reftest 运行可通过单一命令执行（如 `cargo run --bin wpt-reftest`）
- [ ] CI 管线中集成 reftest 运行（至少 CPU 模式）

## DC-2: CSS 2.1 核心通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css2/` 和 `css/CSS2/` 目录导入**全部**范围内 reftest（排除 skip list 中的范围外 case）
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] 覆盖：盒模型、margin 折叠、BFC、inline formatting、颜色、背景、边框、基础定位
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

## DC-3: Flexbox + Grid 通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-flexbox/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-grid/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

## DC-4: Positioning + Float + Table + Multicol 通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-position/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-float/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-tables/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-multicol/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

## DC-5: 文字排版通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-text/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-writing-modes/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-fonts/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-text-decor/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

## DC-6: Quirks Mode 完整实现

- [ ] CSS parser 在 quirks mode 下正确调整解析行为（quirky color values、quirky unitless lengths 等）
- [ ] Style system 在 quirks mode 下应用特定样式规则（如表格高度 quirks、百分比高度 quirks）
- [ ] Layout engine 在 quirks mode 下实现特定布局行为
- [ ] DOM parser 的 quirks mode 状态正确传递到 CSS parser → style system → layout engine 链路

## DC-7: 测试与质量不可退让

- [ ] 所有现有测试持续全绿（`cargo test` 零失败），包含移除 `#[ignore]` 后的全部测试
- [ ] **真实网站测试保留 `#[ignore]`**：`tests/integration/src/real_website_compat.rs` 中的真实网站兼容性测试因本地网络不稳定，保留 `#[ignore]` 标记，不计入本目标通过率统计。其余所有测试零 `#[ignore]`
- [ ] 所有新增渲染修复必须有对应单元测试覆盖
- [ ] `cargo build` 零错误、`cargo clippy` 零警告
- [ ] Reftest 通过率报告持久化到 `docs/goal/rendering-compat/evidence/` 目录
- [ ] 每轮执行的 reftest 通过率变化可追溯（有历史记录）

## DC-8: CPU 渲染器图元覆盖 100%

以下全部 13 种图元类型都必须能在 CPU 渲染器中正确渲染，不允许跳过任何一种：

- [x] `FillPrimitive` — 纯色矩形（✅ 已有）
- [x] `RoundedRectPrimitive` — 圆角矩形（✅ 已有）
- [x] `GradientPrimitive` — 线性/径向/锥形渐变渲染 ✅(cpu/gradient.rs render_gradient + get_pixel 单测)
- [x] `ShadowPrimitive` — 高斯模糊阴影渲染 ✅(cpu/shadow.rs render_shadow + get_pixel 单测)
- [x] `ImagePrimitive` — 图片解码和渲染（RGBA → framebuffer 合成） ✅(cpu/mod.rs render_image + R665 cpu_full_scene_image_solid_red 像素断言 红)
- [x] `StrokePrimitive` — 线段渲染（实线/虚线/点线，支持 LineCap） ✅(cpu/stroke.rs render_stroke + R665 cpu_full_scene_stroke_horizontal_line 像素断言 中心黑)
- [x] `PathFillPrimitive` — 路径填充（任意多边形） ✅(cpu/stroke.rs render_path_fill + R665 cpu_full_scene_path_fill_black_rect 像素断言 中心黑/外部白)
- [x] `PathStrokePrimitive` — 路径描边 ✅(cpu/stroke.rs render_path_stroke + R665 cpu_full_scene_path_stroke_closed_rect 像素断言 描边边黑/内部白)
- [x] `TransformPrimitive` — 2D 仿射变换应用到后续图元 ✅(cpu/mod.rs apply_transform_post + R666 cpu_full_scene_transform_translates_content 像素断言 tx=8 内容右移)
- [x] `ClipPrimitive` — 矩形裁剪区域 ✅(cpu/mod.rs apply_clip + R666 cpu_full_scene_clip_rect_clears_outside 像素断言 区内保留黑/区外清白)
- [x] `FilterPrimitive` — CSS 滤镜（至少 blur、brightness、contrast、opacity） ✅(cpu/effects.rs apply_filter + get_pixel 单测)
- [x] `BlendModePrimitive` — 混合模式合成 ✅(cpu/effects.rs apply_blend_mode + 单测)
- [x] `GlyphPrimitive` — 文字渲染（✅ 已有）

## DC-9: GPU 渲染器图元覆盖 100%

以下全部 13 种图元类型都必须能在 GPU 渲染器中正确渲染，不允许跳过任何一种。GPU 渲染必须使用真实的 GPU 渲染管线（wgpu + WGSL shaders），不允许将 GPU 渲染实现为对 CPU 渲染器的 passthrough 调用：

- [x] `FillPrimitive` — 纯色矩形（✅ 已有）
- [x] `RoundedRectPrimitive` — 圆角矩形 ✅(draw_rounded_rect_pass + test_gpu_full_scene_rounded_rect；原「GPU 未实现」已过时)
- [x] `GradientPrimitive` — 渐变渲染（WGSL shader 或 GPU compute） ✅(draw_gradient_pass + test_gpu_full_scene_gradient 像素断言 left=R/right=B)
- [x] `ShadowPrimitive` — 阴影渲染（高斯模糊 pass 或近似算法） ✅(collect_shadow_vertices + test_gpu_full_scene_shadow)
- [x] `ImagePrimitive` — 图片纹理采样渲染 ✅(draw_image_pass + prepare_image_resources + R663 test_gpu_full_scene_image 像素断言 红 RGBA)
- [x] `StrokePrimitive` — 线段渲染 ✅(collect_stroke_vertices + push_stroke_mesh + test_gpu_full_scene_stroke)
- [x] `PathFillPrimitive` — 路径填充 ✅(collect_path_fill_vertices + R663 test_gpu_full_scene_path_fill 像素断言 中心黑/外部白)
- [x] `PathStrokePrimitive` — 路径描边 ✅(collect_path_stroke_vertices + R663 test_gpu_full_scene_path_stroke 像素断言 描边边黑/内部白)
- [x] `TransformPrimitive` — 2D 变换（顶点变换） ✅(test_gpu_full_scene_transform_translation)
- [x] `ClipPrimitive` — 裁剪（scissor rect 或 stencil buffer） ✅(test_gpu_renderer_clip_rect_limits_rendering)
- [x] `FilterPrimitive` — CSS 滤镜（post-processing pass） ✅(test_gpu_full_scene_filter_* 8 变体像素断言)
- [x] `BlendModePrimitive` — 混合模式（blend equation 或 shader） ✅(test_render_scene_with_alpha_blending)
- [x] `GlyphPrimitive` — 文字渲染（✅ 已有，glyph atlas）

## DC-10: 浏览器图元消费完整性

- [x] ✅(M7) `append_webview_primitives()` 处理 `RenderPrimitives` 的所有 13 个字段（fills、rounded_rects、path_fills、path_strokes、strokes、gradients、shadows、images、glyphs、filters、blend_modes、transforms、clips）— app_render.rs:1512 遍历全 13 字段（1526-1840）无丢弃
- [x] ✅(R968) 所有图元类型正确应用 `scale_factor` 和 `offset` — `transform_webview_primitives`（app_render_primitives.rs）对全部 13 种图元应用 `out = in * scale + offset`；单测 `transform_webview_primitives_applies_scale_and_offset_to_all_types` 覆盖 fills / rounded_rects（4 圆角独立缩放）/ gradients（Linear/Radial/Conic 三变体；Conic `start_angle` 无量纲不缩放）/ shadows（offset+blur+spread）/ strokes（端点+线宽）/ glyphs（font_size）/ transforms（rect+origin+tx/ty），逐类型断言 scale=2 + offset=(10,20) 的精确输出
- [x] ✅(R968) 所有图元类型正确应用视口裁剪（`clip_y` + `clip_rounded`）— `transform_webview_primitives` 用 `ViewportClip`（轴对齐矩形）裁剪全部 13 种图元（rect 类走 `clip_rect_field`/`clip_axis_aligned_rect` 求交，path 走 `path_vertices_bbox`，glyph 走 font_size 包围盒）；单测 `transform_webview_primitives_culls_primitives_outside_viewport` 验证视口外 rounded_rects/gradients/path_fills/glyphs 被裁掉、视口内保留。注：`clip_y`（水平带）+ `clip_rounded`（圆角矩形）是 `append_webview_primitives`（fills+glyphs 混入 chrome 层）的细粒度裁剪，用于圆角 page frame；全 13 类主消费路径用 `ViewportClip`
- [x] ✅(R155) 图元渲染顺序遵循 CSS painting order（background → borders → content → outline）— `draw_order` 是 painter 生产默认（R149/R152/R155，painter/mod.rs:1459 `draw_order 是默认渲染路径`）；paint 系统按 CSS painting order 发射图元，浏览器消费层（本文件）保持发射顺序按类型重组，不重排

## DC-11: 布局正确性

- [x] **Margin 折叠** — 相邻块级元素的 margin-top/margin-bottom 按规范折叠（正 margin 取最大、负 margin 取最负、正负抵消）— ✅ **R323 实测通过**（taffy 0.7 CollapsibleMarginSet；6 探针 case + 5 reftest 全过）
- [x] ✅ **BFC 创建** — `overflow: hidden/auto/scroll`、`display: flow-root`、浮动元素、`position: absolute/fixed` 正确创建 BFC，隔离浮动和 margin 折叠（`establishes_bfc` 全条件 + margin 隔离 R323 实测 + `use_bfc_float_containment` float containment；见 known-gaps BFC 行 ✅）
- [x] ✅ **Float 布局** — float 定位（float: left/right）+ clear + float containment（BFC）+ inline exclusion — `float_positioning.rs::adjust_float_positions(_with_context)` 处理 float 定位 + active_left/right_float_bottom（clear 语义）+ `use_bfc_float_containment`（containment）；**R895 实测验证**（float:left 100×80 盒正确居容器左上、块级兄弟盒 border-box 全宽在 float 之后、inline 文本绕 float 排版 y<80 时 x<95 无文本/x≥100 有文本）；原 master.md「无原生 float 布局」描述已过时。残余 = CSS2 float reftest 边缘 case（结构性 plateau，非核心布局缺口）
- [x] **Position: fixed** — 相对 viewport 定位（当前错误地映射为 absolute）— ✅ **R324 修复**（`adjust_fixed_to_viewport` 改为扣除祖先偏移；fixed 在有偏移 positioned 祖先内也视口相对，与 R98 absolute-viewport 约定一致；新单测 + 8 旧单测更新 + 全量 reftest 438/490 零回归）
- [ ] **Position: sticky** — 滚动时正确固定在指定偏移范围内 — ✅ **R1982 静态部分已验证**：`position:sticky` 经 converter:366 映射 taffy Relative + engine.rs:1419 应用 inset，scroll=0 时正确等价 relative（offset 生效，单测 `r1982_position_sticky_at_scroll0_acts_as_relative` 守 y=static+offset）。⏳ 残余 = 动态 sticking（滚动时固定在阈值范围内），属 host 层（browser scroll 输入路由）interactive 特性，非 rendering-compat reftest 范围（reftest 静态 scroll=0）。
- [ ] **Overflow: scroll/auto** — 可滚动容器功能，scroll 偏移正确应用到子元素布局 — ✅ **R1982 静态部分已验证**：`overflow:auto/scroll` 容器经 taffy（converter:85 overflow 字段）正确保持显式 height（不撑满 content，单测 `r1982_overflow_*_container_keeps_explicit_height` 守 height=100 非 300）+ R1861 paint clip 已工作。⏳ 残余 = 动态 scroll offset 应用到子元素，属 host 层 interactive 特性，非 reftest 范围。
- [x] **替换元素** — `<img>` 的固有尺寸（intrinsic size）正确计算，`object-fit`（fill/contain/cover/none/scale-down）正确应用 — ✅ **R323 代码核查 + R325 修复**（`apply_replaced_element_sizing`：HTML `width`/`height` 属性 + SVG data URI + 解码固有尺寸三来源；CSS §10「两侧尺寸都显式时不强制固有宽高比」R325 修复，旧实现 `<img style="width:200px;height:50px">` 被 taffy 拉成 200×200；`compute_object_fit_rect` 全 5 值；R318 图片数据端到端贯通）
- [x] ✅ **百分比高度** — containing block 有明确高度时百分比高度正确解析；无明确高度时 height: auto（`clamp_percentage_max_height` engine.rs:1422 按 definite CB 高度解析 %，R168 table height-as-minimum）
- [x] ✅ **Auto margin 居中** — `margin: auto` 在 block/flex/grid 中正确居中（block/table R165 compute()根居中+table_shrink both_margins_auto；flex/grid taffy native auto-margin）
- [x] ✅ **min/max-width/height** — 约束正确应用到最终尺寸（`clamp_percentage_max_height` + R517 cascade 负值 max/min-height + R428 flex min-size:auto）

## DC-12: 高级视觉效果

- [x] ✅ **text-shadow** — 文字阴影渲染（offset + blur + color）— painter/text.rs:1067 `if has_text_shadow { push shadow glyph at (x+ox,y+oy,shadow_color) }` + test_paint_text_shadow_basic
- [x] ✅ **多背景图层** — `background-image` 多层叠加渲染 — painter/effects.rs:134 `for layer in background_image.iter().rev()` 全图层叠加 + test_paint_multiple_overlapping_backgrounds
- [x] ✅ **重复渐变** — `repeating-linear-gradient` / `repeating-radial-gradient` — cpu/gradient.rs:28 `if gradient.repeating {` 处理重复渐变
- [x] ✅ **border-image** — 边框图片渲染（slice + repeat + width）— painter/mod.rs 实现 + paint/tests/border_image_repeat.rs 单测（M9）
- [x] ✅ **clip-path** — CSS clip-path 基础形状（circle、ellipse、polygon、inset）— painter/effects_indicators.rs + helpers.rs 实现（M9）
- [x] ✅ **backdrop-filter** — 元素背后内容的滤镜效果 — painter/effects.rs 实现；**R894 实测验证**（gradient 背景 + overlay backdrop-filter:blur(15px)，with-vs-without 渲染 diff = 15314 px 恰落在 overlay 盒 y[0,80] 带、带外 0 px，证 blur 效果正确限定在元素盒内）
- [x] ✅ **CSS mask** — 基础遮罩效果（渐变蒙版裁剪 + alpha 衰减）— painter/effects.rs 实现（M9）
- [ ] **scroll-snap** — 滚动吸附行为（需宿主层滚动输入路由）
- [~] **打印媒体查询** — `@media print` 基础支持（可选，降低优先级）— ✅ **R1981 cascade 支持落地**（部分）：`StyleSystem` 加 `media_type` 字段（default Screen）+ `set_media_type()` API + 接入 `MediaContext`（lib.rs:355）；@media print/screen/all 级联过滤现在按渲染媒体类型正确生效（单测验证：Screen 模式 @media print 不应用 / Print 模式 @media print 生效 + @media screen 失效）。default Screen = 零生产行为变更。✅ **R1991 reftest runner `--media print|screen` flag 落地**：`RenderPipeline::set_media_type`（镜像 set_prefers_color_scheme）+ `ReftestConfig.media_type`（默认 Screen，零回归）+ `render_to_framebuffer_with_layout_with_base` 接线 + main.rs `--media` flag + cmd_reftest/upstream/oracle 三路径 thread + e2e 单测（Screen vs Print 渲染显著不同）。✅ **R1992 webview 生产接线落地**：`WebView::set_media_type`（镜像 set_prefers_color_scheme：字段持久化 + 6 render 入口重放 pipeline.set_media_type）+ engine `MediaType` re-export + integration smoke test——嵌入边界（外部 app + zero-browser）可切 Print 媒体类型。✅ **R1993 browser Ctrl+P print-preview 落地**：7-layer 全栈镜像 set_color_scheme（protocol IPC `SetMediaType` + process_backend + tab_manager + tab_worker + renderer + app `toggle_print_preview` + Ctrl+P handler），双后端（in-process + 多进程 IPC）贯通——浏览器 Ctrl+P 即时重渲染使 @media print 样式生效（minimal preview）+ toggle test。⏳ 待真 print-layout（@page/page-break 分页渲染，打印分页）+ print-mode chromium oracle capture 后 `make reftest-oracle --media print` 量真实 @media print WPT yield。

## DC-13: 产品静态页面视觉 smoke

- [x] ✅(R1600+R1601) `apps/browser/assets/welcome.html` 通过 ZeroBrowser 窗口/无头路径截图，并与 Chromium 在相同 viewport 下的参考截图对比 — R1600 修复 headless `captureScreenshot` 渲染全 13 图元（旧版仅 fills+glyphs 丢 11 种）；R1601 扩协议返回 base64 PNG 像素 + 新增 `browsingContext.loadHtml` 命令（绕过 fetch_url HTTP-only，加载自包含 fixture）+ `test_dc13_line315_welcome_headless_vs_chromium_oracle` 单测：welcome 经 headless loadHtml→captureScreenshot→base64 PNG 解码 vs `welcome-chromium.png` oracle（tracked），实测 diff 10.80%（< 25% 阈值，同字体墙 baseline ~17% 谱系，per-channel>8 严于 compare_pixels 故数值偏低）；验真实 ZeroBrowser headless 渲染管线（WebView + render_full_scene 全图元）端到端
- [ ] `https://morning.work/page/2026-02/fedora-macbook-three-finger-drag.html` 录制为固定 HTML/CSS fixture，并通过 ZeroBrowser/WebView/Chromium 三方截图对比；fixture 必须包含原页面依赖的 `/article.css`、`/styles/github.css`、`/JetBrainsMono/JetBrainsMono.css` 或明确记录不可用资源
- [ ] `https://wintertc.org/` 录制为固定 HTML/resource fixture，并通过 ZeroBrowser/WebView/Chromium 三方截图对比；fixture 必须包含内联 Twind CSS、`/static/logo.svg`、`/static/logos/*.svg`、`/static/logos/*.png` 等首页可见图片资源
- [x] ✅(R658) **Legacy Static Web smoke（HTML 3.2/4 + CSS1/2）**：建立固定 fixture 集并纳入 product-smoke 路径，首批至少 20 页，覆盖无 CSS 老式文档、HTML presentational attributes、表格布局、图片与文本环绕、列表/链接/标题、`font` 标签、`hr`、基础 CSS1/2 外链样式。每页必须有 Chromium oracle 截图和 ZeroWeb CPU 输出截图，失败时持久化 diff 与资源清单
- [x] ✅(R658, fixture 020) Legacy fixture 中必须包含类似 `testpage.htm` 的最小代表页：`BODY BGCOLOR/TEXT/LINK/VLINK`、`TABLE BORDER/CELLPADDING`、`TR BGCOLOR`、`IMG ALIGN=TOP`、`FONT SIZE`、`UL/LI`、`A href`。该页用于防止"WPT 分目录推进但老式静态页仍不可读"的回归
- [x] ✅(R658 逐项体检通过) Legacy Static Web smoke 的短期验收口径是"可读且结构不崩"：正文不重叠、不串行；表格单元格边框/内边距可见；图片按替换元素参与 inline 布局；链接颜色/下划线可见；`font size/color` 影响文本；body 背景和文本色生效。像素阈值可先作为趋势指标记录，不得用它替代 WPT/DC-14 达标口径
- [x] ✅(R213) URL 导航路径必须加载并应用 `<link rel="stylesheet">` 外部样式表；外链 CSS 抓取失败应作为可诊断的资源加载错误记录，不得静默退化为仅内联 CSS 渲染
- [x] ✅(R318) URL 导航路径必须加载 `<img src>` 图片子资源，将解码后的 SVG/PNG/JPEG/WebP 像素数据写入 `ImageCache`，并在 ZeroBrowser CPU/GPU 渲染路径传入 renderer；图片缺失不得被 alt 文本或占位 glyph 静默替代
- [x] ✅(R662) 同一输入通过 `zero-webview` 直接渲染路径截图，并与 Chromium 参考截图对比，避免产品层和 WebView 层互相掩盖问题 — `product-smoke --via-webview`（`render_via_webview_to_framebuffer` 走 `WebView::load_html` 嵌入边界）；welcome.html via-webview vs engine-direct **0.00% byte-identical**，两侧 vs Chromium 均 16.16%（字体墙），证明产品层↔WebView 层不互相掩盖
- [x] ✅(R1597) 至少覆盖桌面和窄屏两个 viewport；桌面 viewport 下必须验证 hero 标题、四个 feature card、快捷键区、快速访问区和 footer 的相对位置 — 桌面（800px）计数断言 `card:4`+`shortcut:6`+`link-tile:4`+`footer:1`+行数 `title:1`/`tagline:2`；窄屏（375/320）`card:4`+struct-check（welcome 无 width 媒体查询，grid 保持 2 列）
- [x] ✅(R1597) welcome.html 自动检查文本不重叠（`check_sibling_overlaps`）、不同 sibling card/link/shortcut 的文本不串联（R1597 `check_text_concatenation`，信号=容器 `text_node_line_heights` 吸收子元素子树非空白文本节点）、`ZeroBrowser` 标题在宽屏下不被错误拆行（`--expect-lines title:1`）、`<br>` 后的中英文 tagline 保持两行（`--expect-lines tagline:2`）
- [x] ✅(R1598) morning.work 文章页自动检查 nav/title/date/tag badges/阅读时间不串联（`--expect-class item-tag:3` + struct-check `check_text_concatenation`）、正文段落不被压成同一行（concat + sibling-overlap）、inline code 保持行内位置（concat 守容器不吸收子元素文本）、table 仍按表格布局绘制（`check_collapsed_containers` 守 table 不塌缩）、pre/code 块保持独立背景和换行（`--expect-class lang-bash:1` 在位 + 不塌缩）
- [x] ✅(R1598) WinterTC 首页自动检查 header logo 可见（`--check-img-visibility` = `check_replaced_collapse` 守 14 个 logo 不塌缩，R1578b 谱系）、标题/副标题不串联（`check_text_concatenation`）、四个 nav button 分列（`--expect-class bg-orange-500:4` + sibling-overlap）、正文段落按宽度换行并保持 justify（`--expect-lines-min text-justify:2`）、参与方 Logo 网格中 SVG/PNG Logo 可见且不会退化为短横/alt glyph（`--check-img-visibility` 守全部 logo 几何非塌缩；像素级 alt-glyph 退化需 oracle 像素对比，超出 struct-check 范围）
- [x] ✅(R2004) ZeroBrowser 不得对 WebView glyph 做会改变布局语义的整行重排；如需字体 fallback 或选择命中，应在不改变原始 glyph 坐标语义的路径上实现 — `transform_webview_primitives`（app_render_primitives.rs:412）+ `append_webview_primitives`（:65）按输入顺序逐个映射 glyph（仅 scale+offset + 裁剪，无 sort/reorder）；单测 `transform_webview_primitives_preserves_glyph_order` 构造会被 sort-by-Y/font-batch 优化重排的 glyph 序列（混合行 y=10/30 + 混合字体 font0/1），断言输出严格保输入顺序 [A,B,C,D]，守此不变量防未来 glyph batching/scanline-sort 优化破坏跨行布局语义
- [x] ✅ 截图、对比报告和失败根因持久化到 `docs/goal/rendering-compat/evidence/product-static/`（welcome PNG + `legacy-html/` 20 fixture+diff-summary + `morning-work/` + `wintertc/` + `narrow/` + README + 各轮 rXXX evidence 根因分析）

## DC-14: 真通过标准（anti-false-pass）— 验证可信度门禁

> 本 DC 防止 reftest 通过率被「同源假通过」「宽容差」「子集分母」污染。**DC-2~13 的通过率数字只有在本 DC 同时满足时才可信、才计入达标判定。**

> **字体光栅化非渲染差异来源（2026-06-17 AA 基准实测）**：fontdue 与 chromium 对同一 glyph 光栅化几乎完全一致（W 0.1% / i 3.0%，见 `evidence/aa-baseline-2026-06-17.txt`）。welcome 26% / Oracle 污染 48.6% 的大头是**布局/度量（line-height / R109 inline→block / 多行结构）**，非字体光栅化。**勿再以「fontdue AA 噪声」为渲染差异归因**（纠正 R174/R187 误诊）；字体攻坚应停止，转向布局/度量。fontdue 无需替换。

> **★ 多行 y 堆叠已修（R630，2026-06-25，commit d31cf03a）**：「多行结构」差异的一个具体子项——paint Path B 对 auto-wrap 多行块用 `all_fragments()`（y 恒 0）致**多行文字垂直堆叠看不清**——已修复（统一用 `all_fragments_with_line_y()`）。这是用户可见「文字堆叠」的直接修复，同源 reftest net +24（normal-flow +6 / positioning +19）。**注意区分**：font-weight 加粗（R229c 证伪，字体墙死路）≠ 多行 y 堆叠（paint 逻辑 bug，已修）——「真实网站没法看」是多个独立 bug 叠加，逐个定位比笼统归「字体架构」有效。残余行级度量差异（product-smoke morning/welcome +0.3~0.8pp）是 R374 字体匹配问题（堆叠掩盖→分行显现），独立多会话。详见 master.md R630 + [`evidence/r630-paint-pathb-multiline-y-fix-2026-06-25.txt`](./evidence/r630-paint-pathb-multiline-y-fix-2026-06-25.txt)。

> **★ 字体归因三证推翻（R229c/R631，2026-06-25）**：「字体问题」是误诊——三角度全证伪字体为 diff 主因：(1) font-weight 加粗（R229c）product-smoke 5 组参数全退步；(2) 字体选择对齐（R631）强制 sans-serif→NotoSansCJK（chromium 经 fontconfig 的同款字体）后 morning 17.16→17.15% / welcome 17.27→17.20% **零变化**，推翻 R374「字体不匹配」归因；(3) 光栅化（R388）fontdue≈chromium。morning/welcome 17% 真因 = **布局/行盒度量**（line-height/baseline/行间距），非字体。**勿再以「字体问题」为真实网站 diff 归因**——真正 lever = Phase A 行盒度量统一（line-height 计算/baseline 定位/行盒 y，R630 已修多行 y 堆叠子项）。详见 [`evidence/r631-font-match-refuted-2026-06-25.txt`](./evidence/r631-font-match-refuted-2026-06-25.txt)。

> **★ 行盒度量连续修复（R630/R632，2026-06-25）**：确证 morning/welcome 17% 真因是行盒度量后，连续两步实质进展——R630（commit d31cf03a）修 paint Path B 多行 y 堆叠（同源 net +24）；R632（commit 0911a2ac）修 paint Path B line-height 忽略 CSS（compute_final 不存 override → fallback 19.2，line-height 1.5/2.0 产出相同行位置；修复后正确响应 CSS，reftest net +5，welcome -1.11pp）。R627 的 pre-wrap -15 被 R630 吸收。残余：morning 中文 +0.99pp = frag.height 字体**度量**（NotoSansCJK 行高 ≠ chromium，R374 谱系，区别于字体选择 R631 已证伪）。下一步 = baseline 定位 / 字体度量统一。详见 [`evidence/r632-line-height-override-fix-2026-06-25.txt`](./evidence/r632-line-height-override-fix-2026-06-25.txt)。

- [x] ✅(R669) **独立 Oracle（reference 不得由被验证者自渲染）**：reftest 的参考基准必须是 **Chromium 渲染 test.html**，不得是 ZeroWeb 自渲染 ref.html。**✅ R669 落地 `zero-wpt-runner reftest-oracle` 子命令 + `make reftest-oracle [DIR=...] [ORACLE_PASS_RATIO=...]`**：渲染上游 WPT test 页（`render_to_framebuffer_with_base`）vs chromium oracle-shot（`oracle-shots/{safe_id}.png`，**13793 张**，经 `capture-chromium-screenshots.mjs`/`capture-oracle-per-dir.mjs` 抓取），报告 chromium-Oracle 真一致率（z_vs_chr < `ORACLE_PASS_RATIO`，默认 1%）+ top-15 最差发散修复候选 + per-dir 分解 + self-source 假通过对照。**doc-maintainer spot-check 复现**：`make reftest-oracle DIR=css-grid` = 16/49 = **32.7%**（z_vs_chr<1%，R669 时基线），与 R560 文档基线 + self-source ~56.5%/DC-14 46.5% 假通过一致。**R1762 fresh 复测 = 28/49 = 57%**（z_vs_chr<1%，strict 真通过 10/20.4% + near 18/36.7%）——历轮 grid work（R1015/R1291 +10/R1293/R1469 等）将该 dir 从 32.7% 推到 57%，证明 grid 修复有效；残余 top = grid-container-baseline-synthesized-*（vertical-mode R1043/R1050 entangled）/ table-grid-item-dynamic-*（JS）/ stretch-grid-item-button/text-input-overflow（native-widget R1695）/ fragmentation-print（tentative），均已知 blocked 类别非 clean lever。**范围注（诚实）**：默认 `reftest` 路径仍 ZeroWeb self-ref（保留作同源自一致性参考），R669 的 `reftest-oracle` 作为**一等独立 Oracle 指标**补充——满足本项「至少抽样跑 ZeroWeb-test vs Chromium-test + 量化污染比例」要求，且覆盖**全量 corpus 优于抽样**。原「闲置 capture-chromium-screenshots.mjs」已接入。原「reftest.rs:230-232 用 ZeroWeb 渲染 ref」描述适用于默认 reftest 路径，reftest-oracle 路径已用 chromium Oracle
- [x] ✅(R852 oracle + R970 self-source) **非平凡性检查**：拒绝 `test == ref` 且接近纯色（或 PNG 退化）的 case 自动判 PASS——必须标记为「可疑/退化」并单独审计，防止 harness PNG 加载 bug 等导致的退化假绿（历史已发生，见 `archive/rounds-r23-r139.md` R135/R149）。**R852 落地 oracle 路径**：`frame_is_near_solid`（采样每 16 像素，主色占比 >99.9% 判退化）+ 报告「退化可疑 pass 排除 + credible pass + 审计列表」；实测 3% corpus 近纯色（parsing/animation/print/crashtest headless 空白）；**R970 落地 self-source 路径**：`frame_is_near_solid` 移到 `reftest_compare`（pub，两路径共享）+ `ReftestResult.test_near_solid` 字段 + `run_reftest_with_base`/`run_reftest_gpu_with_base` 计算；`print_dc14_three_state` 把 strict-pass 拆成 可信(非近纯色)/可疑(近纯色，列审计列表)；内置 css21 实测 可信 569(82.9%)/可疑 70(10.2%，多为简单 smoke 理性近纯色)/近似 47(6.9%)/不一致 0
- [x] ✅(R851 oracle 三态 + R969 self-source 三态 + R1599 非平凡性 empirical-verified) **严格容差复跑 + 三态分类**：必须在文档锁定容差（布局 ≤ 0.1% / 文字 ≤ 0.5%，优先 WPT fuzzy 注解）下复跑全量，输出 **真通过 / 近似通过（超锁定容差但更宽松）/ 假通过（退化或同源）** 三态。唯一可信达标指标 = **严格容差真通过率**。当前 vertical-rl clearance 用 5% 容差属近似通过，不计入真通过。**R851 落地 oracle 路径三态**（strict<0.1%/0.5% / near(strict..1%) / mismatch）；全量 corpus 实测 loose 38.4%、揭示 strict 真通过率（positioning 0.6% vs loose 45.6%，字体光栅噪声主导）；**R969 落地 self-source 路径三态**：`print_dc14_three_state`（cmd_reftest + cmd_reftest_upstream 调用）——`strict_pass + near_pass == pass_count`、`mismatch == fail_count` 自洽，strict 边界 = `category.strict_max_diff_ratio`（0.1%/0.5%）+ `strict_max_channel_diff`（2/5），near/mismatch 边界用 `result.passed`（编码实际有效 loose 阈值含 fuzzy override）；**self-source 非平凡性 R970 已落地**（见 line 344）+ **R1599 empirical 验证**：`make reftest` self-source 三态 真通过-可信 527(76.8%)/可疑-近纯色 112(16.3% 审计列表)/近似 47(6.9%)/不一致 0——三态分类 + 非平凡性（test==ref 退化假绿）均工作；纠正旧「非平凡性 pending」表述（与 line 344 [x] R970 矛盾）
- [x] ✅(R1599 verified) **容差锁定不可放宽**：布局类 ≤ 0.1%、文字类 ≤ 0.5% 为硬上限。不允许以「实测校准」「字体差异」为由放宽容差；文字类大面积失败必须修渲染，不得放宽容差——strict 阈值 `ReftestCategory::strict_max_diff_ratio`（Layout 0.001/Text 0.005/Unknown 0.001）+ `strict_max_channel_diff`（2/5/2）（reftest.rs:98-109）锁定 + 防放宽不变量单测 `dc14_locked_strict_thresholds_invariant`（reftest.rs:2207，断言锁定值防回归放宽）
- [x] ✅(R484) **分母真实性（去子集化）**：分母 = 上游每目录**全部**范围内 reftest（从 `MANIFEST.json` 自动提取）。✅ R484 完成 10/10 目录全量去子集化（~9967 reftest 全覆盖，见 known-gaps「真实 WPT reftest」行 ✅）；`tests/wpt-runner/scripts/audit-reftest-coverage.py` 对账机制存在。原「`COUNT=60` 子集」描述已过时（pre-R484）
- [x] ✅(R660/R661) **GPU 非 passthrough**：GPU 渲染器每种图元有独立 wgpu+mesh 管线（`draw_gradient/image/rounded_rect_pass` + `collect_*_vertices`，非 CPU 后处理），DC-9 13/13 图元均附 framebuffer 像素断言测试（见 DC-9）。原「DC-9 多图元 CPU 后处理对齐=passthrough」描述已过时（pre-M7）
- [x] ✅ **内联 smoke 不计达标**：DC-2~5 的内联 reftest 100% 仅作 smoke，不计入达标判定。master.md DC 进度表中任何「内联 100%」不得标记为该 DC 达标（685 inline reftest 明确 smoke-only，通过率统计分母为上游真实 reftest）

### 通过率统计口径

- **统计对象**：从上游 WPT 仓库（`https://github.com/web-platform-tests/wpt`）导入的**真实 reftest case**，**不含**现有 1,341 个手写 `TestCase`，也**不含** 685 个手写 inline reftest
- **现有 1,341 个手写 TestCase + 685 个 inline reftest**：保留为 smoke test 套件，继续全绿运行，但**不计入**本目标的 reftest 通过率统计
- **产品静态页面视觉 smoke**：不计入 WPT reftest 通过率分子/分母，但属于目标完成门禁；它用于捕获 WPT 子集未覆盖的产品可见排版退化
- **分母**：上游 WPT 每个目录下**全部**范围内 reftest case（即上游该目录中所有不属于 skip list 的 reftest），**不是**人为挑选的子集。必须从上游 WPT 的 `MANIFEST.json` 自动提取 reftest 列表，不允许手动筛选
- **分子**：运行后判定为 PASS 的 case 数量
- **通过率** = 分子 / 分母 × 100%
- **可信度前提（DC-14）**：上述通过率只有在 reference 独立于被验证者（Chromium 渲染，非 ZeroWeb 自渲染）、严格容差、全量分母时才可信。不满足 DC-14 的通过率（含当前 436/490，reference 为 ZeroWeb 自渲染）只能作「自一致性」参考，**不构成达标证据**
- **失败 case 约束**：通过率 ≥ 95% 的情况下，仍需对所有失败 case 进行根因分析并记录到 evidence。不允许有「未分析的失败」。失败的根因分类为：CSS parser 错误、样式计算错误、布局算法错误、渲染器错误、JS 执行错误、范围外误入、已知 fontdue/Skia 字体差异（仅文字类）。根因为渲染错误的必须有修复计划
- **要求**：每个 WPT 目录的分母 = 上游该目录全部 reftest − skip list 中范围外 reftest。分母不允许人为缩减——不允许跳过已知会失败的 case，不允许只挑选简单 case
- **最低分母要求**：每个目录范围内 reftest 必须 ≥ 50 个（如果上游该目录范围内 reftest 不足 50 个，则导入全部）
- **禁止**：
  - 不允许通过缩小导入范围来人为提高通过率
  - 不允许用 inline 手写 reftest 替代上游真实 reftest 来充数
  - 不允许只导入预期会通过的简单 case 而跳过已知的困难 case
  - 不允许通过放宽容差来掩盖真实的渲染差距
