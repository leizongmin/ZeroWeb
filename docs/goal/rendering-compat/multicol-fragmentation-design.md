# 设计草图：multicol 列感知 IFC 碎片化（column-aware fragmentation）

**版本**：v0.2（**R200 重大纠正**：列分配已正确，原 balance 方向错误）
**日期**：2026-06-17
**状态**：分析完成；**列分配（balance）方向已证伪关闭**（R200）
**关联**：rendering-compat master.md R113/R122/R128/R131/R157/R199/R200；css-multicol 17/57 失败

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
- **核心问题**：当前 multicol 列分配**仅在 paint 阶段**（`painter/text.rs:948`）对 `!has_in_flow_children && balance && height:auto` 的纯行内容器做 `target_h = total_height / col_count` **均高分配**——这是简化近似，非 CSS 规范的 balance 算法，且：
  1. **不含 block 子元素**的 multicol（混合内容）被 `has_in_flow_children` 门控（text.rs:711）跳过，整列堆叠（R157）。
  2. **paint IFC 与 multicol.rs block 分配不协调**——`target_h` 不扣 col1 已被 block 子元素占据的高度（R157 标记的核心缺口）。
  3. **明确高度 + column-fill:auto**（嵌套/breaking 用例）涉及 column breaking（§6），当前完全不支持（R113 两趟循环依赖）。
- **推荐方案**：**列感知 IFC**——让 IFC 在生成行盒时知道「当前列的可用高度」，按列高把行盒碎片化到各列（R131），协调 block 子元素已占空间。分轮渐进：先纯行内 balance 精确化（最大子集），再混合内容门控放宽，再 breaking。
- **首个落地步骤**：实现 `column_heights` 测量工具（从 multicol 容器几何 + block 子元素已占高度计算每列可用高度）+ 单元测试，**不接线**（先证明测量正确，同 flex-grid-two-pass 的「测量先行」方法学）。

---

## 1. 现状与根因（已实证，见 master.md R113/R122/R128/R131/R157）

### 1.1 当前实现链路

| 阶段 | 位置 | 行为 |
|------|------|------|
| 布局 | `layout-engine/multicol.rs` | 计算 `col_count`/`col_width`（`compute_single_column_width`，公式 `W=(container-(count-1)*gap)/count` 已验证正确 R185）；block 子元素由 taffy 堆叠；**不做行内流列分配** |
| 提取 | `engine.rs extract_layout` | `is_multicol` 标志（line 651）；multicol 容器的 LayoutBox 携带列几何 |
| 绘制门控 | `painter/text.rs:711` | 仅 `!has_in_flow_children && is_balance_mode && height_auto` 触发 paint 列分配；其余 multicol 整列堆叠 |
| 列分配 | `painter/text.rs:948-984` | `target_h = total_height/col_count` 均分；按 `line.y/target_h.floor()` 分列；`col_first_y` rebase 到列内 y=0；逐列裁剪 |

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

IFC 输出 `Vec<ColumnContent>`，每列含其行盒 + 行盒在列内的 y。paint 按列渲染（列 x 偏移 + 列内 y + 裁剪），复用现有 `text.rs:974-984` 的逐列裁剪。

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

### Round 1：列高测量工具（不接线，零风险）

- 新增 `crates/layout-engine/src/multicol_fragment.rs`（同 `intrinsic_sizing.rs` 模式）：
  - `compute_column_heights(container, col_count, block_filled) -> Vec<f32>`：纯计算每列可用高度。
  - `balance_lines_to_columns(lines, col_count, col_filled) -> Vec<Vec<line_idx>>`：shortest-column 分配。
- 单元测试：4 行/2 列均匀、11 行/6 列（2,2,2,2,2,1）、含 block 已占（col1 已占 30px）。
- **门禁**：不接入 compute/paint，上游 reftest 438/490 持平。

### Round 2：纯行内 balance 精确化（类 A 用例，接线 paint）

- `painter/text.rs:948` 的均高分配改为调 `balance_lines_to_columns`（shortest-column）。
- 移除 `col_first_y` fractional rebase（balance 算法天然每列首行 y=0）。
- **门禁**：multicol-columns-001/fill-000/count-computed-003/004 改善且全量零回归（paint-only 改动，layout 不动）。

### Round 3：混合内容门控放宽 + 协调（类 B 用例）

- 放宽 text.rs:711 门控：`!has_in_flow_children` → 允许有 block 子元素，但用 `col_filled_height`（block 已占）初始化 balance。
- `col_filled_height` 从 layout 提取：multicol 容器内每列的 block 子元素累积高度。
- **门禁**：multicol-containing-002/block-no-clip-002 改善；逐用例 set-diff 确认协调正确（R157 标记的「target_h 未扣 col1 block」）。

### Round 4：column breaking（类 C，结构性里程碑）

- 明确高度 + column-fill:auto 的碎片化（§6）：内容按列高断行到下一列，含 column-span:all 中断。
- 嵌套 multicol 两趟（R113）：内层 balance 高度依赖外层列宽——首趟用外层列宽测内层高度，二趟分配。
- **门禁**：multicol-breaking-004/005/006/nobackground-004；column-balancing-paged。此轮最重，可能需多子轮。

### Round 5：baseline + column-span（类 D，多子系统）

- multicol baseline 导出（§css-align baseline-export）、column-span:all 作为 spanner、abspos CB。
- 依赖 Round 3-4 的列结构。
- **门禁**：baseline-007/008/abspos-containing-block-outside-spanner/column-height-009。

---

## 4. 关键约束与风险

1. **paint-only Round 1-2 安全**：改 paint 列分配不动 layout 几何，同源 reftest 自源中性风险低（multicol 用例 test/ref 同分配逻辑）。
2. **Round 3 协调是核心难点**：`col_filled_height` 需从 layout 的 block 子元素几何提取，跨 layout→paint 边界传递（类似 R109 fragment 注册表）。
3. **Round 4 breaking 是结构里程碑**：column breaking 涉及行盒跨列断裂（一行可能 split 到两列），需 IFC 支持 fragment-aware line breaking（非整行分配）。R113 的嵌套循环依赖是已知硬点。
4. **font_size Phase A 交互**：multicol 容器被 Phase A font_size 死锁反向依赖（R158 multicol-fill-auto 仅因 16px bug 通过）。Round 2-3 改 multicol 列分配时须验证 multicol-fill-auto 不回归（其 0.63% 余量小）。

## 5. 预期收益

- Round 1-2（纯行内 balance）：预计解锁 multicol-columns-001/fill-000/count-computed-003/004（~4 用例），438→~442。
- Round 3（混合内容）：~2 用例。
- Round 4-5（breaking/baseline）：~8 用例（结构性，多轮）。
- 全部完成：css-multicol 40/57 → ~55/57（≥95%），438→~453/490（92%）。

**首步（下轮）= Round 1 测量工具**：纯计算 + 单测，零风险不接线，证明 balance 算法正确。
