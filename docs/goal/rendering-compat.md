# 页面渲染兼容性 — WPT Reftest 驱动的渲染正确性目标

**版本**: v1.0
**日期**: 2026-06-06
**状态**: Active
**执行模式**: 长期无人值守持续执行（rally run）
**父目标**: `docs/goal/zero-web.md`（ZeroWeb 总体目标）

> **说明**
> 本文档是 ZeroWeb 页面渲染兼容性的专项目标执行契约。目标是以 WPT reftest 通过率为验证标准，将 ZeroWeb 的 CSS 渲染输出对齐到 Chromium（Chrome/Edge）水平。本文定义了使命、边界、完成标准、执行协议和文档治理规则，供后续 `rally run` 会话作为稳定输入使用。

---

## Mission

以 **上游 WPT 真实 reftest 通过率 95%+** 为核心验证指标，确保 ZeroWeb 的页面渲染效果在核心 CSS 领域与 Chromium（Chrome/Edge）一致。

**关键约束**：所有验证必须基于从上游 WPT 仓库（`https://github.com/web-platform-tests/wpt`）导入的**真实 reftest**，不允许使用手写 inline reftest 替代或充数。通过率统计的分母是上游 WPT 目录中**所有**属于范围内、不在 skip list 中的 reftest case，不允许人为缩小导入范围。

**⚠️ 优化目标 = chromium Oracle 一致率，非同源通过率（DC-14，2026-06-16 实测确立）**：reftest runner 当前用 ZeroWeb 自渲染 ref 作参考（`reftest.rs:230-232`），同源通过率含 **46.5% 假通过**（全量实测，见 `evidence/cross-validate-full-2026-06-16.txt`）——真实「与 chromium 一致」通过仅 ~37%。**同源通过率（当前 436/489）不再作为优化目标或达标依据**；优化目标改为「chromium Oracle 一致率」，修复优先取 `evidence/analyze-pollution-2026-06-16.txt` 的 18 个真 bug 候选，每项修复用 `scripts/cross-validate.py` 验证（而非仅看同源通过）。**★ R669 起 chromium Oracle 已集成为一等 harness 指标**：`make reftest-oracle [DIR=...]` 直接报 per-dir chromium-Oracle 真一致率 + top 发散修复候选（DC-14 独立 Oracle 项 ✅，见下），取代 post-hoc cross-validate.py 作主测量路径。

覆盖范围：

1. **渲染器图元覆盖** — CPU 渲染器和 GPU 渲染器必须支持所有 13 种 `RenderPrimitives` 图元类型，浏览器必须正确消费所有图元
2. **CSS 2.1 核心**（`css/css2/`, `css/CSS2/`）— 渲染兼容性的基石
3. **Flexbox + Grid**（`css/css-flexbox/`, `css/css-grid/`）— 现代布局引擎必备
4. **Positioning + Float + Table + Multicol**（`css/css-position/`, `css/css-float/`, `css/css-tables/`, `css/css-multicol/`）— 传统布局模式完整覆盖
5. **文字排版全套**（`css/css-text/`, `css/css-writing-modes/`, `css/css-fonts/`, `css/css-text-decor/`）— 文本渲染正确性
6. **布局正确性** — Margin 折叠、BFC、Float 布局、滚动容器等核心 CSS 2.1 布局行为
7. **高级视觉效果** — text-shadow、多背景图层、clip-path、backdrop-filter 等

执行方式：**交替推进** — 每轮执行同时扩展上游 WPT 真实 reftest 导入范围和修复发现的渲染缺口，直到目标通过率达标。

运行环境：**CPU 软件渲染 + GPU 渲染都必须通过** 上游 WPT 真实 reftest 验证。

参考基准：**Chromium（Chrome/Edge）** 的渲染输出作为 reftest 的参考截图来源。

### 优先级修订：Legacy Static Web（HTML 3.2/4 + CSS1/2）

**背景记录（2026-06-26）**：用户反馈 `http://172.27.46.54:8000/testpage.htm` 一类老式静态页面渲染效果差。该页面不是 IE1 专属兼容目标，而是典型的 HTML 3.2/4 + CSS1/2 静态网页模式：`BODY BGCOLOR/TEXT/LINK/VLINK`、`TABLE BORDER/CELLPADDING`、`TR BGCOLOR`、`IMG ALIGN=TOP`、`FONT SIZE`、标题/段落/列表/链接等基础结构。当前 `rendering-compat` 主线以 WPT reftest + Chromium oracle 为核心，虽已覆盖部分 CSS2/presentational hints，但没有把这类老式静态网页作为独立产品验收面。

**裁决**：在不降低 WPT/DC-14 最终目标的前提下，将 **HTML 3.2/4 常见静态文档 + CSS1/2 常见布局** 提升为短期高优先级推进面。理由是：

- 这类页面大量依赖 UA stylesheet、HTML presentational attributes、基础 block/inline、表格、图片、列表和链接颜色，修复通常比 multicol/writing-modes/font-feature 等现代或结构性子域更局部。
- 用户可见收益更直接：静态文档、内网页、说明页、老式工具页不需要 JS/现代 CSS，也能暴露基础排版/绘制链路问题。
- 该方向不是完整 CSS2 达标的替代品；完整 CSS2 `chr<1%` 仍是长期目标，但短期应优先让 legacy static pages “可读、布局不崩、核心语义可见”。

**Legacy Static Web Tier 1 范围**：

- HTML presentational hints：`body bgcolor/text/link/vlink/alink`、`table border/cellpadding/cellspacing/width/height`、`tr/td/th bgcolor/align/valign/width/height`、`img width/height/align`、`font size/color/face`、`hr` 基础属性。
- UA stylesheet 基线：`h1`-`h6`、`p`、`ul/ol/li`、`b/strong`、`i/em`、`a`、`table/tr/td/th`、`font`、`hr` 的默认 display、margin、font-size、font-weight、font-style、text-decoration、border/padding 语义。
- CSS1/2 常见模式：颜色/背景、字体大小与继承、普通流、inline formatting 基础、表格基础布局、替换元素尺寸与 baseline/vertical-align、margin/padding/border、float/clear 基础。
- 明确暂不扩展到 IE 专属行为或浏览器 bug 兼容；quirks mode 只按标准/Chromium 可解释行为推进。

