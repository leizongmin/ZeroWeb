# Spec：Print Layout Phase P1 — 相对定位分页模型（page-break-before 强制换页）

**版本**：v1.0
**日期**：2026-07-24（R1999，承接 R1998「Phase P1 须 dedicated spec-rfc 设计相对定位 pagination 模型」）
**作者**：rally agent（rendering-compat 目标）
**状态**：已确认（rally 自主模式——ruling #4 2026-07-16 预授权深水区多会话切片；每切片 kill-switch + A/B 守 net≥0）

---

## 0. 执行摘要

- **一句话目标**：在 `media_type == Print` 时，把文档主块流按固定页高（page box）分页，使 `page-break-before/after: always`（CSS2）/ `break-before/after: page`（CSS3）强制换页生效——Ctrl+P 打印预览显示**分页**内容（当前 Print 模式仅套 @media print CSS、不分页）。
- **本期范围（Phase P1a）**：仅在**文档块流根的单一直接子层级**（body 的直接 in-flow block 子）做分片；强制换页 + 自然页填充（复用 multicol 分配算法）；tall-framebuffer 输出（页边界以空白间隔可见）。
- **明确排除（本期不做）**：① 嵌套强制换页的**精确**定位（break 声明在深层元素时只把整单元推到新页，非精确在该元素处断——P2）；② `page-break-inside: avoid`（P3）；③ `@page { size; margin }` 解析（P4，本期用默认页尺寸常量）；④ 页边界分隔线绘制（P1.5，需 LayoutResult 新字段 + paint 步骤）；⑤ 分页输出模型 / 多页预览翻页 / 实际打印（P5，根本改动，独立 spec-rfc）。
- **核心约束**：① **Screen 模式字节零变更**（分页仅 Print 触发）；② kill-switch `ZW_PRINT_PAGINATE` default-**off**（首切片，A/B 证明后下轮翻 default-on）；③ 不侵入主 layout / multicol（独立 post-process pass，避 R125/R206/R213 deadlock 史）；④ 每切片 A/B 守 net≥0 + product-smoke 绿。
- **推荐方案**：**单层分片**——镜像 multicol 的 `assign_children_to_columns_with_breaking`（multicol.rs:1217）→ 新 `assign_children_to_pages`，把「相对定位 sibling-shift」复杂度**边界化到单一层级**（详见 §8.4）。
- **首个落地步骤**：style-system 加 `media_type()` getter → LayoutEngine 加 `media_type` 字段 + `set_media_type` → pipeline.set_media_type 同设 layout_engine → 新增 `paginate_for_print` post-process（engine.rs compute() 末尾，gate media_type==Print + env）。

> **本 spec 解决的 R1998 缺口**：R1998 确认 LayoutBox 位置**相对父内容区**（types/mod.rs:30），故分页偏移级联**非简单 walk**——须算绝对 y、须 shift 同父兄弟、跨祖先子树 shift 复杂。本 spec 的核心贡献：证明**把分片限制在单一层级**（body 直接子）可把该复杂度消解为「一次线性分配」——sibling shift 就是分配算法本身，后代因相对父单元而**自动跟随**，跨祖先复杂度只在**多层**（嵌套）分片时出现（→ P2）。P1a 因此是 clean bounded 切片。

---

## 1. 背景与目标

### 1.1 背景

@media print 全弧已完成（R1981 cascade → R1991 reftest `--media` → R1992 webview API → R1993 browser Ctrl+P → R1994 UX 徽标）。当前 Print 模式 = 「Screen 页面套 @media print CSS」（隐藏 screen-only / 显 print-only 内容），但**不分页**——`page-break-*` / `break-*` 的**解析 + 计算值已就绪**（`BreakValue`/`PageBreakValue` enum + `ComputedStyle.break_before/after/inside` + `page_break_before/after/inside` + 继承），却**未被布局消费**：Print 模式仍单视口整页渲染。

R1995（print-layout-pagination-design.md v0.1）做了范围界定 + fragmentation 复用评估 + 5-phase 计划，但**未详相对定位 pagination 模型**。R1998 核查确认：LayoutBox 位置相对父内容区 → Phase P1 offset pagination 非简单 walk，须 dedicated spec-rfc。**本 spec 即该 dedicated 设计**，把 R1995 的「Phase P1」细化为可实施的 P1a 首切片。

### 1.2 目标

- **业务目标**：让 ZeroBrowser Ctrl+P 打印预览显示**分页**内容（用户可见的 print-preview 正确性，DC-12/DC-13），并具备真 print-layout 能力（@media print 的「完整化」）。
- **用户目标**：Ctrl+P 后看到内容按页分隔（强制换页处内容跳到下一页），而非当前的单视口长页。

### 1.3 范围边界

- **在范围内（P1a）**：
  - Print 模式（`media_type == Print`）触发分页 post-process。
  - 强制换页：`page-break-before: always` / `break-before: page`（及 after 对称）。
  - 自然页填充：内容超过页高自动续到下一页（复用 multicol column breaking，纵向版）。
  - 单层分片：文档块流根（body）的直接 in-flow block 子作为分片单元。
  - 默认页尺寸常量（A4 @96dpi，P4 前 @page 未解析）。
  - tall-framebuffer 输出（文档总高 = 页数 × 页高；页边界以空白间隔可见）。
- **不在范围内（后续 phase）**：
  - 嵌套强制换页的**精确**定位（P2，绝对坐标 remap）。
  - `page-break-inside: avoid`（P3）。
  - `@page { size; margin }` 解析（P4）。
  - 页边界分隔线绘制（P1.5）。
  - 分页输出模型 / 多页预览翻页 / 打印对话框 / PDF（P5，根本改动）。
  - `orphans` / `widows`（后续）。

