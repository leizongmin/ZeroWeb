# WebView 渲染合规测试：以 WPT Reftest 为主、像素基线为辅

## 任务规划摘要

### 5W1H 分析

| 维度 | 当前理解 | 调研处理 |
|------|----------|----------|
| What | 如何自动化验证 WebView 对网页排版、渲染、CSS 样式的结果符合预期 | 聚焦浏览器引擎测试，而不是普通 Web 应用 E2E |
| Why | ZeroWeb 自建 DOM、CSSOM、样式、布局、paint、render，需要可持续验证兼容性 | 找主流浏览器和 WPT 的测试分层 |
| Where | `ZeroWebView`、`RenderPipeline`、`tests/wpt-runner`、CI | 对照本仓库现有实现给出落地路径 |
| When | 当前主流实践 | 以 WPT、Chromium、WebKit、Firefox 当前官方文档为主 |
| Who | 浏览器内核开发者、CI 维护者、WebView API 使用者 | 区分 engine 合规、应用视觉回归、产品验收 |
| How | 定义“预期”的测试 oracle，并降低跨平台渲染噪声 | 采用 WPT testharness/reftest/crashtest、内部 tree dump、少量 pixel baseline |

### 术语映射

| 用户描述 | 行业术语 | 搜索种子词 |
|----------|----------|------------|
| webview 的渲染符合预期 | rendering conformance / visual correctness / test oracle | WPT reftest, browser rendering tests |
| 页面排版 | layout correctness / layout tree | Blink LayoutTests, Firefox reftest |
| CSS 样式符合预期 | CSS conformance / cascade / computed style | CSS WPT, CSS metadata, testharness.js |
| 自动化截图验证 | visual regression / pixel test / screenshot comparison | pixel baseline, Playwright visual comparisons |
| 一般浏览器怎么做 | browser engine test infrastructure | Chromium web tests, WebKit LayoutTests, Firefox reftest |

### 子任务

1. 识别浏览器行业测试分类：WPT testharness、reftest、visual、crashtest、wdspec。
2. 对比 Chromium、WebKit、Firefox、Servo 的实践。
3. 对照 ZeroWeb 现状，判断已有 `wpt-runner` 能证明什么、不能证明什么。
4. 给出适合 ZeroWeb/WebView 的分阶段测试架构。

## 来源分级总表

| 分级 | 本文使用方式 | 主要来源 |
|------|--------------|----------|
| 一手事实 | 官方文档、项目源码、仓库内代码直接观察 | WPT、Chromium、WebKit、Firefox、Servo 文档；ZeroWeb 源码 |
| 外部搜索 | 生态工具与辅助实践 | Playwright 视觉对比文档、wpt.fyi 文档 |
| 💡 推理 | 从官方测试分类和 ZeroWeb 当前结构推导落地路线 | 本文第 5 章 |
| ⚠️ 假设 | ZeroWeb 暂以 CPU framebuffer 和无头渲染为主推进渲染合规测试 | 基于 `render-foundation::cpu` 已存在和 GPU 差异较难稳定的工程判断 |

## 30 秒速览

- 浏览器引擎通常不靠“整页截图像素完全一致”证明渲染正确；截图只是其中一层。
- 主流共识是：API/DOM/CSSOM 用 `testharness.js`，布局和视觉效果用 reftest，无法 reftest 的少数场景才用 visual/pixel baseline。
- Reftest 的核心是“测试页”和“参考页”在同一浏览器里渲染后比较；参考页不使用被测特性，能显著降低跨平台 golden image 的维护成本。
- Chromium 明确偏好顺序是 JavaScript tests -> reference tests -> pixel tests -> text/internal dump tests；pixel tests 因平台、字体、GPU、驱动差异更脆弱。
- Firefox/WebKit/Servo 都把 WPT 或 reftest 作为重要兼容性信号，同时保留各自内部测试、预期失败列表、模糊匹配和重复验证机制。
- 对 ZeroWeb，当前 `tests/wpt-runner` 更像 smoke/invariant runner：能证明“不 panic、有 DOM/layout/primitives”，还不能证明 CSS/layout/render 与标准或参考图一致。

## 执行摘要

