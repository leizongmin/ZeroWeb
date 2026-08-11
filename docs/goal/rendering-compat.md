# 页面渲染兼容性 — WPT Reftest 驱动的渲染正确性目标

**版本**: v1.0
**日期**: 2026-06-06
**状态**: Active / Continuous（2026-07-29 用户推翻 agent 自我暂停，恢复持续推进）
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（ZeroWeb 总体目标）

> **说明**
> 本文档是 ZeroWeb 页面渲染兼容性的专项目标执行契约。目标是以 WPT reftest 通过率为验证标准，将 ZeroWeb 的 CSS 渲染输出对齐到 Chromium（Chrome/Edge）水平。本文定义了使命、边界、完成标准、执行协议和文档治理规则，供后续 `rally run` 会话作为稳定输入使用。

> **🔗 当前阻塞 + 可执行方案（2026-07-25）**：reftest ~57% plateau（自主 clean-lever 穷尽）。四大阻塞（Phase A / font-stack / P1b / P3）方案见 [`rendering-compat/blockers-resolution-plan-2026-07-25.md`](rendering-compat/blockers-resolution-plan-2026-07-25.md)。**实施入口**：Phase A 首切片（pre-authorized）= [`rendering-compat/phase-a-slice1-inline-block-linebox-mechanism-2026-07-25.md`](rendering-compat/phase-a-slice1-inline-block-linebox-mechanism-2026-07-25.md)；P1b 独立 RFC = [`rendering-compat/p1b-rfc-2026-07-25.md`](rendering-compat/p1b-rfc-2026-07-25.md)。运行时控制面板 = [`rendering-compat/master.md`](rendering-compat/master.md)。
>
> **🧭 方向裁决（2026-07-28）**：当前执行策略改为"转投高收益项目 + plateau-guard"。后续 agent 不应再把 WPT 95% 当作短期冲刺目标，也不应在已反复证伪的单点切片上循环。允许继续推进的工作仅限：（1）有明确 driving test、低风险、A/B 零回归的 CSS2/parser/selector clean lever；（2）产品/legacy smoke 的可见稳定性修复；（3）为 Phase A 完整 inline-box-model / IFC coherence 输出可回退实施设计。暂跳过：旧 Phase A 首切片、R109 单点、37-form-controls 单点、font-stack rebuild/M18、P1b JS Bridge 深改、P3 真窗口/GPU 验收、inline SVG/SVG intrinsic sizing、sticky/scroll-snap/动态滚动。旧 2026-07-25 blocker 文档保留作历史依据，不再作为默认开工入口；最新执行方向以 `rendering-compat/master.md` 顶部裁决包为准。
>
> **▶ 轻量修复优先裁决（2026-07-29 用户两次指令）**：用户指令——「**永远不要停止，把待决策的记录到文档，在我没决策之前，继续推进其他剩余任务**」+「**我们主要做轻量修复，调整文档方向确保不会跑偏**」。据此：(1) **主线 = 轻量修复**：沿用 `2026-07-28 方向裁决` 允许范围——CSS2/parser/selector clean lever（driving test + 低风险 + A/B 零回归）、产品/legacy smoke 可见稳定性修复、文档与代码不一致的纠偏（本 goal 滞后严重）；每修 net≥0 即 land。(2) **永不停**：轻量修复持续做，遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复，不因单项阻塞停 goal。(3) **深结构护栏·防跑偏**（纠正早些「一律放行深结构」措辞）：font-metric 生产激活+A/B（R2202 dormant 已落地、**勿继续推激活**）、vertical-mode native R1043、taffy replaced-element border-box R2174、Phase A slice-3 IFC 深构造、font-stack C-dep rebuild 等**深结构多会话方向不自主开工**，记待决策清单等用户点名；clean lever 九重穷尽（R2183-R2190）后若暂无新轻量候选，做文档纠偏 + plateau-guard，**勿借机跳深结构**。(4) 本文「执行协议」生效——CONTINUE 是默认输出、未完成/证据不足/状态不一致是继续信号而非 BLOCK、自主修复不等用户逐步指令。
>
> **▶ 主线切换裁决（2026-08-04 用户决策）**：用户裁决——「工作切回父目标 zero-web（恢复 P1 DOM/JS Bridge 原生化），渲染兼容性先缓一缓」。据此：(1) **本 goal 降频守成**，不再作为自主主线推进：保留低频 plateau-guard（`make test` triple-guard 周期拉长——父目标侧有 .rs 变更或每 ~10 轮跑一次，守护 13192 全绿基线）。(2) **待用户决策清单原样保留**：vertical-mode R1043 / taffy R2174 / Phase A IFC / font-stack C-dep / srcset 等深结构继续等点名；用户点名任一深结构 → 立即切回本 goal。(3) 文档纠偏停止主动排程（design-doc 引用核验已近收官），仅在有 .rs 变更引发 drift 时顺手修。(4) 本裁决不推翻 2026-07-29「永不停」指令——守成 + 待命即执行形态，恢复时零成本；父目标主线（P1a 事件循环/fetch/Observer 真实化）与渲染侧无冲突。
>
> **▶ 字体栈实施裁决（2026-08-09 用户决策）**：用户批准 font-stack coherence rebuild（`fontdue-replacement-scoping.md` v0.2.3 / `unified-font-stack-design.md`）——**接受 HarfBuzz C 依赖**（FreeType 已 default-on），本 goal 由降频守成**恢复主动实施**。执行形态：拆分为独立可验证切片（度量统一 → 光栅统一 → 塑形 HarfBuzz → 字体回退逻辑），每片 kill-switch + 结构签名 gate + 全量 oracle A/B，net≥0 才落地；第一刀选最小可度量收益切片。与父目标 P1a（engine DOM 桥）工作面不重叠，可并行。深结构清单其余项（vertical-mode R1043 / taffy R2174 / Phase A IFC / srcset）仍等用户点名。
>
> **▶ 字体栈当前进展（R3241-F·2026-08-11）**：R3240 已提供 IFC ordered-face seam；R3241-F 把它接入真实 layout：新增 layout/painter 共用的 weight/style/family ordered resolver，stored、measure、float、multicol 五个字体注入点统一生成 `NodeId→Vec<font_id>`，匿名 flex/grid 文本也调用 list-aware source。首次无保护 A/B 暴露 primary contract 分裂，css-fonts sum `862.56→890.90`、`font-family-name-025 +19.65pp`；修为仅当 list 首项等于既有 `TextRun.font_id` 时消费多 face，否则回退 singleton，避免 fallback 接线隐式改写 primary。最终 A/B 回到 R3239 的受控结果：sum `862.56→862.67`（`+0.11pp`），pass/credible/strict `84/74/54` 不变；012 `+0.26pp`、013 `-0.17pp`。默认门禁 reftest `687/687`，product-smoke 全 PASS，welcome `15.90%`。因此 `ZW_SHAPED_FALLBACK` 继续 default-off；下一步建立 resolved-face `font-size-adjust` metric contract，并单独统一 primary face matching。详细执行态见 [`rendering-compat/master.md`](rendering-compat/master.md)。
>
> **📋 待用户决策清单（遇需拍板项在此追加，跳过并继续其他轻量修复）**：
> - 格式：`- [ ] <事项> — 为何需用户（深结构 / 许可证 / 破坏性操作 / 改 Mission / 超大下载）— 建议 — 追加时间`
> - **深结构方向（用户 2026-07-29「主做轻量修复」指令划入护栏，等点名，不自主开工）**：
>   - [x] ~~font-metric 生产激活+A/B — R2202 dormant 基础设施（webview+renderer，env `ZW_PERFONT_LINEHEIGHT=1`）已落地未激活；深 plumbing + 需 product-smoke A/B 量化 CJK 收益 — 待用户授权激活并跑 A/B~~ ✅ **已完结（2026-08-01）**：用户授权后 A/B 完成 = **net 负，保持 dormant**（welcome 英文 −0.44pp；morning 中文零变化——全显式 line-height 无 normal 行，「CJK lever」假设证伪）；证据 `evidence/font-metric-activation-ab-2026-08-01.md`（R2393）
>   - [ ] vertical-mode native R1043 — 四层协调深改，R1043 谱系停止条件曾触发 — 等点名
>   - [ ] taffy replaced-element border-box sizing R2174 — 深 multi-session — 等点名
>   - [ ] Phase A slice-3 IFC 深构造（IFC 单一权威化）— 深 architectural，设计已就绪 — 等点名
>   - [x] ~~font-stack coherence rebuild + Phase A IFC line-box-metric 统一（R2025 user-blocked；RFC-ready [`unified-font-stack-design.md`](rendering-compat/unified-font-stack-design.md) v0.2.3）~~ ✅ **已批准（2026-08-09 用户决策）**：接受 HarfBuzz C 依赖，恢复主动实施，分片执行中 — **⚠️ R2869 勘误 R2867（历史依据，不改变批准）**：Skia/raster C-dep **非** font-wall unlock（R1560 real-skia-safe A/B net-24 已证伪；光栅层 R1068/R1159 FreeType default-on 已对齐 chromium）；font-wall 残余在 **layout/metric coherence（Phase A IFC）**，须 **full font-stack rebuild（layout/paint/wrap metric coherence）整体做**（isolated slice 全 net-negative：line-height ×3 + advance ×4 + raster ×1，不可切片），二者皆 deep multi-week user-gated；DC-2~5/2026 65% oracle absent 此授权 = unreachable
>   - [ ] 响应式图片 srcset / `<picture>` / CSS `image-set()`（R2412 发现）— `extract_img_resources` 仅取 `<img src>`，不解析 srcset/source；srcset-only 图缺抓、其余仅次优分辨率。正确选源须 DPR+`sizes`+布局（layout-dependent）+ painter effective-src plumbing — 深，须 RFC+布局集成 — 等点名
> - 真正需用户拍板的 4 类（不兼容/闭源许可证、破坏性 git/文件操作、改 Mission/Done/范围、超大磁盘网络下载工具审批无法覆盖）同上格式追加。当前该 4 类无悬而未决项。
>   - [x] ~~**Mission 95% 的时间账本校准（A1）** — 改 Mission/Done/范围 — Ladybird 7 年/8 人全职/428 贡献者才到同源 93.33%（2026-08-05 实测，官方算法复算），ZeroWeb 当前 oracle ~57% + G0 单维护者；95% 作为短期冲刺目标与幂律现实不匹配是 plateau 反复的根源之一~~ ✅ **已拍板（2026-08-07）**：采纳分阶段里程碑（2026 65% → 2027 80% → 长期 95%），Mission 已更新
>
> **~~⏸️ 旧暂停裁决（2026-07-29，agent 自设；已被上方用户指令推翻，不再约束执行，仅作历史留档）~~**：当时 agent 判定 clean-lever 穷尽、改为「转其他 goal + 低频 plateau-guard」、要求结构性方向须用户点名授权。**此判定与更早的 `2026-07-16 默认决策边界`（已授权上述结构性方向）冲突，agent 当时选了更保守的一方并自我停手，用户 2026-07-29 明确推翻并要求持续推进。**

