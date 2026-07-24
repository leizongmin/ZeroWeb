# Spec / RFC：Box-level Bidi（IFC 全行 bidi 重排序）

**日期**：2026-07-24（R2021，承接 R2020 text-level bidi correctness fix + box-level bidi 识别）；**R2023 更新**：BL2 Option A 经实证确认 dead-end（见 §8）
**状态**：scoping / RFC v0.1（**未进入实施**——substantial IFC 重构，须确认范围 + 第一切片 A/B 后再推进；非 font-stack 依赖，可作 font-stack-parallel 非-font 推进面）。**R2023 更新**：BL2 Option A 已实证为 dead-end（bidi-007a framebuffer 字节一致 off/on + bidi-text dir 零 flip），code 已 revert——**真路径 = BL3 Option B item-level**（见 §4.2/§8）。
**前置**：R2020 text-level bidi correctness（`bidi_reorder` per-run，控制码 + paragraph level 修），`css/CSS2/bidi-text` 31/105 (30%)。本文档定义 box-level bidi（全行/跨 run bidi）范围与设计。

> **范围诚实（EV）**：bidi-text 105 案，box-level bidi 潜在 +40-74 案（30%→~60-100%，含 display:table/inline-block bidi 隔离语义则更高），= headline +0.4-0.7%。**非 font-stack-gated**（bidi 是文本/盒序，非字形渲染）——独立非-font lever。reftest EV 中等（小 dir），但价值 = bidi dir 真解锁 + DC/Writing-Modes 合规 + RTL 产品页正确性。

---

## 0. 执行摘要

**目标**：把 bidi 算法（UAX #9）从**per-TextRun**（`bidi_reorder(&run.text)`，inline/mod.rs:892）升级为**全 IFC 行**——对所有 inline item（text runs + inline boxes）在逻辑序上运行 bidi，按 bidi level 重排序 item，使 bidi 控制码/`direction`/`unicode-bidi` 属性的**跨 run/盒**效果生效。

**当前状态（per-run text-level bidi，R2020）**：
- `bidi_reorder`（text_metrics.rs:444，`unicode_bidi` crate）per-TextRun 调用，处理 run **内**控制码 + paragraph level（R2020 修）。
- **跨 run bidi 失效**：bidi 控制码（U+202A-202E / U+2066-2069）/ `direction:rtl` / `unicode-bidi` 属性的**跨 run/盒**效果不传递（per-run fresh state）→ bidi-007/008/009/010（控制码跨 span）视觉序错。
- R2020 A/B bidi oracle 逐案字节一致（0 yield）证 per-run 不触及跨 run 用例。

**关键设计判断**：box-level bidi 触 IFC **layout 核心**（item 收集 + 行布局序）——非 per-run 单点改。须**独立 post-process pass**（避类似 vertical-mode 4-layer 耦合 deadlock，R1043/R1052），每切片 kill-switch + A/B 守 net≥0。

**复杂度**：substantial（多 session）。UAX #9 全行 + CSS Writing Modes §2.4 bidi-box-model（inline-box embedding/isolation level）+ display:table/inline-block bidi 隔离语义。**非 font-stack 依赖**（font-stack 授权前可独立推进）。

**建议**：本文档为范围界定 + 设计选项，**不直接进入实施**。先确认范围（§3）+ 设计选项（§4），再按 §6 分阶段切片。

---

## 1. 背景

R2020 纠 master.md「bidi shaping/font-stack-gated」误判：`bidi_reorder`（text-level，unicode_bidi）已实现 + active。text-level 真 bug 修 LANDED（控制码 + paragraph level，单测验 RLO 反转，零回归，0 reftest yield）。**真阻塞 = box-level bidi**（跨 run/盒 bidi）——R2020 识别为独立非-font lever。**本 spec 即该 dedicated 设计**。

### 1.1 bidi-text 用例分类（R2020 实测）

| 用例簇 | 结构 | 需 box-level? |
|--------|------|---------------|
| bidi-007/008/009/010（高 diff 11-72%） | `<p>` + bidi 控制码跨 `<span>` 边界 + `display:table`/`inline-block` | **是**（跨 run 控制码 + 盒隔离） |
| bidi-002/003/004（中 diff 5-13%） | 混合 LTR/RTL 文本 + 控制码 | 部分（跨 run 控制码） |
| bidi-005/006 + bidi-breaking/line-breaking | 简单 bidi + 换行 | 部分（per-run text-level 或已工作） |

---

## 2. CSS Writing Modes §2.4 + UAX #9 范围