**核心结论**：要自动化验证 WebView 的网页排版、CSS 样式和渲染结果，应该定义多层 oracle，而不是追求单一“截图正确”。对 ZeroWeb 这类自建浏览器核心，推荐的主线是：

| 层级 | 验证目标 | 典型方法 | 是否适合每次 PR |
|------|----------|----------|----------------|
| Parser/style/layout 单元测试 | 局部规则正确 | Rust 单元测试、computed style/layout 数值断言 | 是 |
| DOM/CSSOM/API 行为 | 标准 Web API 正确 | WPT `testharness.js` | 是，先跑子集 |
| 布局和视觉效果 | 与参考实现同等视觉结果 | WPT reftest / 自建 reftest | 是，跑精选集 |
| Paint/GPU/字体等非 reftest 场景 | 具体像素不回归 | pixel/golden image baseline + fuzzy threshold | 否，精选且固定环境 |
| 稳定性 | 页面能加载、绘制、不崩溃 | crashtest、fuzz、重复验证 | 是/夜间 |
| 真实站点 | 端到端用户可见退化 | 真实页面 smoke + 截图/人工 triage | 夜间/发布前 |

**对“符合预期”的定义**：在浏览器工程里，“预期”不是产品经理或设计稿的一张图，而是每个测试声明的断言：某个 CSS 规则的 computed value、某个布局盒子的几何、某个页面与参考页是否 pixel match、某个页面是否完成加载且不崩溃。WPT 的 CSS 测试还要求用 `rel=help` 指向规格章节，并推荐用 `meta name=assert` 写清测试意图 [5]。

> **📌 来源说明（任务规划与摘要）**
>
> - **一手事实** [1][2][5][10][11][A][B][C][D][E][F]：WPT 测试类型、Chromium 测试偏好、ZeroWeb 当前代码结构。
> - **外部搜索** [19]：Playwright 视觉对比用于补充普通应用截图回归的边界。
> - **💡 推理**：将官方测试分类映射到 ZeroWeb 的 `RenderPipeline` 和 `wpt-runner`。
> - **⚠️ 假设**：ZeroWeb 的渲染合规测试会优先采用无头 CPU framebuffer，后续再扩展 GPU/platform matrix。

## 1. 浏览器一般怎么测渲染正确性

浏览器的渲染测试通常不是一类测试，而是一组互补测试：

| 测试类型 | 主要回答的问题 | Oracle | 典型用途 |
|----------|----------------|--------|----------|
| `testharness.js` | DOM/CSSOM/API 行为是否符合规范 | JS 断言 | `getComputedStyle`、CSSOM、DOM API、事件、异步 API |
| reftest | 页面最终视觉结果是否等价 | 测试页 vs 参考页截图 | CSS layout、paint、视觉样式 |
| visual test | 无法构造可靠参考页时，是否可人工/平台截图判断 | 平台特定截图或人工判定 | 下划线、字体、UA 差异大的视觉特性 |
| crashtest | 页面是否能加载并完成绘制而不崩溃 | load + paint 完成 | 崩溃、断言、泄漏、sanitizer |
| text/tree dump | 内部结构是否按预期变化 | DOM/layout/render/composited tree 文本基线 | 难以用 Web API 或视觉等价表达的内部行为 |
| pixel/golden image | 渲染输出是否与仓库中的图片基线一致 | 当前截图 vs `expected.png` | 不能 reftest 的 paint/GPU/字体/平台视觉场景 |

WPT 官方文档把 `testharness.js` 列为 API 测试的首选，reftest 用于 rendering/layout，visual test 用于 reftest 不现实的渲染场景，crashtest 用于确认文档加载不崩溃 [1][2][4][5]。Chromium 的 Blink 文档给出的偏好顺序更直接：JavaScript tests 优先，然后 reference tests，再 pixel tests，最后才是内部 text/layout tree dump；原因是 pixel 和 tree dump 更慢、更脆、更依赖平台 [9]。

**关键点**：浏览器工程中的“正确”通常是局部、可声明、可追踪的。一个测试要能说清楚它测试的规范段落、输入、断言、预期失败和平台差异。WPT 的 CSS metadata 要求 CSS 测试至少带一个 `rel=help` 规格链接，并建议用 `meta name=assert` 明确测试试图证明什么 [6]。

