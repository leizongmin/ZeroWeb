# RFC：BFC 元素旁 float 放不下须下沉（general BFC-relocate-below-float）

**版本**：v1.2
**日期**：2026-07-19
**状态**：Slice 1+2 合并 LANDED（R1619，with-margin-008/009 flip，CSS2 NET +2 0 回归）；Slice 3 精神落地于 R1728（002r，左 float fits_beside gate，§10.1）；Slice 4-D top-below 子簇 002r ✅ / 001r+003l defer 为 Slice 5 多-float 协调（§10.2）；Slice 3 百分比宽 / 4-B/4-E/4-F 待续
**起源**：R1616 forward「剩 close case floats-bfc-003 / with-margin-008/009」；R1617 探针确认 BFC 不下沉 = 真根因。R1725–R1729 续 floats-wrap-top-below-bfc 子簇（§10）。

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

## 9. R1621 floats-wrap-bfc-005 TABLE 子案 root-cause（defer，单 6.67% 案，intricate）

- 探针 dump floats-wrap-bfc-005 TABLE 子案（左 + 右 float，inner table width:50%）：
  inner table 已被正确推到 float 下方（y=20, w=150），**td(cell) h=40**（R1620/table_float_fix
  step D 长到 40），但 **TableRow / TableRowGroup / 外层 Table 仍 h=20** → cell 溢出 row，
  外层 table h=20 使 4 子案 table 堆叠重叠（y=0/20/40/60 应 0/40/80/120）= 残 6.67%。
- **root-cause（ordering）**：外层 table 的 row 高度在 step8 `position_cells`（table.rs:1218
  `row_box.height = row_height`，row_height 取 `cell_float_aware_content_height`）计算——此时
  inner table 尚未被 step8.5 `table_float_fix` 推下（仍在 y=0），故 cell_float_aware=20 →
  row=20。step8.5 推下 inner table 并（经 R1620）长 td 到 40，但**不回传 row/rowgroup/外层 table
  高度**。
- **fix 方向（defer，须 dedicated pass）**：step8.5 后加 growth-only pass——对每个 TableRow，
  `row.height = max(row.height, max cell.height)`；delta 传播到 rowgroup → 外层 table 高度；
  再 `reflow_siblings_after_table_height_change` 移后续 table。**风险**：broad table-height-chain
  改动（R1518/R1610 教训：全树 table/float 改动易回归 margin-collapse / 其他 table 案）。
  须 kill-switch + 全量 CSS2 A/B（尤其 css-tables + margin-collapse + floats-clear）net≥0。
- **裁决**：单 6.67% 案不值得本轮冒险 broad pass（危及已得 +9）；defer 到 dedicated 轮次，
  先写 scoped growth-only pass + 严 A/B。转更可解的单案或 floats-bfc-003。
- **R1622 试 land growth-only pass = NET -6 revert**：实现 `grow_tables_for_cell_overflow`
 （table.rs，bottom-up 对 row/rowgroup/table growth-only 取 max 子底边 + reflow_siblings，
  kill-switch ZW_TABLE_HEIGHT_GROW）。A/B 全量 CSS2 **5510→5504 NET -6**：floats-wrap-bfc-005
  改善 6.67→1.76（未 flip）但 **6+ 案回归**（growth-only 假设「cell 溢出 row 必长 row」错——
  baseline 对齐 / 显式高 table / vertical-align 场景下 cell.y>0 使 max(c.y+c.height) 误长）。
  revert。**结论**：floats-wrap-bfc-005 须**根本上不同**方案（非 broad growth-only）——
  例如只在「cell 因 float 推下而溢出」的精确结构签名触发（gate on cell 内有被 float 推下的
  BFC/table 子），而非对所有 row growth。defer 到更精确 gate 的专项轮次；当前转其他 target。

## 10. R1728/R1729 floats-wrap-top-below-bfc 子簇 resolution（Slice 4-D 落地 + 多-float 协调 Slice 5 提案）

**版本**：v1.1 续（2026-07-19）。承接 Slice 4+ 的 D（top-below）。R1725 解除 dump 工具坑
（REFTEST_DUMP 实为可靠，R1724「UB」误诊推翻）后，对全 6 变体（001l/r、002l/r、003l/r）
逐案 ZW-TEST vs chromium-oracle 像素 diff 分侧（R1726 triage）：**l 变体=REF-side（ZW TEST 对、
REF 渲染错），r 变体=TEST-side（ZW TEST 偏离 chromium）**。推翻「全簇=单一 BFC avoidance bug」
框定——实为两条独立线。

### 10.1 002r ✅ LANDED（R1728，commit f2a85f487）= Slice 3「放不下 float 旁可用宽」精神落地

- **结构**：body 400 + `float:right 150×75` + `float:left 300×75`（450>400 不并行，left float 下沉
  到 right float 底）+ 2× BFC span（block overflow:hidden，**声明宽 200**）。
- **根因（R1727 instrument ZW_R1727_PROBE 实证）**：span2 进 BFC-avoid 时在自然位 x=0 y=50 w=200，
  float:left 300 宽（右可用仅 [300,400]=100）。R1369 gate `overflows = child.x+child.width > container_width`
  = `200 > 400` = **false** → pushdown 不 fire，落 squeeze 分支缩到 w=100 留 beside。