> **范围诚实（EV）**：reftest 低 EV（仅 ~6 个 @media print WPT case）。本 feature 主要价值 = **产品 print-preview 正确性**（DC-12/13）+ 真 print-layout 能力，**不动 headline**（≥95% 仍 font-stack-gated，ruling #2）。

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|------|---------|------|
| 业务需求 | 是 | print-preview 正确性（DC-12/13） |
| 用户需求 | 是 | Ctrl+P 看到分页内容 |
| 解决方案需求 | 是 | 复用 multicol fragmentation 模型（§8） |
| 功能需求 | 是 | §3（FR-001~FR-006） |
| 非功能需求 | 是 | §4（Screen 零回归、kill-switch、性能） |
| 接口需求 | 是 | §5（LayoutEngine.media_type / env / LayoutResult） |
| 过渡需求 | 否 | 新 feature，无迁移 |

---

## 3. 功能需求

### FR-001：Print 模式触发分页 post-process
- **描述**：当 `media_type == Print` 且 env `ZW_PRINT_PAGINATE != "0"` 时，layout 在所有现有 post-process（multicol / relative / clamp）之后，对文档块流根的直接 in-flow block 子执行分页 post-process；Screen 模式（默认）**不执行**该 pass。
- **优先级**：必须
- **来源**：R1998（media_type 未流入 layout）/ R1995 §5.1

**验收场景**：

```
场景: Print 模式触发分页（正常路径）
  假设 media_type==Print、ZW_PRINT_PAGINATE 未设（default-off）或 =1
  当 加载含 page-break-before:always 的页面并 layout
  那么 有 forced-break 的元素被推到下一页边界（abs_y 落在 k*page_height 处）
  验证: 单测 r1999_print_paginate_forced_break_pushes_to_page_boundary（probe，见 §8.8）

场景: Screen 模式不触发分页（零回归，异常/守卫路径）
  假设 media_type==Screen（默认）
  当 加载同一页面并 layout
  那么 LayoutResult.root 与未接入分页 post-process 时**逐字段相等**（分页 pass 未运行）
  验证: 单测 r1999_screen_mode_layout_byte_identical_no_pagination

场景: ZW_PRINT_PAGINATE=0 关闭分页（kill-switch，守卫路径）
  假设 media_type==Print 但 ZW_PRINT_PAGINATE=0
  当 layout
  那么 分页 pass 不运行（LayoutResult 与 Screen 相同语义，Print CSS 仍级联但不分页）
  验证: 单测 r1999_print_paginate_killswitch_disables_pass
```

### FR-002：强制换页 page-break-before / break-before:page
- **描述**：Print 模式下，声明 `page-break-before: always`（CSS2）或 `break-before: page`（CSS3）的块流单元，起始于新页（其顶部 abs_y 对齐到 `ceil(abs_y / page_height) * page_height`）；该单元后的所有同父兄弟单元相应下移。
- **优先级**：必须
- **来源**：CSS2 §13 / CSS3 Fragmentation / R1995 FR-002

**验收场景**：

```
场景: page-break-before 推到下一页边界（正常路径）
  假设 Print 模式、page_height=H、body 直接子 [A 高100, B(page-break-before:always) 高50, C 高30]，A 起于 y=0
  当 分页 post-process
  那么 B 的 abs_top 落在 H（第2页顶），C 落在 B 之后（H+50），A 仍在第1页（y=0..100）
  验证: r1999_print_paginate_forced_break_pushes_to_page_boundary（断言 B.abs_top == H、C.abs_top == H+50）

场景: break-before:page 等价 page-break-before:always（正常路径，CSS3 变体）
  假设 同上但 B 用 break-before:page
  当 分页
  那么 结果与 page-break-before:always 逐字节相同（BreakValue::Page 与 PageBreakValue::Always 等价处理）
  验证: r1999_print_paginate_break_before_page_equivalent

场景: 无 forced break 时不移动单元（守卫路径）
  假设 Print 模式、body 子全 break-before:auto
  当 分页（仅自然页填充或无操作）
  那么 单元顺序与原始 taffy 位置一致（无强制位移）；自然页填充按 page_height 切分（P1a 含）
  验证: r1999_print_paginate_no_forced_break_preserves_order
```

### FR-003：强制换页 page-break-after / break-after:page（对称）
- **描述**：Print 模式下，声明 `page-break-after: always` / `break-after: page` 的单元之后强制换页（其下一个兄弟起于新页）。
- **优先级**：应该
- **来源**：CSS2 §13 / R1995 FR-003

**验收场景**：

```
场景: page-break-after 推下一个兄弟到新页（正常路径）
  假设 Print、body 子 [A(page-break-after:always) 高100, B 高50]
  当 分页
  那么 A 在第1页（y=0..100），B 在第2页顶（abs_top == H）
  验证: r1999_print_paginate_break_after_pushes_next_sibling

场景: 末单元 page-break-after 无后续兄弟（守卫/无操作路径）
  假设 Print、body 子 [A(page-break-after:always) 高100]（无后续兄弟）
  当 分页
  那么 A 位置不变；after-break 无作用对象（no-op，不创建空页）
  验证: r1999_print_paginate_break_after_trailing_noop
```

### FR-004：自然页填充（block fragmentation，纵向版）
- **描述**：Print 模式下，内容流超过 page_height 时自动续到下一页（复用 multicol `assign_children_to_columns_with_breaking` 的 column breaking 逻辑，纵向堆叠而非横向并排）。
- **优先级**：应该
- **来源**：CSS §13 / multicol.rs:1217 复用

**验收场景**：

```
场景: 内容超页高自然续页（正常路径）
  假设 Print、page_height=H=100、body 子 [A 高60, B 高60]（无 forced break）
  当 分页（自然填充）
  那么 A 在第1页（y=0..60），B 放不下第1页剩余40 → 移到第2页顶（abs_top == 100）
  验证: r1999_print_paginate_natural_fill_overflows_to_next_page

场景: 单元高于整页（oversized）拆分（异常路径，复用 multicol breaking）
  假设 Print、page_height=100、body 子 [A 高250]（单元素超高）
  当 分页
  那么 A 拆为多页片段（第1页 0..100、第2页 100..200、第3页 200..250），复用 fragment_y_offset 机制
  验证: r1999_print_paginate_oversized_unit_fragments_across_pages
```

### FR-005：默认页尺寸常量（@page 未解析前的占位）
- **描述**：Print 模式分页使用默认页高常量（A4 @96dpi：宽 793.7px / 高 1122.5px；Letter 备选 816×1056），直到 P4 解析 `@page { size }`。
- **优先级**：必须
- **来源**：R1995 FR-005 / CSS Paged Media

