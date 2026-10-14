---
date: 2026-10-14
modules: crates/engine
---

# js_dom_shim 同一树节点存在多族稳定 JS 包装——跨族 identity 恒败与吸附目标重投影陷阱

## 问题描述

selection-compat M2 切片 2（anchor-removal）排查中实证：同一 DOM 节点在
shim 内同时存在多族稳定 JS 对象，`===` 跨族恒为 false：

- **树行走/getElementById/querySelector 族**：`_makeProxy` 按
  `(sel, handle)` 缓存的 sel-proxy（带 `__zwSelector`/`__zwHandle` 键）。
- **named-prop 全局族**：R3K（innerHTML 解析 walk，part04）写入的
  `globalThis[id]` own property——解析 wrapper **就是**活树真实节点对象
  （shim 树节点是普通 JS 对象，proxy 才是包装）；WPT 页面脚本裸全局
  （`parentParagraph`）即此族。
- **纯文本包装族**：`_wrapNodeEntry` 每次访问新建（`__zwIsText`/
  `__zwChildIndex` own 键），不跨访问稳定。

后果：caret/选区端点保存树族对象后，与页面脚本持有的 named-prop 族引用
比较 identity 必败；getElementById 族与 bare global 也互不相同。

## 根因分析

两层独立缺陷叠加：

1. **R3K 世代滞留**：注册 guard 写 `globalThis[nid] === undefined`，同一
   页面第二次 `innerHTML` 重建树后，同名 global 仍是上一代死 wrapper——
   `.remove()` 等方法落在死节点上不触达活树，且删除断言（探针
   `docContains`、删除事件）全部静默失真。这是「家族 census 探针显示
   remove 无效」的真相：不是链路僵尸，是 own property 遮蔽。
2. **吸附目标重投影**：为跨族匹配引入 `_zwSelNAHome`（经
   `globalThis[id]` 取孪生重投影）后，若把重投影对象用作选区吸附的
   **目标**（而非仅比较），`globalThis[id]` 即便连通也可能与测试/脚本
   持有的查询 proxy 异族，换对象反而丢 identity（bridge test 场景②
   `focusNode === querySelector('#r')` 实证）。

## 解决方案

- **R3K 换代刷新**：注册值打 `__zwR3kNamedId` 标记；同名现值带同标记 =
  本机制上一代注册 → 覆盖刷新；脚本自有值（无标记）不遮蔽。
- **重投影只用于比较，不用于落值**：删除吸附的规则①目标保持
  `oldParent262` 原对象（与 live range 吸附同源，identity 天然一致——
  removed.parentNode 本就是解析族对象时与 bare global 同物）。
- `_zwSelNAHome` 加**连通性门**（`_zwDocContains36`）：历史代残留
  globalThis[id] 不接；nodeType + 文本 nodeValue 双校验防同名误配。

## 避免方式

- 涉及节点 identity 的断言/保存路径，先确认对象来源族（proxy 还是解析
  wrapper），不要假设「同一节点只有一个 JS 对象」。
- 新增 own-property 全局注册（`globalThis[id] = node`）时，guard 不能只
  查 `undefined`——重复解析同一文档是常态，需自带世代标记并可刷新。
- 跨族匹配用「双侧重投影到同一族再比」的 match-only 模式；凡是**写回**
  端点/容器的值，保持调用方路径原对象。
