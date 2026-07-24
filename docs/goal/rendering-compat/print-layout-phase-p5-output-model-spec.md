# Spec / RFC：Print Layout Phase P5 — 分页输出模型（page-sequence output）

**日期**：2026-07-24（R2014，承接 R2013 layout-width-for-print LANDED）
**状态**：scoping / RFC v0.1（**未进入实施**——substantial 架构改动 + multicol deadlock 史，须用户/下一轮确认范围与优先级后再切片；font-stack rebuild 优先级高于本 phase，见 design §7.4）
**前置**：@media print 全弧 + Phase P1a（分页 R1999/R2000）+ P1.5（页边界分隔线 R2001）+ P4（`@page { size; margin }` R2010/R2011 + layout-width-for-print R2013）+ P5-bounded（tall-framebuffer R2005）。本文档定义「真分页输出模型」（CSS §13 page-sequence）的范围与设计，作为 tall-FB 模型的下一阶段。

> **范围诚实（EV）**：reftest 低 EV（仅 ~6 个 `--media print` WPT case，且 oracle `emulateMediaType` 不真分页，弱信号）。本 phase 主要价值 = **产品 print-preview 正确性**（DC-12/13：Ctrl+P 翻页预览 / 实际打印每页一帧）+ 解锁 oversized 真分片 / P2 嵌套精确断 / print-to-PDF 的渲染前置。**不动 headline**（≥95% 仍 font-stack-gated，ruling #2）。

---

## 0. 执行摘要

**目标**：在 `media_type == Print` 时，把渲染输出从「单 tall-framebuffer（连续内容 + 页边界分隔线）」升级为「**page 序列**」——每页一个独立 clipped 渲染帧 + 超高内容跨页**真分片**（fragmentation），使 Ctrl+P 打印预览可翻页查看每页、实际打印每页一帧。

**当前状态（tall-FB 模型，R1999-R2013）**：
- 分页 post-process（`paginate_for_print`）把块流根直接子按页高推页（forced break + 自然填充 + 嵌套提升）。
- 输出 = 单 tall-framebuffer（`ZW_PRINT_TALL_FB=1`，R2005），页边界由分隔线标记（`inject_print_page_dividers`，R2001）。
- **超高元素（outer_h > page_height）整体留原位**（R2000 FR-004 deferred），在 tall-FB 中连续渲染、分隔线横穿——**视觉上近似分片**，但非真分片（背景/边框连续非每页重复，无 per-page clip）。
- **嵌套强制断**（P2）用 FR-006 子树提升近似（整单元换页，非精确祖先拆分）。

**tall-FB 模型的天花板**：R2014 实证复核确认——oversized 真分片 / P2 嵌套精确断 / P3（page-break-inside:avoid，已证与 FR-004 冗余）**全部依赖 content fragmentation**（拆父盒跨页），而 tall-FB 连续模型不支持。故 P5（page-sequence + content fragmentation）是 @media print 真完整化的唯一剩余架构路径。

**关键设计判断**：P5 是**根本改动**（layout/render 管线假设单 viewport → page 序列）。须**独立 post-process**（避 multicol fragmentation deadlock 史 R125/R206/R213/R1052），每切片 kill-switch + A/B 守 net≥0（镜像 P1a 方法论）。

**复杂度**：substantial（多 session）。CSS §13 page model + content fragmentation + 新输出模型 + 与现有 tall-FB 协调。**优先级低于 font-stack rebuild**（design §7.4；font-stack 是 headline unlock，P5 是 print 完整化）。

**建议**：本文档为范围界定 + 设计选项，**不直接进入实施**。先确认范围（§3 in/out）与设计选项（§4 A/B/C 裁决），再按 §6 分阶段切片。

---

## 1. 背景与目标

### 1.1 背景