**验收场景**：

```
场景: 默认页高 = A4 @96dpi（正常路径）
  假设 Print、无 @page 规则
  当 分页
  那么 page_height == 1122.5（A4 297mm @96dpi）；常量可通过常量名引用（PRINT_PAGE_HEIGHT_A4）
  验证: r1999_print_paginate_default_page_height_a4

场景: @page { size } 存在但未解析（守卫/降级路径，P4 前）
  假设 Print、CSS 含 @page { size: Letter }
  当 分页（P4 未实现，@page 未解析）
  那么 仍用 A4 默认（@page 被忽略，降级到默认），文档记录「@page 解析 = P4」
  验证: r1999_print_paginate_at_page_unparsed_falls_back_to_default
```

### FR-006：嵌套强制换页的提升近似（P1a approximation）
- **描述**：当强制换页声明在**非块流根直接子**的深层元素时（如 `<body><section>...<h1 break-before:page>`），P1a 把 break **提升到最近的块流根直接子单元**（含该深层 break 的整单元起于新页），而非精确在该深层元素处断。精确嵌套断 = P2。
- **优先级**：可以（P1a 明确近似，P2 精确化）
- **来源**：本 spec §8.4（把相对定位复杂度边界化）

**验收场景**：

```
场景: 嵌套 break 提升到顶层单元（正常/近似路径）
  假设 Print、body 子 [section(含 <h1 break-before:page>), div]，section 起于 y=0
  当 分页（P1a）
  那么 section 整体被推到新页（section.abs_top == H），而非仅 h1（近似；P2 才精确断在 h1）
  验证: r1999_print_paginate_nested_break_promoted_to_top_unit

场景: 顶层单元自身声明 break（精确，非近似）（正常路径）
  假设 Print、body 直接子 section 自身声明 break-before:page
  当 分页
  那么 section 精确起于新页（与提升近似结果一致，但语义是精确而非近似）
  验证: r1999_print_paginate_top_unit_break_is_exact
```

---

## 4. 非功能需求

### NFR-001：Screen 模式零回归
- **描述**：分页 post-process 仅在 `media_type == Print` 触发；Screen 模式（默认）LayoutResult 与未接入时逐字段相等。
- **测量标准**：`make product-smoke`（welcome / morning / wintertc / legacy，全 Screen）diff 与基线一致 + 单测 `r1999_screen_mode_layout_byte_identical_no_pagination`。
- **优先级**：必须

### NFR-002：kill-switch 可紧急关闭
- **描述**：env `ZW_PRINT_PAGINATE=0` 禁用分页 pass（即便 Print 模式），回退到「Print CSS 级联但不分页」。
- **测量标准**：单测 `r1999_print_paginate_killswitch_disables_pass`。
- **优先级**：必须

### NFR-003：性能——分页 pass 仅 Print 触发
- **描述**：分页 pass 是 O(N) 单次树遍历（N = 块流根直接子数 + 嵌套 break 子树扫描），仅 Print 触发；Screen 路径零开销。
- **测量标准**：Screen 路径无新增 pass（条件分支跳过）；Print 路径 pass 时间可忽略（单次遍历）。
- **优先级**：应该

### NFR-004：不侵入主 layout / multicol
- **描述**：分页作独立 post-process pass，在 `adjust_multicol_layout` / `apply_relative_offsets` / `clamp_percentage_max_height` 之后调用，不修改这些 pass 的逻辑（避 R125/R206/R213 deadlock 史）。
- **测量标准**：code review——分页 pass 仅读 multicol/relative/clamp 的输出、不改其代码；multicol reftest-oracle A/B net≥0。
- **优先级**：必须

---

## 5. 接口需求

### IF-001：LayoutEngine.media_type 字段 + setter（镜像 viewport）
- **类型**：API（Rust struct field + setter）
- **规格**：
  - `LayoutEngine` 新增字段 `media_type: MediaType`（default `MediaType::Screen`），与 `viewport_width/height` 同级（engine.rs）。
  - `LayoutEngine::set_media_type(&mut self, media_type: MediaType)` setter（镜像 `set_viewport`）。
- **错误处理**：无（枚举赋值）。
- **默认动作**：不调用时 default Screen（零行为变更）。
- **交叉引用**：§8.4 接线图。

### IF-002：StyleSystem.media_type() getter
- **类型**：API（Rust getter）
- **规格**：`StyleSystem::media_type(&self) -> MediaType`（暴露现有私有 `media_type` 字段；当前只有 setter `set_media_type`，无 getter——R1998 确认）。
- **错误处理**：无。
- **默认动作**：返回当前值（default Screen）。
- **交叉引用**：§8.4。

### IF-003：env kill-switch ZW_PRINT_PAGINATE
- **类型**：环境变量（进程级）
- **规格**：`ZW_PRINT_PAGINATE`——未设或 `=1` 启用（Print 模式下）；`=0` 禁用。gate = `media_type == Print && env != "0"`。
- **错误处理**：非法值按「未设」处理（启用）。
- **默认动作**：**首切片 default-off**——即 gate 实际为 `media_type == Print && env == "1"`（须显式开启）；A/B 证明后下轮改 default-on（`env != "0"`）。
- **交叉引用**：FR-001 / NFR-002。

---

## 6. 约束与假设

### 6.1 必须约束（Must）
- Screen 模式 LayoutResult 与未接入分页时逐字段相等（NFR-001）。
- 分页 pass 须有 env kill-switch（`ZW_PRINT_PAGINATE=0` 禁用，NFR-002）。
- 分页作独立 post-process，不改 multicol / relative / clamp pass 逻辑（NFR-004）。
- 强制换页检测复用 multicol 已有逻辑（`break_before == BreakValue::Page` + `page_break_before == PageBreakValue::Always`，multicol.rs:697 已含 Page 变体）。

