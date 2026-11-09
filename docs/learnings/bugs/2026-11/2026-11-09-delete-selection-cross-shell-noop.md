---
date: 2026-11-09
modules: engine
---

# deleteSelection 跨壳边界选区静默空转：收集器两道叠加缺陷

## 问题描述

`foo<sub>[bar</sub>baz]` 形选区（起点在壳内文本、终点在宿主文本）执行 `delete` 后树原样不动——四个 F6b 域键（N4/N5/N6/W2）与任意 inline 壳（含 `<font>`）跨边界选区全部空转；壳内完整选区（`<font size=2><sub>[bar]</sub></font>`）反而正常。

## 根因分析

`_zwJusCollectContainedNodes`（`crates/engine/src/js_dom_shim/part01c.js:393`）两处叠加：

1. **stop 哨兵 null 弃权**：`stop = NextNodeDescendants(endContainer)` 在端界落宿主链顶时向根爬升返回 null，循环卫兵 `node && stop` 把 null 当"无后续"，整个收集直接放弃。
2. **含界判定过严**：复用的 `_zwJusIsContained` 是严格版（`bpS > 0 && bpE < 0`），把与选区端界等值的节点排除在外。起点恰在壳界（`<sub>` 内文本头）、终点恰在宿主界的节点全部落榜——这正是跨壳选区的典型形状。spec [editing #contained](https://w3c.github.io/editing/execCommand.html#contained) 语义是含界（不早于 range 起点、不晚于终点）。

## 解决方案

收集循环内就地展开含界比较（`bp >= 0 && bpE <= 0`），卫兵改 `(!stop || isBefore(node, stop))`——stop 为 null 时迭代至树尽（界外溢收集不存在）；`_zwJusIsContained` 严格版保留给其余四个语义确实要排除边界的调用面。教训：NextNodeDescendants 的 null 返回是"到根"不是"没节点"，哨兵不要直接充当循环终止条件；共享谓词换调用面时先核语义口径，收集/枚举类语义在 editing spec 里几乎都是含界。
