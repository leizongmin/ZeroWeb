# Print Layout 分页设计（Spec + RFC 范围界定）

**日期**：2026-07-24（R1995，承接 @media print 全弧完成 R1981-R1994）
**状态**：范围界定 / 设计草案 v0.1（未进入实施——substantial feature，须 spec-rfc 确认范围后再切片）
**前置**：@media print 全弧已完成——cascade（R1981 `StyleSystem.media_type`）+ measurement tooling（R1991 reftest `--media` + capture `--media`）+ embedding API（R1992 `WebView::set_media_type`）+ browser UI（R1993 Ctrl+P toggle）+ UX 徽标（R1994）。本文档定义「真 print-layout」（CSS §13 分页）的范围与设计，作为 @media print 的下一阶段。

---

## 0. 执行摘要

**目标**：在 `media_type == Print` 时把页面内容按打印页（page box）分页渲染，支持 `page-break-before/after/inside`（及 `break-before/after/inside: page`）强制分页 + 自然页填充，使 Ctrl+P 打印预览显示**分页**内容（而非当前的单视口整页）。

**当前缺口**：`page-break-*` / `break-*` **解析 + 计算值已就绪**（`PageBreakValue` / `BreakValue` enum + `parse_page_break` / `parse_break_before` + `ComputedStyle.page_break_before/after/inside` / `break_before` + 继承），但**布局未消费**——分页应用（按页高把内容拆成多页）完全缺失。

**关键设计判断**：ZW 现有 **multicol fragmentation**（`ColumnFragment` + `column_height_limit` + column breaking）是一套**可复用的分片机制**（把内容按固定高度拆成多区域）。Print 分页可**复用该分片逻辑**，但有三处关键差异须新设计（见 §3）。

**复杂度**：substantial（多 session）。CSS §13 page model + 与现有 fragmentation 协调 + 新输出模型（分页序列 vs 单视口）。有 multicol deadlock 史（R125/R206/R213/R1052）须谨慎。

**建议**：本文档为范围界定，**不直接进入实施**。先确认范围（§4 in/out），再按 §5 分阶段切片（每切片 kill-switch + A/B 守 net≥0 + product-smoke）。

---

## 1. 背景：当前 print 链路

| 层 | 状态 | 说明 |
|----|------|------|
| `@media print` 级联 | ✅ R1981 | `StyleSystem.media_type` + cascade 过滤 |
| reftest `--media` | ✅ R1991 | 量 yield（6 case，低 EV） |
| `WebView::set_media_type` | ✅ R1992 | 嵌入 API |
| browser Ctrl+P | ✅ R1993 | 7-layer 双后端 toggle |
| UX 徽标 | ✅ R1994 | 持久「Print Preview」徽标 |
| **page-break 解析** | ✅ 已就绪 | `PageBreakValue`/`BreakValue` + computed fields + 继承 |
| **page-break 布局应用** | ❌ **缺口** | 无分页——Print 模式仍单视口整页渲染 |
| **@page 规则** | ❌ 未解析 | `@page { size; margin }` 未解析 |
| **分页输出模型** | ❌ 缺失 | 布局产出单 viewport，无 page 序列 |

**结论**：Print 模式当前 = 「Screen 页面套 @media print CSS」（隐藏 screen-only / 显 print-only 内容），但**不分页**。真 print-layout 须补：分页应用 + @page 规则 + 分页输出。

---

## 2. CSS §13 范围（Fragmentation of boxes）

**In scope（本设计覆盖）**：
- **Page box 模型**：每页固定尺寸（默认 A4 = 8.16×10.56in @96dpi ≈ 783×1014px，或 Letter 8.5×11in ≈ 816×1056px）+ 页边距（默认由 UA 定）。
- **强制分页**：`page-break-before/after: always`（CSS2）= `break-before/after: page`（CSS3）→ 强制换页。
- **避免分页**：`page-break-inside: avoid` → 尽量不把元素拆到两页。
- **自然页填充**：内容流超过页高时自动换页（block-level fragmentation）。

**Out of scope（后续 feature）**：
- `@page { size; margin; @top-center {...} }` 命名页 + 页边距盒（running headers/footers）—— substantial，独立 feature。
- `orphans` / `widows`（页底/页首最少行数）—— 精细化，后续。
- 打印对话框 / 系统打印集成（host 层）—— 非 rendering。
- 实际打印输出（PDF/打印机）—— host 层。

---

## 3. 现有 fragmentation 复用评估

### 3.1 multicol fragmentation 现状
- `ColumnFragment { ...; fragment_y_offset: f32 }`（multicol.rs:44）：超高子元素拆成多列片段，每列只显示 `fragment_y_offset..fragment_y_offset+max_col_height`。
- `column_height_limit`（multicol.rs:94）：列高限制（column-fill:auto 用容器高，balance 无限制）。
- column breaking：子元素超高 → 创建多个 `ColumnFragment` 跨列分配。

### 3.2 可复用部分
- **分片数据模型**：`ColumnFragment` 的「子元素 + y_offset + 可见区间」模型直接映射 page fragment（page = 固定高度的列）。
- **break 判定**：`column_height_limit` 的「容器高作上限」逻辑 → page 用 page-height 作上限。
- **forced break 检测**：multicol 已有 forced break 处理（R1820 forced-break overflow column）。