### 6.2 禁止约束（Must Not）
- 不得在 Screen 模式运行分页 pass（条件分支须先判 media_type==Print）。
- 不得修改主 layout（taffy 树构建 / compute_layout_with_measure）或 multicol 逻辑——分页只在其输出上 post-process。
- 不得放宽容差掩盖分页偏差（DC-14 容差锁定）。
- 不得跳过范围内困难 case（DC-14 分母真实性）。

### 6.3 已定决策
- **分片层级 = 单一**（body 直接 in-flow block 子），非多层嵌套（§8.4 论证）。嵌套精确断 = P2。
- **算法 = 镜像 multicol** `assign_children_to_columns_with_breaking`（multicol.rs:1217）→ 新 `assign_children_to_pages`。
- **输出 = tall-framebuffer**（height = 页数 × 页高），不破单视口假设（P5 才动输出模型）。
- **media_type 接线 = 镜像 viewport**（LayoutEngine 字段 + setter，pipeline.set_media_type 同设）。
- **默认页尺寸 = A4 @96dpi 常量**（P4 前 @page 未解析）。

### 6.4 技术约束
- Rust edition 2024 / MSRV 1.85。
- 不引入新 crate 依赖（复用现有 layout-engine / style-system / css-parser）。
- 遵循 CLAUDE.md：单文件 ≤2000 行（分页逻辑独立模块 `print_pagination.rs`）；`cargo fmt` + `clippy -D warnings` + `make test` + `make product-smoke` 全过。

### 6.5 假设
- **A1**：LayoutBox 位置相对父内容区（types/mod.rs:30 注释明示）——**已验证**（R1998 + 本轮 code-archaeology）。
- **A2**：body 的直接 in-flow block 子是文档主内容流的合理分片单元——**待验证**（实施时 probe 实际页面结构；若 body 缺失或结构异常，回退到 root 的直接子）。
- **A3**：multicol `assign_children_to_columns_with_breaking` 可安全复用为纵向版（仅排列方向 + 输出模型差异）——**待验证**（实施时 A/B multicol reftest-oracle net≥0，证复用未破坏 multicol）。
- **A4**：chromium print oracle 的默认页尺寸 = A4（或 reftest runner 须 `--media print` 时设匹配页尺寸）——**待验证**（P4 @page 解析后精确对齐；P1a 用 A4 常量，print oracle delta 仅作趋势）。

### 6.5A 实现来源说明

| 能力/行为 | 来源类型 | 具体来源 | 备注 |
|----------|----------|----------|------|
| 强制换页检测（break_before==Page / page_break_before==Always） | 复用现有模块 | multicol.rs:697（已含 BreakValue::Page）+ style-system ComputedStyle 字段 | 无需新解析 |
| 分片分配算法（children → pages，含 oversized 拆分） | 复用现有模块（纵向版） | multicol.rs:1217 `assign_children_to_columns_with_breaking` → 新 `assign_children_to_pages` | 仅排列方向（纵向堆叠 vs 横向并排）+ 无列宽 |
| media_type 流入 layout | 仓内自实现（镜像 viewport） | LayoutEngine 字段 + setter（engine.rs）+ pipeline.set_media_type（pipeline/mod.rs:293） | 镜像 set_viewport 模式 |
| StyleSystem.media_type 读取 | 仓内自实现 | style-system lib.rs：新 getter `media_type()` | 当前私有无 getter（R1998） |
| 默认页尺寸 | 仓内自实现（常量） | print_pagination.rs：`PRINT_PAGE_HEIGHT_A4 = 1122.5` | P4 前 @page 未解析 |
| tall-framebuffer 高度 | 复用现有能力 | paint_cull_viewport（pipeline/mod.rs:794）已算 doc_h.max(viewport_h) | Print 时 doc_h = 页数×页高 |

### 6.6 代码变更边界
- **允许修改**：
  - `crates/style-system/src/lib.rs`（加 `media_type()` getter）
  - `crates/layout-engine/src/engine.rs`（LayoutEngine 字段 + setter + compute() 末尾调分页 pass）
  - `crates/layout-engine/src/print_pagination.rs`（**新增**模块）
  - `crates/layout-engine/src/lib.rs`（pub mod print_pagination）
  - `crates/engine/src/pipeline.rs`（set_media_type 同设 layout_engine）
  - `tests/integration/src/`（probe / regression test，新文件）
- **禁止修改**：
  - `crates/layout-engine/src/multicol.rs` —— 仅读其算法模式，不改逻辑（A/B 守 multicol net≥0）
  - 主 layout 树构建（`build_layout_tree*` / `compute_layout_with_measure`）
  - Screen 路径任何行为（条件分支保护）

### 6.7 执行技能提示
- 无专用 skill 需求（通用 Rust 执行器即可）。渲染/布局变更须 `make product-smoke`（run-rules.md 强制）。

---

## 7. 优先级与里程碑建议

| ID | 需求 | 优先级 | 理由 | 里程碑 |
|----|------|--------|------|--------|
| FR-001 | Print 触发分页 pass + kill-switch | 必须 | 整个 feature 的 gate | P1a-M1 |
| FR-005 | 默认页尺寸常量 | 必须 | 分页的前提 | P1a-M1 |
| FR-002 | page-break-before 强制换页 | 必须 | 最小可见核心 | P1a-M1 |
| FR-004 | 自然页填充 | 应该 | 复用 multicol breaking，低成本 | P1a-M2 |
| FR-003 | page-break-after | 应该 | 对称 | P1a-M2 |
| FR-006 | 嵌套 break 提升 | 可以 | P1a 近似 | P1a-M2 |

### 建议里程碑
- **P1a-M1（首切片）**：FR-001 + FR-005 + FR-002（强制 before 换页）+ media_type 接线 + kill-switch default-off + probe test。验证：Screen 零回归 + probe 数学正确。
- **P1a-M2**：FR-003（after）+ FR-004（自然填充）+ FR-006（嵌套提升）。验证：`make reftest-oracle --media print` delta + product-smoke。
- **P1a-M3**：A/B 证明后翻 kill-switch default-on。
- **后续**：P2（嵌套精确，绝对坐标 remap）/ P3（inside:avoid）/ P4（@page）/ P1.5（分隔线）/ P5（输出模型）。

