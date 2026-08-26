# R292 Evidence — querySelector 结构元素身份归一（body/head/html 特例，matches 双套件 100%）

**日期**: 2026-08-27
**切片**: M4——R292(a) R291 归因收窄的靶簇公共根因修复（matches 2F + querySelector-All 域复核）
**改动面**: `part03.js`（detached-doc 的 `queryOne`/`queryAll` 结构元素归一 + docEl 属性反射面 in part05）+ `part23.rs`（+1 单测）

## 一、修复内容

### (a) 结构元素身份归一（`_r292StructNode`）

R291 定位的根因：iframe 子文档的 `doc.querySelector('#body'/'body')` 返
`_zwParseEl` wrapper（JSON 往返产物，probe 实证 `q === doc.body` false、
`parentNode` null）——wrapper 上 `.matches('body')` 无根上下文恒 false。

修：detached-doc 的 `queryOne`/`queryAll`（iframe 子文档共用此族）对结构
元素形态（`body`/`head`/`html` 的 tag 或 `#id` 形态）直返 **doc 视图对象**：
- 动态读 `doc.body`/`doc.head`/`doc.documentElement` 槽（R221 fresh-doc 的
  appendChild 重绑会换 body/head 节点——静态闭包引用会 stale）；
- 无动态槽时回落闭包 `body`/`headEl`；
- detHtml 包装层（R159 `<html><head/><body/></html>`）的命中即结构元素自身
  （HTML 文档结构元素唯一，无内容树同名歧义）。

### (b) docEl 属性反射最小面

归一后 `doc.querySelector('html')` 直返工厂 docEl——消费方
（`found.getAttribute('id')` 断言族）对无方法的工厂对象直接 TypeError
（首版回归 +2F 实证）。补 `getAttribute`/`hasAttribute`：惰性解析
`_r159HtmlAttrs` 串（markup 提取的 `<html attrs>`）。

## 二、验证

| 套件 | R291 | R292 | Δ |
|---|---|---|---|
| Element-matches | 674P/1F | **675P/0F（100%）** | +1 |
| Element-webkitMatchesSelector | 668P/1F | **669P/0F（100%）** | +1 |
| ParentNode-querySelector-All.html | 1971P/4F | 1971P/4F | 持平（tree-order 4F 是内容树 wrapper 域，另簇） |
| Element-closest/Node-contains/Document-createElement(-NS)/getElementById | 全绿 | 同 | 持平 |
| ParentNode-querySelector-escapes/scope/querySelector-mixed-case | 2F/2F/1F | 同 | 持平（pre-existing） |
| engine 单测 | 2429 | **2430** | +1（r292 单测） |

## 三、dom 全量（单跑）

| 域 | R291 | R292 | Δ |
|---|---|---|---|
| dom 全量 | 54047P/95F | **54045P/93F** | -2F（P ±2 flaky 波动） |
| dom/nodes | 12669P/51F | **12668P/49F** | -2（-1 flaky） |
| 其余四域 | 同 | 同 | 持平 |

set-diff：消失的恰为 Element-matches/webkitMatchesSelector 的 body
type selector 2F，**零新增失败**。

## 四、R293 靶点

- **(a) querySelector-All tree-order 4F**（内容树 wrapper 的 identity/
  树序——R292 已消结构元素歧义，剩余是内容元素 wrapper 域，R167 桥归一的
  覆盖缺口）。
- **(b) insertAdjacentText/insert-adjacent 2F**（定位域独立，小簇）。
- **(c) MutationObserver 8F**（childList Range 系 3F + document parser 3F +
  inner-outer 2F）。
- **(d) Text/Comment-constructor 跨 globals 2F**（iframe doc childNodes
  计数 1 vs 2——doctype 链入域）。
- **(e) variant 基建最小支持**（解锁 ranges/in-shadow 2F + events/
  handler-count；低优先备档）。