**验收方式**：新增 `legacy-html` 产品 smoke fixture 集，至少包含 20 个 HTML 3.2/4 + CSS1/2 静态页面（真实录制 + 合成最小页各占一部分），使用 Chromium 参考截图做 oracle，并在 ZeroWeb CPU 路径输出截图后做像素对比。该 fixture 集不替代 WPT 通过率，但作为短期修复优先级和回归门禁；每次修复必须同时说明它对应的 WPT/CSS 规范点或 legacy fixture。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| WPT reftest 基础设施 | 导入上游 WPT reftest、解析 test list（含 fuzzy 注解）、截图对比、通过率报告、CI 集成 | **扩展**现有 `tests/wpt-runner/src/reftest.rs` 和 `manifest.rs`，不重写 |
| Chromium 参考截图 | 自动化 headless Chromium（Puppeteer/Playwright）截图工具链 | 作为 M1 基础设施的一部分构建，零手动操作 |
| Reftest 分类容差 | 布局类 reftest（不含文字渲染）用严格容差；文字类 reftest 用宽松容差；WPT fuzzy 注解按 test 覆盖 | 解决 fontdue vs Skia 字体像素差异问题 |
| WPT fuzzy 注解支持 | 解析上游 WPT MANIFEST.json 中每个 reftest 的 `fuzzy()` 元数据，并应用到像素对比 | 上游 reftest 自带容差声明，必须遵守 |
| Viewport 对齐 | ZeroWeb 和 Chromium 截图必须在相同 viewport 尺寸下捕获（默认 800×600，可配置） | viewport 不同则对比无意义 |
| JS 执行支持 | Reftest harness 在截图前执行页面 JavaScript（通过现有 `script-sandbox` V8 runtime） | 很多 WPT CSS reftest 依赖 JS 动态设置条件 |
| Quirks mode | CSS parser / style system / layout engine 中实现完整的 quirks mode 调整 | ⚠️ 状态以 R248 实证为准：DOM 已存储 + style-system/css-parser 已消费（3 quirks 预烘焙 + mode-gated，生产接线）；下游「完全忽略」过时。layout-engine 无独立 quirks 层（由 style-system 预烘焙覆盖）。wpt-data 仅 ~3 quirks 用例全过（z_vs_chr≤0.13%），非缺口、非 reftest 杠杆 |
| CSS 2.1 渲染 | 盒模型、颜色、背景、边框、margin 折叠、inline formatting、BFC、浮动清除、基础定位 | 这是最大的 reftest 覆盖面，优先级最高 |
| Inline formatting 所有权 | 文本节点、inline 元素、inline-block、`<br>`、混合中英文文本必须在 layout 和 paint 之间只有一个权威行内布局结果 | 防止父容器重新收集整棵 inline 子树文本，同时子 inline 元素又作为独立 LayoutBox 递归绘制，导致 sibling 文本串联、重复或错位 |
| Layout/Paint IFC 一致性 | Layout engine 必须持久化最终 IFC 片段结果到 `LayoutBox`，paint 必须复用该结果，不允许用不同 style map、float exclusion、container width 再跑第二套 IFC | `apps/browser/assets/welcome.html` 这种简单静态页已经暴露 layout box 与 glyph 输出不同源的问题 |
| 外部样式表加载 | WebView/Browser URL 导航必须识别 `<link rel="stylesheet">`，按文档 URL / `<base>` 解析相对地址，完成安全检查、HTTP 缓存、CSS 抓取和级联顺序合并后再进入样式计算 | 真实静态页面通常依赖外链 CSS；`https://morning.work/page/2026-02/fedora-macbook-three-finger-drag.html` 依赖 `/article.css`、`/styles/github.css`、字体 CSS |
| 图片子资源与替换元素 | WebView/Browser URL 导航必须抓取 `<img src>`、CSS `url()`、favicon/metadata 中实际参与渲染的图片资源，支持 PNG/JPEG/WebP 基础解码和 SVG 栅格化，并把解码后的像素数据通过 `ImageCache` 传给 CPU/GPU renderer | `https://wintertc.org/` 主要内容依赖 SVG/PNG Logo；**已实现**（R318 实测）：`fetch_image_subresources` 抓取+解码（PNG/JPEG/SVG via resvg），`image_cache` 传 renderer，WinterTC logo 全部正确渲染。残余缺口 = WebP 解码未接入、CSS `url()` 背景图未抓取 |
| Flexbox 渲染 | 所有 flex 属性的正确布局和绘制 | 已有 taffy 支撑，主要验证 + 修复边界 case |
| Grid 渲染 | 所有 grid 属性的正确布局和绘制 | 已有 taffy 支撑，主要验证 + 修复边界 case |
| Float 布局 | 完整的 float 布局算法，float exclusion、clear、BFC 触发 | 当前仅有 inline context 的 float exclusion zone，无原生 float layout |
| Table 布局 | 完整的 table layout 算法，table-layout: auto/fixed、border-collapse、spanning | 当前属性已存储但无专用布局算法 |
| Multi-column 布局 | column-count/column-width 的实际列排布、column-rule、column-span | 当前属性已存储但无列布局算法 |
| 文字排版 | OpenType shaping（liga/kern/features）、BiDi 算法、CJK 排版优化、text-align justify、word-break/overflow-wrap、writing-mode、vertical text | 当前 fontdue 仅做简单 character-to-glyph 映射 |
| Position 定位 | absolute/relative/fixed/sticky 的精确坐标计算 | fixed/sticky 当前有简化处理 |
| Reftest 验证 | CPU 软件渲染模式 + GPU 渲染模式的截图对比 | 两种模式都需通过 |
| 产品静态页面视觉 smoke | `apps/browser/assets/welcome.html` 等内置静态页面、录制的真实静态文章页和图片密集静态站点必须通过 ZeroBrowser/WebView 路径与 Chromium 参考截图对比 | WPT 子集通过不能替代用户可见产品页验收；静态页无 JS 仍错位说明基础排版/绘制链路未达标 |
| 浏览器层 glyph 保真 | ZeroBrowser 消费 WebView `GlyphPrimitive` 时必须保持 engine 输出坐标、baseline、font size 和 fragment 边界的语义；字体 fallback 或选择功能不得重新排版整行 glyph | 浏览器层后处理不能把不同 grid/flex/card 中同一 baseline 的文本合并为一行 |
| 范围外 reftest 过滤 | 导入时自动过滤或标记范围外 reftest（SVG、Canvas、WebGL 等），维护 skip list | 防止范围外 case 膨胀分母 |
| 渲染缺口修复 | 任何导致 reftest 失败的渲染错误 | 由 reftest 结果驱动 |
| 渲染器图元覆盖 | CPU 渲染器和 GPU 渲染器必须能够渲染所有 `RenderPrimitives` 类型：fills、rounded_rects、gradients、shadows、images、strokes、path_fills、path_strokes、transforms、clips、filters、blend_modes、glyphs | ✅ **已实现（M7）**：CPU（crates/render-foundation/src/cpu/ 下 gradient.rs/shadow.rs/image/stroke.rs/effects.rs）+ GPU（gpu/renderer/mod.rs draw_gradient/image/rounded_rect_pass + collect_*_vertices + gpu/mesh.rs）均已实现全 13 种图元渲染并附单测；原 pre-M7「仅 3/2 种」描述已过时（详见 master.md R660） |
| 浏览器图元消费 | `append_webview_primitives()` 必须将所有 `RenderPrimitives` 类型传递到渲染器，不能静默丢弃 | ✅ **已实现（M7）**：`append_webview_primitives()`（app_render.rs:1512）遍历全 13 字段无丢弃（见 DC-10） |
| 渐变渲染 | 线性渐变、径向渐变、锥形渐变、重复渐变的 CPU + GPU 渲染 | ✅ **已实现（M7）**：CPU `render_gradient`（逐像素插值，cpu/gradient.rs）+ GPU `draw_gradient_pass` + `test_gpu_full_scene_gradient` |
| 阴影渲染 | `box-shadow` 的高斯模糊阴影渲染（offset + blur + spread + color） | ✅ **已实现（M7）**：CPU `render_shadow`（cpu/shadow.rs）+ GPU `collect_shadow_vertices` + `test_gpu_full_scene_shadow` |
| 图片渲染 | 背景图片（`background-image`）、`<img>` 元素、`list-style-image` 的图片解码和渲染 | ✅ **已实现（M7）**：CPU `render_image`（cpu/mod.rs）+ GPU `draw_image_pass`（纹理采样） |
| 线段/路径渲染 | `StrokePrimitive`（线段）、`PathFillPrimitive`（路径填充）、`PathStrokePrimitive`（路径描边）的渲染 | ✅ **已实现（M7）**：render_stroke/render_path_fill/render_path_stroke（cpu/stroke.rs）；原「渲染器未实现」已过时 |
| 变换渲染 | CSS 2D transform（translate、rotate、scale、skew、matrix）的正确应用 | `TransformPrimitive` ✅ 已实现（M7，CPU+GPU，见 DC-8/9）；原「渲染器未实现」描述已过时；3D transform 降级为 2D |
| 裁剪渲染 | `overflow: hidden/clip` 的矩形裁剪，`border-radius` 的圆角裁剪 | `ClipPrimitive` ✅ 已实现（M7，CPU+GPU，见 DC-8/9）；原「渲染器未实现」描述已过时；当前裁剪仅在浏览器层做像素级处理，不在渲染器层 |
| 滤镜渲染 | CSS filter（blur、brightness、contrast、grayscale、hue-rotate、invert、opacity、saturate、sepia、drop-shadow） | `FilterPrimitive` ✅ 已实现（M7，CPU+GPU，见 DC-8/9）；原「渲染器未实现」描述已过时 |
| 混合模式渲染 | `mix-blend-mode` 的 16 种混合模式（normal、multiply、screen、overlay、darken、lighten 等） | `BlendModePrimitive` ✅ 已实现（M7，CPU+GPU，见 DC-8/9）；原「渲染器未实现」描述已过时 |
| Margin 折叠 | 相邻块级元素 margin-top/margin-bottom 的正确折叠算法 | ✅ **已实现（R323 实测）**：taffy 0.7 `CollapsibleMarginSet` 内置块级 margin 折叠；R323 探针实测 6 case 全过（相邻兄弟 max 折叠 / 父子折叠 / border 阻断 / 负 margin 30+(-10)=20 / 祖父嵌套 max(40,0,35)=40 / BFC `overflow:hidden` 子不折叠），margin reftest 5/5 全绿（`block-in-inline-...-margin-collapse` 0.00%） |
| BFC（Block Formatting Context） | `overflow: hidden/auto/scroll`、`display: flow-root`、浮动等正确创建 BFC，隔离浮动和 margin 折叠 | 部分实现：BFC **margin 隔离**已工作（R323 实测 `overflow:hidden` 子元素 margin 不与父折叠）；`display:flow-root`/`is_layout_container` 标志已落地（R127 float-container margin-uncollapse 修复）。浮动包含（float containment）部分由 taffy + R129 float shrink-to-fit 覆盖 |
| 替换元素布局 | `<img>`、`<video>`、`<iframe>`、`<canvas>` 的固有尺寸计算和 `object-fit` | ✅ **已实现**（`apply_replaced_element_sizing` 三来源：HTML `width`/`height` 属性、SVG data URI、解码固有尺寸；R325 修 CSS §10 两侧显式尺寸不强制固有宽高比；`compute_object_fit_rect` 全 5 值；R318 图片数据端到端贯通）。`<video>`/`<iframe>`/`<canvas>` 固有尺寸仍按默认，非 reftest 杠杆 |
| 滚动容器 | `overflow: scroll/auto` 的可滚动容器，滚动偏移的正确应用 | 当前滚动偏移仅在浏览器层通过 `scroll_y` 手动偏移，无真正的滚动容器 |
| text-shadow | 文字阴影（offset + blur + color） | paint 阶段未生成 text-shadow 图元 |
| 多背景图层 | `background-image` 多层叠加渲染 | ✅ **已实现**：painter/effects.rs:134 遍历 `background_image` 全图层 `.rev()` 叠加渲染 + test_paint_multiple_overlapping_backgrounds；原「仅渲染第一个」已过时 |
| clip-path | CSS clip-path（circle、ellipse、polygon、inset） | ✅ **已实现（M9）**：painter/effects_indicators.rs + helpers.rs 全形状裁剪（原「仅生成指示器」描述已过时） |
| backdrop-filter | 元素背后内容的滤镜效果 | ✅ **已实现（M9，R894 实测验证）**：painter/effects.rs，blur 效果正确限定在元素盒内（原「完全未实现」描述已过时） |
| CSS mask | CSS 遮罩效果 | ✅ **已实现（M9）**：painter/effects.rs 渐变蒙版裁剪 + alpha 衰减（原「完全未实现」描述已过时） |
| 重复渐变 | `repeating-linear-gradient`、`repeating-radial-gradient` | ✅ **已实现**：cpu/gradient.rs:28 `if gradient.repeating` 处理 |

### 不在范围内（明确排除）

- **非 CSS 渲染领域的兼容性**：JS/DOM API 兼容性、网络协议兼容性、安全策略兼容性不在本目标范围内（由父目标 `zero-web.md` 覆盖）
- **Canvas / WebGL / WebGPU**：不在本目标 reftest 范围内
- **动画/交互的帧级正确性**：CSS animation/transition 的视觉正确性验证不作为 reftest 核心指标（但如果有 reftest 覆盖则需通过）
- **性能优化**：本目标关注渲染正确性，不关注渲染性能（由父目标的性能基准体系覆盖）
- **Chromium 专属行为**：只对齐标准规范行为，不复制 Chromium 的 bug 或非标准行为
- **新 crate 依赖的大规模引入**：最小化新依赖，仅在必要时引入许可证兼容的 crate
- **SVG 文档/内联 SVG 渲染**：不在本目标范围。作为 `<img>` / CSS `url()` 图片资源参与页面渲染的 SVG 栅格化属于“图片子资源与替换元素”范围，至少要覆盖产品静态 smoke 中的 Logo 场景

### 依赖约束

- **原则**：最小化新依赖引入
- **许可证**：如果必须引入新 crate，仅接受 MIT / Apache-2.0 / BSD 许可证
- **评估标准**：新依赖必须论证"不引入则无法达成 reftest 目标"的必要性
- **已知可能需要的新依赖**：
  - 文字排版 shaping：可能需要 `rustybuzz`（MIT）替代 fontdue 的简单 glyph 映射
  - BiDi 算法：可能需要 `unicode-bidi`（MIT/Apache-2.0）或 `icu_normalizer`
  - Chromium 截图：需要 Puppeteer 或 Playwright（通过 Node.js 脚本调用 headless Chromium）
  - WPT 工具：可能需要辅助工具来 fetch/解析上游 WPT 仓库
- **已有可复用基础设施**（M1 必须**扩展**而非重写）：
  - `tests/wpt-runner/src/reftest.rs`：像素对比引擎（`ReftestConfig`：`max_diff_ratio`, `max_channel_diff`）、`run_reftest()`、`compare_pixels()`、16 个内建 reftest case
  - `tests/wpt-runner/src/manifest.rs`：WPT MANIFEST.json 解析器、`filter_by_type()`、`filter_by_path_prefix()`
  - `crates/render-foundation/src/cpu.rs`：`render_scene_to_framebuffer()` — CPU 软件渲染截图（✅ M7 已支持全 13 种图元：gradient/shadow/image/stroke/path_fill/path_stroke/filter/blend 等）
  - `crates/render-foundation/src/gpu/`：GPU 渲染器 + WGSL shaders（✅ M7 已支持 rounded_rect/gradient/image/shadow/stroke/path 等 wgpu+mesh 管线）
  - `crates/render-foundation/src/primitive/mod.rs`：13 种图元类型定义（✅ 完整，Paint 系统已能全部生成）
  - `crates/engine/src/paint/`：Paint 系统（✅ 完整，能生成所有 13 种图元类型）
  - `crates/script-sandbox/`：V8 runtime — 用于 reftest harness 中执行 JS
  - `tests/wpt-runner/src/runner/mod.rs`：`TestExpectations` 机制 — 可扩展为 reftest skip list
- **渲染器扩展约束**（M7 必须遵循）：
  - CPU 渲染器：基于现有 `render_scene_to_framebuffer()` 函数扩展，不重写架构
  - GPU 渲染器：基于现有 wgpu + WGSL pipeline 扩展，不更换图形后端
  - 优先 CPU 渲染器实现（更简单、更易调试），然后映射到 GPU 渲染器
  - 每个新增图元渲染能力必须有对应单元测试

### 渐进覆盖策略

上游 WPT reftest 数以万计，按以下优先级分批导入。**最终目标**是导入每个上游 WPT 目录中**全部**范围内 reftest。分母 = 上游该目录全部 reftest − skip list 中范围外 reftest。不允许以「80% 已足够」为理由跳过任何范围内 reftest。

**Phase 1 — 基础设施 + CSS 2.1 核心导入**：
- 建立 WPT reftest 导入/运行/对比/报告基础设施
- 从上游 WPT 仓库自动 fetch 并导入 `css/css2/` + `css/CSS2/` 的**全部**范围内 reftest（从 `MANIFEST.json` 自动提取，不手动挑选）
- 建立初始基线（记录初始通过率，不要求达标但必须可测量）
- 修复发现的 CSS 2.1 渲染缺口

