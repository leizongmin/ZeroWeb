# M4 切片 R54 — live collection els 泄漏「并入点过滤」修复

**日期**: 2026-08-15
**Commit**: `ba775545`
**上游诊断**: R53（evidence/2026-08-15-r53-dirty-state-live-collection-diagnosis.json——两版修复尝试均回归后干净回退，留下「并入点过滤而非移除点剔除」的精确方向）

## 问题（R53 已根因定位）

`setupRangeTests` 每 subtest 重建整树：旧 paras append 到 detached/foreign 容器（detachedDiv/foreignDoc/xmlDoc），从未 remove（用例语义，真实浏览器靠 GC）。旧 `_zwHCLiveInvalidate` 的 add 分支无条件把 `addFlat` 并入文档级集合 → `getElementsByTagName('p')` 的 els 每 setup 净 +2 → els 千级 → 失效循环每 mutation O(els) → 脏状态级联（干净 setup 1.5ms → 混入 data 写后 334ms/setup，220x）。

## 修复：挂载点判定（并入点过滤）

`_zwMutationInDoc(mutSel, mutHandle)`（part05.js）：

- **mutSel 非空** → `__zw_contains('html', mutSel)`（host 快照一查，R52 起有 parse 缓存；'html'/'body'/'head' 短路）
- **mutHandle** → 沿 `_zwNodeParent` 反链**逐跳**上行（每跳是 append 当时刚记账的链——与 R53 失败两版的区别：不从子节点上行，pending 树 sel 链断在未 apply 的容器上；挂载点链在记账时刻完整），遇 parentSel 走 sel 分支；无链（detached/foreign 容器根）→ false；guard 8 跳

两处应用：

1. `_zwHCLiveInvalidate` add 分支：`addFlat.length && _r54InDoc` 才并入
2. `_zwMakeCollection` 构建期 pending 并入：`_pnd.__zwHandle` 挂载链不在文档 → skip

**不动**（R53 教训）：els 快照基线、removed 剔除路径（remFlat 循环）、`_handleChildren`/`_zwNodeParent` registry（CE 断连传播仍依赖，R2994）。

## 验证

- **单测 +3**（part18）：detached 容器子树不进文档级集合 / 失效循环 add 分支过滤（R53 泄漏精确复现：5 轮 detached append 期间 len 恒 1，挂入后 6）/ 构建期 pending 并入过滤
- **计时 A/B**（同树 stash 对照）：deleteData **71s→35s（2.0x）**、insertData 31s→23s；appendChild/appendData/replaceChild/removeChild 持平
- **零回归**：nodes 3006P / collections 48P/0F / traversal 953P / events 189P 与基线逐项一致（name-validation 在全目录跑 diff 出 3F，隔离复跑两树完全一致——60s case 超时边界的计时抖动）
- engine v8 2143 / quickjs 1415 全绿；双矩阵 clippy 干净；fmt 无 diff；pre-commit-guard PASS

## 遗留

- insertBefore/dataChange 仍 >420s（两树相同）——R51c/R52 已归因的 per-subtest testFn 成本（Range getter 断言族 + per-op host 往返），**M1 L2 正题**
- detached 容器强引用残余缓涨（foreignDoc/xmlDoc 整树，TBD WeakRef / 无外部引用检测）
