# R57 证据：M3 oracle A/B 诚实化 + 修复批次（2026-08-16）

## 基线（R56h 终态，batch4 测量法）

- oracle A/B（全页对比 + channel=0）：真通过 2/117（1.7%）——**假测量**
  - 页面头部 h1/p.desc 字体差异 ~4% 地板主导全页 diff（canvas WPT 页面特有头部）
  - 两个「通过」用例均为退化（oracle 帧纯白）
  - 27% 的 filters dropShadow 差异实为 oracle 捕获环境（Chromium 150）不支持
    CanvasFilter（`typeof CanvasFilter === 'undefined'`）致脚本中止，oracle 帧无效

## 测量重构（wpt-runner reftest-oracle）

1. **canvas 内容矩形区域对比**：布局树定位 canvas 元素盒（painter 同偏移累计），
   多 canvas 取包围盒（canvas-grid reftest）；无 canvas 页面保持全页对比（零行为变化）。
   DC-3 测的是 canvas 绘制结果——头部文本是页面家具非 canvas 正确性。
2. **channel 容差用 DC-14 严格定义**（布局 ≤2 / 文字 ≤5，原实现误用 0）：Skia 预乘
   u8 整数管线 vs 我们的 float 管线 ±1 舍入差是跨引擎系统性差异（WPT fuzzy 注解
   容忍，如 2d.composite.full.mode.alpha 的 fuzzy="maxDifference=0-1"）。
3. **oracle 环境不支持排除（221 用例）**：test_html 含 `CanvasFilter` 或 `beginLayer`
   → 排除并计数。实测 Chromium 150：CanvasFilter undefined、`beginLayer()` 抛
   TypeError——捕获时脚本中止，oracle 帧为半渲染/空白，A/B 对比无意义（同
   testharness 面 reftest-format 的 NotRun 语义）。

**新基线（147 可测）**：真通过 **9（20.9%）**、近似 7（16.3%）、不一致 27
（原 117）。oracle-pass 16（37.2%），credible 15（34.9%）。

## 修复（WPT driving，全部带单测）

### bridge 接线补齐（R56h 只经 Rust 单测验证、bridge 漏分发的同模式 2 处）

- **setFilterDropShadow op**：JS shim 已发、Rust API 已有（6ae8da9bc），bridge 无
  arm → dropShadow 在页面路径从未生效。2d.filter.canvasFilterObject.dropShadow 的
  27% 差异实为 oracle 无效（Chromium 无 CanvasFilter），但该接线缺口真实存在。
- **setGradientInterpolation op**：colorInterpolationMethod/hueInterpolationMethod
  从未生效（2d.gradient.colorInterpolationMethod 的插值空间全被忽略）。

### dropShadow 字符串 filter（2d.filter.drop-shadow-globalAlpha 47.2% → 4.8%）

- `ctx.filter = 'drop-shadow(10px 5px 0px rgb(255,165,0))'` 字符串形式从未接线
  （R56h 只接 CanvasFilter 对象形式）→ 解析 offset/blur/color → host shadow 机制
- `_zwValidFilterList` 顶层逗号分割（rgb() 内逗号曾把函数拆断 → 字符串 filter
  整体忽略）；drop-shadow 参数括号深度分词

### 渐变插值空间补全（CSS Color 4 全 16 空间）

- `GradientColorSpace` + DisplayP3/DisplayP3Linear/A98Rgb/Rec2020/XyzD50
  （矩阵 + EOTF，与 css-parser color_math.rs 同源复制——层规则不跨依赖）
- JS VALID 列表补 6 名（canvas-grid 14 格曾 6 格 TypeError 中止 → 空白）

### 布局修复

- **shrink_inline_blocks_to_content 排除 replaced 元素**：canvas 曾被 fallback 文本
  宽（188px）收缩（2d.reset.render.global_composite_operation 的 0.47 横向压缩）
- **shift_siblings_after_ifc_grow**：inline-level 后继的重叠目标含 prev
  margin-bottom（块后继不加——margin 字段可能是塌穿折叠值，r1316 单测守护）；
  重叠目标统一 `pb + margin_target`（独立 else-if 会在首分支已移过时被跳过）