### 实施交接（Implementation Handoff）

#### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险/注意事项 |
|----------|------|------|---------------|
| `crates/style-system/src/lib.rs` | 修改 | 加 `media_type()` getter | 仅暴露现有私有字段，零行为变更 |
| `crates/layout-engine/src/engine.rs` | 修改 | LayoutEngine 加 `media_type` 字段 + `set_media_type` + compute() 末尾调分页 | 字段 default Screen；分页 pass 须在 clamp_percentage_max_height（:571）之后 |
| `crates/layout-engine/src/print_pagination.rs` | 新增 | 分页 post-process 逻辑（`paginate_for_print` + `assign_children_to_pages`） | 独立模块 ≤2000 行；镜像 multicol.rs:1217 |
| `crates/layout-engine/src/lib.rs` | 修改 | `pub mod print_pagination;` | 模块声明 |
| `crates/engine/src/pipeline.rs` | 修改 | `set_media_type`（:249）同设 `layout_engine.media_type` | 1 行追加 |
| `tests/integration/src/print_pagination_probe.rs` | 新增 | probe + regression test | 验 FR-001~FR-006 场景 |

#### 职责映射

| 模块/文件 | 职责 | 依赖/被依赖 | 验证方式 |
|----------|------|------------|----------|
| print_pagination.rs | Print 分页 post-process（单层分片 + 分配） | 读 LayoutBox/ComputedStyle；被 engine.rs compute() 调用 | probe test + reftest-oracle --media print |
| engine.rs compute() | 调度分页 pass（gate media_type==Print + env） | 调 print_pagination；被 pipeline 调 | make test + product-smoke |
| pipeline.set_media_type | 把 media_type 传到 layout_engine | 调 layout_engine.set_media_type | webview smoke（R1992 谱系） |

#### 推荐修改顺序

1. **先加 getter + 字段 + setter（零行为变更）**：style-system `media_type()` getter + LayoutEngine `media_type` 字段/setter + pipeline.set_media_type 同设。验证：`cargo build` + 现有 test 全绿（纯接线，default Screen）。
2. **再加分页 pass 骨架 + gate（仍零行为变更）**：`print_pagination.rs` 空骨架 + engine.rs compute() 末尾条件调用（gate media_type==Print + env）。验证：Screen 路径 product-smoke 零回归（pass 未运行）。
3. **最后填分页算法 + probe**：实现 `assign_children_to_pages` + `paginate_for_print`（镜像 multicol）+ probe test 证数学。验证：probe 绿 + Screen 零回归 + print-oracle delta 趋势。

#### 首批提交建议

| 提交/批次 | 范围 | 预期结果 | 验证 |
|----------|------|----------|------|
| Commit 1（本轮） | spec-rfc doc（本文件） | 设计 de-risk，R1998 缺口补齐 | doc 落盘 |
| Commit 2（本轮或下轮） | getter + 字段 + setter + 分页 pass + probe（P1a-M1，default-off） | Print forced-break 推到页边界；Screen 零回归 | make test + product-smoke + probe 绿 |

> **rally 节奏**：本轮（R1999）产 spec-rfc（Commit 1）+ 若时间允许落 P1a-M1（Commit 2）。下轮（R2000）落 P1a-M2 + 翻 default-on。

---

## 8. 技术设计（RFC）

### 8.1 现状分析

**当前架构**（layout → paint 链路）：
1. `LayoutEngine::compute_with_img_intrinsic`（engine.rs:305）构建 taffy 树 → `compute_layout_with_measure` → `extract_layout` 得 `root_box`（LayoutBox 树，根 = html）。
2. 一系列 **post-process pass** 递归改写 `root_box`（顺序，engine.rs compute() 内）：
   - `adjust_multicol_layout`（:443 / :715）—— multicol 列分配
   - `apply_relative_offsets_inline`（:458）—— relpos inset
   - `clamp_percentage_max_height`（:571）—— 百分比 max-height
   - 返回 `LayoutResult { root, viewport_width, viewport_height }`（:619）
3. `RenderPipeline`（pipeline.rs）多处 render 站点调 `layout_engine.compute*` 得 `layout_result` → `painter.paint(&layout_result.root, ...)` → `paint_cull_viewport`（pipeline/mod.rs:794，算 `doc_h.max(viewport_h)`）。

**痛点 / 缺口**：
- **media_type 未流入 layout**：`StyleSystem.media_type`（lib.rs:217，私有，有 setter 无 getter）仅用于 cascade 过滤（lib.rs:482）；`LayoutEngine` 无 media_type 字段、无 Print 条件分支。→ 分页 pass 无法知是否 Print 模式。
- **page-break 计算值未被消费**：`BreakValue`/`PageBreakValue` enum + ComputedStyle 字段已就绪，但 layout 无分页逻辑。
- **单视口假设**：layout/render 假设单 viewport；Print 须 tall-framebuffer（页序列平铺为高 framebuffer）。

**相关代码**（实证，本轮 code-archaeology）：
- `multicol.rs:44 ColumnFragment { child_idx, fragment_y_offset, visual_height }` —— 分片数据模型（可复用）。
- `multicol.rs:1217 assign_children_to_columns_with_breaking(children, col_count, max_col_height, forced_breaks, forced_breaks_after)` —— 分片分配算法（**直接镜像为 `assign_children_to_pages`**）。
- `multicol.rs:697 forced_breaks[i] = matches!(s.break_before, BreakValue::Column | BreakValue::Page)` —— forced-break 检测已含 Page 变体（共享）。
- `types/mod.rs:30 LayoutBox.y` 注释「相对于父元素的内容区域」—— **相对定位**（R1998 核心发现）。

### 8.2 目标状态

- `LayoutEngine` 持 `media_type`（镜像 viewport）；`pipeline.set_media_type` 同设 style_system + layout_engine。
- 新增 `print_pagination.rs`：`paginate_for_print(root, page_height, styles)` post-process，在 clamp 之后、LayoutResult 构造之前调用，gate `media_type==Print && env`。
- Print 模式下，body 直接 in-flow block 子按页高分页（强制 + 自然），文档总高 = 页数 × 页高，tall-framebuffer 输出。
- Screen 模式：pass 不运行，LayoutResult 逐字段不变。

