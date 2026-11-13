# R5514 轮次证据：inline 壳 CB 透明——split inline 壳不 reset 百分比高 definite 链

日期：2026-11-13
轮次：R5514（corpus FAIL 盘点择案轻量，R5513 后继）
择案：css-tables/percent-height-replaced-in-percent-cell-004（✗2.08%，账本仅 r4001
「逐案独立域待归因」白名单提及，002/003 兄弟案全绿）
结论：根因＝table-cell > split inline 壳 > block canvas height:200% 形下，R695 % 高
walk 在壳（span，display:inline、height:auto）reset `cb_definite` → canvas %
compute-to-auto + attr 固有 1×1，绿面整缺；修复＝walk 对 display:inline 非替换中间层
CB 透明（CSS2 §9.2.1.1/§10.1：inline 非块容器、永非后代盒级 % 解析的包含块），链经
壳时 walk 代写 px（taffy 原生 % 基准是直属父＝壳 auto 高，不可达链上块容器）。
004 ✗2.08% → ✓0.00%；家族 4/4 全绿。

## 一、缺陷签名与取证

- 案面：cell(height:50px) > span > canvas(display:block, height:200%, 1×1 attr) +
  第二 cell div(height:100px)。chromium：200%×50px=100 高、1:1 ratio 推宽 100 →
  绿方块 100×100。ZW 差值 2.08% = 10000px² = 绿面整缺（PNG bbox 全空扫实证）。
- 因子分离（零编译 product-smoke 探针矩阵，/tmp/r5514/）：
  - probeB（block canvas 直入 cell，无壳）：✓ 100×100——cell 尺寸管线无恙。
  - probeC（inline canvas 在 span 内）：✓ 100×100。
  - probeD（span 壳 + canvas height:100px 定值）：✓ 100×100——壳形树位 px 高渲染
    正确（width ratio 推导正常）＝walk 代写 px 可行的实证。
  - probeE（span 壳 + canvas height:50%）：✗ 1×1 塌——walk 改写 auto+attr 链确认。
  - probeF（plain div(50px) > span > block canvas 200%）：✓ 100×100——plain 树
    R1044 纯块子形 span 无盒（LAYOUT_DUMP 实证 div > canvas 直连），不经过本缺陷。
- LAYOUT_DUMP（runner layout-dump 子命令）：004/E 形树 = cell > span 壳 > [canvas]，
  壳 h=1＝canvas 塌 1×1；F 形树 = div > canvas 直连无壳。delta＝容器类型（cell 保
  壳 vs plain 跳壳），与 span 内容无关。

## 二、归因链

1. R109 split（tree.rs compute_inline_block_split）：cell 路径的 split inline 保壳
   （taffy/LayoutBox 均为壳 > 片段/canvas），canvas taffy 直属父 = 壳。
2. R695 % 高 walk（sizing.rs apply_indefinite_percent_height_to_auto）：壳有样式
   （height:auto）→ `other` 臂 resolve=None → `my_definite=None` → canvas Percentage
   臂 cb_definite=None → standards else 臂 compute-to-auto + attr 固有 1×1。
3. spec 口径：CSS2 §9.2.1.1 + §10.1——inline 盒不是块容器，后代盒级解析的包含块是
   最近块容器祖先（= cell）。壳 reset 本身违背 CB 链语义（与无样式匿名盒透传口径
   不一致）。
4. taffy 原生 % 解析基准 = taffy 直属父（壳，height:auto 内容定高）——仅靠链透传
   不能修复（indefinite 基准循环），需 walk 代写 px 钉死（probeD 实证该树位 px 高
   渲染正确、width 由 ratio 推导）。

## 三、修复（sizing.rs walk 三处）

- 透明集判定 `self_is_inline_cb_transparent`：display:inline + 非替换 + float:none
  + 非 abs/fixed + **非 flex/grid item 语境**（item 块化，R2091/R5346 臂依赖 styled
  块照常入账；canvas UA display=inline 的替换元素自身高度解析臂不得跳）。kill-switch
  `ZW_INLINE_CB_TRANSPARENT=0`。透明时跳过 styled 块 → `my_definite` 保持
  `cb_definite` 透传（对齐无样式匿名盒口径）。
- Percentage 明确 CB 臂：`cb_through_inline=true` 时 walk 代写
  `Dimension::length(resolved)`（直连父场景不写，taffy 原生解析同值——probeB 口径）。
- 子代传播 `child_cb_through_inline`；内层 walk 新增尾参，根调 false。

## 四、锚测试

`r5514_inline_cb_transparent_tests`（3 案）：①004 案形 canvas 100×100（驱动）；②
plain 无壳形 50%×50px=25 不变（cb_through_inline=false 回归守卫）；③inline-block
（块容器 auto 高）内 50% 塌 0 不变（非透明语义守卫）。红态证据：kill-switch 关闭
时①红（canvas 1×1）②③绿——恰为旧行为签名。

## 五、验收

- lib：**2498P/0F**（2495 + 3 新锚；中途 3F＝item 语境/canvas 替换元素误判透明，
  分别以 `!parent_is_flex_grid`、`!b.is_replaced` 排除后归零）。
- solo：percent-height-replaced-in-percent-cell 家族 4/4 全绿（004 ✗2.08% →
  ✓0.00%；002/003/tentative 维持）。
- corpus 全量 A/B（16594 vs /tmp/r5513 轮 687F 重格式化）：**FIXED={004}+表族互翻
  5 案落绿相（127/152/153/154/border-collapse-dynamic-002，R5418/R5420/R5421 在册
  带）REGRESSED=0**——表观 4 红（table-anonymous-objects-128、run-in-inline-block-
  between-003、selection-input-011、intrinsic-percent-replaced-dynamic-007）solo
  全 ✓0.00-0.30%＝批量互翻 flake 态，非本轮因果。
- make reftest：725/725（521 layout + 204 text）。
- make test：见 /tmp/r5514/make-test.log（后台跑，交付前核账）。
- product-smoke：welcome chromium oracle **17.23%**（R5501 基线同值）+ struct 全
  PASS + form-input-perf PASS；legacy 42 fixture struct-check **0 失败**。
- fmt（零 diff）+ clippy `--all-targets -D warnings` 净。

## 六、本轮动作与卫生

- whitespace-reattach（css-grid，2.08%）探针排除：静态终态两变体（空白/非空白文本
  + 绿 div）product-smoke 双 ✓＝静态语义正确，红来自 splitText/replaceChild 增量改
  排域 → 按 flex-wrap-006「维持动态域」先科排除。
- 候选盘点：css-flexbox 桶 R5410-R5412 已全处置（flex-wrap-006 动态域/contains-size
  多特性域）；css-overflow/css-grid/css-contain 带扫无新零账本候选；004 经 r4001
  evidence 案名 grep 命中（首轮只查主控面漏 evidence 目录——简称面教训同 R5412）。
- 临时探针全走零编译（product-smoke + layout-dump），无源码残留；探针文件留
  /tmp/r5514/（tmp 域）。
