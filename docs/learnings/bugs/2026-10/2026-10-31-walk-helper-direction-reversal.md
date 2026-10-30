---
date: 2026-10-31
modules: js_dom_shim
---

# walk 辅助函数方向语义反转：skipWs 被反向调用臂复用后删错节点

## 问题描述

editing join 语料 paragraphs 面 forwarddelete/select-boundary 各 316F：
`<div normal>abc[]</div>` + 换行空白 + `<div nowrap>def</div>` 上 Delete，
引擎侧删掉的是 **caret 侧 "abc"**（actual `"\n<div nowrap>def</div>"`——
左块内容消失），而非消费右块。JS 侧 join 形状单独复算全部正确，
失败只在语料管线里出现。

## 根因分析

`_zwCeSkipWs(n)` 跳过空白文本节点时恒走 `previousSibling`——这是为
backspace 前邻臂写的方向。后来三个**后邻**查找臂（forwarddelete 块界/
forwarddelete select/backspace select）直接复用了它，传入 `x.nextSibling`
却仍向后走：`</div>\n<div>` 几何下从空白文本继续 backward 回走命中
**前邻**（`skipWs(左块.next = "\n")` 返回左块自己）→ 后邻查找返回
caret 侧块 → join 把 caret 侧内容当「待并入首行」提取移除。

三个放大器：

- **无空白几何不受影响**：语料另一文件（lrlp）块间为 JS 串接直邻
  （无文本节点），`skipWs` 原样返回元素节点——同函数同臂在该文件全对，
  散布的 pass 把嫌疑引向引擎寻址层。
- **失败形状指向下游**：引擎 wire 上唯一的 op 是 `remove_child_at_path
  [0,0]` 删 "abc"——寻址精确、语义错误，看起来像 stale-path 错位，
  实际是上游就把错误的 startNode 递给了正确的寻址管线。
- **单测不可见**：join 形状 Node 最小 DOM 复算用的是**正确调用方式**，
  复算恒绿；只有走完整 caret→walk→join 管线才触发。

## 解决方案

方向参数化：`_zwCeSkipWs(n, back)`，`back ? previousSibling :
nextSibling`；前邻臂传 `true`，后邻臂走默认 forward。规则：**带方向的
walk 辅助函数被反方向调用臂复用时，方向必须参数化**，不靠调用方传对
起点兄弟隐式保证。

定位方法（可复用）：对「删错节点」类失败，给 wire 回调与 apply 臂加
`ZW_WIRE_TRACE` eprintln 看唯一 op 命中了谁，再经 JS 侧阶段通道（env
不可读，注册 `__zw_wire_trace_note` 回调 + typeof 探测门控）打印
walk 入参——一轮 trace 即把「寻址错」与「入参错」分开；本例入参
`sn=caret 侧块` 直接暴露根因。
