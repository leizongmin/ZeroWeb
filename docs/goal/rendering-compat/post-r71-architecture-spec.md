# Rendering Compat Post-R71 专项改造 Spec

**状态**: 待实施  
**适用阶段**: Post-R71  
**主控文档**: [master.md](./master.md)  
**目标**: 打破 `393/490 (80.2%)` 稳定天花板，明确 Post-R71 阶段专项架构改造的范围、顺序与验收标准。

---

## 1. 背景

R37-R71 共 35 轮增量修复已确认触及当前实现的阶段性天花板：

- 上游真实 reftest 稳定在 **393/490 (80.2%)**
- 内联 reftest 稳定在 **685/685 (100%)**
- 所有局部修补、override 追加、paint/layout 局部兜底路径均已穷尽

当前阻塞不是“再修几个 bug”，而是三类结构性缺口：

1. `taffy-IFC` 架构分裂：layout 与 paint 分别计算 inline 几何，导致文本定位、baseline、背景定位、absolute 静态位置长期分叉
2. `multicol` 内容跨列拆分缺失：当前只能按子盒高度切片或整体移列，不能按 line/fragment 真实延续内容
3. `writing-mode` 垂直布局不完整：已有部分视觉支持，但 float/clearance/static position 仍缺逻辑轴统一

本文件定义 Post-R71 阶段的实施范围、约束、顺序和验收标准。

---

## 2. 非目标

以下内容不属于本阶段：

- 继续扩展 paint IFC override map 作为主方案
- 重新尝试已回退的“直接启用旧版 `compute_final_inline_layouts()`”方案
- 修复与三大结构性瓶颈无关的零散失败用例
- 引入与当前目标无关的新渲染能力或抽象

---

## 3. 成功标准

### 3.1 总体成功标准

- 建立 **单一可信的最终 inline 几何来源**
- 让 `multicol` 支持 **line/fragment 级内容跨列分配**
- 让 `writing-mode` 的 float/clearance/static position 切换到 **逻辑轴求解**
- 全程保持内联 reftest **685/685** 无回归

### 3.2 分阶段成功标准

#### 阶段 A — Final Inline Layout Pass

- paint 文本主路径不再重跑 IFC 来决定 fragment 几何
- inline 背景、baseline、absolute 静态位置与文本共享同一份最终 inline 几何
- 相关定向测试通过，且全量上游 reftest 零大规模回归

#### 阶段 B — Multicol Fragmentation

- 单个长段落或超高 inline 内容可跨列延续
- `multicol-breaking-*`、`multicol-fill-auto-*`、`multicol-clip-*` 有可验证提升
- 不回退阶段 A 的几何统一

#### 阶段 C — Writing-Mode Logical Axis

- `vertical-rl/lr` 下 float exclusion、clearance、static position 走逻辑轴计算
- `direction-vrl/vlr-*`、`clear-clearance-calculation-vrl-*` 有可验证提升
- 水平 writing-mode 行为不变

---

## 4. 现状约束

### 4.1 已知代码事实