- **R2156 门**：含 block 子的 inline 不跳过 taffy（span>div+canvas 曾整棵丢失）；
  **R109 Inline 片段原子元素建独立子盒**（canvas 获得 LayoutBox，painter 可桥接）；
  **R109 片段不做 DOM 文本 remeasure**（span 全量内容测量曾把空白片段膨胀 721px）
- **replaced+fallback 元素不布局子内容**（HTML §4.8.10）：canvas/video/audio/
  iframe/embed/object/applet 的 fallback 子不建盒——fallback p 的 margin 塌穿
  canvas 16px + painter 叠绘 "FAIL (fallback content)" 文本

### reftest 脚本

- module 脚本块作用域包裹：真 module 每文件独立作用域，经典脚本同全局作用域让
  多 module 脚本的顶层 `const canvas = ...` 重声明互撞（canvas-grid 10 格 canvas
  各一个 module 脚本，第 2 格起全 SyntaxError 中止 → 格子空白）

### createElement('canvas') DOM 集成（2d.composite.full.mode.alpha 5.76% → 1.25%）

- standalone canvas（part05 _zwMakeCanvas）无 __zwHandle → appendChild 静默跳过
  （mutation 未记录 → 布局无 canvas 盒 → 渲染空白）。创建时同步 host handle +
  getContext 写 data-zw-canvas-ctx + width/height 同步属性

## 修复后逐用例

| 用例 | 修复前 | 修复后 | 说明 |
|------|--------|--------|------|
| 2d.reset.render.global_composite_operation | 6.68% | **0.17% 近似** | 布局全链修复（AA 边残差） |
| 2d.filter.drop-shadow-globalAlpha | 47.2% | **4.8% 近似** | 字符串 filter 接线（AA 边残差） |
| 2d.composite.full.mode.alpha | 5.76% | **1.25%** | standalone canvas DOM 集成 |
| 2d.gradient.colorInterpolationMethod | 10.71% | 92.9%* | grid 结构已渲染；22px IFC 偏移残差 |
| 真通过 | 2（假测量） | **9（20.9%）** | 诚实基线 |

\* 原 10.71% 为空白格子的假低值；修复后 canvas 渲染了，但 grid 用例的 22px
IFC 行内定位偏移使区域对比失准。

## 剩余不一致聚类（27 项）

1. **grid 结构用例 ~15 项**（gradient 92-95% / composite.grid 30-39% / TextCluster
   4.9-10%）：canvas 盒在 R109 匿名片段内 y 偏移 22px（IFC 行内定位深层问题——
   sync/valign 链，探针证实 IFC run.y=0 正确但最终盒 y=20）——深项，待后续
2. **fontKerning.none2 8.5%**：serif 字体度量差异（rendering-compat 域）
3. **drop-shadow AA 4.8%**：无抗锯齿光栅 vs Chromium AA（280px 级边差）
4. **text-outside-of-the-flat-tree 0.55%**：退化（oracle 纯白页）

## 验证

- canvas 800 / layout 1380 / engine 2156 / webview 599 / wpt-runner 171 全绿
- clippy 零警告；fmt 无 diff
- CSS reftest 冒烟 4 目录（flexbox/position/text/writing-modes/grid）0 Fail
- 测试资产：bridge 级回归 ×3（dropShadow 接线/reset 双矩形/渐变 fillRect+插值 op）、
  canvas 固有尺寸 taffy style 断言、canvas-grid span 结构断言

## process_learnings

- **R56h 教训复现**：桥接层（JS shim ↔ Rust API）的 op 分发是第三处断裂点——
  shim 已发 + API 已有 ≠ 链路通。跨层功能的验证必须走端到端路径（oracle A/B 或
  bridge 级测试），Rust 单测会假绿
- **oracle 帧有效性校验**：Chromium 版本不支持 tentative API 时，捕获的 oracle
  帧是「脚本中止」的产物——必须先排除（同 NotRun 语义）再谈通过率
