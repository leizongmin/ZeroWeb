# R5330 — column flex item 主轴高回填豁免（css-flexbox-1 §4.1 × R1743 回填臂）

- **日期**: 2026-10-29
- **驱动案**: `css/css-flexbox/flex-minimum-height-flex-items-027.xht` ✗2.08% → **✓0.00%**
- **corpus**: 804F（R5329 交付态）→ **803F**（16590 案；**FIXED=1**，REGRESSED=0），
  通过率 15787/16590 = **95.2%**
- **域**: flex 布局主轴定尺 × R1743 parent height backfill 误扩展
- **代码**: `4cf07ff30`（fix(layout): column flex item 主轴高豁免 R1743 max-bottom 回填）

## 定性

027 结构：OuterFlexbox（flex column 100×100）> InnerFlexbox（flex column
`height:100%`）> `.InnerFlexbox-body`（`min-height:0`）> 子 div 200px。期望绿色
100×100；ZW 渲 100×200——item 被「回撑」到子内容底。

链路取证（检查点二分 C0-extract→C7-end，`ZW_R5330_DIAG` 临时插桩已回退）：

1. taffy 第一趟 compute：item 已正确收缩 **100×100**（flex 算法 ✓）。
2. extract_layout（taffy→LayoutBox）：item 仍 100。
3. `shift_siblings_after_ifc_grow`（postprocess 链）：item **100→200** ✗。

根因：R1743 parent height backfill（`shift_siblings_after_ifc_grow_inner` 末尾
max-bottom 回填）的 `parent_backfill_active` gate 按显示谱系（Block/Flow/FlowRoot/
ListItem）放行 flex item——item 的 display 是 Block（不是 Flex），注释声称「排除
flex/grid」但只排除了**容器**（is_plain_real_block 用于 prev/shift 链），未排除
**item** 本身的回填臂。flex 算法收缩定尺的 item 主轴高被重新撑回子内容底，违反：

- css-flexbox-1 §4.1：flexed main size 即 definite used size，内容溢出可见但不
  回撑 item；
- R3903 既有原则（`LayoutBox::is_flex_grid_item` 注释）：flex item 尺寸由 flex/
  grid 算法决定，BFC auto-height 重算不适用。

## 实现

`shift_siblings_after_ifc_grow_inner` 新增 `in_column_flex_main: bool` 参数，
`parent_backfill_active` 链追加
`&& !(in_column_flex_main && col_flex_no_backfill_enabled)`；仅 gate max-bottom
回填分支，cumulative_shift 分支不动（无驱动案不扩范围）。父层递归调用点按
`is_col_flex_item_no_backfill(child)` 传入：

- 父为 column/column-reverse flex（`self_is_column_flex`）；
- 子 `is_flex_grid_item`；
- 子**指定 min-height 非 auto**。

**min-height:auto 收窄是回归驱动**：全域豁免（只按前两条）首跑 corpus A/B 致
`css/css-sizing/aspect-ratio/flex-aspect-ratio-026.html` ✗3.71%（REGRESSED=1）。
诊断（LAYOUT_DUMP）：其 min-height:auto item 依赖 R1743 回填补偿 taffy automatic
min 短板——taffy automatic min 测量回 **transferred size**（25px，宽经 ratio 1/1
传递）而非 content size suggestion（190+10=200px），子例 2/3/6/7 实证。收窄为
仅豁免 automatic minimum 被显式关闭（指定 min-height）的 item：026 复 ✓0.46%、
027 保持 ✓0.00%。撤销回填补偿须先补 taffy content-suggestion 臂。

row 容器 item 的 height 是 cross 轴（内容决定型），不在豁免域（R1743 修复位在
该域合法）；R1743 块流域（body/div br-split 谱系）不受影响。

kill-switch `ZW_FLEX_COL_ITEM_NO_HEIGHT_BACKFILL=0`（default-on，OnceLock 进程级
单读，R5108 教训）。

## A/B 与回归

- corpus 同树 A/B（基线 R5329 交付态 804F，fail-list `/tmp/r5329-after-fails.txt`）：
  → **803F**，集合 diff 恰 {−flex-minimum-height-flex-items-027}，**REGRESSED=0**。
  全域豁免首跑（收窄前）曾测得 803F 但含 {−027, −fieldset-as-item-overflow(bonus),
  +flex-aspect-ratio-026(回归)}——fieldset bonus 在收窄后回落 ✗1.91%（基线本就
  失败，中性）。
- kill-switch =0：027 复现基线 ✗2.08%（双向可判）。
- **+4 pin**（`engine/tests/r5330_col_flex_item_no_backfill_tests.rs`）：027 形
  （min-height:0 保持 flex 定尺 100）、R1743 块流域守卫（body≈div br-split）、
  row 容器 cross 轴守卫（item 200 不受豁免）、min-height:auto 回填补偿位守卫
  （item ≈200）。**mutant**：stash 修复 hunk 实测驱动腿 FAIL 三守卫腿 PASS；
  kill-switch =0 驱动腿 FAIL。双向可判别。

## 门禁

- fmt：`cargo fmt --all` 收敛，0 漂移。
- clippy：workspace `--all-targets -D warnings` 0（guarded 入口）。
- **zero-layout-engine lib 2254P/0F**（2250+4）。
- workspace v8 测试（`--no-fail-fast`）：唯一红 = zero-engine js_dom_bridge fetch
  域 9-10F——**clean-HEAD stash 实测同红**（9F 逐项一致；flaky 边界 9↔10），账本
  「zero-engine fetch 10F」既有家族零位移，与本次布局修改无关。
- **product-smoke**：welcome vs chromium **15.68%** ≤ 20%（R5324 起零位移）+
  struct PASS；**legacy** 42 fixtures struct 0 失败。
- **bench-gate（定向 zero-layout-engine）GATE PASS 25 指标 NEW=0**——静窗实测，
  R5328+R5329 合并清单的静窗复测挂账**一并收口**（R5328/R5329 轮的环境归因定谳：
  当时测量期并行流 encoding rally 163% CPU loadavg 8.95 所致）。

## 挂账

- **收口**：flex-minimum-height-flex-items-027。
- **静窗复测挂账关闭**：bench-gate 定向 zero-layout-engine（R5328+R5329 清单）。
- **同族余案（未驱动，转菜单）**：flex-minimum-height-flex-items-029 ✗8.33% =
  嵌套 column flex 容器 min-content 高（Σ item intrinsic）另臂；
  flex-minimum-width-flex-items-015 ✗6.25% = aspect-ratio transferred minimum ×
  % min-height 解析（R3859 谱系延伸）。
- **taffy automatic min 短板**（content size suggestion 测量回 transferred size，
  flex-aspect-ratio-026 补偿位依赖）转观察项。
- **维持**：zero-engine fetch 10F + details 闪断、R5327 fragment 臂结构域、
  safe/unsafe overflow-position 域、content-none-select-1 脆弱族、c44-003 残差、
  va 族余案、c548 待用户决策、fmt drift 移交。
- **下一步 R5331**：①803F 账本续扫择轻（避开 va 域与 OOF 锚域）②multicol 嵌套
  分片专项预研（8×1.04% 聚簇）③029/015 分诊驱动择机。