> ### 💡 推理分析：为什么不能只做截图测试
>
> **观察**：WPT、Chromium、WebKit、Firefox 都使用 reftest/pixel/text/API 多层测试，而不是单一 screenshot baseline [1][9][11][13]。
>
> **推理**：CSS 和 layout 的正确性既包含可由 JS 读取的规则结果，也包含最终视觉结果；截图能发现视觉差异，但很难定位是 parser、cascade、layout、paint、font 还是 compositor 的错误。
>
> **结论**：ZeroWeb 应该把 screenshot/pixel comparison 放在较高层，用于 paint/render 回归；底层仍需要 computed style、layout tree、render primitive 的精确断言。

> **📌 来源说明（第 1 章）**
>
> - **一手事实** [1][2][4][5][6][9][11][13]：WPT、Chromium、WebKit、Firefox 官方测试分类。
> - **💡 推理**：从多浏览器测试分类归纳“多 oracle”原则。
> - **⚠️ 假设**：ZeroWeb 的目标是浏览器内核兼容性，而不是普通网页应用 UI 回归。

## 2. Reftest 是浏览器渲染测试的主力

Reftest 的基本结构是：测试页使用被测特性，参考页用更简单、已知可靠的写法构造同样视觉结果；自动化运行时分别渲染两页并做截图比较 [2]。WPT 用 `<link rel="match" href="...">` 或 `<link rel="mismatch" href="...">` 声明参考关系；`match` 要求两页在指定视口内像素一致，`mismatch` 要求不一致 [2]。

典型例子：

```html
<!-- test.html: 测试 flex 是否能把元素放到预期位置 -->
<link rel="match" href="test-ref.html">
<style>
.box { display: flex; width: 100px; height: 100px; }
.item { width: 100px; height: 100px; background: green; }
</style>
<div class="box"><div class="item"></div></div>
```

```html
<!-- test-ref.html: 不使用 flex，用简单 block 构造相同画面 -->
<style>
.item { width: 100px; height: 100px; background: green; }
</style>
<div class="item"></div>
```

这样做的优势是：不需要维护跨平台 golden PNG；测试页和参考页在同一个浏览器、同一个环境中渲染，平台字体、抗锯齿、色彩管理的影响被抵消了一部分。Firefox 的 reftest 文档也明确说明，和保存历史截图相比，用 HTML reference 作为显式 pass criteria 可以减少平台和时间变化带来的问题 [13]。

但 reftest 不是万能的。WPT 文档指出，有些效果难以构造参考，例如下划线的位置和粗细受 UA、字体、平台影响 [2]。这类测试才进入 visual/pixel baseline 或人工判定。为处理抗锯齿等小差异，WPT reftest 支持 `meta name=fuzzy`，以单通道最大差异和总差异像素数作为容忍边界 [2]；Firefox 也有 `fuzzy()` 和 `fuzzy-if()`，并建议尽量使用最窄边界，避免掩盖真实回归 [13]。

> **📌 来源说明（第 2 章）**
>
> - **一手事实** [2][3][4][13]：WPT reftest 结构、fuzzy 机制、视觉测试限制、Firefox reftest rationale。
> - **💡 推理**：将 reftest 的优势解释为“减少 golden PNG 和跨平台噪声”。
> - **⚠️ 假设**：ZeroWeb 早期可以只支持 `match`，后续补 `mismatch`、多 reference、`reftest-wait`。

## 3. 主流浏览器的实际测试体系

### Chromium / Blink

Chromium 明确建议 Blink 暴露给 Web 的 surface 尽量覆盖并贡献到 WPT；无法用 WPT 或 C++ 单测覆盖时，再使用 Blink web tests [9]。Blink web tests 的类型包括 JavaScript tests、reference tests、pixel tests、text tests 和 audio tests，其中 JavaScript tests 最可靠，reference tests 用于 JS 不足以测试的 paint/layout，pixel tests 只在不能用 reference tests 时使用 [9]。

Chromium 对 pixel tests 的态度很谨慎：页面渲染会受显卡、驱动、平台文本渲染、OS 设置等影响，因此常常需要每个平台一套参考图，维护成本高 [9]。它也保留 layout tree dump/text dump，但文档明确说这种内部结构基线受平台和实现结构影响，应作为 last resort [9]。