---

## Mission

以 **上游 WPT 真实 reftest 通过率 95%+** 为长期愿景（核心 CSS 领域与 Chromium 一致），并采用**分阶段里程碑**校准执行预期（2026-08-07 用户拍板 A1；决策依据 [`ladybird-timeline-calibration-2026-08-07.md`](rendering-compat/ladybird-timeline-calibration-2026-08-07.md)）：

| 阶段 | 目标（oracle 一致率） | 说明 |
|---|---|---|
| 2026 年内 | **65%** | 从当前 ~57% 起步；轻量修复 + 守成形态 |
| 2027 | **80%** | 结构性缺口（IFC 等）解耦后 |
| 长期 | **95%** | Ladybird 同口径参考：8 人全职 + 400 贡献者 7 年才到同源 93.33%——95% 是多年级愿景 |

分阶段目标不降低长期 Mission；每阶段达标即验收，plateau 属幂律曲线预期内（不是失败信号）。

**关键约束**：所有验证必须基于从上游 WPT 仓库（`https://github.com/web-platform-tests/wpt`）导入的**真实 reftest**，不允许使用手写 inline reftest 替代或充数。通过率统计的分母是上游 WPT 目录中**所有**属于范围内、不在 skip list 中的 reftest case，不允许人为缩小导入范围。

