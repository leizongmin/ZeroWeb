---
date: 2026-11-05
modules: engine
---

# br 可见性谓词互递归：extraneous ↔ isVisible 无界环

## 问题

editing 值命令 qV 走查（`__zw_ce_value_query` →
`_zwValGetAllEffectivelyContained` 逐 document 序节点调
`_zwValIsFormattable`）在树含 `<br>` 且端界后仍有节点时，qV 恒返空串。
临时把 catch 的异常 stash 出来才看到：
`RangeError: Maximum call stack size exceeded`，栈帧恒一
（`_zwJusNodeIndex ← _zwJusPrecedesLineBreak ← _zwJusIsExtraneousLineBreak
← _zwJusIsVisible ← _zwJusIsBlockStartPoint`）。

## 根因

spec 形谓词互相引用成环，且每层调用局部 guard（256 步上限）不约束
**调用深度**，只约束单次游走长度：

1. `isVisible(br)` → br 臂 → `isExtraneousLineBreak(br)`
2. → `precedesLineBreak(br)`：br 无子级 → 爬升
   `offset = NodeIndex(br)+1; n = parentNode`
3. → while 条件 `isBlockBoundaryPoint(parent, brIdx+1)`
   → `isBlockStartPoint` 检查 `childNodes[brIdx]`（恰是该 br）
   的可见性 → **回到 1 重入，无界**

关键陷阱：手建同形树复现不出来——只有当走查真的走到该 br 才触发
（走查 `stop = NextNodeDescendants(endContainer)`，端界是树尾时 stop
为 null，循环不执行，br 永不被问）。所以 A/B 手工探针「有 br 不炸」
不能证明谓词无环；必须让选区端界后方留后继节点。

## 解决与避免

- 重入印记 + 候选跳过：`isExtraneousLineBreak` 进入时给 br 打
  `__zwExProbeInFlight` 印记（重入即返 false），`isBlockStartPoint`
  把带印记的 br 子级**不作 boundary 候选**（视为不存在）——precedes
  爬升后转而回看更前节点取真实语义（`o<br>bar` 形逐字 trace：跳 br
  → 假 → 继续 `childNodes[2] = bar` 可见 → precedes=false → br 可见 ✓）。
  语义不变、递归深度有界。`isBlockEndPoint` 无需同改（br 非块节点，
  其臂恒 false）。
- 模式总结：spec 的 visible/extraneous/block-boundary 三元组互相
  定义，任何「谓词 A 调 B、B 爬升后又问 A 所判对象」的形状都要显式
  断环；迭代 guard 只防横走，不防纵深入。
- 对外暴露面上，catch 兜底（qV 空串）会把这类栈溢出吞成「合法空值」，
  大面积 F 与真值面缺损不可区分——排查查询面怪异空串时优先怀疑
  被吞异常（临时 stash 异常文本是低成本定位手段，用后即还原）。

## 关联

`crates/engine/src/js_dom_shim/part01c.js` `_zwJusIsVisible`/
`_zwJusIsExtraneousLineBreak`/`_zwJusIsBlockStartPoint`/`
_zwJusPrecedesLineBreak`；案例详录
`docs/goal/editing-compat/evidence/2026-11-05-qv-peel-asym-recursion-break.md`。
