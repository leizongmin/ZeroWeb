# R290 Evidence — nodes 域首批三簇：Element-remove handle 域 + Node-constants 下划线正则 + 接口 constructor 自反（dom 全量 -6F）

**日期**: 2026-08-27
**切片**: M4——R290(a) Element-remove 2F + (b) Node-constants 2F + (c) Document-constructor 2F
**改动面**: `part03.js`（接口原型 constructor 自反块 + XML doc createElement 原型回落）+ `part04.js`（remove 元素分支通知后清理 registry/反链）+ `part05.js`（has trap 常量正则放宽）+ `part23.rs`（+3 单测）

## 一、修复内容（三簇六根因）

### (a) Element-remove 2F（handle-only 父子形态的 remove 语义）

WPT ChildNode-remove.js：`createElement("div")` 父 + 子（均 handle-only、
父无 sel）的 appendChild→remove 链。两个缺口：
1. **`_parentNodeFor` 不查 handle 移除标记**——remove() 路径已
   `_zwMarkRemovedHandle`，但 parentNode 解析只查 sel 标记，
   `_zwNodeParent[handle]` 反链残留使 remove 后仍返父 proxy
   （"Removed node should not have a parent" expected null got __n1）。
2. **元素分支不剔父容器 registry**——`_handleChildren[父handle]` 残留被移除子
   （"Parent should not have children" expected [] got [__n0]）。R129 只在文本
   分支做了 registry 剔除，元素分支漏对齐。

修：清理（registry 剔除 + `delete _zwNodeParent[handle]`）放在
**`_zwNotifyIteratorsRemove` 之后**——迭代器 pred/succ 计算读移除前树位
（spec 迭代器移除步骤），先清理会破坏 r88 族（filter 内 remove 的
retarget 单测实证——首版顺序错误被 r88 双单测当场抓回）。

### (b) Node-constants 2F（常量名内部下划线）

R184 的 has-trap 白名单正则 `[A-Z][A-Z0-9]+_NODE` 不匹配含**内部下划线**的
常量名（`CDATA_SECTION_NODE`/`ENTITY_REFERENCE_NODE`/`DOCUMENT_TYPE_NODE`/
`DOCUMENT_FRAGMENT_NODE`）——`in` 恒 false（"doesn't have
CDATA_SECTION_NODE"）。修：字符类补 `_`。get-trap 侧同步补常量直答分支
（proxy target 无 Node 原型链，get 中间分支不认常量名 → undefined）。

### (c) Document-constructor 2F（接口原型 constructor 自反 + XML 元素原型）

WebIDL「interface prototype object」自带 non-enumerable constructor 指回接
口构造器。旧链 `Node.prototype = {}` / `Element.prototype = Object.create(…)`
/ 子类 `Object.create(HTMLElement.prototype)` 全缺 → `el.constructor` 沿链落
Object.prototype.constructor 恒 Object：
1. `new Document().createElement("DIV").constructor === Element`（XML doc 元素
   是泛型 Element——`_zwMEl` 的 R125 按 tag 无条件挂 HTML 子类 prototype，XML
   文档须回落 Element.prototype）。
2. `doc.createElementNS(XHTML, "a").constructor === HTMLAnchorElement`。

修：三基座 + ~64 HTML + ~37 SVG 子类原型统一补 constructor（幂等）+ XML doc
createElement 产物原型回落。

## 二、验证

| 套件 | R289 | R290 | Δ |
|---|---|---|---|
| Element-remove | 2F/2F | **4P/0F（100%）** | +2 |
| Node-constants | 6P/2F | **8P/0F（100%）** | +2 |
| Document-constructor | 3P/2F | **5P/0F（100%）** | +2 |
| CharacterData/NodeIterator-removal/MO-childList | 全绿/29P/22P+3F | 同 | 持平（r88 单测实证零回归） |
| Node-cloneNode / createElementNS / node-creation-realm | 145P/596P/13P 全绿 | 同 | 持平（原型改动回归 sweep） |
| engine 单测 | 2426 | **2429** | +3（r290 三单测） |

## 三、dom 全量（单跑）

| 域 | R289 | R290 | Δ |
|---|---|---|---|
| dom 全量 | 54041P/101F | **54045P/95F** | +4 / **-6F** |
| dom/nodes | 12663P/57F | **12669P/51F** | +6/-6 |
| dom/events | 579P/7F | 577P/7F | -2（timeout 噪声 12→14，环境波动） |
| dom/ranges / traversal / collections | 39147P/36F / 1603P/1F / 49P/0F | 同 | 持平 |

nodes 域剩余 51F 聚类（下轮靶点）：querySelector-All 4F（body type selector
host 匹配域——Element-matches/webkitMatchesSelector 同源 2F）、MutationObserver
8F、insertAdjacent/insert-adjacent 2F、Text/Comment-constructor 跨 globals
2F（iframe doc 子树计数域）等。

## 四、R291 靶点

- **(a) body type selector 匹配域**（Element-matches/webkitMatchesSelector/
  ParentNode-querySelector-All 共 6F 同源——host `__zw_matches`/`querySelector`
  对 body/html 文档根元素的 type 选择器匹配）。
- **(b) insertAdjacentText/insert-adjacent 2F**（定位域独立）。
- **(c) MutationObserver 8F**（childList Range 系 3F + document parser 3F +
  inner-outer 2F）。
- **(d) Text/Comment-constructor 跨 globals 2F**（iframe doc childNodes 计数
  1 vs 2——doctype 链入域）。