**⚠️ 优化目标 = chromium Oracle 一致率，非同源通过率（DC-14，2026-06-16 实测确立）**：reftest runner 当前用 ZeroWeb 自渲染 ref 作参考（`reftest.rs:278-283` `run_reftest_with_base` 把 test 与 ref 都经同一 `RenderPipeline` 渲染），同源通过率含 **46.5% 假通过**（全量实测，见 `evidence/cross-validate-full-2026-06-16.txt`）——真实「与 chromium 一致」通过仅 ~37%。**同源通过率（当前 436/489）不再作为优化目标或达标依据**；优化目标改为「chromium Oracle 一致率」，修复优先取 `evidence/analyze-pollution-2026-06-16.txt` 的 18 个真 bug 候选，每项修复用 `scripts/cross-validate.py` 验证（而非仅看同源通过）。**★ R669 起 chromium Oracle 已集成为一等 harness 指标**：`make reftest-oracle [DIR=...]` 直接报 per-dir chromium-Oracle 真一致率 + top 发散修复候选（DC-14 独立 Oracle 项 ✅，见下），取代 post-hoc cross-validate.py 作主测量路径。

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
- 该方向不是完整 CSS2 达标的替代品；完整 CSS2 `chr<1%` 仍是长期目标，但短期应优先让 legacy static pages "可读、布局不崩、核心语义可见"。

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
| WPT reftest 基础设施 | 导入上游 WPT reftest、解析 test list（含 fuzzy 注解）、截图对比、通过率报告、CI 集成 | 详见 evidence/ |
| Chromium 参考截图 | 自动化 headless Chromium（Puppeteer/Playwright）截图工具链 | 详见 evidence/ |
| Reftest 分类容差 | 布局类严格容差、文字类宽松容差、WPT fuzzy 注解覆盖 | 详见 evidence/ |
| CSS 2.1 渲染 | 盒模型、颜色、背景、边框、margin 折叠、inline formatting、BFC、浮动清除、基础定位 | 详见 current-baseline.md |
| Inline formatting 所有权 | 文本节点、inline 元素、inline-block、`<br>`、混合中英文文本单一权威 | 详见 current-baseline.md |
| Flexbox 渲染 | 所有 flex 属性的正确布局和绘制 | 详见 current-baseline.md |
| Grid 渲染 | 所有 grid 属性的正确布局和绘制 | 详见 current-baseline.md |
| Float 布局 | 完整的 float 布局算法，float exclusion、clear、BFC 触发 | ✅ 核心 float 定位、clear、float containment 与 inline exclusion 已实现（R895 / DC-11） |
| Table 布局 | 完整的 table layout 算法，table-layout: auto/fixed、border-collapse、spanning | ✅ 表格网格构建、auto table layout、colspan、border-spacing、匿名表格盒已实现 |
| Multi-column 布局 | column-count/column-width 的实际列排布、column-rule、column-span | ✅ column-count/column-width、column-gap 和基础列分配已实现 |
| 文字排版 | OpenType shaping、BiDi 算法、CJK 排版优化、text-align justify、word-break/overflow-wrap、writing-mode、vertical text | ✅ 已集成 rustybuzz、unicode-bidi、CJK line-breaking；残余详见 current-baseline.md |
| Position 定位 | absolute/relative/fixed/sticky 的精确坐标计算 | ✅ fixed 已修复（R324）；sticky 静态部分已验证（R1982）；残余详见 current-baseline.md |
| Reftest 验证 | CPU 软件渲染模式 + GPU 渲染模式的截图对比 | 详见 current-baseline.md |
| 产品静态页面视觉 smoke | `apps/browser/assets/welcome.html` 等内置静态页面、录制的真实静态文章页和图片密集静态站点必须通过 ZeroBrowser/WebView 路径与 Chromium 参考截图对比 | ✅ 已建立产品 smoke 证据链；详见 dc-progress.md DC-13 |
| 渲染器图元覆盖 | CPU 渲染器和 GPU 渲染器必须能够渲染所有 `RenderPrimitives` 类型（fills、rounded_rects、gradients、shadows、images、strokes、path_fills、path_strokes、transforms、clips、filters、blend_modes、glyphs） | ✅ **已实现（M7）**：CPU + GPU 均已实现全 13 种图元渲染并附单测；详见 current-baseline.md |
| 浏览器图元消费 | `append_webview_primitives()` 必须将所有 `RenderPrimitives` 类型传递到渲染器，不能静默丢弃 | ✅ **已实现（M7）**：遍历全 13 字段无丢弃；详见 current-baseline.md |
| 渐变渲染 | 线性渐变、径向渐变、锥形渐变、重复渐变的 CPU + GPU 渲染 | ✅ **已实现（M7）**；详见 current-baseline.md |
| 阴影渲染 | `box-shadow` 的高斯模糊阴影渲染（offset + blur + spread + color） | ✅ **已实现（M7）**；详见 current-baseline.md |
| 图片渲染 | 背景图片（`background-image`）、`<img>` 元素、`list-style-image` 的图片解码和渲染 | ✅ **已实现（M7）**；详见 current-baseline.md |
| 线段/路径渲染 | `StrokePrimitive`（线段）、`PathFillPrimitive`（路径填充）、`PathStrokePrimitive`（路径描边）的渲染 | ✅ **已实现（M7）**；详见 current-baseline.md |
| 变换渲染 | CSS 2D transform（translate、rotate、scale、skew、matrix）的正确应用 | ✅ **已实现（M7）**；详见 current-baseline.md |
| 裁剪渲染 | `overflow: hidden/clip` 的矩形裁剪，`border-radius` 的圆角裁剪 | ✅ **已实现（M7）**；详见 current-baseline.md |
| 滤镜渲染 | CSS filter（blur、brightness、contrast、grayscale、hue-rotate、invert、opacity、saturate、sepia、drop-shadow） | ✅ **已实现（M7）**；详见 current-baseline.md |
| 混合模式渲染 | `mix-blend-mode` 的 16 种混合模式（normal、multiply、screen、overlay、darken、lighten 等） | ✅ **已实现（M7）**；详见 current-baseline.md |
| Margin 折叠 | 相邻块级元素 margin-top/margin-bottom 的正确折叠算法 | ✅ **已实现（R323 实测）**；详见 current-baseline.md |
| BFC（Block Formatting Context） | `overflow: hidden/auto/scroll`、`display: flow-root`、浮动等正确创建 BFC，隔离浮动和 margin 折叠 | ✅ margin 隔离已实现（R323 实测）；详见 current-baseline.md |
| 替换元素布局 | `<img>`、`<video>`、`<iframe>`、`<canvas>` 的固有尺寸计算和 `object-fit` | ✅ **已实现**；详见 current-baseline.md |
| 滚动容器 | `overflow: scroll/auto` 的可滚动容器，滚动偏移的正确应用 | ✅ 静态部分已验证（R1982）；残余详见 current-baseline.md |
| text-shadow | 文字阴影（offset + blur + color） | ✅ 已实现 text-shadow paint 图元生成与渲染；详见 dc-progress.md DC-12 |
| 多背景图层 | `background-image` 多层叠加渲染 | ✅ **已实现**；详见 current-baseline.md |
| clip-path | CSS clip-path（circle、ellipse、polygon、inset） | ✅ **已实现（M9）**；详见 current-baseline.md |
| backdrop-filter | 元素背后内容的滤镜效果 | ✅ **已实现（M9，R894 实测验证）**；详见 current-baseline.md |
| CSS mask | CSS 遮罩效果 | ✅ **已实现（M9）**；详见 current-baseline.md |
| 重复渐变 | `repeating-linear-gradient`、`repeating-radial-gradient` | ✅ **已实现**；详见 current-baseline.md |