- **fix**：R1369 左 float 分支 `must_pushdown = overflows || (left_fit_pushbelow && is_definite_width
  && !declared_width_auto && !fits_beside)`，`fits_beside = child.width <= container_width - avoidance_x`。
  = **Slice 3「> available_width_beside_float」判定的左 float 落地**，加 `!declared_width_auto` gate
  排除 auto 宽 BFC（floats-bfc-003 / new-fc-beside-float 回归源）。
- **验证**：002r ZW-TEST vs chromium **4.00%→1.00%**；A/B floats+floats-clear **net 0**（0 回归 0 新
  self-source pass——002r self-source 仍 fail 因 REF 侧 IFC 独立 bug，见 10.3）；welcome product-smoke
  16.98% PASS；2 load-bearing 单测；kill-switch `ZW_BFC_LEFT_FIT_PUSHBELOW`。
- **R1727 三假设全证伪**：float_geometries 含两 float ✓ / establishes_bfc(span) ✓ / 非分支覆盖——
  是 gate 条件本身漏判。

### 10.2 001r / 003l = 多-float BFC 协调（Slice 5 提案，per-float 循环无法解，defer）

两案**同根因**：BFC 同时垂直重叠 ≥2 float，per-float 循环独立处理致 over-pushdown。

| case | 配置 | chromium span2 | ZW span2 | 缺口 |
|---|---|---|---|---|
| 001r | 2× `float:right clear:right`（50+100 宽，叠右侧）+ BFC span `margin-left:auto` w=200 | 与 span1 相邻成 1 blob x=[111,360] y=[11,110]；span2 **右对齐到 div2 左缘** x=[111,311] y=[60,110] | span2 推到两 float 下 y=[164,214]，x=[161,361]（右缘=div1 左缘，未重右对齐到 div2） | R1722 右 pushdown **逐 float 重复 fire**；且正确行为=「margin-left:auto BFC 右对齐到最左 obstructing float 左缘保持宽」，当前代码只有 pushdown/shrink 两分支无此第三行为 |
| 003l | `float:left 250` + `float:right 250`（500>400 不并行）+ BFC span w=100 | span2 **下到 float L 底 y=[89,139] 且 旁 float R** x=[11,110] | span2 推到 float R 下 y=[164,214] x=[11,110] | span2 应「下到 float L 底 + 旁 float R」（y=89, x<161），per-float 循环无「跨 float 协调找同时不重叠位」能力 |

**Slice 5 算法草案（multi-float BFC coordination）**：BFC-avoid 段对每个 BFC 子收集**所有**垂直重叠
float（不再逐 float 独立 fire），然后：
1. 计算候选 y 序列 = {child 自然 y} ∪ {每个重叠 float 的 bottom}（升序）；
2. 对每个候选 y（从低到高），检查 BFC 在该 y 是否与任一 float 重叠；若 BFC 声明宽 > 该 y 处旁置可用宽
   （考虑该 y 处所有现役 float 的联合占用）→ 排除该候选；
3. 取**首个可行候选 y**（最低可行）。若 none → 下到最晚 float bottom。
- **margin-left:auto 特化（001r）**：可行候选处，BFC 右对齐到该 y 处最左 obstructing float 左缘
  （`child.x = min_obstructing_float_x - child.width`），保持声明宽。
- **scope gate**：仅当 BFC 垂直重叠 ≥2 float 时走协调路径（单 float 走既有 R1369/R1722/R1728 分支，
  零回归）。
- **风险**：高——重写 BFC-avoid 核心，须守 R1369/R1619/R1722/R1723/R1728 全部既有行为；margin-collapse
  + clear + container-height 连锁。须 kill-switch `ZW_BFC_MULTIFLOAT_COORD` default-OFF 渐进 A/B。
- **验收**：001r/003l ZW-TEST vs chromium → <1.5%（border 噪声）；A/B floats+floats-clear+margin-collapse+
  css-position net≥0；product-smoke；load-bearing 单测覆盖「≥2 float 协调」+「margin-auto 右对齐到 float」。

### 10.3 l 变体（001l/002l/003r）= REF 侧 IFC inline-block+float 行盒缩短（R109/Phase-A，独立线）

ZW 渲染 TEST 页**正确**（匹配 chromium oracle，~1% border 噪声）。self-source fail 真因 = ZW 渲染
REF 页错：REF 用 `display:inline-block; vertical-align:top` span 旁 float，chromium 把 span1 放 float
右侧（行盒被 float 缩短），ZW 放 float 下方/原位（IFC 行盒未对 float 缩短）。= **IFC inline-level
行盒被相邻 float 缩短缺失 = R109/Phase-A 谱系**。**更高 yield**：l 簇 REF 修好后 001l/002l/003l/003r
self-source 均可 flip（多案）。独立于本 RFC 的 BFC-relocate（属 IFC，另案 RFC / Phase-A 设计）。

### 10.4 forward（续跑入口）

- **Slice 5（多-float 协调）实现轮**：按 10.2 算法草案 + kill-switch default-OFF 渐进 A/B；目标 001r/003l
  TEST-side 对齐 chromium。
- **l 簇 IFC 行盒缩短**：R109/Phase-A 子症状，独立多会话线（修后多案 self-source flip）。
- **bfc-006 / with-margin-001a**：JS-entangled（onload=go），须 harness JS 执行后再 dump。