- **测量范围决定语义**：canvas WPT 页面的头部文本（~4% 地板）不属于 DC-3 的
  「canvas 绘制结果」——区域对比让测量与目标对齐
- **±1 舍入差是跨引擎系统性差异**：Skia 预乘 u8 整数管线 vs float 管线，WPT
  用 fuzzy 注解容忍——oracle A/B 的 channel 容差应与其一致（DC-14 定义 ≤2）

## R57 batch-2 追加（2026-08-16 晚）

### grid 22px 偏移根因（R1286 strut 只给真 br）

- 现象：canvas-grid 结构（span > div + canvas）的 canvas 盒在 R109 匿名片段内
  y 偏移 22px（extract1 时 taffy y=0 正确，adjust_inline_block_positions 后 y=20）
- 根因链：① block 子（label div）在 IFC 中产生代理断行 → ② 断行前被折叠的空白
  文本行获 R1286 strut 高度（20px）→ ③ canvas 被推到第 2 行
- 修复：`InlineItem::BlockBreak` 变体（in-flow block 子代理断行，无 strut）；
  **float 保留 Br**（r1733 float-avoidance 依赖旧 strut 语义定位可用宽——单测守护）；
  纯空白文本 run 行盒 0 高（CSS §10.8.1）
- 效果：composite.grid 45%→24-38%、gradient 从空白到正确渲染（22px 归零）

### drawImage CTM 逆映射

- 现象：composite.grid 的 rotate(90°)+scale(0.6,1.2) 变换后绿矩形覆盖差 2.6 倍
  （436 vs 164 像素）
- 根因：draw_image_sized 沿**未变换**的 px/py 网格采样源——旋转/缩放下源采样
  方向错误（源坐标 = 网格索引而非逆映射位置）
- 修复：遍历变换后矩形包围盒 + 逆变换映射回目标矩形空间 → 源坐标
  （轴对齐下逐点等价零回归）；修复后覆盖 411（ref 436，残差为 AA 边）

### modern 颜色函数 stop → OKLab 默认插值

- oracle 实证：2d.gradient.colorInterpolationMethod 各格（srgb/hsl/hwb）中点均为
  (208,170,1) = OKLab 红→绿中点 (208,168,0)——Chromium 150 忽略 tentative 的
  colorInterpolationMethod 属性（srgb/hsl/hwb 接受但无效、srgb-linear 等抛
  TypeError 中止脚本）
- R56h 只覆盖 Mix/RelativeColor stop；补 color()/lab()/lch()/oklab()/oklch()/hwb()
- 该用例（colorInterpolationMethod + hueInterpolationMethod ×2）加入 oracle 环境
  不支持排除（Chromium 150 无法作 tentative API 的参考）

### canvas 文本真字体光栅（reftest 路径）

- 现象：TextCluster/fontKerning 等 canvas 文本用例 blank（9-10% 差异为「空白 vs
  文本」而非字形差）
- 根因：CanvasRegistry 默认 FontLoader::new() 为空——reftest 路径的 getContext2d
  `set_font_loader(reg 的空 loader)` → resolve_font_id None → draw_text_glyphs 只入
  primitives 不写像素 → canvas 快照（snapshot_rgba = pixel buffer）无文本
- 修复：reftest 注入 base 字体集（create_font_loader——系统 + CJK + Ahem + 泛型键）
- 效果：canvas 文本真正渲染（R56h 意图达成）；剩余差异为字体度量对齐
  （rendering-compat 域：serif/emoji 字形像素差）
- 注：2 个「真通过」用例系文本缺失的假通过，注入后被纠正（数字下降是诚实化）

### 剩余 27 项聚类

| 聚类 | 项数 | 值 | 类 |
|------|------|-----|-----|
| composite.grid | 12 | 24-38% | AA 边 + 合成边界舍入 |
| TextCluster/fontKerning/fontVariantCaps | 7 | 3.3-12.8% | 字体度量（rendering-compat 域） |
| reset miter_limit/after-rasterization | 2 | 1.4-2% | stroke/AA 边 |
| drop-shadow-globalAlpha | 1 | 4.8% | AA 边 |
| text-outside-of-the-flat-tree | 1 | 0.58% | 退化 oracle |

