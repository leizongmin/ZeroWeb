# RFC：BFC 元素旁 float 放不下须下沉（general BFC-relocate-below-float）

**版本**：v1.1
**日期**：2026-07-17
**状态**：Slice 1+2 合并 LANDED（R1619，with-margin-008/009 flip，CSS2 NET +2 0 回归）；Slice 3/4 待续
**起源**：R1616 forward「剩 close case floats-bfc-003 / with-margin-008/009」；R1617 探针确认 BFC 不下沉 = 真根因。

---

## 0. 执行摘要

- **一句话目标**：建立 BFC 的块级元素（`overflow:hidden` / `display:flow-root` / table / inline-block 等）其 border-box 与同 BFC 浮动重叠且**水平放不下**时，须**下沉到 float 底边之下**（CSS 2.1 §9.5）。当前 ZW 仅左 float + definite-width + 无后续同胞时下沉（R1369），右 float 仅收缩宽度（不下沉），嵌套 BFC（在非 BFC wrapper 内）甚至看不到祖先 float。
- **本期范围**：仅本 RFC（设计 + 簇映射 + slice 拆分 + 回归风险），不下结论先大规模改代码；Slice 1（右 float 对称下沉 + 直接同胞）作为首个可 A/B 验证切片。
- **明确排除**：font-wall（R1605 deadlock）；vertical-mode float（R109 blocked）；floats-wrap-bfc-006（R1608/FR-001 独立 track，本 RFC 不覆盖其 nested table 谱系）。
- **核心约束**：① 每 slice scoped 到结构签名（R1518e V2 方法论），禁全树重跑（R1518 net-2 / R1610 margin-collapse 回归教训）；② 每 slice 全量 A/B（floats + floats-clear + margin-collapse + tables + css-position 全套），net<0 即 revert；③ root-cause-first，kill-switch + default-on。

---

## 1. 背景

R1608/R1616 起 float REAL-BUG 簇 scoped 推进已 LANDED +6 flip（table_float_fix R1609/R1612/R1613 + remeasure_text definite-height R1616）。扫 floats 全量 self-source 剩余 FAIL，发现一批**同根因簇**：BFC 元素旁 float 放不下时应下沉，ZW 未做或仅部分做。

## 2. 簇分类与 self-source 实测（R1617 扫描）

| 子类 | 代表 case | self-source | 结构特征 | ZW 缺口 |
|------|-----------|-------------|----------|---------|
| A. 右 float + 嵌套 BFC 放不下 | with-margin-008/009 | 1.04% / 1.04% | BFC 在 `margin-right/left:50px` wrapper 内，旁 right/left float | 嵌套 BFC 看不到祖先 float + 右 float 不下沉 |
| B. BFC shrink-to-fit + 迭代重试 | floats-bfc-003 | 1.04% | overflow:hidden BFC 先试 200 宽（旁 float1）→ 撞 float2 → 重试 100 宽 | 无 BFC shrink-to-fit-with-retry（table_float_fix 仅 table） |
| C. 百分比宽 BFC 旁 float | floats-wrap-bfc-005 | 7.92% | overflow:hidden width:50% 旁 200px float（300 容器） | 百分比宽不触发 R1369 definite-width 分支 |
| D. BFC top-below 定位 | floats-wrap-top-below-bfc-003l/003r 等 6 变体 | 3.29–6.04% | span display:block overflow:hidden 旁 float | top-below 几何分歧 |
| E. table 旁 float 放不下 | floats-wrap-bfc-001/002-right-table | 3.12 / 7.29% | table 旁 right float | table_float_fix 部分覆盖，right-table 残缺 |
| F. zero-height float wrap | floats-zero-height-wrap-001/002 | 6.25% | zero-height float + BFC | zero-height float 几何 |

簇规模 ≈ 18+ case，全 self-source ≥1%（多数 ≥3%，非 font-wall，真 geometry gap）。

## 3. root-cause（R1617 探针实证）

`float_positioning.rs::adjust_float_positions_with_context` 的 BFC 浮动排斥段（:881-940）：

```
if child block-level && !abspos && establishes_bfc(child) {
    for each float_geometry (同容器 float 同胞):
        左 float: avoidance_x = float 右 margin-box 边
                  if overflows && definite-width && !has_following_block_sibling → child.y = float_bottom（R1369 下沉）
                  else if avoidance_x > child.x → 推右 + shrink-to-fit
        右 float: child.width = float_x - child.x（仅收缩，不下沉）  ← ★ 缺口 1
}
```