Chromium 的 expected/baseline 文档还强调，web tests 首先是回归测试套件：既关心是否正确，也更关注行为是否按预期变化 [10]。这点对 ZeroWeb 很重要：当实现还不完整时，可以先记录“当前已知失败”，再用期望文件区分真实回归和已有缺口。

### WebKit

WebKit 的 WebCore 主力是 LayoutTests，测试文件本质上是 HTML/CSS/JS 页面，由 WebKitTestRunner 或 DumpRenderTree 执行 [11]。输出可以是文本、render tree dump、PNG，也支持 reference tests。WebKit 也有 platform-specific expected files 和 TestExpectations，用于处理平台差异、已知失败、crash、timeout、image-only failure 等 [11]。

WebKit 明确把 WPT 导入 `LayoutTests/imported/w3c/web-platform-tests`，并在导入后运行 `run-webkit-tests` 生成或更新 expectations [12]。这说明主流引擎不是“直接跑全量 WPT 就完事”，而是 WPT + 本地 expectations + 平台基线 + 内部测试。

### Firefox / Gecko

Firefox reftest 是专门用于 layout engine visual tests 的框架，运行时捕获测试页和参考页图像，并按 `==`、`!=`、`load`、`script`、`print` 等类型判定 [13]。其 manifest 支持 `skip-if`、`fails-if`、`random-if`、`fuzzy`、`pref` 等条件，说明成熟浏览器需要细粒度记录平台、配置和已知不稳定性 [13]。

Firefox 文档还把 reftest 适用范围讲得很清楚：CSS cascade、canvas compositing、CSS counters、margin collapsing、动态变化和增量布局，都能通过“等价页面”表达 [13]。这和 ZeroWeb 当前重点模块（CSS parser/style/layout/paint）高度匹配。

### Servo

Servo 作为 Rust 浏览器引擎，直接把 WPT 作为推进兼容性的核心工具。Servo Book 描述了 `tests/wpt` 中的 WPT 集成、`include.ini` 子集、`meta` expected failures、`mach test-wpt`、`update-wpt` 和 reftest analyzer [15]。Servo 官方博客也提到 WPT 用于捕获回归和不稳定性，并指导 layout engine migration 与 CSS2 conformance [16]。

Servo 的实践对 ZeroWeb 最接近：早期不需要从全量 WPT 开始，而是维护一个当前可跑的 WPT 子集、期望文件和按 CSS/layout 主题划分的通过率看板。

> **📌 来源说明（第 3 章）**
>
> - **一手事实** [9][10][11][12][13][15][16]：Chromium、WebKit、Firefox、Servo 官方文档和项目资料。
> - **💡 推理**：把 Chromium/WebKit/Firefox 的成熟模式迁移到 ZeroWeb 的阶段性实现路线。
> - **⚠️ 假设**：ZeroWeb 当前更接近 Servo 早期阶段，而不是 Blink/WebKit/Gecko 的成熟阶段。

## 4. ZeroWeb 当前测试能证明什么

ZeroWeb 当前已有 `tests/wpt-runner`，并且 `manifest.rs` 能解析 `testharness`、`manual`、`reftest`、`wdspec`、`performance` 等 WPT manifest 类型（`tests/wpt-runner/src/manifest.rs`）。但执行层目前使用内置 `TestCase`，把 HTML/CSS 字符串喂给 `RenderPipeline`，再检查 DOM、layout 和 render primitives（`tests/wpt-runner/src/runner/mod.rs`）。

当前 runner 的主要断言包括：

- DOM 是否有 `body`、文本、特定 tag。
- render 是否完成、是否有 fill/glyph/stroke/image/shadow primitives。
- layout root 是否有 children、viewport 是否有效、宽高是否为正。
- `no_panic`、`nonzero_primitives` 等 smoke 级断言。

这些测试有价值，但它们证明的是“管线能跑、不会 panic、生成了一些结构”，不能证明：

- CSS selector/cascade/computed value 与规范一致。
- block/inline/flex/grid 具体几何正确。
- paint order、background、border、shadow、clip、overflow 等视觉输出正确。
- 两个视觉等价页面是否真的渲染一致。
- 字体、抗锯齿、DPI、平台 surface/GPU 路径是否稳定。

