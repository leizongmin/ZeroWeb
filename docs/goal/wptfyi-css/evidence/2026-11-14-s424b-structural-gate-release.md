# s424b：selector 臂结构性放行刀——受控 A/B 定谳与回退

日期：2026-11-14。承接 s424a 挂账（evidence
2026-11-14-s424a-class-gate-widening-experiment.md §3：anchor-name-mutation
15F 页变异经 template cloneNode+append 结构性队列，class 门拓宽清偿不了）。
本刀撤 s423 family 谓词本身（callbacks.rs s423 臂 gate 改恒真，对齐 s421
handle 臂无门全重放语义）。**全程未提交**：callbacks.rs 已回 HEAD 零残留，
工件留 `.acceptance/wptfyi-css/`（run-s424b-structural.sh / results-s424b-
structural / logs/zw-s424b-structural.log）。

## 1. 刀口与 A/B

单变量 = 写后同 turn 读门（s423 收窄引入的 family 谓词）撤除与否；基座同
s424a 对照组（peer g38 提交态 19:14:21 与 19:02:37 WIP 落盘同内容，engine
门 = HEAD），二进制 19:18:31 链出（33s 增量构建）。

- **对照组**（postrevert，门 = HEAD）：265 页 12,061P/792F/4T（s424a §3 同
  面貌）。
- **实验组（结构性放行）**：265 页 **10,996P/1,857F/4T，净 −1065**。

## 2. 翻转面分解（1069 down / 4 up）

**UP（4）**：anchor-position-dynamic-001 `#target 1` / dynamic-004
`.target 1` / anchor-fallback-invalidation 1 案 / zz-g6probe-mut5 探针 1——
与 s424a 拓宽组 +3 完全同面，证明门机械上确实开了。

**DOWN（1069）**＝s424a 揭真池 865 ＋ 结构放行新增暴露 204：

| 页 | 案数 | 性质 |
|---|---|---|
| anchor-position-writing-modes-001 | 864 | s424a 同池：writing-mode × anchor 组合未建模伪过揭真 |
| zz-g36probe-wm001-drain | 1 | 同上金丝雀 |
| try-tactic-position-area | 131 | **新增**：try tactic 翻转面变异（非 class 族），class 门时代闭臂 |
| position-area-in-position-try | 64 | **新增**：同上组合面 |
| position-area-value | 7 | **新增** |
| position-try-tree-scoped | 1 | **新增** |
| zz-g6probe-mut6 | 1 | 探针 |

新增暴露 204 案全部为 position-try / position-area 变异依赖页——即 s424a
定性「等值语义页」之外的变异真读面，其真值与快照陈旧读分叉。

## 3. 目标页定谳：15F 清偿证伪

anchor-name-mutation **15F → 15F**（案级一一对应），但失败形态质变：
`expected 0 but got 1` / `expected 100 but got 101` / `expected 200 but got
201`——三个断言轴全部恒 +1。即结构性放行后同 turn 读已走活值（不再是
"got 0/陈旧" 形态），**变异后重布局已在跑，但重布局几何与期望恒差 +1px**。
剩余根因在 layout-engine 锚重布局几何（初布局与重布局 1px 漂移——舍入或
参照盒口径），归 VRL in-flow 布局刀挂账域（layout-engine anchor_pos peer
活跃在途），本流不追。

## 4. 回退决策

净 −1065（比 s424a 拓宽 −862 更差：额外揭真 204 无建模承接）且目标 15F
零清偿；s423 收窄为已确立约定。callbacks.rs 回 HEAD（零残留实证：`git diff
HEAD -- js_dom_bridge/` 空）。**s423 family 谓词两轮实验定谳收口：class 拓宽
与结构放行均证伪，同 turn 读门维持收窄态；清偿路径唯一 = VRL in-flow 布局
建模落地后重开（届时揭真桶 204+865 转真增益、+4 同回）。**