**Phase 2 — 布局模式全覆盖**：
- 从上游 WPT 导入 `css/css-flexbox/` + `css/css-grid/` 的全部范围内 reftest
- 从上游 WPT 导入 `css/css-position/` + `css/css-float/` + `css/css-tables/` + `css/css-multicol/` 的全部范围内 reftest
- 修复所有布局模式渲染缺口

**Phase 3 — 文字排版 + 全量扩展**：
- 从上游 WPT 导入 `css/css-text/` + `css/css-writing-modes/` + `css/css-fonts/` + `css/css-text-decor/` 的全部范围内 reftest
- 确保每个目录导入了上游该目录**全部**范围内 reftest（从 `MANIFEST.json` 自动提取，排除 skip list 中的范围外 case）
- 达到各领域上游真实 WPT reftest 通过率 ≥ 95%

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT Reftest 基础设施就位

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

### DC-2: CSS 2.1 核心通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css2/` 和 `css/CSS2/` 目录导入**全部**范围内 reftest（排除 skip list 中的范围外 case）
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] 覆盖：盒模型、margin 折叠、BFC、inline formatting、颜色、背景、边框、基础定位
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

### DC-3: Flexbox + Grid 通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-flexbox/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-grid/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

### DC-4: Positioning + Float + Table + Multicol 通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-position/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-float/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-tables/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-multicol/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

### DC-5: 文字排版通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-text/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-writing-modes/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-fonts/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-text-decor/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

### DC-6: Quirks Mode 完整实现

- [ ] CSS parser 在 quirks mode 下正确调整解析行为（quirky color values、quirky unitless lengths 等）
- [ ] Style system 在 quirks mode 下应用特定样式规则（如表格高度 quirks、百分比高度 quirks）
- [ ] Layout engine 在 quirks mode 下实现特定布局行为
- [ ] DOM parser 的 quirks mode 状态正确传递到 CSS parser → style system → layout engine 链路

### DC-7: 测试与质量不可退让

- [ ] 所有现有测试持续全绿（`cargo test` 零失败），包含移除 `#[ignore]` 后的全部测试
- [ ] **真实网站测试保留 `#[ignore]`**：`tests/integration/src/real_website_compat.rs` 中的真实网站兼容性测试因本地网络不稳定，保留 `#[ignore]` 标记，不计入本目标通过率统计。其余所有测试零 `#[ignore]`
- [ ] 所有新增渲染修复必须有对应单元测试覆盖
- [ ] `cargo build` 零错误、`cargo clippy` 零警告
- [ ] Reftest 通过率报告持久化到 `docs/goal/rendering-compat/evidence/` 目录
- [ ] 每轮执行的 reftest 通过率变化可追溯（有历史记录）

### DC-8: CPU 渲染器图元覆盖 100%

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

### DC-9: GPU 渲染器图元覆盖 100%

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

### DC-10: 浏览器图元消费完整性

- [x] ✅(M7) `append_webview_primitives()` 处理 `RenderPrimitives` 的所有 13 个字段（fills、rounded_rects、path_fills、path_strokes、strokes、gradients、shadows、images、glyphs、filters、blend_modes、transforms、clips）— app_render.rs:1512 遍历全 13 字段（1526-1840）无丢弃
- [x] ✅(R968) 所有图元类型正确应用 `scale_factor` 和 `offset` — `transform_webview_primitives`（app_render_primitives.rs）对全部 13 种图元应用 `out = in * scale + offset`；单测 `transform_webview_primitives_applies_scale_and_offset_to_all_types` 覆盖 fills / rounded_rects（4 圆角独立缩放）/ gradients（Linear/Radial/Conic 三变体；Conic `start_angle` 无量纲不缩放）/ shadows（offset+blur+spread）/ strokes（端点+线宽）/ glyphs（font_size）/ transforms（rect+origin+tx/ty），逐类型断言 scale=2 + offset=(10,20) 的精确输出
- [x] ✅(R968) 所有图元类型正确应用视口裁剪（`clip_y` + `clip_rounded`）— `transform_webview_primitives` 用 `ViewportClip`（轴对齐矩形）裁剪全部 13 种图元（rect 类走 `clip_rect_field`/`clip_axis_aligned_rect` 求交，path 走 `path_vertices_bbox`，glyph 走 font_size 包围盒）；单测 `transform_webview_primitives_culls_primitives_outside_viewport` 验证视口外 rounded_rects/gradients/path_fills/glyphs 被裁掉、视口内保留。注：`clip_y`（水平带）+ `clip_rounded`（圆角矩形）是 `append_webview_primitives`（fills+glyphs 混入 chrome 层）的细粒度裁剪，用于圆角 page frame；全 13 类主消费路径用 `ViewportClip`
- [x] ✅(R155) 图元渲染顺序遵循 CSS painting order（background → borders → content → outline）— `draw_order` 是 painter 生产默认（R149/R152/R155，painter/mod.rs:1459 `draw_order 是默认渲染路径`）；paint 系统按 CSS painting order 发射图元，浏览器消费层（本文件）保持发射顺序按类型重组，不重排

### DC-11: 布局正确性

- [x] **Margin 折叠** — 相邻块级元素的 margin-top/margin-bottom 按规范折叠（正 margin 取最大、负 margin 取最负、正负抵消）— ✅ **R323 实测通过**（taffy 0.7 CollapsibleMarginSet；6 探针 case + 5 reftest 全过）
- [x] ✅ **BFC 创建** — `overflow: hidden/auto/scroll`、`display: flow-root`、浮动元素、`position: absolute/fixed` 正确创建 BFC，隔离浮动和 margin 折叠（`establishes_bfc` 全条件 + margin 隔离 R323 实测 + `use_bfc_float_containment` float containment；见 known-gaps BFC 行 ✅）
- [x] ✅ **Float 布局** — float 定位（float: left/right）+ clear + float containment（BFC）+ inline exclusion — `float_positioning.rs::adjust_float_positions(_with_context)` 处理 float 定位 + active_left/right_float_bottom（clear 语义）+ `use_bfc_float_containment`（containment）；**R895 实测验证**（float:left 100×80 盒正确居容器左上、块级兄弟盒 border-box 全宽在 float 之后、inline 文本绕 float 排版 y<80 时 x<95 无文本/x≥100 有文本）；原 master.md「无原生 float 布局」描述已过时。残余 = CSS2 float reftest 边缘 case（结构性 plateau，非核心布局缺口）
- [x] **Position: fixed** — 相对 viewport 定位（当前错误地映射为 absolute）— ✅ **R324 修复**（`adjust_fixed_to_viewport` 改为扣除祖先偏移；fixed 在有偏移 positioned 祖先内也视口相对，与 R98 absolute-viewport 约定一致；新单测 + 8 旧单测更新 + 全量 reftest 438/490 零回归）
- [ ] **Position: sticky** — 滚动时正确固定在指定偏移范围内
- [ ] **Overflow: scroll/auto** — 可滚动容器功能，scroll 偏移正确应用到子元素布局
- [x] **替换元素** — `<img>` 的固有尺寸（intrinsic size）正确计算，`object-fit`（fill/contain/cover/none/scale-down）正确应用 — ✅ **R323 代码核查 + R325 修复**（`apply_replaced_element_sizing`：HTML `width`/`height` 属性 + SVG data URI + 解码固有尺寸三来源；CSS §10「两侧尺寸都显式时不强制固有宽高比」R325 修复，旧实现 `<img style="width:200px;height:50px">` 被 taffy 拉成 200×200；`compute_object_fit_rect` 全 5 值；R318 图片数据端到端贯通）
- [x] ✅ **百分比高度** — containing block 有明确高度时百分比高度正确解析；无明确高度时 height: auto（`clamp_percentage_max_height` engine.rs:1422 按 definite CB 高度解析 %，R168 table height-as-minimum）
- [x] ✅ **Auto margin 居中** — `margin: auto` 在 block/flex/grid 中正确居中（block/table R165 compute()根居中+table_shrink both_margins_auto；flex/grid taffy native auto-margin）
- [x] ✅ **min/max-width/height** — 约束正确应用到最终尺寸（`clamp_percentage_max_height` + R517 cascade 负值 max/min-height + R428 flex min-size:auto）

### DC-12: 高级视觉效果

- [x] ✅ **text-shadow** — 文字阴影渲染（offset + blur + color）— painter/text.rs:1067 `if has_text_shadow { push shadow glyph at (x+ox,y+oy,shadow_color) }` + test_paint_text_shadow_basic
- [x] ✅ **多背景图层** — `background-image` 多层叠加渲染 — painter/effects.rs:134 `for layer in background_image.iter().rev()` 全图层叠加 + test_paint_multiple_overlapping_backgrounds
- [x] ✅ **重复渐变** — `repeating-linear-gradient` / `repeating-radial-gradient` — cpu/gradient.rs:28 `if gradient.repeating {` 处理重复渐变
- [x] ✅ **border-image** — 边框图片渲染（slice + repeat + width）— painter/mod.rs 实现 + paint/tests/border_image_repeat.rs 单测（M9）
- [x] ✅ **clip-path** — CSS clip-path 基础形状（circle、ellipse、polygon、inset）— painter/effects_indicators.rs + helpers.rs 实现（M9）
- [x] ✅ **backdrop-filter** — 元素背后内容的滤镜效果 — painter/effects.rs 实现；**R894 实测验证**（gradient 背景 + overlay backdrop-filter:blur(15px)，with-vs-without 渲染 diff = 15314 px 恰落在 overlay 盒 y[0,80] 带、带外 0 px，证 blur 效果正确限定在元素盒内）
- [x] ✅ **CSS mask** — 基础遮罩效果（渐变蒙版裁剪 + alpha 衰减）— painter/effects.rs 实现（M9）
- [ ] **scroll-snap** — 滚动吸附行为（需宿主层滚动输入路由）
- [ ] **打印媒体查询** — `@media print` 基础支持（可选，降低优先级）

### DC-13: 产品静态页面视觉 smoke