`tests/integration/src/webview_full_pipeline.rs` 也是类似情况：它验证 WebView 生命周期、加载 HTML、注入 CSS、resize、重复加载和脚本执行不出错，但不验证 CSS 样式是否按预期产生具体几何或像素结果。`RenderPipeline` 本身已把 HTML parse、CSS parse、style、layout、paint 串起来并返回 `RenderPrimitives` 与 `LayoutResult`，这为后续添加 layout dump、primitive snapshot 和 framebuffer pixel comparison 提供了合适插入点（`crates/engine/src/pipeline.rs`）。

> ### 💡 推理分析：ZeroWeb 测试现状定位
>
> **观察**：`wpt-runner` 已有 manifest 解析和无头渲染执行，但没有执行真实 WPT 文件、没有解析 `rel=match`、没有截图比较，也没有 expected metadata。
>
> **推理**：它当前更像“WPT-shaped smoke runner”，不是浏览器行业意义上的 WPT runner/reftest harness。
>
> **结论**：下一步不应再大量添加“has_children/has_fills”类断言，而应引入真正的 oracle：computed style/layout 数值断言、reftest screenshot comparison、expectation metadata。

> **📌 来源说明（第 4 章）**
>
> - **一手事实** [A][B][C][D][E]：ZeroWeb 本地源码直接观察。
> - **💡 推理**：根据 runner 断言类型判断其证明能力边界。
> - **⚠️ 假设**：当前没有其他隐藏测试系统在执行真实 reftest 或 pixel baseline。

## 5. 推荐给 ZeroWeb 的测试架构

### 5.1 分层测试金字塔

| 层 | ZeroWeb 应补能力 | 推荐优先级 | 通过标准 |
|----|------------------|------------|----------|
| L0 单元测试 | parser、selector、cascade、computed style、layout algorithm 数值断言 | P0 | 精确断言，无截图 |
| L1 primitive/layout snapshot | 对 `LayoutResult`、`RenderPrimitives` 做稳定文本/JSON snapshot | P0 | 可读 diff，固定排序，容忍浮点误差 |
| L2 reftest | 支持 `rel=match/mismatch`，同一引擎渲染 test/ref 后比较 framebuffer | P0 | pixel exact 或 fuzzy 通过 |
| L3 WPT subset runner | 导入真实 WPT 子集，维护 include list 和 metadata expectations | P1 | PR 跑核心子集，夜间跑扩大集 |
| L4 pixel baseline | 只对无法 reftest 的场景维护 PNG golden | P2 | 固定 OS/font/DPI/scale，带阈值 |
| L5 product/webview E2E | 对 `ZeroWebView` demo/host 做截图和交互 smoke | P2 | 可视回归 + 用户流程 |
| L6 differential/compat dashboard | 与 Servo/Firefox/Chromium 对比 WPT 分类通过率 | P3 | 趋势看板，不作为单 PR 阻断全部 |

### 5.2 最小可行实现

第一步建议做一个“ZeroWeb reftest harness”，范围不要过大：

1. 测试文件格式采用 WPT HTML 约定：解析 `<link rel="match" href="...">` 和 `<link rel="mismatch" href="...">`。
2. 固定 viewport：先用 800x600，和 WPT/Chromium 常见默认保持一致 [2][9]。
3. 渲染路径：`RenderPipeline::render_html` → `RenderPrimitives` → `render-foundation::cpu::render_scene_to_framebuffer`。
4. 比较器：实现 `max_channel_diff`、`total_different_pixels`、可选 `fuzzy`。
5. 输出：失败时保存 `actual.png`、`expected.png`、`diff.png` 和 JSON summary。
6. 期望文件：先做简单 `zero-web-wpt.ini`，支持 `expected: FAIL/SKIP/PASS`，后续再兼容 WPT metadata。

这个最小版本不需要立刻完整实现 WPT 的 `reftest-wait`、web fonts、HTTP server、multi-global、testdriver、wdspec。按照 WPT 自定义 runner 文档，枚举和执行测试的边界比表面看起来复杂，最好逐步兼容，并把真实 WPT manifest 作为 canonical enumeration 的长期目标 [8]。

### 5.3 哪些场景用什么测试

