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

以 **WPT reftest 通过率 95%+** 为核心验证指标，确保 ZeroWeb 的页面渲染效果在核心 CSS 领域与 Chromium（Chrome/Edge）一致。覆盖范围：

1. **CSS 2.1 核心**（`css/css2/`, `css/CSS2/`）— 渲染兼容性的基石
2. **Flexbox + Grid**（`css/css-flexbox/`, `css/css-grid/`）— 现代布局引擎必备
3. **Positioning + Float + Table + Multicol**（`css/css-position/`, `css/css-float/`, `css/css-tables/`, `css/css-multicol/`）— 传统布局模式完整覆盖
4. **文字排版全套**（`css/css-text/`, `css/css-writing-modes/`, `css/css-fonts/`, `css/css-text-decor/`）— 文本渲染正确性

执行方式：**交替推进** — 每轮执行同时扩展 WPT reftest 基础设施覆盖范围和修复发现的渲染缺口，直到目标通过率达标。

运行环境：**CPU 软件渲染 + GPU 渲染都必须通过** reftest 验证。

参考基准：**Chromium（Chrome/Edge）** 的渲染输出作为 reftest 的参考截图来源。

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
| Quirks mode | CSS parser / style system / layout engine 中实现完整的 quirks mode 调整 | DOM parser 已存储 quirks mode 但下游完全忽略；很多 CSS 2.1 reftest 会触发 quirks mode |
| CSS 2.1 渲染 | 盒模型、颜色、背景、边框、margin 折叠、inline formatting、BFC、浮动清除、基础定位 | 这是最大的 reftest 覆盖面，优先级最高 |
| Flexbox 渲染 | 所有 flex 属性的正确布局和绘制 | 已有 taffy 支撑，主要验证 + 修复边界 case |
| Grid 渲染 | 所有 grid 属性的正确布局和绘制 | 已有 taffy 支撑，主要验证 + 修复边界 case |
| Float 布局 | 完整的 float 布局算法，float exclusion、clear、BFC 触发 | 当前仅有 inline context 的 float exclusion zone，无原生 float layout |
| Table 布局 | 完整的 table layout 算法，table-layout: auto/fixed、border-collapse、spanning | 当前属性已存储但无专用布局算法 |
| Multi-column 布局 | column-count/column-width 的实际列排布、column-rule、column-span | 当前属性已存储但无列布局算法 |
| 文字排版 | OpenType shaping（liga/kern/features）、BiDi 算法、CJK 排版优化、text-align justify、word-break/overflow-wrap、writing-mode、vertical text | 当前 fontdue 仅做简单 character-to-glyph 映射 |
| Position 定位 | absolute/relative/fixed/sticky 的精确坐标计算 | fixed/sticky 当前有简化处理 |
| Reftest 验证 | CPU 软件渲染模式 + GPU 渲染模式的截图对比 | 两种模式都需通过 |
| 范围外 reftest 过滤 | 导入时自动过滤或标记范围外 reftest（SVG、Canvas、WebGL 等），维护 skip list | 防止范围外 case 膨胀分母 |
| 渲染缺口修复 | 任何导致 reftest 失败的渲染错误 | 由 reftest 结果驱动 |

### 不在范围内（明确排除）

- **非 CSS 渲染领域的兼容性**：JS/DOM API 兼容性、网络协议兼容性、安全策略兼容性不在本目标范围内（由父目标 `zero-web.md` 覆盖）
- **Canvas / WebGL / WebGPU**：不在本目标 reftest 范围内
- **动画/交互的帧级正确性**：CSS animation/transition 的视觉正确性验证不作为 reftest 核心指标（但如果有 reftest 覆盖则需通过）
- **性能优化**：本目标关注渲染正确性，不关注渲染性能（由父目标的性能基准体系覆盖）
- **Chromium 专属行为**：只对齐标准规范行为，不复制 Chromium 的 bug 或非标准行为
- **新 crate 依赖的大规模引入**：最小化新依赖，仅在必要时引入许可证兼容的 crate
- **SVG 渲染**：不在本目标范围

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
  - `crates/render-foundation/src/cpu.rs`：`render_scene_to_framebuffer()` — CPU 软件渲染截图
  - `crates/script-sandbox/`：V8 runtime — 用于 reftest harness 中执行 JS
  - `tests/wpt-runner/src/runner/mod.rs`：`TestExpectations` 机制 — 可扩展为 reftest skip list