- [ ] `apps/browser/assets/welcome.html` 通过 ZeroBrowser 窗口/无头路径截图，并与 Chromium 在相同 viewport 下的参考截图对比
- [ ] `https://morning.work/page/2026-02/fedora-macbook-three-finger-drag.html` 录制为固定 HTML/CSS fixture，并通过 ZeroBrowser/WebView/Chromium 三方截图对比；fixture 必须包含原页面依赖的 `/article.css`、`/styles/github.css`、`/JetBrainsMono/JetBrainsMono.css` 或明确记录不可用资源
- [ ] `https://wintertc.org/` 录制为固定 HTML/resource fixture，并通过 ZeroBrowser/WebView/Chromium 三方截图对比；fixture 必须包含内联 Twind CSS、`/static/logo.svg`、`/static/logos/*.svg`、`/static/logos/*.png` 等首页可见图片资源
- [x] ✅(R658) **Legacy Static Web smoke（HTML 3.2/4 + CSS1/2）**：建立固定 fixture 集并纳入 product-smoke 路径，首批至少 20 页，覆盖无 CSS 老式文档、HTML presentational attributes、表格布局、图片与文本环绕、列表/链接/标题、`font` 标签、`hr`、基础 CSS1/2 外链样式。每页必须有 Chromium oracle 截图和 ZeroWeb CPU 输出截图，失败时持久化 diff 与资源清单
- [x] ✅(R658, fixture 020) Legacy fixture 中必须包含类似 `testpage.htm` 的最小代表页：`BODY BGCOLOR/TEXT/LINK/VLINK`、`TABLE BORDER/CELLPADDING`、`TR BGCOLOR`、`IMG ALIGN=TOP`、`FONT SIZE`、`UL/LI`、`A href`。该页用于防止“WPT 分目录推进但老式静态页仍不可读”的回归
- [x] ✅(R658 逐项体检通过) Legacy Static Web smoke 的短期验收口径是“可读且结构不崩”：正文不重叠、不串行；表格单元格边框/内边距可见；图片按替换元素参与 inline 布局；链接颜色/下划线可见；`font size/color` 影响文本；body 背景和文本色生效。像素阈值可先作为趋势指标记录，不得用它替代 WPT/DC-14 达标口径
- [x] ✅(R213) URL 导航路径必须加载并应用 `<link rel="stylesheet">` 外部样式表；外链 CSS 抓取失败应作为可诊断的资源加载错误记录，不得静默退化为仅内联 CSS 渲染
- [x] ✅(R318) URL 导航路径必须加载 `<img src>` 图片子资源，将解码后的 SVG/PNG/JPEG/WebP 像素数据写入 `ImageCache`，并在 ZeroBrowser CPU/GPU 渲染路径传入 renderer；图片缺失不得被 alt 文本或占位 glyph 静默替代
- [x] ✅(R662) 同一输入通过 `zero-webview` 直接渲染路径截图，并与 Chromium 参考截图对比，避免产品层和 WebView 层互相掩盖问题 — `product-smoke --via-webview`（`render_via_webview_to_framebuffer` 走 `WebView::load_html` 嵌入边界）；welcome.html via-webview vs engine-direct **0.00% byte-identical**，两侧 vs Chromium 均 16.16%（字体墙），证明产品层↔WebView 层不互相掩盖
- [ ] 至少覆盖桌面和窄屏两个 viewport；桌面 viewport 下必须验证 hero 标题、四个 feature card、快捷键区、快速访问区和 footer 的相对位置
- [ ] welcome.html 自动检查文本不重叠、不同 sibling card/link/shortcut 的文本不串联、`ZeroBrowser` 标题在宽屏下不被错误拆行、`<br>` 后的中英文 tagline 保持两行
- [ ] morning.work 文章页自动检查 nav/title/date/tag badges/阅读时间不串联，正文段落不被压成同一行，inline code 保持行内位置，table 仍按表格布局绘制，pre/code 块保持独立背景和换行
- [ ] WinterTC 首页自动检查 header logo 可见、标题/副标题不串联、四个 nav button 分列、正文段落按宽度换行并保持 justify、参与方 Logo 网格中 SVG/PNG Logo 可见且不会退化为短横/alt glyph
- [ ] ZeroBrowser 不得对 WebView glyph 做会改变布局语义的整行重排；如需字体 fallback 或选择命中，应在不改变原始 glyph 坐标语义的路径上实现
- [x] ✅ 截图、对比报告和失败根因持久化到 `docs/goal/rendering-compat/evidence/product-static/`（welcome PNG + `legacy-html/` 20 fixture+diff-summary + `morning-work/` + `wintertc/` + `narrow/` + README + 各轮 rXXX evidence 根因分析）

### DC-14: 真通过标准（anti-false-pass）— 验证可信度门禁

> 本 DC 防止 reftest 通过率被「同源假通过」「宽容差」「子集分母」污染。**DC-2~13 的通过率数字只有在本 DC 同时满足时才可信、才计入达标判定。**

> **字体光栅化非渲染差异来源（2026-06-17 AA 基准实测）**：fontdue 与 chromium 对同一 glyph 光栅化几乎完全一致（W 0.1% / i 3.0%，见 `evidence/aa-baseline-2026-06-17.txt`）。welcome 26% / Oracle 污染 48.6% 的大头是**布局/度量（line-height / R109 inline→block / 多行结构）**，非字体光栅化。**勿再以「fontdue AA 噪声」为渲染差异归因**（纠正 R174/R187 误诊）；字体攻坚应停止，转向布局/度量。fontdue 无需替换。

> **★ 多行 y 堆叠已修（R630，2026-06-25，commit d31cf03a）**：「多行结构」差异的一个具体子项——paint Path B 对 auto-wrap 多行块用 `all_fragments()`（y 恒 0）致**多行文字垂直堆叠看不清**——已修复（统一用 `all_fragments_with_line_y()`）。这是用户可见「文字堆叠」的直接修复，同源 reftest net +24（normal-flow +6 / positioning +19）。**注意区分**：font-weight 加粗（R229c 证伪，字体墙死路）≠ 多行 y 堆叠（paint 逻辑 bug，已修）——「真实网站没法看」是多个独立 bug 叠加，逐个定位比笼统归「字体架构」有效。残余行级度量差异（product-smoke morning/welcome +0.3~0.8pp）是 R374 字体匹配问题（堆叠掩盖→分行显现），独立多会话。详见 master.md R630 + [`evidence/r630-paint-pathb-multiline-y-fix-2026-06-25.txt`](./evidence/r630-paint-pathb-multiline-y-fix-2026-06-25.txt)。

> **★ 字体归因三证推翻（R229c/R631，2026-06-25）**：「字体问题」是误诊——三角度全证伪字体为 diff 主因：(1) font-weight 加粗（R229c）product-smoke 5 组参数全退步；(2) 字体选择对齐（R631）强制 sans-serif→NotoSansCJK（chromium 经 fontconfig 的同款字体）后 morning 17.16→17.15% / welcome 17.27→17.20% **零变化**，推翻 R374「字体不匹配」归因；(3) 光栅化（R388）fontdue≈chromium。morning/welcome 17% 真因 = **布局/行盒度量**（line-height/baseline/行间距），非字体。**勿再以「字体问题」为真实网站 diff 归因**——真正 lever = Phase A 行盒度量统一（line-height 计算/baseline 定位/行盒 y，R630 已修多行 y 堆叠子项）。详见 [`evidence/r631-font-match-refuted-2026-06-25.txt`](./evidence/r631-font-match-refuted-2026-06-25.txt)。

> **★ 行盒度量连续修复（R630/R632，2026-06-25）**：确证 morning/welcome 17% 真因是行盒度量后，连续两步实质进展——R630（commit d31cf03a）修 paint Path B 多行 y 堆叠（同源 net +24）；R632（commit 0911a2ac）修 paint Path B line-height 忽略 CSS（compute_final 不存 override → fallback 19.2，line-height 1.5/2.0 产出相同行位置；修复后正确响应 CSS，reftest net +5，welcome -1.11pp）。R627 的 pre-wrap -15 被 R630 吸收。残余：morning 中文 +0.99pp = frag.height 字体**度量**（NotoSansCJK 行高 ≠ chromium，R374 谱系，区别于字体选择 R631 已证伪）。下一步 = baseline 定位 / 字体度量统一。详见 [`evidence/r632-line-height-override-fix-2026-06-25.txt`](./evidence/r632-line-height-override-fix-2026-06-25.txt)。

- [x] ✅(R669) **独立 Oracle（reference 不得由被验证者自渲染）**：reftest 的参考基准必须是 **Chromium 渲染 test.html**，不得是 ZeroWeb 自渲染 ref.html。**✅ R669 落地 `zero-wpt-runner reftest-oracle` 子命令 + `make reftest-oracle [DIR=...] [ORACLE_PASS_RATIO=...]`**：渲染上游 WPT test 页（`render_to_framebuffer_with_base`）vs chromium oracle-shot（`oracle-shots/{safe_id}.png`，**13793 张**，经 `capture-chromium-screenshots.mjs`/`capture-oracle-per-dir.mjs` 抓取），报告 chromium-Oracle 真一致率（z_vs_chr < `ORACLE_PASS_RATIO`，默认 1%）+ top-15 最差发散修复候选 + per-dir 分解 + self-source 假通过对照。**doc-maintainer spot-check 复现**：`make reftest-oracle DIR=css-grid` = 16/49 = **32.7%**（z_vs_chr<1%），与 R560 文档基线 + self-source ~56.5%/DC-14 46.5% 假通过一致。**范围注（诚实）**：默认 `reftest` 路径仍 ZeroWeb self-ref（保留作同源自一致性参考），R669 的 `reftest-oracle` 作为**一等独立 Oracle 指标**补充——满足本项「至少抽样跑 ZeroWeb-test vs Chromium-test + 量化污染比例」要求，且覆盖**全量 corpus 优于抽样**。原「闲置 capture-chromium-screenshots.mjs」已接入。原「reftest.rs:230-232 用 ZeroWeb 渲染 ref」描述适用于默认 reftest 路径，reftest-oracle 路径已用 chromium Oracle
- [x] ✅(R852 oracle + R970 self-source) **非平凡性检查**：拒绝 `test == ref` 且接近纯色（或 PNG 退化）的 case 自动判 PASS——必须标记为「可疑/退化」并单独审计，防止 harness PNG 加载 bug 等导致的退化假绿（历史已发生，见 `archive/rounds-r23-r139.md` R135/R149）。**R852 落地 oracle 路径**：`frame_is_near_solid`（采样每 16 像素，主色占比 >99.9% 判退化）+ 报告「退化可疑 pass 排除 + credible pass + 审计列表」；实测 3% corpus 近纯色（parsing/animation/print/crashtest headless 空白）；**R970 落地 self-source 路径**：`frame_is_near_solid` 移到 `reftest_compare`（pub，两路径共享）+ `ReftestResult.test_near_solid` 字段 + `run_reftest_with_base`/`run_reftest_gpu_with_base` 计算；`print_dc14_three_state` 把 strict-pass 拆成 可信(非近纯色)/可疑(近纯色，列审计列表)；内置 css21 实测 可信 569(82.9%)/可疑 70(10.2%，多为简单 smoke 理性近纯色)/近似 47(6.9%)/不一致 0
- [ ] ⚠️(R851 oracle 路径已实现·self-source 三态已实现 R969 / 非平凡性仍 pending) **严格容差复跑 + 三态分类**：必须在文档锁定容差（布局 ≤ 0.1% / 文字 ≤ 0.5%，优先 WPT fuzzy 注解）下复跑全量，输出 **真通过 / 近似通过（超锁定容差但更宽松）/ 假通过（退化或同源）** 三态。唯一可信达标指标 = **严格容差真通过率**。当前 vertical-rl clearance 用 5% 容差属近似通过，不计入真通过。**R851 落地 oracle 路径三态**（strict<0.1%/0.5% / near(strict..1%) / mismatch）；全量 corpus 实测 loose 38.4%、揭示 strict 真通过率（positioning 0.6% vs loose 45.6%，字体光栅噪声主导）；**R969 落地 self-source 路径三态**：`print_dc14_three_state`（cmd_reftest + cmd_reftest_upstream 调用）——`strict_pass + near_pass == pass_count`、`mismatch == fail_count` 自洽，strict 边界 = `category.strict_max_diff_ratio`（0.1%/0.5%）+ `strict_max_channel_diff`（2/5），near/mismatch 边界用 `result.passed`（编码实际有效 loose 阈值含 fuzzy override）；内置 css21 实测 strict 639 (93.1%) / near 47 (6.9%) / mismatch 0（loose 100% 含 6.9% 字体噪声近似通过）；**self-source 非平凡性（test==ref 退化）仍 pending**（须 frame 接入 `frame_is_near_solid`，下轮 slice）
- [ ] **容差锁定不可放宽**：布局类 ≤ 0.1%、文字类 ≤ 0.5% 为硬上限。不允许以「实测校准」「字体差异」为由放宽容差；文字类大面积失败必须修渲染，不得放宽容差
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

---

## Current Proven Baseline

截至 2026-06-06，项目渲染兼容性现状：

