# R216 Evidence — iframe doc 的 docEl 入 doc 树（25,x setup 前置解锁）

**日期**: 2026-08-24
**切片**: M4——R216(a) insertNode 残余聚类（902P 基线：Maximum call stack 72F / null-nodeType 66F / HRE 94F）的 25,x 结构根因
**改动面**: `part05.js`（docEl 入 doc.childNodes + parentNode 链）+ `part21.rs`（回归单测）

## 一、根因（探针）

- **25,x**（`[document, 0, document, 1]` + 元素 node）：setup 阶段
  `setEnd(document, 1)` 抛 IndexSizeError——iframe doc 的 `childNodes` 为空
  （docEl 从未入列）。spec：documentElement 是 Document 的子。
- **12,x**（docEl-rooted range）：upwalk 链断在 docEl（parentNode null——
  furthestAncestor 得 docEl 而非 doc，根比较错位）。

## 二、实现

docEl 入 doc 树（`docEl.parentNode = doc` + `doc.childNodes.push(docEl)` +
`doc.children` 补）。**doctype 入 childNodes 首位评估回退**（实测 -55——
restoreIframe 清理循环节奏 + referenceDoc 语义扰动面比 docEl 单独入树更广，
保持 R209 的 getter-only，`[document,0,N]` 边界经 docEl 单子近似）。

## 三、验证链

- **单文件**：insertNode 902P 计数持平（25,x 的 setup IndexSizeError 消除——
  DOM 比较层的下一缺口暴露为形态重分布）；surround 865 / delete 56 /
  extract 98 / clone 149 全部不变（docEl 入树零扰动）
- **全量（polyfill）**：R215 基线 51575P/3463F/21T → **51576P/3463F/20T**
  （净 +1P——中性但结构正确：F2P 1（TreeWalker-realm）/ P2F 1
  （ParentNode-querySelector-All，既存 flake 家族））
- **全量（native 对照）**：**51575P/3463F/21T**——flips 仅 1 既存 flaky
  （replaceWith-document-element-crash）
- **engine 单测**：2355 全绿（新增 `test_iframe_docelement_in_doctree_r216`
  ——docEl.parentNode / childNodes 成员 / doc-rooted range setup 三断言组）
- **fmt / clippy**：零 diff / 零警告
- **make test**：（见 master.md R216 行）

## 四、R217 靶点

1. 25,x DOM 比较层（insertNode into Document 的子位置规则——element 与既有
   element 子 / doctype 位序；R216 只解了 setup 层）
2. 12,x Maximum call stack 72F（docEl 入树后 chain 已通——定位新的溢出点）
3. insertNode HRE 94F 残余 + null-nodeType 残余

## 五、commit

（落盘时待填）