### 渐进覆盖策略

WPT reftest 数以万计，不可能一次导入全部。按以下优先级分批导入：

**Phase 1 — 基础设施 + CSS 2.1 核心抽样**：
- 建立 WPT reftest 导入/运行/对比/报告基础设施
- 导入 CSS 2.1 核心子集（~200 个 reftest），建立初始基线
- 修复发现的 CSS 2.1 渲染缺口

**Phase 2 — 布局模式全覆盖**：
- 导入 Flexbox + Grid reftest 子集
- 导入 Positioning + Float + Table + Multicol reftest 子集
- 修复所有布局模式渲染缺口

**Phase 3 — 文字排版 + 全量扩展**：
- 导入文字排版全套 reftest
- 扩大各领域 reftest 覆盖到目标范围
- 达到 95%+ 总通过率

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
  - 布局类（不含文字渲染）：严格容差（max_diff_ratio ≤ 1%, max_channel_diff ≤ 5）
  - 文字类：宽松容差（具体数值由首轮 reftest 实测校准）
  - 优先使用 WPT fuzzy 注解的 per-test 容差，无注解时使用分类默认值
- [ ] **范围外 reftest 过滤**：导入时自动过滤或标记范围外 reftest（SVG、Canvas、WebGL），维护 skip list 文件（如 `tests/wpt-runner/reftest-skip-list.txt`）
- [ ] 通过率报告按 WPT 目录分类输出（文本 + JSON 格式）
- [ ] Reftest 运行可通过单一命令执行（如 `cargo run --bin wpt-reftest`）
- [ ] CI 管线中集成 reftest 运行（至少 CPU 模式）

### DC-2: CSS 2.1 核心通过率 ≥ 95%

- [ ] `css/css2/` 和 `css/CSS2/` 目录下导入的 reftest 子集中，通过率 ≥ 95%
- [ ] 覆盖：盒模型、margin 折叠、BFC、inline formatting、颜色、背景、边框、基础定位
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标

### DC-3: Flexbox + Grid 通过率 ≥ 95%

- [ ] `css/css-flexbox/` 导入的 reftest 子集中，通过率 ≥ 95%
- [ ] `css/css-grid/` 导入的 reftest 子集中，通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标

### DC-4: Positioning + Float + Table + Multicol 通过率 ≥ 95%

- [ ] `css/css-position/` 导入的 reftest 子集中，通过率 ≥ 95%
- [ ] `css/css-float/` 导入的 reftest 子集中，通过率 ≥ 95%
- [ ] `css/css-tables/` 导入的 reftest 子集中，通过率 ≥ 95%
- [ ] `css/css-multicol/` 导入的 reftest 子集中，通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标

### DC-5: 文字排版通过率 ≥ 95%

- [ ] `css/css-text/` 导入的 reftest 子集中，通过率 ≥ 95%
- [ ] `css/css-writing-modes/` 导入的 reftest 子集中，通过率 ≥ 95%
- [ ] `css/css-fonts/` 导入的 reftest 子集中，通过率 ≥ 95%
- [ ] `css/css-text-decor/` 导入的 reftest 子集中，通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标

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

### 通过率统计口径