### 已有能力

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
| 浏览器图元消费 | ✅ **已实现（M7）** | `append_webview_primitives()`（app_render.rs:1512）遍历全 13 字段无丢弃 |
| Margin 折叠 | ✅ 已实现（taffy 0.7 CollapsibleMarginSet；R323 实测 6 探针 case + 5 reftest 全过） | 块级元素间距与主流浏览器一致 |
| BFC（margin 隔离部分） | ✅ 已实现（overflow:hidden/flex/grid 等 BFC 的子元素 margin 不与父折叠；R323 实测） | margin 折叠隔离正确；浮动包含（float containment）部分未单独验证 |
| 滚动容器 | ⚠️ 简化处理 | 无真正滚动容器，浏览器层手动偏移 |

### 已知关键缺口

| 缺口 | 影响范围 | 严重性 | 当前状态 |
|------|----------|--------|----------|
| **渲染器图元覆盖** | **所有视觉输出** | ✅ **已实现（M7）** | CPU（cpu/gradient.rs+shadow.rs+image+stroke.rs+effects.rs filter/blend）+ GPU（gpu/renderer/mod.rs draw_gradient/image/rounded_rect_pass + collect_shadow/stroke/path_fill/path_stroke/color_filter/blur_filter + mesh）均已实现全 13 种图元，附单测（test_gpu_full_scene_gradient/shadow/stroke 等）。**注**：granular DC-8/9 各项的 framebuffer 像素断言 rigor 仍待逐项复核。原「CPU 仅 3 种 / GPU 仅 2 种 / 全部无法渲染」描述已过时（pre-M7） |
| **浏览器图元消费** | **所有视觉输出** | ✅ **已实现（M7）** | `append_webview_primitives()`（app_render.rs:1512）遍历 `RenderPrimitives` 全 13 字段（line 1526-1840）无静默丢弃。原「仅 fills+glyphs，11 种丢弃」描述已过时（pre-M7） |
| **Inline formatting 所有权分裂** | **静态页面基础排版** | **P1-严重** | inline/inline-block 在 taffy 中映射为 block，同时 IFC 又通过 `text_content()` 收集 inline 子树文本；父容器和子 inline 盒可能重复或错位绘制文本。`welcome.html` 中 `ZeroBrowser`、card/link/shortcut 文本串联是该类缺口的产品可见症状 |
| **Layout/Paint IFC 双路径** | **文本布局与 glyph 输出一致性** | **P1-严重** | layout 阶段和 paint 阶段不是同一份 IFC 结果；paint 二次运行 IFC 时 style map、float exclusion、container width 可能不同，导致 box 背景位置与 glyph 位置不一致 |
| ~~**外部样式表加载缺失**~~ | **真实静态网页 CSS** | ✅ **已贯通（R213）** | ✅ **已修复**：fetch_url 三条成功路径（SW 拦截 line 396 / HTTP 缓存命中 421 / 正常 fetch 448）现均 `load_html(&html, Some(&external_css))`（非 None）；prepare_page_subresources → resolve_external_css（webview.rs:256）经 extract_stylesheet_hrefs 提取 `<link rel="stylesheet">` + base URL 解析 + 逐个 HTTP 抓取 + 合并注入级联，抓取/解码失败记 `tracing::warn!`（274-276）不阻断（宽松降级）；R213 测试 test_fetch_url_loads_external_stylesheet + ..._missing_does_not_break 覆盖。~~原（已过时）~~： `WebView::fetch_url()` 三条成功路径都会调用 `load_html(&html, None)`；`RenderPipeline::collect_stylesheets()` 只收调用方传入 CSS 和文档内 `<style>`，不抓取 `<link rel="stylesheet">`。morning.work 文章页依赖外链 CSS，当前会静默退化为仅内联样式 |
| ~~**图片子资源/ImageCache 未贯通**~~ | **Logo/图片密集静态页面** | ✅ **已贯通（R318 实测）** | `<img>` paint 生成 `ImagePrimitive`；`WebView::fetch_image_subresources`（webview.rs:265）在 `fetch_url` 导航三条路径（line 370/395/423）抓取 + 解码 `<img src>`（PNG/JPEG 魔数 + SVG via resvg/tiny-skia），写入 `image_cache`；`app_platform.rs` render_cpu/render_gpu/render_frame 三处传 `Some(&mut webview.image_cache())`（非 None）。**R318 实测**：WinterTC 首页 header logo + 13 个参与方 SVG/PNG logo（alibaba/bytedance/cloudflare/deno/fastly/igalia/netlify/nodejs/shopify/suborbital/vercel/azion/matrix）全部正确渲染（非占位 glyph），产品 smoke diff=13.70%（残余为 system-ui 字体度量/line-height，非图片缺口）。原「传 None / Logo 缺失」描述已过时 |
| **浏览器层 glyph 重排** | **ZeroBrowser 产品渲染路径** | **P1-严重** | ZeroBrowser 在消费 WebView 图元前会按 baseline 对 glyph 做后处理重排；该逻辑可能破坏 engine 已经计算好的 fragment x 坐标，尤其影响 grid/flex 中同一 baseline 的不同卡片文本 |
| **真实静态页面 smoke 缺失** | **验收有效性** | **P1-严重** | 当前没有把 `apps/browser/assets/welcome.html`、morning.work 文章页和 WinterTC 图片密集首页这类无页面 JS 的真实静态页面作为 Chromium 截图对比门禁；因此 WPT 子集或内联 reftest 全绿仍可能漏掉用户第一眼可见的错位、正文重叠、tag 串联、表格退化和 Logo 缺失 |
| ~~**Margin 折叠**~~ | CSS 2.1 布局正确性 | ✅ **已实现（R323 实测）** | taffy 0.7 `CollapsibleMarginSet` 内置；R323 探针 6 case（相邻/父子/border 阻断/负 margin/祖父嵌套/BFC 子不折叠）全过 + margin reftest 5/5 全绿。原「完全未实现」描述过时 |
| ~~**BFC**~~ | 布局隔离 | ✅ **已实现** | `establishes_bfc`（全条件：overflow/float/abspos/flow-root/flex/grid/table/multicol）接线生产；margin 隔离 R323 实测 6 探针全过；`use_bfc_float_containment` 落地 float containment。原「无 BFC 概念，overflow: hidden 不隔离浮动、不阻止 margin 折叠」描述过时 |
| ~~**替换元素**~~ | 图片/媒体渲染 | ✅ **已实现** | `<img>` 固有尺寸（HTML 属性 + SVG data URI + 解码固有尺寸三来源）已实现；R325 修 CSS §10 两侧显式尺寸不强制固有宽高比（旧 `<img style="width:200px;height:50px">` 被拉成 200×200）；`compute_object_fit_rect` 全 5 值；R318 图片数据端到端贯通。原「无固有尺寸计算，图片无法正确显示」描述过时 |
| **滚动容器** | 页面滚动 | P1-严重 | overflow: scroll/auto 无真正滚动，长页面无法正确浏览 |
| Float 布局 | CSS 2.1 核心功能 | P2-中等 | 仅有 inline context 的 float exclusion zone，clear 和 float containment 不完整 |
| ~~Position: fixed~~ | 视口定位 | ✅ **R324 已修复** | `adjust_fixed_to_viewport` 改为扣除累积祖先偏移（旧「加上」仅 parent_offset=0 时正确）；fixed 在有偏移 positioned 祖先内也视口相对，与 R98 absolute-viewport 约定一致。新单测 + 8 旧单测更新 + 全量 reftest 零回归 |
| Position: sticky | 滚动吸附 | P2-中等 | 需 host layer 动态调整，未完整实现 |
| text-shadow | 文字效果 | P2-中等 | paint 阶段未生成 text-shadow 图元 |
| ~~多背景图层~~ | 视觉丰富度 | ✅ **已实现** | effects.rs:134 全图层 `.rev()` 叠加（原「仅第一个」已过时） |
| ~~重复渐变~~ | 视觉丰富度 | ✅ **已实现** | cpu/gradient.rs:28 `if gradient.repeating`（原「未实现」已过时） |
| ~~clip-path~~ | CSS 裁剪 | ✅ 已实现（M9） | painter/effects_indicators.rs + helpers.rs 全形状裁剪（原「仅生成指示器」已过时） |
| ~~backdrop-filter~~ | 模糊背景 | ✅ 已实现（M9，R894 实测） | painter/effects.rs；blur 效果正确限定元素盒内（原「完全未实现」已过时） |
| ~~CSS mask~~ | 遮罩效果 | ✅ 已实现（M9） | painter/effects.rs 渐变蒙版裁剪 + alpha 衰减（原「完全未实现」已过时） |
| 3D transform | 3D 效果 | P3-低 | 仅 2D 支持，3D 函数忽略 |
| ~~真实 WPT reftest~~ | 验证有效性 | ✅ **已实现（R484）** | R484 完成 10/10 目录全量去子集化——从上游 WPT（`https://github.com/web-platform-tests/wpt`）`MANIFEST.json` 自动提取并导入 **~9967 个真实 reftest**（css2/CSS2/flexbox/grid/position/float/tables/multicol/text/writing-modes/fonts/text-decor 全覆盖），并建 DC-14 chromium Oracle 一致率基线 3608/9967=36.2%（chr<1%）。原 685 个 inline reftest 现仅作 smoke（不计入通过率）。原「685 inline 未用真实 WPT」描述已过时（pre-R484） |

### 测试基线

- 总测试数：~12,001，全绿
- Coverage：95.46% line, 96.94% function, 94.88% region
- Inline reftest：685 个，100% 通过（⚠️ 手写简单场景，容差过宽松，**不计入本目标通过率统计**。本目标的通过率必须基于上游真实 WPT reftest）
- **关键事实**：当前 WPT runner 是 smoke test，不证明渲染正确性。本目标的核心挑战是从"不崩溃"升级到"渲染正确"。（历史：M7 前渲染器仅支持 3/13 种图元；**M7 后 CPU/GPU 均已支持全 13 种图元**——见 Current Proven Baseline 表。当前 reftest 通过率受字体度量 / 布局结构性 plateau 限制，非图元覆盖限制。）

---

## Single Active Milestone

**当前活跃里程碑**：✅ **M7 已完成**（渲染器图元覆盖 + 浏览器图元消费）—— DC-8 CPU 13/13 + DC-9 GPU 13/13 + DC-10 浏览器消费全 13 字段，均附 framebuffer 像素断言测试（见 master.md R660-R666）。**当前实际活跃工作**：M10（上游 WPT 真实 reftest 通过率）+ DC-13 产品 smoke + 真 incomplete 特性（Float/sticky/scroll 宿主层 + scroll-snap + @media print；clip-path/backdrop-filter/mask 已 ✅ M9/R894）+ Phase A 字体度量 plateau（按 rally 跨会话续跑协议推进）。

### M7 目标

消除渲染管线中最大的视觉输出缺口：让 CPU 渲染器、GPU 渲染器和浏览器 `append_webview_primitives()` 能够处理所有 13 种 `RenderPrimitives` 图元类型。完成此里程碑后，页面将从「只有色块和文字」升级到「接近主流浏览器的视觉输出」。

### M7 背景事实（⚠️ pre-M7 历史快照；M7 已完成，「断桥」已修复，见 DC-8/9/10 ✅）

**当时（pre-M7）**渲染管线存在一个严重的「断桥」（下列 3 项现已全部修复——CPU/GPU/浏览器消费全 13 图元）：
1. **Paint 系统**（`crates/engine/src/paint/`）已能生成 13 种图元类型 ✅
2. **CPU 渲染器**仅渲染其中 3 种（fills、rounded_rects、glyphs）❌
3. **GPU 渲染器**仅渲染其中 2 种（fills、glyphs）❌
4. **浏览器 `append_webview_primitives()`** 仅传递 2 种到渲染器 ❌

这意味着渐变、阴影、图片、线段（边框虚线/点线）、路径、变换、裁剪、滤镜、混合模式全部在渲染阶段被静默丢弃。

### M7 完成标准