**In scope（本设计覆盖）**：
- **全行 bidi**：对 IFC 一行的所有 inline item（text + inline boxes）按逻辑序运行 UAX #9，分配 bidi embedding level。
- **inline-box bidi level**：`direction`/`unicode-bidi`（normal/embed/isolate/override）属性为 inline-box 分配 bidi level（CSS Writing Modes §2.4）。
- **item 重排序**：按 bidi level 重排 inline item（视觉序），layout 按视觉序。
- **控制码跨 run**：U+202A-202E/2066-2069 跨 run 效果（全行 bidi 状态连续）。

**Out of scope（后续 / 依赖）**：
- **display:table bidi 隔离**：table 形成 bidi isolate（bidi-008 用 `display:table`），须 table-layout bidi 集成——substantial 独立切片（P2）。
- **垂直 writing-mode bidi**：vertical-rl/lr 的 bidi（与 R1043 vertical deadlock 耦合，deferred）。
- **Arabic/Hebrew shaping**：字形 shaping（contextual forms/ligatures）须 font-stack（fontdue 无 shaping）——bidi 仅排字序，不 shaping；纯 Latin + 控制码用例（bidi-007/008）不依赖 shaping。

---

## 3. 设计需求（FR）

### FR-BL-001：全行 bidi level 分配（首切片基础）
- **描述**：IFC 一行收集所有 inline item（逻辑序）后，对其文本内容（含 inline-box 边界标记）运行 UAX #9，分配 per-char + per-box bidi embedding level。**dormant 首切片**（仅算 level，不改 layout 序），env-gated。
- **来源**：UAX #9 / CSS Writing Modes §2.4

### FR-BL-002：item 重排序（核心）
- **描述**：按 bidi level 重排 inline item（text runs + inline boxes）为视觉序，layout（inline/mod.rs:891+）按视觉序处理。镜像 chromium 行内 bidi 盒序。
- **风险**：触 IFC layout 核心（line-break + 盒定位）——须独立 post-process，避 vertical-mode 4-layer deadlock。

### FR-BL-003：`direction`/`unicode-bidi` 属性消费（CSS Writing Modes §2.4）
- **描述**：`direction:rtl`/`unicode-bidi:embed|isolate|override` 为 inline-box 分配 bidi level（非仅控制码驱动）。
- **依赖**：style-system 已解析 direction/unicode-bidi（R2020 实测 direction 消费于 inline_finalization.rs 文本对齐）。

### FR-BL-004：display:table/inline-block bidi 隔离（P2，deferred）
- **描述**：table/inline-block 形成 bidi isolate（独立 paragraph）。bidi-008/009 用 display:table。
- **依赖**：FR-BL-002 + table-layout bidi 集成。独立 substantial 切片。

---

## 4. 设计选项（RFC 核心）

### 4.1 Option A：全行文本 bidi（忽略盒边界，简化首切）
- **方案**：concatenate IFC 一行所有 text run 文本（逻辑序，inline-box 作中性边界），UAX #9 全行算 level + 重排字符，map 回 run 切分。
- **优点**：最小改动（不改 item 序，仅全行文本序）；快速验「跨 run 控制码」用例（bidi-007）。
- **缺点**：inline-box **不重排**（盒序仍逻辑）→ display:table/inline-block 用例（bidi-008）不 fully 对；map 回 run 复杂（视觉字符跨 run）。

### 4.2 Option B：item-level bidi 重排序（完整 box-level，CSS Writing Modes §2.4）
- **方案**：每个 inline-item（text run + inline-box）分配 bidi level（含 `direction`/`unicode-bidi`），UAX #9 算 item level，重排 item 为视觉序，layout 按视觉序。
- **优点**：完整 box-level（盒 + 文本都重排），匹配 chromium；覆盖 display:table/inline-block（须 FR-BL-004）。
- **缺点**：触 IFC layout 核心（item 序 + line-break + 盒定位）——substantial + 耦合风险（line-break 须在视觉序做）。

### 4.3 裁决建议
- **首切片（FR-BL-001 + Option A 简化版）**：全行文本 bidi（concatenate + UAX #9 + 重排字符），env-gated default-off，A/B bidi-007（控制码跨 span，无 display:table）。若 bidi-007 改善 → Option A 路径可行 → 续 Option B。若 0/回归 → Option A 不足（须盒重排），跳 Option B。
- **完整（FR-BL-002 + Option B）**：item-level 重排序，须 dedicated sub-spec + line-break-视觉序 重做（避 vertical-mode deadlock）。

---

## 5. 非功能需求

### NFR-BL-001：无 bidi 文档零回归
- **描述**：纯 LTR 文档（无 bidi 字符/控制码/`direction:rtl`）IFC layout 与未接入时逐字段相等。`needs_bidi` 全行 fast-path（无 bidi 原序）。
- **测量**：`make product-smoke`（welcome/morning/wintertc 全 LTR）diff 一致 + 单测纯 LTR byte-identical。

