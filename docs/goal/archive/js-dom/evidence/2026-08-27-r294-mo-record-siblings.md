# R294 Evidence — MutationObserver record 兄弟字段 + addedNodes identity（dom 全量 -2F）

**日期**: 2026-08-27
**切片**: M4——R294(a) MutationObserver 8F 按簇推进（childList Range 系 -1F + inner-outer -1F）
**改动面**: `part04.js`（removeChild 的 handle-子 record 兄弟捕获 + innerHTML 纯文本 addedNodes 的 textEl identity）+ `part23.rs`（+1 单测）

## 一、修复内容（两件 land + 一件回退）

### (a) removeChild 的 handle-子 record 兄弟字段（childList Range.deleteContents -1F）

spec MutationRecord 的 previousSibling/nextSibling——旧 record（part04 removeChild
的 `child.__zwHandle` 分支）缺两字段恒 null。WPT MutationObserver-childList
"Range.deleteContents: child and data removal" 断言 `previousSibling ===
n71.firstChild`（"CHAN" text）。修：移除前从父融合 childNodes 定位
（identity 优先 + handle/data 内容键回退），在 registry 剔除**前**读。

### (b) innerHTML 纯文本 addedNodes 的 textEl identity（inner-outer -1F）

旧 record 用 `_zwFragmentAdded` 的独立 wrapper（data 相同 identity 不同——
"expected Text node got Text node"）。修：纯文本形态 addedNodes 用注册表同一
textEl 节点（`_zwTextElsByHandle/BySel` 取——消费方 `el.firstChild` 读同一注册表
节点，identity 对齐）。

### (c) 试加 R129 sel-parent record 后回退（教训）

为 extractContents 形态试在 R129 的 sel-父分支补 record——r259 单测当场回归
（bodyLast 'leaf' vs '#text'）：`_mo_notify` 的 `_zwNodeParent` 清链副作用改变
surroundContents HRE 路径的回滚依赖。**回退**该件（记录教训：R129 分支的
无-record 行为是某些 HRE 回滚路径的隐式依赖，改它须先理 leaf-HRE 的
append/rollback 域）。extractContents 形态（record 的真实来源在别处——
rmNode remove→removeChild 之外的路径）记 R295 靶点。

## 二、验证

| 套件 | R293 | R294 | Δ |
|---|---|---|---|
| MutationObserver-childList | 22P/3F | **23P/2F** | -1F（deleteContents 形态） |
| MutationObserver-inner-outer | 0P/2F | **1P/1F** | -1F（innerHTML mutation） |
| MutationObserver-document | 1P/3F | 同 | 持平（parser 域另簇） |
| MO-takeRecords/attributes/characterData | 全绿 | 同 | 持平 |
| Range-delete/extract/surround + Node-removeChild | 129/192/1840/28P 全 0F | 同 | 持平（删除流回归 sweep） |
| engine 单测 | 2431 | **2432** | +1（r294 单测） |

## 三、dom 全量（单跑）

| 域 | R293 | R294 | Δ |
|---|---|---|---|
| dom 全量 | 54066P/91F | **54070P/89F** | +4 / **-2** |
| dom/nodes | 12688P/47F | **12692P/45F** | +4/-2 |
| 其余四域 | 同 | 同 | 持平 |

set-diff：消失的恰为两 MO subtest，**零新增失败**。

## 四、R295 靶点

- **(a) MO 剩余 6F**：extractContents 形态 1F（record 真实来源定位——
  rmNode 之外的路径）+ surroundContents 形态 1F（"#s1" previousSibling 反向）
  + inner-outer "2 children" 1F（wrapper identity 域）+ document parser 3F。
- **(b) querySelector-All tree-order 4F**（内容树 wrapper identity）。
- **(c) Text/Comment-constructor 跨 globals 2F**（doctype 链入域）。
- **(d) Node-insertBefore 1F**（host ref 校验域）。
- **(e) variant 基建最小支持**（低优先备档）。