- **统计对象**：仅指从上游 WPT 导入的真实 reftest case，**不含**现有 1,341 个手写 `TestCase`
- **现有 1,341 个手写 TestCase**：保留为 smoke test 套件，继续全绿运行，但不计入本目标的 reftest 通过率统计
- **分母**：每个 WPT 目录下**已导入并注册**的 reftest case 数量（不是上游全部 reftest 数量），排除 skip list 中的范围外 case
- **分子**：运行后判定为 PASS 的 case 数量
- **通过率** = 分子 / 分母 × 100%
- **要求**：分母必须 ≥ 每个目录 50 个 reftest case（确保统计有意义），否则需扩大导入范围
- **禁止**：不允许通过缩小导入范围来人为提高通过率

---

## Current Proven Baseline

截至 2026-06-06，项目渲染兼容性现状：

### 已有能力

| 领域 | 状态 | 详情 |
|------|------|------|
| 渲染管线 | ✅ 全链路贯通 | HTML → CSS → Style → Layout → Paint → Composite 完整可用 |
| CSS 属性解析 | ✅ 100+ 属性 | box model、flexbox、grid、border、background、transform、animation、transition 等 |
| Flexbox/Grid 布局 | ✅ 基于 taffy | 所有子属性均已接入 |
| Block/Inline 布局 | ✅ 基础可用 | Block via taffy, Inline via 自建 InlineFormattingContext |
| 文字渲染 | ⚠️ 基础可用 | fontdue 加载 + glyph 映射，CJK fallback chain，text-align/word-break/white-space 等 |
| WPT runner | ⚠️ smoke 级 | 1,341 个手写 TestCase，证明"不 panic + 有 primitives"，不证明渲染正确 |
| Reftest harness | ⚠️ 最小可用 | 16 个内建 reftest case，pixel-level 对比基础设施已有，未接入 CI |
| CPU 软件渲染 | ✅ 可用 | `render-foundation/src/cpu/` |
| GPU 渲染 | ✅ 可用 | `render-foundation/src/gpu/`，wgpu + WGSL shaders |

### 已知关键缺口

| 缺口 | 影响范围 | 当前状态 |
|------|----------|----------|
| Float 布局 | CSS 2.1 核心功能 | 仅有 inline context 的 float exclusion zone，无原生 float layout 算法 |
| Table 布局 | 表格渲染 | 属性已存储，无专用 table layout 算法 |
| Multi-column | 多列布局 | 属性已存储，无列布局算法 |
| OpenType shaping | 文字排版质量 | fontdue 仅做简单 character-to-glyph，无 liga/kern/features |
| BiDi 算法 | RTL 文本 | 属性已存储（direction, unicode-bidi），无实现 |
| Vertical writing-mode | 竖排文本 | 属性已存储，无实现 |
| Quirks mode | CSS 2.1 兼容性 | DOM parser 存储了 quirks mode 但 CSS parser / style system / layout engine 完全忽略它；很多 CSS 2.1 reftest 会触发 quirks mode |
| WPT reftest 导入 | 验证基础设施 | 无真实 WPT 上游测试导入能力 |
| Chromium 参考截图 | 验证基准 | 无自动化 headless Chromium 截图工具链（需构建 Puppeteer/Playwright 方案） |
| 字体像素差异 | reftest 对比可行性 | fontdue vs Skia 字体渲染结果像素级不同，现有容差（1%/5ch）对文字类 reftest 太严格 |
| WPT fuzzy 注解 | reftest 对比精度 | 上游 reftest 自带 fuzzy() 容差声明，现有代码不解析也不应用 |
| Viewport 对齐 | reftest 对比正确性 | ZeroWeb 和 Chromium 截图必须在相同 viewport 下捕获，当前无强制机制 |
| JS 执行 | reftest 覆盖范围 | 很多 WPT CSS reftest 依赖 JS 动态设置条件，但 `RenderPipeline::render_html()` 不执行 JS |
| 范围外 reftest 过滤 | 导入范围控制 | 无 skip list / expectation file 机制，导入时可能包含 SVG/Canvas/WebGL 等 range 外 case |
| 视觉回归系统 | 持续验证 | 无 golden image 系统 |
| `#[ignore]` 测试 | 测试完整性 | 60 个测试标记 `#[ignore]`，因本地网络不稳定，保留 `#[ignore]` |