**三个独立缺口**：

1. **右 float 不下沉**（:932 仅收缩 `new_width = float_x - child.x`）：右 float 旁 definite-width BFC 放不下时，应像左 float（R1369）一样下沉到 float_bottom，而非收缩到 0/负宽。
2. **嵌套 BFC 看不到祖先 float**：`float_geometries` 每容器按自身 float 子重建（:1153 递归时非 BFC 子继承 `left_ctx/right_ctx` 底边，但**不传 float 的 x/border/margin 几何**）。with-margin-008 的 BFC 在 `margin-right:50px` wrapper（非 BFC）内，wrapper 递归时 float_geometries 不含 wrapper 级 float → BFC 不触发排斥。
3. **百分比宽不进 R1369**：`is_definite_width = child.width < container_width - 0.5`（:914）对 width:50%（=150 < 300）为 true，但 `overflows = child.x + width > container_width`（:913）对 width:50% @ x=0 = 150 < 300 = false → 不下沉。需改为「放不下 float 旁可用宽」判定。

## 4. 推荐方案：分轨 scoped slice

每 slice 独立 kill-switch + default-on + 全量 A/B net≥0。按 ROI×风险序：

### Slice 1（首切，低风险）：右 float 对称下沉 + 直接同胞 definite-width BFC
- **改**：:932 右 float 分支加 R1369 对称判定——definite-width + overflows（`child.x + width > float_x`，即放不下 float 左侧可用宽）+ 无后续 in-flow block 同胞 → `child.y = float_bottom`，不收缩。
- **scope gate**：仅 `establishes_bfc && definite_width && !has_following_block_sibling && 右 float`。
- **目标 flip**：直接同胞右 float BFC case（flips-bfc-001 等）。
- **风险**：低（对称 R1369 已 A/B 验证左 float 安全）。

### Slice 2（中风险）：嵌套 BFC 祖先 float 几何透传
- **改**：非 BFC 子递归时（:1162），除 `left_ctx/right_ctx` 底边外，透传祖先 float 的 `(dir, x, border_w, margin_r)` 几何到子容器 `float_geometries`，使嵌套 BFC 能触发排斥。
- **scope gate**：透传仅对「子容器内有 establishes_bfc 后代」生效（避免无 BFC 后代容器的开销 + 回归）。
- **目标 flip**：with-margin-008/009（+2）。
- **风险**：中（margin-collapse 交互，R1610 教训；须守全量 A/B）。

### Slice 3（高风险）：百分比宽 BFC 下沉判定改「放不下 float 旁可用宽」
- **改**：R1369 `overflows` 判定从「> container_width」改为「> available_width_beside_float」。
- **目标 flip**：floats-wrap-bfc-005（百分比宽）。
- **风险**：高（available_width 计算须精确，易误触发 shrink→flip 回归）。

### Slice 4+：B（shrink-to-fit-retry）/ D（top-below）/ E（right-table）/ F（zero-height）
- 各自独立 root-cause，按 R1608/R1613 方法论逐案 dump ref + 对齐。B 需 BFC shrink-to-fit-with-retry（扩展 table_float_fix V2 到 generic BFC），最大工程量。

## 4a. R1618 实证 findings（Slice 1 + Slice 2 试 land，均 net 0 revert）

- **Slice 1（右 float 对称下沉）单独 land**：floats 242→242、CSS2 5505→5505 **全量 net 0**。原因：当前 FAIL 簇的目标案（with-margin-008/009）是**嵌套 BFC**（BFC 在非 BFC margin-div 内），直接同胞路径根本不触达；无「直接同胞右 float definite-width BFC」FAIL case 命中。→ revert。
- **Slice 2（嵌套 BFC 祖先 float 几何透传）单独 land**：CSS2 5505→5505 **net 0**。透传机制**证正确**（margin-div 经 `inherited_floats` 收到 wrapper 的 right float 几何，转 child 帧 `fx - child.x`；BFC 排斥段读到该 float），但**右 float 分支只 shrink 不 push-below**（Slice 1 revert）→ BFC 从 w=100 重叠 变 w=50 并排，**y 不变（仍 0）、wrapper 高度不变（仍 50）**→ diff 像素位置略移但未过阈 → net 0。→ revert。
- **★ 关键阻塞（精确化）**：Slice 1+2 **必须同时 land**，且 Slice 1 的 `is_definite_width = child.width < container_width - 0.5` 启发式在**嵌套+margin 上下文失效**——with-margin-008 的 BFC width=100 **溢出**其窄父 margin-div（content_width=50，因 margin-right:50px），`100 < 49.5` = false → push-below 不触发。须把 `is_definite_width` 重定义为「BFC 有 declared（非 auto）宽度」而非「width < container_width」，push-below 触发改「BFC definite 宽 > float 旁可用宽」。

