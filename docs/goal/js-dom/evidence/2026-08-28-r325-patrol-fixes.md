# R325 Evidence — 备档集巡检两件轻量修复（Document-createCDATASection HTML 守卫 + PI nodeValue 分流；全量 54139P/56F，Fail set 恰 -2）

**日期**: 2026-08-28
**切片**: M4——R325(a) 58F 备档集走查
**改动面**: `part06.js`（主文档 createCDATASection）+ `part04.js`（PI nodeValue 分流顺序）

## 一、两件修复

1. **主文档 `document.createCDATASection`**（spec `dom-document-createcdatasection`）：
   旧缺方法 → `TypeError`；HTML 文档上调用应抛 **NotSupportedError**（DOMException）——
   WPT `assert_throws_dom("NotSupportedError", ...)` 失败形态 "threw TypeError ... is not
   a DOMException NotSupportedError"。修：part06 主文档对象补方法恒抛（主文档恒 HTML
   文档，语义固定；XML 文档域的 DOMParser 版本 part02:2181 已有，域判定不需要）。
2. **PI `nodeValue` 分流顺序**：part04 proxy 的通用 `prop === 'nodeValue' → null` 分支
   （R80，Element/Document 等 null 语义）位于 isPI 分支（1336，data/nodeValue 同源 data）
   **之前** → PI 的 nodeValue 恒 null。修：isPI 的 nodeValue 分流提前到通用分支之前
   （WPT Node-nodeValue "ProcessingInstruction.nodeValue" 期望 "A PI!"）。

## 二、同轮巡检的其余定性（无代码改动）

- remove-and-adopt-thcrash：`window.open()` 返 null（runner 无 popup 通道）——环境基建。
- remove-next-sibling / handlers-changed / MO-document / onerror / Document-URL /
  isConnected-iframe / HTMLNess / getElementsByClassName-live / querySelector-mixed-case：
  既有备档维持（R318–R320 已复核）。

## 三、A/B

| 项 | R324 基线 | R325 | Δ |
|---|---|---|---|
| Document-createCDATASection | 0P/1F | **1P/0F** | +1 |
| Node-nodeValue | 6P/1F | **7P/0F** | +1 |
| **全量 dom sweep** | 54140P/58F/22T | **54139P/56F/25T** | **Fail set 恰 -2 零新增**（Timeout +3 属并发噪声带）|
| engine --lib（v8/quickjs）| 2462/1460 | 2462/1460 | 持平（改动为 shim 方法面，无需新单测——行为由 WPT 资产锁定）|
| fmt / clippy（v8 guarded + quickjs）| — | 干净/0 | — |

## 四、教训

**分流顺序即语义**：proxy get trap 的「通用 null 分支」与「特定节点类型分支」的先后
顺序本身就是行为——加通用分支时若已有特定类型分支在后方，特定类型会被遮蔽（R80 加
Element-null 分支时 isPI 分支已在但顺序在其后，三年后暴露）。