### 测试基线

- 总测试数：~12,001，全绿
- Coverage：95.46% line, 96.94% function, 94.88% region
- **关键事实**：当前 WPT runner 是 smoke test，不证明渲染正确性。本目标的核心挑战是从"不崩溃"升级到"渲染正确"。

---

## Single Active Milestone

**当前活跃里程碑**：M1 — WPT Reftest 基础设施搭建

### M1 目标

建立能够导入、运行、对比和报告 WPT reftest 的完整基础设施。

### M1 完成标准

1. [ ] 可以 fetch 上游 WPT 仓库（或指定目录子集）
2. [ ] **扩展** `tests/wpt-runner/src/manifest.rs`：解析 WPT MANIFEST.json 中 reftest 条目的 `fuzzy()` 元数据（maxDiff、maxPixel）
3. [ ] 可以用 ZeroWeb 的 CPU 软件渲染器对 reftest HTML 文件截图（**复用** `render_scene_to_framebuffer`）
4. [ ] 可以用 ZeroWeb 的 GPU 渲染器对 reftest HTML 文件截图
5. [ ] **自动化 headless Chromium 截图工具**：通过 Puppeteer/Playwright 脚本自动在 headless Chromium 中渲染 reftest HTML 并截图，输出到指定目录
6. [ ] **Viewport 对齐机制**：ZeroWeb 和 Chromium 截图在相同 viewport 下捕获（默认 800×600），配置可传递
7. [ ] **JS 执行集成**：Reftest harness 在截图前通过 `script-sandbox` V8 runtime 执行页面 JS
8. [ ] **分类容差机制**：在**扩展** `tests/wpt-runner/src/reftest.rs` 的 `ReftestConfig` 基础上，支持分类容差（布局类 vs 文字类）和 per-test WPT fuzzy 注解
9. [ ] **范围外 reftest 过滤**：维护 `tests/wpt-runner/reftest-skip-list.txt`，导入时自动过滤 SVG/Canvas/WebGL 等范围外 case
10. [ ] 按目录分类的通过率报告输出（文本 + JSON）
11. [ ] 单一命令运行全部已导入 reftest（如 `cargo run --bin wpt-reftest`）
12. [ ] 导入 CSS 2.1 核心子集 ≥ 50 个 reftest case 并建立初始基线
13. [ ] 记录初始通过率（不要求达标，但必须可测量）
14. [ ] **确认 `#[ignore]` 标记状态**：`tests/integration/src/real_website_compat.rs` 中的真实网站测试因本地网络不稳定保留 `#[ignore]`；确认其余所有测试零 `#[ignore]`

### M1 影响范围

- **主要修改**：`tests/wpt-runner/`（升级现有 WPT runner，**扩展而非重写** `reftest.rs` 和 `manifest.rs`）
- **新增文件**：Chromium 截图工具脚本（如 `tests/wpt-runner/scripts/capture-chromium-screenshots.mjs`）、reftest skip list 文件
- **可能修改**：`crates/render-foundation/src/surface.rs`（截图功能）、`crates/render-foundation/src/cpu/`（CPU 渲染截图输出）、`crates/engine/src/pipeline.rs`（JS 执行集成）
- **确认状态**：`tests/integration/src/real_website_compat.rs`（保留 60 个 `#[ignore]` 标记，因本地网络不稳定）
- **不允许修改**：`crates/css-parser/`、`crates/style-system/`、`crates/layout-engine/`（M1 只建基础设施，不改渲染逻辑；`crates/engine/` 仅允许 JS 执行集成改动）

---

## Ordered Next Milestones

### M2 — CSS 2.1 核心渲染修复 + Quirks Mode