## 5. 验收标准

- 每 slice：全量 A/B（`make reftest` 走 reftest-upstream CSS2 + reftest-oracle floats + product-smoke + make test）net≥0，目标 case flip，0 回归（尤其 margin-collapse-clear 簇、floats-clear 簇）。
- kill-switch + default-on；load-bearing 单测（parse_html + LayoutEngine::compute 几何断言，仿 r1616/r1277 模式）。
- 门禁：fmt / clippy -D warnings / make test 全绿 / product-smoke exit 0。

## 6. 回归风险（R1518/R1610 教训）

- **margin-collapse**：BFC 下沉改变 y 后，后续同胞定位 + 容器高度 + clear 几何连锁。须守 `floats-clear` + `margin-collapse-*` 全量 A/B。
- **全树重跑禁令**：R1518 全树 adjust_float_positions 净 -2；R1610 全局 BFC-avoid 回归。每 slice 限结构签名 gate，早返回无该结构的 case。
- **taffy native float**：taffy 0.12 自带部分 float 定位，改动须与 taffy 已定位协调（R1369 注释 :909 提及「taffy 0.12 native float 可能已推 BFC 到 float 右」）。
- **R1618 经验**：Slice 1/Slice 2 各自 net 0（目标案是嵌套=Slice 2 territory + 启发式需 Slice 1 配合）→ **必须合并提交 + 重设计 `is_definite_width` 启发式（declared-width 而非 < container_width）**，单独任一 slice 无 yield。

## 7. forward / 续跑入口

- R1617：RFC landed + root-cause 实证；0 code land。
- R1618：Slice 1 + Slice 2 各自试 land 均 net 0 revert；精确化阻塞 = 两 slice 必须合并 + 重设计 `is_definite_width`（declared-width 而非 < container_width）。
- **R1619（LANDED）**：合并 Slice 1+2 + 重设计。float_positioning.rs：`adjust_float_positions_with_context` 加 `inherited_floats: &[FloatGeom]`（透传祖先 float 几何到非 BFC 后代，转 child 帧），hoist `float_geometries`+`all_floats` 到函数作用域；BFC 排斥段**新增**独立 `inherited_floats` 下沉分支（不触 R1369 直接同胞路径）——`!child.declared_width_auto && child.width > float 旁可用宽 && !has_following_block_sibling` → `child.y = float_bottom`。kill-switch `ZW_NESTED_BFC_FLOAT_AVOID` default-on。A/B：with-margin-008/009 **0.00% FLIP**，CSS2 self-source 5505→**5507 NET +2 0 回归**（per-case 确认）；fmt/clippy(-D)/make test/product-smoke 全绿；+2 load-bearing 单测。容器高度增长由既有 containment 机制自动处理（无需新增 pass）。**float 簇累计 +8**（table_float_fix +3 / remeasure_text +3 / nested-BFC +2）。
- 下轮：Slice 3（百分比宽 floats-wrap-bfc-005，7.92%）或单案逐 root-cause（floats-bfc-003 BFC shrink-retry / floats-wrap-top-below-bfc-* top-below）。每轮记 master.md + evidence。

## 8. R1620 后续（cell-content-height，非 BFC-relocate 但同簇连带）

- floats-wrap-bfc-005 探针实证：BFC（R1369 推下）定位已对，**cell 高度未长** = `cell_float_aware_content_height`（R1390）`SUM(heights)` 在子元素重定位/margin 折叠时低估/高估。改 `MAX(c.y+c.height+mb)`（spec §17.5.3）LANDED：CSS2 NET +3（floats-wrap-bfc-007 + 2 margin-collapse bonus），0 回归。floats-wrap-bfc-005 本身 7.92→6.67%（TABLE 子案 table_float_fix 残缺未过阈）。
