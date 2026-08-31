# M4 切片 R47 — childList fragment 展开 record + sibling 字段 + remove record + surroundContents 顺序解耦

**日期**: 2026-08-15
**里程碑**: M4 / DC-3（nodes MutationObserver）
**证据**: [../evidence/2026-08-15-r47-mo-childlist-records.json](../evidence/2026-08-15-r47-mo-childlist-records.json)

## 修复四件

1. **fragment 展开 record**：appendChild/insertBefore/replaceChild 带 DocumentFragment 时 `addedNodes` = fragment **子节点**（flatten 前快照 = 既有 `ceAdded`）——fragment 自身不入树不出现在 record（旧三者均记 `[fragmentProxy]`）
2. **sibling 字段**：childList record 补 `previousSibling`/`nextSibling`——appendChild 写入前捕获容器 lastChild；insertBefore 捕获 refNode 前兄弟 + refNode；remove 捕获双兄弟
3. **`el.remove()` record**：补 childList removed record（旧完全缺失——surroundContents 的逐 removed record 依赖它）；新增 `_zwSuppressRemoveRecord` flag 供组合操作抑制逐次 notify
4. **surroundContents 顺序解耦**：record 须按**文档序**（WPT 期望 [removed-first, removed-last, added]），树操作须**逆序 remove**（正序破坏 nth-child selector 移错节点）——先按文档序快照兄弟，置 suppress flag，逆序 remove（树正确），再按文档序统一发 records

## 同轮回归两起（均 renderer quickjs R2930 测试捕获）

1. 正序 remove 直接改——nth-child 前移致 #sc 剩 2 子（R2930 注释的逆序理由是对的）→ 顺序解耦方案
2. 解耦初版引用 `_makeRange` 闭包外 `sel/handle`（QuickJS `sel is not defined`）→ startContainer 快照

## 结果

| 项 | 前 | 后 |
|----|-----|-----|
| MutationObserver-childList | 10P/4F | **15P/1F（+5）** |
| dom/nodes polyfill | 2547P | **2552P（+5）** |
| dom/nodes native | 2517P | 2522P |
| Element-classlist | 1420P/0F | 维持 |

零回归：events 189P / collections 24P / traversal 9P / ranges 39P / renderer quickjs 55P+126P（R2930 恢复）。

## 剩 1F 诊断

surroundContents 第 2 条 removed record 的 previousSibling：JS 侧 `previousSibling` 读 host 状态——首个 remove 尚未 apply（延迟批处理），host 树里 s1 还在 → 返 s1 而非 null。需移除的**同步兄弟链可见性**（M1 L2 同根因，R43 诊断同族）。

## 验证门禁

- 单测 `test_mutation_observer_childlist_fragment_r47`
- engine v8 2128 / quickjs 1415 全绿；quickjs 矩阵 14 crate 全绿（renderer 恢复）
- clippy 零警告，fmt 无 diff