### 不在范围内（明确排除）

- **非 CSS 渲染领域的兼容性**：JS/DOM API 兼容性、网络协议兼容性、安全策略兼容性不在本目标范围内（由父目标 `zero-web.md` 覆盖）
- **Canvas / WebGL / WebGPU**：不在本目标 reftest 范围内
- **动画/交互的帧级正确性**：CSS animation/transition 的视觉正确性验证不作为 reftest 核心指标（但如果有 reftest 覆盖则需通过）
- **性能优化**：本目标关注渲染正确性，不关注渲染性能（由父目标的性能基准体系覆盖）
- **Chromium 专属行为**：只对齐标准规范行为，不复制 Chromium 的 bug 或非标准行为
- **新 crate 依赖的大规模引入**：最小化新依赖，仅在必要时引入许可证兼容的 crate
- **SVG 文档/内联 SVG 渲染**：不在本目标范围。作为 `<img>` / CSS `url()` 图片资源参与页面渲染的 SVG 栅格化属于"图片子资源与替换元素"范围，至少要覆盖产品静态 smoke 中的 Logo 场景

### 依赖约束

- **原则**：最小化新依赖引入
- **许可证**：如果必须引入新 crate，仅接受 MIT / Apache-2.0 / BSD 许可证
- **评估标准**：新依赖必须论证"不引入则无法达成 reftest 目标"的必要性
- **Taffy 迁移裁决（2026-07-16）**：用户已裁决 `taffy 0.7 → 新版 taffy` 应尽早推进，取消旧记录中的"暂缓/未决"状态。迁移不是一次性大爆炸升级，必须先设计并拆分为可回退切片，重点核查 `computed_style_to_taffy()` 适配层、baseline、intrinsic sizing、flex/grid/table/multicol、margin collapse、abspos/fixed/sticky 等行为差异；每个切片都必须用 Chromium oracle reftest、产品 smoke 和现有单测做 A/B，确认 net≥0 且无关键产品回归后才能落地。
- **默认决策边界（2026-07-16）**：为避免执行中反复请求人工决策，以下事项默认已批准继续推进：兼容许可证下的字体/光栅化/shaping C/C++ 依赖调研与小切片试验；R1035/LayoutNG 的本地源码、sparse checkout 或人工片段路线；vertical writing-mode 的 native/scoped/env-gated 改造；table、multicol、R109、Phase A 等结构性多会话工作。只有以下情况需要重新询问用户：不兼容许可证或闭源商业 SDK、大量磁盘/网络下载且工具审批无法覆盖、改变 Mission/Done Criteria/范围边界、破坏性 git/文件操作。