### NFR-BL-002：kill-switch
- **描述**：env `ZW_BOX_BIDI=0` 禁用 box-level bidi（回退 per-run text-level R2020）。

### NFR-BL-003：性能——bidi pass 仅 bidi 文档触发
- **描述**：全行 bidi 仅 `needs_bidi`（含 RTL 脚本/控制码/`direction:rtl`）触发；LTR 文档零开销（fast-path 原序）。

### NFR-BL-004：不侵入 line-break 核心（deadlock 防护）
- **描述**：box-level bidi 作 layout 前 item 重排（视觉序），line-break 在重排后 item 序上做（非改 line-break 算法）。避 vertical-mode「单层 net-neg」deadlock（R1052）——每切片 A/B 守 net≥0。

---

## 6. 分阶段计划（narrow slices + gates）

| Phase | 范围 | 风险 | gate |
|-------|------|------|------|
| **BL1** | FR-BL-001 全行 bidi level 分配（dormant，仅算 level 不改 layout） | 低 | `ZW_BOX_BIDI_LVL` default-off |
| **BL2** | Option A 全行文本 bidi（concatenate + UAX #9 + 字符重排 + map 回 run），A/B bidi-007 | 中 | `ZW_BOX_BIDI` default-off |
| **BL3** | Option B item-level 重排序（盒 + 文本），A/B bidi-007/008 | **高（触 IFC 核心）** | `ZW_BOX_BIDI_ITEM` default-off |
| **BL4** | FR-BL-003 `direction`/`unicode-bidi` 属性驱动盒 level | 中 | 同 BL3 |
| **BL5** | FR-BL-004 display:table/inline-block bidi 隔离（须 table 集成） | 高 | 独立 sub-spec |

**首切片（BL1+BL2）**：全行文本 bidi（Option A），env-gated default-off，A/B bidi-007（控制码跨 span，无 display:table）。若 bidi-007 改善 → 路径可行续 BL3；若 0/回归 → Option A 不足须 BL3。

---

## 7. 风险

| 风险 | 概率 | 缓解 |
|------|------|------|
| IFC layout 核心耦合（vertical-mode 4-layer deadlock 类） | **高（BL3）** | bidi 作 layout 前 item 重排（非改 line-break）；BL2 Option A 先验简化路径；每切片 kill-switch + A/B net≥0；net-neg 立即 revert |
| map 回 run 复杂（视觉字符跨 run） | 中（BL2） | Option A 须 robust map；若 map 太复杂跳 BL3 item-level |
| display:table bidi 隔离须 table 集成（BL5） | 中 | BL5 独立 sub-spec，不阻塞 BL1-BL4 |
| 纯 LTR 回归 | 低 | NFR-BL-001 fast-path + product-smoke 守 |

---

## 8. 何时止步（kill conditions）

- **BL2（Option A 全行文本 bidi）= R2023 实证 DEAD-END（已满足本 kill condition）**：A/B bidi-007a ZW framebuffer **字节一致** off/on（`ZW_DUMP_FB_TAG` PNG cmp IDENTICAL，6 位精确 z_vs_chr 11.507917% 两路径一致），bidi-text dir oracle-pass 31/105 两路径逐案相同（零 flip）。painter 收到重排后 fragment 文本（run2 "h\u{202d}g\u{202c}f"→"fgh"）但像素不变——根因 = bidi-007 失败是**结构性/box-level**（图分：字符垂直堆叠、inline-box 定位错乱），text-level 重排触不到。map-back 本身正确（单测验 cross-run 控制码重排）。**code 已 revert**（0-yield 实验非永久 feature）。→ **跳 BL3 Option B item-level**。
- BL3（item-level）若触 line-break 耦合 net-neg（vertical-mode deadlock 类）且独立 post-process 不可行 → **box-level bidi 止于 BL3 评估**，bidi dir 跨 run 用例标 structural（同 R109 entanglement），per-run text-level（R2020）为 bidi 最终形态。
- 任意切片 net<0 → revert + 评估。

---

## 9. 交叉引用

- R2020 text-level bidi correctness（`bidi_reorder` 控制码 + paragraph level）+ box-level 识别。
- CSS Writing Modes §2.4 bidi-box-model / UAX #9。
- vertical-mode 4-layer deadlock 先例（R1043/R1050/R1052）——box-level bidi 须避同类耦合。
- `bidi_reorder`（text_metrics.rs:444，per-run）/ IFC layout（inline/mod.rs:615/891）。
- font-stack rebuild（headline ≥95% 主 unlock，ruling #2）——box-level bidi 是 font-stack-parallel 非-font 推进面。