1. [x] ✅ **CPU 渲染器**（`crates/render-foundation/src/cpu/`）实现以下图元渲染（DC-8 13/13，全 framebuffer 像素断言测试）：
   - `GradientPrimitive` — 线性/径向/锥形渐变（逐像素插值或分段近似）
   - `ShadowPrimitive` — 高斯模糊阴影（box-blur 近似或高斯核卷积）
   - `ImagePrimitive` — RGBA 像素数据合成到 framebuffer
   - `StrokePrimitive` — 线段渲染（支持 solid/dashed/dotted + LineCap）
   - `PathFillPrimitive` — 多边形扫描线填充
   - `PathStrokePrimitive` — 多边形描边
   - `TransformPrimitive` — 仿射变换应用到后续图元坐标
   - `ClipPrimitive` — 矩形裁剪（像素级 discard）
   - `FilterPrimitive` — 至少 blur（box-blur 近似）和 opacity
   - `BlendModePrimitive` — 至少 normal、multiply、screen
2. [x] ✅ **GPU 渲染器**（`crates/render-foundation/src/gpu/`）实现以下图元渲染（DC-9 13/13，wgpu+mesh 管线 + framebuffer 像素断言测试）：
   - `RoundedRectPrimitive` — 扩展 WGSL shader 支持圆角
   - `GradientPrimitive` — 渐变 shader（1D texture lookup 或 shader 内插值）
   - `ShadowPrimitive` — 阴影 pass（模糊 texture 或近似）
   - `ImagePrimitive` — 图片纹理上传和采样
   - `StrokePrimitive` — 线段顶点生成
   - `TransformPrimitive` — 顶点坐标变换
   - `ClipPrimitive` — scissor rect 或 stencil buffer
   - `FilterPrimitive` — post-processing pass（至少 blur）
   - `BlendModePrimitive` — blend equation 配置
3. [x] ✅ **浏览器图元消费**（`apps/browser/src/app_render.rs`）：`append_webview_primitives()` 遍历全 13 字段（DC-10 item 1 ✅）：
   - `append_webview_primitives()` 处理 `RenderPrimitives` 的所有 13 个字段
   - 所有图元类型正确应用 `scale_factor` 和 `offset`
   - 所有图元类型正确应用视口裁剪（`clip_y` + `clip_rounded`）
   - 图元渲染顺序遵循 CSS painting order
4. [x] ✅ **验证**：`cargo test` 全绿（render-foundation 504 测试，R666）+ `cargo clippy` -D warnings 零警告
5. [x] ✅ **视觉验证**：每种图元有独立 framebuffer 像素断言单测（test_gpu_full_scene_* / cpu_full_scene_*，DC-8/9 13/13），生成已知输入图元 → 渲染到 framebuffer → 断言关键像素值正确（不允许仅凭「看起来对了」声称通过）：
   - 渐变：输入 `GradientPrimitive { kind: Linear, stops: [red→blue] }` → 断言矩形左端像素为红色、右端为蓝色
   - 阴影：输入 `ShadowPrimitive { blur_radius: 10, color: black }` → 断言阴影区域的 alpha 渐变存在
   - 图片：输入 `ImagePrimitive { RGBA 数据 }` → 断言输出像素与输入一致
   - 线段：输入 `StrokePrimitive { style: Dashed }` → 断言输出包含间断的线段像素
   - 变换：输入 `TransformPrimitive { rotate: 90° }` + 子图元 → 断言子图元坐标被正确旋转
   - 裁剪：输入 `ClipPrimitive` + 超出裁剪区域的子图元 → 断言超出部分不渲染
   - 滤镜：输入 `FilterPrimitive { blur }` → 断言输出被模糊
   - 混合模式：输入 `BlendModePrimitive { multiply }` → 断言混合计算正确

### M7 影响范围

- **主要修改**：
  - `crates/render-foundation/src/cpu.rs`（CPU 渲染器扩展）
  - `crates/render-foundation/src/gpu/renderer.rs`（GPU 渲染器扩展）
  - `crates/render-foundation/src/gpu/pipeline.rs`（WGSL shader 扩展）
  - `apps/browser/src/app_render.rs`（`append_webview_primitives()` 扩展）
- **可能修改**：
  - `crates/render-foundation/src/primitive/mod.rs`（如需调整图元定义）
  - `crates/render-foundation/src/gpu/`（新增 shader 文件、纹理管理）
- **不修改**：
  - `crates/css-parser/`、`crates/style-system/`、`crates/layout-engine/`（M7 不改解析/布局逻辑）
  - `crates/engine/src/paint/`（Paint 系统已能正确生成所有图元）

### M7 技术约束

- CPU 渲染器保持逐像素/逐行扫描的方式，不引入 GPU 依赖
- GPU 渲染器基于现有 wgpu + WGSL 架构扩展，不更换图形后端
- GPU 渲染必须是独立的 GPU 渲染管线（wgpu + WGSL shaders），**不允许**将 GPU 渲染实现为对 CPU 渲染器的 passthrough 调用。每种图元类型在 GPU 渲染器中必须有对应的 GPU 端实现（vertex shader / fragment shader / compute shader）
- 优先 CPU 渲染器实现（更简单、更易调试），然后映射到 GPU 渲染器
- 每个图元类型的实现必须有对应单元测试（输入图元 → 输出像素 → 验证）

---

## Ordered Next Milestones

### M2 — CSS 2.1 核心渲染修复 + Quirks Mode

**目标**：修复 CSS 2.1 reftest 发现的渲染错误，实现完整 quirks mode，达到 CSS 2.1 核心通过率 ≥ 95%（基于上游真实 WPT reftest）。

**范围**：
- 盒模型计算精度
- Margin 折叠
- BFC 触发与隔离
- Inline formatting 正确性
- 颜色和背景绘制
- 边框绘制（border-radius、border-style）
- 基础定位（static/relative/absolute）
- Float 基础布局（含 clear）
- **Quirks mode 完整实现**：
  - CSS parser：quirky color values、quirky unitless lengths、quirky hash-less color
  - Style system：quirks mode 特定样式规则（表格高度 quirks、百分比高度 quirks、inline 元素宽高 quirks）
  - Layout engine：quirks mode 特定布局行为
  - DOM parser quirks mode 状态传递到下游链路

**依赖**：M1 完成（需要 reftest 基础设施来验证修复）

### M3 — Flexbox + Grid 渲染修复

**目标**：修复 Flexbox 和 Grid reftest 发现的渲染错误，达到各自通过率 ≥ 95%（基于上游真实 WPT reftest）。

**范围**：
- Flexbox 所有子属性的正确布局
- Grid 所有子属性的正确布局
- 响应式布局 edge case
- 嵌套 flex/grid 场景

**依赖**：M1 完成

### M4 — Float + Table + Multicol 布局算法实现

**目标**：实现缺失的布局算法（Float、Table、Multi-column），达到各自 reftest 通过率 ≥ 95%（基于上游真实 WPT reftest）。

**范围**：
- 完整 float 布局算法
- 完整 table layout 算法（table-layout: auto/fixed、border-collapse、spanning）
- Multi-column 布局算法
- position: fixed/sticky 的精确实现

**依赖**：M1 完成（M2/M3 可并行）

### M5 — 文字排版能力实现

**目标**：实现完整的文字排版能力，达到文字排版 reftest 通过率 ≥ 95%（基于上游真实 WPT reftest）。

**范围**：
- OpenType shaping（ligatures、kerning、features）— 可能引入 `rustybuzz`
- BiDi 算法实现 — 可能引入 `unicode-bidi`
- CJK 排版优化
- writing-mode: vertical-* 实现
- text-align: justify 的精确实现
- word-break / overflow-wrap / hyphens 的完整实现
- text-decoration 的精确绘制

**依赖**：M1 完成（M2/M3/M4 可并行）

### M6 — 全量扩展 + 通过率冲刺（已声称完成）

**目标**：扩大各领域 reftest 覆盖范围，达到总体 95%+ 通过率。

**范围**：
- 扩大每个目录的 reftest 导入数量（目标每个目录 ≥ 100 个 case）
- 修复所有剩余渲染缺口
- CPU + GPU 双模式验证
- 回归测试确保已通过的 case 不退化

**依赖**：M2-M5 完成

**状态**：⚠️ 已声称完成（685/685 inline reftest 100% 通过），但审计发现这些 reftest 均为手写简单场景，**不是上游 WPT 真实 reftest**，未覆盖渲染器实际输出能力缺口。真实渲染效果仍然与主流浏览器差距巨大。后续 M7-M11 里程碑旨在解决这些根本问题。**本目标的通过率标准必须基于上游真实 WPT reftest，685 个 inline reftest 不计入通过率统计。**

### M7 — 渲染器图元覆盖 + 浏览器图元消费（Critical Path）

**目标**：消除渲染管线最大的视觉输出缺口 — 让 CPU/GPU 渲染器和浏览器 `append_webview_primitives()` 能处理所有 13 种 `RenderPrimitives` 图元类型。

**范围**：
- CPU 渲染器（`crates/render-foundation/src/cpu/`）：新增 GradientPrimitive、ShadowPrimitive、ImagePrimitive、StrokePrimitive、PathFillPrimitive、PathStrokePrimitive、TransformPrimitive、ClipPrimitive、FilterPrimitive、BlendModePrimitive 渲染能力
- GPU 渲染器（`crates/render-foundation/src/gpu/`）：扩展 WGSL shader + 顶点格式，新增 RoundedRectPrimitive、GradientPrimitive、ShadowPrimitive、ImagePrimitive、StrokePrimitive、TransformPrimitive、ClipPrimitive、FilterPrimitive、BlendModePrimitive 渲染能力
- 浏览器（`apps/browser/src/app_render.rs`）：`append_webview_primitives()` 处理所有 13 个 `RenderPrimitives` 字段
- 图元渲染顺序遵循 CSS painting order（background → borders → content → outline）

**背景事实**：Paint 系统已能生成 13 种图元，但 CPU 渲染器仅渲染 3 种、GPU 渲染器仅渲染 2 种、浏览器仅传递 2 种。渐变、阴影、图片、边框虚线、路径、变换、裁剪、滤镜、混合模式全部在渲染阶段静默丢弃。

**依赖**：无（可与 M2-M6 并行，但建议优先完成，因为这是视觉输出最大的瓶颈）

**⚠️ 关键要求**：M7 的验证不能仅依赖手写 inline reftest。M7 完成标准中的图元渲染验证必须通过以下方式之一：（1）单元测试直接验证 framebuffer 像素输出（推荐），或（2）从上游 WPT 导入至少 20 个涉及渐变/阴影/图片/边框的真实 reftest 来验证。不允许仅凭手写 inline reftest 声称 M7 完成。

### M8 — 布局正确性（Margin 折叠 + BFC + Float + Replaced Elements）

**目标**：实现 CSS 2.1 核心布局算法，使块级布局结果与主流浏览器一致。

**范围**：
- **Margin 折叠算法** — 相邻块级元素 margin-top/margin-bottom 折叠（正取最大、负取最负、正负抵消）；父子 margin 折叠
- **BFC（Block Formatting Context）** — `overflow: hidden/auto/scroll`、`display: flow-root`、浮动元素、`position: absolute/fixed` 正确创建 BFC；BFC 包含浮动、隔离 margin 折叠
- **Float 布局完善** — 完整的 float 定位、clear（clear: left/right/both）、float containment
- **Position: fixed** — 相对 viewport 定位（修复当前错误映射为 absolute 的问题）
- **Position: sticky** — 滚动时正确固定在指定偏移范围内
- **替换元素** — `<img>` 固有尺寸（intrinsic size）计算，`object-fit` 正确应用
- **百分比高度** — containing block 有明确高度时百分比高度正确解析
- **Auto margin 居中** — `margin: auto` 在 block 中正确水平居中
- **min/max-width/height** — 约束正确应用到最终尺寸

**依赖**：M7 完成（需要渲染器能正确渲染才能验证布局结果）