---

## 当前能力/缺口基线

**当前能力/缺口详细基线**：详见 [current-baseline.md](rendering-compat/current-baseline.md)（完整能力矩阵和已知缺口表）。

**关键状态摘要**（截至 2026-08-07·R2863）：
- ✅ **已完成**：CPU/GPU 渲染器全 13 种图元（M7）、浏览器图元消费（M7）、Margin 折叠（R323）、BFC margin 隔离（R323）、Float 核心布局（R895）、Position fixed（R324）、外部样式表加载（R213）、图片子资源贯通（R318）、产品 smoke 证据链
- ⚠️ **P1-严重缺口**：Inline formatting 所有权分裂、Layout/Paint IFC 双路径、滚动容器（「浏览器层 glyph 重排」R2004 已修复——`transform_webview_primitives` 逐个映射仅 scale+offset+clip 无 sort/reorder + 单测 `transform_webview_primitives_preserves_glyph_order` 守护，详见 current-baseline.md / DC-13；不再列 open）
- 📊 **测试基线**：总测试数 13495 全绿（`make test` R2563 周期复跑 + R2572-R2577 六连 lever 各轮零回归 13190/0/74 精确持平 + R2592 text-decoration shorthand thickness 接线 +1→13191 R2597 持平确认 + R2637 registry box-dimension initial-value 纠偏 + 守卫测试 +1→13192 + R2638 column-gap initial-value 纠偏 + 守卫测试 +1→13193（rendering-compat 侧 held）；R2638 后经父目标 zero-web P1a DOM/JS Bridge + 缺失 Web API 系列（R2704-R2863）+291 推进至 13484（R2862 plateau-guard 复跑确认）；R2873 var()-in-shorthand pending-substitution +8 单测 → 13492；R2878 两值 background-size +1 单测 → 13493；R2879 background 简写 gradient+color 拆分 +2 单测 → 13495（零回归）；74 ignored = 网络型 real_website_compat 用例），覆盖率 95.46% line / 96.94% function / 94.88% region

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

**详细进度**：DC-1~14 完整进度详见 [dc-progress.md](rendering-compat/dc-progress.md)。

### DC-1: WPT Reftest 基础设施就位

- [x] 能够从上游 WPT 仓库 fetch 并解析 reftest test list（**扩展**现有 `manifest.rs`，不重写）
- [x] 解析上游 WPT MANIFEST.json 中每个 reftest 的 `fuzzy()` 元数据，并传递给像素对比引擎
- [x] 能够用 CPU 软件渲染器对 ZeroWeb 渲染输出截图（**复用**现有 `render_scene_to_framebuffer`）
- [x] 能够用 GPU 渲染器对 ZeroWeb 渲染输出截图
- [x] **自动化 headless Chromium 截图**：通过 Puppeteer/Playwright 脚本自动在 headless Chromium 中渲染 reftest HTML 并截图，作为参考基线（零手动操作）
- [x] **Viewport 对齐**：ZeroWeb 截图和 Chromium 截图在相同 viewport 尺寸下捕获（默认 800×600，可配置）
- [x] **JS 执行支持**：Reftest harness 在截图前通过 `script-sandbox` V8 runtime 执行页面 JavaScript
- [x] **分类容差机制**：支持按 reftest 分类设置不同像素容差阈值（布局类 ≤ 0.1%，文字类 ≤ 0.5%）；优先使用 WPT fuzzy 注解；容差锁定不可放宽
- [x] **范围外 reftest 过滤**：导入时自动过滤或标记范围外 reftest（SVG、Canvas、WebGL），维护 skip list 文件
- [x] 通过率报告按 WPT 目录分类输出（文本 + JSON 格式）
- [x] Reftest 运行可通过单一命令执行——`make reftest`（Makefile:74，test-guard 包裹 `cargo run --release --bin zero-wpt-runner -- reftest`）
- [x] CI 管线中集成 reftest 运行（至少 CPU 模式）——`.github/workflows/ci.yml` `reftest` job（workflow_dispatch：fetch-wpt-data + reftest-smoke 快门禁 + 全量 CPU reftest --format json + 报告 artifact 上传）+ `.github/workflows/weekly.yml` `reftest-trend` job（schedule + dispatch，周记录趋势）

