# 设计草图：multicol 列感知 IFC 碎片化（column-aware fragmentation）

**版本**：v0.4（**R310 §3/§5 重定向**：Round 1-2 balance 工具方向废弃——R200 证列分配本就正确；最大 near-pass 聚类改为 Round 1' baseline-export probe，根因疑为 flex×multicol first-baseline wiring；Round 2' = breaking wiring[R201]）
**日期**：2026-06-19
**状态**：分析完成；**列分配（balance）方向已证伪关闭**（R200）；**multicol-breaking 真实机制 dump 实测定性**（R201）；**near-pass 聚类重定向 baseline-export**（R307/R310）
**关联**：rendering-compat master.md R113/R122/R128/R131/R157/R199/R200/R201；css-multicol 17/57 失败

---

## ⚠️ R201 dump 实测定性（2026-06-17）— multicol-breaking 真实阻塞点

承接 R200（balance 方向关闭）。本轮对 multicol-breaking-004/005/006/nobackground-004（css-multicol 唯一剩余的「嵌套列碎片化」失败聚类）做 **REFTEST_DUMP + REFTEST_BBOX + 逐行像素扫描** 实测，**纠正 R113/R132 的「内层 multicol 高度依赖外层列宽→两趟循环依赖」假设**——真实阻塞点更具体、更靠 paint 侧，且**碎片化算法本身已存在**。

### 实测证据（multicol-breaking-004，5.60%）

- 结构：`.outer`(height:125px, column-count:4, column-fill:auto, column-rule:4px blue) > `.inner`(column-count:2, height:300px, border-bottom:25px green, box-decoration-break:clone) 内含 17 行文本。
- **REF**（期望）：3 列可见内容，每列含 2 个子列（col0=AAAAA-EEEEE+FFFFF-JJJJJ，col1=KKKKK-NNNNN+OOOOO-QQQQQ，col2=空+绿 border），列间蓝色 column-rule。
- **ZeroWeb 实测**：inner 文本**仅在 col0 渲染**（x≈8-55，单子列），col1/col2 **完全无文本**（仅洋红背景）；**蓝色 column-rule 全部漏画**；绿 border 位置错误（col2 y≈60-80 而非 3 列 y=100-125）。
- BBox：x=[8,603] y=[8,132]（diff 止于 col2 末尾，col3 空=匹配）。

### 3 个真实阻塞点（非 R113「循环依赖」）

| # | 阻塞点 | 位置 | 性质 |
|---|--------|------|------|
| **A** | **paint multicol 门控 `height_auto`**：inner 有明确高度（300px）→ `height_auto=false` → `compute_multicol_info_for_paint` 返回 None → inner 的 2 子列布局**从未计算** | `painter/text.rs:560-569` | wiring 缺失 |
| **B** | **`column_span_offsets` paint 路径不渲染碎片化 inline 内容**：outer 的 column breaking 把 inner 碎片化到 col0/1/2（写 column_span_offsets），但该 paint 路径**不重绘 inner 的 IFC 文本到非主位置列** → inner 文本只在 col0 | `painter/mod.rs`（column_span_offsets 消费）+ R131 同源 | wiring 缺失（核心） |
| **C** | **column-rule §5.2 内容检测只查 `child.x` 主位置**：被碎片化的唯一子元素只在 col0 有 c.x，其余列仅存于 column_span_offsets → 误判「无内容」漏画 rule | `text_multicol.rs:130` | 已实测**不可单点安全修**（见下） |

### 关键纠正：碎片化算法**已存在**，缺口是 wiring

`multicol.rs` 已实现 **`assign_children_to_columns_sequential`（顺序填充+列高预算）和 `assign_children_to_columns_with_breaking`（带 column breaking 的块级子元素碎片化）**——这正是 CSS Multicol §6 fragmentation 所需的顺序填充算法。R113 设想的「内层 multicol 两趟测量」**算法层面已具备**，只是**未接线到 inline 内容的 paint 路径**。

**因此：勿再建「行内流行盒碎片化」measure-first 工具**——它会是 `assign_children_to_columns_sequential` 的重复实现（同 R199 balance 工具被 R200 证伪移除的命运）。真实工作 = 让 paint 侧把 inner 的 IFC 内容按 outer 列高预算碎片化重绘到各列（接 B），并放宽门控（接 A）。

### column-rule 修复（C）实测回归，已回退