**目标**：修复 CSS 2.1 reftest 发现的渲染错误，实现完整 quirks mode，达到 CSS 2.1 核心通过率 ≥ 95%。

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

**目标**：修复 Flexbox 和 Grid reftest 发现的渲染错误，达到各自通过率 ≥ 95%。

**范围**：
- Flexbox 所有子属性的正确布局
- Grid 所有子属性的正确布局
- 响应式布局 edge case
- 嵌套 flex/grid 场景

**依赖**：M1 完成

### M4 — Float + Table + Multicol 布局算法实现

**目标**：实现缺失的布局算法（Float、Table、Multi-column），达到各自 reftest 通过率 ≥ 95%。

**范围**：
- 完整 float 布局算法
- 完整 table layout 算法（table-layout: auto/fixed、border-collapse、spanning）
- Multi-column 布局算法
- position: fixed/sticky 的精确实现

**依赖**：M1 完成（M2/M3 可并行）

### M5 — 文字排版能力实现

**目标**：实现完整的文字排版能力，达到文字排版 reftest 通过率 ≥ 95%。

**范围**：
- OpenType shaping（ligatures、kerning、features）— 可能引入 `rustybuzz`
- BiDi 算法实现 — 可能引入 `unicode-bidi`
- CJK 排版优化
- writing-mode: vertical-* 实现
- text-align: justify 的精确实现
- word-break / overflow-wrap / hyphens 的完整实现
- text-decoration 的精确绘制

**依赖**：M1 完成（M2/M3/M4 可并行）

### M6 — 全量扩展 + 通过率冲刺

**目标**：扩大各领域 reftest 覆盖范围，达到总体 95%+ 通过率。

**范围**：
- 扩大每个目录的 reftest 导入数量（目标每个目录 ≥ 100 个 case）
- 修复所有剩余渲染缺口
- CPU + GPU 双模式验证
- 回归测试确保已通过的 case 不退化

**依赖**：M2-M5 完成

---

## Testing & Quality Gates

### 测试层次

| 层次 | 内容 | 运行频率 |
|------|------|----------|
| 单元测试 | 每个 crate 的 `#[test]` 测试 | 每次修改后 |
| 集成测试 | 跨 crate pipeline 测试 | 每次修改后 |
| WPT reftest（CPU 模式） | ZeroWeb CPU 渲染 vs Chromium 截图 | 每个 milestone 验证 |
| WPT reftest（GPU 模式） | ZeroWeb GPU 渲染 vs Chromium 截图 | 每个 milestone 验证 |
| 全量回归 | `cargo test` + reftest 全量 | 每轮执行结束 |

### 质量门禁

| 门禁 | 标准 | 不通过时的处理 |
|------|------|----------------|
| 编译 | `cargo build` 零错误 | 立即修复 |
| Clippy | `cargo clippy -- -D warnings` 零警告 | 立即修复 |
| 现有测试 | `cargo test` 零失败 | 立即修复，不允许带着红灯继续 |
| 格式化 | `cargo fmt` 无变更 | 提交前格式化 |
| 新增代码测试覆盖 | 每个渲染修复必须有对应单元测试 | 不允许只改代码不加测试 |
| Reftest 通过率 | 按 Done Criteria 中各领域 ≥ 95% | 继续修复直到达标 |

### Coverage 要求

- 现有测试必须持续全绿
- **真实网站测试保留 `#[ignore]`**：`tests/integration/src/real_website_compat.rs` 中的真实网站兼容性测试因本地网络不稳定，保留 `#[ignore]` 标记，不要求执行。这些测试不计入本目标的通过率统计
- 其余所有测试零 `#[ignore]`：除真实网站测试外，不允许引入新的 `#[ignore]` / skip 标记。如果某个测试需要外部资源（网络、文件），应在测试中做超时和错误处理，而不是跳过
- 新增功能、行为变化、兼容性扩展和回归修复必须同步补单元测试
- Coverage 作为长期主线任务的一部分持续扩大
- 不允许通过缩小统计范围来伪造达标
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