| 场景 | 首选测试 |
|------|----------|
| CSS token/parse/value 解析 | Rust unit test |
| selector specificity / cascade / inheritance | Rust unit test + computed style snapshot |
| `display:block`、margin、padding、width/height | layout geometry assertion |
| flex/grid/inline layout 最终视觉 | reftest |
| background/border/box-shadow/clip/overflow | reftest + primitive snapshot |
| text shaping、font fallback、underline | 少量 pixel/visual + 固定字体 |
| paint invalidation / dirty rect | primitive/layout diff + targeted pixel test |
| WebView load/resize/inject CSS | integration test + screenshot smoke |
| crash/hang/leak | crashtest + sanitizer/nightly |

### 5.4 CI 策略

| CI 阶段 | 跑什么 | 失败处理 |
|---------|--------|----------|
| PR fast | Rust unit tests、integration tests、核心 reftest subset | 阻断 |
| PR extended | WPT CSS/layout smoke subset、known-fail metadata | 阻断新增 unexpected fail |
| Nightly | 扩大 WPT subset、crashtest、fuzz、multi viewport | 生成报告 |
| Release | 固定环境 pixel baseline、WebView product smoke | 阻断严重视觉回归 |

不要让 pixel baseline 泛滥。Playwright 的视觉对比文档也提醒截图会随 OS、版本、设置、硬件、电源、headless/headed 等因素变化，基线应在相同环境生成和运行 [14]。这条经验对自建浏览器同样适用。

> **📌 来源说明（第 5 章）**
>
> - **一手事实** [2][7][8][9][13][14][15]：WPT reftest/metadata/custom runner、Chromium 测试偏好、Firefox fuzzy、Playwright 截图稳定性、Servo WPT 集成。
> - **一手事实** [D][E][F]：ZeroWeb `RenderPipeline` 与 CPU framebuffer 已具备接入点。
> - **💡 推理**：本章架构与优先级是作者综合，基于浏览器行业模式和 ZeroWeb 当前实现阶段。
> - **⚠️ 假设**：ZeroWeb 的短期目标是先建立稳定可跑的小型兼容性测试体系，再逐步扩大 WPT 覆盖。

## 6. 推荐落地路线

### Phase 1：把当前 smoke runner 升级为可比较输出

- 为 `LayoutResult` 增加稳定 dump：节点类型、x/y/w/h、display、主要 computed style。
- 为 `RenderPrimitives` 增加稳定 dump：fill/stroke/glyph/image/shadow 的 rect、color、z/order。
- 给已有内置测试补精确断言，减少 `layout_has_children` 这类弱断言。
- 成功标准：CSS/layout 改动能看到可读 diff，而不只是“有 children”。

### Phase 2：实现最小 reftest harness

- 在 `tests/wpt-runner` 增加 reftest case loader。
- 支持本地 test/ref HTML 文件和 inline CSS。
- 用 CPU framebuffer 做 pixel compare。
- 支持 `match`、`mismatch`、固定 viewport、简单 fuzzy。
- 成功标准：能跑一组自建 `tests/reftests/css/{block,margin,padding,background}`。

### Phase 3：导入真实 WPT 子集

- 建议先从 CSS 基础能力开始：`css/CSS2`、`css/css-display`、`css/css-sizing`、`css/css-flexbox` 中非常小的子集。
- 维护 include list，不追求一次导入全量。
- 维护 metadata/expectations，区分 `PASS`、`FAIL`、`SKIP`、`TIMEOUT`、`CRASH`。
- 成功标准：新增实现导致通过率提升时更新 expectations；无关改动不得引入 unexpected fail。

### Phase 4：WebView 产品级验收

- 对 `webview-demo` 或 headless WebView 增加固定页面截图 smoke。
- 覆盖 load、resize、inject CSS、navigation、scroll 后的可见结果。
- 这层用于“产品体验没有明显坏”，不要替代 engine 合规测试。

## 7. 结论

回答原问题：“如果通过自动化测试验证 webview 的渲染、排版、CSS 样式符合预期，一般浏览器怎么做？”

**简短答案**：一般浏览器用 WPT + 自家测试体系分层验证。CSSOM/API 用 JS 断言；layout/render 用 reftest；不能 reftest 的才用 pixel/visual baseline；稳定性用 crashtest；内部细节用 layout/render tree dump；CI 用 expected metadata 管理已知失败、平台差异和 flakiness。

