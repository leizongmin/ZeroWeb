---
date: 2026-11-05
modules: engine
---

# shim/DOM 桥 insertBefore 解析文本 ref 静默丢弃

## 问题

editing shim 值命令管线（`part01d.js` peel/wrap 系列）按参考实现把节点
搬移/插挂写成 `parent.insertBefore(node, ref)`。当 ref 是「解析来的文本
ref」（来自 `childNodes[idx]`、`nextSibling` 等直接解析产物，无 `__zwSelector`
选择器印、无 `__zwHandle` 章、非 textEl 注册文本）时，插入**静默 no-op**：
上游日志显示 insert 调用成功返回、无异常，但树里目标节点不出现
（实证链：`peel ins parent[...5 kids...] cloneIn=at2` → `peel done
parent[...4 kids...]`，克隆刚插进去就没了）。

## 根因

`part04.js` M 代理 insertBefore wire 分发（~:8866-9676）按 refNode 形状
分派：

1. `__zwSelector` 印（解析元素 ref——选择器印臂）✓
2. `__zwHandle` 章（stamped ref——章臂，触发后做 registry splice）✓
3. textEl 注册的文本 ref ✓
4. plain 元素 ref（`nodeType === 1`，D3 `_zwPosSelOf` 臂）✓
5. **解析文本 ref（无印无章非注册）——全分支 miss，静默 no-op** ✗

`refNode === null` 的 append 分支独立且正常。即：桥面 ref 可用性白名单
=「解析元素 / 章戳 / textEl 文本 / plain 元素」，解析文本 ref 不在其中，
且 miss 无任何日志或异常。

## 解决与避免

- 搬移 API 换 `_zwJusMovePreservingRanges`（内部同样走桥，但调用面
  已按其约束选位）；插挂 ref 必须选**解析元素或 null（append）**。
- 管线设计层面：凡「插到 X 之后」的形状都要检查 X 是否可能是文本节点
  （`baz` 等裸文本兄弟不可作 ref）。修法把插入点改为天然避开：克隆落
  目标元素**之前**（ref = 该元素），或改为免插入的裸搬移（子级直接
  `movePreserving` 上移到 parent）。
- 排查此类「树没变但无报错」时，先核 ref 形状再怀疑搬移语义：
  在 wire 分发处临时 log ref 的 `__zwSelector/__zwHandle/nodeType`
  三元组可一步定位。

## 关联

`crates/engine/src/js_dom_shim/part04.js`（wire 分发）、`part03.js`
（Node.prototype.insertBefore 派发）、`part01d.js` `_zwVaPeelOne`
（本例修复面）；案例详录
`docs/goal/editing-compat/evidence/2026-11-05-qv-peel-asym-recursion-break.md`。