### 3.3 关键差异（须新设计）
1. **排列方向**：multicol 列**横向并排**（column-count 列左→右）；print 页**纵向顺序**（page 1, 2, ... 上下）。→ page 分片须纵向累加，非横向。
2. **输出模型**：multicol 仍渲染在**单 viewport**（列并排在视口内）；print 须产出**页序列**（每页独立 framebuffer，或一个高 = N×page-height 的 tall framebuffer 带页边界）。这是**根本差异**——当前 layout/render 管线假设单 viewport。
3. **触发条件**：multicol 由 `column-count/width` 触发；print 由 `media_type==Print` 触发（整页都分页）。

### 3.4 结论
分片**逻辑**可复用（`ColumnFragment` 模型 + break 判定），但**输出模型**（页序列）是新工程——layout 须支持「paginated mode」产出多页，render 须能把多页绘制成可预览/打印的形式。

---

## 4. 设计需求（FR）

- **FR-001**：`media_type==Print` 时，layout 按 page-height 把 block 流内容分页（自然页填充）。
- **FR-002**：`page-break-before: always` / `break-before: page` → 元素起始于新页。
- **FR-003**：`page-break-after: always` / `break-after: page` → 元素后强制换页。
- **FR-004**：`page-break-inside: avoid` → 尽量不拆元素（若整元素高于单页则仍拆）。
- **FR-005**：page box 默认尺寸（A4 或 Letter）+ 可被 `@page { size }` 覆盖（@page 解析为后续切片，首期用默认尺寸）。
- **FR-006**：打印预览能显示分页（页边界视觉分隔 / 多页滚动）。
- **FR-007**：零回归——`media_type==Screen`（默认）行为字节不变（分页仅 Print 触发）。

---

## 5. 分阶段计划（每切片 kill-switch + A/B）

### Phase P1：page-break-before 强制换页（最小可见切片）
- 范围：Print 模式下 `page-break-before: always` → 该元素及其后内容渲染到新「页」（垂直偏移 page-height + 页边界分隔线）。
- 实现：在 print 布局后处理中，对有 forced break 的元素按 page-height 偏移 + 插入页分隔（tall framebuffer 模型，单视口内用分隔线表示页边界）。
- 验证：A/B `make reftest-oracle DIR=css/CSS2/page-box --media print`（at-page-rule-001 等少数 case）+ product-smoke（默认 Screen 零回归）。
- 不含：自然页填充（仅 forced break）。

### Phase P2：自然页填充（block fragmentation）
- 范围：内容超过 page-height 自动换页（复用 multicol column breaking 逻辑，纵向版）。
- 风险：与 multicol fragmentation 协调（避免 deadlock 史 R125/R206/R213）—— 须独立 post-process pass，非侵入主 layout。

### Phase P3：page-break-inside: avoid
- 范围：avoid 元素尽量不拆（整元素移到下页若当前页放不下）。

### Phase P4：@page 规则解析（size/margin）
- 范围：解析 `@page { size: A4; margin: 2cm }`，覆盖默认页尺寸/边距。

### Phase P5：分页输出模型（页序列 / 多页预览）
- 范围：layout 产出 page 序列，render 支持多页（打印预览翻页 / 实际打印每页一帧）。
- 这是**根本**改动（layout/render 管线假设单 viewport）——须独立 spec-rfc。

---

## 6. 风险

| 风险 | 概率 | 缓解 |
|------|------|------|
| fragmentation deadlock（multicol 史 R125/R206/R213） | 高 | print 分页作**独立 post-process pass**，非侵入主 layout/multicol；每切片 kill-switch + A/B |
| layout/render 单 viewport 假设破坏 | 高 | Phase P1-P4 用 tall-framebuffer + 分隔线（不破单视口）；Phase P5 才动输出模型，须独立 spec-rfc |
| reftest 低 EV（仅 6 @media print case） | 中 | 主要价值是**产品 print-preview 正确性**（DC-13 / DC-12），非 reftest pass-rate |
| @page 规则解析复杂（CSS Paged Media） | 中 | Phase P4 独立切片，首期 P1-P3 用默认页尺寸 |

---

## 7. 建议

1. **不直接进入实施**——本文档为范围界定，须用户/下一轮确认范围（§2 in/out）后再切片。
2. **首切片 = Phase P1**（page-break-before forced break，tall-framebuffer + 分隔线）——最小可见、不破单视口假设、可 A/B。
3. **EV 诚实**：reftest 低 EV（6 case），主要价值是产品 print-preview 正确性 + 真 print-layout 能力。headline reftest ≥95% 仍 font-stack-gated，**本 feature 不动 headline**。
4. **forward 优先级**：font-stack 授权（headline unlock）> print-layout。print-layout 是 @media print 的「完整化」但非 headline 杠杆。

---

## 8. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1 | 2026-07-24 | R1995 初始范围界定（fragmentation 复用评估 + 5-phase 计划 + 风险） |