**状态**：✅ **全部就位**——fetch/parse test list（manifest.rs）、MANIFEST.json fuzzy 元数据、CPU+GPU 截图、headless Chromium oracle 抓取、viewport 对齐（800×600）、V8 JS 执行、分类容差锁定（DC-14）、范围外 skip list、文本+JSON 报告、单一命令（`make reftest`）、CI 集成（ci.yml `reftest` + weekly.yml `reftest-trend`）全实现；详见 dc-progress.md

### DC-2: CSS 2.1 核心通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css2/` 和 `css/CSS2/` 目录导入**全部**范围内 reftest（排除 skip list 中的范围外 case）
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] 覆盖：盒模型、margin 折叠、BFC、inline formatting、颜色、背景、边框、基础定位
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

**状态**：详见 dc-progress.md

### DC-3: Flexbox + Grid 通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-flexbox/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-grid/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

**状态**：详见 dc-progress.md

### DC-4: Positioning + Float + Table + Multicol 通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-position/`、`css/css-float/`、`css/css-tables/`、`css/css-multicol/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

**状态**：详见 dc-progress.md

### DC-5: 文字排版通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-text/`、`css/css-writing-modes/`、`css/css-fonts/`、`css/css-text-decor/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

**状态**：详见 dc-progress.md

### DC-6: Quirks Mode 完整实现

- [ ] CSS parser 在 quirks mode 下正确调整解析行为
- [ ] Style system 在 quirks mode 下应用特定样式规则
- [ ] Layout engine 在 quirks mode 下实现特定布局行为
- [ ] DOM parser 的 quirks mode 状态正确传递到 CSS parser → style system → layout engine 链路

**状态**：✅ 实质已实现（CSS parser + style system 两层活跃；layout-engine 无独立 quirks 层由 style-system 预烘焙覆盖）；详见 dc-progress.md

### DC-7: 测试与质量不可退让

- [x] 所有现有测试持续全绿（`cargo test` 零失败）—— held baseline **13495/0/74**（74 ignored = real_website_compat 网络型用例，见下条）
- [x] **真实网站测试保留 `#[ignore]`**：`tests/integration/src/real_website_compat.rs` 中的真实网站兼容性测试因本地网络不稳定，保留 `#[ignore]` 标记，不计入本目标通过率统计。其余所有测试零 `#[ignore]`
- [x] 所有新增渲染修复必须有对应单元测试覆盖，**且把对应的上游 WPT reftest 用例导入常驻断言集（测试资产化，2026-08-06 落地）**：`make import-wpt TEST=<wpt 路径> REF=<ref 路径> [NOTE="R21xx 备注"]` —— 文件本体进入 `tests/wpt-runner/wpt-data/`（独立 repo），条目记入 `tests/wpt-runner/imported-tests.txt` 账本（随修复提交），manifest 自动重新生成
- [x] `cargo build` 零错误、`cargo clippy` 零警告（R2865 `cargo clippy --workspace --all-targets -- -D warnings` 全绿复跑确认；R2873 var()-in-shorthand 改动后 workspace clippy 复跑全绿，held baseline 13492/0/74）—— **DC-7 全部子项现已闭环**
- [x] Reftest 通过率报告持久化到 `docs/goal/rendering-compat/evidence/wpt-trends/`（`scripts/record-wpt-trend.sh` → `trend.csv` 绝对数 + JSON 快照；本地 `make reftest-trend`，每周 CI 自动记录，2026-08-06 落地）
- [x] 每轮执行的 reftest 通过率变化可追溯（`evidence/wpt-trends/trend.csv` 历史记录，含日期/模式/绝对数/git_sha，2026-08-06 落地）

**状态**：详见 dc-progress.md

### DC-8: CPU 渲染器图元覆盖 100%

**状态**：✅ **已完成（M7）** —— CPU 渲染器已实现全 13 种图元，详见 dc-progress.md

### DC-9: GPU 渲染器图元覆盖 100%

**状态**：✅ **已完成（M7）** —— GPU 渲染器已实现全 13 种图元（非 CPU passthrough），详见 dc-progress.md

### DC-10: 浏览器图元消费完整性

**状态**：✅ **已完成（M7）** —— `append_webview_primitives()` 遍历全 13 字段无丢弃，详见 dc-progress.md

### DC-11: 布局正确性

**状态**：✅ Margin 折叠、BFC 创建、Float 布局、Position fixed、替换元素、百分比高度、Auto margin 居中、min/max-width/height 已实现；⚠️ Position sticky、Overflow scroll/auto 残余属 host 层 interactive 特性；详见 dc-progress.md

### DC-12: 高级视觉效果

**状态**：✅ text-shadow、多背景图层、重复渐变、border-image、clip-path、backdrop-filter、CSS mask 已实现；[~] 打印媒体查询部分实现（R1981/R1991/R1992）；[ ] scroll-snap 需宿主层滚动输入路由；详见 dc-progress.md

### DC-13: 产品静态页面视觉 smoke