R1995（print-layout-pagination-design.md v0.1）做了范围界定 + fragmentation 复用评估 + 5-phase 计划，把 P5 列为「分页输出模型——根本改动，须独立 spec-rfc」。R1999-R2013 完成 P1a/P1.5/P4 + P5-bounded（tall-FB），但 P5 本体（page 序列 + content fragmentation）**未启动**。R2014 复核确认 tall-FB 模型已达天花板——剩余 print 特性（oversized 真分片 / P2 嵌套精确断 / print-to-PDF 渲染前置）**全部阻塞于 P5**。**本 spec 即该 dedicated 设计**。

### 1.2 目标

1. **page-sequence 输出**：layout 产出页序列（每页 = 页内容盒区域 + 该页可见内容片段），render 可按页输出独立 clipped 帧。
2. **content fragmentation**：超高元素（outer_h > page_height）跨页**真分片**——每页显示该页区间的内容片段（背景/边框按页重复），非整体留原位。
3. **嵌套精确断（解 P2）**：后代 forced break 在精确后代处断，祖先盒跨页分片（替 FR-006 子树提升近似）。
4. **产品 print-preview/print 正确性**：Ctrl+P 翻页预览每页独立帧 / 实际打印每页一帧。

### 1.3 范围边界

**在范围内（P5）**：
- page-sequence 数据模型（页边界 + 每页内容片段索引）。
- per-page render clip（每页作独立 clipped 帧）。
- content fragmentation（block-level 跨页分片：超高元素 + 嵌套断的祖先拆分）。

**不在范围内（后续 feature / 其他层）**：
- `orphans` / `widows`（页底/页首最少行数——须 line-level fragmentation，后续）。
- `@page` margin boxes（`@top-center {...}` running headers/footers——substantial 独立 feature）。
- 命名页（`page: name`）+ 页面选择器。
- 打印对话框 / 系统打印集成（host 层）。
- 实际打印输出编码（PDF/打印机——host 层；P5 仅产渲染前置）。
- table thead 跨页重复（须 table fragmentation，独立 substantial）。

---

## 2. CSS §13 范围（Fragmentation of boxes，page-sequence 版）

**In scope**：
- **Page box 序列**：N 页，每页固定尺寸（@page size，default A4）+ 页边距（@page margin）。页内容盒 = 页框 − margin。
- **Block fragmentation**：block-level 内容流跨页拆分——自然页填充（R2000 FR-004 已有单层版）+ 超高元素跨页分片（P5 新增）。
- **Break 点**：forced break（R2000 FR-002/003）+ 自然 break（页边界）+ 嵌套 break（P2 精确，P5 新增）。

**Out of scope**：见 §1.3。

---

## 3. 设计需求（FR）

### FR-P5-001：page-sequence 元数据（dormant 首切片）
- **描述**：分页 post-process 产出 page 序列元数据（`Vec<PrintPage>`，每页 = 页索引 + 内容盒 y 区间 `[k*ph+mt, (k+1)*ph-mb]` + 该页可见块流子/片段索引），**不改渲染**（dormant）。为 P5b/P5c 提供数据基础。
- **来源**：CSS §13 / R1995 P5
- **首切片范围**：仅元数据计算（页边界 + 每页子区间映射），env-gated `ZW_PRINT_PAGESEQ`，default-off，零渲染变更。

### FR-P5-002：per-page render clip（可见首切片）
- **描述**：render 按 `PrintPage` 序列，每页产一个 clipped 帧（clip 到页内容盒 y 区间）。tall-FB 模式（R2005）保留为连续预览；page-sequence 模式产每页独立帧（供 print 翻页 / 打印每页一帧）。
- **来源**：CSS §13 page model / DC-12/13 print-preview
- **首切片范围**：clip 逻辑（render 每页区间）+ framebuffer-per-page 输出选项；oversized 仍整体留原位（clip 截断，P5c 才真分片）。

### FR-P5-003：超高元素真分片（content fragmentation，核心难点）
- **描述**：超高元素（outer_h > page_height）跨页分片——每页显示该页 y 区间的内容片段；背景/边框按页重复（clip + per-page paint）。替 R2000「整体留原位」。
- **来源**：CSS §13 / multicol ColumnFragment 模型（`fragment_y_offset`）
- **风险**：**multicol deadlock 史**（R125/R206/R213/R1052）——须独立 post-process pass，非侵入主 layout/multicol。