### 8.3 影响范围分析

| 影响项 | 影响程度 | 说明 |
|--------|----------|------|
| Screen 路径（默认） | 无 | 条件分支跳过 pass（NFR-001） |
| Print 路径（Ctrl+P / --media print） | 高 | 新增分页，print-preview 行为改变（feature 目标） |
| multicol reftest | 低 | 分页 pass 在 multicol 之后、只读其输出，不改 multicol（A/B 守 net≥0） |
| product-smoke（Screen） | 无 | 全 Screen，pass 不运行 |
| reftest-oracle（Screen 默认） | 无 | config.media_type 默认 Screen |
| reftest-oracle --media print | 中 | print case delta（~6 case，低 EV） |

### 8.4 详细设计

#### 8.4.1 相对定位分页挑战（R1998 缺口的正式陈述）

LayoutBox 位置**相对父内容区**（`box.y` = 相对父 content-box 原点）。故：
- **绝对 y = Σ 沿祖先链 (ancestor.y + ancestor.content_y + ancestor.margin/border/padding)** —— 非单字段可读。
- **shift 一个元素不自动传播**：若把 box B 的 y 加 Δ（推到下页），其**同父兄弟** C（taffy 已按原 B 位置算好 C.y）不会跟随 → C 视觉上与 B 重叠。
- **跨祖先子树 shift 复杂**：深层 break（`<body><section><h1 break-before:page>`）推 h1 到下页时，section 高度不增（taffy 已定）→ h1 溢出 section 底；section 的兄弟也不跟随。

→ 朴素「遍历加偏移」失效。R1998 据此判「非 clean slice，须 dedicated spec-rfc」。

#### 8.4.2 解决：把分片边界化到单一层级（核心洞察）

**关键洞察**：上述复杂度**只在多层（嵌套）分片时出现**。若把分片限制在**单一层级**（body 的直接 in-flow block 子），则：
- **sibling shift = 分配算法本身**：`assign_children_to_pages` 给每个单元算新 y（基于页高 + forced break + 自然填充），单元间相对顺序由分配决定——不需要单独的「shift 传播」。
- **后代自动跟随**：单元的后代 y 相对该单元内容区；单元被分配到新页（改 unit.y）后，后代**整体平移**，相对结构不变。
- **无跨祖先问题**：不分片嵌套层 → 不存在「深层 break 推祖先溢出」。

```
单层分片示意（body 直接子 = 分片单元）：

  BEFORE (taffy 原始, 相对 body)      AFTER (分页, page_height=H)
  body                                  body
  ├─ A  y=0   h=100                    ├─ A  y=0        (第1页: 0..H)
  ├─ B* y=100 h=50   (*=break-before)  ├─ <页边界空白>   (y=H 处)
  └─ C  y=150 h=30                    ├─ B* y=H        (第2页顶)
                                       └─ C  y=H+50     (B 之后, 同页)

  → B 被 forced break 推到第2页顶 (y=H)；C 作为 B 的同父兄弟, 由分配算出 y=H+50
  → A/B/C 的后代相对各自单元, 自动跟随 (无需单独 shift)
  → 文档总高 = 2*H (tall-framebuffer)
```

**与 multicol 的对应**（证明可复用）：

| 概念 | multicol（横向） | print（纵向，本 spec） |
|------|-----------------|----------------------|
| 分片容器 | multicol 容器（column-count） | body / 块流根（media_type==Print） |
| 分片单元 | 容器的直接 block 子 | body 的直接 in-flow block 子 |
| 排列方向 | 列**横向**并排（column-count 列） | 页**纵向**顺序堆叠 |
| 单元定位 | `child.y = col_cumulative - fragment_y_offset`（multicol.rs:728） | `unit.y = page_idx * H + intra_page_y` |
| forced break | `break_before==Column\|Page`（multicol.rs:697） | `break_before==Page` + `page_break_before==Always` |
| oversized 拆分 | column breaking（fragment_y_offset） | page breaking（同机制，纵向） |
| 输出 | 单 viewport 内列并排 | tall-framebuffer（高 = 页数×H） |

**唯一新增**：纵向堆叠（page_idx×H 偏移）+ tall-framebuffer 高度。分配算法本体（`assign_children_to_columns_with_breaking`）直接复用。

#### 8.4.3 算法（伪代码）

```text
# 入口：paginate_for_print(root, page_height, styles)
# 仅对 body（块流根）的直接 in-flow block 子分片；后代不动。
fn paginate_for_print(root: &mut LayoutBox, page_height: f32, styles: &HashMap) {
    let body = find_block_flow_root(root);        # 下降到 body（或 root 的内容流容器）
    let units = body.children.iter().filter(is_in_flow_block).collect();
    # 1. 算每单元高度 + forced break 标志
    let unit_specs: Vec<(usize, f32, bool, bool)> = units.map(|u| {
        let h = u.height + u.margin_top + u.margin_bottom;
        let fb = has_forced_break_before(u, styles) || subtree_has_forced_break(u, styles);  # FR-006 提升
        let fa = has_forced_break_after(u, styles);
        (u.idx, h, fb, fa)
    });
    let forced_before: Vec<bool> = unit_specs.map(|s| s.2);
    let forced_after:  Vec<bool> = unit_specs.map(|s| s.3);
    let heights: Vec<(usize, f32)> = unit_specs.map(|s| (s.0, s.1));
    # 2. 分配到页（镜像 multicol assign_children_to_columns_with_breaking，纵向）
    let pages = assign_children_to_pages(heights, page_height, forced_before, forced_after);
    #    pages: Vec<Vec<PageFragment { child_idx, fragment_y_offset, visual_height, page_idx }>>
    # 3. 重定位：每个单元的新 y = page_idx * page_height + intra_page_y - fragment_y_offset
    for page in pages {
        let mut intra_y = 0.0;
        for frag in page {
            let u = &mut body.children[frag.child_idx];
            u.y = frag.page_idx * page_height + intra_y - frag.fragment_y_offset;
            intra_y += frag.visual_height;
        }
    }
    # 4. 扩 body 高度 = 页数 * page_height（tall-framebuffer）
    body.content_height = pages.len() * page_height;
    body.height = body.content_height + body.padding_top + body.padding_bottom + ...;
}

fn assign_children_to_pages(children, page_height, forced_before, forced_after)
    -> Vec<Vec<PageFragment>> {
    # 与 multicol assign_children_to_columns_with_breaking 同构：
    #   - col_count = ∞（页数动态增长，按需 append 新页，非固定列数）
    #   - max_col_height = page_height
    #   - forced_breaks / forced_breaks_after 同语义
    #   - oversized 单元（> page_height）→ fragment_y_offset 拆多页（复用 multicol breaking 分支）
    #   - 自然填充：单元放不下当前页剩余 → 移到下页（复用 multicol "可整体放入下列" 分支）
    # 差异仅：横向 col → 纵向 page（page_idx 累加而非 col_idx），无列宽概念。
}
```

