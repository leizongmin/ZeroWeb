# R293 Evidence — insertAdjacent 家族全解（Element-insertAdjacentText + insert-adjacent 双文件 100%，dom 全量 +21P/-2F）

**日期**: 2026-08-27
**切片**: M4——R293(b) insertAdjacentText/insert-adjacent 2F（两文件整文件崩→全 100%）
**改动面**: `part04.js`（insertAdjacentText spec 语义重写 + insertAdjacentElement 参数校验/父记账修正）+ `part03.js`（Element.prototype 的 insertAdjacentElement/Text plain 链方法）+ `part05.js`（工厂 docEl 的双方法）+ `part23.rs`（+1 单测）

## 一、修复内容（四层）

### (a) insertAdjacentText spec 语义（proxy 版，part04）

旧版只推 host mutation（异步 apply）——无 position 校验、无 HRE、同轮断言
（`previousSibling.nodeValue`/`firstChild`）读快照恒旧值。重写对齐 R133 的
insertAdjacentElement 收口：
1. position 四值（ASCII case-insensitive）非法同步抛 SyntaxError（WPT
   "Inserting to an invalid location"——旧 host 报错异步不可达使整文件崩）；
2. documentElement 的 beforebegin/afterend 抛 HierarchyRequestError（WPT
   "Adding more than one child to document"）；
3. **同轮可见性**：host mutation 后经 proxy 自身的 insertBefore/appendChild
   做同步子视图（pending overlay 记账 + handle 反链，R182 管线）+ 兄弟缓存
   失效（`_zwSiblingBaseInvalidateAll`——sibling getter 按快照代缓存）。

### (b) Element.prototype 的 insertAdjacentElement/Text（plain 链，part03）

createHTMLDocument().documentElement 走 `_zwMEl` plain 链——旧缺方法直接
TypeError（WPT insert-adjacent 的 "invalid caller object" 双形态）。补
generic 版：position 校验 + doc 根 HRE + childNodes 就位（plain 数组语义）。

### (c) 工厂 docEl 双方法（part05）

mEl 提取路径的 docEl（iframe 子文档）同款补齐（R292 docEl 属性反射面的
后续——结构元素归一使工厂对象直接可达消费方）。

### (d) insertAdjacentElement 参数校验收紧 + 父记账修正（part04）

1. 参数须 **Element**（nodeType === 1）——旧只查 `nodeType !== undefined` 使
   DocumentType 落 sel 分支推 host mutation → apply 报 "no child match for
   undefined" **整文件崩**（WPT "invalid object argument" 期望 TypeError）。
2. handle 子路径的 pending bucket 按**实际插入父**记账——beforebegin/
   afterend 的子落在 target 的父（旧按 target sel 记账使 overlay 不可见 →
   同轮 `el.previousSibling.id` null）。nextSibling 定位 + `_zwNodeParent`
   反链先行 + 兄弟缓存失效。

## 二、验证

| 套件 | R292 | R293 | Δ |
|---|---|---|---|
| Element-insertAdjacentText | 整文件崩 | **6P/0F（100%）** | +6 |
| insert-adjacent.html | 整文件崩 | **14P/0F（100%）** | +14 |
| Element-insertAdjacentElement | 6P/0F | 同 | 持平 |
| Range-insertNode / MutationObserver-childList | 1841P/0F / 22P+3F | 同 | 持平（MO 3F 预存） |
| engine 单测 | 2430 | **2431** | +1（r293 单测） |

## 三、dom 全量（单跑）

| 域 | R292 | R293 | Δ |
|---|---|---|---|
| dom 全量 | 54045P/93F | **54066P/91F** | **+21 / -2** |
| dom/nodes | 12668P/49F | **12688P/47F** | +20/-2 |
| 其余四域 | 同 | 同 | 持平 |

set-diff：消失的恰为两文件级失败，**零新增失败**。

## 四、R294 靶点

- **(a) MutationObserver 8F**（childList Range 系 3F + document parser 3F +
  inner-outer 2F）。
- **(b) querySelector-All tree-order 4F**（内容树 wrapper identity——R167 桥
  归一覆盖缺口）。
- **(c) Text/Comment-constructor 跨 globals 2F**（iframe doc childNodes 计数
  1 vs 2——doctype 链入域）。
- **(d) Node-insertBefore 1F**（apply 报「节点不是子节点」——host insert_before
  ref 校验域）。
- **(e) variant 基建最小支持**（解锁 ranges/in-shadow 2F + events/
  handler-count；低优先备档）。