### FR-P5-004：嵌套精确断（解 P2）
- **描述**：后代 forced break 在精确后代处断；祖先盒跨页分片（祖先 fragment 化），替 FR-006 子树提升近似。
- **来源**：CSS §13 / R1995 P2
- **依赖**：FR-P5-003 content fragmentation（祖先拆分同机制）。

---

## 4. 设计选项（RFC 核心：A/B/C 裁决）

### 4.1 Option A：`Vec<PrintPage>` 输出 + per-page clipped re-render
- **方案**：layout 产 `PrintPage` 序列（每页记录可见子/片段）；render 对每页独立 re-render（clip 到页内容盒）。tall-FB 退化为一特例（单帧 = 全 tall）。
- **优点**：语义清晰（页 = 一等公民）；print 翻页 / 每页一帧天然支持。
- **缺点**：render 改动大（每页 re-render 或 per-page clip pass）；性能（N 页 = N 次 clip render，可优化为单 pass + per-page clip）。

### 4.2 Option B：tall-FB + page 元数据（扩展现模型）
- **方案**：保留 tall-FB 连续渲染，附加 `PrintPage` 元数据（页边界 + 每页区间）；host/print 路径按元数据从 tall-FB 裁出每页。
- **优点**：最小改动（render 不变，仅 host 裁剪）；tall-FB 预览 + 每页输出共存。
- **缺点**：content fragmentation（FR-P5-003）仍须 layout 侧分片（tall-FB 只渲染）；祖先拆分（P2）同。

### 4.3 Option C：fragment-based（multicol ColumnFragment 类比）
- **方案**：仿 multicol `ColumnFragment`（`fragment_y_offset` + 可见区间），把超高/嵌套断元素产多个 page-fragment，每页消费一个片段。
- **优点**：复用 multicol 已验证的分片数据模型。
- **缺点**：**最高 deadlock 风险**（multicol ColumnFragment 自身经 R125/R206/R213/R1052 deadlock 史）；print 与 multicol 分片需求不同（纵向页序 vs 横向列并排）。

### 4.4 裁决建议
- **首切片（FR-P5-001 元数据）**：**Option B**（tall-FB + page 元数据，dormant，零渲染变更，最低风险，镜像 R2005 bounded 首切）。
- **per-page 输出（FR-P5-002）**：**Option B → A 渐进**（先 host 按元数据裁 tall-FB，再视需要 render 侧 per-page clip）。
- **content fragmentation（FR-P5-003/004）**：**Option C 谨慎评估**——须先证 multicol ColumnFragment deadlock 在 print 纵向语境是否复现；若复现，回 Option A（独立 page-fragment 模型，非复用 multicol）。**deferred 到独立 sub-spec-rfc**（本 v0.1 不定，P5c 启动前须 dedicated 设计 + deadlock 复现测试）。

---

## 5. 非功能需求

### NFR-P5-001：Screen 模式零回归
- **描述**：page-sequence / fragmentation 仅 `media_type == Print` 触发；Screen LayoutResult 与未接入时逐字段相等。
- **测量**：`make product-smoke`（全 Screen）diff 与基线一致 + 单测 Screen byte-identical。

### NFR-P5-002：kill-switch 可紧急关闭
- **描述**：env `ZW_PRINT_PAGESEQ=0`（FR-P5-001）/ `ZW_PRINT_PERPAGE=0`（FR-P5-002）等禁用各切片，回退 tall-FB 模型。
- **测量**：单测 killswitch。

### NFR-P5-003：不侵入主 layout / multicol（deadlock 防护）
- **描述**：page-sequence + fragmentation 作**独立 post-process pass**，在现有 pass（multicol/relative/clamp/print_paginate）之后，仅读其输出、不改其代码（镜像 NFR-004）。
- **测量**：code review + multicol reftest-oracle A/B net≥0。