**forced-break 检测**（复用 multicol.rs:697）：
```text
fn has_forced_break_before(u, styles) -> bool {
    matches!(s.break_before, BreakValue::Page) || matches!(s.page_break_before, PageBreakValue::Always | Left | Right)
}
```

#### 8.4.4 media_type 接线图

```
RenderPipeline.set_media_type(mt)          [pipeline/mod.rs:293]
  ├─ self.style_system.set_media_type(mt)   [已有]
  └─ self.layout_engine.set_media_type(mt)  [新增，镜像 set_viewport]

LayoutEngine.compute_with_img_intrinsic()   [engine.rs:305]
  └─ ... adjust_multicol / apply_relative / clamp_percentage_max_height ...
  └─ if self.media_type == Print && print_paginate_enabled() {   [新增 gate]
         crate::print_pagination::paginate_for_print(&mut root_box, PRINT_PAGE_HEIGHT_A4, styles);
     }
  └─ LayoutResult { root, viewport_width, viewport_height }   [body 高已扩为页数×H]
```

`print_paginate_enabled()` = `std::env::var("ZW_PRINT_PAGINATE").as_deref() == Ok("1")`（首切片 default-off）。

#### 8.4.5 输出模型（tall-framebuffer）

- 分页后 body.height = 页数 × page_height；`paint_cull_viewport`（pipeline/mod.rs:794）已算 `doc_h.max(viewport_h)` → cull rect 自动覆盖全部页。
- framebuffer 高度：render 站点（pipeline/mod.rs:404/...）当前用 `self.viewport_height`。Print 模式下 doc_h > viewport_h，须确保 framebuffer 高 = doc_h（paint_cull_viewport 已提供 doc_h）。**接线点**：render 站点用 cull rect 的 height 而非 viewport_height 创建 framebuffer（若当前已如此则零改动；否则 P1a-M1 顺带修）。
- 页边界**分隔线**（P1.5，deferred）：当前页边界以**空白间隔**可见（单元被推到 page_idx×H，中间无内容）；分隔线需 LayoutResult 新字段 `page_breaks: Vec<f32>` + paint 步骤，P1.5 独立切片。

### 8.5 安全考虑
- **无信任边界 / 数据保护**风险（纯布局计算，无用户输入 / 网络调用）。
- **潜在风险**：分页 pass 改写 body 高度 → 若 Screen 误触发会破坏产品渲染。缓解：双重 gate（media_type==Print + env）+ Screen 零回归测试（NFR-001）。
- **kill-switch**：`ZW_PRINT_PAGINATE=0` 紧急关闭（NFR-002）。

### 8.6 替代方案

#### 方案对比表

| 维度 | 方案 A：单层分片（P1a，本 spec 推荐） | 方案 B：绝对坐标 remap（P2，嵌套精确） | 方案 C：固定页高 margin（layout 期） |
|------|------|------|------|
| 实现复杂度 | 🟢 低（镜像 multicol） | 🔴 高（绝对 y 重算 + 祖先高度增长） | 🟡 中（taffy 介入） |
| 正确性（强制换页） | 🟢 高（单层精确） | 🟢 高（任意层精确） | 🔴 低（非 boundary 对齐 + 巨大 gap） |
| 嵌套 break | 🟡 近似（提升到顶层） | 🟢 精确 | 🔴 不支持 |
| 侵入主 layout | 🟢 无（post-process） | 🟡 中（树重写） | 🔴 高（改 taffy 输入） |
| deadlock 风险（R125/R206/R213 史） | 🟢 低（独立 pass） | 🟡 中 | 🔴 高 |
| 可回滚 | 🟢 易（kill-switch） | 🟡 中 | 🟡 中 |
| **推荐度** | ⭐⭐⭐（首切片） | ⭐⭐（P2） | ⭐（拒绝） |

**最终选择**：方案 A（P1a 首切片）→ 方案 B（P2 嵌套精确，后续）。方案 C 拒绝（R1998 实证：fixed-margin 不精确非 boundary 对齐 + ~1014px 巨大 gap，UX 差）。

**理由**：
1. 方案 A 把 R1998 的相对定位复杂度**边界化到单一层级**，消解为「一次线性分配」——clean bounded 切片，镜像已验证的 multicol 模式。
2. 方案 B 是方案 A 的自然延展（嵌套精确），但引入「祖先高度增长 + 跨祖先子树 shift」复杂度，须 dedicated 多 session——P2。
3. 方案 C 不 boundary 对齐（页边界处可能有内容残留）+ gap 巨大，UX 不可接受。

### 8.7 实施计划

1. **Step 1（接线，零行为变更）**：style-system `media_type()` getter + LayoutEngine `media_type` 字段/setter + pipeline.set_media_type 同设。验证：`cargo build` + `make test` 全绿（default Screen）。
2. **Step 2（pass 骨架 + gate，零行为变更）**：`print_pagination.rs` 空骨架 + `paginate_for_print`（暂 no-op）+ engine.rs compute() gate 调用。验证：Screen product-smoke 零回归。
3. **Step 3（算法 + probe，P1a-M1）**：实现 `assign_children_to_pages` + `paginate_for_print` + probe test。验证：probe 绿（FR-001/002/005 场景）+ Screen 零回归。
4. **Step 4（P1a-M2，下轮）**：FR-003（after）+ FR-004（自然填充）+ FR-006（嵌套提升）+ `make reftest-oracle --media print` delta。
5. **Step 5（下轮）**：A/B 证明后翻 `ZW_PRINT_PAGINATE` default-on。