## R57 batch-3 探索记录（2026-08-16 深夜，否定结果——避免未来重复探索）

### 1px 格子/元素偏移（composite.grid 12 项 + drop-shadow 4.8% + reset 边残差的主因）

- 现象：canvas 元素 y 差 1px（ref 114 vs 我们 115，drop-shadow；composite.grid 每格
  累积 1px）。实测 ref 的 canvas_y=114（阴影顶 129 = 15+114 精确）、紫色矩形
  y[124,153] = 10+114 精确。
- 尝试 1（NORMAL_LINE_HEIGHT_RATIO 1.164 → generic 1.15）：R1185 实测 chromium
  generic = 1.150 + Liberation Serif hhea = 1.1499 双证。但 canvas 从 +1 变 -1
  （113 vs 114）——无净改善（h1 行盒走 IFC strut 1.2 近似，不受 resolve 影响；
  组合浮点 113.64 vs ref 114.4 取整差）。
- 尝试 2（ZW_PERFONT_LINEHEIGHT=1 + FontMetricMap 空族 fallback）：p.desc 18.4
  生效但 canvas 113（取整向下）——仍差 1；composite.grid 反变差（24→28）。
- 尝试 3（直接改 strut 1.2）：未实施——R834 记录 strut 真实化是深结构（welcome
  +0.44pp 风险，net 负先例）。
- 结论：1px = h1/p.desc 行盒（IFC strut 1.2 近似）+ 系统字体行高差（R631 域）
  + 浮点取整的组合——R834/R631 深项，canvas 专项无法轻量修复。

### AA 边（drop-shadow 阴影边、reset 边、composite.grid 旋转边）

- 整数坐标矩形（drop-shadow 紫色/阴影）：ref 边在整数像素（123 白/124 紫）——
  **无 AA 混合**——硬边即正确。
- 需要 AA 的仅非轴对齐/亚像素形状（composite.grid 的旋转边）——但 1px 格子偏移
  主导 composite.grid 的 24-38%（旋转边仅 ~25px/cell coverage 差）。
- 结论：AA 修复无法单独使 composite.grid < 1%（须 1px + AA 组合）——深项组合。

### reset.render.text 0.68% 近通过

- 差异 = 画布内文本字形像素差（y=116-123 文本行，1088 超容差）——字体度量域。

## R57 batch-4 探索记录（2026-08-16 深夜续）

### taffy round_layout（取整）探索

- 发现：taffy `use_rounding` 把 measure 的浮点行高取整（37.248→38、18.624→19）——
  Chromium 布局浮点不取整。疑似 1px 偏移根因。
- 实测 `disable_rounding()`：p taffy h=18.624（浮点生效）——但 drop-shadow 4.8%
  不变、composite.grid 24→25.85%（略差）——ref 的 canvas_y 恰为整数 114.0，我们的
  浮点 114.31（1.164 路径）覆盖率混合后边像素半色调 vs ref 满色——0.3px 覆盖相位差。
- 组合取整禁用 + 空族 1.15（p 行高 18.4）：h1 连带 1.15（36.8）——canvas 113.64 →
  123 半色调 vs ref 124 满色——仍差。h1 需要 1.164 而 p 需要 1.15（同字体无法
  路径区分——R834/R631 深项）。
- 结论：1px 偏移 = taffy 取整（0.3px）+ p 行高（0.2px）+ h1 行高（系统字体
  0.2-0.8px）的累积——各组件单独修复均被其他组件抵消——深项组合，已回退。

### AA 覆盖率（已实施保留）

- `rect_coverage`：非轴对齐 CTM 下像素相对变换后矩形的 4×4 超采样覆盖率——
  旋转 fillRect/clear_rect 边界像素源 alpha × coverage（ref Chromium AA 半色调）。
  轴对齐恒 1（整数矩形硬边零回归——802 测试全绿，含 2 新单测）。
- composite.grid 的 24-38% 主因为 1px 格子偏移（非 AA）——AA 覆盖率是 spec 正确性
  修复（旋转边），单测守护。