对 ZeroWeb，最务实的下一步不是直接上全量截图回归，而是：

1. 先把当前 smoke runner 强化为精确 layout/primitive snapshot。
2. 再实现最小 reftest：test/ref 渲染到 CPU framebuffer 后比较。
3. 然后导入真实 WPT CSS/layout 小子集，维护 include list 和 expectation metadata。
4. 最后才引入少量固定环境 pixel baseline 和 WebView 产品截图验收。

## 参考资料

| 编号 | 来源 | 类型 | 用途 |
|------|------|------|------|
| [1] | [WPT: Writing Tests](https://web-platform-tests.org/writing-tests/) | 一手事实 | WPT 测试类型总览 |
| [2] | [WPT: Reftests](https://web-platform-tests.org/writing-tests/reftests.html) | 一手事实 | reftest 结构、fuzzy、限制 |
| [3] | [WPT: Rendering Test Guidelines](https://web-platform-tests.org/writing-tests/rendering.html) | 一手事实 | rendering test 设计原则 |
| [4] | [WPT: Visual Tests](https://web-platform-tests.org/writing-tests/visual.html) | 一手事实 | visual test 使用边界 |
| [5] | [WPT: crashtest tests](https://web-platform-tests.org/writing-tests/crashtest.html) | 一手事实 | crash/load 测试 |
| [6] | [WPT: CSS Metadata](https://web-platform-tests.org/writing-tests/css-metadata.html) | 一手事实 | CSS spec link 与 assert metadata |
| [7] | [WPT: Test Metadata](https://web-platform-tests.org/tools/wptrunner/docs/expectation.html) | 一手事实 | expected/fuzzy/disabled metadata |
| [8] | [WPT: Writing Your Own Runner](https://web-platform-tests.org/running-tests/custom-runner.html) | 一手事实 | 自定义 runner 注意事项 |
| [9] | [Chromium: Writing Web Tests](https://chromium.googlesource.com/chromium/src/+/main/docs/testing/writing_web_tests.md) | 一手事实 | Blink 测试类型偏好 |
| [10] | [Chromium: Web Test Expectations and Baselines](https://chromium.googlesource.com/chromium/src/+/main/docs/testing/web_test_expectations.md) | 一手事实 | expectations/baseline 思路 |
| [11] | [WebKit: Testing](https://docs.webkit.org/Build%20%26%20Debug/Tests.html) | 一手事实 | LayoutTests、expected files、TestExpectations |
| [12] | [WebKit: Web Platform Tests Integration](https://docs.webkit.org/Infrastructure/WPTTests.html) | 一手事实 | WebKit WPT 导入流程 |
| [13] | [Firefox: Layout Engine Visual Tests (reftest)](https://firefox-source-docs.mozilla.org/layout/Reftest.html) | 一手事实 | Gecko reftest manifest、fuzzy、适用范围 |
| [14] | [Playwright: Visual Comparisons](https://playwright.dev/docs/test-snapshots) | 外部搜索 | 应用级截图回归的环境稳定性提示 |
| [15] | [Servo Book: Testing](https://book.servo.org/contributing/testing.html) | 一手事实 | Rust 浏览器引擎 WPT 集成实践 |
| [16] | [Servo and the Web Platform Tests](https://servo.org/blog/2023/07/20/servo-web-platform-tests/) | 一手事实 | WPT 指导 Servo layout/CSS 进展 |
| [A] | `README.md` | 一手事实 | ZeroWeb 当前状态与 WPT 覆盖不足 |
| [B] | `docs/architecture.md` | 一手事实 | ZeroWeb 渲染链路与测试基础设施 |
| [C] | `tests/wpt-runner/src/manifest.rs` | 一手事实 | 当前 manifest 类型解析 |
| [D] | `tests/wpt-runner/src/runner/mod.rs` | 一手事实 | 当前 runner 断言能力 |
| [E] | `tests/integration/src/webview_full_pipeline.rs` | 一手事实 | 当前 WebView lifecycle 测试 |
| [F] | `crates/engine/src/pipeline.rs`、`crates/render-foundation/src/cpu.rs` | 一手事实 | render pipeline 与 CPU framebuffer 接入点 |