### 8.8 测试策略

- **单元测试 / probe**（`tests/integration/src/print_pagination_probe.rs`）：
  - `r1999_print_paginate_forced_break_pushes_to_page_boundary`（FR-002 核心：B.abs_top == H）
  - `r1999_screen_mode_layout_byte_identical_no_pagination`（NFR-001 Screen 零回归）
  - `r1999_print_paginate_killswitch_disables_pass`（NFR-002）
  - `r1999_print_paginate_break_after_pushes_next_sibling`（FR-003）
  - `r1999_print_paginate_natural_fill_overflows_to_next_page`（FR-004）
  - `r1999_print_paginate_oversized_unit_fragments_across_pages`（FR-004 oversized）
  - `r1999_print_paginate_default_page_height_a4`（FR-005）
  - `r1999_print_paginate_nested_break_promoted_to_top_unit`（FR-006）
- **集成测试**：probe 直接构造 LayoutBox 树 + styles，调 `paginate_for_print`，断言单元新 y / body 高度。
- **产品 smoke**（run-rules.md 强制）：`make product-smoke`（welcome / morning / wintertc / legacy，全 Screen）diff 与基线一致。
- **reftest-oracle**：`make reftest-oracle --media print DIR=css/CSS2/page-box`（~6 case）delta 趋势（低 EV，主要验不破坏）。
- **回归守卫**：multicol reftest-oracle A/B net≥0（证复用未破坏 multicol，A3）。

> **脆弱逻辑覆盖**：分片分配依赖「body 直接子 = 分片单元」假设（A2）——probe 须覆盖 body 缺失 / 结构异常回退（find_block_flow_root 回退到 root 直接子）。

### 8.9 回滚计划
- **kill-switch**：`ZW_PRINT_PAGINATE=0`（运行时禁用，无需重编译）。
- **代码回滚**：分页 pass 是独立模块 + engine.rs 单点条件调用，`git revert` 单 commit 即恢复（Step 1-3 各自独立 commit）。
- **Screen 不受影响**：即便分页 pass 有 bug，Screen 路径（默认）零接触（双重 gate）。

---

## 9. Spec Lint 报告

### 结构完整性

| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要存在性 | ✅ Pass | §0 含一句话目标 / 范围 / 排除 / 约束 / 方案 / 首步 |
| 场景存在性 | ✅ Pass | FR-001~FR-006 每个含 ≥2 验收场景（§3） |
| 异常路径覆盖 | ✅ Pass | 每 FR 含守卫/异常场景（如 Screen 零回归 / kill-switch / 末单元 no-op / oversized） |
| 测试绑定 | ✅ Pass | 每场景绑 `r1999_*` 单测名（§8.8） |
| TBD 清零 | ✅ Pass | §10 无阻塞性 TBD（A2/A3/A4 为「待验证」假设，非阻塞） |
| 约束覆盖 | ✅ Pass | §6.1 每条 Must 被 NFR/FR 场景覆盖 |
| 实施交接完备 | ✅ Pass | §7 含文件清单 / 职责映射 / 修改顺序 / 首批提交 |
| 首步可执行性 | ✅ Pass | §8.7 Step 1 明确（getter + 字段 + setter，零行为变更） |

### 语言精确性

| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ⚠️ Warning | FR-004「自动续到下一页」/ FR-006「提升」——已用 §8.4.3 伪代码精确化，保留业务措辞可接受 |
| 无量化描述 | ✅ Pass | 页高量化（A4 1122.5px）、net≥0 量化 |
| 非确定性措辞 | ✅ Pass | 用「必须 / 不得」，FR-006「近似」明示为 P1a approximation |

### 一致性

| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 在/不在范围无交集（嵌套精确 = P2 明确排除） |
| 约束冲突 | ✅ Pass | §6.1/6.2 Must/Must Not 无矛盾 |
| 方案漂移 | ✅ Pass | §8.4 设计（单层分片）与 §1.3 范围 / §6.3 决策一致 |
| 章节引用正确 | ✅ Pass | §8.4 / §7 / §3 交叉引用均落地 |
| 实现来源闭合 | ✅ Pass | §6.5A 每能力注明来源（multicol 复用 / 仓内自实现） |
| 类型分层清晰 | ✅ Pass | 需求(FR)/决策(§6.3)/假设(§6.5)/TBD(§10) 分离 |
| 代码边界完备 | ✅ Pass | §6.6 允许/禁止修改路径列明 |
| 清单数量一致 | ✅ Pass | §7 文件清单 6 项与 §6.6 一致 |

**汇总**：19 Pass / 1 Warning / 0 Fail / 0 Skip
**门禁判定**：Fail = 0 → **允许确认 / 允许实施**

---

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|----|------|--------|----------|--------|
| TBD-1 | body 结构异常时 find_block_flow_root 回退策略（A2） | 重要 | 实测 body 缺失/嵌套场景 | P1a-M1 实施时 probe 确定（回退到 root 直接子） |
| TBD-2 | print oracle 默认页尺寸是否 = A4（A4） | 可选 | chromium print oracle 抓取参数 | P4 @page 解析后精确对齐；P1a 用 A4 常量 |
| TBD-3 | framebuffer 高度接线点是否需改（§8.4.5） | 重要 | 实测 render 站点是否已用 cull rect height | P1a-M1 Step 3 顺带核查 |

> 无阻塞性 TBD（TBD-1/3 实施时 probe 即可定，TBD-2 属 P4）。

---

## 11. 修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-07-24 | R1999 初始 spec-rfc：解决 R1998 相对定位 pagination 模型缺口——单层分片（P1a）把复杂度边界化 + 镜像 multicol 算法 + media_type 接线 + kill-switch + A/B 计划 + 替代方案对比（P2 绝对 remap / P3 inside:avoid / C 拒绝） |