**尚未启动**。本目标文档刚创建，无执行证据。

执行代理首轮必须完成的工作见下方"首轮进入检查清单"。

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
- [ ] 检查现有 WPT runner 和 reftest harness 的具体实现状态
- [ ] 确认现有测试基线（运行 `cargo test` 确保全绿）
- [ ] **确认 `#[ignore]` 标记状态**：`tests/integration/src/real_website_compat.rs` 中的真实网站测试因本地网络不稳定保留 `#[ignore]`（这是已知的、合理的例外）。确认仓库其余部分零 `#[ignore]`
- [ ] 创建 `docs/goal/rendering-compat/master.md`，包含完整的当前状态评估和 M1 计划
- [ ] 创建 `docs/goal/rendering-compat/archive/` 目录
- [ ] 创建 `docs/goal/rendering-compat/evidence/` 目录
- [ ] 选定并启动第一个活跃里程碑（M1 — WPT Reftest 基础设施搭建）

**关键要求**：完成 master.md 和目录初始化后，执行代理**必须**在同一轮内继续启动 M1，直接推进核心基础设施能力。**不允许**把"文档框架已建立"当作里程碑完成或收工理由。

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

1. ✅ Done Criteria DC-1 到 DC-7 全部满足
2. ✅ 所有四个 WPT 领域（CSS 2.1、Flexbox+Grid、布局模式、文字排版）通过率均 ≥ 95%
3. ✅ CPU 软件渲染 + GPU 渲染双模式均达标
4. ✅ `cargo build` + `cargo test` + `cargo clippy` 全通过
5. ✅ 有结构化的 reftest 通过率报告作为自动化证据
6. ✅ master.md 内部自洽，archive 已建立，进度已归档
7. ✅ 渲染能力本身达到可验证的 production-ready 质量

### 禁止输出 DONE 的情况

即使以下情况中部分条件看起来"还行"，也**不允许**输出 DONE：

- ❌ master.md 缺失、必填章节缺失、archive/evidence 为空且无有效里程碑
- ❌ 无 reftest 证据，或 reftest 存在未分析的失败项
- ❌ 无实际代码/测试进度（仅有文档和计划）
- ❌ 通过率无法证明（无 reftest 报告、无截图证据）
- ❌ master.md 各章节矛盾（如"通过率未达标但证据声称全部满足"）
- ❌ 所有 master.md 章节都填了、archive 建了、计划列了，但没有真实 reftest 运行结果和渲染修复
- ❌ 测试全绿、reftest 通过率达标、文档完整，但目标渲染能力本身未达到可验证的 production-ready 质量
- ❌ 只验证了 CPU 渲染或 GPU 渲染其中一种模式

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

1. **扩展基础设施**：导入更多 WPT reftest case，扩大覆盖范围
2. **运行 reftest**：获取当前通过率，分析失败 case
3. **修复渲染缺口**：针对失败 case 修复 CSS parser / style system / layout engine / render foundation
4. **补充测试**：为每个修复添加单元测试
5. **验证回归**：确保修复不破坏已有通过的 case
6. **更新文档**：更新 master.md 状态和 evidence

### 现有基础设施复用原则

M1 及后续 milestone **必须优先扩展现有模块**，禁止重写已有功能：

- 像素对比引擎：扩展 `tests/wpt-runner/src/reftest.rs` 的 `ReftestConfig` 和 `compare_pixels()`，添加分类容差和 WPT fuzzy 注解支持
- WPT MANIFEST 解析：扩展 `tests/wpt-runner/src/manifest.rs`，添加 fuzzy 元数据解析
- CPU 截图：复用 `render_scene_to_framebuffer()`
- Smoke test 套件：保留现有 1,341 个手写 TestCase 继续运行，不删除、不替换
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
