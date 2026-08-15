# R51c — pending 差量表 O(n²) 膨胀消除（Range-mutations 三文件超时根因）

**日期**: 2026-08-15
**Commit**: `81ca25b5`
**里程碑**: M4（WPT dom 基线）+ M1 方向（per-op 成本诊断直指 L2）

## 背景

R51b 修复死循环后，Range-mutations-dataChange/insertBefore/replaceData 三文件 300s/560s 仍超时（其余 mutations 族文件已可跑完）。

## 诊断过程（方法可复用）

1. **增速探针**：1500 次 setupRangeTests 每 250 次打点 → 5s→10.4s→16.3s→22.6s→29.8s **线性增速**（O(n²) 确诊）；
2. **步骤分解探针**：query/remove/createElement+append/textContent 分段计时 → **append 段占 915→977ms/250 且增长**；
3. **内部表窥视**（临时 diag 暴露 `__zwR51Diag`）→ pa（pending added）**+19/subtest**、textEls +5、桶 Map +6 桶；
4. **invalidate 调用日志**（`__zwDiagLog`）→ 第 2 次 setup **只有 add 无 remove**——`querySelector('#test')` 返 null（pending 旧树对 host 查询不可见）→ setup 跳过 removeChild → 整树泄漏；
5. **最小案例**（append→append→remove + pa 计数）→ 对冲机制逐项验证（remInPA identity 全 true 但表不降 → 时序/读取位置陷阱排除后定位到 setup 侧根因）。

## 五项修复

| # | 修复 | 语义 |
|---|------|------|
| 1 | **消零语义** | pending-added 节点被 remove → add+remove 对冲为零，**不入 removed 表**（host 快照从未见过它，removed 条目剔除恒 no-op——旧实现无条件 push，每 subtest 泄漏 ~30 死 proxy） |
| 2 | **pending 按父分桶** | `_zwPendingByParent: Map<parentSel/_h:handle, {added, removed, addedSet, removedSet}>`——childNodes overlay 读 O(桶)（旧全表扫）；invalidate 记账带 mutSel/mutHandle（part01 汇流点传参扩展） |
| 3 | **removed 表 512 软上限压实** | handle-only 条目（无 `__zwSelector`）对 host 快照**结构性不可见**（快照条目皆有 selector）→ 纯死数据，溢出时一次性丢弃 |
| 4 | **querySelector('#id') pending-id 回落** | host 快照 miss 时按 `_zwPendingAddedById` 索引回落（WPT setup 模式 `querySelector('#test')` 取旧树 removeChild 的可见性前提）；id 索引在 invalidate 记账时维护（O(1) 回落，不引入新全表扫） |
| 5 | **registry/注销补齐** | `_appendVariadic` 补 `_recordHandleChild`（`append('text')` 建的文本子此前对 subtree collection 不可见→永不对冲）；`_zwUnregisterTextSubtree`（removeChild 子树注销 `_zwTextEls`，经 childNodes 融合视图递归——part06 顶层全局作用域够不着 IIFE 私有 `_handleChildren`）；`_zwHCCollectSubtree` registry 空时回落本节点 pending 桶 |

## 结果

- **增速消除**：append 路径 929ms/250（增长）→ **恒定**（perf2 分解复测）；
- **净增**：dom/nodes 3049→3096P（+47，querySelector 回落解锁）、dom/traversal 925→953P（+28）；
- **零回归**：dom/events 189P、dom/collections 48P/0F、mutations 族 7 文件 586P/1010F 与 R51b 完全一致；
- engine v8 **2140** / quickjs **1415** 全绿（part18 +2 单测：pending-id 回落三态 + remove 对冲不复活）；双矩阵 clippy 干净、fmt 无 diff。

## 遗留（下轮）

1. **dataChange 全量仍 >560s**：恒定 per-op 成本 **~7ms/append**（两次 host 往返 + `__zw_query_match` 每 call `parse_html` re-parse）——**这正是 M1 L2（polyfill-live 合一）的正题**：桥改读 live Document 消 re-parse 后三文件大概率自然跑完。R43 的 L2 诊断文档（evidence/2026-08-14-r43-l2-live-view-diagnosis.md）是直接设计输入；
2. **detached 容器子树残余缓涨**：setupRangeTests 的 foreignDoc/xmlDoc/docfrag 每次重建、旧容器整树丢弃但**从未 remove**（用例语义，真实浏览器靠 GC 回收）——我们的 pending 表持强引用。缓涨（rm 段 28→42ms/250）在 5000 subtest 量级仍可感。修法方向：WeakRef 化或「无外部引用检测」（需引擎 API 支持，标 TBD）；
3. mutations 语义面（Range 端点 identity）不变（R51b 遗留）。

## 教训（沉淀候选）

- **单 turn 批处理架构下的差量表必须有对冲/压实语义**——「add 无条件入账、remove 无条件入账」在 turn 内多轮 add/remove 的用例（testharness mega-case）下必爆炸；
- **诊断探针要放在被测函数的「效应出口」**（本次两次被入口 diag 的时序骗过：读到过滤前值）；
- **同 clone 双 rally 流**的 target/ 目录竞争（本轮 target/ 整目录消失一次，并行流清理）——再次验证 run-rules §8 独立 clone 的必要性。
