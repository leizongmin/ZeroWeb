# R217 Evidence — insertNode 的 Document 子位置规则

**日期**: 2026-08-24
**切片**: M4——R217(a) 25,x DOM 比较层（R216 解 setup 层后的下一层）
**改动面**: `part06.js`（insertNode 校验的 Document 专属分支）+ `part21.rs`（回归单测）

## 一、根因

R215 校验四件缺 **「If parent is a Document」四分支**（spec
`dom-node-pre-insert` 的 Document 专属子位置规则）。25,0 探针实证：host 对
element 入已有 element 子（docEl）的 Document 不抛——sim（common.js
ensurePreInsertionValidity switch）返 HRE → "did not throw"。

## 二、实现（R215 校验块内新增 Document 分支）

- node 是 **DocumentFragment**：多 element 子 / Text 子 → HRE；单 element 子
  时——parent 已有 element 子 / 插入点是 doctype / 插入点后有 doctype → HRE
- node 是 **Element**：parent 已有 element 子 / 插入点是 doctype /
  插入点后有 doctype → HRE
- node 是 **DocumentType**：parent 已有 doctype 子 / 插入点前有 element 子 /
  无插入点但 parent 有 element 子 → HRE

## 三、验证链

- **单文件**：insertNode **902P→916P（+14）**；surround 865 / delete 56 /
  extract 98 / clone 149 全部不变（零扰动）
- **全量（polyfill）**：R216 基线 51576P/3463F/20T → **51589P/3449F/21T
  （净 +13P，P2F=0 纯增）**——F2P 14 全在 Range-insertNode
- **全量（native 对照）**：**51589P/3449F/21T 逐计数一致**——flips 仅 2
  既存 flaky（insertBefore-iframe-crash / EventListener-incumbent-global-subsubframe）
- **engine 单测**：2357 全绿（新增
  `test_insert_node_document_position_rules_r217`——element-vs-element /
  frag 多 element / frag Text / doctype-vs-doctype 四断言组）
- **fmt / clippy**：零 diff / 零警告
- **make test**：（见 master.md R217 行）

## 四、R218 靶点

1. 12,x Maximum call stack 72F（docEl 入树后 upwalk 已通——新溢出点定位）
2. insertNode 剩余 ~925F 重新聚类（HRE 残余 / is-not-a-function / null 族）

## 五、commit

（落盘时待填）
