# R5308 — float BFC 根不与子折叠 margin（CSS2 §8.3.1）——floats-rule3/7-outside-{left,right}-002/001 ×4 回收翻绿 + margin-collapse-through-percentage-padding、clip-text-descendants 附带翻绿，corpus 15743P/849F→15750P/842F 零意外翻转（taffy-local 一处枚举常量）

**日期**: 2026-10-28
**驱动案**: `css/CSS2/floats/floats-rule3-outside-left-002.xht` 等 rule3/7-outside 家族 4 案（R5307 挂账 ref 侧阻塞）
**corpus**: 两次独立复跑 **16592 / 15750P / 842F**，失败集逐案一致。

## 定性（R5307 下一步①float 根 margin 折叠命中）

R5307 定谳的 ref 侧阻塞：root float 500×500 → 块子 `margin-top:300`，chromium 子渲染在
内容系 y=300，ZW 子渲染在 y=0——mt 字段完好（300）但位置为 0，auto 高 float 根高只剩
子高（引擎级复现钉 `r5308_float_root_child_margin.rs` 修复前双红：子 y=0、auto 根高 10）。

根因（taffy-local `compute/block.rs` float 分支）：float 盒自身布局以
`perform_child_layout(..., Line::TRUE)` 执行——`vertical_margins_are_collapsible.start=TRUE`
流入 `compute_block_layout` 的 `own_margins_collapse_with_children.start`（float 根无
padding/border、position=Relative 满足其余条件）→ 首/末流内子 margin 被**上提**进
`first_child_top_margin_set` / `last_child_bottom_margin_set`（子 y=0 落位），而 float
分支只消费 `item_layout.size` 与 float 自身 margin（`margin_box = size + own margins`），
上提值**无人接收整体丢失**。`Line::TRUE` 系 taffy 0.7.7→0.10.1 迁移提交（40f9b55cd）
携入，非校准值。

CSS2 §8.3.1：float 建立 BFC，「Margins between a floated box and any other box do not
collapse (**not even between a float and its in-flow children**)」——float 根与子的
折叠旗标须为 `Line::FALSE`，子的 mt 留在根内作局部偏移（`y_margin_offset` 路径，
placement_y = committed + 300），auto 高根内在高度含子 mt。

## 实现（一处枚举常量 + 注释，无新 kill-switch）

`compute/block.rs` float 分支唯一 `perform_child_layout` 调用：`Line::TRUE` →
`Line::FALSE`。行为面：全部 float 根的首/末流内子 margin 从「上提丢失」改为「保留根内」
（§8.3.1 正确语义）；float 根自身 margin（`item_non_auto_margin`）消费不变；
R5211 clearance 路径（`clear_pos`/`FLOAT_CHILD_COLLAPSE_THROUGH`）不经过此旗标，无交互。
上游 taffy 无 float 支持，该分支系 ZW 扩展（float_layout feature），vendored 上游语义
测试不覆盖、无回归面。

## 交付

- 驱动家族 **×4 免改 test 侧直接翻绿**（R5307 预测兑现）：
  floats-rule3-outside-left-002、floats-rule3-outside-right-002、
  floats-rule7-outside-left-001、floats-rule7-outside-right-001 全部 ✓0.00；
  rule3/7-outside-*-001 墙模型回收案维持绿。
- 附带翻绿 2 案（因果可溯）：`margin-collapse-through-percentage-padding`
  （float 根 + collapse-through 域直击，隔离复跑 ✓0.00）、`clip-text-descendants`
  （`.clip-text` 内 float:left 子，auto 高根，隔离复跑 ✓0.41%）。
- 全量 corpus 两次独立复跑 **16592/15750P/842F**，失败集逐案一致；归一化 diff vs
  R5307 定版 849F 恰 = {−4 驱动家族， −margin-collapse-through-percentage-padding,
  −clip-text-descendants}，**零意外翻转、零新增红**。账本口径：849−6=843 预期 vs
  实际 842——`multicol-fill-auto-block-children-002`（✓0.23%，页内**零 float** 无修复
  因果路径，近阈翻转）按 flake 记账不计回收（R5299 content-none-select-1 同判例）；
  `size-adjust.tentative`（R5303 flake）本轮通过态、`content-none-select-1` 本轮通过态，
  双向摆动维持既有记账。
- 回归测试 +2（`r5308_float_root_child_margin.rs`，引擎级 LayoutEngine API）：
  ①定高 float 根（ref 页形态）子 y=300 ②auto 高根高 310（含子 mt）。修复前双红
  （y=0 / 高 10）证据在案。

## 门禁

- layout-engine 全套 **2206P/0F**（lib 2189——含对端流 rebase 带入 s122 +4——+ r5307 ×5
  + r5308 ×2 + r4543 ×4 + 定向 ×5+1 等）。
- fmt 0 diff；clippy zero-layout-engine `--all-targets -D warnings` 0 告警。
- product-smoke struct ×6 全 PASS + form-perf PASS；welcome oracle **15.68%**
  （+0.18pp vs R5307 15.50%）——**A/B 归因对端流**：stash 本修复复跑同值 75254px
  （byte-identical），welcome.html 零 float 无因果路径；位移来自 rebase 带入的
  50569142e（s122 html-semantics classic script 编码链，对端流域），移交对端知悉，
  非本流回归。
- bench-gate 定向 zero-layout-engine：WIP 跑 GATE FAIL 7 指标（NEW=0），量级
  3.4–3.7×（incremental 10.3ms vs 基线 3.0ms、page/medium layout 456ms vs 123ms）——
  单枚举常量替换零指令增量，物理不可达，负载签名（当窗 loadavg 8.5→19）；clean 树
  stash A/B 复跑被守卫 ABORT（loadavg 19 > 阈值 12，对端流连轴 sweep）。按
  R5299/R5303/R5307 判例挂账空闲窗复跑定版，零 relax 零基线改动。
- make test 全量未跑（改动面 taffy-local float 分支一处枚举常量 + 同域回归钉落
  layout-engine tests，消费面 corpus 两次全量 + product-smoke + lib 覆盖；
  taffy-local 自身 upstream 单测不含 float_layout 分支无覆盖价值，R5010/R5306 判例）。

## 挂账

- bench-gate 空闲窗复跑（本修复 + R5307 残余 flex +3% 两项并账；对端流 sweep 收敛后）。
- welcome oracle 15.68% 位移移交对端流（50569142e 归因证据在案；>16% 时升级碰头）。
- R5307 挂账全维持：同 walk 直接兄弟 float flat 形态叠压（Phase 1 结构轮）、earliest-fit
  kill-switch 回退行模型无 inherited 带集（FIXME 已注）、float_positioning.rs 3813+ 行
  超 2000 行规则、R5306 及既往挂账全维持。
- flake 账本 +1：multicol-fill-auto-block-children-002（近阈 0.23% 双向摆动，无 float
  域）。

## 下一步 R5309

①842F 账本续扫择轻（multicol ×余、featureless 家族、content-visibility slot ×2 等
待查池）②013 ellipsis host 传播评估（结构轮，R3768 先例，视账本轻案供给）③bench-gate
空闲窗并账复跑。