本轮实现 C 的修复（`in_range` 闭包额外查 `column_span_offsets` 的列 x），实测：
- multicol-breaking-004 5.60→**5.39%**、006 1.20→**1.12%**（蓝色 rule 正确补画，改善）。
- **但 `column-rule-002` 0.00→1.25%（PASS→FAIL，回归）**。
- 根因：column-rule-002（`columns:3; column-fill:auto` + 单个 height:250px 子元素被碎片化到 3 列）的 REF **恰好匹配旧的 c.x-主位置 检测行为**；旧 §5.2 启发式虽对嵌套 multicol-breaking 不完美，但对 column-rule-002 这类用例**正确**。
- **结论：C 非安全单点修复**（不可简单加 column_span_offsets 感知）。已回退，git diff clean，baseline 438/490 恢复。未来若修 C 须区分 column_span_offsets 的来源（column breaking vs column-span:all）并精确实现 §5.2 语义。

### 对实施计划的影响（Round 4 重新定向）

原 Round 4（column breaking / 两趟循环依赖）的**算法前提不成立**——碎片化算法已存在。真实 Round 4 应改为：

- **Round 4'（wiring）**：让 `column_span_offsets` paint 路径对被碎片化的 IFC 容器子元素，按列高预算（来自 outer column breaking 的 fragment 切片）重绘其 inline 内容到每个非主位置列 + 列裁剪。这是 R131「paint IFC 与 multicol block 分配协调」的具体落地，**paint 侧多轮子系统**（非 layout 两趟）。
- 前置：放宽 text.rs:569 门控（A），让有明确高度的 inner 也能触发 paint 列分配（但须守 multicol-fill-auto-001 不回归，同 R198 font_size 死锁交互）。

**预期**：Round 4' 解锁 multicol-breaking-004/006/nobackground-004（3 用例，005 因 balance+balance 嵌套更难独立）。仍是多轮 paint 子系统，非单会话。

### ⚠️ R317 更新（2026-06-19）— Round 4' paint 侧方向实证证伪

R201 Round 4' 的前置「放宽 text.rs:569 `height_auto` 门控（阻塞点 A）」**经 R317 单点实现 + 实验证伪**。把 `!has_in_flow_children && is_balance_mode && height_auto` 放宽为去掉 `height_auto` 后实测 multicol 子集 **40/57→35/57（净 -5 回归）**：multicol-breaking-001/002、nobackground-001/002/005 翻 FAIL，且目标 multicol-breaking-004 **反而恶化** 5.60→6.17%。

**根因**：paint 侧 `compute_multicol_info_for_paint` 的列分布用 `total_height/col_count` 均衡分配，对**明确高度 + 嵌套**用例结构性错误（比单块回退更差）。multicol-fill-auto-* 不受影响（column-fill:auto → is_balance_mode=false），假设验证正确，但 balance 侧大面积回归。

**裁决**：阻塞点 A 的「放宽门控」paint 侧方向**关闭**。这是 R203「paint 侧协调不可解」的第 N 次实证（R157/R198/R203/R122/R317）。Round 4'「paint 侧多轮子系统」整条方向 **ruled out**。

**重定向（与 R203/R131 一致）**：真修复须 **layout 侧 column-aware IFC**——在 layout 阶段（非 paint）计算 multicol 容器的 IFC 行盒，按列高预算（balance = `total/count` 顺序填充到平衡高度；auto/fill = 列高限制顺序填充 + breaking）把**行盒**碎片化到各列，结果存 `LayoutBox`（如复用 `inline_layout` 或新增 `inline_column_lines`），paint 直接消费存储结果（**不再走 text.rs:569 paint 门控重算**）。这与 multicol.rs 已有的 **block** 子元素碎片化（`assign_children_to_columns_*`）互补——前者管 inline 行盒，后者管 block 子元素，**非重复**（纠正 §0「勿再建 measure-first 工具」对 inline 行盒场景的误套：该警告针对 block 子元素测量，不适用于 inline 行盒列分布）。

**下一步（多会话 spec-rfc）**：设计 layout 侧 IFC 行盒→列碎片化的接口（`InlineFormattingContext` 接受 `ColumnFragmentationContext`，输出 `Vec<Vec<InlineLine>>` 每列行盒），Phase 1 = 死字段 + 测量基线（净 0），守 multicol-fill-auto-001 sentinel。**勿再以 paint 侧门控放宽/协调重试**（R157/R198/R203/R317 共 4 轮证伪）。

---

## ⚠️ R200 纠正（2026-06-17）

**原 R199 假设**：multicol 失败因列分配算法（shortest-column round-robin balance 替代均高）。
**实证推翻**：multicol-columns-001（11 行/6 列）接入 round-robin balance 后 **4.88→4.92%（略差）**。