### M9 — 滚动容器 + 高级视觉效果

**目标**：实现滚动容器功能和高级 CSS 视觉效果。

**范围**：
- **滚动容器** — `overflow: scroll/auto` 创建可滚动容器；scroll 偏移正确应用到子元素；scrollbar UI（至少 native scrollbar）
- **text-shadow** — 文字阴影渲染（offset + blur + color）
- **多背景图层** — `background-image` 多层叠加渲染
- **重复渐变** — `repeating-linear-gradient` / `repeating-radial-gradient`
- **border-image** — 边框图片渲染（slice + repeat + width）
- **clip-path** — CSS clip-path 基础形状（circle、ellipse、polygon、inset）
- **backdrop-filter** — 元素背后内容的滤镜效果
- **CSS mask** — 基础遮罩效果（至少 image mask）
- **scroll-snap** — 滚动吸附行为

**依赖**：M7 完成（需要渲染器支持所有基础图元）

### M10 — 上游 WPT 真实 Reftest 导入与验证

**目标**：从上游 WPT 仓库导入**全部**范围内真实 reftest，建立可信的渲染正确性验证基线。所有后续通过率统计必须基于这些上游真实 reftest。

**范围**：
- **上游 WPT 真实 reftest 导入** — 从上游 WPT 仓库（`https://github.com/web-platform-tests/wpt`）自动 fetch 并导入以下目录的**全部**范围内 reftest（从 `MANIFEST.json` 自动提取 reftest 列表，排除 skip list 中的范围外 case，不允许手动筛选或挑拣）：
  - `css/css2/`、`css/CSS2/`
  - `css/css-flexbox/`、`css/css-grid/`
  - `css/css-position/`、`css/css-float/`、`css/css-tables/`、`css/css-multicol/`
  - `css/css-text/`、`css/css-writing-modes/`、`css/css-fonts/`、`css/css-text-decor/`
- **导入完整性要求** — 每个目录必须导入上游该目录**全部**范围内 reftest（从 `MANIFEST.json` 自动提取 reftest 列表，排除 skip list 中的范围外 case）。不允许人为缩减导入范围
- **容差收紧** — 布局类 reftest 容差 ≤ 0.1%；文字类 reftest 容差 ≤ 0.5%；优先使用 WPT fuzzy 注解
- **Chromium 参考截图自动化** — Puppeteer/Playwright 自动截图工具链，为每个上游 reftest 生成 Chromium 参考截图
- **CI 集成** — GitHub Actions 中运行上游真实 WPT reftest
- **通过率基线** — 记录上游真实 reftest 初始通过率，不要求达标但必须可测量
- **失败分析机制** — 每个失败 case 自动分类（CSS parser 错误？样式计算错误？布局错误？渲染器错误？）
- **685 个 inline reftest 处理** — 保留为 smoke test，不计入本目标的通过率统计

**依赖**：M7 完成（渲染器必须支持所有图元类型后，真实 WPT reftest 的结果才有意义）。**建议 M10 基础设施搭建（fetch + 导入 + Chromium 截图工具链）与 M7 并行进行**，这样 M7 完成后可以立即运行真实 WPT reftest 来验证

### M11 — 全量冲刺 + 上游真实 WPT Reftest 通过率达标

**目标**：修复所有剩余渲染缺口，达到上游真实 WPT reftest 各领域通过率 ≥ 95%。所有通过率统计的分母必须是上游 WPT 目录中全部范围内 reftest。

**范围**：
- 修复 M10 发现的所有上游真实 WPT reftest 失败 case
- 确保每个上游 WPT 目录导入了全部范围内 reftest（不允许人为缩减）
- CPU + GPU 双模式验证
- 回归测试确保已通过的 case 不退化
- 性能优化（如渲染器批处理、GPU draw call 合并）
- **不允许**通过缩小导入范围、放宽容差、跳过困难 case 来人为提高通过率

**依赖**：M8-M10 完成

---

## Testing & Quality Gates

### 测试层次

| 层次 | 内容 | 运行频率 |
|------|------|----------|
| 单元测试 | 每个 crate 的 `#[test]` 测试 | 每次修改后 |
| 集成测试 | 跨 crate pipeline 测试 | 每次修改后 |
| WPT reftest（CPU 模式） | ZeroWeb CPU 渲染 vs Chromium 截图 | 每个 milestone 验证 |
| WPT reftest（GPU 模式） | ZeroWeb GPU 渲染 vs Chromium 截图 | 每个 milestone 验证 |
| 产品静态页面视觉 smoke | ZeroBrowser/WebView 渲染内置静态页面、录制真实静态文章页和图片密集静态站点 vs Chromium 截图；重点检查文本重叠、sibling 文本串联、外链 CSS 应用、图片子资源/ImageCache、grid/flex 区块分离、正文段落/table/code 结构 | 每个 milestone 验证 |
| 全量回归 | `cargo test` + reftest 全量 | 每轮执行结束 |

### 质量门禁

| 门禁 | 标准 | 不通过时的处理 |
|------|------|----------------|
| 编译 | `cargo build` 零错误 | 立即修复 |
| Clippy | `cargo clippy -- -D warnings` 零警告 | 立即修复 |
| 现有测试 | `cargo test` 零失败 | 立即修复，不允许带着红灯继续 |
| 格式化 | `cargo fmt` 无变更 | 提交前格式化 |
| 新增代码测试覆盖 | 每个渲染修复必须有对应单元测试 | 不允许只改代码不加测试 |
| Reftest 通过率 | 按 Done Criteria 中各领域 ≥ 95%（基于上游真实 WPT reftest） | 继续修复直到达标 |

### Coverage 要求

- 现有测试必须持续全绿
- **真实网站测试保留 `#[ignore]`**：`tests/integration/src/real_website_compat.rs` 中的真实网站兼容性测试因本地网络不稳定，保留 `#[ignore]` 标记，不要求执行。这些测试不计入本目标的通过率统计
- 其余所有测试零 `#[ignore]`：除真实网站测试外，不允许引入新的 `#[ignore]` / skip 标记。如果某个测试需要外部资源（网络、文件），应在测试中做超时和错误处理，而不是跳过
- 新增功能、行为变化、兼容性扩展和回归修复必须同步补单元测试
- Coverage 作为长期主线任务的一部分持续扩大
- **通过率验证必须基于上游真实 WPT reftest** — 不允许用 inline 手写 reftest 替代上游真实 reftest 来充数
- 不允许通过缩小统计范围、缩小导入范围、放宽容差、跳过困难 case 来伪造达标
- 如果缺少 coverage 测量手段，视为要继续推进的工作内容，不视为终止条件

### 证据持久化

每轮执行结束后，以下证据必须持久化到 `docs/goal/rendering-compat/evidence/`：

```
evidence/
├── reftest-report-<timestamp>.json     # 通过率报告（按目录分类）
├── reftest-report-<timestamp>.txt      # 人类可读报告
├── failures/                           # 失败 case 的截图对比
│   ├── <test-name>-expected.png        # Chromium 参考截图
│   ├── <test-name>-actual-cpu.png      # ZeroWeb CPU 渲染截图
│   └── <test-name>-actual-gpu.png      # ZeroWeb GPU 渲染截图
└── coverage-<timestamp>.txt            # 覆盖率摘要
```

---

## Latest Evidence

**重要状态说明**：

1. M1-M6 里程碑声称全部完成，685 个 inline reftest 100% 通过。但实际渲染效果与主流浏览器差距巨大。
2. **本目标的通过率标准已变更为必须基于上游真实 WPT reftest**。685 个 inline 手写 reftest 不计入通过率统计。M1-M6 中基于 inline reftest 的「100% 通过率」不满足本目标的 Done Criteria。
3. 尚未从上游 WPT 仓库导入任何真实 reftest，因此当前上游真实 WPT reftest 通过率为**未知**。

审计发现根本原因：

### 审计发现（2026-06-07）

> ⚠️ **历史快照**（2026-06-07 时点审计）——下列发现曾驱动 M7-M11 里程碑立项。截至当前多数已解决：**渲染器图元覆盖 / 浏览器图元消费 P0 已由 M7 解决**（见 Support Envelope / Current Proven Baseline / 已知关键缺口表 ✅）；**Margin 折叠 / BFC 已由 R323 解决**；**验证体系已由 R484 全量导入上游真实 WPT reftest + DC-14 chromium Oracle 改善**。当前真实状态以 master.md + 上方各表为准，本表保留作历史记录。

| 问题 | 严重性 | 详情 |
|------|--------|------|
| 渲染器图元覆盖不足 | **P0-致命** | Paint 生成 13 种图元，CPU 渲染器仅渲染 3 种（fills、rounded_rects、glyphs），GPU 渲染器仅渲染 2 种（fills、glyphs） |
| 浏览器图元消费不完整 | **P0-致命** | `append_webview_primitives()` 仅传递 fills 和 glyphs，其余 11 种图元静默丢弃 |
| Margin 折叠未实现 | P1-严重 | 块级元素间距与主流浏览器不一致 |
| BFC 未实现 | P1-严重 | 浮动隔离和 margin 折叠无法正确工作 |
| 验证体系无效 | P1-严重 | 685 个 inline reftest 均为手写简单场景，容差过宽松（1%-5%），未使用上游 WPT 真实 reftest，无法发现真实渲染差距 |

### 已添加的修复里程碑

- **M7** — 渲染器图元覆盖 + 浏览器图元消费（Critical Path）
- **M8** — 布局正确性（Margin 折叠 + BFC + Float + Replaced Elements）
- **M9** — 滚动容器 + 高级视觉效果
- **M10** — 上游 WPT 真实 Reftest 导入与验证
- **M11** — 全量冲刺 + 上游真实 WPT Reftest 通过率达标

执行代理应从 M7 开始执行。

---

## Document Control / Archive Policy

### 文档控制平面

本目标采用**两层文档控制平面**：

#### 入口文档（稳定、不频繁修改）

- **路径**：`docs/goal/rendering-compat.md`（本文件）
- **职责**：定义本目标的 Mission、Done Criteria、执行协议和文档治理规则
- **修改条件**：仅在目标本身发生实质性变化时修改（如调整 WPT 覆盖范围、修改通过率目标、调整技术路线）
- **禁止行为**：每轮执行不重写本文件；日常进度、证据、活跃里程碑更新写入 master.md

#### 运行时控制平面（持续演进）

- **路径**：`docs/goal/rendering-compat/master.md`
- **职责**：当前真实状态的唯一控制面板，包含：
  - 当前活跃里程碑及其完成状态
  - 当前各 WPT 目录的 reftest 通过率数据
  - 已导入的 reftest case 数量和分类
  - 已发现和已修复的渲染缺口清单
  - 当前能力矩阵和已验证项
  - 下一步计划
  - 未解决问题列表
- **治理规则**：
  - master.md 是持续演进的增量控制面板，不是一次性交付物
  - 不允许无限增长 — 过时内容必须重写、压缩或迁移到 archive
  - 各章节之间必须自洽（活跃里程碑、Done Criteria、通过率数据、Latest Evidence 不能互相矛盾）
  - 如果出现矛盾（如"通过率未达标但证据声称全部满足"），执行代理必须先纠正文档和状态评估再继续

#### 归档区域（历史记录）

- **路径**：`docs/goal/rendering-compat/archive/`
- **职责**：存储已完成里程碑的详细过程、关键决策、验证结果、commit hash 和历史证据
- **性质**：archive 是历史记录区，不是当前状态的来源

#### 证据区域（验证数据）

- **路径**：`docs/goal/rendering-compat/evidence/`
- **职责**：存储 reftest 通过率报告、失败截图对比、覆盖率数据等验证证据
- **性质**：持续追加，不修改已有证据文件

### 首轮进入检查清单（Must-Complete-First-Round）

