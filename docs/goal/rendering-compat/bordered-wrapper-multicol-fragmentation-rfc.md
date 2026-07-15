# RFC: bordered-wrapper multicol fragmentation（column-span:all 全宽 + border-skip）

**日期**: 2026-07-15（R1473）
**目标案**: `multicol-span-all-children-height-006`（17.75%）+ 可能 `008`（border:20px）等同族 bordered-container + column-span 案
**规范**: CSS Multicol §6.1（column-span:all spanner 跨全宽，拆列流）+ css-tables fragmentation（容器 border 在 spanner 相邻边 skip）

## 1. 问题（R1473 PIL 实测）

`multicol-span-all-children-height-006`：`article{column-count:2 width:400} > container{height:250 border:20px solid purple margin:1em 0} > [block(200) + column-span(50) + block(200)]`。

chromium（ref 用拆分多 container + 选择性 border 模拟）：
- column-span:all spanner **跨全宽 400px**（lblue x[8-407]）。
- container 被 spanner 拆成 2 region（spanner 前后），每个 region 是独立 bordered box。
- container 的 purple border 在 **spanner 相邻边 skip**（region1 border-bottom:none，region2 border-top:none）—— spanner 横穿处 border 中断。

ZW 当前（17.75% diff）：
- spanner **仅 111px 宽**（lblue x[236-347]），未跨全宽。
- container 未被拆分（单 bordered box），border 全周绘制。
- 布局紧凑（y 8-168）vs CHR（y 8-275）。

## 2. 根因

`multicol.rs::try_layout_nested_spanner` 的 `no_box` gate（line 371-376）**排除 bordered/padded wrapper**：
```rust
let no_box = wrapper.border_top < 1.0 && wrapper.border_bottom < 1.0
    && wrapper.border_left < 1.0 && wrapper.border_right < 1.0
    && wrapper.padding_top < 1.0 && wrapper.padding_bottom < 1.0;
```
注释（line 366-368）：「no_box：无 border/padding（避 styled wrapper 如 008 border:20px；bg/border 须分列铺，首版未实现）」。

故 006 的 bordered container 不进 synthetic fragmentation → spanner 不被提升为全宽 + 内容不拆 region → 渲染错。R1341-R1361 仅修了 no-border 族（004a/004b），bordered 族（006/008）留此 RFC。

## 3. 设计

### 3a. layout 侧：扩 no_box gate 允许 bordered wrapper 进 synthetic fragmentation

`try_layout_nested_spanner`：把 `no_box` 拆为两个信号——
- `no_box`（既有，no-border fast path，004a/004b 行为不变）。
- `bordered`（新）：wrapper 有 border/padding 但满足（i）direct-child-spanner（enable_painter_core，同 R1352 gate）（ii）definite height（同 R1357）（iii）border 各侧 < 阈值（如 40px，避过厚 border 几何复杂）。

bordered wrapper 进 synthetic clone 时**保留 border**（synth.children clone 不剥 border），layout_multicol_with_spanners 在 synthetic 上照常区域分割 + spanner 全宽。回填 x/y/cso 同 R1341（坐标减 dx/dy，含 border 偏移）。

### 3b. painter 侧：bordered-wrapper border 分段（spanner 相邻边 skip）

container 的 border 现由 `paint_borders`（mod.rs:666）按完整 box 全周绘制。bordered-wrapper fragmentation 下须**按 region 分段**：
- 从 wrapper 的 column-span 子（spanner）Y 区间推 region 边界（同 paint_column_rules 的 spanner_ranges，text.rs:268）。
- region1 border：top/left/right 绘，**bottom skip**（spanner 上相邻）。
- region2 border：left/right/bottom 绘，**top skip**（spanner 下相邻）。
- 仅 1 spanner → 2 region；N spanner → N+1 region，中间 region 上下均 skip。

实现点：painter 检测 `wrapper.is_nested_spanner_wrapper && wrapper.has_border`（新 flag），调分段 border 绘制（复用 paint_borders 的 per-side 逻辑，gate 各侧 by region）。或更简：layout 侧把 bordered-wrapper 的 border 信息存入 `nested_spanner_col_bg`-like 结构（每 region 的 border 侧集合），painter 按之绘。

### 3c. container/wrapper 高度（同 R1357-1361）

bordered wrapper 的 effective height = Σ region heights + spans（R1357 section_content/col_count 模型，bordered 版须含 border_inset）。article content_height = content_extent（R1358）。

## 4. 验收

- **单案**：`multicol-span-all-children-height-006` oracle < 1%（flip），PIL 核：spanner 全宽 400 + purple border region 分段（spanner 相邻边 skip）。
- **A/B**：`make reftest-oracle DIR=css-multicol` 全量 net ≥ 0（ORACLE_DUMP_ALL per-case）。重点回归：004a/004b（no-border 族须不退）、multicol-span-all-017/parallel-flow（R1341 回归规避案）、column-height-013（auto-height wrapper）。
- **product-smoke** welcome 不变。
- **gates**：fmt + clippy `-D warnings` + make test 全绿。

## 5. 风险 / gate

- **bordered synthetic clone 几何**：border 偏移（dx/dy 含 border）须正确回填，否则 cso/位置错。R1341 no-box 假设 dx/dy = wrapper content origin（无 border）；bordered 版 dx/dy 须含 border_left/top。**高回归风险**——须 006 单案 PIL 逐步验证 + 全量 A/B。
- **painter border 分段**：新代码路径，须不破坏非 spanner 容器的 border（gate `is_nested_spanner_wrapper && has_border`）。
- **border 阈值**：过厚 border（>40px）几何复杂，首版 gate 排除（避 008 border:20px 之外的极端案）。
- **multi-spanner**：3+ region 的中间 region 上下 skip，须正确迭代。
- kill-switch `ZW_BORDERED_WRAPPER_FRAGMENTATION=0`（default-on 前 A/B 证 net≥0）。

## 6. 实施顺序（多 session）

1. layout 侧扩 gate（bordered wrapper 进 fragmentation，spanner 全宽）—— 006 spanner 全宽先对，border 暂全周（diff 部分降）。
2. painter border 分段（spanner 相邻边 skip）—— 006 border 对，flip。
3. 全量 A/B + gate 收紧 + kill-switch default-on。
