---
date: 2026-08-12
modules: crates/engine/src/js_dom_shim/part05.js（_childNodeList）, part04.js（childNodes get-trap）, part02.js（__zw_child_nodes host 回调）
---

# headless handle-only 元素 childNodes 读取限制

- 相关切片：R3316 `slot.assignedNodes/assignedElements` 调查（未 land）

## 问题描述

实现 `HTMLSlotElement.assignedNodes()` 时发现：在 headless JS 沙箱里，**handle-only 元素（`document.createElement` 创建）的 `childNodes` 读取恒为空**，且**把 handle 元素 appendChild 到普通父元素后，父的 `childNodes` 也读不到这些 handle 子**。这阻塞了所有依赖「读普通元素动态子节点」的 API（如 slot.assignedNodes 读 host light children）。

## 根因分析

headless DOM 桥接有两套子节点读取路径，但都覆盖不到「handle 子挂普通元素」场景：

1. **`_childNodeList(sel, handle)`**（part05.js:1484）：
   ```js
   if (!sel || typeof __zw_child_nodes !== 'function') return [];
   var arr = JSON.parse(__zw_child_nodes(sel) || '[]');
   ```
   - handle-only 元素（createElement 产物）无 `sel` → 行 1485 直接 `return []`。
   - 普通元素（parsed DOM，有 sel）→ `__zw_child_nodes(sel)` 读 **parsed DOM 树**的子节点；但动态 appendChild 的 handle 子**不在 parsed DOM 树里**（它们经 mutation 队列，未回写 parsed 树），故读不到。

2. **`_handleChildren` registry**（part04.js，R2927）：仅在 `appendChild` 时为**容器**（shadow root / DocumentFragment handle，`_shadowHandles`/`_fragmentHandles` 标记）记录子节点 handle 列表。**普通元素不是容器**，故 handle 子挂普通父时不进 registry。

综上：handle 子挂 shadow/fragment 容器 → 可经 `_handleChildren` 读；handle 子挂普通元素（含 createElement host、parsed 元素）→ **两套路径都读不到**。

## 影响

- `slot.assignedNodes()`：slot 经 parent 找到 shadow root → host（`_shadowHandleMeta`）正确；但读 host 的 light children（`_childNodeList(hostSel, hostHandle)`）→ host 是普通元素 → 返 `[]` → assignedNodes 恒空。**slot 分配语义不可观察，API 无法可靠实现**。
- 任何「JS 动态构建子树后读普通父的 childNodes」的场景都有此限制（但多数 API 通过 `_handleChildren` 容器路径或直接持节点引用绕过）。

## 解决方案（当前：记录，未修）

闭合 assignedNodes 需先解决「普通父的 handle 子可读」。可选路径（均跨层、非 trivial）：

1. **`_handleChildren` registry 扩展到普通元素**：appendChild 时无论父是否容器都记录子 handle（双向：父→子列表）。代价：所有 appendChild 都写 registry（内存 + 一致性维护），且需与 `__zw_child_nodes(sel)` 的 parsed 树读取合并（双源去重/优先级）。
2. **mutation 应用回写 parsed DOM 树**：appendChild 的 handle 子同步插入 host 的 sel 对应的 DOM 节点（query DOM 树持久化）。代价：headless 当前 mutation 是队列式（apply_dom_mutations 批量应用，非实时回写树），改架构风险高。
3. **assignedNodes 限定 host 为容器**：spec 上 host 是普通元素，此方案非 spec 合规。

当前结论：assignedNodes 暴露会误导（对真实场景不工作），故 R3316 不 land 实现。slot 分配的真实闭合依赖路径 1 或 2（架构级，需用户点名或独立切片评估）。

## 如何避免

- 实现「读普通元素动态子节点」的 API 前，先核查 `_childNodeList`/`_handleChildren` 对目标父元素类型的覆盖。若父是普通元素且子是 handle 创建 → 当前架构下不可靠。
- 优先用 `_handleChildren` 容器路径（shadow/fragment）或让调用方直接持节点引用（appendChild 返回值 / createElement 变量），避免依赖 childNodes 反查。
- 涉及 slot/Web Components 分配的 API，确认 host light DOM 子节点可读再实现，否则记录架构 gap 不强行 land。