**根因**：chromium multicol §8 是**顺序填充**（先填 col0 到平衡高度 H=T/N，再 col1），**非 round-robin**。而旧代码 `line.y/target_h`（`target_h=total/col_count`）**本就是顺序填充 + 平衡高度**——已正确！我的 round-robin balance 反而破坏了顺序（col0=line0,6 vs 正确 col0=line0,1）。

**结论**：**multicol 列分配已正确**（旧 even-split sequential-fill）。类 A 低 diff 用例（columns-001 4.88%/fill-000 6.54%/count-computed-003 2.06%/004 2.50%）的 diff **不是列分配问题**，而是：
- 列宽精度 / column-gap 子像素；
- 列内 glyph x 位置（estimate_char_width vs 真实 advance，同 DC-13 R188）；
- 平衡高度 H 的精确值（chromium 的平衡二分搜索 vs 我们的 T/N 近似）。

**R199 的 multicol_fragment.rs（round-robin）已移除**（错误算法）。本设计文档保留分析价值（§1 现状、§1.2 四类失败），但 **Round 1-2 balance 方向关闭**。

---

## 0. 执行摘要（已纠正）

- **一句话目标**：让 multicol 容器（`column-count`/`column-width`）把行内流内容**按列高碎片化分配到各列**（CSS Multicol §8 balance + §6 fragmentation），而非当前的 paint 阶段近似均分。
- **核心问题**：当前 multicol 列分配**仅在 paint 阶段**（`painter/text.rs:854`）对 `!has_in_flow_children && balance && height:auto` 的纯行内容器做 `target_h = total_height / col_count` **均高分配**——这是简化近似，非 CSS 规范的 balance 算法，且：
  1. **不含 block 子元素**的 multicol（混合内容）被 `has_in_flow_children` 门控（text.rs:569）跳过，整列堆叠（R157）。
  2. **paint IFC 与 multicol.rs block 分配不协调**——`target_h` 不扣 col1 已被 block 子元素占据的高度（R157 标记的核心缺口）。
  3. **明确高度 + column-fill:auto**（嵌套/breaking 用例）涉及 column breaking（§6），当前完全不支持（R113 两趟循环依赖）。
- **推荐方案**：**列感知 IFC**——让 IFC 在生成行盒时知道「当前列的可用高度」，按列高把行盒碎片化到各列（R131），协调 block 子元素已占空间。分轮渐进：先纯行内 balance 精确化（最大子集），再混合内容门控放宽，再 breaking。
- **首个落地步骤**：⚠️ 经 R201 dump 实测纠正——碎片化算法（`assign_children_to_columns_sequential`/`_with_breaking`）**已存在**，**勿再建 measure-first 工具**（会重复 R199→R200 证伪命运）。真实首步 = Round 4' wiring：放宽 text.rs:569 门控 + 让 `column_span_offsets` paint 路径重绘碎片化 IFC 内容到各列。详见上方「R201 dump 实测定性」。

---

## 1. 现状与根因（已实证，见 master.md R113/R122/R128/R131/R157）

### 1.1 当前实现链路

| 阶段 | 位置 | 行为 |
|------|------|------|
| 布局 | `layout-engine/multicol.rs` | 计算 `col_count`/`col_width`（`compute_single_column_width`，公式 `W=(container-(count-1)*gap)/count` 已验证正确 R185）；block 子元素由 taffy 堆叠；**不做行内流列分配** |
| 提取 | `engine.rs extract_layout` | `is_multicol` 标志（line 651）；multicol 容器的 LayoutBox 携带列几何 |
| 绘制门控 | `painter/text.rs:569` | 仅 `!has_in_flow_children && is_balance_mode && height_auto` 触发 paint 列分配；其余 multicol 整列堆叠 |
| 列分配 | `painter/text.rs:853-882` | `target_h = total_height/col_count` 均分；按 `line.y/target_h.floor()` 分列；`col_first_y` rebase 到列内 y=0；逐列裁剪 |

### 1.2 根因（4 类失败）