**状态**：✅ welcome.html、Legacy Static Web smoke、URL 导航外链 CSS、图片子资源、via-webview 路径、viewport 覆盖、自动检查、glyph 重排保护、证据持久化已实现（R1600/R1601/R658/R213/R318/R662/R1597/R1598/R2004）；⚠️ morning.work、wintertc.org fixture 录制待完成；详见 dc-progress.md

### DC-14: 真通过标准（anti-false-pass）— 验证可信度门禁

> 本 DC 防止 reftest 通过率被「同源假通过」「宽容差」「子集分母」污染。**DC-2~13 的通过率数字只有在本 DC 同时满足时才可信、才计入达标判定。**

**状态**：✅ 独立 Oracle、非平凡性检查、严格容差三态分类、容差锁定、分母真实性（R484 全量去子集化）、GPU 非 passthrough、内联 smoke 不计达标均已实现；详见 dc-progress.md

**关键事实**：字体光栅化（fontdue ≈ chromium）非渲染差异主因；多行 y 堆叠（R630）和字体归因三证推翻（R631）证实行盒度量为真因；详见 dc-progress.md 和相关 evidence 文件。

---

## 活跃里程碑（M7-M11）

**历史里程碑**：M2-M6 已完成或已过时，详见 [archive/milestones-history.md](rendering-compat/archive/milestones-history.md)。

### M7 — 渲染器图元覆盖 + 浏览器图元消费（✅ 已完成）

**目标**：消除渲染管线最大的视觉输出缺口 — 让 CPU/GPU 渲染器和浏览器 `append_webview_primitives()` 能处理所有 13 种 `RenderPrimitives` 图元类型。

**状态**：✅ **已完成（DC-8 CPU 13/13 + DC-9 GPU 13/13 + DC-10 浏览器消费全 13 字段，均附 framebuffer 像素断言测试）**

### M8 — 布局正确性（Margin 折叠 + BFC + Float + Replaced Elements）（⚠️ 部分完成）

**目标**：实现 CSS 2.1 核心布局算法，使块级布局结果与主流浏览器一致。

**状态**：✅ Margin 折叠、BFC margin 隔离、Float 核心布局、Position fixed、替换元素、百分比高度、Auto margin 居中已实现；⚠️ Position sticky、Overflow scroll/auto 残余属 host 层 interactive 特性

### M9 — 滚动容器 + 高级视觉效果（✅ 基本完成）

**目标**：实现滚动容器功能和高级 CSS 视觉效果。

**状态**：✅ text-shadow、多背景图层、重复渐变、border-image、clip-path、backdrop-filter、CSS mask 已实现；⚠️ scroll-snap 需宿主层滚动输入路由

### M10 — 上游 WPT 真实 Reftest 导入与验证（✅ 已完成）

**目标**：从上游 WPT 仓库导入**全部**范围内真实 reftest，建立可信的渲染正确性验证基线。

**状态**：✅ **已完成（R484 全量去子集化 ~9967 reftest + R669 chromium Oracle harness + DC-14 三态分类）**

### M11 — 全量冲刺 + 上游真实 WPT Reftest 通过率达标（⚠️ 进行中）

**目标**：修复所有剩余渲染缺口，达到上游真实 WPT reftest 各领域通过率 ≥ 95%。

**状态**：⚠️ **自主 clean-lever surface 经 6 vein definitively 穷尽**——R2572 订正旧「8 angle 穷尽」框架（过早），续经 4 法 land 六连 lever R2572-R2577（counters() / ::marker / list-style-type:string / border-image-outset / border-image-width / word-break:break-word；directed probe + Explore-agent fan-out + exhaustive field 审计 + exhaustive variant 审计）；R2578 exhaustive value-variant 审计（全值枚举变体→消费核验）= clean lever 零产出（残余全 false-positive / deep / host-layer / 0-test）；R2581 missing-property 批量核验（60 常见 CSS 属性→51 未应用全 deep/niche/host-layer）+ R2582 伪元素 parse-vs-apply 审计（19 伪元素→16 未 apply 全 Phase A IFC-deep/host-layer/niche/OOS）亦 = clean lever 零产出。**6 vein（directed probe + agent fan-out + exhaustive field + exhaustive variant + missing-property 批量 + 伪元素 parse-vs-apply）rigorous 证 clean-lever surface definitively 穷尽**。活跃自主面仅 ① 低频周期 plateau-guard（R2577 `make test` 13190/0/74 绿）+ ② 文档纠偏；**唯一推向 95% = 用户点名授权深结构专项**（最高 value = Phase A IFC line-box-metric 统一，first-letter/first-line 亦属此 territory；次 = R1043 vertical-mode / R2174 taffy border-box / font-stack C-dep / Phase 2 multicol fragmentation / individual+3D transforms；受字体度量 / 布局结构性 plateau 限制）。详见 `rendering-compat/master.md` R2572-R2582。

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
10. ✅ 产品静态页面视觉 smoke 通过
11. ✅ 渲染能力本身达到可验证的 production-ready 质量

### 禁止输出 DONE的情况

即使以下情况中部分条件看起来"还行"，也**不允许**输出 DONE（包括但不限于）：

