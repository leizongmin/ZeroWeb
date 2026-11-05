---
date: 2026-11-05
modules: engine, js_dom_shim
---

# registry indexOf 快道的「登记序 = DOM 序」前提在部分覆盖视图下静默失效

## 问题

WPT `Range-mutations-appendChild`/`insertBefore` 尾部插入 8 例恒红
（expected offset 1 got 2）：`paras[0].appendChild(paras[1])` 后，边界点
恰在插入位次（offset == DOM 插入 index）的 live range 被 bump 一格。
R263 `_zwAdjustRangesForInsert` 的比较本身是 spec 正确的
（`offset > newIndex` 才 +1），探针装饰显示调用方视图 newIndex=1、
offset=1，不该 bump 却 bump 了。

## 根因

`ensureNewIndex263` 的 O(1) 快道用 `_handleChildren[handle].indexOf(node)`
取 newIndex，前提注释「登记序 = DOM 序」。该前提只在 registry **完整覆盖**
融合视图子时成立；`textContent=` 建的 textEl 文本子不进 registry（物化前
是视图-only 成员），混装箱 `[textEl 文本子, handle 子]` 中 registry 是视图
的**子集**——append 后 registry=[a1]、regIdx 0，而 DOM idx 1。快道的
fallback 条件写的是「registry 缺该子才退 live NL 扫描」，漏掉「registry
**有**该子但位次错」形态。于是 `offset(1) > regIdx(0)` 误成立，+1 到 2。
R262 `_zwAdjustRangesForRemove` 是同一条注释、同款快道、同款 fallback
条件的对称面（remove 侧 oldIndex 同样可取到 regIdx 0），属同根因潜伏。

共性：**位次型快道的「registry 序 = 视图序」假设，在视图含非 registry
成员（部分覆盖/子集视图）时静默失效**——两种失效形态（成员缺席 / 成员
在但位次错）只防了第一种。

## 解决

两快道加**完整序前提校验**：registry 子数 == 融合视图子数
（`newParent.childNodes.length === reg.length`）才信 indexOf，不齐退回
原有 live NL 扫描 fallback。要点：

- 校验放快道内、fallback 不动——完整覆盖容器照走 O(1)，部分覆盖形态本就
  该走 O(n) 权威路径，只是把「成员缺席才退」修成「视图不齐即退」。
- `childNodes` 读是该函数既有 fallback 路径的同款读，只把时机提前，无新
  副作用面；惰性化门（preHit 前筛）保证无 range 锚定时零成本。
- 对称面同轮修：R262/R263 是复制繁衍的对偶实现，一处前提破裂即两处同病，
  只修观测到红的一面会留下潜伏翻转。

## 验证与避免

- 探针法定位：包装 `globalThis.__zwAdjustRangesForInsert/Remove` 记录每次
  调用前后 `_startOffsetBase`/`_endOffsetBase` 与调用方视图位次，一次跑出
  「视图 ni=1 而函数内 bump」的直接矛盾，避免在读码上耗尽。
- 以后在 js_dom_shim 引入「registry 序当视图序」的位次快道时，必须同处
  写下完整序校验或显式断言视图构成；`_handleChildren` 与融合视图
  （`_zwLocalChildNodes` + registry + pending overlay）的成员差是常态而
  非异常（textEl 未物化、M 域解析代理等形态）。
- A/B 验证口径沿用：dom 通道逐子测试集合差（0 Pass→Fail、0 新增 Fail），
  截断深度波动案只看集合差不看墙钟。
