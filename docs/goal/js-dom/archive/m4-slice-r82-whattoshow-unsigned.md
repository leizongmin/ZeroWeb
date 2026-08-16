# M4 切片 R82 — NodeIterator/TreeWalker whatToShow 无符号 + referenceNode 同步（WPT 驱动）

**日期**: 2026-08-16
**Commit**: `8a8d458e`
**Evidence**: [../evidence/2026-08-16-r82-whattoshow-unsigned.json](../evidence/2026-08-16-r82-whattoshow-unsigned.json)

## 驱动用例簇

- `dom/traversal/NodeIterator.html` / `TreeWalker.html`：`.whatToShow` 152 subtest（0xFFFFFFFF 期望 4294967295 得 -1）+ `.pointerBeforeReferenceNode` 40 subtest + referenceNode/nextNode 序列族

## 三重根因

1. **whatToShow 符号位截断**：`| 0` 是 ToInt32 → 0xFFFFFFFF 读回 -1；spec WebIDL unsigned long 须 ToUint32（`>>> 0`）。位掩码行为旧实现侥幸正确（-1 补码全 1）。
2. **缺省/显式 null 混同**：`whatToShow == null → 0xFFFFFFFF` 把显式 null 也当 SHOW_ALL；spec：optional 参数**省略**才缺省，显式 null 走 ToUint32(null)=0——`arguments.length` 区分。
3. **同步 wrapper 被覆盖**（最大单簇根因）：referenceNode/pointerBeforeReferenceNode 的同步 wrapper 定义在 R51 lazy `walker.nextNode = function` 重赋值**之前**，被静默覆盖 → 步进后 reference/pointer 恒不更新。

## 修复

- `>>> 0` + `arguments.length` 区分省略/显式 null
- wrapper 移到工厂尾部（全部 lazy 定义之后统一包装）
- 语义：nextNode 命中 → reference=node + before=false；**耗尽返 null 不动**（首版「到尾也翻 false」被全量跑出 597 fail / -306P 否决——WPT 期望「立即耗尽的迭代器 before 保持 true」）

## 验证

| 项 | 结果 |
|----|------|
| dom/traversal | 1195P → **1486P（+291 净）**，94F 剩余 |
| dom/nodes / events / collections | 6596P / 189P / 48P 不变（零回归） |
| 单测 | part18 +1（`r82_whattoshow_unsigned_and_pointer_semantics`）；engine v8 **2170** / quickjs **1427** 全绿 |
| fmt / clippy | 无 diff / 双矩阵零警告 |

## 剩余（traversal 94F）

- NodeIterator `nextNode() expected DocumentType/DocumentFragment but got null`（doctype/docfrag mask=0 不展示——maskFor 未覆盖 nodeType 10/11）
- TreeWalker previousSibling `__n6` 族（handle 子树兄弟导航边角）
- cross-realm / removal-during-filtering 深结构