| 失败类 | 用例（diff） | 根因 | 轮次实证 |
|--------|------|------|---------|
| **A. 纯行内 balance 精度** | multicol-columns-001(4.88%)/multicol-fill-000(6.54%)/multicol-count-computed-003(2.06%)/004(2.50%) | 均高分配 ≠ CSS §8 shortest-column balance；fractional target_h 致列内偏移 | R128/R185 |
| **B. 混合内容门控** | multicol-containing-002(3.92%)/multicol-block-no-clip-002(1.81%) | `has_in_flow_children` 门控跳过列分配，混合内容整列堆叠 | R157（放宽门控净中性，需协调） |
| **C. 嵌套/breaking** | multicol-breaking-004(5.60%)/005/006(1.20%)/nobackground-004(4.41%)/column-balancing-paged | column breaking（§6 fragmentation）+ 嵌套 multicol 两趟循环（内层高度依赖外层列宽） | R112(dead end)/R113/R132 |
| **D. baseline + column-span** | baseline-007(1.04%)/008(1.46%)/abspos-containing-block-outside-spanner(4.31%)/column-height-009 | multicol baseline 导出 + column-span:all + abspos CB（多子系统交互） | R130 |

**CSS §8 balance 算法**（规范）：把内容填入首列至列高限制，溢出到次列，循环直至「shortest column」平衡。当前 `total/count` 均分是 O(1) 近似，对整除情况（如 4 行/2 列）正确，对非整除（11 行/6 列）产生列高不均 + fractional offset。

---

## 2. 目标状态：列感知 IFC

### 2.1 核心机制

让 IFC（`InlineFormattingContext`）接受一个**列碎片化上下文**（`ColumnFragmentationContext`），在 `break_items_into_lines` 生成行盒后，按列高把行盒分配到列：

```text
ColumnFragmentationContext {
    col_count: usize,
    col_width: f32,
    col_gap: f32,
    /// 每列已占用高度（block 子元素占的空间），从 layout 提取
    col_filled_height: Vec<f32>,
    /// 容器可用高度（balance=auto/∞，definite=style.height）
    available_height: f32,
    /// column-fill: balance | auto
    fill_mode: ColumnFill,
}
```

IFC 输出 `Vec<ColumnContent>`，每列含其行盒 + 行盒在列内的 y。paint 按列渲染（列 x 偏移 + 列内 y + 裁剪），复用现有 `text.rs:881` 的逐列裁剪。

### 2.2 balance 算法（§8，shortest-column-first）

```
1. 所有行盒按 max-content 高度排成序列 lines[0..n]
2. col_heights = col_filled_height.clone()  // 起始 = block 已占
3. for line in lines:
     col = argmin(col_heights)  // 最短列
     assign line to col, line.y_in_col = col_heights[col]
     col_heights[col] += line.height
```

对纯行内（col_filled_height 全 0），结果 = 尽量均匀分布（shortest-column），比均高更接近 chromium（实测 multicol-columns-001 ref 即此模式）。

---

## 3. 分轮实施计划（渐进，每轮零回归门禁）

> **⚠️ v0.4 修订（R310）**：原 Round 1-2（balance 测量工具 + shortest-column 接线）经 **R200 证伪**——multicol 列分配（顺序填充 + 平衡高度 `total/col_count`）**本就正确**，类 A 低 diff 用例（columns-001/fill-000/count-computed-003/004）的 diff 不是列分配问题，是列宽精度 / glyph x 位置 / 平衡高度精确值。**Round 1-2 已废弃，勿再建 balance 工具**（R199 建过 `multicol_fragment.rs`，R200 移除）。R201 进一步证实碎片化算法（`assign_children_to_columns_sequential`/`_with_breaking`）**已存在**，缺口是接线。下述计划据 R200/R201/R307/R309 重定向。

### Round 1'：baseline-export pre-pass（类 D 子集，最大 near-pass 聚类）— ⚠️ R316 证伪（ZeroWeb 后处理侧穷尽）

> **R316 更新（2026-06-19）**：baseline-export 经 **4 轮**（R310 探针 / R312 双侧探针 / R313 baseline_overrides 证伪 / **R316 flex 后处理实现+证伪**）从 **ZeroWeb 后处理侧彻底 ruled out**。R316 实测 baseline-003：两 flex 项（"PA" 文本 + multicol）**均已被 taffy 基线对齐**（同 y=0/h=19）；flex-baseline 后处理（`adjust_flex_baseline_alignment`）用兄弟项派生 target→no-op（两 item 均 taffy_baseline=None），用容器 taffy_baseline(19.2) 作 target→**回归 baseline-001/002**（把已对齐项错误下移）。三种机制覆盖 field-fill（R266 净 0）/ inline-flex 后处理（R313 无效）/ block-flex 后处理（R316 回归）全谱。**真修复须 taffy inline-level-box 基线合成或升级 taffy（0.8+ baseline_overrides，R304 DEFER prohibitive）**。下方原始 Round 1' 计划保留作历史，但**勿再以 ZeroWeb 后处理方式重试**。