### NFR-P5-004：性能——page-sequence 仅 Print 触发
- **描述**：page-sequence 计算 O(N)（N = 页数 + 块流子数），仅 Print 触发；Screen 零开销。

---

## 6. 分阶段计划（narrow slices + gates，每 slice env-gated 零回归）

| Phase | 范围 | 风险 | 依赖 | gate |
|-------|------|------|------|------|
| **P5a** | FR-P5-001 page-sequence 元数据（dormant） | 低 | R2013 | `ZW_PRINT_PAGESEQ` default-off |
| **P5b** | FR-P5-002 per-page render clip（tall-FB 裁每页） | 中 | P5a | `ZW_PRINT_PERPAGE` default-off |
| **P5c** | FR-P5-003 超高真分片（content fragmentation） | **高（deadlock 史）** | P5a + dedicated sub-spec | `ZW_PRINT_FRAGMENT` default-off |
| **P5d** | FR-P5-004 嵌套精确断（解 P2） | 高 | P5c | 同 P5c |

**首切片（P5a）**：page-sequence 元数据计算——复用 `paginate_for_print` 已算的页边界 + 每页子 y 区间，产 `Vec<PrintPage>`（页索引 + 内容盒 y 区间 + 该页可见子索引），dormant（不改渲染），env-gated default-off。**不涉 fragmentation**（P5c 才动），故无 deadlock 风险。单测：N 页文档 → N 个 `PrintPage` + 区间正确 + Screen 不触发。

---

## 7. 风险

| 风险 | 概率 | 缓解 |
|------|------|------|
| content fragmentation deadlock（multicol 史 R125/R206/R213/R1052） | **高**（P5c） | print 分页作**独立 post-process pass**，非复用 multicol ColumnFragment（Option A 独立模型优先）；P5c 启动前 dedicated sub-spec-rfc + deadlock 复现测试；每切片 kill-switch + A/B |
| layout/render 单 viewport 假设破坏 | 高 | P5a/P5b 用 Option B（tall-FB + 元数据，不破单视口）；P5c 才动 fragmentation，须独立设计 |
| reftest 低 EV（~6 print case，oracle 不真分页） | 中 | 主要价值 = 产品 print-preview/print 正确性（DC-12/13），非 reftest pass-rate；真 print A/B 须 printToPDF infra（WSL2 chromium flaky） |
| 优先级低于 font-stack（design §7.4） | 中 | font-stack 授权前 P5 为低 EV 自主推进；授权后 font-stack 优先 |

---

## 8. 何时止步（kill conditions）

- P5a 单测 + A/B 证明 page-sequence 元数据正确且 Screen 零回归 → 继续 P5b。
- P5b per-page clip 证可见改善（print-preview 每页独立帧）→ 继续 P5c；否则停在 tall-FB 模型（R2005）。
- P5c fragmentation 若 deadlock 复现（multicol 史）且独立模型亦不可行 → **P5 止于 P5b**，content fragmentation 标 structural-deadlock（同 R177 COLUMN border-conflict），tall-FB 模型为 print 最终形态。
- 任意切片若 net<0（Screen 回归 / print-preview 退化）→ revert + 评估。

---

## 9. 交叉引用

- 范围界定：[`print-layout-pagination-design.md`](./print-layout-pagination-design.md) §3.3 / §5 P5 / §7.4 优先级。
- P1a 实现：[`print-layout-phase-p1-spec.md`](./print-layout-phase-p1-spec.md) + `crates/layout-engine/src/print_pagination.rs`。
- tall-FB：R2005（`reftest.rs` `ZW_PRINT_TALL_FB`）。
- 页边界分隔线：R2001（`pipeline.rs` `inject_print_page_dividers`）。
- @page size/margin/width：R2010/R2011/R2013。
- fragmentation deadlock 史：R125 / R206 / R213 / R1052（multicol ColumnFragment）+ R177（border-conflict COLUMN structural deadlock）。
- font-stack 优先级：[`unified-font-stack-design.md`](./unified-font-stack-design.md) §7.4 / ruling #2。