- ❌ CPU 或 GPU 渲染器不支持全部 13 种图元类型
- ❌ GPU 渲染器是 CPU 渲染器的 passthrough 封装
- ❌ `append_webview_primitives()` 丢弃任何图元类型
- ❌ ZeroBrowser 对 WebView glyph 做会改变布局语义的后处理重排
- ❌ 只通过了手写 inline reftest，未使用上游 WPT 真实 reftest
- ❌ reftest reference 由 ZeroWeb 自渲染（同源），未接入 Chromium 独立 Oracle（DC-14）
- ❌ 通过率含同源假通过而未做非平凡性检查（DC-14）
- ❌ 分母为子集，非上游全量（DC-14）
- ❌ 容差过宽松
- ❌ 无 reftest 证据，或 reftest 存在未分析的失败项
- ❌ 无实际代码/测试进度（仅有文档和计划）

### BLOCK 策略

- "未完成、证据不足、暂时无法验证通过率、文档状态不一致" 都是**继续推进的信号**，不是 BLOCK 的理由
- 即使遇到困难，如果仍有可能推进，输出 `CONTINUE: <下一步>`
- 只有在真正无法继续（外部依赖不可用且无替代方案、平台根本性不支持）时才输出 BLOCK
- 缺少 coverage 测量手段、缺少统一统计脚本、缺少报告链路 — 这些是要继续推进的工作内容，不是 BLOCK 的理由

---

## Execution Protocol

### 高收益执行模式（2026-07-28 裁决）

当前目标进入 **plateau-guard + 高收益推进** 模式：

1. **守住已获得收益**：每轮先确认 `make test`、产品 smoke、legacy smoke 没有新回归；reftest 作为回归守卫和机会性扫描，不再作为短期 95% 冲刺。
2. **只接 low-risk clean lever**：只有同时满足"明确 driving test、根因清楚、改动面小、A/B 无新失败、产品 smoke 无结构回归"的修复才继续落地。
3. **及时跳过死胡同**：同一方向连续 2-3 轮 empirical 扫描 negative、或需要高风险架构重写但没有新设计时，立即记录结论并转向，不继续消耗会话。
4. **Phase A 只做设计后实施**：完整 inline-box-model / IFC coherence 是潜在高收益架构方向，但必须先写可回退实施设计，包含 kill-switch、结构签名 gate、三态 A/B 门禁和净负回退策略；禁止直接按旧 `phase-a-slice1` 开工。
5. **明确暂跳过项**：font-stack rebuild/M18、P1b JS Bridge 深改、P3 真窗口/GPU 验收、R109 单点、37-form-controls 单点、inline SVG/SVG intrinsic sizing、sticky/scroll-snap/动态滚动不作为当前 rendering-compat 主线。
6. **文档优先级**：入口文档定义长期目标；当前执行方向以 `docs/goal/rendering-compat/master.md` 顶部"当前裁决"块为准；archive/evidence 只作历史证据，不覆盖最新裁决。

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
2. **运行上游真实 reftest + chromium Oracle 交叉验证**：同源通过率仅作自一致性参考；**优化目标 = chromium Oracle 一致率**，修复优先取真 bug 候选（chromium 大幅不一致但同源「通过」的用例），每项修复**用 chromium Oracle 验证**而非仅看同源通过
3. **修复渲染缺口**：优先修复被同源假通过掩盖的真实缺口
4. **补充测试**：为每个修复添加单元测试
5. **验证回归**：确保修复不破坏已有通过的 case
6. **更新文档**：更新 master.md 状态和 evidence

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。遇到 flaky test、遗留失败、环境脚本问题时，当作当前任务的一部分修复
2. **Reftest 失败分析**：每个失败 case 必须分析根因（CSS parser 错误？样式计算错误？布局算法错误？绘制错误？）
3. **技术决策**：在 master.md 中记录关键决策及其理由（如是否引入新依赖、选择哪种实现方案）
4. **依赖问题**：优先自行解决；只有真正无法解决时才 BLOCK
5. **范围变更**：如果发现目标需要调整，在 master.md 中记录并说明理由，但不修改本文件（除非 Mission 本身变化）

### 当 verify 发现缺口时

- 默认输出 `CONTINUE: <下一步>` 并返回执行
- 不输出 DONE 或大段解释
- 如果仍有可能推进，就不结束

---

## Document Control / Archive Policy

> **📄 2026-07-29 结构性精简**：本文件从 982 行精简到 ~460 行。详细内容转子文档：DC 进度→ [`dc-progress.md`](rendering-compat/dc-progress.md)、当前能力/缺口基线→ [`current-baseline.md`](rendering-compat/current-baseline.md)、已完成里程碑→ [`archive/milestones-history.md`](rendering-compat/archive/milestones-history.md)。**精简前完整原文（982 行，零删减保底）→ [`archive/rendering-compat-pre-slimdown-2026-07-29.md`](rendering-compat/archive/rendering-compat-pre-slimdown-2026-07-29.md)**。`master.md` 同期精简（750→~135 行），完整原文→ [`archive/master-pre-slimdown-2026-07-29.md`](rendering-compat/archive/master-pre-slimdown-2026-07-29.md)。

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

### 文档治理原则

1. master.md 各章节必须自洽 — 活跃里程碑、Done Criteria、通过率数据、Latest Evidence 不能互相矛盾
2. 如果发现矛盾，执行代理必须先纠正文档再继续
3. master.md 不允许无限增长 — 过时内容必须压缩或归档
4. archive 是只追加的 — 不修改已归档内容
5. 所有验证证据必须以结构化形式持久化（reftest 报告、截图、覆盖率数据）

---

## 单文件行数限制

- 单个 `.rs` 文件不超过 2000 行
- 如果超过，按职责拆分为多个模块