- **目标用例**：baseline-000/003/004/005/006（self 0.12-0.14%，5 案）+ baseline-001/007/008。结构 = `display:flex; align-items:baseline` 含 multicol flex 项（如 baseline-003 = flex > "PA" 文本 + `columns:3` multicol > `column-span:all` "SS"）。
- **根因（区别于 R266）**：R266 查 `LayoutBox.taffy_baseline` field-fill 净 0（消费 guard 仅 InlineFlex|InlineGrid）；但 baseline-003 是 **flex 项（multicol）的 first baseline 须传给 taffy 供 `align-items:baseline`**——taffy 内部对 multicol block 项无正确 first baseline（multicol 内首列首行 baseline 未在 layout 侧计算/暴露），故 flex 基线对齐用错值。
- **修复方向**：在 multicol layout（`multicol.rs`）计算首列首行 baseline，写入 `LayoutBox`（新字段或复用 `taffy_baseline`），converter/extract 把它作为该 block 项的 first baseline 喂给 taffy 的 baseline 合成。
- **门禁**：baseline-003/004/005/006 改善且全量 loose 438/490 / strict 不退；chromium-Oracle z_vs_chr 下降。**前置实证**：先 probe 确认 multicol 项当前传给 taffy 的 first baseline 值（疑为 0 或 box bottom）。

### Round 2'：breaking wiring（类 C，R201 Round 4' 重定向）

- **目标用例**：multicol-breaking-004/005/006/nobackground-000/001/003/004（self 0.17-1.21%）。
- **根因（R201）**：碎片化算法已存在，缺口 = ① paint 门控 `height_auto`（text.rs:569）挡住有明确高度 inner 的子列布局；② `column_span_offsets` paint 路径不重绘碎片化 IFC 内容到非主位置列。
- **风险**：R198/R209 证 multicol-fill-auto-001 经 font_size 存储/列分配耦合易回归（0.63→9.15）；放宽门控须守此用例。R203 证 paint 侧简单协调全 net-negative，须 layout 侧 column-aware IFC（R131）。
- **门禁**：逐用例 set-diff，multicol-fill-auto-001 不回归。

### Round 3'：column-rule + 精度收尾（类 A 残余 + C 子项）

- column-rule §5.2 内容检测（R201 标 C，但 column-rule-002 回归须区分 column_span_offsets 来源）。
- 类 A 残余（列宽精度 / glyph x）属 advance-width 谱系（R225 证伪独立死路），非 multicol 专属。

---

## 4. 关键约束与风险

1. **R200 纠正**：列分配（balance）方向**已关闭**——旧 `total/col_count` 顺序填充正确，类 A 残余是精度非算法。
2. **R201 纠正**：碎片化算法**已存在**，缺口是 wiring（paint 门控 + column_span_offsets 重绘），非新建 measure-first 工具。
3. **Round 1' baseline 是 flex×multicol 跨子系统**：须厘清 taffy 如何消费 block 项 first baseline（cached_baselines 补丁路径，engine.rs:1002），可能需 converter 侧改动。
4. **Round 2' breaking 是结构里程碑**：column breaking 涉及行盒跨列断裂，须 layout 侧 column-aware IFC（R131）；R203 证 paint 侧不可解。
5. **font_size Phase A 交互**：multicol 容器被 Phase A font_size 死锁反向依赖（R158/R198/R209）。任何改 multicol 列分配/存储的轮次须验证 multicol-fill-auto-001 不回归（余量 0.63% 小）。

## 5. 预期收益（v0.4 修订）

- **Round 1' baseline-export**：baseline-000/003/004/005/006 + 001/007/008（~8 用例），strict +5~8，最大 near-pass 聚类。**前置 probe 验证假设**（multicol 项 first baseline 是否传错）。
- **Round 2' breaking wiring**：multicol-breaking-004/005/006/nobackground-*（~6 用例），结构性多轮。
- **Round 3' column-rule + 精度**：零散，低收益。
- 全部完成：css-multicol 当前 strict 失败 ~20 → ~5（≥95%），但 Round 2' 是硬里程碑。

**首步（下轮）= Round 1' baseline-export probe**：先 read-only probe 确认 multicol flex 项传给 taffy 的 first baseline 值（baseline-003），验证「multicol first baseline 未正确导出」假设，再决定修复路径（layout 侧计算首列首行 baseline 喂 taffy）。这是当前最大 near-pass 聚类且根因疑为 flex×multicol 跨子系统 wiring（区别于 R266 的 field-fill 净 0 结论），值得先 probe。
