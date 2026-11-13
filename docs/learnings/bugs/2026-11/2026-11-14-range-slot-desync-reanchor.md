---
date: 2026-11-14
modules: engine
---

# 序列上下文 range 位移槽与面值脱钩——同值公开 API 重锚修复

## 问题

strike P5 retag 臂（part01d td decl span 原位 retag 裸 `<s>`）在 face
语料 `?2001-last` multitest 窗口 apply 整段空转：retag 后选区折叠，
CONTAINED 集空，got `foo<s>bar</s>baz`。同一份代码的孤立探针页全绿，
无法复现（f6bwin 孤立/序列分叉同族）。

## 根因

两层叠加，且第二层只在序列上下文显形：

1. retag 子节点搬运走 `_zwJusMovePreservingRanges` + 宿主
   insertBefore/removeChild，move 补偿把 start 端点折叠进随迁节点
   （compensation 本身按规则工作，但结果对后续 apply 是毒）。
2. splitText 重绑后，range 的**内部位移槽**（`_endOffsetBase` 类）与
   (container, offset) 面值脱钩：读面返回旧基，快照/回写按面值操作救不
   回——首版仅做"快照 + retag 后回写"孤立页转绿、face 仍红，即此层证
   据。宿主 mutation（如诊断性 appendChild）会触发补偿重锚、顺带修好
   该脱钩——探针本身改变被探状态，构成 Heisenbug 修面。

## 解决

- retag 前 `rng.setStart/setEnd` 按快照**同值重写**：经公开 API 走一遍
  校验/槽刷新路径，位移槽与面值重新对齐；retag 重挂后按快照回写兜底。
  两写均同值，零语义变更。
- 与 d12 补偿缺失（[plain 树 mutator 缺 R263 插入补偿](2026-11-13-live-range-insert-compensation-plain-mutators.md)）
  互补：那条是补偿没接，这条是补偿接了仍在序列上下文脱钩；Accessor 槽
  直写（`_endOffsetBase`）与同值公开 API 重写是两个方向的修复面。

## 避免方式

- runner 无 console 桥时，面包屑经 `createTextNode` 挂 editing host，
  从 compare-innerHTML 断言的 got 串回传（唯一回传通道）；注意
  pre-retag 的诊断性 append 本身触发宿主补偿重锚——面包屑要打在关键
  读面之后，或用不落宿主的载体。
- 孤立探针绿不能为序列上下文背书：range 状态类修复必须跑 face 语料
  multitest 窗对拍；孤立红/序列红分叉先怀疑位移槽脱钩族。
- 对 live range 做 DOM 重排（retag/move/unwrap）前，先经公开 API 同值
  重写端点刷新槽一致性，再动树；改动后按快照回写兜底。