- layout 流程末尾的最终 inline 存储调用当前被注释：[`crates/layout-engine/src/engine.rs:193`](../../crates/layout-engine/src/engine.rs#L193)
- `LayoutBox.inline_layout` / `inline_layout_width` 已存在：[`crates/layout-engine/src/types/mod.rs:167`](../../crates/layout-engine/src/types/mod.rs#L167)
- 旧版 `compute_final_inline_layouts()` 已实现，但运行时机不对：[`crates/layout-engine/src/engine.rs:1189`](../../crates/layout-engine/src/engine.rs#L1189)
- paint 侧已有 stored-path，但当前只作旁路：[`crates/engine/src/paint/painter/text.rs:784`](../../crates/engine/src/paint/painter/text.rs#L784)
- multicol breaking 当前按子盒高度切片：[`crates/layout-engine/src/multicol.rs:326`](../../crates/layout-engine/src/multicol.rs#L326)

### 4.2 已证伪路径

以下路径已在 R37-R71 中被反复验证不可作为主方案：

- 给 paint IFC 继续补充更多 override map
- 直接传完整 styles 给 paint IFC
- 单纯恢复旧版 `compute_final_inline_layouts()`
- 依赖 measure callback 缓存结果作为最终 fragment 几何
- 通过 margin override 解决当前主瓶颈

结论：问题根因不是“override 不够”，而是 **layout 与 paint 同时拥有 fragment 决策权**。

---

## 5. 实施总原则

1. **单一事实来源**：fragment 几何只能由 layout 最终阶段产出一次，paint 只消费。
2. **后处理后统一**：所有 final inline 几何必须在 taffy 主布局、remeasure、table/multicol 后处理之后生成。
3. **分阶段提交**：A/B/C 三阶段必须可独立验收和回退，不允许合并成一次大爆炸改造。
4. **零散失败不插队**：除非直接阻塞 A/B/C 之一，否则不处理与专项目标无关的单测或 reftest。
5. **结果可验证**：每一阶段都必须有独立的定向验证和全量验证入口。

---

## 6. 方案 A：Final Inline Layout Pass

### 6.1 目标

消灭 `taffy-IFC` 双轨，建立最终 inline 几何的唯一来源。

### 6.2 设计

在 layout 主流程末尾新增或重构一个 **final inline layout pass**：

- 执行时机必须晚于：
  1. taffy 主布局
  2. float remeasure / sibling reflow
  3. table 后处理
  4. multicol 后处理
- 该 pass 为所有需要的容器生成最终 `inline_layout`
- paint 不再用 IFC 决定 fragment `x/y/width/height`

### 6.3 数据要求

`inline_layout` 必须从“可选缓存”升级为“最终几何结果”。每个 fragment 至少需要：

- `node_id`
- `text`
- `x`
- `y`
- `width`
- `height`
- `font_size`
- `is_ahem`
- `baseline_y` 或等价基线偏移

如果 paint 仍需从 fragment 推导 advance/spacing，补充：

- `letter_spacing`
- `word_spacing`

### 6.4 代码改造边界

允许修改：

- `crates/layout-engine/src/engine.rs`
- `crates/layout-engine/src/types/mod.rs`
- `crates/engine/src/paint/painter/text.rs`
- 与 inline 背景、absolute static position、baseline 直接耦合的消费点

禁止作为主方案修改：

- 持续膨胀 override map 数量来模拟最终几何
- 在 paint 阶段重建另一套 fragment 布局

### 6.5 验收

定向验证：

- `inline-formatting-context-*`
- `baseline-*`
- `border-padding-bleed-*`
- 涉及 absolute static position 的相关测试

全量验证：

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run --bin zero-wpt-runner -- reftest`

---

## 7. 方案 B：Multicol Fragmentation

### 7.1 目标

让单个超高 block child 的文本内容按 line/fragment 跨列延续，而不是按整块高度裁剪。

### 7.2 设计

扩展 `assign_children_to_columns_with_breaking()` 的输入与输出语义：

- 输入不再只看 `(child_idx, child_height)`
- 对 oversized child，读取其最终 `inline_layout`
- 按 line box 或 fragment range 进行列分配

`ColumnFragment` 应能表达：

- `child_idx`
- `line_range` 或 `fragment_range`
- `source_y_start/source_y_end`
- `visual_height`

### 7.3 关键实现原则

- 优先支持 inline-only 或 text-dominant block child
- paint 只绘制该列对应的 line/fragment 切片
- 避免继续沿用“整块重画 + clip”作为最终语义

### 7.4 验收

定向验证：

- `multicol-breaking-*`
- `multicol-fill-auto-*`
- `multicol-clip-*`
- `multicol-containing-*`

全量验证：

- `cargo run --bin zero-wpt-runner -- reftest`

---

## 8. 方案 C：Writing-Mode Logical Axis

### 8.1 目标

让垂直 writing-mode 下的 float、clearance、static position 从物理坐标分支逻辑切换到逻辑轴统一求解。

### 8.2 设计

在相关布局路径中引入逻辑轴抽象：

- `inline_start`
- `inline_end`
- `block_start`
- `block_end`

计算顺序：

1. 在逻辑轴中计算 float exclusion
2. 在逻辑轴中计算 clearance / static position
3. 再统一映射回物理 `x/y/width/height`

### 8.3 关键实现原则

- layout 逻辑化，paint 物理化
- 不再在各处分支手工做 `x/y` 互换
- 水平模式必须共享同一套逻辑轴框架，避免垂直模式成为旁路实现

### 8.4 验收

定向验证：

- `direction-vrl-*`
- `direction-vlr-*`
- `clear-clearance-calculation-vrl-*`
- 相关 orthogonal float/clearance 测试

全量验证：

- `cargo run --bin zero-wpt-runner -- reftest`

---

## 9. 推荐实施顺序

1. **阶段 A：Final Inline Layout Pass**
2. **阶段 B：Multicol Fragmentation**
3. **阶段 C：Writing-Mode Logical Axis**

原因：

- 阶段 A 解决“fragment 几何双轨”的根问题，是后两项的共同前提
- 阶段 B 直接依赖最终 inline 几何，且收益明确
- 阶段 C 影响面最广，适合在 A/B 稳定后推进

---

## 10. 最终结论

Post-R71 阶段不是继续做增量修补，而是进入以三项结构性改造为核心的专项实施阶段：

- 阶段 A：统一最终 inline 几何来源
- 阶段 B：实现 multicol 的 line/fragment 级内容跨列拆分
- 阶段 C：完成 writing-mode 的逻辑轴布局求解

只有按上述顺序推进，才有机会系统性突破当前 `393/490` 的稳定天花板。