执行代理在首次进入时**必须**完成以下操作，这些不是可选的，也不是可以推迟的工作：

- [ ] 探索当前仓库渲染管线事实（CSS parser 能力、style system 能力、layout engine 能力、render foundation 能力）
- [ ] **审计渲染器图元覆盖**：确认 CPU 渲染器和 GPU 渲染器实际支持哪些图元类型
- [ ] **审计浏览器图元消费**：确认 `append_webview_primitives()` 实际传递哪些图元类型
- [ ] 检查现有 WPT runner 和 reftest harness 的具体实现状态
- [ ] 确认现有测试基线（运行 `cargo test` 确保全绿）
- [ ] **确认 `#[ignore]` 标记状态**：`tests/integration/src/real_website_compat.rs` 中的真实网站测试因本地网络不稳定保留 `#[ignore]`（这是已知的、合理的例外）。确认仓库其余部分零 `#[ignore]`
- [ ] 创建或更新 `docs/goal/rendering-compat/master.md`，包含完整的当前状态评估和 M7 计划
- [ ] 创建 `docs/goal/rendering-compat/archive/` 目录
- [ ] 创建 `docs/goel/rendering-compat/evidence/` 目录
- [ ] 选定并启动第一个活跃里程碑（M7 — 渲染器图元覆盖 + 浏览器图元消费）

**关键要求**：完成 master.md 和目录初始化后，执行代理**必须**在同一轮内继续启动 M7，直接推进渲染器图元覆盖能力。**不允许**把"文档框架已建立"当作里程碑完成或收工理由。

### 文档治理原则

1. master.md 各章节必须自洽 — 活跃里程碑、Done Criteria、通过率数据、Latest Evidence 不能互相矛盾
2. 如果发现矛盾，执行代理必须先纠正文档再继续
3. master.md 不允许无限增长 — 过时内容必须压缩或归档
4. archive 是只追加的 — 不修改已归档内容
5. 所有验证证据必须以结构化形式持久化（reftest 报告、截图、覆盖率数据）

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足，目标能力达到 production-ready 水平 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进，还有未完成的工作 | `CONTINUE: <下一步>` | **这是默认输出** |
| 遇到真正的外部阻塞（依赖不可用、平台不支持） | `BLOCK: <原因>` | 罕见使用 |
| verify 发现未满足条件但进展仍可推进 | `CONTINUE: <下一步>` | 返回执行，不是 DONE |

### DONE 允许条件

**同时满足以下所有条件时才允许输出 DONE**：

1. ✅ Done Criteria DC-1 到 DC-14 全部满足（**DC-14 真通过标准是 DC-2~13 通过率数字的可信度前提**）
2. ✅ CPU 渲染器 + GPU 渲染器均支持全部 13 种 `RenderPrimitives` 图元类型
3. ✅ 浏览器 `append_webview_primitives()` 正确消费并渲染所有图元类型
4. ✅ 所有四个 WPT 领域（CSS 2.1、Flexbox+Grid、布局模式、文字排版）通过率均 ≥ 95%（基于真实上游 WPT reftest，且为**严格容差真通过率**、reference 为 **Chromium 独立 Oracle**、分母为上游全量——即满足 DC-14）
5. ✅ Margin 折叠、BFC、Float 布局、滚动容器等核心布局行为与 Chromium 一致
6. ✅ CPU 软件渲染 + GPU 渲染双模式均达标
7. ✅ `cargo build` + `cargo test` + `cargo clippy` 全通过
8. ✅ 有结构化的 reftest 通过率报告作为自动化证据（包含真实 WPT reftest 结果）
9. ✅ master.md 内部自洽，archive 已建立，进度已归档
10. ✅ 产品静态页面视觉 smoke 通过，至少包含 `apps/browser/assets/welcome.html`、morning.work 录制静态文章页和 WinterTC 录制图片密集首页的 ZeroBrowser/WebView/Chromium 对比证据，且无文本重叠、sibling 文本串联、外链 CSS 缺失、图片缺失、正文段落压缩、table 退化或宽屏标题误拆行
11. ✅ 渲染能力本身达到可验证的 production-ready 质量 — 满足以下所有客观标准：
    - 加载至少 5 个主流网站（如 github.com、wikipedia.org、twitter.com 等），为每个网站截图
    - 截图必须通过上游 WPT reftest 的同等级像素对比（max_diff_ratio ≤ 1%，因为真实网站涉及文字渲染）
    - 截图证据持久化到 `docs/goal/rendering-compat/evidence/` 目录，包含 ZeroWeb 截图和 Chromium 参考截图
    - 不允许仅凭「看起来对了」声称通过，必须有自动化像素对比数据作为证据

### 禁止输出 DONE 的情况

即使以下情况中部分条件看起来"还行"，也**不允许**输出 DONE：

- ❌ CPU 或 GPU 渲染器不支持全部 13 种图元类型（渐变、阴影、图片、线段等缺失）
- ❌ GPU 渲染器是 CPU 渲染器的 passthrough 封装（必须使用独立的 GPU 渲染管线：wgpu + WGSL shaders）
- ❌ `append_webview_primitives()` 丢弃任何图元类型
- ❌ ZeroBrowser 对 WebView glyph 做会改变布局语义的后处理重排
- ❌ `apps/browser/assets/welcome.html` 等内置静态页面、morning.work 录制静态文章页或 WinterTC 录制图片密集首页在 Chromium 对比下出现文本重叠、sibling 文本串联、外链 CSS 未加载、图片子资源缺失、宽屏标题误拆行、正文压缩、table 退化或 card/link 区块错位
- ❌ Margin 折叠未实现或未验证
- ❌ BFC 未实现或未验证
- ❌ 只通过了手写 inline reftest，未使用上游 WPT 真实 reftest
- ❌ reftest reference 由 ZeroWeb 自渲染（同源），未接入 Chromium 独立 Oracle（DC-14）——通过率仅证明自一致性，不证明与标准一致
- ❌ 通过率含同源假通过（test==ref 退化或近纯色）而未做非平凡性检查（DC-14）
- ❌ 分母为子集（每目录约 60 个），非上游全量，未做覆盖率对账（DC-14）
- ❌ DC-2~5 以内联 reftest 100% 冒充达标（内联仅 smoke，不计达标，DC-14）
- ❌ GPU 渲染器图元为 CPU 后处理 passthrough，无独立 WGSL pipeline（违反 DC-9/DC-14）
- ❌ reftest 容差过宽松（布局类 > 0.5%，文字类 > 2%）
- ❌ master.md 缺失、必填章节缺失、archive/evidence 为空且无有效里程碑
- ❌ 无 reftest 证据，或 reftest 存在未分析的失败项
- ❌ 无实际代码/测试进度（仅有文档和计划）
- ❌ 通过率无法证明（无 reftest 报告、无截图证据）
- ❌ master.md 各章节矛盾（如"通过率未达标但证据声称全部满足"）
- ❌ 所有 master.md 章节都填了、archive 建了、计划列了，但没有真实 reftest 运行结果和渲染修复
- ❌ 测试全绿、reftest 通过率达标、文档完整，但目标渲染能力本身未达到可验证的 production-ready 质量
- ❌ 只验证了 CPU 渲染或 GPU 渲染其中一种模式
- ❌ 无法加载和正确渲染至少 5 个主流网站的页面

### BLOCK 策略

- "未完成、证据不足、暂时无法验证通过率、文档状态不一致" 都是**继续推进的信号**，不是 BLOCK 的理由
- 即使遇到困难，如果仍有可能推进，输出 `CONTINUE: <下一步>`
- 只有在真正无法继续（外部依赖不可用且无替代方案、平台根本性不支持）时才输出 `BLOCK`
- 缺少 coverage 测量手段、缺少统一统计脚本、缺少报告链路 — 这些是要继续推进的工作内容，不是 BLOCK 的理由

---

## Execution Protocol

### 自主执行原则

执行代理必须：

1. **自主探索**当前渲染管线状态，识别能力缺口
2. **自主导入** WPT reftest，扩大覆盖范围
3. **自主运行** reftest，分析失败原因
4. **自主修复**渲染错误，不等待用户逐步指令
5. **自主添加**测试，新修复必须有对应单元测试
6. **自主验证**，运行 reftest + `cargo test` 确认修复有效
7. **自主归档**，完成的里程碑记录到 archive
8. **持续推动**，直到 Done Criteria 全部满足

### 交替推进策略

每轮执行的工作模式：

1. **扩展基础设施**：从上游 WPT 仓库导入更多真实 reftest case，扩大覆盖范围
2. **运行上游真实 reftest + chromium Oracle 交叉验证**：同源通过率仅作自一致性参考；**优化目标 = chromium Oracle 一致率**（`scripts/chromium-oracle-shot.mjs` + `scripts/cross-validate.py`），污染分析用 `scripts/analyze-pollution.py`
3. **修复渲染缺口**：优先修 `evidence/analyze-pollution-2026-06-16.txt` 的真 bug 候选（chromium 大幅不一致但同源「通过」的用例，即被同源假通过掩盖的真实缺口），每项修复**用 chromium Oracle 验证**而非仅看同源通过
4. **补充测试**：为每个修复添加单元测试
5. **验证回归**：确保修复不破坏已有通过的 case
6. **更新文档**：更新 master.md 状态和 evidence

### 现有基础设施复用原则

M1 及后续 milestone **必须优先扩展现有模块**，禁止重写已有功能：

- 像素对比引擎：扩展 `tests/wpt-runner/src/reftest.rs` 的 `ReftestConfig` 和 `compare_pixels()`，添加分类容差和 WPT fuzzy 注解支持
- WPT MANIFEST 解析：扩展 `tests/wpt-runner/src/manifest.rs`，添加 fuzzy 元数据解析
- CPU 截图：复用 `render_scene_to_framebuffer()`
- Smoke test 套件：保留现有 1,341 个手写 TestCase 和 685 个 inline reftest 继续运行，不删除、不替换。但这些**不计入**本目标的通过率统计，本目标的通过率必须基于上游真实 WPT reftest
- JS runtime：复用 `crates/script-sandbox/` 的 V8 runtime

### `#[ignore]` 管理要求

- `tests/integration/src/real_website_compat.rs` 中的真实网站测试保留 `#[ignore]` 标记，因本地网络不稳定
- 首轮必须搜索全仓库确认：仅 `real_website_compat.rs` 中有 `#[ignore]` 标记（因本地网络不稳定），其余文件零 `#[ignore]`
- 运行 `cargo test` 确认除真实网站测试外全部通过
- 后续不允许新增任何 `#[ignore]` / skip 标记（除真实网站测试外）

### 代码提交规则

- 有阶段性进展时及时提交代码并推送到远端
- 及时拉取远端更新并 rebase
- 提交信息使用英文，文档和注释使用中文

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。遇到 flaky test、遗留失败、环境脚本问题时，当作当前任务的一部分修复
2. **Reftest 失败分析**：每个失败 case 必须分析根因（CSS parser 错误？样式计算错误？布局算法错误？绘制错误？）
3. **技术决策**：在 master.md 中记录关键决策及其理由（如是否引入新依赖、选择哪种实现方案）
4. **依赖问题**：优先自行解决；只有真正无法解决时才 BLOCK
5. **范围变更**：如果发现目标需要调整，在 master.md 中记录并说明理由，但不修改本文件（除非 Mission 本身变化）
6. **渲染管线修改**：修改 `computed_style_to_taffy()` 适配层时，必须确保不破坏已有布局正确性

### 当 verify 发现缺口时

- 默认输出 `CONTINUE: <下一步>` 并返回执行
- 不输出 DONE 或大段解释
- 如果仍有可能推进，就不结束

### 单文件行数限制

- 单个 `.rs` 文件不超过 2000 行
- 如果超过，按职责拆分为多个模块
