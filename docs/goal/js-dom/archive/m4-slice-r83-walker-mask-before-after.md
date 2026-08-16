# M4 切片 R83 — walker 全 nodeType 掩码 + fresh 起点语义 + handle before/after（WPT 驱动）

**日期**: 2026-08-16
**Commit**: `d38a9081`
**Evidence**: [../evidence/2026-08-16-r83-walker-mask-before-after.json](../evidence/2026-08-16-r83-walker-mask-before-after.json)

## 驱动用例簇

- `dom/nodes/ChildNode-before.html` / `ChildNode-after.html`（84F——parent=createElement('div') handle 容器）
- `dom/traversal/NodeIterator.html`（doctype/docfrag 掩码 + previousNode 逆向 + undefined 形态）
- `dom/traversal/TreeWalker-acceptNode-filter.html`（"this value and node argument"——首 nextNode 的 filter 收 A1 非 root）

## 六重根因与修复

1. **maskFor 全 nodeType**：`(1 << (nt-1)) >>> 0` 覆盖 1..13（doctype/fragment/PI/document/CDATA 不再被掩掉）。
2. **TreeWalker/NodeIterator fresh 起点区分**：walker 的 currentNode=root 已位于 root → 首步越过；iterator 集合含 root → 首步返 root。R2803 两条旧断言按 spec 纠正。
3. **previousNode 重写**：结构序 lazy 逆向步进（与 nextNode 对称）——旧 accepted-index 模型对滤掉的 currentNode 错误回落「从尾」。
4. **WebIDL optional 三态**：undefined（或省略）→ SHOW_ALL；显式 null → 0。
5. **handle before/after**：JS 侧父 `_handleChildren` splice 插入 + 反链 + childList notify（host 无 by-handle 兄弟 mutation；sel 路径不变）。
6. **handle innerHTML 融合序列化**：appendChild 子树可见（text 转义/comment/元素 outerHTML），无子回落 host。

## 验证

| 项 | 结果 |
|----|------|
| dom/nodes | 6596P → **6635P（+39 净）** |
| dom/traversal | 1486P → **1527P（+41 净）** |
| dom/events / collections | 189P / 48P 不变（零回归） |
| 单测 | part18 +2；engine v8 **2172** / quickjs **1427** 全绿 |
| fmt / clippy | 无 diff / 双矩阵零警告 |

## 剩余（traversal 53F）

- NodeIterator `nextNode() N time(s) expected null` 超步族（document 树节点计数与 oracle 差一——doctype 计入/排除歧义待深查）
- TreeWalker previousSibling `__n6` 族（handle 子树兄弟导航边角）
- cross-realm / removal-during-filtering 深结构
