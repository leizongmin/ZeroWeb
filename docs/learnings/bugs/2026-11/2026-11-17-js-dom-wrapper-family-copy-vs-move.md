---
date: 2026-11-17
modules: zero-engine
---

# JS DOM 桥 wrapper 家族 appendChild 语义二分：序列化拷贝 vs 真移动——摘原须按 childNodes indexOf 判位

## 问题描述

insertimage 出壳刀（IMG-BREAKOUT）mid-shell 双 B 钉红：`<div><b>A[]B</b></div>`
caret 插图期望 `<div><b>A</b><img><b>B</b></div>`，实测
`<div><b>AB</b><img><b>B</b></div>`——尾半 `B` 滞留原 b 壳，右半壳「分裂」
形在宿主视图缺失。首版实现按 `parentNode` 判断「原节点是否仍在源位」，该
判断在拷贝态恒误判「已不在」，摘原被跳过。

## 根因分析

同一 DOM 节点在 js_dom_shim 内有多个 wrapper 家族，`appendChild` 语义不同：

- **plain 家族**（`document.createElement` 产物壳）：`appendChild` 是
  **序列化拷贝**——副本经序列化/重解析进入目标壳 childNodes，原节点滞留
  源 childNodes，而原节点 `parentNode` 已指向新壳（簿记不一致态）。
- **innerHTML 镜像元素**：`appendChild` 是**真移动**——源 childNodes
  清空，节点身份随迁。

依赖 `parentNode`（或身份已在目标内）推断「源位已空」的摘除逻辑，在拷贝
态失效：`parentNode` 已切壳但原节点仍在源 childNodes，双 B 残留即此。

连带发现 splitText 家族差：part03 `_zwAttachCharacterDataMethods` 只给 M
家族 Text 节点挂 `splitText`（data 截断＋尾插入均发 mutation，宿主 getter
视图全同步）；root-text 直子等裸族无此方法，JS-local 回落
`_zwVaSplitText` 只改 JS 树——宿主侧后续按索引寻址的 mutation
（RemoveChildAt 等）会失配视图，除非后续有宿主直子级插入触发的子树重
序列化承载。

## 解决方案

- 摘原按 `childNodes` 的 `indexOf` 位置判定：在位才 `removeChild`。拷贝
  态在位即摘、真移动态缺席自然跳过，两种语义统一，不依赖 parentNode。
- 文本拆分优先节点自带 `splitText`（M 家族全同步发射），仅无该法时回落
  JS-local 拆分，并确认后续存在宿主重序列化承载点（宿主直子级插入）。
- 通用规则：在 JS 桥内对共享节点做移动/摘除前，先确认其 wrapper 家族及
  对应 `appendChild`/`removeChild` 语义（拷贝 vs 移动）；判「在位」用
  childNodes 索引，不用 parentNode 或目标内身份。getter 视图（如
  `innerHTML`）读的是 JS 侧 pending-mutation 重构面，与 JS 树分叉时以
  发射 mutation 对齐宿主为准。
